---
title: Agent Infrastructure
category: proposal
audience: [ai-dev]
status: accepted
last-verified: 2026-04-05
depends-on:
  - docs/project/rfp/agent-instructions-improvements.md
  - AGENTS.md
  - CLAUDE.md
  - .github/copilot-instructions.md
  - CONTRIBUTING.md
---

# Proposal: Agent Infrastructure

*Generated from [agent-instructions-improvements.md](../rfp/agent-instructions-improvements.md) on 2026-04-04.*
*Split from the combined agent-instructions-improvements proposal. The [agent-pipeline-skills](agent-pipeline-skills.md) prerequisite has been implemented (2026-04-04).*
*Accepted 2026-04-05.*

---

## Decision Register

| # | Decision | Outcome |
|---|----------|---------|
| 1 | Canonical file for agent instructions | AGENTS.md is canonical; CLAUDE.md and .github/copilot-instructions.md are symlinks to AGENTS.md |
| 2 | Canonical skills location | `.agents/skills/`; `.claude/skills/` and `.github/skills/` are symlinks |
| 3 | Agent-friendly style guidelines location | CONTRIBUTING.md (existing guidelines extended there) |
| 4 | Rust style guideline location | CONTRIBUTING.md |
| 5 | AGENTS.md maintenance skill behavior | Auto-apply changes, with human confirmation before saving |
| 6 | `/update-agents` behavior when `docs/INDEX.md` is missing | Report the gap to the human and abort |
| 7 | Acceptance test for `/update-agents` missing-skill detection | Omit; no acceptance test required |

---

## Feature: Vendor-Agnostic File Layout

### What Changes

**AGENTS.md** becomes the single canonical agent instruction file, combining the content currently in `AGENTS.md` and `CLAUDE.md`. Both `CLAUDE.md` and `.github/copilot-instructions.md` become symlinks pointing to `AGENTS.md`.

**Skills** move to `.agents/skills/`. Both `.claude/skills/` and `.github/skills/` become symlinks pointing to `.agents/skills/`.

### Issues to Watch Out For

- **Symlink on Windows**: GitHub Copilot and Claude Code read files via the OS filesystem. On Linux/macOS, symlinks are transparent. On Windows, symlinks require developer mode or admin rights. This project runs on Linux — document as a known limitation for Windows contributors.
- **YAML frontmatter**: The merged AGENTS.md will have a single YAML frontmatter block. Claude Code displays project instructions verbatim, including frontmatter. Test whether frontmatter renders acceptably when shown as project instructions — if not, remove it from AGENTS.md.
- **Skill discovery**: `.agents/skills/` is not a path either vendor uses natively — both vendors discover skills via their own vendor-specific directories, which will be symlinks. Verify that both vendors follow directory symlinks after the migration.
- **Claude Code `.claude/skills/` symlink**: Claude Code expects `.claude/skills/<name>/SKILL.md`. If the symlink is a directory symlink (`.claude/skills/ → ../.agents/skills/`), Claude Code must follow it. Test this before removing the old files.
- **In-progress work**: If any agent is executing a plan during migration, the skill files will move. Coordinate the migration with plan completion.

### Current skill inventory (pre-migration)

After agent-pipeline-skills implementation, skills are split across two vendor directories:
- `.github/skills/`: propose, accept, audit, revise, plan-implementation, implement
- `.claude/skills/`: bug-fix, troubleshoot, verify

All nine skills must be present under `.agents/skills/` after migration.

### Implementation

1. Create `.agents/skills/` directory.
2. Copy all skill files from `.claude/skills/` and `.github/skills/` to `.agents/skills/` (preserving the per-skill subdirectory structure).
3. Remove `.claude/skills/` directory and replace with symlink: `.claude/skills → ../.agents/skills`.
4. Remove `.github/skills/` directory and replace with symlink: `.github/skills → ../../.agents/skills`.
5. Verify that `/revise`, `/propose`, `/plan-implementation`, `/implement`, `/accept`, `/audit`, `/bug-fix`, `/troubleshoot`, `/verify` all resolve correctly from both vendor contexts.
6. Write the new AGENTS.md (see below).
7. Replace `CLAUDE.md` with a symlink: `CLAUDE.md → AGENTS.md`.
8. Replace `.github/copilot-instructions.md` with a symlink: `.github/copilot-instructions.md → ../../AGENTS.md`.
9. Verify that both Claude Code and GitHub Copilot load the new AGENTS.md content on session start.

