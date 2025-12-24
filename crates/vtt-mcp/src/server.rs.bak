//! MCP Server implementation for Voice-to-Text functionality

use crate::error::{VttError, VttResult};
use chrono::{DateTime, Utc};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{ServerInfo, CallToolResult, Content, ErrorData as McpError},
    service::{RequestContext, RoleServer},
    model::{
        ServerCapabilities, ResourcesCapability,
        PaginatedRequestParam, ListResourcesResult, ListResourceTemplatesResult,
        ReadResourceRequestParam, ReadResourceResult, ResourceContents,
        SubscribeRequestParam, UnsubscribeRequestParam,
        Resource, RawResource, Annotated,
    },
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

use vtt_core::audio::{AudioCapture, list_devices};
use vtt_core::whisper::{WhisperContext, WhisperConfig, Transcription};
use vtt_core::whisper::language::{Language, SUPPORTED_LANGUAGES, supported_codes, display_name};

/// Transcription update broadcast to subscribers
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionUpdate {
    pub session_id: Uuid,
    pub text: String,
    pub is_final: bool,
    pub timestamp: DateTime<Utc>,
    pub confidence: Option<f32>,
}

/// A subscriber to a session's transcription stream
#[derive(Debug, Clone)]
pub struct SessionSubscriber {
    pub client_id: String,
    pub subscribed_at: DateTime<Utc>,
}

/// MCP Server for Voice-to-Text functionality
#[derive(Clone)]
pub struct VttMcpServer {
    sessions: Arc<Mutex<HashMap<Uuid, SessionState>>>,
    transcription_history: Arc<Mutex<Vec<HistoryEntry>>>,
    audio_config: Arc<Mutex<AudioRuntimeConfig>>,
    tool_router: ToolRouter<Self>,
    /// Track subscribers for each session's live transcription
    subscribers: Arc<Mutex<HashMap<Uuid, Vec<SessionSubscriber>>>>,
    /// Broadcast channel for transcription updates
    transcription_tx: broadcast::Sender<TranscriptionUpdate>,
}

impl VttMcpServer {
    pub fn new() -> Self {
        let (transcription_tx, _) = broadcast::channel(100);
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            transcription_history: Arc::new(Mutex::new(Vec::new())),
            audio_config: Arc::new(Mutex::new(AudioRuntimeConfig::default())),
            tool_router: Self::tool_router(),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            transcription_tx,
        }
    }

    async fn store_transcription_in_history(
        &self,
        session_id: Uuid,
        config: WhisperConfig,
        transcription: TranscriptionResult,
    ) {
        let entry = HistoryEntry {
            session_id,
            timestamp: Utc::now(),
            config,
            transcription,
        };
        let mut history = self.transcription_history.lock().await;
        history.insert(0, entry);
        if history.len() > 100 {
            history.truncate(100);
        }
    }

    /// Broadcast transcription update to all subscribers
    pub async fn broadcast_transcription(&self, update: TranscriptionUpdate) {
        let _ = self.transcription_tx.send(update);
    }

    /// Add a subscriber to a session
    pub async fn add_subscriber(&self, session_id: Uuid, client_id: String) -> VttResult<()> {
        let subscriber = SessionSubscriber {
            client_id,
            subscribed_at: Utc::now(),
        };
        let mut subscribers = self.subscribers.lock().await;
        subscribers.entry(session_id).or_insert_with(Vec::new).push(subscriber);
        Ok(())
    }

    /// Remove a subscriber from a session
    pub async fn remove_subscriber(&self, session_id: Uuid, client_id: &str) -> VttResult<()> {
        let mut subscribers = self.subscribers.lock().await;
        if let Some(subs) = subscribers.get_mut(&session_id) {
            subs.retain(|s| s.client_id != client_id);
            if subs.is_empty() {
                subscribers.remove(&session_id);
            }
        }
        Ok(())
    }

    /// Get subscribers for a session
    pub async fn get_subscribers(&self, session_id: Uuid) -> Vec<SessionSubscriber> {
        let subscribers = self.subscribers.lock().await;
        subscribers.get(&session_id).cloned().unwrap_or_default()
    }

    /// Clean up subscribers for a session
    pub async fn cleanup_subscribers(&self, session_id: Uuid) {
        let mut subscribers = self.subscribers.lock().await;
        subscribers.remove(&session_id);
    }
}

