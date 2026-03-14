---
title: "AI Playbook: Low-Assurance Code"
category: ai-guide
audience: [ai-agent]
status: placeholder
last-verified: 2026-03-14
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md]
---

# Playbook: Low-Assurance Code

Use this playbook when generating or modifying code intended to be **low-assurance** — the low-ceremony, dynamic end of ish's continuum.

## When to Use

- Prototyping or exploratory code
- Scripts and one-off utilities
- Code under the `streamlined` standard or no explicit standard
- When no assurance level is specified (low-assurance is the default)

## Guidelines

1. **Omit type annotations** unless the user requests them. Low-assurance code relies on type inference and dynamic dispatch.
2. **Prefer structural typing**. Don't declare nominal types unless the user asks for them.
3. **Use minimal boilerplate**. Low-assurance code should look and feel like a dynamic language.
4. **Let values be flexible**. A variable can hold different types over its lifetime in low-assurance mode.

## Example

```
// Low-assurance ish
let greet = fn(name) {
    "Hello, " + name + "!"
}

let result = greet("world")
print(result)
```

## What NOT to Do

- Don't add type annotations "for safety" — that's high-assurance style.
- Don't warn the user about missing types — they chose low-assurance intentionally.
- Don't introduce nominal types or interfaces unless asked.

See also: [High-assurance playbook](playbook-high-assurance.md) | [Mixed playbook](playbook-mixed.md)

---

## Referenced by

- [docs/ai-guide/INDEX.md](INDEX.md)
- [docs/ai-guide/orientation.md](orientation.md)
