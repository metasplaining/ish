---
title: "Project Roadmap"
category: project
audience: [human-dev, ai-agent]
status: placeholder
last-verified: 2026-03-10
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

### In Progress

- [ ] Memory management design (GC vs. manual vs. arena)
- [ ] Polymorphism strategy (structural vs. nominal interplay)
- [ ] Error handling design

### Future

- [ ] Bytecode compiler
- [ ] Standard library
- [ ] Package management
- [ ] Tooling (formatter, linter, LSP)
- [ ] Self-hosting

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
