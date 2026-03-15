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
│   ├── project/           Roadmap, maturity, decisions, history, rfp
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

When a prompt file is used to generate a proposal via the `/propose` skill, the prompt is first converted into a **Request for Proposal (RFP)**. The RFP is a cleaned-up version of the original prompt with corrected grammar, formatting, and typos, but with all original meanings preserved.

- RFPs are saved to `docs/project/rfp/` with a meaningful filename.
- RFPs use standard YAML frontmatter with `category: rfp`.
- The RFP index at `docs/project/rfp/INDEX.md` tracks all RFPs.
- Proposals reference their RFP, not the original prompt file.

---

## Change Protocol

After every incremental change to the system:

1. **Identify affected documents** using `depends-on` frontmatter or by searching for references to the changed component.
2. **Update affected documents** to reflect the new state.
3. **Update status.** If a `stable` document was modified, its status reverts to `review`.
4. **Update `last-verified` date** once confirmed accurate.
5. **Update the maturity document** ([docs/project/maturity.md](docs/project/maturity.md)) if the change affects what is implemented.
6. **Add a history file** under `docs/project/history/` named `<isodate>-<topic>.md` and update the [history index](docs/project/history/INDEX.md). History files are written for a human audience — use narrative prose that tells the story of what changed and why, showing the proposal → feedback → decision flow where applicable. Do not write terse agent-style summaries.
7. **Track documentation debt** in [docs/project/documentation-debt.md](docs/project/documentation-debt.md) if docs cannot be updated immediately.

---

## Code Conventions

### Rust (prototype)

- Build: `cd proto && cargo build --workspace`
- Test: `cd proto && cargo test --workspace`
- Run demos: `cd proto && cargo run -p ish-shell`
- All tests must pass before committing.
- Follow standard Rust formatting (`cargo fmt`).
- Use `Result` types for error handling — do not panic.

### Terminology

Use terms as defined in [GLOSSARY.md](GLOSSARY.md). Do not introduce synonyms.

---

## Referenced by

- [AGENTS.md](AGENTS.md)
- [docs/INDEX.md](docs/INDEX.md)
