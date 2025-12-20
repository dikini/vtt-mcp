# AGENTS.md

## Task tracking (Beads)

This repository uses **Beads** (`bd`) as the canonical task / issue tracker.

**Rules for humans and agents (including goose):**

- Use `bd` for all task tracking instead of maintaining separate TODO lists.
- Prefer JSON output for agent workflows where available: use `--json`.
- When you make any Beads changes (create/update/close/dep/link), **run `bd sync` before ending the session**.
  - This follows Beads upstream agent workflow guidance (avoids debounce + git sync surprises).

## goose-specific guidance

- goose should use the Beads-oriented Goose skills in `./.goose/skills/` when responding to requests about tasks, blockers, priorities, or PR preparation.
- For PR workflows, goose may use the GitHub CLI (`gh`) if it is available and authenticated.

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