---

## Feature: AGENTS.md Rewrite

### What Changes

The new AGENTS.md combines all content from the current AGENTS.md and CLAUDE.md, adds the "Never Touch" section, expands the tech stack section, adds the no-backward-compatibility rule, and references the full set of nine skills. Target: ~400–500 lines.

Agent-friendly style guidelines and Rust style guidelines are added to CONTRIBUTING.md (not AGENTS.md). AGENTS.md references CONTRIBUTING.md for these.

### Issues to Watch Out For

- **500-line target**: After merging AGENTS.md (~108 lines) and CLAUDE.md (~46 lines) = ~154 lines. Adding Never Touch, expanded tech stack, architecture index, and updated playbooks may reach 300–400 lines. Padding to 500 is wrong — stop at natural completion.
- **"Would the agent make a mistake without this?"**: Apply this test to every line. Remove content the agent handles correctly without instruction.
- **Backward compatibility rule**: The RFP requested explicitly adding "ignore backward compatibility" to AGENTS.md. This is a project-stage rule (prototype phase), not a permanent principle. Mark it clearly as a prototype-phase rule.
- **All nine skills now known**: Write AGENTS.md with post-migration paths — all nine skills at `.agents/skills/<name>/`.

### Content Specification

AGENTS.md must contain the following sections, in this order:

**1. Build & Test** (keep current, already well-done)

**2. Never Touch**
- `proto/target/` — build artifacts
- `Cargo.lock` — unless the task explicitly requires a dependency change
- `.env` or any secrets/credential files
- `.github/workflows/` — unless explicitly in the implementation plan
- Any file not referenced in the current implementation plan

**3. Project Stage Rule**
- This project is in the prototype stage. Do not add backward-compatibility shims, migration paths, or deprecation warnings. Change the code directly.

**4. Project Structure** (keep current table, update paths if needed)

**5. Key Concepts** (keep current)

**6. Tech Stack** (expand)
- Rust edition (current: 2021)
- Tokio: version and runtime config (`Runtime::new_current_thread`, `LocalSet`)
- pest: version and grammar file location
- Reedline: version
- Key crate versions from `proto/Cargo.toml`

**7. Prototype Crate Map** (keep current)

**8. Proposal Process**
- Merge content from CLAUDE.md: RFP → Design Proposal → Implementation Plan → Implementation
- Authority order (12 steps, from CLAUDE.md)
- Implementation discipline (from CLAUDE.md)
- Resuming implementation (from CLAUDE.md)

**9. Task Playbooks** (full ten-skill set)

| Role | Skill |
|------|-------|
| Creating a design proposal | `/propose` — [.agents/skills/propose/SKILL.md](.agents/skills/propose/SKILL.md) |
| Revising a proposal | `/revise` — [.agents/skills/revise/SKILL.md](.agents/skills/revise/SKILL.md) |
| Accepting a proposal | `/accept` — [.agents/skills/accept/SKILL.md](.agents/skills/accept/SKILL.md) |
| Creating an implementation plan | `/plan-implementation` — [.agents/skills/plan-implementation/SKILL.md](.agents/skills/plan-implementation/SKILL.md) |
| Implementing a plan | `/implement` — [.agents/skills/implement/SKILL.md](.agents/skills/implement/SKILL.md) |
| Auditing feature coherence | `/audit` — [.agents/skills/audit/SKILL.md](.agents/skills/audit/SKILL.md) |
| Fixing a bug | `/bug-fix` — [.agents/skills/bug-fix/SKILL.md](.agents/skills/bug-fix/SKILL.md) |
| Troubleshooting | `/troubleshoot` — [.agents/skills/troubleshoot/SKILL.md](.agents/skills/troubleshoot/SKILL.md) |
| Verifying implementation | `/verify` — [.agents/skills/verify/SKILL.md](.agents/skills/verify/SKILL.md) |
| Updating AGENTS.md | `/update-agents` — [.agents/skills/update-agents/SKILL.md](.agents/skills/update-agents/SKILL.md) |

