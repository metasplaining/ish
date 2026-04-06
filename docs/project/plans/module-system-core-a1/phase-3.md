# Phase 3: ish-ast Code Changes

*Part of: [module-system-core-a1/overview.md](overview.md)*

## Context Files

- [context/ish-ast-current.md](context/ish-ast-current.md) — current Visibility, Use, ModDecl, IncompleteKind

## Requirements

- `Visibility` has exactly `Priv`, `Pkg`, `Pub` variants (no others).
- `Statement::Use` has `module_path`, `alias`, `selective` fields with a `SelectiveImport` struct.
- `Statement::ModDecl` does not exist.
- `Statement::DeclareBlock { body: Vec<Statement> }` exists.
- `Statement::Bootstrap { source: BootstrapSource }` exists with `BootstrapSource` enum.
- `IncompleteKind::DeclareBlock` exists in the Brace-delimited group.
- `has_incomplete_continuable` and `has_any_incomplete` compile with no match-exhaustiveness warnings.
- `display.rs` renders all new/changed nodes.
- `builder.rs` has helpers for `UseDirective`, `DeclareBlock`, and `Bootstrap`.

## Tasks

### proto/ish-ast/src/lib.rs

- [x] 1. Replace the `Visibility` enum — `proto/ish-ast/src/lib.rs`

  Remove:
  ```rust
  pub enum Visibility {
      Private,
      Public,
      PubScope(String),
  }
  ```

  Replace with:
  ```rust
  pub enum Visibility {
      Priv,   // priv — current module only
      Pkg,    // pkg — all project members (default when omitted)
      Pub,    // pub — external dependents
  }
  ```

- [x] 2. Add `SelectiveImport` struct — `proto/ish-ast/src/lib.rs`

  Add before or after the `Visibility` enum:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub struct SelectiveImport {
      pub name: String,
      pub alias: Option<String>,
  }
  ```

- [x] 3. Replace `Statement::Use` variant — `proto/ish-ast/src/lib.rs`

  Remove:
  ```rust
      Use {
          path: Vec<String>,
      },
  ```

  Replace with:
  ```rust
      Use {
          module_path: Vec<String>,
          alias: Option<String>,
          selective: Option<Vec<SelectiveImport>>,
      },
  ```

- [x] 4. Remove `Statement::ModDecl` variant — `proto/ish-ast/src/lib.rs`

  Delete entirely:
  ```rust
      ModDecl {
          name: String,
          body: Option<Box<Statement>>,
          visibility: Option<Visibility>,
      },
  ```

- [x] 5. Add `Statement::DeclareBlock` variant — `proto/ish-ast/src/lib.rs`

  Add after `Statement::Use`:
  ```rust
      DeclareBlock {
          body: Vec<Statement>,
      },
  ```

- [x] 6. Add `BootstrapSource` enum and `Statement::Bootstrap` variant — `proto/ish-ast/src/lib.rs`

  Add the enum near `Visibility` / `SelectiveImport`:
  ```rust
  #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
  pub enum BootstrapSource {
      Path(String),
      Url(String),
      Inline(String),
  }
  ```

  Add the variant after `Statement::DeclareBlock`:
  ```rust
      Bootstrap {
          source: BootstrapSource,
      },
  ```

- [x] 7. Add `IncompleteKind::DeclareBlock` — `proto/ish-ast/src/lib.rs`

  In the `IncompleteKind` enum, add to the Brace-delimited group:
  ```rust
      DeclareBlock,
  ```

  The group comment `// Brace-delimited (5)` should be updated to `(6)`.

- [x] 8. Update `has_incomplete_continuable` match — `proto/ish-ast/src/lib.rs`

  In the terminal arm, remove `| Statement::ModDecl { .. }` and add the new variants. The updated arm:
  ```rust
              Statement::ShellCommand { .. }
              | Statement::TypeAlias { .. }
              | Statement::Use { .. }
              | Statement::DeclareBlock { .. }
              | Statement::Bootstrap { .. }
              | Statement::StandardDef { .. }
              | Statement::EntryTypeDef { .. }
              | Statement::Yield => false,
  ```

  Also add `DeclareBlock` recursion — add a new arm before the terminal:
  ```rust
              Statement::DeclareBlock { body } => {
                  body.iter().any(|s| s.has_incomplete_continuable())
              }
  ```

