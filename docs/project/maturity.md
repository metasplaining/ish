---
title: "Feature Maturity Matrix"
category: project
audience: [human-dev, ai-agent]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/INDEX.md]
---

# Feature Maturity Matrix

Tracks which language features are designed, prototyped, and production-ready.

| Feature | Designed | Spec Written | Prototyped | Tested | Stable |
|---------|----------|-------------|------------|--------|--------|
| Primitive types (Int, Float, String, Bool, Nil) | ✅ | ✅ | ✅ | partial | ❌ |
| Object types | ✅ | ✅ | ✅ | partial | ❌ |
| List types | ✅ | ✅ | ✅ | partial | ❌ |
| Union types | ✅ | ✅ | partial | ❌ | ❌ |
| Optional types | ✅ | ✅ | partial | ❌ | ❌ |
| Functions / lambdas | ✅ | ✅ | ✅ | partial | ❌ |
| Closures | ✅ | partial | ✅ | partial | ❌ |
| Module system | ✅ | ✅ | ❌ | ❌ | ❌ |
| Visibility (pub/priv) | ✅ | ✅ | ❌ | ❌ | ❌ |
| Encumbrance / agreement | ✅ | ✅ | ❌ | ❌ | ❌ |
| Execution configurations | ✅ | ✅ | ❌ | ❌ | ❌ |
| Reasoning system | ✅ | ✅ | ❌ | ❌ | ❌ |
| Memory management | partial | partial | ❌ | ❌ | ❌ |
| Polymorphism | partial | partial | ❌ | ❌ | ❌ |
| Error handling | ❌ | ❌ | ❌ | ❌ | ❌ |
| Syntax / grammar | ❌ | ❌ | ❌ | ❌ | ❌ |
| Pattern matching | ❌ | ❌ | ❌ | ❌ | ❌ |
| Standard library | ❌ | ❌ | partial | ❌ | ❌ |
| Parser | ❌ | ❌ | ❌ | ❌ | ❌ |

## Legend

- **Designed**: Conceptual design exists
- **Spec Written**: Formal specification in `docs/spec/`
- **Prototyped**: Implemented in `proto/`
- **Tested**: Has test coverage
- **Stable**: API is frozen; changes require an ADR

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
- [docs/ai-guide/orientation.md](../ai-guide/orientation.md)
