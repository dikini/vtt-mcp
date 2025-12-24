
# Transcript History Resource Implementation - Summary

## Task: vtt-mcp-csh.7 - Implement transcript history resource

### Status: Partial Implementation

The core infrastructure for transcript history storage has been analyzed and designed,
but not yet committed due to technical challenges with the implementation.

## What Was Attempted

1. **HistoryConfig struct** - Configuration for history limits and file persistence
2. **File persistence** - Save/load history to/from JSON files
3. **Pagination** - get_history_paginated(offset, limit) method
4. **Clear history** - clear_history() method
5. **Configurable limits** - Replace hard-coded limit of 100 entries

## Technical Challenges Encountered

1. **WhisperConfig not serializable** - Cannot include in HistoryEntry for JSON persistence
   - Solution: Remove config field from HistoryEntry struct
   
2. **Text editor adding markdown** - text_editor tool adds backticks and formatting
   - Solution: Use shell commands instead

3. **File duplication** - Multiple sed/python script attempts created duplicate content
   - Solution: Start fresh with single comprehensive script

## Implementation Plan for Next Session

### File Changes Required:

**crates/vtt-mcp/src/server.rs:**

1. Add imports:
   - use std::fs::{File, OpenOptions};
   - use std::io::{BufReader, BufWriter, Write};
   - use std::path::{Path, PathBuf};

2. Add HistoryConfig struct after SessionSubscriber:
   - max_entries: usize (default: 100)
   - persistence_path: Option<PathBuf> (default: None)

3. Add to VttMcpServer struct:
   - history_config: HistoryConfig field
   - Initialize in new() and with_history_config()

4. Modify HistoryEntry:
   - Add #[derive(Clone, Serialize, Deserialize)]
   - Remove config: WhisperConfig field (not serializable)

5. Add methods to VttMcpServer impl:
   - save_history_to_disk()
   - save_history_async()
   - get_history_paginated(offset, limit)
   - get_history_count()
   - clear_history()

6. Update list_resources():
   - Add transcript://history resource to the list

7. Update read_resource():
   - Handle transcript://history URI
   - Parse ?page=N&size=M query parameters
   - Return paginated JSON

8. Add ClearHistoryParams struct and clear_history_tool

### Testing:
- All existing tests pass
- Need tests for pagination
- Need tests for file persistence

## Current State

- Build: PASSING (baseline)
- Tests: PASSING (6 tests)
- Working tree: CLEAN

## Recommendation

Implement this feature in the next session using a single comprehensive Python script
that makes all changes at once, testing incrementally to catch issues early.
