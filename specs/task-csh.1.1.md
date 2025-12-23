# Task Specification: vtt-mcp-csh.1.1

**Task ID**: vtt-mcp-csh.1.1  
**Title**: Setup Cargo workspace structure  
**Parent Task**: vtt-mcp-csh.1 (Project Setup and Audio Capture Pipeline)  
**Status**: open  
**Priority**: 2  
**Estimated Effort**: 30 minutes  
**Created**: 2025-12-23

---

## 1. Task Overview

### Objective
Establish the fundamental Cargo workspace structure for the vtt-mcp project, creating three separate crates (vtt-core, vtt-mcp, vtt-cli) with proper workspace configuration. This foundational structure enables modular development and clear separation of concerns between core functionality, MCP server implementation, and CLI interface.

### Context
- This is the first implementation task in Phase 1 (Project Setup)
- Parent task vtt-mcp-csh.1 focuses on complete audio capture pipeline setup (2-3h)
- This task unblocks vtt-mcp-csh.1.2 (configure dependencies)
- Project uses Rust edition 2021, GPL-3.0 license
- Target architecture: three-crate workspace for modularity and maintainability

---

## 2. Implementation Requirements

### 2.1 Directory Structure
Create the following directory hierarchy:

```
vtt-mcp/                    # Root workspace
├── Cargo.toml              # Workspace configuration (modify existing)
├── crates/                 # New directory for all crates
│   ├── vtt-core/          # Core voice-to-text functionality
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── vtt-mcp/           # MCP server implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── vtt-cli/           # Command-line interface
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
```

### 2.2 Root Workspace Configuration
Modify existing `./Cargo.toml` to configure as workspace:

```toml
[workspace]
members = [
    "crates/vtt-core",
    "crates/vtt-mcp",
    "crates/vtt-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"
authors = ["VTT-MCP Contributors"]
repository = "https://github.com/yourusername/vtt-mcp"

[workspace.dependencies]
# Common dependencies will be added in task vtt-mcp-csh.1.2
# Examples: tokio, serde, anyhow, etc.
```

**Note**: Keep existing `[package]` section if present, or remove it since this will become a pure workspace.

### 2.3 Crate-Specific Cargo.toml Files

#### crates/vtt-core/Cargo.toml
```toml
[package]
name = "vtt-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
# Core dependencies for audio processing, VAD, STT
# Will be populated in vtt-mcp-csh.1.2

[dev-dependencies]
# Test dependencies
```

#### crates/vtt-mcp/Cargo.toml
```toml
[package]
name = "vtt-mcp"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
vtt-core = { path = "../vtt-core" }
# MCP protocol dependencies
# Will be populated in vtt-mcp-csh.1.2

[dev-dependencies]
# Test dependencies
```

#### crates/vtt-cli/Cargo.toml
```toml
[package]
name = "vtt-cli"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "vtt-cli"
path = "src/main.rs"

[dependencies]
vtt-core = { path = "../vtt-core" }
vtt-mcp = { path = "../vtt-mcp" }
# CLI dependencies (clap, etc.)
# Will be populated in vtt-mcp-csh.1.2

[dev-dependencies]
# Test dependencies
```

### 2.4 Source File Templates

#### crates/vtt-core/src/lib.rs
```rust
//! Core voice-to-text functionality
//!
//! This crate provides the fundamental building blocks for voice-to-text
//! processing, including audio capture, voice activity detection, and
//! speech-to-text transcription.

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Core library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
```

#### crates/vtt-mcp/src/lib.rs
```rust
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
        assert!(!VERSION.is_empty());
    }
}
```

#### crates/vtt-cli/src/main.rs
```rust
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
    fn test_basic() {
        assert!(true);
    }
}
```

---

## 3. Acceptance Criteria

All criteria must be satisfied for task completion:

- [ ] Directory `crates/` created at project root
- [ ] Three subdirectories created: `crates/vtt-core/`, `crates/vtt-mcp/`, `crates/vtt-cli/`
- [ ] Root `Cargo.toml` configured with `[workspace]` section
- [ ] Root `Cargo.toml` lists all three crates as workspace members
- [ ] Each crate has its own `Cargo.toml` with correct package name and workspace inheritance
- [ ] `vtt-core` has `src/lib.rs` with documented module structure
- [ ] `vtt-mcp` has `src/lib.rs` with documented module structure
- [ ] `vtt-cli` has `src/main.rs` with basic main function
- [ ] `cargo build` succeeds without errors or warnings
- [ ] `cargo test` passes all tests (including basic template tests)
- [ ] `cargo fmt --all --check` passes (code is formatted)
- [ ] `cargo clippy --all-targets -- -D warnings` passes (no clippy warnings)
- [ ] All changes committed with descriptive message
- [ ] `bd sync` executed to synchronize task status

