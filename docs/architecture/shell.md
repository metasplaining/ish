---
title: "Architecture: ish-shell"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md]
---

# ish-shell

**Source:** `proto/ish-shell/src/`

CLI binary demonstrating the 6 end-to-end verifications.

---

## Verifications

Each demo is independent and self-contained. They share a single `IshVm` instance with stdlib loaded. The compilation demos use `CompilationDriver` with a relative path to `ish-runtime`.

1. **Interpreted factorial(10)** — AST → interpreter → result
2. **Compiled factorial(10)** — AST → Rust source → `.so` → dynamic load → result
3. **Analyzer detects undeclared variable** — self-hosted analyzer on AST-as-values
4. **Generator produces compilable Rust** — self-hosted generator → compile → run
5. **Stdlib functions** — `abs(-42)` and `sum(range(5))`
6. **Consistency: interpreted == compiled** — compare results for factorial(5, 8, 12)

---

## Tests

The shell binary itself has no unit tests — it is the integration test.

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
