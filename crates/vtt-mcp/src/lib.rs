//! MCP (Model Context Protocol) server implementation
//!
//! This crate implements the MCP server that exposes voice-to-text
//! functionality as tools for integration with AI assistants like Goose.

#![warn(missing_docs)]
#![warn(clippy::all)]

/// MCP server library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, "0.1.0");
    }
}
