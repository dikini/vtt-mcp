//! Command-line interface for vtt-mcp
//!
//! Provides a CLI for interacting with the voice-to-text system,
//! useful for testing and standalone usage.

fn main() {
    println!("vtt-cli v{}", env!("CARGO_PKG_VERSION"));
    println!("Voice-to-text MCP server - CLI interface");
    println!("Run with --help for usage information");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_version_const() {
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(version, "0.1.0");
    }
}
