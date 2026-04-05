# Phase 5: CONTRIBUTING.md Extensions

*Part of: [agent-infrastructure/overview.md](overview.md)*

## Context Files

- [context/contributing-extensions.md](context/contributing-extensions.md) — verbatim content for both new sections

## Requirements

- `CONTRIBUTING.md` contains an "Agent-Friendly Style Guidelines" section with 7 bullet points.
- `CONTRIBUTING.md` contains a "Rust Style Guidelines" section with 6 bullet points.
- Existing `CONTRIBUTING.md` content is preserved unchanged.

## Tasks

- [x] 1. Add "Agent-Friendly Style Guidelines" and "Rust Style Guidelines" sections to `CONTRIBUTING.md` — `CONTRIBUTING.md`

  Append both sections from `context/contributing-extensions.md` to the `## Code Conventions` area of `CONTRIBUTING.md`. Add them before the `## Referenced by` section.

  The two sections to add (verbatim from context file):

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

## Verification

Run: `grep -n "Agent-Friendly Style\|Rust Style Guidelines" CONTRIBUTING.md`
Check: both headings appear.

Run: `grep -c "imperative mood\|unwrap()" CONTRIBUTING.md`
Check: returns 2 (one match for each section's key bullet).

Invoke: `/verify agent-infrastructure/phase-5.md`
