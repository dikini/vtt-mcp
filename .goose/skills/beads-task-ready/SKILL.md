---
name: beads-task-ready
description: List Beads tasks that are ready to start (no open blockers)
---

# Beads: list ready tasks

## When to use
Use this skill when the user asks:
- what to work on next
- what tasks are unblocked/ready
- what blockers exist (start by listing ready; then investigate not-ready tasks if needed)

## Procedure
1. Run:

```bash
bd ready --json
```

2. Summarize for the user:
- List task IDs and titles
- Call out priorities if present
- Offer to `bd show <id> --json` for details

## Notes
- Prefer `--json` output for agent parsing.
- If `--json` isn’t supported in the user’s installed bd version, fall back to `bd ready`.
