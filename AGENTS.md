---
title: ish — Agent Instructions
category: project
audience: [ai-dev]
status: draft
last-verified: 2026-03-10
depends-on: [GLOSSARY.md, CONTRIBUTING.md, docs/INDEX.md]
---

# ish — Agent Instructions

Instructions for AI agents working in this repository.

---

## Build & Test

```bash
cd proto && cargo build --workspace    # Build everything
cd proto && cargo test --workspace     # Run all tests (45 tests)
cd proto && cargo run -p ish-shell     # Run end-to-end demos (6 verifications)
cd proto && bash ish-tests/run_all.sh  # Run acceptance tests (150 tests)
```

---

## Project Structure

| Path | Contents |
|------|----------|
| `docs/spec/` | Language specification (types, modules, reasoning, assurance ledger, execution, memory, polymorphism) |
| `docs/architecture/` | Architecture and internals of the language processor |
| `docs/user-guide/` | User guide for human developers |
| `docs/ai-guide/` | User guide and playbooks for AI developers |
| `docs/project/` | Roadmap, maturity, decisions, history, open questions |
| `docs/errors/` | Error catalog |
| `proto/` | Prototype implementation in Rust (6 crates) |
| `proto/ish-tests/` | Bash acceptance tests (run with `bash ish-tests/run_all.sh`) |

---

## Key Concepts

- See [GLOSSARY.md](GLOSSARY.md) for all terminology.
- ish has a **low-assurance ↔ high-assurance continuum** — see [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md).
- The prototype has a **PEG parser** (`ish-parser` crate) using pest, with a **parser-matches-everything** philosophy — the parser always succeeds, encoding incomplete input as `Incomplete` AST nodes.
- The prototype proves three mechanisms: interpreted execution, compiled execution (AST → Rust → `.so` → dynamic load), and self-hosting (analyzer, generator, stdlib all written as ish programs).
- The shell (`ish-shell`) is a Reedline-based interactive REPL with file/inline execution, multiline input via parser-based validation, and shell command execution.

---

## Prototype Crate Map

| Crate | Purpose |
|-------|---------|
| `ish-ast` | AST node types, builder API, display formatting |
| `ish-parser` | PEG parser (pest), AST builder, parser-matches-everything |
| `ish-vm` | Tree-walking interpreter, GC-managed values, builtins, AST↔Value reflection, shell command execution |
| `ish-stdlib` | Self-hosted analyzer, Rust generator, and standard library (all written as ish programs) |
| `ish-runtime` | Minimal value type shared between interpreter and compiled `.so` files |
| `ish-codegen` | Compilation driver: generates temp Cargo project → `cargo build` → loads `.so` |
| `ish-shell` | Interactive REPL (Reedline), file execution, inline execution |

---

## Task Playbooks

Load only the files you need for the task at hand.

| Task | Load these files |
|------|-----------------|
| Adding a new builtin function | [docs/architecture/vm.md](docs/architecture/vm.md) § Builtins |
| Adding a new AST node | [docs/architecture/ast.md](docs/architecture/ast.md) |
| Modifying the type spec | [docs/spec/types.md](docs/spec/types.md) |
| Working on modules | [docs/spec/modules.md](docs/spec/modules.md) |
| Understanding execution configs | [docs/spec/execution.md](docs/spec/execution.md) |
| Reviewing open questions | [docs/project/open-questions.md](docs/project/open-questions.md) |
| Adding acceptance tests | [proto/ish-tests/lib/test_lib.sh](proto/ish-tests/lib/test_lib.sh), existing tests in `proto/ish-tests/` |
| Understanding a design decision | [docs/project/decisions/INDEX.md](docs/project/decisions/INDEX.md) |

---

## Conventions

- See [CONTRIBUTING.md](CONTRIBUTING.md) for full conventions.
- Use terms as defined in [GLOSSARY.md](GLOSSARY.md) — do not introduce synonyms.
- Every documentation file requires YAML frontmatter and a `## Referenced by` section.
- After code changes, update affected docs and add a history entry.
- Track documentation debt in [docs/project/documentation-debt.md](docs/project/documentation-debt.md).
- When creating documentation with minimal specification, set `status: placeholder`.
- History files are written for humans. Use narrative prose that tells the story of what happened, including the evolution of ideas through proposal/response/decision exchanges. Do not write terse summaries or bullet-point logs.

---

## Referenced by

- [docs/INDEX.md](docs/INDEX.md)
