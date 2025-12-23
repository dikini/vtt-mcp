#!/bin/bash
# Beads task creation script - Phase 5

echo "=== Creating Phase 5: Polish & Distribution tasks ==="

bd create --title "GPU detection and auto-configuration"   --description "Detect CUDA/ROCm availability at runtime. Auto-select appropriate Whisper build. Provide clear error messages for missing GPU support. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Model download automation"   --description "Implement automatic Whisper model download on first run. Show progress bar. Verify checksums. Handle download failures gracefully. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Create installation script"   --description "Write install.sh for automated setup. Check dependencies (Rust, GPU drivers). Create config directory structure. Set up systemd service (optional). ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Build .deb package for Debian/Ubuntu"   --description "Create debian/ directory with control files. Configure package metadata. Setup build process with cargo-deb. Test installation on Ubuntu 22.04+. ~3h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Create AUR package for Arch"   --description "Write PKGBUILD for vtt-mcp. Test build and installation on Arch. Submit to AUR. Document installation steps. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Setup GitHub Actions CI/CD"   --description "Create .github/workflows/ci.yml for tests and linting. Add release.yml for automated builds. Configure artifact uploads. Test release process. ~3h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Profile-guided optimization"   --description "Collect profiles from real usage. Apply PGO to Rust build. Measure performance improvements. Document build flags. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Create demo video and screenshots"   --description "Record asciinema demo of setup and usage. Create video showing Goose integration. Take screenshots for README. Edit and upload to repo. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Prepare release documentation"   --description "Write CHANGELOG.md. Update README with installation options. Create release notes template. Prepare crates.io metadata. ~2h"   --parent vtt-mcp-csh --priority 2 --type task

bd create --title "Publish v1.0.0 release"   --description "Create GitHub release with binaries. Publish to crates.io. Announce on relevant forums/channels. Monitor for issues. ~1h"   --parent vtt-mcp-csh --priority 2 --type task

echo "All Phase 5 tasks created successfully"
