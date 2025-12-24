# RMCP Investigation Report

## Problem Statement

The rmcp crate's #[tool] and #[tool_router] macros were failing to compile with the error:
error[E0277]: the trait bound '(...): IntoToolRoute<..., _>' is not satisfied

## Root Cause Analysis

After deep investigation into the rmcp source code (version 0.12.0), several issues were identified:

### 1. Incorrect Function Signature Pattern

The #[tool] macro generates code that wraps async functions to return:
Pin<Box<dyn Future<Output = ReturnType> + Send + '_>>

For the tool function to implement CallToolHandler<S, A>, it must follow specific parameter patterns.

WRONG:
async fn echo(&self, message: String) -> Result<CallToolResult, McpError>

CORRECT:
async fn echo(&self, params: Parameters<EchoParams>) -> Result<CallToolResult, McpError>

The parameters must be wrapped in Parameters<T> which implements FromContextPart trait to extract JSON arguments from the MCP request.

### 2. Macro System Architecture

The rmcp tool macro system works in layers:

Layer 1: #[tool] Attribute
- Generates a _tool_attr() function that returns the Tool metadata
- Modifies the original function to return a boxed Future
- Creates input schema from the Parameters type automatically

Layer 2: #[tool_router] Attribute
- Scans the impl block for functions marked with #[tool]
- Generates a tool_router() function that creates a ToolRouter and calls with_route for each tool

Layer 3: #[tool_handler] Attribute
- Implements ServerHandler trait
- Generates list_tools and call_tool methods
- Delegates to the ToolRouter instance

### 3. Dependencies and Features

Required Features:
- server: Enables server functionality and tool system
- macros: Enables procedural macros
- transport-io: Enables stdio transport (needed for main.rs)

Dependency Version Constraints:
- schemars must match the version used by rmcp (1.1.0)
- Using a different version causes trait implementation failures

## Solution Implemented

### Updated crates/vtt-mcp/Cargo.toml

Added required features and correct schemars version:
rmcp = { version = "0.12", features = ["server", "macros", "transport-io"] }
schemars = "1.1"

### Updated crates/vtt-mcp/src/server.rs

1. Parameters wrapped in Parameters<T>
2. Proper imports for Content, CallToolResult
3. JsonSchema derive on parameter structs
4. ServerHandler implementation with get_info

### Updated crates/vtt-mcp/src/main.rs

1. Added ServiceExt import
2. Use stdio() transport
3. Proper async initialization

## Test Results

running 2 tests
test tests::test_version ... ok
test server::tests::test_server_creation ... ok

test result: ok. 2 passed; 0 failed

The server compiles successfully and is ready for integration testing.

## Key Takeaways

1. Parameter Extraction: Tool parameters are automatically extracted from JSON using the Parameters<T> wrapper
2. Schema Generation: The #[tool] macro automatically generates JSON schemas for both input and output
3. Async Wrapping: The macro wraps async functions to return boxed futures
4. Version Alignment: Dependency versions must align, especially schemars
