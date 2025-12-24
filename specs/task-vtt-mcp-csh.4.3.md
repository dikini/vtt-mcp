# Task: Implement transcript://live Resource

**Task ID:** vtt-mcp-csh.4.3
**Status:** 🚧 In Progress - Design Complete, Implementation Started
**Phase:** 4 - Latency and Performance Optimization

## Description

Add MCP resource for real-time streaming transcript updates. Implement subscription mechanism to push incremental transcription updates to clients as they arrive, enabling true real-time feedback without polling.

## Background

### MCP Resources vs Tools

**Tools (Current):** Pull-based, request/response
```rust
client.call_tool("get_last_transcription") -> result
// Client must poll repeatedly
```

**Resources (Target):** Push-based, streaming
```rust
client.subscribe("transcript://live/{session_id}")
// Server pushes updates automatically as they arrive
```

### Why Resources?

- ✅ **Real-time**: Updates pushed immediately (no polling delay)
- ✅ **Efficient**: No repeated requests from client
- ✅ **MCP-compliant**: Uses standard protocol features
- ✅ **Scalable**: Broadcast to multiple subscribers

## Design

### Architecture

```
┌─────────────────┐
│  MCP Client     │ subscribe("transcript://live/abc-123")
│  (Goose)        │────────────────────────────────────┐
└─────────────────┘                                    │
                                                       │
┌─────────────────────────────────────────────────────▼──────────┐
│  VttMcpServer                                                │
│  ┌─────────────────────────────────────────────────────┐      │
│  │  subscribers: HashMap<SessionId, Vec<Subscriber>>    │      │
│  └─────────────────────────────────────────────────────┘      │
│                              │                                │
│  ┌─────────────────────────────────────────────────────┐      │
│  │  transcription_tx: broadcast::Sender<Update>        │      │
│  └─────────────────────────────────────────────────────┘      │
│                              │                                │
│  ┌─────────────────────────────────────────────────────┐      │
│  │  Audio Thread → Whisper → Transcription             │      │
│  └──────────────────┬──────────────────────────────────┘      │
│                     │broadcast(update)                         │
└─────────────────────┼─────────────────────────────────────────┘
                      │
                      │ push notification
                      ▼
              [Client receives update in real-time]
```

### Resource URIs

- **Live Stream**: `transcript://live/{session_id}`
  - Available for active listening sessions
  - Returns real-time transcription updates
  
- **Future**: `transcript://history`
  - Read-only access to historical transcripts
  - Not implemented in this task

### Data Structures

```rust
/// Track subscribers for each session
subscribers: Arc<Mutex<HashMap<Uuid, Vec<SessionSubscriber>>>>

/// A single subscriber to a session
pub struct SessionSubscriber {
    client_id: String,
    subscribed_at: DateTime<Utc>,
}

/// Transcription update broadcast to subscribers
pub struct TranscriptionUpdate {
    session_id: Uuid,
    text: String,
    is_final: bool,
    timestamp: DateTime<Utc>,
    confidence: Option<f32>,
}

/// Broadcast channel for updates
transcription_tx: broadcast::Sender<TranscriptionUpdate>
```

### MCP Resource Methods

```rust
impl ServerHandler for VttMcpServer {
    // List available resources (active sessions)
    fn list_resources() -> Result<ListResourcesResult, McpError> {
        // Returns: [transcript://live/uuid1, transcript://live/uuid2, ...]
    }
    
    // Subscribe to session updates
    fn subscribe(uri) -> Result<(), McpError> {
        // Parse session_id from URI
        // Add client to subscribers list
        // Start receiving broadcast updates
    }
    
    // Unsubscribe from updates
    fn unsubscribe(uri) -> Result<(), McpError> {
        // Remove client from subscribers list
    }
    
    // Read current state (optional, for compatibility)
    fn read_resource(uri) -> Result<ReadResourceResult, McpError> {
        // Return current transcription text
    }
}
```

## Implementation Plan

### Phase 1: Add Subscription State ✅
- [x] Design completed
- [ ] Add `subscribers` field to `VttMcpServer`
- [ ] Add `transcription_tx` broadcast channel
- [ ] Create `TranscriptionUpdate` struct

### Phase 2: Implement Resource Handlers
- [ ] Override `list_resources()` to expose active sessions
- [ ] Implement `subscribe()` with URI parsing
- [ ] Implement `unsubscribe()` to clean up
- [ ] Optional: `read_resource()` for current state

### Phase 3: Integrate with Transcription Loop
- [ ] Modify audio processing to broadcast updates
- [ ] Send updates via `transcription_tx`
- [ ] Handle subscriber errors gracefully

### Phase 4: Enable Resources Capability
- [ ] Update `get_info()` to advertise resources
- [ ] Add `ResourcesCapability` to server info

### Phase 5: Testing
- [ ] Unit tests for subscription management
- [ ] Integration test for single subscriber
- [ ] Integration test for multiple subscribers
- [ ] Test cleanup on unsubscribe

## Acceptance Criteria

1. **Resource Discovery**
   - `list_resources()` returns active listening sessions
   - URIs format: `transcript://live/{uuid}`

2. **Subscription**
   - Client can subscribe to session via `subscribe()`
   - Subscription persists until `unsubscribe()`

3. **Real-time Updates**
   - New transcription pushed immediately to subscribers
   - Updates contain: text, timestamp, is_final flag

4. **Multiple Subscribers**
   - Multiple clients can subscribe to same session
   - All receive identical updates

5. **Cleanup**
   - Unsubscribe removes client from list
   - Session end cleans up all subscribers

## Testing Strategy

### Unit Tests

1. **Subscription Management**
   ```rust
   #[tokio::test]
   async fn test_add_subscriber() {
       let server = VttMcpServer::new();
       server.add_subscriber(session_id, client_id).await;
       assert!(server.has_subscribers(session_id).await);
   }
   ```

2. **URI Parsing**
   ```rust
   #[test]
   fn test_parse_session_id() {
       let uri = "transcript://live/abc-123-def-456";
       let session_id = parse_session_id(uri).unwrap();
       assert_eq!(session_id, Uuid::parse_str("abc-123-def-456").unwrap());
   }
   ```

### Integration Tests

1. **Single Subscriber Flow**
   - Start listening session
   - Subscribe to resource
   - Generate audio input
   - Verify transcription update received

2. **Multiple Subscribers**
   - Two clients subscribe to same session
   - Generate audio
   - Verify both receive updates

3. **Unsubscribe Behavior**
   - Subscribe → receive updates → unsubscribe
   - Verify no more updates received

## Dependencies

**None required** - rmcp 0.12 includes resource support.

However, we'll use:
- `tokio::sync::broadcast` - For fanout updates to subscribers
- Existing types: `Arc<Mutex<>>`, `Uuid`, `DateTime<Utc>`

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| **Subscription leak** | Auto-cleanup after 5 min inactivity |
| **Channel overflow** | Use bounded channel (100 capacity) |
| **Client disconnect** | Handle send errors, remove dead subscribers |
| **Memory growth** | Prune inactive sessions periodically |

## Future Enhancements

- [ ] `transcript://history` - Read historical transcripts
- [ ] Resource templates for dynamic session creation
- [ ] Per-client filtering (language, confidence threshold)
- [ ] Webhook support for external integrations

## Progress

### Completed
- ✅ Design and architecture
- ✅ Data structures defined
- ✅ Implementation plan

### In Progress
- 🚧 Adding subscription state to VttMcpServer

### Remaining
- ⏭️ Implement resource handlers
- ⏭️ Integrate with transcription loop
- ⏭️ Testing

