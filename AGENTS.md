---
title: ish — Agent Instructions
category: project
audience: [ai-dev]
status: draft
last-verified: 2026-04-05
depends-on: [GLOSSARY.md, CONTRIBUTING.md, docs/INDEX.md]
---

# ish — Agent Instructions

Instructions for AI agents working in this repository.

---

## Build & Test

```bash
cd proto && cargo build --workspace    # Build everything
cd proto && cargo test --workspace     # Run all tests (317 tests)
cd proto && cargo run -p ish-shell     # Run end-to-end demos (6 verifications)
cd proto && bash ish-tests/run_all.sh  # Run acceptance tests (255 tests)
```

---

## Never Touch

Do not modify these files unless the task explicitly requires it:

- `proto/target/` — build artifacts
- `Cargo.lock` — unless the task explicitly requires a dependency change
- `.env` or any secrets/credential files
- `.github/workflows/` — unless explicitly in the implementation plan
- Any file not referenced in the current implementation plan

---

## Project Stage Rule

This project is in the prototype stage. Do not add backward-compatibility shims, migration paths, or deprecation warnings. Change the code directly.

---

## Project Structure

| Path | Contents |
|------|----------|
| `docs/spec/` | Language specification (types, modules, reasoning, assurance ledger, execution, memory, polymorphism) |
| `docs/architecture/` | Architecture and internals of the language processor |
| `docs/user-guide/` | User guide for human developers |
| `docs/ai-guide/` | User guide and playbooks for AI developers |
| `docs/project/` | Roadmap, maturity, decisions, history, open questions, proposals, plans |
| `docs/errors/` | Error catalog |
| `proto/` | Prototype implementation in Rust (6 crates) |
| `proto/ish-tests/` | Bash acceptance tests (run with `bash ish-tests/run_all.sh`) |

---

## Key Concepts

- See [GLOSSARY.md](GLOSSARY.md) for all terminology.
- ish has a **low-assurance ↔ high-assurance continuum** — see [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md).
- The project uses a **three-document proposal lifecycle** (RFP → Design Proposal → Implementation Plan) with **authority-ordered execution**. See §Proposal Process below and [CONTRIBUTING.md](CONTRIBUTING.md).
- The prototype has a **PEG parser** (`ish-parser` crate) using pest, with a **parser-matches-everything** philosophy — the parser always succeeds, encoding incomplete input as `Incomplete` AST nodes.
- The prototype proves three mechanisms: interpreted execution, compiled execution (AST → Rust → `.so` → dynamic load), and self-hosting (analyzer, generator, stdlib all written as ish programs).
- The shell (`ish-shell`) is a Reedline-based interactive REPL with file/inline execution, multiline input via parser-based validation, and shell command execution.

---

## Tech Stack

| Component | Detail |
|-----------|--------|
| Rust edition | 2021 (all crates) |
| Tokio | version 1 (workspace); features: `rt`, `time`, `process`, `sync`; `ish-shell` adds `rt-multi-thread`, `macros` |
| Tokio runtime | `Runtime::new_current_thread` with `LocalSet` (see `ish-shell` source) |
| pest | ~2.7; grammar at `proto/ish-parser/src/ish.pest` |
| Reedline | 0.46 |

---

## Prototype Crate Map

| Crate | Purpose |
|-------|---------|
| `ish-core` | Shared types (`TypeAnnotation`) used by both `ish-ast` and `ish-runtime` |
| `ish-ast` | AST node types, builder API, display formatting |
| `ish-parser` | PEG parser (pest), AST builder, parser-matches-everything |
| `ish-vm` | Tree-walking interpreter, Environment, builtins, AST↔Value reflection, shell command execution |
| `ish-stdlib` | Self-hosted analyzer, Rust generator, and standard library (all written as ish programs) |
| `ish-runtime` | Runtime types: Value, Shim, RuntimeError, ErrorCode, IshFunction. Compiled packages depend on this crate |
| `ish-codegen` | Compilation driver: generates temp Cargo project → `cargo build` → loads `.so` |
| `ish-shell` | Interactive REPL (Reedline), file execution, inline execution |

---

## Proposal Process

This project uses a structured proposal process for all non-trivial changes:

1. **RFP** → 2. **Design Proposal** (iterative) → 3. **Implementation Plan** → 4. **Implementation**

See [GLOSSARY.md](GLOSSARY.md) for definitions of these terms.

### Authority Order

When implementing changes, update project artifacts in this order:

1. GLOSSARY.md (new terms)
2. Roadmap (status → "in progress")
3. Maturity matrix (update affected rows)
4. Specification docs
5. Architecture docs
6. User guide / AI guide
7. Agent documentation (AGENTS.md, skills)
8. Acceptance tests
9. Code
10. Unit tests
11. Roadmap (status → "completed")
12. Maturity matrix (update affected rows)
13. History
14. Index files

Always update more authoritative artifacts before less authoritative ones. If you read an artifact during implementation and it seems to contradict the implementation plan, the implementation plan takes precedence.

### Implementation Discipline

- The implementation plan is the single source of truth during implementation.
- Complete all TODO items in the implementation plan before reporting success.
- At each checkpoint, verify your work against the implementation plan.
- Do not inject behavior that contradicts the implementation plan, even if it seems like an improvement. Propose changes in a follow-up, not during implementation.

