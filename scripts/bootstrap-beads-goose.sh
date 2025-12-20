#!/usr/bin/env bash
set -euo pipefail

# bootstrap-beads-goose.sh
#
# One-shot bootstrap for this repo:
#  1) Installs tools best-effort (gh + bd) via init-beads-goose.sh
#  2) Initializes Beads in this repo (bd init)
#  3) Creates an initial epic bead for this project (optional, best-effort)
#
# Usage:
#   ./scripts/bootstrap-beads-goose.sh
#   ./scripts/bootstrap-beads-goose.sh --user
#   ./scripts/bootstrap-beads-goose.sh --title "My Project Epic"
#

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

INSTALL_MODE="local"  # local | user
EPIC_TITLE="vtt-mcp: initial epic and task graph"
PRIORITY="2"

usage() {
  cat <<'USAGE'
Usage: scripts/bootstrap-beads-goose.sh [options]

Options:
  --user              Install tools into ~/.local/bin (instead of ./.tooling/bin)
  --title "<title>"    Title for initial epic bead (default: repo-derived)
  --priority <0-3>    Priority for initial epic bead (default: 2)
  -h, --help          Show help

Notes:
- This is best-effort; it will not fail the whole script if gh isn't installable.
- It will only create the initial bead if bd is available.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --user) INSTALL_MODE="user"; shift ;;
    --title) EPIC_TITLE="$2"; shift 2 ;;
    --priority) PRIORITY="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 2 ;;
  esac
done

log() { printf "\n[%s] %s\n" "$(date +'%H:%M:%S')" "$*"; }
warn() { printf "\n[WARN] %s\n" "$*" >&2; }

command_exists() { command -v "$1" >/dev/null 2>&1; }

INIT_ARGS=()
if [[ "$INSTALL_MODE" == "user" ]]; then
  INIT_ARGS+=("--user")
fi
INIT_ARGS+=("--init-beads")

main() {
  log "Bootstrapping Beads + Goose tooling in $ROOT_DIR"

  log "Step 1: Install tools (best-effort)"
  (cd "$ROOT_DIR" && ./scripts/init-beads-goose.sh "${INIT_ARGS[@]}") || true

  log "Step 2: Verify bd"
  if ! command_exists bd; then
    # If installed locally but PATH not updated yet, provide a helpful hint.
    if [[ "$INSTALL_MODE" == "local" ]] && [[ -x "$ROOT_DIR/.tooling/bin/bd" ]]; then
      warn "bd installed at ./.tooling/bin/bd but not on PATH. Run:"
      warn "  export PATH=\"$ROOT_DIR/.tooling/bin:\$PATH\""
    fi
    warn "bd is not available; skipping epic task creation."
    exit 0
  fi

  log "Step 3: Create initial epic bead (best-effort)"
  # Prefer --json output; if not supported, fall back.
  set +e
  OUT=$(cd "$ROOT_DIR" && bd create "$EPIC_TITLE" -p "$PRIORITY" --json 2>/dev/null)
  RC=$?
  set -e

  if [[ $RC -ne 0 || -z "$OUT" ]]; then
    warn "bd create --json failed or returned no output; retrying without --json"
    set +e
    OUT=$(cd "$ROOT_DIR" && bd create "$EPIC_TITLE" -p "$PRIORITY" 2>/dev/null)
    RC=$?
    set -e
  fi

  if [[ $RC -ne 0 ]]; then
    warn "Could not create initial bead. You can create one manually with:"
    warn "  bd create \"$EPIC_TITLE\" -p $PRIORITY"
    exit 0
  fi

  log "Created initial bead. Output:"
  echo "$OUT"

  log "Step 4: Recommended next step: sync"
  warn "Beads changes should be synced at the end of the session:"
  warn "  bd sync"
}

main
