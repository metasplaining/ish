---
title: Getting Started with ish
category: user-guide
audience: [human-dev]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/execution.md]
---

# Getting Started with ish

> **Note:** ish is in early development. There is no parser yet — the prototype constructs programs as ASTs in Rust. This guide describes the *intended* developer experience. Runnable examples will be added once the parser is implemented.

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

## Hello World (Intended Syntax)

```
println("Hello, world!");
```

---

## Next Steps

- [Language Basics](language-basics.md) — Syntax, expressions, statements
- [Types](types.md) — The ish type system
- [Functions](functions.md) — Functions, closures, lambdas
- [The Assurance Level Continuum](assurance-levels.md) — What makes ish different

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
