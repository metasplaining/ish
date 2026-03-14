---
title: "AI Playbook: High-Assurance Code"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-14
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md]
---

# Playbook: High-Assurance Code

Use this playbook when generating or modifying code intended to be **high-assurance** — the high-constraint, static end of ish's continuum.

## When to Use

- Production code
- Library APIs
- Code under a strict standard (e.g., `@standard[rigorous]`)
- When the user asks for "strict", "typed", or "safe" code

## Guidelines

1. **Always provide type annotations** on function parameters, return types, and variable declarations.
2. **Use nominal types** where appropriate — define named types and interfaces, not just structural shapes.
3. **Apply a standard explicitly**. Use `@standard[rigorous]` or the appropriate standard to set the assurance level.
4. **Declare invariants**. If a type has constraints (non-empty, positive, bounded), encode them.
5. **Prefer exhaustive handling**. Pattern matches and union type branches should be exhaustive.

## Example

```
// High-assurance ish
@standard[rigorous] {
    type Greeting = {
        message: String
        timestamp: Int
    }

    fn greet(name: String) -> Greeting {
        {
            message: "Hello, " + name + "!",
            timestamp: now()
        }
    }

    let result: Greeting = greet("world")
    print(result.message)
}
```

## What NOT to Do

- Don't omit types for brevity — high-assurance code pays the ceremony cost for safety.
- Don't use untyped variables or `any` equivalents.
- Don't skip applying a standard when high-assurance is intended.

See also: [Low-assurance playbook](playbook-low-assurance.md) | [Mixed playbook](playbook-mixed.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
