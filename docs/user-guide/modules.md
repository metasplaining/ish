---
title: "User Guide: Modules"
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-04-06
depends-on: [docs/spec/modules.md]
---

# Modules

ish code is organized into modules. Every `.ish` file under `src/` defines a module, and the module path mirrors the file path relative to `src/`.

For the full specification, see [docs/spec/modules.md](../spec/modules.md).

---

## Getting Started

A minimal ish project looks like this:

```
myproject/
  project.json
  src/
    math.ish
  scripts/
    main
```

`src/math.ish` defines a module with a public function:

```ish
pub fn add(a: Int, b: Int) -> Int {
  a + b
}
```

`scripts/main` is a script that imports and uses the module:

```ish
#!/usr/bin/env ish
use math { add }
println(add(2, 3))
```

Run the script:

```bash
ish scripts/main
# Output: 5
```

---

## Visibility

Every declaration has a visibility level. The default is `pkg`.

| Keyword | Scope | When to Use |
|---------|-------|-------------|
| `priv` | Current file only | Internal helpers that no other file should call. |
| `pkg` | Entire project | **Default.** Functions and types used within the project but not exposed to external consumers. |
| `pub` | Everyone, including external dependents | Functions and types that form the project's public API. |

```ish
fn project_helper() { ... }       // pkg — visible within the project (default)
priv fn file_only() { ... }        // priv — visible only in this file
pub fn api_function() { ... }      // pub — visible to external consumers
```

### Who Gets `pkg` Access?

Any file physically located under the project root (the directory containing `project.json`):

- Module files under `src/`
- Scripts in `scripts/`, `tools/`, or anywhere else under the project root

Files **not** under the project root — including inline scripts (`ish -e "..."`) and REPL sessions — do not get `pkg` access.

---

## Writing Importable Modules

A module file must have the `.ish` extension and be located under `src/`. It should contain only declarations (function definitions, type definitions). Files with top-level commands (like `println(...)`) cannot be imported — attempting to do so produces error E018.

```ish
// src/greet.ish — importable module
pub fn hello(name: String) -> String {
  str_concat("Hello, ", name)
}
```

---

## Writing Scripts

Scripts are executable files that contain top-level commands. By convention, scripts use a shebang line and omit the `.ish` extension:

```ish
#!/usr/bin/env ish
use greet { hello }
println(hello("world"))
```

A file that contains only declarations is both runnable and importable. A file that contains top-level commands is runnable only.

---

## Project Layout

```
myproject/
  project.json     // project manifest (required)
  src/             // module source files (required)
    net/
      http.ish     // module: net/http
      index.ish    // module: net (not net/index)
    util.ish       // module: util
  scripts/         // public scripts (optional, distributed with the project)
    server
    cli
  tools/           // internal scripts (optional, not distributed)
    generate
    test-runner
```

### `src/` — Where Modules Live

All importable `.ish` files go under `src/`. The `src/` prefix is never part of a module path — `src/net/http.ish` is imported as `use net/http`, not `use src/net/http`.

### `scripts/` — Public Scripts

Scripts for users of your project. These are distributed with the package.

### `tools/` — Internal Scripts

Scripts for project development: build tools, code generators, test runners. Not distributed.

---

## The `use` Directive

Import symbols from other modules with `use`. There are four forms:

### Qualified Import

```ish
use net/http
// Access as: http.Get(...), http.Post(...)
```

### Aliased Import

```ish
use net/http as h
// Access as: h.Get(...), h.Post(...)
```

### Selective Import

```ish
use net/http { Get, Post }
// Access directly: Get(...), Post(...)
```

### Selective Import with Rename

```ish
use net/http { Get as HttpGet }
// Access as: HttpGet(...)
```

### Qualified Access Without `use`

You can also access a module by its full path without importing it:

```ish
net/http.Get("https://example.com")
```

This resolves the module at point of use without bringing names into scope.

---

## `declare { }` Blocks

A `declare { }` block lets you define mutually recursive functions:

```ish
declare {
  fn is_even(n: Int) -> Bool {
    if n == 0 { true } else { is_odd(n - 1) }
  }
  fn is_odd(n: Int) -> Bool {
    if n == 0 { false } else { is_even(n - 1) }
  }
}

println(is_even(4))  // true
println(is_odd(3))   // true
```

### Rules

- Only declarations (function definitions, type definitions) are allowed inside `declare { }`. Top-level commands like `println(...)` produce error E020.
- Functions inside the block can call each other freely, including cyclically.
- Two separate `declare { }` blocks in the same file cannot reference each other.

### REPL Usage

In the REPL, mutual recursion must be submitted as one unit. Use a `declare { }` block:

```
ish> declare {
...    fn is_even(n: Int) -> Bool { if n == 0 { true } else { is_odd(n - 1) } }
...    fn is_odd(n: Int) -> Bool { if n == 0 { false } else { is_even(n - 1) } }
...  }
ish> is_even(10)
true
```

---

## The `bootstrap` Directive

Standalone scripts — files not under any `project.json` — can use `bootstrap` to configure dependencies and settings:

```ish
#!/usr/bin/env ish
bootstrap { "ish": ">=1.0", "dependencies": { "example.com/http": "v1.2.3" } }

use example.com/http { Get }
println(Get("https://api.example.com/data"))
```

`bootstrap` grants access to the `pub` APIs of listed packages. It does **not** grant `pkg` access to any project.

If a file is already inside a project (under a `project.json`), using `bootstrap` is an error (E021). Project members get their configuration from `project.json`.

> **Note:** In the current prototype, `bootstrap` only checks that you're not inside a project. Config parsing and dependency resolution are deferred to a future version.

---

## Interface Files

Interface files (`.ishi`) lock down a module's public API. They are optional but, once present, enforced by the compiler.

### Generating Interface Files

```bash
ish interface freeze              # generates .ishi for all modules under src/
ish interface freeze net/http     # generates .ishi for a single module
```

This creates a `.ishi` file next to each `.ish` file containing only the `pub` declarations (signatures, no bodies):

```ish
// src/math.ishi — generated by ish interface freeze
pub fn add(a: Int, b: Int) -> Int
pub fn multiply(a: Int, b: Int) -> Int
```

### Enforcement

Once committed, the `.ishi` file is enforced:

| Condition | Error |
|-----------|-------|
| Symbol in `.ishi` missing from `.ish` | E022 — `InterfaceSymbolNotInImplementation` |
| `pub` symbol in `.ish` missing from `.ishi` | E023 — `InterfaceSymbolNotInInterface` |
| Signatures differ | E024 — `InterfaceSymbolMismatch` |

To update the interface after changing `pub` declarations, run `ish interface freeze` again.

---

## Common Mistakes

### Importing a Script File (E018)

```ish
// scripts/main contains: println("hello")
use main  // ERROR E018: module/script-not-importable
```

Files with top-level commands cannot be imported. Move declarations to a module under `src/`.

### Path Conflict (E019)

Having both `src/foo.ish` and `src/foo/index.ish` is an error — they both resolve to module `foo`. Remove one.

### `bootstrap` Inside a Project (E021)

```ish
// This file is under a project.json hierarchy
bootstrap { "ish": ">=1.0" }  // ERROR E021: module/bootstrap-in-project
```

Project members get configuration from `project.json`. Remove the `bootstrap` directive.

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
