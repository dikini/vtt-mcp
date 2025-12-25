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
24: 
25: use vtt_core::audio::{AudioCapture, list_devices};
26: use vtt_core::whisper::{WhisperContext, WhisperConfig, Transcription};
27: use vtt_core::whisper::language::{Language, SUPPORTED_LANGUAGES, display_name};
28: 
29: /// Transcription update broadcast to subscribers
30: #[derive(Debug, Clone, Serialize)]
31: /// Real-time transcription update sent to subscribers
32: ///
33: /// Emitted when new transcription results are available during
34: /// a listening session.
35: pub struct TranscriptionUpdate {
36:     pub session_id: Uuid,
37:     pub text: String,
38:     pub is_final: bool,
39:     pub timestamp: DateTime<Utc>,
40:     pub confidence: Option<f32>,
41: }
42: 
43: /// A subscriber to a session's transcription stream
44: #[derive(Debug, Clone)]
45: /// Subscriber receiving transcription updates
46: ///
47: /// Holds a receiver for broadcast channel updates.
48: pub struct SessionSubscriber {
49:     pub client_id: String,
50:     pub subscribed_at: DateTime<Utc>,
51: }
52: 
53: /// MCP Server for Voice-to-Text functionality
54: #[derive(Clone)]
55: /// VTT MCP Server implementation
56: ///
57: /// Main server implementing the Model Context Protocol for
58: /// voice-to-text functionality.
59: pub struct VttMcpServer {
60:     sessions: Arc<Mutex<HashMap<Uuid, SessionState>>>,
61:     transcription_history: Arc<Mutex<Vec<HistoryEntry>>>,
62:     audio_config: Arc<Mutex<AudioRuntimeConfig>>,
63:     tool_router: ToolRouter<Self>,
64:     /// Track subscribers for each session's live transcription
65:     subscribers: Arc<Mutex<HashMap<Uuid, Vec<SessionSubscriber>>>>,
66:     /// Broadcast channel for transcription updates
67:     transcription_tx: broadcast::Sender<TranscriptionUpdate>,
68: }
69: 
70: impl VttMcpServer {
71:     pub fn new() -> Self {
72:         let (transcription_tx, _) = broadcast::channel(100);
73:         Self {
74:             sessions: Arc::new(Mutex::new(HashMap::new())),
75:             transcription_history: Arc::new(Mutex::new(Vec::new())),
76:             audio_config: Arc::new(Mutex::new(AudioRuntimeConfig::default())),
77:             tool_router: Self::tool_router(),
78:             subscribers: Arc::new(Mutex::new(HashMap::new())),
79:             transcription_tx,
80:         }
81:     }
82: 
83:     async fn store_transcription_in_history(
84:         &self,
85:         session_id: Uuid,
86:         config: WhisperConfig,
87:         transcription: TranscriptionResult,
88:     ) {
89:         let entry = HistoryEntry {
90:             session_id,
91:             timestamp: Utc::now(),
92:             config,
93:             transcription,
94:         };
95:         let mut history = self.transcription_history.lock().await;
96:         history.insert(0, entry);
97:         if history.len() > 100 {
98:             history.truncate(100);
99:         }
100:     }
101: 
102:     /// Broadcast transcription update to all subscribers
103:     pub async fn broadcast_transcription(&self, update: TranscriptionUpdate) {
104:         let _ = self.transcription_tx.send(update);
105:     }
106: 
107:     /// Add a subscriber to a session
108:     pub async fn add_subscriber(&self, session_id: Uuid, client_id: String) -> VttResult<()> {
109:         let subscriber = SessionSubscriber {
110:             client_id,
111:             subscribed_at: Utc::now(),
112:         };
113:         let mut subscribers = self.subscribers.lock().await;
114:         subscribers.entry(session_id).or_insert_with(Vec::new).push(subscriber);
115:         Ok(())
116:     }
117: 
118:     /// Remove a subscriber from a session
119:     pub async fn remove_subscriber(&self, session_id: Uuid, client_id: &str) -> VttResult<()> {
120:         let mut subscribers = self.subscribers.lock().await;
121:         if let Some(subs) = subscribers.get_mut(&session_id) {
122:             subs.retain(|s| s.client_id != client_id);
123:             if subs.is_empty() {
124:                 subscribers.remove(&session_id);
125:             }
126:         }
127:         Ok(())
128:     }
129: 
130:     /// Get subscribers for a session
131:     pub async fn get_subscribers(&self, session_id: Uuid) -> Vec<SessionSubscriber> {
132:         let subscribers = self.subscribers.lock().await;
133:         subscribers.get(&session_id).cloned().unwrap_or_default()
134:     }
135: 
136:     /// Clean up subscribers for a session
137:     pub async fn cleanup_subscribers(&self, session_id: Uuid) {
138:         let mut subscribers = self.subscribers.lock().await;
139:         subscribers.remove(&session_id);
140:     }
141: }
142: 
143: impl Default for VttMcpServer {
144:     fn default() -> Self {
145:         Self::new()
146:     }
147: }
148: 
149: /// Implement ServerHandler for rmcp
150: impl ServerHandler for VttMcpServer {
151:     fn get_info(&self) -> ServerInfo {
152:         ServerInfo {
153:             protocol_version: rmcp::model::ProtocolVersion::default(),
154:             capabilities: ServerCapabilities {
155:                 tools: Some(rmcp::model::ToolsCapability::default()),
156:                 resources: Some(ResourcesCapability::default()), // Enable resources
157:                 ..Default::default()
158:             },
159:             server_info: rmcp::model::Implementation {
160:                 name: env!("CARGO_PKG_NAME").to_string(),
161:                 version: env!("CARGO_PKG_VERSION").to_string(),
162:                 ..Default::default()
163:             },
164:             instructions: Some(
165:                 "Voice-to-Text MCP server providing real-time transcription via Whisper. Resources: transcript://live/{session_id}".to_string()
166:             ),
167:         }
168:     }
169: 
170:     /// List available resources (active listening sessions)
171:     fn list_resources(
172:         &self,
173:         _request: Option<PaginatedRequestParam>,
174:         _context: RequestContext<RoleServer>,
175:     ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
176:         async move {
177:             let sessions = self.sessions.lock().await;
178:             let resources: Vec<Resource> = sessions
179:                 .iter()
180:                 .filter(|(_, s)| s.status == SessionStatus::Listening)
181:                 .map(|(id, _)| {
182:                     let raw = RawResource {
183:                         uri: format!("transcript://live/{}", id),
184:                         name: format!("session-{}", id),
185:                         title: Some(format!("Live transcription for session {}", id)),
186:                         description: Some("Real-time transcription stream".to_string()),
187:                         mime_type: Some("text".to_string()),
188:                         size: None,
189:                         icons: None,
190:                         meta: None,
191:                     };
192:                     Annotated::new(raw, None)
193:                 })
194:                 .collect();
195: 
196:             Ok(ListResourcesResult {
197:                 resources,
198:                 next_cursor: None,
199:                 meta: None,
200:             })
201:         }
202:     }
203: 
204:     /// Subscribe to a session's live transcription
205:     fn subscribe(
206:         &self,
207:         request: SubscribeRequestParam,
208:         context: RequestContext<RoleServer>,
209:     ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
210:         async move {
211:             let uri = request.uri;
212: 
213:             // Parse session_id from URI
214:             if !uri.starts_with("transcript://live/") {
215:                 return Err(McpError::from(VttError::invalid_params(
216:                     "Invalid resource URI. Expected: transcript://live/{session_id}"
217:                 )));
218:             }
219: 
220:             let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
221:             let session_id = Uuid::parse_str(session_id_str)
222:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;
223: 
224:             // Verify session exists and is listening
225:             let sessions = self.sessions.lock().await;
226:             if !sessions.contains_key(&session_id) {
227:                 return Err(McpError::from(VttError::invalid_params("Session not found")));
228:             }
229:             drop(sessions);
230: 
231:             // Generate a unique client ID from connection info
232:             let client_id = format!("{:?}", std::ptr::addr_of!(context));
233: 
234:             // Add subscriber
235:             self.add_subscriber(session_id, client_id.clone()).await
236:                 .map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
237: 
238:             tracing::info!("Client {} subscribed to transcript://live/{}", client_id, session_id);
239: 
240:             Ok(())
241:         }
242:     }
243: 
244:     /// Unsubscribe from a session's live transcription
245:     fn unsubscribe(
246:         &self,
247:         request: UnsubscribeRequestParam,
248:         context: RequestContext<RoleServer>,
249:     ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
250:         async move {
251:             let uri = request.uri;
252: 
253:             if !uri.starts_with("transcript://live/") {
254:                 return Err(McpError::from(VttError::invalid_params("Invalid resource URI")));
255:             }
256: 
257:             let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
258:             let session_id = Uuid::parse_str(session_id_str)
259:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;
260: 
261:             let client_id = format!("{:?}", std::ptr::addr_of!(context));
262: 
263:             self.remove_subscriber(session_id, &client_id).await
264:                 .map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
265: 
266:             tracing::info!("Client {} unsubscribed from transcript://live/{}", client_id, session_id);
267: 
268:             Ok(())
269:         }
270:     }
271: 
272:     /// Read current transcription state
273:     fn read_resource(
274:         &self,
275:         request: ReadResourceRequestParam,
276:         _context: RequestContext<RoleServer>,
277:     ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
278:         async move {
279:             let uri = request.uri;
280: 
281:             if !uri.starts_with("transcript://live/") {
282:                 return Err(McpError::from(VttError::invalid_params("Invalid resource URI")));
283:             }
284: 
285:             let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
286:             let session_id = Uuid::parse_str(session_id_str)
287:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;
288: 
289:             let sessions = self.sessions.lock().await;
290:             let session = sessions.get(&session_id)
291:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;
292: 
293:             let text = session.transcription.as_ref()
294:                 .map(|t| t.text.clone())
295:                 .unwrap_or_else(|| "No transcription yet".to_string());
296: 
297:             let contents = vec![
298:                 ResourceContents::text(text, uri)
299:             ];
300: 
301:             Ok(ReadResourceResult { contents })
302:         }
303:     }
304: 
305:     /// List resource templates (not used yet, returning empty)
306:     fn list_resource_templates(
307:         &self,
308:         _request: Option<PaginatedRequestParam>,
309:         _context: RequestContext<RoleServer>,
310:     ) -> impl std::future::Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
311:         async move {
312:             Ok(ListResourceTemplatesResult::default())
313:         }
314:     }
315: }
316: 
317: /// Tool router implementation
318: #[tool_router]
319: impl VttMcpServer {
320:     /// List supported languages
321:     #[tool(description = "List all supported languages for transcription")]
322:     async fn list_languages(
323:         &self,
324:         _params: Parameters<ListLanguagesParams>,
325:     ) -> Result<CallToolResult, McpError> {
326:         let mut lines = vec!["Supported Languages:".to_string()];
327:         
328:         // Add auto-detect option
329:         lines.push(format!("  auto - Auto-detect language"));
330:         
331:         // Add all supported languages
332:         for lang in SUPPORTED_LANGUAGES {
333:             lines.push(format!("  {} - {}", lang.code, lang.name));
334:         }
335:         
336:         lines.push(String::new());
337:         lines.push(format!("Total: {} languages", SUPPORTED_LANGUAGES.len() + 1));
338:         
339:         Ok(CallToolResult::success(vec![
340:             Content::text(lines.join("\n"))
341:         ]))
342:     }
343: 
344:     /// Transcribe an audio clip file
345:     #[tool(description = "Transcribe an audio clip from a WAV file")]
346:     async fn transcribe_clip(
347:         &self,
348:         params: Parameters<TranscribeClipParams>,
349:     ) -> Result<CallToolResult, McpError> {
350:         let p = params.0;
351:         
352:         use hound::WavReader;
353:         use std::path::Path;
354: 
355:         let path = Path::new(&p.audio_file);
356:         if !path.exists() {
357:             return Err(McpError::from(VttError::invalid_params(format!("Audio file not found: {}", p.audio_file))));
358:         }
359: 
360:         let reader = WavReader::open(&path)
361:             .map_err(|e| McpError::from(VttError::AudioFile(e)))?;
362: 
363:         let samples: Vec<f32> = reader
364:             .into_samples::<i16>()
365:             .filter_map(|s| s.ok())
366:             .map(|s| s as f32 / 32768.0)
367:             .collect();
368: 
369:         if samples.is_empty() {
370:             return Err(McpError::from(VttError::NoAudioData("Audio file contains no samples".to_string())));
371:         }
372: 
373:         // Validate language if provided
374:         if let Some(ref lang) = p.language {
375:             if lang != "auto" && !Language::is_valid(lang) {
376:                 return Err(McpError::from(VttError::invalid_params(format!(
377:                     "Unsupported language code: '{}'. Use list_languages tool to see supported languages.",
378:                     lang
379:                 ))));
380:             }
381:         }
382: 
383:         let model_path = p.model_path
384:             .or_else(|| std::env::var("WHISPER_MODEL").ok())
385:             .unwrap_or_else(|| "models/ggml-base.bin".to_string());
386: 
387:         let threads = p.threads
388:             .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
389:             .unwrap_or_else(|| num_cpus::get()) as i32;
390: 
391:         let use_gpu = p.use_gpu
392:             .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
393:             .unwrap_or(true);
394: 
395:         // Convert language option for Whisper config (None means auto-detect)
396:         let language = p.language.as_ref().and_then(|l| {
397:             if l == "auto" { None } else { Some(l.clone()) }
398:         });
399: 
400:         let config = WhisperConfig {
401:             model_path,
402:             language,
403:             use_gpu,
404:             n_threads: threads,
405:             ..Default::default()
406:         };
407: 
408:         let config_for_history = config.clone();
409: 
410:         let ctx = WhisperContext::new(config)
411:             .map_err(|e| McpError::from(VttError::Model(e.to_string())))?;
412: 
413:         let start_ms = 0u64;
414:         let duration_ms = (samples.len() as f64 / 16000.0 * 1000.0) as u64;
415: 
416:         let transcription = ctx.transcribe(&samples, 16000)
417:             .map_err(|e| McpError::from(VttError::Transcription(e)))?;
418: 
419:         let result = TranscribeClipResult {
420:             text: transcription.text.clone(),
421:             confidence: None,
422:             start_ms,
423:             end_ms: start_ms + duration_ms,
424:         };
425: 
426:         let history_entry = TranscriptionResult {
427:             text: result.text.clone(),
428:             confidence: result.confidence,
429:             start_ms: result.start_ms,
430:             end_ms: result.end_ms,
431:         };
432: 
433:         let session_id = Uuid::new_v4();
434:         self.store_transcription_in_history(session_id, config_for_history, history_entry).await;
435: 
436:         Ok(CallToolResult::success(vec![
437:             Content::text(format!(
438:                 "Transcription: {}\nConfidence: {:?}\nDuration: {}ms\nLanguage: {:?}",
439:                 result.text,
440:                 result.confidence,
441:                 result.end_ms - result.start_ms,
442:                 p.language.unwrap_or_else(|| "auto".to_string())
443:             ))
444:         ]))
445:     }
446: 
447:     /// Start listening for audio
448:     #[tool(description = "Start capturing audio from microphone")]
449:     async fn start_listening(
450:         &self,
451:         params: Parameters<StartListeningParams>,
452:     ) -> Result<CallToolResult, McpError> {
453:         let p = params.0;
454:         
455:         let session_id = Uuid::new_v4();
456:         let start_time = Utc::now();
457: 
458:         let model_path = p.model_path
459:             .or_else(|| std::env::var("WHISPER_MODEL").ok())
460:             .unwrap_or_else(|| "models/ggml-base.bin".to_string());
461: 
462:         let threads = p.threads
463:             .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
464:             .unwrap_or_else(|| num_cpus::get()) as i32;
465: 
466:         let use_gpu = p.use_gpu
467:             .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
468:             .unwrap_or(true);
469: 
470:         // Validate language if provided
471:         if let Some(ref lang) = p.language {
472:             if lang != "auto" && !Language::is_valid(lang) {
473:                 return Err(McpError::from(VttError::invalid_params(format!(
474:                     "Unsupported language code: '{}'. Use list_languages tool to see supported languages.",
475:                     lang
476:                 ))));
477:             }
478:         }
479: 
480:         // Convert language option for Whisper config (None means auto-detect)
481:         let language = p.language.as_ref().and_then(|l| {
482:             if l == "auto" { None } else { Some(l.clone()) }
483:         });
484: 
485:         let config = WhisperConfig {
486:             model_path: model_path.clone(),
487:             language,
488:             use_gpu,
489:             n_threads: threads,
490:             ..Default::default()
491:         };
492: 
493:         let _audio_config = self.audio_config.lock().await;
494:         let capture = AudioCapture::new().map_err(|e| McpError::from(VttError::Audio(e)))?;
495: 
496:         let session = SessionState {
497:             status: SessionStatus::Listening,
498:             start_time,
499:             capture: Some(capture),
500:             config,
501:             transcription: None,
502:             transcription_timestamp: None,
503:             error: None,
504:         };
505: 
506:         let mut sessions = self.sessions.lock().await;
507:         sessions.insert(session_id, session);
508: 
509:         let language_display = p.language.as_ref()
510:             .map(|l| display_name(l))
511:             .unwrap_or_else(|| "Auto-detect".to_string());
512: 
513:         Ok(CallToolResult::success(vec![
514:             Content::text(format!(
515:                 "Started listening session: {}\nModel: {}\nLanguage: {}\nGPU: {}\nResource: transcript://live/{}",
516:                 session_id, model_path, language_display, use_gpu, session_id
517:             ))
518:         ]))
519:     }
520: 
521:     /// Stop listening and transcribe
522:     #[tool(description = "Stop capturing audio and optionally transcribe")]
523:     async fn stop_listening(
524:         &self,
525:         params: Parameters<StopListeningParams>,
526:     ) -> Result<CallToolResult, McpError> {
527:         let p = params.0;
528:         let session_uuid = p.session_id.parse::<Uuid>()
529:             .map_err(|_| McpError::from(VttError::invalid_params("Invalid session_id format")))?;
530: 
531:         let mut sessions = self.sessions.lock().await;
532:         
533:         let (config_clone, duration_ms) = {
534:             let session = sessions.get_mut(&session_uuid)
535:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;
536: 
537:             if session.status != SessionStatus::Listening {
538:                 return Err(McpError::from(VttError::invalid_params("Session is not listening")));
539:             }
540: 
541:             let duration_ms = (Utc::now() - session.start_time).num_milliseconds() as u64;
542:             let config_clone = session.config.clone();
543: 
544:             (config_clone, duration_ms)
545:         };
546: 
547:         let session = sessions.get_mut(&session_uuid).unwrap();
548:         let _samples_captured = 0;
549: 
550:         let transcription = if p.transcribe.unwrap_or(true) {
551:             Some(TranscriptionResult {
552:                 text: "Placeholder transcription".to_string(),
553:                 confidence: None,
554:                 start_ms: 0,
555:                 end_ms: duration_ms,
556:             })
557:         } else {
558:             None
559:         };
560: 
561:         session.status = if transcription.is_some() {
562:             SessionStatus::Transcribed
563:         } else {
564:             SessionStatus::Stopped
565:         };
566: 
567:         session.transcription = transcription.clone();
568:         session.transcription_timestamp = Some(Utc::now());
569: 
570:         // Cleanup subscribers when session ends
571:         drop(sessions);
572:         self.cleanup_subscribers(session_uuid).await;
573: 
574:         if let Some(tx) = &transcription {
575:             let tx_clone = tx.clone();
576:             self.store_transcription_in_history(session_uuid, config_clone, tx_clone).await;
577:         }
578: 
579:         Ok(CallToolResult::success(vec![
580:             Content::text(format!(
581:                 "Session {} stopped. Status: {}. Duration: {}ms",
582:                 session_uuid,
583:                 if transcription.is_some() { "transcribed" } else { "stopped" },
584:                 duration_ms
585:             ))
586:         ]))
587:     }
588: 
589:     /// Get last transcription
590:     #[tool(description = "Get the most recent transcription")]
591:     async fn get_last_transcription(
592:         &self,
593:         params: Parameters<GetLastTranscriptionParams>,
594:     ) -> Result<CallToolResult, McpError> {
595:         let p = params.0;
596:         
597:         if let Some(session_id_str) = p.session_id {
598:             let session_uuid = session_id_str.parse::<Uuid>()
599:                 .map_err(|_| McpError::from(VttError::invalid_params("Invalid session_id format")))?;
600: 
601:             let sessions = self.sessions.lock().await;
602:             let session = sessions.get(&session_uuid)
603:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;
604: 
605:             let transcription = session.transcription.as_ref()
606:                 .ok_or_else(|| McpError::from(VttError::invalid_params("Session has no transcription".to_string())))?;
607: 
608:             Ok(CallToolResult::success(vec![
609:                 Content::text(format!(
610:                     "Session: {}\nText: {}\nConfidence: {:?}\nTime: {}ms",
611:                     session_uuid,
612:                     transcription.text,
613:                     transcription.confidence,
614:                     transcription.end_ms - transcription.start_ms
615:                 ))
616:             ]))
617:         } else {
618:             let history = self.transcription_history.lock().await;
619:             let entry = history.first()
620:                 .ok_or_else(|| McpError::from(VttError::internal("No transcriptions available")))?;
621: 
622:             Ok(CallToolResult::success(vec![
623:                 Content::text(format!(
624:                     "Session: {}\nText: {}\nConfidence: {:?}\nTime: {}ms",
625:                     entry.session_id,
626:                     entry.transcription.text,
627:                     entry.transcription.confidence,
628:                     entry.transcription.end_ms - entry.transcription.start_ms
629:                 ))
630:             ]))
631:         }
632:     }
633: 
634:     /// List audio devices
635:     #[tool(description = "List available audio capture devices")]
636:     async fn list_audio_devices(
637:         &self,
638:         _params: Parameters<ListAudioDevicesParams>,
639:     ) -> Result<CallToolResult, McpError> {
640:         let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
641: 
642:         let audio_config = self.audio_config.lock().await;
643:         let default_device = audio_config.default_device
644:             .clone()
645:             .unwrap_or_else(|| {
646:                 devices.first()
647:                     .map(|d| d.name.clone())
648:                     .unwrap_or_else(|| "default".to_string())
649:             });
650: 
651:         let device_list: Vec<String> = devices
652:             .iter()
653:             .map(|d| format!("{}{}", 
654:                 if audio_config.default_device.as_ref() == Some(&d.name) { "* " } else { "" },
655:                 d.name
656:             ))
657:             .collect();
658: 
659:         Ok(CallToolResult::success(vec![
660:             Content::text(format!(
661:                 "Audio Devices:\nDefault: {}\n\n{}",
662:                 default_device,
663:                 device_list.join("\n")
664:             ))
665:         ]))
666:     }
667: 
668:     /// Configure audio settings
669:     #[tool(description = "Configure audio capture settings")]
670:     async fn configure_audio(
671:         &self,
672:         params: Parameters<ConfigureAudioParams>,
673:     ) -> Result<CallToolResult, McpError> {
674:         let p = params.0;
675:         
676:         let mut config = self.audio_config.lock().await;
677: 
678:         if let Some(ref device_name) = p.device_name {
679:             let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
680:             let device_exists = devices.iter().any(|d| d.name == *device_name);
681:             if !device_exists {
682:                 return Err(McpError::from(VttError::device_not_found(device_name)));
683:             }
684:             config.default_device = Some(device_name.clone());
685:         }
686: 
687:         if let Some(sensitivity) = p.vad_sensitivity {
688:             config.vad_config.energy_threshold = sensitivity.clamp(0.0, 1.0);
689:         }
690: 
691:         let default_device = config.default_device.clone();
692:         let vad_config = config.vad_config.clone();
693: 
694:         drop(config);
695: 
696:         let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
697: 
698:         let device_list: Vec<String> = devices
699:             .iter()
700:             .map(|d| format!("{}{}", 
701:                 if default_device.as_ref() == Some(&d.name) { "* " } else { "" },
702:                 d.name
703:             ))
704:             .collect();
705: 
706:         Ok(CallToolResult::success(vec![
707:             Content::text(format!(
708:                 "Audio configuration updated:\nDefault device: {}\nVAD threshold: {:.2}\n\nDevices:\n{}",
709:                 default_device.unwrap_or_else(|| "default".to_string()),
710:                 vad_config.energy_threshold,
711:                 device_list.join("\n")
712:             ))
713:         ]))
714:     }
715: }
716: 
717: // Internal types
718: 
719: #[derive(Debug, Clone)]
720: struct SessionState {
721:     status: SessionStatus,
722:     start_time: DateTime<Utc>,
723:     capture: Option<AudioCapture>,
724:     config: WhisperConfig,
725:     transcription: Option<TranscriptionResult>,
726:     transcription_timestamp: Option<DateTime<Utc>>,
727:     error: Option<String>,
728: }
729: 
730: impl SessionState {
731:     fn status_display(&self) -> &str {
732:         match self.status {
733:             SessionStatus::Listening => "listening",
734:             SessionStatus::Stopped => "stopped",
735:             SessionStatus::Transcribed => "transcribed",
736:             SessionStatus::Error => "error",
737:         }
738:     }
739: }
740: 
741: #[derive(Debug, Clone, PartialEq)]
742: enum SessionStatus {
743:     Listening,
744:     Stopped,
745:     Transcribed,
746:     Error,
747: }
748: 
749: #[derive(Clone)]
750: /// Entry in transcript history
751: ///
752: /// Stores a completed transcription with metadata.
753: struct HistoryEntry {
754:     session_id: Uuid,
755:     timestamp: DateTime<Utc>,
756:     config: WhisperConfig,
757:     transcription: TranscriptionResult,
758: }
759: 
760: #[derive(Debug, Clone)]
761: struct AudioRuntimeConfig {
762:     default_device: Option<String>,
763:     vad_config: VadConfigInfo,
764: }
765: 
766: impl Default for AudioRuntimeConfig {
767:     fn default() -> Self {
768:         Self {
769:             default_device: None,
770:             vad_config: VadConfigInfo::default(),
771:         }
772:     }
773: }
774: 
775: // Tool parameter types
776: 
777: #[derive(Debug, Clone, Deserialize, JsonSchema)]
778: pub struct ListLanguagesParams {}
779: 
780: #[derive(Debug, Clone, Deserialize, JsonSchema)]
781: pub struct TranscribeClipParams {
782:     pub audio_file: String,
783:     #[serde(default)]
784:     pub model_path: Option<String>,
785:     #[serde(default)]
786:     pub language: Option<String>,
787:     #[serde(default)]
788:     pub use_gpu: Option<bool>,
789:     #[serde(default)]
790:     pub threads: Option<usize>,
791: }
792: 
793: #[derive(Debug, Clone, Deserialize, JsonSchema)]
794: pub struct StartListeningParams {
795:     #[serde(default)]
796:     pub model_path: Option<String>,
797:     #[serde(default)]
798:     pub language: Option<String>,
799:     #[serde(default)]
800:     pub use_gpu: Option<bool>,
801:     #[serde(default)]
802:     pub threads: Option<usize>,
803:     #[serde(default)]
804:     pub device_name: Option<String>,
805: }
806: 
807: #[derive(Debug, Clone, Deserialize, JsonSchema)]
808: pub struct StopListeningParams {
809:     pub session_id: String,
810:     #[serde(default)]
811:     pub transcribe: Option<bool>,
812: }
813: 
814: #[derive(Debug, Clone, Deserialize, JsonSchema)]
815: pub struct GetLastTranscriptionParams {
816:     #[serde(default)]
817:     pub session_id: Option<String>,
818: }
819: 
820: #[derive(Debug, Clone, Deserialize, JsonSchema)]
821: pub struct ListAudioDevicesParams {}
822: 
823: #[derive(Debug, Clone, Deserialize, JsonSchema)]
824: pub struct ConfigureAudioParams {
825:     #[serde(default)]
826:     pub device_name: Option<String>,
827:     #[serde(default)]
828:     pub vad_sensitivity: Option<f32>,
829: }
830: 
831: // Tool result types
832: 
833: #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
834: pub struct VadConfigInfo {
835:     pub energy_threshold: f32,
836:     pub speech_frames_threshold: u32,
837:     pub silence_frames_threshold: u32,
838:     pub min_speech_duration: u32,
839: }
840: 
841: impl Default for VadConfigInfo {
842:     fn default() -> Self {
843:         Self {
844:             energy_threshold: 0.01,
845:             speech_frames_threshold: 3,
846:             silence_frames_threshold: 10,
847:             min_speech_duration: 30,
848:         }
849:     }
850: }
851: 
852: #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
853: pub struct TranscriptionResult {
854:     pub text: String,
855:     pub confidence: Option<f32>,
856:     pub start_ms: u64,
857:     pub end_ms: u64,
858: }
859: 
860: impl From<Transcription> for TranscriptionResult {
861:     fn from(tx: Transcription) -> Self {
862:         Self {
863:             text: tx.text,
864:             confidence: None,
865:             start_ms: tx.start_timestamp.max(0) as u64,
866:             end_ms: tx.end_timestamp.max(0) as u64,
867:         }
868:     }
869: }
870: 
871: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
872: pub struct TranscribeClipResult {
873:     pub text: String,
874:     pub confidence: Option<f32>,
875:     pub start_ms: u64,
876:     pub end_ms: u64,
877: }
878: 
879: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
880: pub struct StartListeningResult {
881:     pub session_id: String,
882:     pub status: String,
883:     pub start_time: DateTime<Utc>,
884:     pub model_path: String,
885:     pub language: Option<String>,
886:     pub use_gpu: bool,
887: }
888: 
889: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
890: pub struct StopListeningResult {
891:     pub session_id: String,
892:     pub status: String,
893:     pub duration_ms: u64,
894:     pub samples_captured: usize,
895:     pub transcription: Option<TranscriptionResult>,
896:     pub error: Option<String>,
897: }
898: 
899: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
900: pub struct LastTranscriptionResult {
901:     pub session_id: String,
902:     pub timestamp: DateTime<Utc>,
903:     pub text: String,
904:     pub confidence: Option<f32>,
905:     pub start_ms: u64,
906:     pub end_ms: u64,
907:     pub model_path: String,
908:     pub language: Option<String>,
909: }
910: 
911: #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
912: 
        /// Information about an audio device.
        pub struct AudioDeviceInfo {
            /// The name of the audio device.
            pub name: String,
            /// Indication if this is the default device.
            pub is_default: bool,
        }
    
