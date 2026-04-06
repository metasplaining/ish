---
title: "Plan: Module System Core A-1 — Language Representation"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-05
depends-on: [docs/project/proposals/module-system-core-a1.md, docs/project/proposals/module-system-core.md, docs/spec/modules.md, GLOSSARY.md]
---

# Plan: Module System Core A-1 — Language Representation

*Derived from [module-system-core-a1.md](../../proposals/module-system-core-a1.md) on 2026-04-05.*

## Overview

This plan implements all changes in Proposal A-1: the language representation layer of the module system. It updates the AST node types, the PEG grammar, the AST builder, and supporting documentation. No runtime behavior changes. When complete, the prototype parses and represents all new module syntax correctly, and the existing test suite continues to pass.

Proposal A-2 (Execution and Tooling) depends on this plan and cannot begin until these changes are merged and the prototype builds cleanly.

## Requirements

1. `Visibility` has exactly three variants: `Priv`, `Pkg`, `Pub`. No other variants exist.
2. `None` in `Option<Visibility>` means "default" (pkg). `Priv` and `Pub` are present only when explicitly written in source.
3. `Statement::Use` has fields: `module_path: Vec<String>`, `alias: Option<String>`, `selective: Option<Vec<SelectiveImport>>`.
4. `SelectiveImport` is a public struct with fields `name: String` and `alias: Option<String>`.
5. `Statement::ModDecl` does not exist anywhere in the codebase.
6. `Statement::DeclareBlock { body: Vec<Statement> }` exists and is parsed from `declare { ... }`.
7. `Statement::Bootstrap { source: BootstrapSource }` exists and is parsed from `bootstrap "..."` and `bootstrap { ... }`.
8. `BootstrapSource` has variants: `Path(String)`, `Url(String)`, `Inline(String)`.
9. `IncompleteKind::DeclareBlock` exists and is produced for unterminated `declare {`.
10. The grammar uses `visibility = { "priv" | "pkg" | "pub" }` (not `pub_modifier`).
11. Module paths use `/` as separator (not `::`). External paths begin with a domain segment (`foo.bar/...`).
12. `use_stmt` parses all four forms: plain, aliased (`as`), selective (`{ Name }`), selective-with-rename (`{ Name as N }`).
13. `declare_block` replaces `mod_stmt` in the grammar. `mod_stmt` does not exist.
14. `bootstrap_stmt` parses path-form (string literal) and inline-JSON-form (`{ ... }`).
15. `priv`, `pkg`, `declare`, and `bootstrap` are keywords. `mod` is not a keyword.
16. All exhaustive matches on `Statement` compile without warnings. No `ModDecl` arm exists anywhere.
17. `display.rs` produces correct output for all new/changed node types.
18. GLOSSARY.md has entries for all 13 terms listed in the proposal.
19. `docs/spec/syntax.md` has a "Module Directives" section covering `use`, `declare`, `bootstrap`, and visibility keywords.
20. `docs/architecture/ast.md` documents the new and changed AST node types.

## Phase Dependency Graph

```
Phase 1 (GLOSSARY)
    ↓
Phase 2 (docs: syntax.md + ast.md)
    ↓
Phase 3 (ish-ast code)
    ↓
Phase 4 (ish-parser code)       Phase 5 (ish-vm ModDecl removal)
         ↘                    ↗
              Phase 6 (unit tests + build verification)
```

Phases 4 and 5 are independent once Phase 3 is done. Phase 6 requires both.

## Authority Order

1. GLOSSARY.md — new terms
2. `docs/spec/syntax.md` — language syntax spec
3. `docs/architecture/ast.md` — architecture doc
4. `proto/ish-ast/` — AST types, display, builder
5. `proto/ish-parser/` — grammar, AST builder, existing tests
6. `proto/ish-vm/` — remove ModDecl arms, add no-op arms for new variants
7. Unit tests — new tests in ish-ast and ish-parser

## Context Files

- [context/ish-ast-current.md](context/ish-ast-current.md) — current Visibility, Use, ModDecl, IncompleteKind
- [context/grammar-modules-current.md](context/grammar-modules-current.md) — current grammar module rules
- [context/ast-builder-current.md](context/ast-builder-current.md) — current build_visibility, build_use_stmt, build_mod_stmt
- [context/vm-moddecl-arms.md](context/vm-moddecl-arms.md) — all ModDecl arms in ish-vm files

## Phases

- [phase-1.md](phase-1.md) — GLOSSARY.md (13 terms)
- [phase-2.md](phase-2.md) — docs/spec/syntax.md + docs/architecture/ast.md
- [phase-3.md](phase-3.md) — ish-ast: Visibility, Use, ModDecl removal, DeclareBlock, Bootstrap, display, builder
- [phase-4.md](phase-4.md) — ish-parser: grammar + ast_builder + phase5.rs
- [phase-5.md](phase-5.md) — ish-vm: remove ModDecl arms, add no-op arms for new variants
- [phase-6.md](phase-6.md) — unit tests + full build verification

## Referenced by

- [docs/project/proposals/module-system-core-a1.md](../../proposals/module-system-core-a1.md)
- [docs/project/plans/INDEX.md](../INDEX.md)
