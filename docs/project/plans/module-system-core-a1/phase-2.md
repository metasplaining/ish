# Phase 2: Documentation

*Part of: [module-system-core-a1/overview.md](overview.md)*

## Context Files

None required — content is derived from the accepted proposal.

## Requirements

- `docs/spec/syntax.md` has a "Module Directives" section with: `use` (all four forms), `declare { }`, `bootstrap` (all three forms), visibility keywords (`priv`, `pkg`, `pub`), and a note on `index.ish`.
- `docs/architecture/ast.md` documents the updated `Visibility` enum, `Statement::Use` with its fields, `Statement::DeclareBlock`, `Statement::Bootstrap`/`BootstrapSource`, and notes the removal of `Statement::ModDecl`.

## Tasks

### docs/spec/syntax.md

- [x] 1. Add a "Module Directives" section to `docs/spec/syntax.md` — **after** the existing Visibility section (currently around line 449).

  The section must cover:

  **Visibility keywords**

  Three levels, used before `fn`, `let`, or `type`:

  ```ish
  priv fn internal() { ... }   // current module only
  fn default_fn() { ... }      // pkg — all project members (default when omitted)
  pkg fn also_default() { ... } // pkg — explicit, same as omitting
  pub fn exported() { ... }    // external dependents
  ```

  **`use` directive — four forms**

  ```ish
  use net/http                      // plain — module bound to last segment (http)
  use net/http as h                 // aliased — module bound to h
  use net/http { Client }           // selective — only Client imported
  use net/http { Client as C, Request }  // selective with rename
  use example.com/foo/bar           // external package — domain segment prefix
  ```

  Module paths use `/` as separator. The `src/` prefix is implicit and never written. `index.ish` files are imported by their directory path: `use net` resolves to `src/net/index.ish`.

  **`declare { }` block**

  ```ish
  declare {
    fn even(n: int) -> bool { if n == 0 { return true } else { return odd(n - 1) } }
    fn odd(n: int) -> bool { if n == 0 { return false } else { return even(n - 1) } }
  }
  ```

  An anonymous, declarations-only grouping. May contain `fn`, `let`, and `type` declarations. Top-level commands are rejected at evaluation time. Files loaded via `use` are implicitly wrapped in a `declare { }` block.

  **`bootstrap` directive — three forms**

  ```ish
  bootstrap "path/to/project.json"          // filesystem path
  bootstrap "https://example.com/cfg.json"  // HTTPS URL (resolved via ISH_PROXY)
  bootstrap { "ish": ">=1.0", "deps": {} }  // inline JSON
  ```

  Valid only in standalone scripts (not under any `project.json` hierarchy). Provides the same configuration as `project.json` for a single file.

  **Note on `index.ish`**

  See [docs/spec/modules.md](modules.md) for the full `index.ish` naming convention and module-to-file mapping rules.

- [x] 2. Update the existing Visibility subsection in `docs/spec/syntax.md` (around line 449) to remove the stale `pub(self)` description and reference the new Module Directives section.

- [x] 3. Update `last-verified` in `docs/spec/syntax.md` frontmatter to `2026-04-05`.

### docs/architecture/ast.md

- [x] 4. Update the `Statement` enum listing in `docs/architecture/ast.md` — remove `ModDecl`, add `DeclareBlock` and `Bootstrap`. Update `Use` to show its new fields.

  The Statement section should show:

  ```rust
  pub enum Statement {
      // ... (existing variants unchanged) ...
      TypeAlias { name, definition, visibility: Option<Visibility> },
      Use {
          module_path: Vec<String>,
          alias: Option<String>,
          selective: Option<Vec<SelectiveImport>>,
      },
      DeclareBlock {
          body: Vec<Statement>,
      },
      Bootstrap {
          source: BootstrapSource,
      },
      // ModDecl removed
  }

  pub struct SelectiveImport {
      pub name: String,
      pub alias: Option<String>,
  }

  pub enum BootstrapSource {
      Path(String),
      Url(String),
      Inline(String),
  }
  ```

- [x] 5. Update the `Visibility` enum listing in `docs/architecture/ast.md`:

  ```rust
  pub enum Visibility {
      Priv,   // priv — current module only
      Pkg,    // pkg — all project members (default when omitted)
      Pub,    // pub — external dependents
  }
  ```

  Add a note: `None` in `Option<Visibility>` means default visibility (pkg). `Priv` and `Pub` are only present when explicitly written in source.

- [x] 6. Add `IncompleteKind::DeclareBlock` to the `IncompleteKind` section in `docs/architecture/ast.md` (if that section exists), or add a note in the AST description that `DeclareBlock` is produced for unterminated `declare {`.

- [x] 7. Update `last-verified` in `docs/architecture/ast.md` frontmatter to `2026-04-05`.

## Verification

Run: `grep -n "Module Directives\|declare\|bootstrap\|module_path\|SelectiveImport\|BootstrapSource\|DeclareBlock" docs/spec/syntax.md docs/architecture/ast.md | head -30`

Check: "Module Directives" section exists in syntax.md; all four use forms are shown; DeclareBlock, Bootstrap, BootstrapSource, SelectiveImport appear in ast.md; ModDecl does not appear in ast.md.

Invoke: `/verify module-system-core-a1/phase-2.md`
