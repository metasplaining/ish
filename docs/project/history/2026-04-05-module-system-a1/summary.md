---
title: Module System A-1 — Language Representation — Design History
category: project
audience: [all]
status: current
last-verified: 2026-04-05
depends-on: [docs/project/proposals/module-system-core-a1.md, docs/project/history/2026-04-05-module-system/summary.md]
---

# Module System A-1 — Language Representation — Design History

*April 5, 2026*

This directory captures the history of Proposal A-1: Module Core — Language Representation, which covers the AST, grammar, and documentation changes needed to represent the module system at the language level. The runtime implementation work is in Proposal A-2.

---

## Version 1 — Initial and Accepted

*Split from [module-system-core.md](../../proposals/module-system-core.md) v10 on 2026-04-05. Accepted on 2026-04-05.*

Proposal A-1 was created as the first of two child proposals when the module-system-core proposal was split at v10. The split was accepted in v10 after the human and agent together identified more than ten independent implementation steps across two natural layers. The language representation layer (this proposal) and the execution and tooling layer (Proposal A-2) were cleanly separable because the AST and grammar changes have no runtime dependencies — they can be implemented, compiled, and tested before any VM or shell changes begin.

The proposal arrived already fully resolved. Through ten versions of the parent proposal, every decision in the module system design had been made: the three visibility levels (`priv`, `pkg`, `pub`), the four `use` import forms with `/` path separators, `declare { }` blocks replacing `mod { }` blocks, the `bootstrap` directive for standalone scripts, `index.ish` for top-level package exports, strict `.ish` extension for importable modules, `src/` as the required source root. The audit of the existing prototype revealed that the current AST and grammar contain partial, incorrect scaffolding — old `Visibility` variants, `::` separators, `ModDecl` nodes — all of which must be replaced.

The proposal was accepted without further revision because all decision points had been settled in the parent proposal's iterative process and carried over faithfully into the A-1 text. No `-->` markers or open questions remained.

### What A-1 covers

The accepted proposal specifies:

**`ish-ast` changes:** The `Visibility` enum is updated from the incorrect `Private / Public / PubScope(String)` to `Priv / Pkg / Pub`. `Statement::Use` gains `alias` and `selective` fields. `Statement::ModDecl` is removed. `Statement::DeclareBlock` and `Statement::Bootstrap` (with `BootstrapSource`) are added. The display formatter and builder helpers are updated throughout.

**`ish-parser` changes:** The grammar replaces `pub_modifier` with a three-way `visibility` rule, replaces `::` with `/` as the path separator, extends `use_stmt` to cover all four import forms, replaces `mod_stmt` with `declare_block`, and adds `bootstrap_stmt` with inline JSON support. The AST builder functions are updated correspondingly.

**Documentation:** GLOSSARY.md receives thirteen new or updated terms. `docs/spec/syntax.md` gains a "Module Directives" section. `docs/architecture/ast.md` is updated to document the new and changed node types.

**Unit tests:** Parser and AST tests for all new constructs — visibility keywords, all four import forms, external domain paths, `declare` blocks, all three `bootstrap` forms, and the `IncompleteKind::DeclareBlock` incomplete node.

### Relationship to the broader module system

A-1 is a prerequisite for A-2. Once A-1 is implemented and the prototype builds cleanly with the new AST and grammar, A-2 can proceed with the VM module loader, access control, interface file checking, shell subcommands, error codes, and acceptance tests. The language design reference for both proposals is [module-system-core.md](../../proposals/module-system-core.md), which remains authoritative on semantics. A-1 and A-2 cover only the implementation work.

Full text: [v1.md](v1.md), accepted proposal at [module-system-core-a1.md](../../proposals/module-system-core-a1.md)

---

## Referenced by

- [docs/project/proposals/module-system-core-a1.md](../../proposals/module-system-core-a1.md)
- [docs/project/history/INDEX.md](../INDEX.md)
