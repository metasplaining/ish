---
title: "Language Comparison"
category: project
audience: [human-dev]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md]
---

# Language Comparison

How ish relates to other languages. This is **not** a value judgment — each language makes valid tradeoffs. This document clarifies where ish sits in the design space.

## The Continuum Idea

Most languages pick a fixed point on the static ↔ dynamic spectrum:

| Language | Position | Key Tradeoff |
|----------|----------|-------------|
| Python | Dynamic | Fast to write, runtime errors |
| JavaScript | Dynamic | Flexible, unpredictable types |
| TypeScript | Gradual | Optional types, JS interop |
| Java | Static | Verbose, but type-safe |
| Rust | Static | Zero-cost abstractions, steep learning curve |
| Go | Static | Simple, but limited expressiveness |

ish is **not** a gradual typing system like TypeScript. The difference:

- **TypeScript**: You can add types and the compiler checks them. Untyped code is `any`.
- **ish**: The **assurance level** concept is broader than types. It governs visibility, invariants, runtime checks, and more. Moving along the continuum changes what the language *does*, not just what it checks.

## Closest Relatives

- **TypeScript**: Shares the "add types incrementally" workflow, but ish's continuum is richer.
- **Kotlin**: Shares the "pragmatic with safety features" philosophy.
- **Scala**: Shares the blend of functional and object-oriented, with a powerful type system.
- **Lua**: Shares the lightweight, embeddable feel of low-assurance ish.

## What ish Does Differently

1. **Per-feature assurance levels**: Not just gradual types — every feature has its own low-assurance ↔ high-assurance axis.
2. **Assurance ledger**: A formal mechanism for tracking and auditing constraints across module boundaries.
3. **Execution configurations**: The runtime behavior changes based on the deployment context.
4. **Reasoning system**: Built-in support for expressing *why* constraints exist.

---

## Referenced by

- [docs/INDEX.md](INDEX.md)
