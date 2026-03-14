---
title: "User Guide: Assurance Levels"
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-14
depends-on: [docs/spec/types.md, docs/spec/assurance-ledger.md]
---

# Assurance Levels

Assurance levels are ish's central organizing concept. Every feature in the language sits somewhere on a **low-assurance ↔ high-assurance** continuum.

## What Are Assurance Levels?

A feature is **high-assurance** when it carries additional constraints — stricter types, visibility rules, invariant checks, or semantic guarantees. A feature is **low-assurance** when those constraints are relaxed.

Think of it like seat belts: you can drive without one (low-assurance), but wearing one adds a constraint that makes you safer (high-assurance).

## Why?

Most languages force a single point on this spectrum. Static languages like Rust are highly assured — you get safety, but at the cost of ceremony. Dynamic languages like Python are low-assurance — you move fast, but errors appear at runtime.

ish lets you choose, **per feature**, where on the continuum you want to be. Start low-assurance, then increase assurance as the code matures.

## Standards and Entries

Assurance levels are configured through **standards** — named sets of feature configurations applied to scopes. Individual items can be annotated with **entries** that record facts about them.

```
// Low-assurance: no type annotation required
let x = 42

// High-assurance: explicit type annotation (an entry in the ledger)
let x: Int = 42
```

Standards are applied with `@standard[name]`:

```
@standard[rigorous] {
    // All features in this scope are high-assurance
    let x: Int = 42
}
```

See [docs/spec/assurance-ledger.md](../spec/assurance-ledger.md) for the full specification of how standards, entries, and the assurance ledger work.

## Execution Configurations

The assurance level affects how the runtime behaves. Different **execution configurations** (development, testing, production) may enforce different assurance levels.

See [docs/spec/execution.md](../spec/execution.md) for details.

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/INDEX.md](../ai-guide/INDEX.md)
