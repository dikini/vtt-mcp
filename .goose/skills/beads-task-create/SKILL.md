---
name: beads-task-create
description: Create a new Beads task (issue) with priority and (optional) type
---

# Beads: create task

## Inputs to confirm
- Title (required)
- Priority (optional; default: 2)
- Type (optional; e.g. task/bug/epic if your bd supports `-t`)

## Procedure
1. Prefer JSON output:

```bash
bd create "<title>" -p <priority> --json
```

2. If you need a type and your bd supports it, use:

```bash
bd create "<title>" -t <type> -p <priority> --json
```

3. Return to the user:
- The new task ID (e.g. `bd-a1b2`)
- A suggested next step (e.g. add deps, show details)

## Notes
- If `--json` is not available, fall back to `bd create "<title>" -p <priority>`.
- Keep titles specific and action-oriented.
