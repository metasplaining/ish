---
title: "AI Guide: Antipatterns"
category: ai-guide
audience: [ai-agent]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/agreement.md, docs/spec/types.md]
---

# Antipatterns

Things AI agents should **not** do when generating ish code.

## 1. Mixing Modes Without Marking

**Wrong**: Adding type annotations to some variables but not others, without explicit encumbrance marking.

```
// BAD — inconsistent encumbrance, no marking
let x: Int = 42
let y = "hello"
let z: Bool = true
```

**Right**: Either fully streamlined or fully encumbered, with the choice made explicit.

## 2. Applying Other Languages' Idioms

**Wrong**: Using Java-style class hierarchies, Python-style duck typing assumptions, or Rust-style lifetime annotations. ish is its own language.

**Right**: Use ish's own abstractions — structural typing, encumbrance marking, agreement protocol.

## 3. Inventing Syntax

**Wrong**: Using syntax that looks reasonable but isn't specified. ish's syntax is not yet fully designed — don't guess.

**Right**: Check [docs/spec/syntax.md](../spec/syntax.md) and existing examples. When unsure, ask the user or note the uncertainty.

## 4. Over-Encumbering

**Wrong**: Adding types, invariants, and constraints to code the user wanted streamlined.

**Right**: Match the user's requested encumbrance level. When unspecified, default to streamlined.

## 5. Ignoring Agreement Boundaries

**Wrong**: Treating the boundary between streamlined and encumbered code as invisible.

**Right**: Explicitly handle type mismatches at boundaries per [agreement.md](../spec/agreement.md).

## 6. Assuming Runtime Behavior

**Wrong**: Assuming specific garbage collection, threading model, or memory layout.

**Right**: Check [memory.md](../spec/memory.md) and [execution.md](../spec/execution.md) for what is specified.

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
