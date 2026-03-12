---
title: "History: Documentation System"
category: history
audience: [human-dev, ai-agent]
status: stable
last-verified: 2026-03-11
depends-on: [docs/project/decisions/001-documentation-structure.md]
---

# 2026-03-10: Documentation System

The story of how ish got its documentation infrastructure — from a flat pile of Markdown files to a structured, AI-friendly documentation system.

## Where We Started

Before this work, the ish project's documentation was a collection of root-level Markdown files that had grown organically alongside the language design. The language specification lived in files like `TYPES.md`, `MODULES.md`, `REASONING.md`, `AGREEMENT.md`, and `EXECUTION_CONFIGURATIONS.md`. Open questions and unresolved design issues were scattered across companion `*_TODO.md` files (`TYPES_TODO.md`, `MODULES_TODO.md`, etc.) with no cross-references between them. The project overview was embedded in `README.md`, and the entire prototype architecture was described in a single `proto/ARCHITECTURE.md` file.

This was workable when the project was small, but it had become a problem. More than 10 files cluttered the root directory, there was no consistent structure or metadata, and — critically — AI coding agents had no efficient way to navigate the documentation. An agent working on the type system had to guess which files were relevant, load them all, and hope the context window could hold everything.

## The Prompt

The human developer created a `DOCUMENTATION.md` file that laid out 10 kinds of documentation the project needed:

1. A GitHub summary for newcomers
2. A human developer user guide
3. An AI developer user guide with playbooks
4. A language specification
5. An architecture specification
6. Contributor documentation
7. Development history
8. Organized open questions and TODOs
9. A roadmap and project plan
10. Maturity/implementation status tracking

The file also posed several questions: Is context management really critical for AI productivity? What are the best practices for documenting for agentic AIs? Are there additional kinds of documentation beyond the 10 listed?

## The Proposal

The AI developer researched the questions and produced `DOCUMENTATION_PROPOSAL.md` — a comprehensive proposal organized into 9 parts, with 14 decision points marked `[DECISION]` for the human to annotate. The proposal went well beyond answering the original questions; it included a directory layout, file format conventions, a cross-referencing strategy, per-audience writing styles, a migration plan from the existing files, and a layered context-loading strategy specifically designed for AI agents.

### Additional Document Types

The AI proposed six additional documentation types beyond the original 10:

| # | Kind | Purpose |
|---|------|---------|
| 11 | **Glossary** | Define ish-specific terms — critical for AI agents where ambiguous terms cause misinterpretation |
| 12 | **Error catalog** | Enumerate every error with explanations and remediation |
| 13 | **Decision log (ADRs)** | Concise records of each design decision so AI agents don't re-propose rejected ideas |
| 14 | **AI agent instructions** | Machine-targeted file (`AGENTS.md`) with build commands, task playbooks, and conventions |
| 15 | **FAQ / Troubleshooting** | Common problems and solutions to preempt incorrect AI-generated solutions |
| 16 | **Comparison guide** | How ish compares to other languages |

The human's response: *"All of them."*

## The Decision Exchange

What follows is the evolution of the key decisions, showing what was proposed and how the human shaped the outcome.

### Directory Layout

The AI proposed a topic-first directory structure (`docs/spec/`, `docs/architecture/`, `docs/user-guide/`, `docs/ai-guide/`, `docs/project/`, `docs/errors/`) with INDEX files in every directory. Three alternatives were considered and rejected:

- **Flat structure** — keeping all docs in the root. Rejected because it was already not scaling with 10+ files.
- **Audience-first split** (`docs/humans/`, `docs/agents/`) — rejected because most spec and architecture content serves both audiences.
- **Single-file-per-topic with audience sections** — rejected because it makes files too long, which is bad for AI context budgets.

The human accepted the proposed layout and added a requirement: *"Each sub-project and each feature/component sub-directory in a sub-project should have a README.md, limited to 500 lines, that summarizes that project, feature, or component and cross-links back to the relevant files in the main project docs."*

### YAML Frontmatter

The AI proposed YAML frontmatter on every file with fields for `title`, `category`, `audience`, `status`, `last-verified`, and `depends-on`. The alternative was no frontmatter, relying on directory structure alone for categorization. The human chose frontmatter — it enables tooling to select documents by audience, track review status, and verify dependency consistency.

### Backward References