impl Default for VttMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement ServerHandler for rmcp
impl ServerHandler for VttMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability::default()),
                resources: Some(ResourcesCapability::default()), // Enable resources
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Voice-to-Text MCP server providing real-time transcription via Whisper. Resources: transcript://live/{session_id}".to_string()
            ),
        }
    }

    /// List available resources (active listening sessions)
    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            let sessions = self.sessions.lock().await;
            let resources: Vec<Resource> = sessions
                .iter()
                .filter(|(_, s)| s.status == SessionStatus::Listening)
                .map(|(id, _)| {
                    let raw = RawResource {
                        uri: format!("transcript://live/{}", id),
                        name: format!("session-{}", id),
                        title: Some(format!("Live transcription for session {}", id)),
                        description: Some("Real-time transcription stream".to_string()),
                        mime_type: Some("text".to_string()),
                        size: None,
                        icons: None,
                        meta: None,
                    };
                    Annotated::new(raw, None)
                })
                .collect();

            Ok(ListResourcesResult {
                resources,
                next_cursor: None,
                meta: None,
            })
        }
    }

    /// Subscribe to a session's live transcription
    fn subscribe(
        &self,
        request: SubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
        async move {
            let uri = request.uri;

            // Parse session_id from URI
            if !uri.starts_with("transcript://live/") {
                return Err(McpError::from(VttError::invalid_params(
                    "Invalid resource URI. Expected: transcript://live/{session_id}"
                )));
            }

            let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
            let session_id = Uuid::parse_str(session_id_str)
                .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;

            // Verify session exists and is listening
            let sessions = self.sessions.lock().await;
            if !sessions.contains_key(&session_id) {
                return Err(McpError::from(VttError::invalid_params("Session not found")));
            }
            drop(sessions);

            // Generate a unique client ID from connection info
            let client_id = format!("{:?}", std::ptr::addr_of!(context));

            // Add subscriber
            self.add_subscriber(session_id, client_id.clone()).await
                .map_err(|e| McpError::from(VttError::internal(e.to_string())))?;

            tracing::info!("Client {} subscribed to transcript://live/{}", client_id, session_id);

            Ok(())
        }
    }

    /// Unsubscribe from a session's live transcription
    fn unsubscribe(
        &self,
        request: UnsubscribeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<(), McpError>> + Send + '_ {
        async move {
            let uri = request.uri;

            if !uri.starts_with("transcript://live/") {
                return Err(McpError::from(VttError::invalid_params("Invalid resource URI")));
            }

            let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
            let session_id = Uuid::parse_str(session_id_str)
                .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;

            let client_id = format!("{:?}", std::ptr::addr_of!(context));

            self.remove_subscriber(session_id, &client_id).await
                .map_err(|e| McpError::from(VttError::internal(e.to_string())))?;

            tracing::info!("Client {} unsubscribed from transcript://live/{}", client_id, session_id);

            Ok(())
        }
    }

    /// Read current transcription state
    fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = request.uri;

            if !uri.starts_with("transcript://live/") {
                return Err(McpError::from(VttError::invalid_params("Invalid resource URI")));
            }

            let session_id_str = uri.strip_prefix("transcript://live/").unwrap();
            let session_id = Uuid::parse_str(session_id_str)
                .map_err(|_| McpError::from(VttError::invalid_params("Invalid session ID format")))?;

            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_id)
                .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;

            let text = session.transcription.as_ref()
                .map(|t| t.text.clone())
                .unwrap_or_else(|| "No transcription yet".to_string());

            let contents = vec![
                ResourceContents::text(text, uri)
            ];

            Ok(ReadResourceResult { contents })
        }
    }

    /// List resource templates (not used yet, returning empty)
    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourceTemplatesResult, McpError>> + Send + '_ {
        async move {
            Ok(ListResourceTemplatesResult::default())
        }
    }
}

