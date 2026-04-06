*Extracted verbatim from [proto/ish-ast/src/lib.rs](../../../../proto/ish-ast/src/lib.rs) §Visibility, §Statement::Use, §Statement::ModDecl, §IncompleteKind.*

## Visibility enum (current — to be replaced)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Private,
    Public,
    PubScope(String), // e.g. pub(super), pub(global)
}
```

## Statement::Use (current — to be replaced)

```rust
    Use {
        path: Vec<String>,
    },
```

## Statement::ModDecl (current — to be removed entirely)

```rust
    ModDecl {
        name: String,
        body: Option<Box<Statement>>, // None = file module, Some = inline block
        visibility: Option<Visibility>,
    },
```

## IncompleteKind (current — DeclareBlock variant must be added)

The current `IncompleteKind` enum has groups: Brace-delimited (5), Bracket-delimited (5), Paren-delimited (9), String-delimited (11), Comment / angle-bracket (3), Concurrency (2). `DeclareBlock` must be added to the Brace-delimited group.

## has_incomplete_continuable and has_any_incomplete (current — arms for ModDecl that must be updated)

Both match methods have a terminal arm that includes `Statement::ModDecl { .. }`:

```rust
            Statement::ShellCommand { .. }
            | Statement::TypeAlias { .. }
            | Statement::Use { .. }
            | Statement::ModDecl { .. }
            | Statement::StandardDef { .. }
            | Statement::EntryTypeDef { .. }
            | Statement::Yield => false,
```

When `ModDecl` is removed and `DeclareBlock` / `Bootstrap` are added, these arms must be updated.
