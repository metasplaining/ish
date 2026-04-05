---
title: Contributing to ish
category: project
audience: [contributor]
status: draft
last-verified: 2026-03-10
depends-on: [GLOSSARY.md, docs/INDEX.md]
---

# Contributing to ish

Conventions and practices for contributors to the ish project.

---

## Project Structure

```
ish/
├── README.md              Summary for unfamiliar readers
├── GLOSSARY.md            Terminology definitions
├── CONTRIBUTING.md        This file
├── AGENTS.md              AI agent instructions
├── docs/                  All documentation
│   ├── spec/              Language specification
│   ├── architecture/      Architecture and internals
│   ├── user-guide/        User guide for human developers
│   ├── ai-guide/          User guide for AI developers
│   ├── project/           Roadmap, maturity, decisions, history, rfp, proposals, plans
│   └── errors/            Error catalog
└── proto/                 Prototype implementation (Rust)
```

See [docs/INDEX.md](docs/INDEX.md) for the full documentation map.

---

## Documentation Conventions

### File format

All documentation files use Markdown with YAML frontmatter:

```markdown
---
title: <Document Title>
category: user-guide | ai-guide | spec | architecture | project
audience: [all | human-dev | ai-dev | contributor]
status: placeholder | draft | review | stable
last-verified: YYYY-MM-DD
depends-on: [list of files this doc references for its claims]
---

# <Document Title>

<content>
```

### Size and scope

- **Maximum file length:** 500 lines (soft limit). Split if exceeded.
- **One topic per file.** Each file covers exactly one concept or component.
- **Self-contained sections.** Each major section should be understandable without reading the rest of the file.
- **No orphan files.** Every file must be linked from its parent INDEX.md.

### Cross-referencing

- Every fact lives in exactly one file. Other files link to it.
- Use relative markdown links: `[Type System](docs/spec/types.md)`.
- Link to specific sections when relevant: `[Literal Types](docs/spec/types.md#literal-types)`.
- Every file must have a `## Referenced by` section at the bottom listing files that depend on it.

### Writing style by audience

| Audience | Style |
|----------|-------|
| User guide (human) | Conversational, progressive disclosure, lots of examples, explain *why* not just *what* |
| User guide (AI) | Dense, structured, exhaustive, every rule explicit and unambiguous, examples for every pattern |
| Specification | Formal, precise, normative language ("shall", "must", "may"), no handwaving |
| Architecture | Technical, references code by file path, shows data flow |
| Contributor | Prescriptive, imperative ("Do X", "Never Y"), concrete examples |

### Document statuses

| Status | Meaning |
|--------|---------|
| `placeholder` | Generated without sufficient specification. Non-normative — content may be entirely hallucinated. Must be reviewed and rewritten before promotion to `draft`. |
| `draft` | Incomplete or unreviewed. May contain placeholders or known inaccuracies. |
| `review` | Believed complete but not verified against the current implementation. |
| `stable` | Verified against the current implementation and believed accurate. |

### Open questions

Each specification file has an `## Open Questions` section at the bottom. These are also indexed in [docs/project/open-questions.md](docs/project/open-questions.md) organized by topic. Cross-links go both ways.

### Requests for Proposal (RFPs)

When a prompt file is used to generate a design proposal via the `/propose` skill, the prompt is first converted into a **Request for Proposal (RFP)**. The RFP is a cleaned-up version of the original prompt with corrected grammar, formatting, and typos, but with all original meanings preserved.

- RFPs are saved to `docs/project/rfp/` with a meaningful filename.
- RFPs use standard YAML frontmatter with `category: rfp`.
- The RFP index at `docs/project/rfp/INDEX.md` tracks all RFPs.
- Design proposals reference their RFP, not the original prompt file.

### Proposals and Plans

Non-trivial changes follow a three-document lifecycle:

1. **RFP** (`docs/project/rfp/`) — Cleaned-up request from the human.
2. **Design Proposal** (`docs/project/proposals/`) — Analysis with alternatives, decisions, and recommendations. Iterates via complete replacement until accepted.
3. **Implementation Plan** (`docs/project/plans/`) — Consolidated TODO list derived from the accepted design proposal. The single source of truth during implementation.

