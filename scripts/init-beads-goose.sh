#!/usr/bin/env bash
set -euo pipefail

# init-beads-goose.sh
#
# Best-effort bootstrap for Beads + Goose workflows.
# Installs (where possible):
#   - git, curl (base requirements)
#   - gh (GitHub CLI) for PR workflows
#   - bd (Beads CLI) for task tracking
# Optionally:
#   - Python venv + beads-mcp (only if you plan to use beads MCP tooling)
#
# By default, this script installs CLIs into a repo-local bin directory:
#   ./.tooling/bin
# This avoids system-wide changes and makes it easier to keep tool versions
# consistent per-repo.
#
# Usage:
#   ./scripts/init-beads-goose.sh
#   ./scripts/init-beads-goose.sh --user        # install into ~/.local/bin
#   ./scripts/init-beads-goose.sh --with-beads-mcp
#   ./scripts/init-beads-goose.sh --init-beads
#

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

INSTALL_MODE="local"  # local | user
WITH_BEADS_MCP=false
INIT_BEADS=false

usage() {
  cat <<'USAGE'
Usage: scripts/init-beads-goose.sh [options]

Options:
  --user             Install CLIs into ~/.local/bin instead of ./.tooling/bin
  --with-beads-mcp   Create ./.venv and install beads-mcp via pip
  --init-beads       Run 'bd init' in repo root (if not already initialized)
  -h, --help         Show help

Examples:
  ./scripts/init-beads-goose.sh --init-beads
  ./scripts/init-beads-goose.sh --with-beads-mcp
  ./scripts/init-beads-goose.sh --user --init-beads
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --user) INSTALL_MODE="user"; shift ;;
    --with-beads-mcp) WITH_BEADS_MCP=true; shift ;;
    --init-beads) INIT_BEADS=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

log() { printf "\n[%s] %s\n" "$(date +'%H:%M:%S')" "$*"; }
warn() { printf "\n[WARN] %s\n" "$*" >&2; }

command_exists() { command -v "$1" >/dev/null 2>&1; }

detect_pm() {
  if command_exists apt-get; then echo "apt"
  elif command_exists dnf; then echo "dnf"
  elif command_exists yum; then echo "yum"
  elif command_exists pacman; then echo "pacman"
  elif command_exists brew; then echo "brew"
  else echo "unknown"
  fi
}

PM="$(detect_pm)"
SUDO=""
if command_exists sudo; then SUDO="sudo"; fi

if [[ "$INSTALL_MODE" == "local" ]]; then
  PREFIX="$ROOT_DIR/.tooling"
  BIN_DIR="$PREFIX/bin"
else
  PREFIX="$HOME/.local"
  BIN_DIR="$PREFIX/bin"
fi

mkdir -p "$BIN_DIR"

ensure_base_tools() {
  log "Ensuring base tools (git, curl) exist"

  if command_exists git && command_exists curl; then
    return 0
  fi

  case "$PM" in
    apt)
      $SUDO apt-get update -y || true
      $SUDO apt-get install -y git curl ca-certificates || true
      ;;
    dnf)
      $SUDO dnf install -y git curl ca-certificates || true
      ;;
    yum)
      $SUDO yum install -y git curl ca-certificates || true
      ;;
    pacman)
      $SUDO pacman -Sy --noconfirm git curl ca-certificates || true
      ;;
    brew)
      brew install git curl || true
      ;;
    *)
      warn "Unknown package manager; install git and curl manually if missing."
      ;;
  esac
}

install_gh() {
  if command_exists gh; then
    log "gh already installed: $(gh --version | head -n 1)"
    return 0
  fi

  log "Installing GitHub CLI (gh)"
  case "$PM" in
    apt)
      $SUDO apt-get update -y || true
      $SUDO apt-get install -y gh || true
      ;;
    dnf)
      $SUDO dnf install -y gh || true
      ;;
    yum)
      $SUDO yum install -y gh || true
      ;;
    pacman)
      $SUDO pacman -Sy --noconfirm github-cli || true
      ;;
    brew)
      brew install gh || true
      ;;
    *)
      warn "Unknown package manager; install gh manually: https://github.com/cli/cli#installation"
      ;;
  esac

  if command_exists gh; then
    log "gh installed: $(gh --version | head -n 1)"
  else
    warn "gh not found after install attempt. PR automation skills will be limited."
  fi
}