---

## 4. Quality Gates

Execute these verification steps in order:

### 4.1 Build Verification
```bash
cargo clean
cargo build --workspace
# Should succeed with 0 errors, 0 warnings
```

### 4.2 Test Verification
```bash
cargo test --workspace
# Should pass all tests (at least 3 basic tests from templates)
```

### 4.3 Format Verification
```bash
cargo fmt --all
cargo fmt --all --check
# Should show no files need formatting
```

### 4.4 Lint Verification
```bash
cargo clippy --all-targets --workspace -- -D warnings
# Should succeed with 0 warnings
```

### 4.5 Structure Verification
```bash
# Verify workspace members are recognized
cargo metadata --format-version=1 | jq '.workspace_members'
# Should show all three crates

# Verify crate dependencies
cargo tree --workspace --depth 1
# Should show vtt-cli depends on vtt-core and vtt-mcp
# Should show vtt-mcp depends on vtt-core
```

---

## 5. Technical Constraints

### 5.1 Rust Edition & Standards
- **Edition**: 2021 (specified in workspace)
- **MSRV**: Rust 1.70+ (for edition 2021 features)
- **Warnings**: Treat all warnings as errors during development
- **Documentation**: Enable `#![warn(missing_docs)]` for public APIs
- **Linting**: Enable `#![warn(clippy::all)]` in all crates

### 5.2 Workspace Conventions
- **Crate Location**: All crates under `crates/` directory (not at root)
- **Naming**: Use kebab-case for crate names (`vtt-core`, not `vtt_core`)
- **Version Management**: Use workspace inheritance (`version.workspace = true`)
- **License**: GPL-3.0 for all crates (inherited from workspace)

### 5.3 Code Organization
- **Libraries**: Use `lib.rs` for `vtt-core` and `vtt-mcp`
- **Binary**: Use `main.rs` for `vtt-cli`
- **Tests**: Include basic test module in each source file
- **Documentation**: Include module-level doc comments with `//!`

### 5.4 Git Practices
- **Commit Message**: Use format: "task(csh.1.1): Setup Cargo workspace structure"
- **Commit Scope**: Include all new files (Cargo.toml files, src files, directories)
- **Pre-commit**: Ensure `cargo fmt` and `cargo clippy` pass before committing

---

## 6. Integration Points

### 6.1 Parent Task Integration
- **Parent**: vtt-mcp-csh.1 (Project Setup and Audio Capture Pipeline)
- **Position**: First subtask - establishes foundation for all subsequent work
- **Unblocks**: This task must complete before vtt-mcp-csh.1.2 (configure dependencies)

### 6.2 Downstream Dependencies
Tasks that depend on this workspace structure:
- **vtt-mcp-csh.1.2**: Configure dependencies (cpal, whisper-rs, silero-vad, etc.)
- **vtt-mcp-csh.1.3**: Implement basic audio capture in vtt-core
- **vtt-mcp-csh.1.4**: Setup MCP server scaffolding in vtt-mcp

### 6.3 Architecture Alignment
- **Phase 1**: This task is part of Phase 1 (Project Setup) in PLAN.md
- **Modularity**: Three-crate structure enables:
  - Core logic isolation (vtt-core)
  - MCP protocol separation (vtt-mcp)
  - Independent CLI tool (vtt-cli)
- **Future Growth**: Structure supports adding more crates later (e.g., vtt-wasm, vtt-gui)

### 6.4 Tool Integration
- **Beads**: Task tracked with `bd` - must run `bd sync` after completion
- **Git**: Changes committed and synced with remote
- **CI/CD**: Workspace structure must support future GitHub Actions workflows

---

## 7. Success Metrics

### 7.1 Setup Efficiency
- **Target**: Complete setup in <10 minutes
- **Measure**: Time from starting implementation to passing all quality gates
- **Success**: All files created, builds succeed, tests pass on first attempt

