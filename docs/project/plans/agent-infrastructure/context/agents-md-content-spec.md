*Extracted verbatim from [agent-infrastructure.md](../../proposals/agent-infrastructure.md) §Feature: AGENTS.md Rewrite.*

---

## Issues to Watch Out For

- **500-line target**: After merging AGENTS.md (~108 lines) and CLAUDE.md (~46 lines) = ~154 lines. Adding Never Touch, expanded tech stack, architecture index, and updated playbooks may reach 300–400 lines. Padding to 500 is wrong — stop at natural completion.
- **"Would the agent make a mistake without this?"**: Apply this test to every line. Remove content the agent handles correctly without instruction.
- **Backward compatibility rule**: The RFP requested explicitly adding "ignore backward compatibility" to AGENTS.md. This is a project-stage rule (prototype phase), not a permanent principle. Mark it clearly as a prototype-phase rule.
- **All nine skills now known**: Write AGENTS.md with post-migration paths — all nine skills at `.agents/skills/<name>/`.

---

## Content Specification

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

Keep existing non-skill task rows (Adding a new builtin, Adding a new AST node, Modifying the type spec, etc.) unchanged.

**10. Conventions** (keep current content, add two lines)
- Add: "For Rust style guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)."
- Add: "For agent instruction style guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md)."

---

## Tech Stack Source Data

From `proto/Cargo.toml` and individual crate Cargo.toml files:
- Rust edition: 2021 (all crates)
- Tokio: version 1 (workspace), features: rt, time, process, sync; ish-shell adds rt-multi-thread, macros
- pest: ~2.7 (ish-parser/Cargo.toml); grammar file at `proto/ish-parser/src/ish.pest`
- Reedline: 0.46 (ish-shell/Cargo.toml)
- Runtime config: `Runtime::new_current_thread` with `LocalSet` (see ish-shell source)