install_go_if_missing_hint() {
  if command_exists go; then
    return 0
  fi

  warn "Go is not installed. If you install Go, this script can install bd reproducibly into $BIN_DIR via 'go install'."
  warn "Otherwise, we will attempt the upstream curl installer for bd."
}

install_bd_with_go() {
  if ! command_exists go; then
    return 1
  fi

  log "Installing bd with 'go install' into $BIN_DIR"
  GOBIN="$BIN_DIR" go install github.com/steveyegge/beads/cmd/bd@latest || return 1
  return 0
}

install_bd_with_curl_script() {
  log "Attempting bd install via upstream install.sh (may install globally)"
  curl -fsSL https://raw.githubusercontent.com/steveyegge/beads/main/scripts/install.sh | bash || true
}

install_bd() {
  if command_exists bd; then
    log "bd already installed: $(bd --version 2>/dev/null || echo '(version unavailable)')"
    return 0
  fi

  log "Installing Beads CLI (bd)"
  install_go_if_missing_hint

  if install_bd_with_go; then
    log "bd installed at: $BIN_DIR/bd"
  else
    warn "Could not install bd via Go. Falling back to upstream installer."
    install_bd_with_curl_script
  fi

  if command_exists bd; then
    log "bd available on PATH"
  elif [[ -x "$BIN_DIR/bd" ]]; then
    warn "bd installed at $BIN_DIR/bd but not on your PATH yet."
  else
    warn "bd not found. Install options: https://github.com/steveyegge/beads#installation"
  fi
}

maybe_init_beads_repo() {
  if ! $INIT_BEADS; then
    return 0
  fi

  if [[ -d "$ROOT_DIR/.beads" ]]; then
    log ".beads already exists; skipping 'bd init'"
    return 0
  fi

  if [[ ! -d "$ROOT_DIR/.git" ]]; then
    warn "No .git directory found at repo root; skipping 'bd init'."
    return 0
  fi

  if ! command_exists bd; then
    warn "bd is not available; cannot run 'bd init'."
    return 0
  fi

  log "Initializing Beads in repo root"
  (cd "$ROOT_DIR" && bd init) || true
}

maybe_install_beads_mcp() {
  if ! $WITH_BEADS_MCP; then
    return 0
  fi

  log "Setting up Python venv and installing beads-mcp"
  if ! command_exists python3; then
    warn "python3 not found; skipping beads-mcp. Install python3 and rerun with --with-beads-mcp."
    return 0
  fi

  VENV_DIR="$ROOT_DIR/.venv"
  if [[ ! -d "$VENV_DIR" ]]; then
    python3 -m venv "$VENV_DIR" || true
  fi

  if [[ -f "$VENV_DIR/bin/activate" ]]; then
    # shellcheck disable=SC1091
    source "$VENV_DIR/bin/activate"
    python -m pip install --upgrade pip || true
    python -m pip install beads-mcp || true
    log "beads-mcp installed into $VENV_DIR"
  else
    warn "Failed to create venv at $VENV_DIR; skipping beads-mcp."
  fi
}

print_path_hint() {
  if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    cat <<EOF

PATH setup:
  Add this to your shell rc (~/.bashrc, ~/.zshrc, etc.) to use locally installed tools:

    export PATH="$BIN_DIR:\$PATH"

EOF
  fi
}

main() {
  log "Repo root: $ROOT_DIR"
  log "Install mode: $INSTALL_MODE"
  log "Bin dir: $BIN_DIR"

  ensure_base_tools
  install_gh
  install_bd
  maybe_init_beads_repo
  maybe_install_beads_mcp
  print_path_hint

  cat <<'EOF'
Next steps:
  1) If you installed gh, authenticate:
       gh auth login

  2) Verify Beads:
       bd ready --json   (or bd ready)

  3) If you plan to use the Beads recommended workflow for multi-agent repos:
       bd hooks install
EOF
}

main
