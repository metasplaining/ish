---
title: Architecture Index
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/INDEX.md]
---

# Architecture

Internals of the ish language processor. These documents reference source code directly and describe data flow between components.

---

| Document | Component | Status |
|----------|-----------|--------|
| [overview.md](overview.md) | High-level architecture, crate dependency graph, key patterns | draft |
| [ast.md](ast.md) | ish-ast — AST types, builder API, display | draft |
| [vm.md](vm.md) | ish-vm — interpreter, values, environment, builtins, reflection | draft |
| [stdlib.md](stdlib.md) | ish-stdlib — self-hosted analyzer, generator, standard library | draft |
| [codegen.md](codegen.md) | ish-codegen — compilation driver, template generation | draft |
| [runtime.md](runtime.md) | ish-runtime — Value, Shim, RuntimeError, ErrorCode, IshFunction | draft |
| [shell.md](shell.md) | ish-shell — CLI binary, verification demos | draft |

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
