---
title: Getting Started with ish
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/execution.md, docs/spec/syntax.md]
---

# Getting Started with ish

---

## What is ish?

ish is a general-purpose programming language designed to support a wide range of language tradeoffs within a single unified language. Instead of choosing between a dynamically-typed scripting language and a statically-typed compiled language, ish lets you configure where on that spectrum your code sits — from fully low-assurance (interpreted, flexible, forgiving) to fully high-assurance (compiled, strict, optimized).

See [README.md](../../README.md) for the full motivation.

---

## Installation

*Installation instructions will be added once ish has a distributable binary.*

### Running the Prototype

To run the current prototype:

```bash
cd proto
cargo build --workspace
cargo run -p ish-shell     # Runs 6 verification demos
cargo test --workspace     # Runs all 45 tests
```

---

## Hello World

```ish
println("Hello, world!")
```

### String Basics

ish uses shell-convention quoting — single quotes for literal strings, double quotes for interpolating strings:

```ish
let name = 'Alice'                       // literal string (no interpolation)
let greeting = "Hello, {name}!"          // interpolation with {expr}
let home = "Home: $HOME"                 // environment variable expansion
```

See [docs/spec/syntax.md § Strings](../spec/syntax.md#strings) for the full string syntax.

---

## Next Steps

- [Language Basics](language-basics.md) — Syntax, expressions, statements
- [Types](types.md) — The ish type system
- [Functions](functions.md) — Functions, closures, lambdas
- [The Assurance Level Continuum](assurance-levels.md) — What makes ish different

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
