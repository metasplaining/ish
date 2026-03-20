---
title: "AI Guide: Orientation"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-19
depends-on: [GLOSSARY.md, docs/spec/INDEX.md, docs/architecture/INDEX.md, CONTRIBUTING.md]
---

# AI Agent Orientation

This document provides the minimum context an AI coding agent needs to work effectively on the ish project.

## What Is ish?

ish is a general-purpose programming language organized around a **low-assurance ↔ high-assurance continuum**. Every language feature can be used in a low-assurance (fewer constraints, faster to write) or high-assurance (more constraints, safer) mode. The programmer chooses per-feature.

## Key Concepts

| Concept | Definition | Spec |
|---------|-----------|------|
| Assurance level | The degree of constraint applied to a feature | [assurance-ledger.md](../spec/assurance-ledger.md) |
| Low-assurance | Minimal constraints; dynamic-language-like | [assurance-ledger.md](../spec/assurance-ledger.md) |
| High-assurance | Maximum constraints; static-language-like | [assurance-ledger.md](../spec/assurance-ledger.md) |
| Standard | A named configuration that governs what the ledger tracks within its scope | [assurance-ledger.md](../spec/assurance-ledger.md) |
| Entry | A recorded fact about a specific item in the assurance ledger | [assurance-ledger.md](../spec/assurance-ledger.md) |
| Value entry | Type information on values: actual-value, possible-values, allowed-values | [types.md](../spec/types.md), [assurance-ledger.md](../spec/assurance-ledger.md) |
| Open/Closed | Object openness: closed (exact properties) or open (extra allowed) | [types.md](../spec/types.md) |
| Feature state | Two dimensions: `type_annotations` (optional/required) and `type_audit` (runtime/build) | [assurance-ledger.md](../spec/assurance-ledger.md) |
| Execution configuration | Runtime profiles (dev/test/prod) that enforce assurance levels | [execution.md](../spec/execution.md) |
| Polymorphism | Structural typing by default; nominal typing via explicit entry | [polymorphism.md](../spec/polymorphism.md) |
| Error model | Entry-based: Error/CodedError/SystemError hierarchy, throw audit, domain subtypes | [errors.md](../spec/errors.md) |

See [GLOSSARY.md](../../GLOSSARY.md) for the full glossary.

## Project Status

ish is in the **design + prototype** phase. The language specification is incomplete — many features are described conceptually but not yet formalized. The prototype in `proto/` implements a subset.

### What Exists

- **Specification documents** in `docs/spec/` — authoritative but incomplete
- **Prototype** in `proto/` — Rust across 7 crates (parser, AST, VM, stdlib, codegen, runtime, shell)
- **Architecture docs** in `docs/architecture/` — describes the prototype's design
- **PEG parser** (`ish-parser`) — pest grammar, parser-matches-everything philosophy
- **Standard library** (`ish-stdlib`) — self-hosted analyzer, Rust generator, stdlib programs
- **Assurance ledger runtime** — standards, entry types, audit logic in `ish-vm/src/ledger/`

### What Does Not Exist Yet

- Package management
- Build-time (pre-audit) type checking
- Entry-based error model (specification complete; runtime implementation in progress)

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
| Design proposals | `docs/project/proposals/` |
| Implementation plans | `docs/project/plans/` |
| Design history | `docs/project/history/` |
| How to contribute | `CONTRIBUTING.md` |
| Proposal process | `.github/copilot-instructions.md`, `.github/skills/` |

## Proposal Process

Non-trivial changes follow a three-document lifecycle:

1. **RFP** (`docs/project/rfp/`) — Cleaned-up request from the human.
2. **Design Proposal** (`docs/project/proposals/`) — Iterative design with alternatives and decisions.
3. **Implementation Plan** (`docs/project/plans/`) — Consolidated TODO list, the single source of truth during implementation.

Six skills support this lifecycle: `/propose`, `/revise`, `/accept`, `/plan-implementation`, `/implement`, `/audit`. See the skill files in `.github/skills/` for procedures. Authority order (the sequence in which artifacts are updated during implementation) is documented in `CONTRIBUTING.md` and `.github/copilot-instructions.md`.

## Common AI Tasks

See the playbooks for workflow-specific guidance:
- [Low-assurance code](playbook-low-assurance.md)
- [High-assurance code](playbook-high-assurance.md)
- [Mixed-mode code](playbook-mixed.md)
- [Common patterns](patterns.md)
- [Antipatterns to avoid](antipatterns.md)

---

## Naming Conventions

ish uses these naming conventions consistently:

| Kind | Convention | Examples |
|------|-----------|----------|
| Variables, functions, modules, standards | `snake_case` | `get_user`, `is_type`, `std::io`, `cautious` |
| Types, entry types | `PascalCase` | `String`, `Person`, `Error`, `CodedError`, `Mutable` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_PORT` |
| Keywords | `lowercase` | `let`, `fn`, `if`, `mut` |

See the [naming conventions specification](../spec/syntax.md#naming-conventions) for full details.

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [AGENTS.md](../../AGENTS.md)
