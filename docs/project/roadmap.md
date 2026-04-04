---
title: "Project Roadmap"
category: project
audience: [human-dev, ai-agent]
status: placeholder
last-verified: 2026-03-19
depends-on: []
---

# Roadmap

This document tracks high-level milestones for the ish project.

## Current Phase: Design + Prototype

The language is being designed iteratively. A Rust prototype explores key ideas.

### Completed

- [x] Core type system design (primitives, objects, lists, unions, optionals)
- [x] Module system design (visibility, namespacing)
- [x] Assurance Ledger system
- [x] Execution configurations concept
- [x] Reasoning system concept
- [x] Prototype: AST representation
- [x] Prototype: Tree-walking interpreter
- [x] Prototype: Builtins (print, type-of, len, etc.)
- [x] Prototype: Reflection subsystem
- [x] Documentation infrastructure
- [x] Parser: PEG grammar with pest (ish-parser crate)
- [x] Parser: Parser-matches-everything (33 unterminated productions, `Incomplete` AST nodes)
- [x] Shell: Reedline-based interactive REPL with multiline input, syntax highlighting, history
- [x] Shell: File and inline execution modes
- [x] Shell: Command execution (builtins, external commands, pipes, redirections, globs, `$?`)
- [x] Proposal process improvements (three-document lifecycle, six skills, authority order)
- [x] Error handling design (design complete; implementation pending entry-based error model)
- [x] Types, errors, and assurance ledger consistency
- [x] Concurrency design (cooperative multitasking, async/await, Tokio runtime, shell architecture)
- [x] Concurrency prototype (async/await/spawn/yield, Tokio LocalSet, two-thread shell, yield budget, ledger integration)
- [x] Concurrency correctness fixes (FutureRef equality, grammar-level await/spawn, compiled functions)
- [x] Runtime extraction (shim-only architecture, ish-core, ish-runtime type extraction, ErrorCode enum, apply builtin)

### In Progress

- [ ] Stubbed code analyzer and yielding/unyielding function refactoring (PENDING_INTERP_CALL removal)
- [ ] Memory management design (GC vs. manual vs. arena)
- [ ] Polymorphism strategy (structural vs. nominal interplay)

### Future

- [ ] Bytecode compiler
- [ ] Standard library
- [ ] Package management
- [ ] Tooling (formatter, linter, LSP)
- [ ] Self-hosting

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