### 7.2 Build Quality
- **Target**: Zero warnings, zero errors
- **Measure**: Output of `cargo build --workspace`
- **Success**: Clean build with no warnings or errors

### 7.3 Test Coverage
- **Target**: Basic test in each crate passes
- **Measure**: Output of `cargo test --workspace`
- **Success**: At least 3 tests pass (one per crate)

### 7.4 Code Quality
- **Target**: Zero clippy warnings
- **Measure**: Output of `cargo clippy --all-targets --workspace -- -D warnings`
- **Success**: No warnings or errors from clippy

---

## 8. Implementation Workflow

### Step 1: Create Directory Structure
```bash
mkdir -p crates/vtt-core/src
mkdir -p crates/vtt-mcp/src
mkdir -p crates/vtt-cli/src
```

### Step 2: Create/Update Root Cargo.toml
Back up existing Cargo.toml if needed, then update with workspace configuration.

### Step 3: Create Crate Cargo.toml Files
Create the three crate-specific Cargo.toml files as specified in section 2.3.

### Step 4: Create Source Files
Create lib.rs and main.rs files as specified in section 2.4.

### Step 5: Verify Build
```bash
cargo build --workspace
cargo test --workspace
```

### Step 6: Apply Formatting & Linting
```bash
cargo fmt --all
cargo clippy --all-targets --workspace -- -D warnings
```

### Step 7: Commit Changes
```bash
git add crates/ Cargo.toml
git commit -m "task(csh.1.1): Setup Cargo workspace structure"
```

### Step 8: Sync Task Status
```bash
bd close vtt-mcp-csh.1.1
bd sync
git push
```

---

## 9. Troubleshooting Guide

### Issue: "cargo build" fails with workspace errors
**Symptom**: Error about workspace members not found  
**Cause**: Incorrect paths in workspace members list  
**Fix**: Verify paths in root Cargo.toml `[workspace.members]` match actual directories

### Issue: Crate dependencies not resolved
**Symptom**: Cannot find `vtt-core` when building `vtt-mcp`  
**Cause**: Incorrect path dependencies  
**Fix**: Verify `path = "../vtt-core"` is correct relative path

### Issue: Clippy warnings about unused code
**Symptom**: Warnings about unused VERSION constants  
**Cause**: Template code doesn't use all items yet  
**Fix**: Add `#[allow(dead_code)]` temporarily or use items in tests

### Issue: Missing workspace inheritance
**Symptom**: Version not found in crate Cargo.toml  
**Cause**: `[workspace.package]` not defined in root  
**Fix**: Add `[workspace.package]` section with version, edition, license

---

## 10. References

### Internal Documentation
- **PLAN.md**: Section 10.1 (Three-Crate Architecture), Section 10.3 (Development Workflow)
- **AGENTS.md**: Task tracking rules, Beads workflow, session completion requirements
- **README.md**: Project overview and goals

### External Resources
- [Cargo Workspaces Documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Rust Edition Guide - 2021](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
- [Workspace Inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table)

### Related Tasks
- **Parent**: vtt-mcp-csh.1 (Project Setup and Audio Capture Pipeline)
- **Next**: vtt-mcp-csh.1.2 (Configure dependencies)
- **Phase**: Phase 1 - Project Setup

---

## 11. Notes for Agents

### Goose Agent Guidelines
1. **Read this spec completely** before starting implementation
2. **Follow the workflow** in section 8 step-by-step
3. **Run quality gates** in section 4 in order
4. **Do not skip** `bd sync` at completion (see AGENTS.md)
5. **Commit and push** all changes before ending session

### Key Decision Points
- **Existing Cargo.toml**: Check if root Cargo.toml has `[package]` section - if yes, decide whether to keep it or migrate to pure workspace
- **Git History**: Preserve existing git history when moving files if needed
- **Workspace Resolver**: Use resolver "2" for edition 2021 compatibility

### Validation Checklist
Before marking task complete, verify:
- [ ] Can run `cargo build` from root successfully
- [ ] Can run `cargo test` from root successfully  
- [ ] Can run `cargo run --bin vtt-cli` successfully
- [ ] Git status shows all files committed
- [ ] `bd show vtt-mcp-csh.1.1` shows status as closed
- [ ] `git push` succeeds (MANDATORY per AGENTS.md)

---

**Spec Version**: 1.0  
**Last Updated**: 2025-12-23  
**Agent-Ready**: ✓ Yes
