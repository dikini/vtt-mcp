### /home/dikini/Projects/vtt-mcp/specs/task-csh.3.1.md
```markdown
1: # Task Specification: vtt-mcp-csh.3.1
2: ## Setup MCP Server Scaffold
3: 
4: **Parent:** vtt-mcp-csh.3 (Implement MCP Server with Transcription Tools)
5: **Estimate:** ~2h
6: **Status:** IN PROGRESS - Troubleshooting rmcp tool macro
7: 
8: ### Description
9: Add rmcp SDK to vtt-mcp/Cargo.toml. Setup stdio transport. Implement basic MCP server struct with tool registration. Test with echo tool.
10: 
11: ### Implementation Status
12: 
13: #### ✅ Completed:
14: - [x] Rust upgraded to 1.92.0 (from 1.86.0)
15: - [x] Dependencies added to Cargo.toml (rmcp 0.12, tokio, serde, tracing, uuid, chrono)
16: - [x] Basic server structure implemented
17: - [x] ServerHandler trait implemented
18: - [x] stdio transport setup in main.rs
19: 
20: #### ⚠️ In Progress:
21: - [ ] Tool registration - having issues with \`#[tool]\` macro
22: - [ ] Echo tool implementation - trait bound error with \`IntoToolRoute\`
23: 
24: ### Current Issue
25: **Error**: `the trait bound '(Tool, fn(&..., ...) -> ... {...::echo}): IntoToolRoute<..., _>' is not satisfied`
26: 
27: **Attempted Solutions**:
28: 1. ✅ Upgraded Rust from 1.86.0 → 1.92.0
29: 2. ✅ Added rmcp with features ["server", "macros"]
30: 3. ✅ Imported `tool` macro explicitly
31: 4. ✅ Matched README example pattern
32: 5. ✅ Tried returning both `Result<CallToolResult, McpError>` and `CallToolResult`
33: 
34: **Next Steps**:
35: - Check if rmcp version mismatch
36: - Look at rmcp source code for IntoToolRoute trait
37: - Consider filing issue or looking for working examples
38: 
39: ### Files Created
40: - `crates/vtt-mcp/Cargo.toml` - Updated dependencies
41: - `crates/vtt-mcp/src/lib.rs` - Library exports
42: - `crates/vtt-mcp/src/server.rs` - Main MCP server (IN PROGRESS)
43: - `crates/vtt-mcp/src/tools/mod.rs` - Tools module (placeholder)
44: - `crates/vtt-mcp/src/tools/echo.rs` - Echo tool placeholder
45: - `crates/vtt-mcp/src/main.rs` - Binary entry point
46: 
47: ### Notes
48: - rmcp v0.12.0 appears to have complex tool macro requirements
49: - May need to look at actual rmcp examples if they exist
50: - Consider alternative: implement MCP protocol manually if rmcp proves too complex
```


## Investigation Results (2025-12-23)

### Root Cause Identified

The IntoToolRoute trait error was caused by multiple issues:

1. **Incorrect parameter pattern**: Tool functions must use Parameters<T> wrapper
2. **Missing dependency features**: Required transport-io feature
3. **Version mismatch**: schemars version must match rmcp's (1.1.0)

### Resolution

All issues resolved. The MCP server scaffold now compiles successfully with:
- Working #[tool] and #[tool_router] macros
- Echo tool with proper parameter handling
- ServerHandler implementation with get_info
- stdio transport for MCP communication

### Test Results

running 2 tests
test tests::test_version ... ok
test server::tests::test_server_creation ... ok

test result: ok. 2 passed; 0 failed

See docs/RMCP_INVESTIGATION.md for detailed analysis.

### Status

✅ COMPLETE - Server scaffold functional, ready for next subtask.
