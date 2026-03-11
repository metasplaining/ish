---
title: "AI Playbook: Streamlined Code"
category: ai-guide
audience: [ai-agent]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/agreement.md, docs/spec/types.md]
---

# Playbook: Streamlined Code

Use this playbook when generating or modifying code intended to be **streamlined** — the low-ceremony, dynamic end of ish's continuum.

## When to Use

- Prototyping or exploratory code
- Scripts and one-off utilities
- Code explicitly marked as streamlined by the user
- When no encumbrance level is specified (streamlined is the default)

## Guidelines

1. **Omit type annotations** unless the user requests them. Streamlined code relies on type inference and dynamic dispatch.
2. **Prefer structural typing**. Don't declare nominal types unless the user asks for them.
3. **Use minimal boilerplate**. Streamlined code should look and feel like a dynamic language.
4. **Let values be flexible**. A variable can hold different types over its lifetime in streamlined mode.

## Example

```
// Streamlined ish
let greet = fn(name) {
    "Hello, " + name + "!"
}

let result = greet("world")
print(result)
```

## What NOT to Do

- Don't add type annotations "for safety" — that's encumbered style.
- Don't warn the user about missing types — they chose streamlined intentionally.
- Don't introduce nominal types or interfaces unless asked.

See also: [Encumbered playbook](playbook-encumbered.md) | [Mixed playbook](playbook-mixed.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
