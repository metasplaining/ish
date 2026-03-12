---
title: "AI Guide: Orientation"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-10
depends-on: [GLOSSARY.md, docs/spec/INDEX.md, docs/architecture/INDEX.md]
---

# AI Agent Orientation

This document provides the minimum context an AI coding agent needs to work effectively on the ish project.

## What Is ish?

ish is a general-purpose programming language organized around a **streamlined ↔ encumbered continuum**. Every language feature can be used in a streamlined (fewer constraints, faster to write) or encumbered (more constraints, safer) mode. The programmer chooses per-feature.

## Key Concepts

| Concept | Definition | Spec |
|---------|-----------|------|
| Encumbrance | The degree of constraint applied to a feature | [agreement.md](../spec/agreement.md) |
| Streamlined | Minimal constraints; dynamic-language-like | [agreement.md](../spec/agreement.md) |
| Encumbered | Maximum constraints; static-language-like | [agreement.md](../spec/agreement.md) |
| Marking | Explicit syntax that moves a feature along the continuum | [agreement.md](../spec/agreement.md) |
| Agreement | The protocol by which code negotiates encumbrance | [agreement.md](../spec/agreement.md) |
| Execution configuration | Runtime profiles (dev/test/prod) that enforce encumbrance levels | [execution.md](../spec/execution.md) |
| Polymorphism | Currently structural typing; nominal typing is encumbered | [polymorphism.md](../spec/polymorphism.md) |

See [GLOSSARY.md](../../GLOSSARY.md) for the full glossary.

## Project Status

ish is in the **design + prototype** phase. The language specification is incomplete — many features are described conceptually but not yet formalized. The prototype in `proto/` implements a subset.

### What Exists

- **Specification documents** in `docs/spec/` — authoritative but incomplete
- **Prototype** in `proto/` — ~5,600 lines of Rust across 6 crates
- **Architecture docs** in `docs/architecture/` — describes the prototype's design

### What Does Not Exist Yet

- A parser for ish syntax (the prototype uses Rust-based AST builders)
- A formal grammar
- Standard library beyond builtins
- Package management
- Error handling design

## Documentation Loading Strategy

When working on ish, load context in layers:

1. **L0 (always)**: This file + [GLOSSARY.md](../../GLOSSARY.md)
2. **L1 (task-specific)**: The relevant spec file(s) and architecture doc(s)
3. **L2 (if needed)**: The relevant user guide for end-user-facing changes
4. **L3 (deep dives)**: Full spec + architecture for cross-cutting changes

## Where to Find Things

| Need | Location |
|------|----------|
| Language rules | `docs/spec/` |
| Prototype design | `docs/architecture/` |
| Open questions | `docs/project/open-questions.md` |
| Decision records | `docs/project/decisions/` |
| How to contribute | `CONTRIBUTING.md` |

## Common AI Tasks

See the playbooks for workflow-specific guidance:
- [Streamlined code](playbook-streamlined.md)
- [Encumbered code](playbook-encumbered.md)
- [Mixed-mode code](playbook-mixed.md)
- [Common patterns](patterns.md)
- [Antipatterns to avoid](antipatterns.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [AGENTS.md](../../AGENTS.md)
