# Task Specification: vtt-mcp-csh.3.6
## Add error handling and logging

**Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
**Estimate:** ~2h
**Status:** ✅ COMPLETE

### Description
Setup tracing/tracing-subscriber. Define VttError enum hierarchy. Add error propagation to MCP responses. Implement graceful failure modes.

---

## Implementation Status: ✅ COMPLETE

All requirements for this task were implemented as part of the server implementation.

### ✅ Completed Components:

#### 1. Tracing/Tracing-Subscriber Setup

**Location:** `crates/vtt-mcp/src/main.rs`

```rust
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,vtt_mcp=debug,vtt_core=debug"));
    
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_line_number(true)
        )
        .with(env_filter)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "Starting VTT MCP server"
    );
    // ...
}
```

**Features:**
- Environment variable support (`RUST_LOG`)
- Structured logging with target, line numbers
- Default log level: info (vtt_mcp and vtt_core at debug)
- Error logging on server startup failure

#### 2. VttError Enum Hierarchy

**Location:** `crates/vtt-mcp/src/error.rs`

Complete error hierarchy with 11 variants:

```rust
#[derive(Error, Debug)]
pub enum VttError {
    #[error("Audio error: {0}")]
    Audio(#[from] vtt_core::audio::AudioError),

    #[error("Transcription error: {0}")]
    Transcription(#[from] vtt_core::whisper::WhisperError),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Audio device not found: {0}")]
    DeviceNotFound(String),

    #[error("No audio data available: {0}")]
    NoAudioData(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Audio file error: {0}")]
    AudioFile(#[from] hound::Error),
}
```

**Error Categories:**
- **Client Errors (4xx):** InvalidParameters, DeviceNotFound, NoAudioData
- **Server Errors (5xx):** Internal, Model, Transcription, Audio, Configuration
- **System Errors:** Io, AudioFile (automatically converted)

**Helper Methods:**
```rust
impl VttError {
    pub fn session(msg: impl Into<String>) -> Self
    pub fn invalid_params(msg: impl Into<String>) -> Self
    pub fn internal(msg: impl Into<String>) -> Self
    pub fn device_not_found(device: impl Into<String>) -> Self
    pub fn model(msg: impl Into<String>) -> Self
    
    pub fn error_code(&self) -> &'static str
    pub fn is_client_error(&self) -> bool
}
```

#### 3. Error Propagation to MCP Responses

**Location:** `crates/vtt-mcp/src/error.rs`

```rust
pub trait ToMcpError: Sized {
    fn with_context(self, context: &str) -> rmcp::model::ErrorData;
}

impl ToMcpError for VttError {
    fn with_context(self, context: &str) -> rmcp::model::ErrorData {
        match &self {
            VttError::InvalidParameters(msg) => {
                tracing::warn!(context, error = %msg, "Invalid parameters");
            }
            VttError::DeviceNotFound(device) => {
                tracing::warn!(context, device = %device, "Device not found");
            }
            VttError::Session(msg) => {
                tracing::warn!(context, error = %msg, "Session error");
            }
            _ => {
                tracing::error!(context, error = %self, "Operation failed");
            }
        }
        
        let message = format!("{}: {}", context, self);
        match self {
            VttError::InvalidParameters(_) | 
            VttError::DeviceNotFound(_) | 
            VttError::NoAudioData(_) => {
                rmcp::model::ErrorData::invalid_params(message, None)
            }
            _ => {
                rmcp::model::ErrorData::internal_error(message, None)
            }
        }
    }
}
```

**Features:**
- Automatic log level selection (warn for client errors, error for server errors)
- Structured logging with context and error details
- Proper MCP error code mapping (invalid_params vs internal_error)
- From<VttError> for rmcp::model::ErrorData auto-conversion

#### 4. Graceful Failure Modes

**Client vs Server Error Distinction:**

```rust
impl VttError {
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            VttError::InvalidParameters(_)
                | VttError::DeviceNotFound(_)
                | VttError::NoAudioData(_)
        )
    }
}
```

**Error Recovery Strategies:**

| Error Type | HTTP Code | Recovery Strategy |
|------------|-----------|-------------------|
| InvalidParameters | 400 | Client should fix request |
| DeviceNotFound | 404 | Client should select different device |
| NoAudioData | 400 | Client should provide valid audio |
| Model | 500 | Server-side: check model path |
| Transcription | 500 | Server-side: retry with different model |
| Audio | 500 | Server-side: check audio system |
| Io | 500 | Server-side: check file permissions |

