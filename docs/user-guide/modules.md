---
title: "User Guide: Modules"
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/modules.md]
---

# Modules

ish code is organized into modules. Every `.ish` file defines a module, and the module path mirrors the file path.

For the full specification, see [docs/spec/modules.md](../spec/modules.md).

---

## Visibility

Symbols are private by default. Use visibility directives to expose them:

```
pub(project) fn my_function() { ... }   // visible within the project
pub(global) fn api_function() { ... }    // visible to everyone
```

## Importing

Use the `use` directive to import symbols from other modules.

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
