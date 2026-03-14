---
title: ish Syntax
category: spec
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/types.md, docs/spec/assurance-ledger.md]
---

# ish Syntax

> **Note:** The formal syntax of ish has not yet been designed. This document is a placeholder. The prototype constructs programs as ASTs directly — there is no parser.

The syntax will be defined once the language semantics are more complete. Key open questions:

- Is ish C-family, ML-family, Lisp-family, or something novel?
- What are the delimiters (braces, indentation, keywords)?
- How are comments written?
- What is the basic expression and statement syntax?

See [docs/project/open-questions.md](../project/open-questions.md#syntax-and-language-surface) for the full list of syntax open questions.

---

## Assurance Ledger Syntax Constructs

The following syntax constructs have been designed for the assurance ledger system. See [docs/spec/assurance-ledger.md](assurance-ledger.md) for full details.

| Construct | Syntax | Scope |
|-----------|--------|-------|
| Apply standard to scope | `@standard[name]` | block, function, module |
| Inline feature override | `@standard[feature(state)]` | block, function, module |
| Multi-feature override | `@standard[feat1(state), feat2(state)]` | block, function, module |
| Apply entry to item | `@[entry(params)]` | variable, property, function, type, statement |
| Define a standard | `standard name [...]` | module, function, block |
| Extend a standard | `standard name extends base [...]` | module, function, block |
| Define an entry type | `entry type name { ... }` | module level |

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