The proposal asked whether backward references (a "Referenced by" section listing files that depend on the current file) should be required, optional, or auto-generated. The human chose *required*. This means every file must manually maintain its list of dependents, which is more effort but ensures the dependency graph is always visible without tooling.

### Open Questions: The Hybrid Approach

This was one of the more nuanced decisions. Three approaches to handling open questions were considered:

- **Option A: Consolidated file only.** All questions in `docs/project/open-questions.md`. Easy to prioritize, but questions lose their context — you can't see the relevant spec while reading the question.
- **Option B: Adjacent only.** Each spec file has an `## Open Questions` section. Context is preserved, but there's no aggregate view for prioritization.
- **Option C: Hybrid.** Questions appear in *both* places, cross-linked in both directions.

The human chose Option C with a specific elaboration: the open questions section in each spec file links to the corresponding topic in the consolidated file, and vice versa. The questions themselves are duplicated in both locations. This is a deliberate violation of the single-source-of-truth principle, accepted because the two audiences (AI agents reading a spec file, humans reviewing the open questions list) need the information in different places.

### Documentation Debt

The original proposal included documentation debt tracking as part of the open questions file. The human split this out: *"Documentation debt should go in a separate file rather than being part of open questions."* This became `docs/project/documentation-debt.md`.

### Change Protocol

The AI proposed a 5-step change protocol (identify affected docs → update them → update status → update dates → update maturity). The human added two requirements: *"The change protocol should include adding a history file and updating the history index."* This became steps 6 and 7 of the protocol.

### History File Naming

The proposal's conventions for history files included "one file per significant conversation" with date, participants, and summary. The human added: *"History files should be named `<isodate>-<topic>.md` for easy sorting."*

### Migration Mapping

The AI proposed migrating all existing root-level files into the new structure, including merging `proto/README.md` into the architecture overview. The human adjusted two things:

1. *"proto/README.md should stay where it is"* — it serves as the sub-project summary and shouldn't be absorbed into the architecture docs.
2. *"DOCUMENTATION.md should become the first history file"* — rather than archiving or deleting it, preserve it as a record of how the documentation system was conceived.

### Everything Else

The human accepted the remaining proposals without modification:
- File size limits (500 lines soft limit, one topic per file)
- Writing style guidelines (conversational for humans, dense for AI, formal for spec, technical for architecture, prescriptive for contributors)
- Document statuses (draft / review / stable — no deprecated or superseded)
- Redundancy policy (zero for normative claims, permitted for summaries and examples)
- Audit tooling (link checker, frontmatter validator, stale doc detector, glossary checker, and AI-assisted audits — all of them)
- Migration phasing (5 phases: infrastructure → spec docs → architecture docs → new content → tooling)
- Per-document-type conventions (spec, architecture, user guide, AI guide, ADRs, history)

## Implementation

The implementation followed the 5-phase migration plan. Phase 1 created the directory structure, root files (`AGENTS.md`, `GLOSSARY.md`, `CONTRIBUTING.md`), and INDEX files. Phase 2 moved the specification files into `docs/spec/`, merging the `*_TODO.md` files into `## Open Questions` sections. Phase 3 split `proto/ARCHITECTURE.md` into per-module files under `docs/architecture/`. Phase 4 generated the user guide, AI guide, roadmap, and maturity assessment. Phase 5 implemented the audit scripts.

The formal decision was recorded as [ADR-001: Documentation Structure](../decisions/001-documentation-structure.md).

## What Was Deferred

Several items from the proposal were not implemented immediately:

- **FAQ / Troubleshooting** (doc type 15) — acknowledged as valuable but deferred until there are enough common problems to document.
- **Error catalog content** — the `docs/errors/` directory and INDEX were created, but no actual error entries were written. The prototype's error handling is still evolving.
- **Example validator** and **coverage checker** — acknowledged as useful automation but rated as medium/hard feasibility and deferred.

## Participants

- **Human developer:** Authored the original `DOCUMENTATION.md` requirements. Reviewed the proposal and made decisions at every `[DECISION]` point, shaping the outcome in several important ways — requiring sub-project READMEs, choosing required backward references over auto-generated ones, designing the hybrid open questions approach, separating documentation debt from open questions, adding history file requirements to the change protocol, and redirecting the migration to preserve `proto/README.md` in place.

- **AI developer:** Researched documentation best practices for AI-first projects, surveyed emerging conventions from Cursor/Copilot/Aider/Claude Code ecosystems, authored the 9-part proposal with 14 decision points, and implemented the full migration after decisions were made.

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
