---
title: "AI Playbook: Mixed-Mode Code"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-14
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/modules.md]
---

# Playbook: Mixed-Mode Code

Use this playbook when a codebase contains both low-assurance and high-assurance code, or when migrating code between assurance levels.

## When to Use

- Projects with a mix of prototyping and production code
- When increasing the assurance level of a previously low-assurance module
- When boundaries exist between low-assurance and high-assurance components
- When the user says "add types to this" or "make this stricter"

## Guidelines

1. **Respect existing assurance levels**. Don't change a module's standard without being asked.
2. **Handle boundaries explicitly**. Where low-assurance code calls high-assurance code (or vice versa), the assurance ledger governs what happens. See [assurance-ledger.md](../spec/assurance-ledger.md).
3. **Increase assurance incrementally**. When migrating low-assurance → high-assurance, add constraints one at a time rather than rewriting everything at once.
4. **Preserve behavioral equivalence**. Increasing assurance adds constraints — it should not change what the code does, only what it guarantees.

## Boundary Patterns

When low-assurance code calls a high-assurance function:
- The high-assurance function's parameter types are checked at the boundary
- Runtime type errors occur if the low-assurance caller passes invalid values

When high-assurance code calls a low-assurance function:
- The return value is untyped from the high-assurance code's perspective
- The caller must handle the unknown type explicitly

## Migration Workflow

1. Identify the module or function to increase assurance on
2. Add type annotations to public API first
3. Add type annotations to internals
4. Add nominal types where structural types were used
5. Apply a standard (e.g., `@standard[rigorous]`) and verify

See also: [Low-assurance playbook](playbook-low-assurance.md) | [High-assurance playbook](playbook-high-assurance.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
