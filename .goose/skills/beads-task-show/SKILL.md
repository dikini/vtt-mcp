---
name: beads-task-show
description: Show details for a Beads task by id (including audit trail)
---

# Beads: show task

## When to use
- The user asks for details/status/history for a specific bead id (e.g. `bd-a3f8`)
- You need more context before preparing a PR or adding dependencies

## Procedure
1. Run:

```bash
bd show <id> --json
```

2. Summarize key fields:
- Title
- Status
- Priority
- Blockers / dependencies (if present)
- Any description/notes that affect implementation

## Notes
- If `--json` is not available, fall back to `bd show <id>`.