The agent decides whether an RFP needs a design proposal or can proceed directly to an implementation plan.

During the design phase, proposals iterate using these rules:
- Each new version is a **complete replacement**, not a diff.
- Every design proposal includes a **decision register** at the top — the authoritative record of all decisions.
- The `/revise` skill updates the decision register first, then rewrites the body to be consistent.
- After generating or revising a proposal, the agent scans the entire document for contradictions (self-consistency check).
- Prior versions are preserved in the proposal's **design history directory** under `docs/project/history/`.

### Authority Order

When implementing a feature, update project artifacts in this sequence so that any artifact read during implementation reflects the current truth:

1. **Glossary** — New terms must exist before any document references them.
2. **Roadmap** — Feature status set to "in progress."
3. **Maturity matrix** — Update affected rows to reflect current state.
4. **Specification** — The normative definition of behavior.
5. **Architecture** — How the spec is realized in the codebase.
6. **User guide / AI guide** — How to use the feature.
7. **Agent documentation** — Skills, copilot-instructions.md, AGENTS.md.
8. **Acceptance tests** — Tests that define "done."
9. **Code** — Implementation.
10. **Unit tests** — Tests for internal correctness.
11. **Roadmap** — Feature status set to "completed."
12. **Maturity matrix** — Update affected rows to reflect final state.
13. **History** — Narrative of the work.
14. **Index files** — Updated last.

Authority order is enforced by agent instructions only, not by tooling.

---

## Change Protocol

After every incremental change to the system:

1. **Identify affected documents** using `depends-on` frontmatter or by searching for references to the changed component.
2. **Update affected documents** to reflect the new state.
3. **Update status.** If a `stable` document was modified, its status reverts to `review`.
4. **Update `last-verified` date** once confirmed accurate.
5. **Update the maturity document** ([docs/project/maturity.md](docs/project/maturity.md)) if the change affects what is implemented.
6. **Add a history entry** under `docs/project/history/`. For proposals, create a directory named `<isodate>-<topic>/` containing a `summary.md` (narrative of how the proposal evolved) and separate version files (`v1.md`, `v2.md`, etc.). For non-proposal changes, create a single file named `<isodate>-<topic>.md`. Update the [history index](docs/project/history/INDEX.md). History entries are written for a human audience — use narrative prose that tells the story of what changed and why, showing the proposal → feedback → decision flow where applicable. Do not write terse agent-style summaries.
7. **Track documentation debt** in [docs/project/documentation-debt.md](docs/project/documentation-debt.md) if docs cannot be updated immediately.

---

## Code Conventions

### Rust (prototype)

- Build: `cd proto && cargo build --workspace`
- Test: `cd proto && cargo test --workspace`
- Run demos: `cd proto && cargo run -p ish-shell`
- Acceptance tests: `cd proto && bash ish-tests/run_all.sh`
- All tests must pass before committing.
- Follow standard Rust formatting (`cargo fmt`).
- Use `Result` types for error handling — do not panic.

### Terminology

Use terms as defined in [GLOSSARY.md](GLOSSARY.md). Do not introduce synonyms.

---

## Agent-Friendly Style Guidelines

(For writing AGENTS.md and SKILL.md files.)

- Use imperative mood: "Read X" not "You should read X."
- Lead with commands, not explanations.
- Prefer tables and checklists over prose.
- Every instruction must be actionable.
- No ambiguous pronouns.
- File paths must be explicit and relative to the repo root.
- Commands must be copy-pasteable.

---

## Rust Style Guidelines

- Write idiomatic Rust. Follow the Rust API Guidelines where applicable.
- Use `?` for error propagation. Avoid `unwrap()` in non-test code.
- Prefer `match` over long `if let` chains.
- Use `#[derive(...)]` liberally for standard traits.
- No `unsafe` without explicit justification in a comment.
- This project is in the prototype stage — do not add backward-compatibility shims.

---

## Referenced by

- [AGENTS.md](AGENTS.md)
- [docs/INDEX.md](docs/INDEX.md)
