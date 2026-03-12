---
title: "AI Playbook: Mixed-Mode Code"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-10
depends-on: [docs/spec/agreement.md, docs/spec/types.md, docs/spec/modules.md]
---

# Playbook: Mixed-Mode Code

Use this playbook when a codebase contains both streamlined and encumbered code, or when migrating code between modes.

## When to Use

- Projects with a mix of prototyping and production code
- When encumbering a previously-streamlined module
- When boundaries exist between streamlined and encumbered components
- When the user says "add types to this" or "make this stricter"

## Guidelines

1. **Respect existing encumbrance levels**. Don't change a module's encumbrance without being asked.
2. **Handle boundaries explicitly**. Where streamlined code calls encumbered code (or vice versa), the agreement protocol governs what happens. See [agreement.md](../spec/agreement.md).
3. **Encumber incrementally**. When migrating streamlined → encumbered, add constraints one at a time rather than rewriting everything at once.
4. **Preserve behavioral equivalence**. Encumbering code adds constraints — it should not change what the code does, only what it guarantees.

## Boundary Patterns

When streamlined code calls an encumbered function:
- The encumbered function's parameter types are checked at the boundary
- Runtime type errors occur if the streamlined caller passes invalid values

When encumbered code calls a streamlined function:
- The return value is untyped from the encumbered code's perspective
- The caller must handle the unknown type explicitly

## Migration Workflow

1. Identify the module or function to encumber
2. Add type annotations to public API first
3. Add type annotations to internals
4. Add nominal types where structural types were used
5. Verify with execution configuration set to enforce

See also: [Streamlined playbook](playbook-streamlined.md) | [Encumbered playbook](playbook-encumbered.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