**Usage in Server:**
```rust
// Example: transcribe_clip with proper error handling
async fn transcribe_clip_impl(&self, params: TranscribeClipParams) -> VttResult<TranscribeClipResult> {
    let path = Path::new(&params.audio_file);
    if !path.exists() {
        return Err(VttError::invalid_params(format!(
            "Audio file not found: {}",
            params.audio_file
        )));
    }

    let reader = WavReader::open(&path)
        .map_err(|e| VttError::AudioFile(e))?;
    
    // ... transcription logic ...
    
    let ctx = WhisperContext::new(config)
        .map_err(|e: vtt_core::whisper::WhisperError| VttError::Model(e.to_string()))?;
    
    let transcription = ctx.transcribe(&samples, 16000)
        .map_err(|e| VttError::Transcription(e))?;
    
    // ...
}
```

---

## Dependencies

**Already in `crates/vtt-mcp/Cargo.toml`:**
```toml
thiserror = "1.0"      # Error derive macros
anyhow = "1.0"         # Result type for main()
tracing = "0.1"        # Logging facade
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

---

## Tests

**Location:** `crates/vtt-mcp/src/error.rs` (module tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(VttError::invalid_params("test").error_code(), "INVALID_PARAMS");
    }

    #[test]
    fn test_client_error_detection() {
        assert!(VttError::invalid_params("test").is_client_error());
        assert!(!VttError::internal("test").is_client_error());
    }
}
```

**Test Results:**
```bash
$ cargo test --package vtt-mcp --lib

running 2 tests
test error::tests::test_error_codes ... ok
test error::tests::test_client_error_detection ... ok

test result: ok. 2 passed; 0 failed
```

---

## Usage Examples

### Basic Error Handling

```rust
use crate::error::{VttError, VttResult};

fn validate_audio_file(path: &str) -> VttResult<PathBuf> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(VttError::invalid_params(format!(
            "Audio file not found: {}",
            path.display()
        )));
    }
    Ok(path)
}
```

### Error Propagation

```rust
// Automatic conversion with ?
let result = operation_that_fails()
    .map_err(|e| VttError::internal(e.to_string()))?;
```

### MCP Error Conversion

```rust
// Convert VttError to MCP ErrorData
let result: VttResult<TranscriptionResult> = transcribe_audio().await;
let mcp_result: Result<TranscriptionResult, rmcp::model::ErrorData> = 
    result.map_err(|e| e.into());
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level filter | `info,vtt_mcp=debug,vtt_core=debug` |
| `WHISPER_MODEL` | Model path | `models/ggml-base.bin` |
| `WHISPER_THREADS` | Thread count | `<physical cores>` |

**Examples:**
```bash
# Debug logging
RUST_LOG=debug cargo run --package vtt-mcp

# Only errors
RUST_LOG=error cargo run --package vtt-mcp

# Very verbose for vtt-mcp only
RUST_LOG=vtt_mcp=trace cargo run --package vtt-mcp
```

---

## Files Modified/Created

### Modified (as part of server implementation):
- `crates/vtt-mcp/src/main.rs` - Added tracing initialization
- `crates/vtt-mcp/src/error.rs` - Complete error hierarchy
- `crates/vtt-mcp/Cargo.toml` - Dependencies added

### No Additional Files Needed:
All error handling and logging infrastructure was implemented as part of the core server implementation.

---

## Design Decisions

### 1. Structured Logging
- **Decision:** Use tracing instead of log
- **Rationale:** Better async support, structured fields, spans

### 2. Error Hierarchy
- **Decision:** Flat enum with 11 variants
- **Rationale:** Simpler than nested enums, easier to match

### 3. Client vs Server Errors
- **Decision:** Distinguish via `is_client_error()`
- **Rationale:** Maps to HTTP 4xx vs 5xx semantics

### 4. Automatic Conversions
- **Decision:** Use `#[from]` for common errors
- **Rationale:** Reduce boilerplate with `?` operator

---

## Status

✅ **COMPLETE** - All requirements met

**Completed:**
- [x] Setup tracing/tracing-subscriber
- [x] Define VttError enum hierarchy  
- [x] Add error propagation to MCP responses
- [x] Implement graceful failure modes
- [x] Add error handling tests
- [x] Document error codes and recovery

**Integration:**
- Error handling integrated throughout server.rs
- Logging added to all critical paths
- MCP error conversion working

---

*Generated: 2025-12-24*
*Task: vtt-mcp-csh.3.6*
*Status: ✅ COMPLETE*