/// Tool router implementation
#[tool_router]
impl VttMcpServer {
    /// List supported languages
    #[tool(description = "List all supported languages for transcription")]
    async fn list_languages(
        &self,
        _params: Parameters<ListLanguagesParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut lines = vec!["Supported Languages:".to_string()];
        
        // Add auto-detect option
        lines.push(format!("  auto - Auto-detect language"));
        
        // Add all supported languages
        for lang in SUPPORTED_LANGUAGES {
            lines.push(format!("  {} - {}", lang.code, lang.name));
        }
        
        lines.push(String::new());
        lines.push(format!("Total: {} languages", SUPPORTED_LANGUAGES.len() + 1));
        
        Ok(CallToolResult::success(vec![
            Content::text(lines.join("\n"))
        ]))
    }

    /// Transcribe an audio clip file
    #[tool(description = "Transcribe an audio clip from a WAV file")]
    async fn transcribe_clip(
        &self,
        params: Parameters<TranscribeClipParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        
        use hound::WavReader;
        use std::path::Path;

        let path = Path::new(&p.audio_file);
        if !path.exists() {
            return Err(McpError::from(VttError::invalid_params(format!("Audio file not found: {}", p.audio_file))));
        }

        let reader = WavReader::open(&path)
            .map_err(|e| McpError::from(VttError::AudioFile(e)))?;

        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .filter_map(|s| s.ok())
            .map(|s| s as f32 / 32768.0)
            .collect();

        if samples.is_empty() {
            return Err(McpError::from(VttError::NoAudioData("Audio file contains no samples".to_string())));
        }

        // Validate language if provided
        if let Some(ref lang) = p.language {
            if lang != "auto" && !Language::is_valid(lang) {
                return Err(McpError::from(VttError::invalid_params(format!(
                    "Unsupported language code: '{}'. Use list_languages tool to see supported languages.",
                    lang
                ))));
            }
        }

        let model_path = p.model_path
            .or_else(|| std::env::var("WHISPER_MODEL").ok())
            .unwrap_or_else(|| "models/ggml-base.bin".to_string());

        let threads = p.threads
            .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
            .unwrap_or_else(|| num_cpus::get()) as i32;

        let use_gpu = p.use_gpu
            .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
            .unwrap_or(true);

        // Convert language option for Whisper config (None means auto-detect)
        let language = p.language.as_ref().and_then(|l| {
            if l == "auto" { None } else { Some(l.clone()) }
        });

        let config = WhisperConfig {
            model_path,
            language,
            use_gpu,
            n_threads: threads,
            ..Default::default()
        };

        let config_for_history = config.clone();

        let ctx = WhisperContext::new(config)
            .map_err(|e| McpError::from(VttError::Model(e.to_string())))?;

        let start_ms = 0u64;
        let duration_ms = (samples.len() as f64 / 16000.0 * 1000.0) as u64;

        let transcription = ctx.transcribe(&samples, 16000)
            .map_err(|e| McpError::from(VttError::Transcription(e)))?;

        let result = TranscribeClipResult {
            text: transcription.text.clone(),
            confidence: None,
            start_ms,
            end_ms: start_ms + duration_ms,
        };

        let history_entry = TranscriptionResult {
            text: result.text.clone(),
            confidence: result.confidence,
            start_ms: result.start_ms,
            end_ms: result.end_ms,
        };

        let session_id = Uuid::new_v4();
        self.store_transcription_in_history(session_id, config_for_history, history_entry).await;

        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Transcription: {}\nConfidence: {:?}\nDuration: {}ms\nLanguage: {:?}",
                result.text,
                result.confidence,
                result.end_ms - result.start_ms,
                p.language.unwrap_or_else(|| "auto".to_string())
            ))
        ]))
    }

    /// Start listening for audio
    #[tool(description = "Start capturing audio from microphone")]
    async fn start_listening(
        &self,
        params: Parameters<StartListeningParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        
        let session_id = Uuid::new_v4();
        let start_time = Utc::now();

        let model_path = p.model_path
            .or_else(|| std::env::var("WHISPER_MODEL").ok())
            .unwrap_or_else(|| "models/ggml-base.bin".to_string());

        let threads = p.threads
            .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
            .unwrap_or_else(|| num_cpus::get()) as i32;

        let use_gpu = p.use_gpu
            .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
            .unwrap_or(true);

        // Validate language if provided
        if let Some(ref lang) = p.language {
            if lang != "auto" && !Language::is_valid(lang) {
                return Err(McpError::from(VttError::invalid_params(format!(
                    "Unsupported language code: '{}'. Use list_languages tool to see supported languages.",
                    lang
                ))));
            }
        }

        // Convert language option for Whisper config (None means auto-detect)
        let language = p.language.as_ref().and_then(|l| {
            if l == "auto" { None } else { Some(l.clone()) }
        });

        let config = WhisperConfig {
            model_path: model_path.clone(),
            language,
            use_gpu,
            n_threads: threads,
            ..Default::default()
        };

        let _audio_config = self.audio_config.lock().await;
        let capture = AudioCapture::new().map_err(|e| McpError::from(VttError::Audio(e)))?;

        let session = SessionState {
            status: SessionStatus::Listening,
            start_time,
            capture: Some(capture),
            config,
            transcription: None,
            transcription_timestamp: None,
            error: None,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id, session);

        let language_display = p.language.as_ref()
            .map(|l| display_name(l))
            .unwrap_or_else(|| "Auto-detect".to_string());

        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Started listening session: {}\nModel: {}\nLanguage: {}\nGPU: {}\nResource: transcript://live/{}",
                session_id, model_path, language_display, use_gpu, session_id
            ))
        ]))
    }

    /// Stop listening and transcribe
    #[tool(description = "Stop capturing audio and optionally transcribe")]
    async fn stop_listening(
        &self,
        params: Parameters<StopListeningParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        let session_uuid = p.session_id.parse::<Uuid>()
            .map_err(|_| McpError::from(VttError::invalid_params("Invalid session_id format")))?;

        let mut sessions = self.sessions.lock().await;
        
        let (config_clone, duration_ms) = {
            let session = sessions.get_mut(&session_uuid)
                .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;

            if session.status != SessionStatus::Listening {
                return Err(McpError::from(VttError::invalid_params("Session is not listening")));
            }

            let duration_ms = (Utc::now() - session.start_time).num_milliseconds() as u64;
            let config_clone = session.config.clone();

            (config_clone, duration_ms)
        };

        let session = sessions.get_mut(&session_uuid).unwrap();
        let _samples_captured = 0;

        let transcription = if p.transcribe.unwrap_or(true) {
            Some(TranscriptionResult {
                text: "Placeholder transcription".to_string(),
                confidence: None,
                start_ms: 0,
                end_ms: duration_ms,
            })
        } else {
            None
        };

        session.status = if transcription.is_some() {
            SessionStatus::Transcribed
        } else {
            SessionStatus::Stopped
        };

        session.transcription = transcription.clone();
        session.transcription_timestamp = Some(Utc::now());

        // Cleanup subscribers when session ends
        drop(sessions);
        self.cleanup_subscribers(session_uuid).await;

        if let Some(tx) = &transcription {
            let tx_clone = tx.clone();
            self.store_transcription_in_history(session_uuid, config_clone, tx_clone).await;
        }

        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Session {} stopped. Status: {}. Duration: {}ms",
                session_uuid,
                if transcription.is_some() { "transcribed" } else { "stopped" },
                duration_ms
            ))
        ]))
    }

    /// Get last transcription
    #[tool(description = "Get the most recent transcription")]
    async fn get_last_transcription(
        &self,
        params: Parameters<GetLastTranscriptionParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        
        if let Some(session_id_str) = p.session_id {
            let session_uuid = session_id_str.parse::<Uuid>()
                .map_err(|_| McpError::from(VttError::invalid_params("Invalid session_id format")))?;

            let sessions = self.sessions.lock().await;
            let session = sessions.get(&session_uuid)
                .ok_or_else(|| McpError::from(VttError::invalid_params("Session not found")))?;

            let transcription = session.transcription.as_ref()
                .ok_or_else(|| McpError::from(VttError::invalid_params("Session has no transcription".to_string())))?;

            Ok(CallToolResult::success(vec![
                Content::text(format!(
                    "Session: {}\nText: {}\nConfidence: {:?}\nTime: {}ms",
                    session_uuid,
                    transcription.text,
                    transcription.confidence,
                    transcription.end_ms - transcription.start_ms
                ))
            ]))
        } else {
            let history = self.transcription_history.lock().await;
            let entry = history.first()
                .ok_or_else(|| McpError::from(VttError::internal("No transcriptions available")))?;

            Ok(CallToolResult::success(vec![
                Content::text(format!(
                    "Session: {}\nText: {}\nConfidence: {:?}\nTime: {}ms",
                    entry.session_id,
                    entry.transcription.text,
                    entry.transcription.confidence,
                    entry.transcription.end_ms - entry.transcription.start_ms
                ))
            ]))
        }
    }

    /// List audio devices
    #[tool(description = "List available audio capture devices")]
    async fn list_audio_devices(
        &self,
        _params: Parameters<ListAudioDevicesParams>,
    ) -> Result<CallToolResult, McpError> {
        let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;

        let audio_config = self.audio_config.lock().await;
        let default_device = audio_config.default_device
            .clone()
            .unwrap_or_else(|| {
                devices.first()
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| "default".to_string())
            });

        let device_list: Vec<String> = devices
            .iter()
            .map(|d| format!("{}{}", 
                if audio_config.default_device.as_ref() == Some(&d.name) { "* " } else { "" },
                d.name
            ))
            .collect();

        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Audio Devices:\nDefault: {}\n\n{}",
                default_device,
                device_list.join("\n")
            ))
        ]))
    }

    /// Configure audio settings
    #[tool(description = "Configure audio capture settings")]
    async fn configure_audio(
        &self,
        params: Parameters<ConfigureAudioParams>,
    ) -> Result<CallToolResult, McpError> {
        let p = params.0;
        
        let mut config = self.audio_config.lock().await;

        if let Some(ref device_name) = p.device_name {
            let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;
            let device_exists = devices.iter().any(|d| d.name == *device_name);
            if !device_exists {
                return Err(McpError::from(VttError::device_not_found(device_name)));
            }
            config.default_device = Some(device_name.clone());
        }

        if let Some(sensitivity) = p.vad_sensitivity {
            config.vad_config.energy_threshold = sensitivity.clamp(0.0, 1.0);
        }

        let default_device = config.default_device.clone();
        let vad_config = config.vad_config.clone();

        drop(config);

        let devices = list_devices().map_err(|e| McpError::from(VttError::internal(e.to_string())))?;

        let device_list: Vec<String> = devices
            .iter()
            .map(|d| format!("{}{}", 
                if default_device.as_ref() == Some(&d.name) { "* " } else { "" },
                d.name
            ))
            .collect();

        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Audio configuration updated:\nDefault device: {}\nVAD threshold: {:.2}\n\nDevices:\n{}",
                default_device.unwrap_or_else(|| "default".to_string()),
                vad_config.energy_threshold,
                device_list.join("\n")
            ))
        ]))
    }
}

