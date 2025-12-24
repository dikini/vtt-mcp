### /home/dikini/Projects/vtt-mcp/crates/vtt-mcp/src/server.rs
```rust
1: //! MCP Server implementation for Voice-to-Text functionality
2: 
3: use crate::error::{VttError, VttResult};
4: use chrono::{DateTime, Utc};
5: use rmcp::{
6:     handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
7:     model::{ServerInfo, CallToolResult, Content, ErrorData as McpError},
8:     service::{RequestContext, RoleServer},
9:     model::{
10:         ServerCapabilities, ResourcesCapability,
11:         PaginatedRequestParam, ListResourcesResult, ListResourceTemplatesResult,
12:         ReadResourceRequestParam, ReadResourceResult, ResourceContents,
13:         SubscribeRequestParam, UnsubscribeRequestParam,
14:         Resource, RawResource, Annotated,
15:     },
16:     tool, tool_router,
17: };
18: use schemars::JsonSchema;
19: use serde::{Deserialize, Serialize};
20: use std::collections::HashMap;
21: use std::sync::Arc;
22: use tokio::sync::{Mutex, broadcast};
23: use uuid::Uuid;
24: use std::fs::{File, OpenOptions};
25: use std::io::{BufReader, BufWriter, Write};
26: use std::path::{Path, PathBuf};
27: 
28: use vtt_core::audio::{AudioCapture, list_devices};
29: use vtt_core::whisper::{WhisperContext, WhisperConfig, Transcription};
30: use vtt_core::whisper::language::{Language, SUPPORTED_LANGUAGES, supported_codes, display_name};
31: 
32: /// Transcription update broadcast to subscribers
33: #[derive(Debug, Clone, Serialize)]
34: pub struct TranscriptionUpdate {
35:     pub session_id: Uuid,
36:     pub text: String,
37:     pub is_final: bool,
38:     pub timestamp: DateTime<Utc>,
39:     pub confidence: Option<f32>,
40: }
41: 
42: /// A subscriber to a session's transcription stream
43: #[derive(Debug, Clone)]
44: pub struct SessionSubscriber {
45:     pub client_id: String,
46:     pub subscribed_at: DateTime<Utc>,
47: }
48: 
49: /// MCP Server for Voice-to-Text functionality
50: #[derive(Clone)]
51: pub struct VttMcpServer {
52:     sessions: Arc<Mutex<HashMap<Uuid, SessionState>>>,
53:     transcription_history: Arc<Mutex<Vec<HistoryEntry>>>,
54:     audio_config: Arc<Mutex<AudioRuntimeConfig>>,
55:     tool_router: ToolRouter<Self>,
56:     /// Track subscribers for each session's live transcription
57:     subscribers: Arc<Mutex<HashMap<Uuid, Vec<SessionSubscriber>>>>,
58:     /// Broadcast channel for transcription updates
59:     transcription_tx: broadcast::Sender<TranscriptionUpdate>,
60:     history_config: HistoryConfig,
61: }
62: /// Configuration for transcript history storage
63: #[derive(Debug, Clone)]
64: pub struct HistoryConfig {
65:     pub max_entries: usize,
66:     pub persistence_path: Option<PathBuf>,
67: }
68: 
69: impl Default for HistoryConfig {
70:     fn default() -> Self {
71:         Self {
72:             max_entries: 100,
73:             persistence_path: None,
74:         }
75:     }
76: }
77: 
78: 
79: impl VttMcpServer {
80:     pub fn new() -> Self {
81:         let (transcription_tx, _) = broadcast::channel(100);
82:         Self {
83:             sessions: Arc::new(Mutex::new(HashMap::new())),
84:             transcription_history: Arc::new(Mutex::new(Vec::new())),
85:             audio_config: Arc::new(Mutex::new(AudioRuntimeConfig::default())),
86:             tool_router: Self::tool_router(),
87:             subscribers: Arc::new(Mutex::new(HashMap::new())),
88:             transcription_tx,
89:             history_config: HistoryConfig::default(),
90:         }
91:     }
92: 
93:     async fn store_transcription_in_history(
94:         &self,
95:         session_id: Uuid,
96:         transcription: TranscriptionResult,
97:     ) {
98:         let entry = HistoryEntry {
99:             session_id,
100:             timestamp: Utc::now(),
101:             transcription,
102:         };
103:         let mut history = self.transcription_history.lock().await;
104:         history.insert(0, entry);
105:         if history.len() > 100 {
106:             history.truncate(100);
107:         }
108:     }
109: 
110:     /// Broadcast transcription update to all subscribers
111:     pub async fn broadcast_transcription(&self, update: TranscriptionUpdate) {
112:         let _ = self.transcription_tx.send(update);
113:     }
114: 
115:     /// Add a subscriber to a session
116:     pub async fn add_subscriber(&self, session_id: Uuid, client_id: String) -> VttResult<()> {
117:         let subscriber = SessionSubscriber {
118:             client_id,
119:             subscribed_at: Utc::now(),
120:         };
121:         let mut subscribers = self.subscribers.lock().await;
122:         subscribers.entry(session_id).or_insert_with(Vec::new).push(subscriber);
123:         Ok(())
124:     }
125: 
126:     /// Remove a subscriber from a session
127:     pub async fn remove_subscriber(&self, session_id: Uuid, client_id: &str) -> VttResult<()> {
128:         let mut subscribers = self.subscribers.lock().await;
129:         if let Some(subs) = subscribers.get_mut(&session_id) {
130:             subs.retain(|s| s.client_id != client_id);
131:             if subs.is_empty() {
132:                 subscribers.remove(&session_id);
133:             }
134:         }
135:         Ok(())
136:     }
137: 
138:     /// Get subscribers for a session
139:     pub async fn get_subscribers(&self, session_id: Uuid) -> Vec<SessionSubscriber> {
140:         let subscribers = self.subscribers.lock().await;
141:         subscribers.get(&session_id).cloned().unwrap_or_default()
142:     }
143: 
144:     /// Clean up subscribers for a session
145:     pub async fn cleanup_subscribers(&self, session_id: Uuid) {
146:         let mut subscribers = self.subscribers.lock().await;
147:         subscribers.remove(&session_id);
148:     }
149: }
150: 
151: impl Default for VttMcpServer {
152:     fn default() -> Self {
153:         Self::new()
154:     }
155: }
156: 
157: /// Implement ServerHandler for rmcp
158: impl ServerHandler for VttMcpServer {
159:     fn get_info(&self) -> ServerInfo {
160:         ServerInfo {
161:             protocol_version: rmcp::model::ProtocolVersion::default(),
162:             capabilities: ServerCapabilities {
163:                 tools: Some(rmcp::model::ToolsCapability::default()),
164:                 resources: Some(ResourcesCapability::default()), // Enable resources
165:                 ..Default::default()
166:             },
167:             server_info: rmcp::model::Implementation {
168:                 name: env!("CARGO_PKG_NAME").to_string(),
169:                 version: env!("CARGO_PKG_VERSION").to_string(),
170:                 ..Default::default()
171:             },
172:             instructions: Some(
173:                 "Voice-to-Text MCP server providing real-time transcription via Whisper. Resources: transcript://live/{session_id}".to_string()
174:             ),
175:         }
176:     }
177: 
178:     /// List available resources (active listening sessions)
179:     fn list_resources(
180:         &self,
181:         _request: Option<PaginatedRequestParam>,
182:         _context: RequestContext<RoleServer>,
183:     ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
184:         async move {
185:             let sessions = self.sessions.lock().await;
186:             let resources: Vec<Resource> = sessions
187:                 .iter()
188:                 .filter(|(_, s)| s.status == SessionStatus::Listening)
189:                 .map(|(id, _)| {
190:                     let raw = RawResource {
191:                         uri: format!("transcript://live/{}", id),
192:                         name: format!("session-{}", id),
193:                         title: Some(format!("Live transcription for session {}", id)),
194:                         description: Some("Real-time transcription stream".to_string()),
195:                         mime_type: Some("text".to_string()),
196:                         size: None,
197:                         icons: None,
198:                         meta: None,
199:                     };
200:                     Annotated::new(raw, None)
201:                 })
202:                 .collect();
203: 
204:             Ok(ListResourcesResult {
205:                 resources,
206:                 next_cursor: None,
207:                 meta: None,
208:             })
209:         }
210:     }
211: 
212:     /// Subscribe to a session's live transcription
213:     fn subscribe(
214:         &self,
215:         request: SubscribeRequestParam,
216:         context: RequestContext<RoleServer>,
217:     ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
218:         async move {
219:             let uri = request.uri;
220: 
221:             // Parse session_id from URI
222:             if !uri.starts_with("transcript://live/") {
223:                 return Err(McpError::from(VttError::invalid_params(
224:                     "Invalid resource URI. Expected: transcript://live/{session_id}"
225:                 )));
226:             }
227: 
228:             let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
229:             let session_id = Uuid::parse_str(session_id_str)
230:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;
231: 
232:             // Verify session exists and is listening
233:             let sessions = self.sessions.lock().await;
234:             if !sessions.contains_key(&session_id) {
235:                 return Err(McpError::from(VttError::invalid_params("Session not found")));
236:             }
237:             drop(sessions);
238: 
239:             // Generate a unique client ID from connection info
240:             let client_id = format!("{:?}", std::ptr::addr_of!(context));
241: 
242:             // Add subscriber
243:             self.add_subscriber(session_id, client_id.clone()).await
244:                 .map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
245: 
246:             tracing::info!("Client {} subscribed to transcript://live/{}", client_id, session_id);
247: 
248:             Ok(())
249:         }
250:     }
251: 
252:     /// Unsubscribe from a session's live transcription
253:     fn unsubscribe(
254:         &self,
255:         request: UnsubscribeRequestParam,
256:         context: RequestContext<RoleServer>,
257:     ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
258:         async move {
259:             let uri = request.uri;
260: 
261:             if !uri.starts_with("transcript://live/") {
262:                 return Err(McpError::from(VttError::invalid_params("Invalid resource URI")));
263:             }
264: 
265:             let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
266:             let session_id = Uuid::parse_str(session_id_str)
267:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;
268: 
269:             let client_id = format!("{:?}", std::ptr::addr_of!(context));
270: 
271:             self.remove_subscriber(session_id, &client_id).await
272:                 .map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
273: 
274:             tracing::info!("Client {} unsubscribed from transcript://live/{}", client_id, session_id);
275: 
276:             Ok(())
277:         }
278:     }
279: 
280:     /// Read current transcription state
281:     fn read_resource(
282:         &self,
283:         request: ReadResourceRequestParam,
284:         _context: RequestContext<RoleServer>,
285:     ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
286:         async move {
287:             let uri = request.uri;
288: 
289:             if !uri.starts_with("transcript://live/") {
290:                 return Err(McpError::from(VttError::invalid_params("Invalid resource URI")));
291:             }
292: 
293:             let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
294:             let session_id = Uuid::parse_str(session_id_str)
295:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;
296: 
297:             let sessions = self.sessions.lock().await;
298:             let session = sessions.get(&session_id)
299:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;
300: 
301:             let text = session.transcription.as_ref()
302:                 .map(|t| t.text.clone())
303:                 .unwrap_or_else(|| "No transcription yet".to_string());
304: 
305:             let contents = vec![
306:                 ResourceContents::text(text, uri)
307:             ];
308: 
309:             Ok(ReadResourceResult { contents })
310:         }
311:     }
312: 
313:     /// List resource templates (not used yet, returning empty)
314:     fn list_resource_templates(
315:         &self,
316:         _request: Option<PaginatedRequestParam>,
317:         _context: RequestContext<RoleServer>,
318:     ) -> impl std::future::Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
319:         async move {
320:             Ok(ListResourceTemplatesResult::default())
321:         }
322:     }
323: }
324: 
325: /// Tool router implementation
326: #[tool_router]
327: impl VttMcpServer {
328:     /// List supported languages
329:     #[tool(description = "List all supported languages for transcription")]
330:     async fn list_languages(
331:         &self,
332:         _params: Parameters<ListLanguagesParams>,
333:     ) -> Result<CallToolResult, McpError> {
334:         let mut lines = vec!["Supported Languages:".to_string()];
335:         
336:         // Add auto-detect option
337:         lines.push(format!("  auto - Auto-detect language"));
338:         
339:         // Add all supported languages
340:         for lang in SUPPORTED_LANGUAGES {
341:             lines.push(format!("  {} - {}", lang.code, lang.name));
342:         }
343:         
344:         lines.push(String::new());
345:         lines.push(format!("Total: {} languages", SUPPORTED_LANGUAGES.len() + 1));
346:         
347:         Ok(CallToolResult::success(vec![
348:             Content::text(lines.join("\n"))
349:         ]))
350:     }
351: 
352:     /// Transcribe an audio clip file
353:     #[tool(description = "Transcribe an audio clip from a WAV file")]
354:     async fn transcribe_clip(
355:         &self,
356:         params: Parameters<TranscribeClipParams>,
357:     ) -> Result<CallToolResult, McpError> {
358:         let p = params.0;
359:         
360:         use hound::WavReader;
361:         use std::path::Path;
362: 
363:         let path = Path::new(&p.audio_file);
364:         if !path.exists() {
365:             return Err(McpError::from(VttError::invalid_params(format!("Audio file not found: {}", p.audio_file))));
366:         }
367: 
368:         let reader = WavReader::open(&path)
369:             .map_err(|e| McpError::from(VttError::AudioFile(e)))?;
370: 
371:         let samples: Vec<f32> = reader
372:             .into_samples::<i16>()
373:             .filter_map(|s| s.ok())
374:             .map(|s| s as f32 / 32768.0)
375:             .collect();
376: 
377:         if samples.is_empty() {
378:             return Err(McpError::from(VttError::NoAudioData("Audio file contains no samples".to_string())));
379:         }
380: 
381:         // Validate language if provided
382:         if let Some(ref lang) = p.language {
383:             if lang != "auto" && !Language::is_valid(lang) {
384:                 return Err(McpError::from(VttError::invalid_params(format!(
385:                     "Unsupported language code: '{}'. Use list_languages tool to see supported languages.",
386:                     lang
387:                 ))));
388:             }
389:         }
390: 
391:         let model_path = p.model_path
392:             .or_else(|| std::env::var("WHISPER_MODEL").ok())
393:             .unwrap_or_else(|| "models/ggml-base.bin".to_string());
394: 
395:         let threads = p.threads
396:             .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
397:             .unwrap_or_else(|| num_cpus::get()) as i32;
398: 
399:         let use_gpu = p.use_gpu
400:             .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
401:             .unwrap_or(true);
402: 
403:         // Convert language option for Whisper config (None means auto-detect)
404:         let language = p.language.as_ref().and_then(|l| {
405:             if l == "auto" { None } else { Some(l.clone()) }
406:         });
407: 
408:         let config = WhisperConfig {
409:             model_path,
410:             language,
411:             use_gpu,
412:             n_threads: threads,
413:             ..Default::default()
414:         };
415: 
416:         let ctx = WhisperContext::new(config)
417:             .map_err(|e| McpError::from(VttError::Model(e.to_string())))?;
418: 
419:         let start_ms = 0u64;
420:         let duration_ms = (samples.len() as f64 / 16000.0 * 1000.0) as u64;
421: 
422:         let transcription = ctx.transcribe(&samples, 16000)
423:             .map_err(|e| McpError::from(VttError::Transcription(e)))?;
424: 
425:         let result = TranscribeClipResult {
426:             text: transcription.text.clone(),
427:             confidence: None,
428:             start_ms,
429:             end_ms: start_ms + duration_ms,
430:         };
431: 
432:         let history_entry = TranscriptionResult {
433:             text: result.text.clone(),
434:             confidence: result.confidence,
435:             start_ms: result.start_ms,
436:             end_ms: result.end_ms,
437:         };
438: 
439:         let session_id = Uuid::new_v4();
440:         self.store_transcription_in_history(session_id, history_entry).await;
441: 
442:         Ok(CallToolResult::success(vec![
443:             Content::text(format!(
444:                 "Transcription: {}\nConfidence: {:?}\nDuration: {}ms\nLanguage: {:?}",
445:                 result.text,
446:                 result.confidence,
447:                 result.end_ms - result.start_ms,
448:                 p.language.unwrap_or_else(|| "auto".to_string())
449:             ))
450:         ]))
451:     }
452: 
453:     /// Start listening for audio
454:     #[tool(description = "Start capturing audio from microphone")]
455:     async fn start_listening(
456:         &self,
457:         params: Parameters<StartListeningParams>,
458:     ) -> Result<CallToolResult, McpError> {
459:         let p = params.0;
460:         
461:         let session_id = Uuid::new_v4();
462:         let start_time = Utc::now();
463: 
464:         let model_path = p.model_path
465:             .or_else(|| std::env::var("WHISPER_MODEL").ok())
466:             .unwrap_or_else(|| "models/ggml-base.bin".to_string());
467: 
468:         let threads = p.threads
469:             .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
470:             .unwrap_or_else(|| num_cpus::get()) as i32;
471: 
472:         let use_gpu = p.use_gpu
473:             .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
474:             .unwrap_or(true);
475: 
476:         // Validate language if provided
477:         if let Some(ref lang) = p.language {
478:             if lang != "auto" && !Language::is_valid(lang) {
479:                 return Err(McpError::from(VttError::invalid_params(format!(
480:                     "Unsupported language code: '{}'. Use list_languages tool to see supported languages.",
481:                     lang
482:                 ))));
483:             }
484:         }
485: 
486:         // Convert language option for Whisper config (None means auto-detect)
487:         let language = p.language.as_ref().and_then(|l| {
488:             if l == "auto" { None } else { Some(l.clone()) }
489:         });
490: 
491:         let config = WhisperConfig {
492:             model_path: model_path.clone(),
493:             language,
494:             use_gpu,
495:             n_threads: threads,
496:             ..Default::default()
497:         };
498: 
499:         let _audio_config = self.audio_config.lock().await;
500:         let capture = AudioCapture::new().map_err(|e| McpError::from(VttError::Audio(e)))?;
501: 
502:         let session = SessionState {
503:             status: SessionStatus::Listening,
504:             start_time,
505:             capture: Some(capture),
506:             config,
507:             transcription: None,
508:             transcription_timestamp: None,
509:             error: None,
510:         };
511: 
512:         let mut sessions = self.sessions.lock().await;
513:         sessions.insert(session_id, session);
514: 
515:         let language_display = p.language.as_ref()
516:             .map(|l| display_name(l))
517:             .unwrap_or_else(|| "Auto-detect".to_string());
518: 
519:         Ok(CallToolResult::success(vec![
520:             Content::text(format!(
521:                 "Started listening session: {}\nModel: {}\nLanguage: {}\nGPU: {}\nResource: transcript://live/{}",
522:                 session_id, model_path, language_display, use_gpu, session_id
523:             ))
524:         ]))
525:     }
526: 
527:     /// Stop listening and transcribe
528:     #[tool(description = "Stop capturing audio and optionally transcribe")]
529:     async fn stop_listening(
530:         &self,
531:         params: Parameters<StopListeningParams>,
532:     ) -> Result<CallToolResult, McpError> {
533:         let p = params.0;
534:         let session_uuid = p.session_id.parse::<Uuid>()
535:             .map_err(|_| McpError::from(VttError::invalid_params("Invalid session_id format")))?;
536: 
537:         let mut sessions = self.sessions.lock().await;
538:         
539:         let duration_ms = {
540:             let session = sessions.get_mut(&session_uuid)
541:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;
542: 
543:             if session.status != SessionStatus::Listening {
544:                 return Err(McpError::from(VttError::invalid_params("Session is not listening")));
545:             }
546: 
547:             let duration_ms = (Utc::now() - session.start_time).num_milliseconds() as u64;
548: 
549:             duration_ms
550:         };
551: 
552:         let session = sessions.get_mut(&session_uuid).unwrap();
553:         let _samples_captured = 0;
554: 
555:         let transcription = if p.transcribe.unwrap_or(true) {
556:             Some(TranscriptionResult {
557:                 text: "Placeholder transcription".to_string(),
558:                 confidence: None,
559:                 start_ms: 0,
560:                 end_ms: duration_ms,
561:             })
562:         } else {
563:             None
564:         };
565: 
566:         session.status = if transcription.is_some() {
567:             SessionStatus::Transcribed
568:         } else {
569:             SessionStatus::Stopped
570:         };
571: 
572:         session.transcription = transcription.clone();
573:         session.transcription_timestamp = Some(Utc::now());
574: 
575:         // Cleanup subscribers when session ends
576:         drop(sessions);
577:         self.cleanup_subscribers(session_uuid).await;
578: 
579:         if let Some(tx) = &transcription {
580:             let tx_clone = tx.clone();
581:             self.store_transcription_in_history(session_uuid, tx_clone).await;
582:         }
583: 
584:         Ok(CallToolResult::success(vec![
585:             Content::text(format!(
586:                 "Session {} stopped. Status: {}. Duration: {}ms",
587:                 session_uuid,
588:                 if transcription.is_some() { "transcribed" } else { "stopped" },
589:                 duration_ms
590:             ))
591:         ]))
592:     }
593: 
594:     /// Get last transcription
595:     #[tool(description = "Get the most recent transcription")]
596:     async fn get_last_transcription(
597:         &self,
598:         params: Parameters<GetLastTranscriptionParams>,
599:     ) -> Result<CallToolResult, McpError> {
600:         let p = params.0;
601:         
602:         if let Some(session_id_str) = p.session_id {
603:             let session_uuid = session_id_str.parse::<Uuid>()
604:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session_id format")))?;
605: 
606:             let sessions = self.sessions.lock().await;
607:             let session = sessions.get(&session_uuid)
608:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;
609: 
610:             let transcription = session.transcription.as_ref()
611:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session has no transcription".to_string())))?;
612: 
613:             Ok(CallToolResult::success(vec![
614:                 Content::text(format!(
615:                     "Session: {}\nText: {}\nConfidence: {:?}\nTime: {}ms",
616:                     session_uuid,
617:                     transcription.text,
618:                     transcription.confidence,
619:                     transcription.end_ms - transcription.start_ms
620:                 ))
621:             ]))
622:         } else {
623:             let history = self.transcription_history.lock().await;
624:             let entry = history.first()
625:                 .ok_or_else(|| McpError::from(VttError::internal("No transcriptions available")))?;
626: 
627:             Ok(CallToolResult::success(vec![
628:                 Content::text(format!(
629:                     "Session: {}\nText: {}\nConfidence: {:?}\nTime: {}ms",
630:                     entry.session_id,
631:                     entry.transcription.text,
632:                     entry.transcription.confidence,
633:                     entry.transcription.end_ms - entry.transcription.start_ms
634:                 ))
635:             ]))
636:         }
637:     }
638: 
639:     /// List audio devices
640:     #[tool(description = "List available audio capture devices")]
641:     async fn list_audio_devices(
642:         &self,
643:         _params: Parameters<ListAudioDevicesParams>,
644:     ) -> Result<CallToolResult, McpError> {
645:         let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
646: 
647:         let audio_config = self.audio_config.lock().await;
648:         let default_device = audio_config.default_device
649:             .clone()
650:             .unwrap_or_else(|| {
651:                 devices.first()
652:                     .map(|d| d.name.clone())
653:                     .unwrap_or_else(|| "default".to_string())
654:             });
655: 
656:         let device_list: Vec<String> = devices
657:             .iter()
658:             .map(|d| format!("{}{}", 
659:                 if audio_config.default_device.as_ref() == Some(&d.name) { "* " } else { "" },
660:                 d.name
661:             ))
662:             .collect();
663: 
664:         Ok(CallToolResult::success(vec![
665:             Content::text(format!(
666:                 "Audio Devices:\nDefault: {}\n\n{}",
667:                 default_device,
668:                 device_list.join("\n")
669:             ))
670:         ]))
671:     }
672: 
673:     /// Configure audio settings
674:     #[tool(description = "Configure audio capture settings")]
675:     async fn configure_audio(
676:         &self,
677:         params: Parameters<ConfigureAudioParams>,
678:     ) -> Result<CallToolResult, McpError> {
679:         let p = params.0;
680:         
681:         let mut config = self.audio_config.lock().await;
682: 
683:         if let Some(ref device_name) = p.device_name {
684:             let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
685:             let device_exists = devices.iter().any(|d| d.name == *device_name);
686:             if !device_exists {
687:                 return Err(McpError::from(VttError::device_not_found(device_name)));
688:             }
689:             config.default_device = Some(device_name.clone());
690:         }
691: 
692:         if let Some(sensitivity) = p.vad_sensitivity {
693:             config.vad_config.energy_threshold = sensitivity.clamp(0.0, 1.0);
694:         }
695: 
696:         let default_device = config.default_device.clone();
697:         let vad_config = config.vad_config.clone();
698: 
699:         drop(config);
700: 
701:         let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
702: 
703:         let device_list: Vec<String> = devices
704:             .iter()
705:             .map(|d| format!("{}{}", 
706:                 if default_device.as_ref() == Some(&d.name) { "* " } else { "" },
707:                 d.name
708:             ))
709:             .collect();
710: 
711:         Ok(CallToolResult::success(vec![
712:             Content::text(format!(
713:                 "Audio configuration updated:\nDefault device: {}\nVAD threshold: {:.2}\n\nDevices:\n{}",
714:                 default_device.unwrap_or_else(|| "default".to_string()),
715:                 vad_config.energy_threshold,
716:                 device_list.join("\n")
717:             ))
718:         ]))
719:     }
720: }
721: 
722: // Internal types
723: 
724: #[derive(Debug, Clone)]
725: struct SessionState {
726:     status: SessionStatus,
727:     start_time: DateTime<Utc>,
728:     capture: Option<AudioCapture>,
729:     config: WhisperConfig,
730:     transcription: Option<TranscriptionResult>,
731:     transcription_timestamp: Option<DateTime<Utc>>,
732:     error: Option<String>,
733: }
734: 
735: impl SessionState {
736:     fn status_display(&self) -> &str {
737:         match self.status {
738:             SessionStatus::Listening => "listening",
739:             SessionStatus::Stopped => "stopped",
740:             SessionStatus::Transcribed => "transcribed",
741:             SessionStatus::Error => "error",
742:         }
743:     }
744: }
745: 
746: #[derive(Debug, Clone, PartialEq)]
747: enum SessionStatus {
748:     Listening,
749:     Stopped,
750:     Transcribed,
751:     Error,
752: }
753: 
754: #[derive(Clone, Serialize, Deserialize)]
755: struct HistoryEntry {
756:     session_id: Uuid,
757:     timestamp: DateTime<Utc>,
758:     transcription: TranscriptionResult,
759: }
760: 
761: #[derive(Debug, Clone)]
762: struct AudioRuntimeConfig {
763:     default_device: Option<String>,
764:     vad_config: VadConfigInfo,
765: }
766: 
767: impl Default for AudioRuntimeConfig {
768:     fn default() -> Self {
769:         Self {
770:             default_device: None,
771:             vad_config: VadConfigInfo::default(),
772:         }
773:     }
774: }
775: 
776: // Tool parameter types
777: 
778: #[derive(Debug, Clone, Deserialize, JsonSchema)]
779: pub struct ListLanguagesParams {}
780: 
781: #[derive(Debug, Clone, Deserialize, JsonSchema)]
782: pub struct TranscribeClipParams {
783:     pub audio_file: String,
784:     #[serde(default)]
785:     pub model_path: Option<String>,
786:     #[serde(default)]
787:     pub language: Option<String>,
788:     #[serde(default)]
789:     pub use_gpu: Option<bool>,
790:     #[serde(default)]
791:     pub threads: Option<usize>,
792: }
793: 
794: #[derive(Debug, Clone, Deserialize, JsonSchema)]
795: pub struct StartListeningParams {
796:     #[serde(default)]
797:     pub model_path: Option<String>,
798:     #[serde(default)]
799:     pub language: Option<String>,
800:     #[serde(default)]
801:     pub use_gpu: Option<bool>,
802:     #[serde(default)]
803:     pub threads: Option<usize>,
804:     #[serde(default)]
805:     pub device_name: Option<String>,
806: }
807: 
808: #[derive(Debug, Clone, Deserialize, JsonSchema)]
809: pub struct StopListeningParams {
810:     pub session_id: String,
811:     #[serde(default)]
812:     pub transcribe: Option<bool>,
813: }
814: 
815: #[derive(Debug, Clone, Deserialize, JsonSchema)]
816: pub struct GetLastTranscriptionParams {
817:     #[serde(default)]
818:     pub session_id: Option<String>,
819: }
820: 
821: #[derive(Debug, Clone, Deserialize, JsonSchema)]
822: pub struct ListAudioDevicesParams {}
823: 
824: #[derive(Debug, Clone, Deserialize, JsonSchema)]
825: pub struct ConfigureAudioParams {
826:     #[serde(default)]
827:     pub device_name: Option<String>,
828:     #[serde(default)]
829:     pub vad_sensitivity: Option<f32>,
830: }
831: 
832: // Tool result types
833: 
834: #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
835: pub struct VadConfigInfo {
836:     pub energy_threshold: f32,
837:     pub speech_frames_threshold: u32,
838:     pub silence_frames_threshold: u32,
839:     pub min_speech_duration: u32,
840: }
841: 
842: impl Default for VadConfigInfo {
843:     fn default() -> Self {
844:         Self {
845:             energy_threshold: 0.01,
846:             speech_frames_threshold: 3,
847:             silence_frames_threshold: 10,
848:             min_speech_duration: 30,
849:         }
850:     }
851: }
852: 
853: #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
854: pub struct TranscriptionResult {
855:     pub text: String,
856:     pub confidence: Option<f32>,
857:     pub start_ms: u64,
858:     pub end_ms: u64,
859: }
860: 
861: impl From<Transcription> for TranscriptionResult {
862:     fn from(tx: Transcription) -> Self {
863:         Self {
864:             text: tx.text,
865:             confidence: None,
866:             start_ms: tx.start_timestamp.max(0) as u64,
867:             end_ms: tx.end_timestamp.max(0) as u64,
868:         }
869:     }
870: }
871: 
872: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
873: pub struct TranscribeClipResult {
874:     pub text: String,
875:     pub confidence: Option<f32>,
876:     pub start_ms: u64,
877:     pub end_ms: u64,
878: }
879: 
880: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
881: pub struct StartListeningResult {
882:     pub session_id: String,
883:     pub status: String,
884:     pub start_time: DateTime<Utc>,
885:     pub model_path: String,
886:     pub language: Option<String>,
887:     pub use_gpu: bool,
888: }
889: 
890: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
891: pub struct StopListeningResult {
892:     pub session_id: String,
893:     pub status: String,
894:     pub duration_ms: u64,
895:     pub samples_captured: usize,
896:     pub transcription: Option<TranscriptionResult>,
897:     pub error: Option<String>,
898: }
899: 
900: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
901: pub struct LastTranscriptionResult {
902:     pub session_id: String,
903:     pub timestamp: DateTime<Utc>,
904:     pub text: String,
905:     pub confidence: Option<f32>,
906:     pub start_ms: u64,
907:     pub end_ms: u64,
908:     pub model_path: String,
909:     pub language: Option<String>,
910: }
911: 
912: #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
913: pub struct AudioDeviceInfo {
914:     pub name: String,
915:     pub is_default: bool,
916: }
917: 
918: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
919: pub struct AudioDevicesListResult {
920:     pub devices: Vec<AudioDeviceInfo>,
921:     pub default_device: String,
922: }
923: 
924: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
925: pub struct AudioConfigurationResult {
926:     pub default_device: Option<String>,
927:     pub vad_config: VadConfigInfo,
928:     pub available_devices: Vec<AudioDeviceInfo>,
929: }
930: 

impl ResourcesCapability for VttMcpServer {
    async fn list_resources(&self, _request: PaginatedRequestParam) -> Result<ListResourcesResult, McpError> {
        let mut resources = Vec::new();
        let history_raw = RawResource {
            uri: "transcript://history".to_string(),
            name: "history".to_string(),
            title: Some("Transcript History".to_string()),
            description: Some(format!(
                "Historical transcriptions (max {} entries, supports pagination via ?page=0&size=20)",
                self.history_config.max_entries
            )),
            mime_type: Some("application/json".to_string()),
            ..Default::default()
        };
        resources.push(Annotated::new(history_raw, None));
        let sessions = self.sessions.lock().await;
        for session_id in sessions.keys() {
            resources.push(Annotated::new(RawResource {
                uri: format!("transcript://live/{}", session_id),
                name: format!("live-{}", session_id),
                title: Some(format!("Live Session {}", session_id)),
                description: Some("Real-time transcription updates".to_string()),
                mime_type: Some("text/event-stream".to_string()),
                ..Default::default()
            }, None));
        }
        Ok(ListResourcesResult { resources })
    }

    async fn read_resource(&self, request: ReadResourceRequestParam) -> Result<ReadResourceResult, McpError> {
        let uri = request.uri;
        if uri == "transcript://history" || uri.starts_with("transcript://history?") {
            let mut page: usize = 0;
            let mut size: usize = 20;
            if let Some(query_pos) = uri.find('?') {
                for pair in uri[query_pos + 1..].split('&') {
                    let mut parts = pair.split('=');
                    if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                        match key {
                            "page" => page = value.parse().unwrap_or(0),
                            "size" => size = value.parse().unwrap_or(20).min(100),
                            _ => {}
                        }
                    }
                }
            }
            let entries = self.get_history_paginated(page * size, size).await;
            let contents = vec![ResourceContents::text(
                serde_json::to_string_pretty(&entries)
                    .map_err(|e| McpError::internal(format!("Failed to serialize: {}", e)))?,
                uri.clone()
            )];
            return Ok(ReadResourceResult { contents });
        }
        if uri.starts_with("transcript://live/") {
            let session_id = Uuid::parse_str(uri.strip_prefix("transcript://live/").unwrap())
                .map_err(|e| McpError::invalid_params(format!("Invalid session ID: {}", e)))?;
            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id)
                .ok_or_else(|| McpError::resource_not_found(format!("Session {} not found", session_id)))?;
            if let Some(ref transcription) = session.transcription {
                let contents = vec![ResourceContents::text(
                    serde_json::to_string_pretty(transcription)
                        .map_err(|e| McpError::internal(format!("Failed to serialize: {}", e)))?,
                    uri.clone()
                )];
                Ok(ReadResourceResult { contents })
            } else {
                Err(McpError::resource_not_found("No transcription".to_string()))
            }
        } else {
            Err(McpError::resource_not_found(format!("Unknown resource: {}", uri)))
        }
    }

    async fn subscribe(&self, request: SubscribeRequestParam) -> Result<(), McpError> {
        let uri = request.uri;
        if !uri.starts_with("transcript://live/") {
            return Err(McpError::invalid_params("Can only subscribe to live sessions".to_string()));
        }
        let session_id = Uuid::parse_str(uri.strip_prefix("transcript://live/").unwrap())
            .map_err(|e| McpError::invalid_params(format!("Invalid session ID: {}", e)))?;
        let mut subscribers = self.subscribers.lock().await;
        subscribers.entry(session_id).or_insert_with(Vec::new).push(SessionSubscriber {
            client_id: request.meta.map(|m| m.client_id).unwrap_or_default(),
            subscribed_at: Utc::now(),
        });
        Ok(())
    }

    async fn unsubscribe(&self, _request: UnsubscribeRequestParam) -> Result<(), McpError> {
        Ok(())
    }
}

931: #[cfg(test)]
932: mod tests {
933:     use super::*;
934: 
935:     #[tokio::test]
936:     async fn test_server_creation() {
937:         let server = VttMcpServer::new();
938:         assert!(server.sessions.lock().await.is_empty());
939:     }
940: 
941:     #[tokio::test]
942:     async fn test_subscriber_management() {
943:         let server = VttMcpServer::new();
944:         let session_id = Uuid::new_v4();
945:         let client_id = "test-client".to_string();
946: 
947:         server.add_subscriber(session_id, client_id.clone()).await.unwrap();
948:         let subscribers = server.get_subscribers(session_id).await;
949:         assert_eq!(subscribers.len(), 1);
950:         assert_eq!(subscribers[0].client_id, client_id);
951: 
952:         server.remove_subscriber(session_id, &client_id).await.unwrap();
953:         let subscribers = server.get_subscribers(session_id).await;
954:         assert_eq!(subscribers.len(), 0);
955:     }
956: 
957:     #[tokio::test]
958:     async fn test_broadcast_transcription() {
959:         let server = VttMcpServer::new();
960:         let mut rx = server.transcription_tx.subscribe();
961: 
962:         let update = TranscriptionUpdate {
963:             session_id: Uuid::new_v4(),
964:             text: "Hello world".to_string(),
965:             is_final: false,
966:             timestamp: Utc::now(),
967:             confidence: Some(0.95),
968:         };
969: 
970:         server.broadcast_transcription(update.clone()).await;
971:         
972:         let received = rx.recv().await.unwrap();
973:         assert_eq!(received.text, update.text);
974:         assert_eq!(received.session_id, update.session_id);
975:     }
976: 
977:     #[test]
978:     fn test_session_status_display() {
979:         let session = SessionState {
980:             status: SessionStatus::Listening,
981:             start_time: Utc::now(),
982:             capture: None,
983:             config: WhisperConfig::default(),
984:             transcription: None,
985:             transcription_timestamp: None,
986:             error: None,
987:         };
988:         assert_eq!(session.status_display(), "listening");
989:     }
990: }
```