- [x] 9. Update `has_any_incomplete` match — `proto/ish-ast/src/lib.rs`

  Same changes as task 8, but for `has_any_incomplete`. Terminal arm removes `ModDecl`, adds `DeclareBlock { .. }` and `Bootstrap { .. }`. Recursive arm for `DeclareBlock`:
  ```rust
              Statement::DeclareBlock { body } => {
                  body.iter().any(|s| s.has_any_incomplete())
              }
  ```

### proto/ish-ast/src/display.rs

- [x] 10. Remove `Statement::ModDecl` arm from `StmtDisplay` — `proto/ish-ast/src/display.rs`

  Delete the arm:
  ```rust
              Statement::ModDecl { name, body, .. } => {
                  indent(f, d)?;
                  if let Some(b) = body {
                      write!(f, "mod {} {}", name, StmtDisplay(b, d))
                  } else {
                      write!(f, "mod {}", name)
                  }
              }
  ```

- [x] 11. Update `Statement::Use` arm in `StmtDisplay` — `proto/ish-ast/src/display.rs`

  Replace the current `Statement::Use { path }` arm with one that handles all three fields:
  ```rust
              Statement::Use { module_path, alias, selective } => {
                  indent(f, d)?;
                  write!(f, "use {}", module_path.join("/"))?;
                  if let Some(a) = alias {
                      write!(f, " as {}", a)?;
                  }
                  if let Some(sel) = selective {
                      write!(f, " {{")?;
                      for (i, s) in sel.iter().enumerate() {
                          if i > 0 { write!(f, ", ")?; }
                          write!(f, "{}", s.name)?;
                          if let Some(a) = &s.alias {
                              write!(f, " as {}", a)?;
                          }
                      }
                      write!(f, "}}")?;
                  }
                  Ok(())
              }
  ```

- [x] 12. Add `Statement::DeclareBlock` arm to `StmtDisplay` — `proto/ish-ast/src/display.rs`

  ```rust
              Statement::DeclareBlock { body } => {
                  indent(f, d)?;
                  writeln!(f, "declare {{")?;
                  for s in body {
                      writeln!(f, "{}", StmtDisplay(s, d + 1))?;
                  }
                  indent(f, d)?;
                  write!(f, "}}")
              }
  ```

- [x] 13. Add `Statement::Bootstrap` arm to `StmtDisplay` — `proto/ish-ast/src/display.rs`

  ```rust
              Statement::Bootstrap { source } => {
                  indent(f, d)?;
                  match source {
                      BootstrapSource::Path(p) => write!(f, "bootstrap '{}'", p),
                      BootstrapSource::Url(u) => write!(f, "bootstrap '{}'", u),
                      BootstrapSource::Inline(json) => write!(f, "bootstrap {{{}}}", json),
                  }
              }
  ```

### proto/ish-ast/src/builder.rs

- [x] 14. Add `use_directive` builder helper to `ProgramBuilder` — `proto/ish-ast/src/builder.rs`

  ```rust
      pub fn use_directive(self, module_path: Vec<impl Into<String>>) -> Self {
          self.stmt(Statement::Use {
              module_path: module_path.into_iter().map(|s| s.into()).collect(),
              alias: None,
              selective: None,
          })
      }
  ```

- [x] 15. Add `declare_block` builder helper to `ProgramBuilder` — `proto/ish-ast/src/builder.rs`

  ```rust
      pub fn declare_block(self, body: Vec<Statement>) -> Self {
          self.stmt(Statement::DeclareBlock { body })
      }
  ```

- [x] 16. Add `bootstrap` builder helper to `ProgramBuilder` — `proto/ish-ast/src/builder.rs`

  ```rust
      pub fn bootstrap(self, source: BootstrapSource) -> Self {
          self.stmt(Statement::Bootstrap { source })
      }
  ```

## Verification

Run: `cd proto && cargo build -p ish-ast 2>&1`

Check: Build succeeds with zero errors. No `ModDecl` references remain. No unused import warnings.

Run: `cd proto && cargo test -p ish-ast 2>&1`

Check: All existing tests pass.

Invoke: `/verify module-system-core-a1/phase-3.md`
