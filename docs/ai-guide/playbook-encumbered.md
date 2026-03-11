---
title: "AI Playbook: Encumbered Code"
category: ai-guide
audience: [ai-agent]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/agreement.md, docs/spec/types.md]
---

# Playbook: Encumbered Code

Use this playbook when generating or modifying code intended to be **encumbered** — the high-constraint, static end of ish's continuum.

## When to Use

- Production code
- Library APIs
- Code explicitly marked as encumbered by the user
- When the user asks for "strict", "typed", or "safe" code

## Guidelines

1. **Always provide type annotations** on function parameters, return types, and variable declarations.
2. **Use nominal types** where appropriate — define named types and interfaces, not just structural shapes.
3. **Mark encumbrance explicitly**. Every constraint-bearing feature should be visibly marked.
4. **Declare invariants**. If a type has constraints (non-empty, positive, bounded), encode them.
5. **Prefer exhaustive handling**. Pattern matches and union type branches should be exhaustive.

## Example

```
// Encumbered ish
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
```

## What NOT to Do

- Don't omit types for brevity — encumbered code pays the ceremony cost for safety.
- Don't use untyped variables or `any` equivalents.
- Don't skip marking features that should be encumbered.

See also: [Streamlined playbook](playbook-streamlined.md) | [Mixed playbook](playbook-mixed.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
