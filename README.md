# ish

**ish** is a general-purpose programming language where every feature sits on a **low-assurance ↔ high-assurance** continuum. Start fast and loose like a dynamic language. Add constraints incrementally until your code is as strict as Rust. Choose per-feature, per-module, or per-project.

## Why ish?

Most languages pick one point on the static↔dynamic spectrum. ish lets you choose — and change your mind — per feature:

- **Low-assurance**: Interpreted, garbage-collected, structurally typed. Feels like Python or Lua.
- **High-assurance**: Compiled, memory-managed, nominally typed. Feels like Rust or TypeScript at its strictest.
- **Mixed**: Different parts of the same codebase at different points on the continuum.

Increasing assurance doesn't make code faster — it makes hidden concerns visible, so you can address them. See [docs/comparison.md](docs/comparison.md) for how ish relates to other languages.

## Status

ish is in the **design + prototype** phase. The language spec is partially written. A Rust prototype (~5,600 lines) explores core ideas. See [docs/project/maturity.md](docs/project/maturity.md) for what's implemented.

## Documentation

All documentation lives under `docs/`. Start with [docs/INDEX.md](docs/INDEX.md).

| Section | Description |
|---------|-------------|
| [Language Specification](docs/spec/INDEX.md) | Normative rules: types, modules, assurance ledger, execution |
| [Architecture](docs/architecture/INDEX.md) | Prototype internals (6 Rust crates) |
| [User Guide](docs/user-guide/INDEX.md) | Tutorial for human developers |
| [AI Guide](docs/ai-guide/INDEX.md) | Playbooks for AI coding agents |
| [Project](docs/project/roadmap.md) | Roadmap, decisions, open questions |
| [Glossary](GLOSSARY.md) | Terminology definitions |

## Key Concepts

| Concept | Summary | Details |
|---------|---------|---------|
| Assurance level | Per-feature constraints on a low-assurance↔high-assurance axis | [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md) |
| Assurance Ledger | System for tracking and auditing constraints across boundaries | [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md) |
| Execution configs | Dev/test/prod profiles that enforce assurance levels | [docs/spec/execution.md](docs/spec/execution.md) |
| Polymorphism | 5 strategies from static dispatch to associative arrays | [docs/spec/polymorphism.md](docs/spec/polymorphism.md) |
| Memory management | 4 models from stack to garbage collection | [docs/spec/memory.md](docs/spec/memory.md) |
| Reasoning | Metadata annotations that influence language processor behavior | [docs/spec/reasoning.md](docs/spec/reasoning.md) |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for conventions and practices.

## For AI Agents

See [AGENTS.md](AGENTS.md) and [docs/ai-guide/orientation.md](docs/ai-guide/orientation.md).

