# Phase 2: File Layout Migration

*Part of: [agent-infrastructure/overview.md](overview.md)*

## Context Files

- [context/agents-md-content-spec.md](context/agents-md-content-spec.md) — lists all nine skills to verify after migration

## Requirements

- `.agents/skills/` exists and contains all nine skill subdirectories.
- `.claude/skills/` is a symlink to `../.agents/skills` (not a regular directory).
- `.github/skills/` is a symlink to `../.agents/skills` (not a regular directory).
- All nine skills resolve correctly when accessed via `.claude/skills/` and `.github/skills/`.

## Tasks

- [x] 1. Create `.agents/skills/` directory — `mkdir .agents/skills`

- [x] 2. Copy the six skills from `.github/skills/` to `.agents/skills/`, preserving subdirectory structure — `.agents/skills/`

  ```bash
  cp -r .github/skills/propose .agents/skills/propose
  cp -r .github/skills/accept .agents/skills/accept
  cp -r .github/skills/audit .agents/skills/audit
  cp -r .github/skills/revise .agents/skills/revise
  cp -r .github/skills/plan-implementation .agents/skills/plan-implementation
  cp -r .github/skills/implement .agents/skills/implement
  ```

- [x] 3. Copy the three skills from `.claude/skills/` to `.agents/skills/` — `.agents/skills/`

  ```bash
  cp -r .claude/skills/bug-fix .agents/skills/bug-fix
  cp -r .claude/skills/troubleshoot .agents/skills/troubleshoot
  cp -r .claude/skills/verify .agents/skills/verify
  ```

- [x] 4. Replace `.claude/skills/` with a symlink to `.agents/skills/` — `.claude/skills`

  ```bash
  rm -rf .claude/skills
  ln -s ../.agents/skills .claude/skills
  ```

- [x] 5. Replace `.github/skills/` with a symlink to `.agents/skills/` — `.github/skills`

  ```bash
  rm -rf .github/skills
  ln -s ../.agents/skills .github/skills
  ```

- [x] 6. Verify all nine skills are present and accessible via both symlinks — (verification only)

  ```bash
  ls .agents/skills/
  ls .claude/skills/
  ls .github/skills/
  ```

  Each listing must show: bug-fix, troubleshoot, verify, propose, accept, audit, revise, plan-implementation, implement.

## Verification

Run:
```bash
ls .agents/skills/ && echo "---" && ls -la .claude/skills && echo "---" && ls -la .github/skills
```
Check:
- First block: nine skill directory names
- Second block: `.claude/skills` is a symlink (`lrwxrwxrwx`) pointing to `../.agents/skills`
- Third block: `.github/skills` is a symlink pointing to `../.agents/skills`

Run: `cat .claude/skills/bug-fix/SKILL.md | head -3`
Check: outputs the first lines of the bug-fix SKILL.md (confirms symlink resolution works).

Run: `cat .github/skills/propose/SKILL.md | head -3`
Check: outputs the first lines of the propose SKILL.md.

Invoke: `/verify agent-infrastructure/phase-2.md`
