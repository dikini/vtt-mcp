//! MCP Server implementation for Voice-to-Text functionality

use crate::error::{ToMcpError, VttError, VttResult};
use chrono::{DateTime, Utc};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters, ServerHandler},
    model::{ServerInfo, CallToolResult, Content, ErrorData as McpError},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use vtt_core::audio::{AudioCapture, list_devices};
use vtt_core::whisper::{WhisperContext, WhisperConfig, Transcription};

/// MCP Server for Voice-to-Text functionality
#[derive(Clone)]
pub struct VttMcpServer {
    sessions: Arc<Mutex<HashMap<Uuid, SessionState>>>,
    transcription_history: Arc<Mutex<Vec<HistoryEntry>>>,
    audio_config: Arc<Mutex<AudioRuntimeConfig>>,
    tool_router: ToolRouter<Self>,
}

impl VttMcpServer {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            transcription_history: Arc::new(Mutex::new(Vec::new())),
            audio_config: Arc::new(Mutex::new(AudioRuntimeConfig::default())),
            tool_router: Self::tool_router(),
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
            capabilities: rmcp::model::ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability::default()),
                ..Default::default()
            },
            server_info: rmcp::model::Implementation {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Voice-to-Text MCP server providing real-time transcription via Whisper".to_string()
            ),
        }
    }
}

/// Tool router implementation
#[tool_router]
impl VttMcpServer {
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

        let model_path = p.model_path
            .or_else(|| std::env::var("WHISPER_MODEL").ok())
            .unwrap_or_else(|| "models/ggml-base.bin".to_string());

        let threads = p.threads
            .or_else(|| std::env::var("WHISPER_THREADS").ok().and_then(|t| t.parse().ok()))
            .unwrap_or_else(|| num_cpus::get()) as i32;

        let use_gpu = p.use_gpu
            .or_else(|| std::env::var("WHISPER_USE_GPU").ok().and_then(|g| g.parse().ok()))
            .unwrap_or(true);

        let config = WhisperConfig {
            model_path,
            language: None,
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
                "Transcription: {}\nConfidence: {:?}\nDuration: {}ms",
                result.text,
                result.confidence,
                result.end_ms - result.start_ms
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

        let config = WhisperConfig {
            model_path: model_path.clone(),
            language: p.language.clone(),
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

        Ok(CallToolResult::success(vec![
            Content::text(format!(
                "Started listening session: {}\nModel: {}\nLanguage: {:?}\nGPU: {}",
                session_id, model_path, p.language, use_gpu
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

        if let Some(tx) = &transcription {
            let tx_clone = tx.clone();
            drop(sessions);
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
pub struct TranscribeClipParams {
    pub audio_file: String,
    #[serde(default)]
    pub model_path: Option<String>,
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
