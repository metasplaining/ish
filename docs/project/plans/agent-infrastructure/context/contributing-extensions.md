*Extracted verbatim from [agent-infrastructure.md](../../proposals/agent-infrastructure.md) §Feature: CONTRIBUTING.md Extensions.*

---

## What Changes

Add two sections to CONTRIBUTING.md:

---

### Agent-Friendly Style Guidelines

(For writing AGENTS.md and SKILL.md files.)

- Use imperative mood: "Read X" not "You should read X."
- Lead with commands, not explanations.
- Prefer tables and checklists over prose.
- Every instruction must be actionable.
- No ambiguous pronouns.
- File paths must be explicit and relative to the repo root.
- Commands must be copy-pasteable.

---

### Rust Style Guidelines

- Write idiomatic Rust. Follow the Rust API Guidelines where applicable.
- Use `?` for error propagation. Avoid `unwrap()` in non-test code.
- Prefer `match` over long `if let` chains.
- Use `#[derive(...)]` liberally for standard traits.
- No `unsafe` without explicit justification in a comment.
- This project is in the prototype stage — do not add backward-compatibility shims.

---

## Placement in CONTRIBUTING.md

Insert both sections under `## Code Conventions`, after the existing Rust (prototype) subsection. The Rust Style Guidelines go in the Rust (prototype) subsection or after it. The Agent-Friendly Style Guidelines are a new subsection.

Alternatively, add both as top-level `##` sections after `## Code Conventions`. Either placement is acceptable; choose whichever preserves existing document flow.
