---
title: Error Catalog
category: project
audience: [all]
status: draft
last-verified: 2026-03-11
depends-on: [docs/INDEX.md]
---

# Error Catalog

Every error the ish language processor can produce, with explanations and remediation.

> **Note:** The prototype currently uses `RuntimeError` for all error conditions. As the type system matures, specific error codes will be assigned.

---

| Error Code | Category | Summary |
|------------|----------|---------|
| *E001* | Runtime | Unhandled throw — a thrown value escaped all try/catch blocks and function boundaries |
| *E002* | Runtime | Division by zero |
| *E003* | Runtime | Argument count mismatch — function called with wrong number of arguments |
| *E004* | Runtime | Type mismatch — operation applied to incompatible value types |
| *E005* | Runtime | Undefined variable — referenced variable not found in scope |
| *E006* | Runtime | Not callable — attempted to call a non-function value |

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