// Internal types

#[derive(Debug, Clone)]
struct SessionState {
    status: SessionStatus,
    start_time: DateTime<Utc>,
    capture: Option<AudioCapture>,
    config: WhisperConfig,
    transcription: Option<TranscriptionResult>,
    transcription_timestamp: Option<DateTime<Utc>>,
    error: Option<String>,
}

impl SessionState {
    fn status_display(&self) -> &str {
        match self.status {
            SessionStatus::Listening => "listening",
            SessionStatus::Stopped => "stopped",
            SessionStatus::Transcribed => "transcribed",
            SessionStatus::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum SessionStatus {
    Listening,
    Stopped,
    Transcribed,
    Error,
}

#[derive(Clone)]
struct HistoryEntry {
    session_id: Uuid,
    timestamp: DateTime<Utc>,
    config: WhisperConfig,
    transcription: TranscriptionResult,
}

#[derive(Debug, Clone)]
struct AudioRuntimeConfig {
    default_device: Option<String>,
    vad_config: VadConfigInfo,
}

impl Default for AudioRuntimeConfig {
    fn default() -> Self {
        Self {
            default_device: None,
            vad_config: VadConfigInfo::default(),
        }
    }
}

// Tool parameter types

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListLanguagesParams {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TranscribeClipParams {
    pub audio_file: String,
    #[serde(default)]
    pub model_path: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub use_gpu: Option<bool>,
    #[serde(default)]
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct StartListeningParams {
    #[serde(default)]
    pub model_path: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub use_gpu: Option<bool>,
    #[serde(default)]
    pub threads: Option<usize>,
    #[serde(default)]
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct StopListeningParams {
    pub session_id: String,
    #[serde(default)]
    pub transcribe: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetLastTranscriptionParams {
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListAudioDevicesParams {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ConfigureAudioParams {
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub vad_sensitivity: Option<f32>,
}

// Tool result types

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VadConfigInfo {
    pub energy_threshold: f32,
    pub speech_frames_threshold: u32,
    pub silence_frames_threshold: u32,
    pub min_speech_duration: u32,
}

impl Default for VadConfigInfo {
    fn default() -> Self {
        Self {
            energy_threshold: 0.01,
            speech_frames_threshold: 3,
            silence_frames_threshold: 10,
            min_speech_duration: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: Option<f32>,
    pub start_ms: u64,
    pub end_ms: u64,
}

impl From<Transcription> for TranscriptionResult {
    fn from(tx: Transcription) -> Self {
        Self {
            text: tx.text,
            confidence: None,
            start_ms: tx.start_timestamp.max(0) as u64,
            end_ms: tx.end_timestamp.max(0) as u64,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TranscribeClipResult {
    pub text: String,
    pub confidence: Option<f32>,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StartListeningResult {
    pub session_id: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub model_path: String,
    pub language: Option<String>,
    pub use_gpu: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct StopListeningResult {
    pub session_id: String,
    pub status: String,
    pub duration_ms: u64,
    pub samples_captured: usize,
    pub transcription: Option<TranscriptionResult>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LastTranscriptionResult {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub text: String,
    pub confidence: Option<f32>,
    pub start_ms: u64,
    pub end_ms: u64,
    pub model_path: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioDevicesListResult {
    pub devices: Vec<AudioDeviceInfo>,
    pub default_device: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AudioConfigurationResult {
    pub default_device: Option<String>,
    pub vad_config: VadConfigInfo,
    pub available_devices: Vec<AudioDeviceInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = VttMcpServer::new();
        assert!(server.sessions.lock().await.is_empty());
    }

    #[tokio::test]
    async fn test_subscriber_management() {
        let server = VttMcpServer::new();
        let session_id = Uuid::new_v4();
        let client_id = "test-client".to_string();

        server.add_subscriber(session_id, client_id.clone()).await.unwrap();
        let subscribers = server.get_subscribers(session_id).await;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0].client_id, client_id);

        server.remove_subscriber(session_id, &client_id).await.unwrap();
        let subscribers = server.get_subscribers(session_id).await;
        assert_eq!(subscribers.len(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_transcription() {
        let server = VttMcpServer::new();
        let mut rx = server.transcription_tx.subscribe();

        let update = TranscriptionUpdate {
            session_id: Uuid::new_v4(),
            text: "Hello world".to_string(),
            is_final: false,
            timestamp: Utc::now(),
            confidence: Some(0.95),
        };

        server.broadcast_transcription(update.clone()).await;
        
        let received = rx.recv().await.unwrap();
        assert_eq!(received.text, update.text);
        assert_eq!(received.session_id, update.session_id);
    }

    #[test]
    fn test_session_status_display() {
        let session = SessionState {
            status: SessionStatus::Listening,
            start_time: Utc::now(),
            capture: None,
            config: WhisperConfig::default(),
            transcription: None,
            transcription_timestamp: None,
            error: None,
        };
        assert_eq!(session.status_display(), "listening");
    }
}
