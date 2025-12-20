---
name: beads-session-sync
description: Flush and sync Beads changes to git (export/commit/pull/import/push)
---

# Beads: session sync

## When to use
Use this at the end of any session where Beads tasks are changed.

This follows Beads upstream agent workflow guidance: `bd sync` forces an immediate flush of pending changes and synchronizes with the remote.

## Procedure
1. Run:

```bash
bd sync
```

2. Confirm:
- command succeeded (exit code 0)
- repo state is clean (`git status` shows no Beads JSONL changes pending)

## Notes
- If your environment requires explicit git push, Beads' `bd sync` should handle it; if it fails, report the failure clearly and stop.