**10. Conventions** (keep current, add reference to CONTRIBUTING.md for style)
- Add: "For Rust style guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)."
- Add: "For agent instruction style guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)."

### Implementation (AGENTS.md write step)

1. Read current AGENTS.md and CLAUDE.md in full.
2. Draft new AGENTS.md following the content specification above.
3. Apply "Would the agent make a mistake without this?" to each line; prune freely.
4. Count lines. If over 500, prune further.
5. Save to AGENTS.md.

---

## Feature: AGENTS.md Maintenance Skill

### What Changes

A new skill `/update-agents` checks AGENTS.md for staleness, broken references, missing skill entries, and line count. It rewrites AGENTS.md and presents the diff for human confirmation before saving.

### Issues to Watch Out For

- **Conservative pruning**: The skill must apply "Would the agent make a mistake without this?" conservatively. When uncertain, keep the line.
- **Line count**: 500 is a target, not a hard limit. The skill flags if exceeded but does not auto-prune to meet the target.
- **Trigger timing**: Run this skill after any change that adds/removes skills, renames crates, or adds new doc sections. Document this in CONTRIBUTING.md.

### Implementation

New skill: `.agents/skills/update-agents/SKILL.md`

Procedure:
1. Read current AGENTS.md.
2. Read `docs/INDEX.md`, the skills directory listing, and `proto/Cargo.toml` (crate list). If `docs/INDEX.md` is missing, report the gap to the human and abort.
3. For each file reference in AGENTS.md, verify the file exists.
4. For each skill in `.agents/skills/`, verify it appears in the Task Playbooks table.
5. For each crate in `proto/Cargo.toml`, verify it appears in the Prototype Crate Map table.
6. Check that the Never Touch list is current.
7. Count lines; note if over 500.
8. For each line, evaluate "Would the agent make a mistake without this?" — flag lines that seem redundant.
9. Produce a proposed replacement AGENTS.md.
10. Present the diff to the human.
11. If the human confirms, save. Otherwise discard.

---

## Feature: CONTRIBUTING.md Extensions

### What Changes

Add two sections to CONTRIBUTING.md:

**Agent-Friendly Style Guidelines** (for writing AGENTS.md and SKILL.md files):
- Use imperative mood: "Read X" not "You should read X."
- Lead with commands, not explanations.
- Prefer tables and checklists over prose.
- Every instruction must be actionable.
- No ambiguous pronouns.
- File paths must be explicit and relative to the repo root.
- Commands must be copy-pasteable.

**Rust Style Guidelines**:
- Write idiomatic Rust. Follow the Rust API Guidelines where applicable.
- Use `?` for error propagation. Avoid `unwrap()` in non-test code.
- Prefer `match` over long `if let` chains.
- Use `#[derive(...)]` liberally for standard traits.
- No `unsafe` without explicit justification in a comment.
- This project is in the prototype stage — do not add backward-compatibility shims.

---

## Documentation Updates

- `AGENTS.md` — major rewrite
- `CLAUDE.md` — replaced with symlink
- `.github/copilot-instructions.md` — replaced with symlink
- `.claude/skills/` — replaced with symlink to `.agents/skills/`
- `.github/skills/` — replaced with symlink to `.agents/skills/`
- `.agents/skills/` (new) — canonical skill location with all nine existing skills
- `.agents/skills/update-agents/SKILL.md` (new) — maintenance skill
- `CONTRIBUTING.md` — add agent style and Rust style sections
- `GLOSSARY.md` — no changes required by this proposal
- `docs/INDEX.md` — update to reference `.agents/skills/`

Update `## Referenced by` sections in: AGENTS.md.
