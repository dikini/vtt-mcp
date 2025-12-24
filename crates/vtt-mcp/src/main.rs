//! VTT MCP Server binary
//!
//! Entry point for the Voice-to-Text MCP server.

use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use vtt_mcp::VttMcpServer;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with structured logging
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

    // Create and run the server with STDIO transport
    let service = VttMcpServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!(error = %e, "Failed to start server");
        })?;

    tracing::info!("Server listening on STDIO");

    service.waiting().await?;

    Ok(())
}