### Resuming Implementation

If you are asked to continue implementing a feature and an implementation plan exists in `docs/project/plans/`, read it and resume from the first uncompleted TODO item. Do not start over.

---

## Task Playbooks

Load only the files you need for the task at hand.

| Task | Load these files |
|------|-----------------|
| Creating a design proposal | `/propose` — [.agents/skills/propose/SKILL.md](.agents/skills/propose/SKILL.md) |
| Revising a proposal | `/revise` — [.agents/skills/revise/SKILL.md](.agents/skills/revise/SKILL.md) |
| Accepting a proposal | `/accept` — [.agents/skills/accept/SKILL.md](.agents/skills/accept/SKILL.md) |
| Creating an implementation plan | `/plan-implementation` — [.agents/skills/plan-implementation/SKILL.md](.agents/skills/plan-implementation/SKILL.md) |
| Implementing a plan | `/implement` — [.agents/skills/implement/SKILL.md](.agents/skills/implement/SKILL.md) |
| Auditing feature coherence | `/audit` — [.agents/skills/audit/SKILL.md](.agents/skills/audit/SKILL.md) |
| Proposing code refactoring | `/propose-refactoring` — [.agents/skills/propose-refactoring/SKILL.md](.agents/skills/propose-refactoring/SKILL.md) |
| Fixing a bug | `/bug-fix` — [.agents/skills/bug-fix/SKILL.md](.agents/skills/bug-fix/SKILL.md) |
| Troubleshooting | `/troubleshoot` — [.agents/skills/troubleshoot/SKILL.md](.agents/skills/troubleshoot/SKILL.md) |
| Verifying implementation | `/verify` — [.agents/skills/verify/SKILL.md](.agents/skills/verify/SKILL.md) |
| Updating AGENTS.md | `/update-agents` — [.agents/skills/update-agents/SKILL.md](.agents/skills/update-agents/SKILL.md) |
| Adding a new builtin function | [docs/architecture/vm.md](docs/architecture/vm.md) § Builtins |
| Adding a new AST node | [docs/architecture/ast.md](docs/architecture/ast.md) |
| Modifying the type spec | [docs/spec/types.md](docs/spec/types.md) |
| Working on modules | [docs/spec/modules.md](docs/spec/modules.md), [docs/user-guide/modules.md](docs/user-guide/modules.md) |
| Understanding execution configs | [docs/spec/execution.md](docs/spec/execution.md) |
| Reviewing open questions | [docs/project/open-questions.md](docs/project/open-questions.md) |
| Adding acceptance tests | [proto/ish-tests/lib/test_lib.sh](proto/ish-tests/lib/test_lib.sh), existing tests in `proto/ish-tests/` |
| Understanding a design decision | [docs/project/decisions/INDEX.md](docs/project/decisions/INDEX.md) |
| Working on concurrency | [docs/spec/concurrency.md](docs/spec/concurrency.md), [docs/architecture/vm.md](docs/architecture/vm.md), [docs/architecture/shell.md](docs/architecture/shell.md) |
| Writing concurrent ish code | [docs/ai-guide/playbook-concurrency.md](docs/ai-guide/playbook-concurrency.md) |

---

## Module System

When working on the module system, use these locations:

| Concern | Files |
|---------|-------|
| Module path resolution, project root discovery | `proto/ish-vm/src/module_loader.rs` |
| Visibility enforcement (`priv`/`pkg`/`pub`) | `proto/ish-vm/src/access_control.rs` |
| Interface file (`.ishi`) checking | `proto/ish-vm/src/interface_checker.rs` |
| `use`, `declare { }`, `bootstrap` evaluation | `proto/ish-vm/src/interpreter.rs` |
| Interface file generation (`ish interface freeze`) | `proto/ish-shell/src/interface_cmd.rs` |
| Acceptance tests for module features | `proto/ish-tests/modules/` |

**Interface files:** `.ishi` files contain `pub` function signatures and type aliases (no bodies). Generated by `ish interface freeze [module]`. If a `.ishi` file exists next to a `.ish` file, it is enforced at load time.

**Note:** `CLAUDE.md` is a symlink to `AGENTS.md`. Never edit `CLAUDE.md` directly.

---

## Conventions

- See [CONTRIBUTING.md](CONTRIBUTING.md) for full conventions.
- For Rust style guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).
- For agent instruction style guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).
- Use terms as defined in [GLOSSARY.md](GLOSSARY.md) — do not introduce synonyms.
- Every documentation file requires YAML frontmatter and a `## Referenced by` section.
- After code changes, update affected docs and add a history entry.
- When adding new error conditions, update `docs/errors/INDEX.md` with the new error code, domain subtype, and production site.
- Track documentation debt in [docs/project/documentation-debt.md](docs/project/documentation-debt.md).
- When creating documentation with minimal specification, set `status: placeholder`.
- History entries are written for humans. Use narrative prose that tells the story of what happened, including the evolution of ideas through proposal/response/decision exchanges. Do not write terse summaries or bullet-point logs.
- For proposals, history is stored as a directory per proposal under `docs/project/history/` containing `summary.md` and version files.
- Run `/update-agents` after any change that adds or removes skills, renames crates, or adds new doc sections.

---

## Referenced by

- [docs/INDEX.md](docs/INDEX.md)
