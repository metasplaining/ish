# Phase 3: AGENTS.md Rewrite + Symlinks

*Part of: [agent-infrastructure/overview.md](overview.md)*

## Context Files

- [context/agents-md-content-spec.md](context/agents-md-content-spec.md) — full content specification for the new AGENTS.md (10 sections, ordering, issues, tech stack data)

## Requirements

- `AGENTS.md` contains the ten sections specified in the content spec, in order, with all content requirements met.
- `AGENTS.md` is ≤ 500 lines (soft limit; stop at natural completion, do not pad).
- `CLAUDE.md` is a symlink to `AGENTS.md`.
- `.github/copilot-instructions.md` is a symlink to `../AGENTS.md`.
- Reading `CLAUDE.md` or `.github/copilot-instructions.md` produces the same content as `AGENTS.md`.

## Tasks

- [x] 1. Read current `AGENTS.md` and `CLAUDE.md` in full — `AGENTS.md`, `CLAUDE.md`

- [x] 2. Write new `AGENTS.md` following the content specification in `context/agents-md-content-spec.md` — `AGENTS.md`

  Sections in order:
  1. Build & Test — keep current
  2. Never Touch — new section (5 items)
  3. Project Stage Rule — new section (1-sentence rule, mark as prototype-phase)
  4. Project Structure — keep current table
  5. Key Concepts — keep current
  6. Tech Stack — new section (Rust edition, Tokio, pest, Reedline; see context file for version data)
  7. Prototype Crate Map — keep current
  8. Proposal Process — new section (merge CLAUDE.md content: process, authority order, implementation discipline, resuming implementation)
  9. Task Playbooks — update to 10-skill table (see context file for table); keep existing non-skill task rows
  10. Conventions — keep current; add two CONTRIBUTING.md reference lines

  Apply "Would the agent make a mistake without this?" to every line. Prune freely.
  Count lines. If over 500, prune further.

- [x] 3. Replace `CLAUDE.md` with a symlink to `AGENTS.md` — `CLAUDE.md`

  ```bash
  rm CLAUDE.md
  ln -s AGENTS.md CLAUDE.md
  ```

- [x] 4. Replace `.github/copilot-instructions.md` with a symlink to `../AGENTS.md` — `.github/copilot-instructions.md`

  ```bash
  rm .github/copilot-instructions.md
  ln -s ../AGENTS.md .github/copilot-instructions.md
  ```

- [x] 5. Verify symlinks resolve to AGENTS.md content — (verification only)

## Verification

Run: `wc -l AGENTS.md`
Check: line count is ≤ 500.

Run: `head -5 CLAUDE.md`
Check: outputs the first lines of the new AGENTS.md (confirms CLAUDE.md symlink works).

Run: `head -5 .github/copilot-instructions.md`
Check: outputs the same first lines (confirms copilot-instructions.md symlink works).

Run: `ls -la CLAUDE.md .github/copilot-instructions.md`
Check: both show `lrwxrwxrwx` symlink entries with correct targets.

Run: `grep -n "Never Touch\|Project Stage\|Tech Stack\|Proposal Process\|update-agents" AGENTS.md`
Check: all five terms appear, confirming the new sections are present.

Invoke: `/verify agent-infrastructure/phase-3.md`
