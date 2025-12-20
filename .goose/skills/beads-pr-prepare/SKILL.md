---
name: beads-pr-prepare
description: Prepare a pull request title/body from a Beads task (optionally using gh)
---

# Beads: prepare PR

## Goal
Given a Beads task id (e.g. `bd-a3f8`), prepare a PR title and markdown body that references the task.

## Inputs
- `bead_id` (required)
- Whether to use `gh` to create/update PR (optional; only if available + authenticated)

## Procedure
1. Fetch task details:

```bash
bd show <bead_id> --json
```

2. Draft PR metadata:
- Title: keep it short; include bead id prefix (e.g. `bd-a3f8: <title>`)
- Body: use the template at:
  - `./.goose/skills/beads-pr-prepare/templates/pr-body.md`

3. If `gh` is available and authorized, offer optional automation:
- Create PR (example):

```bash
gh pr create --title "<title>" --body-file <path-to-generated-body>
```

## Output
- Always return:
  - PR title
  - PR body markdown
- If `gh` automation is used, also return:
  - PR URL

## Notes
- If `--json` isn’t supported, fall back to `bd show <bead_id>` and extract details from text.
- Running `gh` should be explicit: only do it when the user asks to create the PR.