916: 
917: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
918: 
        /// Result containing the list of audio devices and the default device.
        pub struct AudioDevicesListResult {
            /// Vector of all available audio devices.
            pub devices: Vec<AudioDeviceInfo>,
            /// The default audio device identified.
            pub default_device: String,
        }
    
922: 
923: #[derive(Debug, Serialize, Deserialize, JsonSchema)]
924: 
        /// Configuration result detailing default and available audio devices.
        pub struct AudioConfigurationResult {
            /// The configured default device.
            pub default_device: Option<String>,
            /// Configuration for VAD (Voice Activity Detection).
            pub vad_config: VadConfigInfo,
            /// Available audio devices as a list.
            pub available_devices: Vec<AudioDeviceInfo>,
        }
    
929: 
930: #[cfg(test)]
931: mod tests {
932:     use super::*;
933: 
934:     #[tokio::test]
935:     async fn test_server_creation() {
936:         let server = VttMcpServer::new();
937:         assert!(server.sessions.lock().await.is_empty());
938:     }
939: 
940:     #[tokio::test]
941:     async fn test_subscriber_management() {
942:         let server = VttMcpServer::new();
943:         let session_id = Uuid::new_v4();
944:         let client_id = "test-client".to_string();
945: 
946:         server.add_subscriber(session_id, client_id.clone()).await.unwrap();
947:         let subscribers = server.get_subscribers(session_id).await;
948:         assert_eq!(subscribers.len(), 1);
949:         assert_eq!(subscribers[0].client_id, client_id);
950: 
951:         server.remove_subscriber(session_id, &client_id).await.unwrap();
952:         let subscribers = server.get_subscribers(session_id).await;
953:         assert_eq!(subscribers.len(), 0);
954:     }
955: 
956:     #[tokio::test]
957:     async fn test_broadcast_transcription() {
958:         let server = VttMcpServer::new();
959:         let mut rx = server.transcription_tx.subscribe();
960: 
961:         let update = TranscriptionUpdate {
962:             session_id: Uuid::new_v4(),
963:             text: "Hello world".to_string(),
964:             is_final: false,
965:             timestamp: Utc::now(),
966:             confidence: Some(0.95),
967:         };
968: 
969:         server.broadcast_transcription(update.clone()).await;
970:         
971:         let received = rx.recv().await.unwrap();
972:         assert_eq!(received.text, update.text);
973:         assert_eq!(received.session_id, update.session_id);
974:     }
975: 
976:     #[test]
977:     fn test_session_status_display() {
978:         let session = SessionState {
979:             status: SessionStatus::Listening,
980:             start_time: Utc::now(),
981:             capture: None,
982:             config: WhisperConfig::default(),
983:             transcription: None,
984:             transcription_timestamp: None,
985:             error: None,
986:         };
987:         assert_eq!(session.status_display(), "listening");
988:     }
989: }
```
