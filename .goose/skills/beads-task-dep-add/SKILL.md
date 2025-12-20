---
name: beads-task-dep-add
description: Add a dependency relationship between Beads tasks
---

# Beads: add dependency

## Semantics
This skill links two tasks so that one depends on (is blocked by / is a child of) another.

> Note: Beads supports several relationship types; the essential command from the README is:
>
> `bd dep add <child> <parent>`

## Procedure
1. Confirm which task is the child and which is the parent (blocker).
2. Run:

```bash
bd dep add <child-id> <parent-id>
```

3. Optionally verify by showing the child:

```bash
bd show <child-id> --json
```

## Notes
- Follow up with `bd sync` before ending the session if you made changes.
