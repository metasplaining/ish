---
title: "ADR-001: Documentation Structure"
category: decision
audience: [human-dev, ai-agent]
status: stable
last-verified: 2026-03-10
depends-on: []
---

# ADR-001: Documentation Structure

## Status

Accepted

## Context

The ish project's documentation was a collection of root-level Markdown files (`TYPES.md`, `MODULES.md`, `REASONING.md`, `AGREEMENT.md`, `EXECUTION_CONFIGURATIONS.md`, etc.) without consistent structure, cross-linking, or conventions. Open questions and TODO items were scattered across `*_TODO.md` files without cross-references.

The project needed a documentation system that would:
1. Scale as the language design evolves
2. Be navigable by both humans and AI coding agents
3. Support layered context loading (not everything at once)
4. Track open questions, decisions, and documentation debt
5. Maintain single-source-of-truth for normative claims

## Decision

We adopted a structured documentation system with these key choices:

### Directory Layout

```
docs/
  spec/           # Normative language specification
  architecture/   # Prototype design and internals
  user-guide/     # Human-readable tutorials and guides
  ai-guide/       # AI agent orientation and playbooks
  project/        # Roadmap, decisions, history, open questions
    decisions/    # ADRs (this directory)
    history/      # Chronological project history
  errors/         # Error catalog (future)
  scripts/        # Documentation audit tooling
  comparison.md   # Language comparison
```

### Conventions

- **YAML frontmatter** on every file (title, category, audience, status, last-verified, depends-on)
- **500-line soft limit** per file
- **Backward references** (`## Referenced by` section) required
- **Hybrid open questions**: questions appear in both the spec file and the consolidated `open-questions.md`, cross-linked both ways
- **Documentation debt** tracked in a separate `documentation-debt.md` file
- **INDEX.md** in every directory listing contents and purpose
- **History files** named `<isodate>-<topic>.md`

### Migration

- Original root-level spec files migrated to `docs/spec/`
- `proto/ARCHITECTURE.md` split into per-module architecture files
- `proto/README.md` kept in place
- `DOCUMENTATION.md` became the first history file
- Sub-project directories get their own README.md files

## Consequences

- All documentation is discoverable via INDEX files
- AI agents can load context in layers (L0→L3) without reading everything
- Open questions are visible both in context (spec files) and in aggregate
- The change protocol includes updating history files
- Audit scripts can verify link integrity, frontmatter, staleness, and glossary usage

## Alternatives Considered

- **Option A**: Open questions only in consolidated file — rejected because questions lose context
- **Option B**: Open questions only in spec files — rejected because there's no aggregate view
- **Wiki-based system** — rejected because it doesn't live with the code
- **No frontmatter** — rejected because it prevents automated validation

---

## Referenced by

- [docs/project/decisions/INDEX.md](INDEX.md)
