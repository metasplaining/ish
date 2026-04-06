*Extracted verbatim from [proto/ish-parser/src/ast_builder.rs](../../../../proto/ish-parser/src/ast_builder.rs) §build_visibility, §build_use_stmt, §build_mod_stmt, §build_statement dispatch.*

## build_statement dispatch (current — relevant module entries)

```rust
fn build_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    match pair.as_rule() {
        // ...
        Rule::use_stmt => build_use_stmt(pair),
        Rule::mod_stmt => build_mod_stmt(pair),
        // ...
    }
}
```

## build_visibility (current — to be rewritten)

```rust
fn build_visibility(pair: Pair<Rule>) -> Visibility {
    let inner: Vec<_> = pair.into_inner().collect();
    if inner.is_empty() {
        Visibility::Public
    } else {
        Visibility::PubScope(inner[0].as_str().to_string())
    }
}
```

## build_use_stmt (current — to be replaced)

```rust
fn build_use_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let module_path = pair.into_inner().next().unwrap();
    let path: Vec<String> = module_path
        .into_inner()
        .map(|p| p.as_str().to_string())
        .collect();
    Ok(Statement::Use { path })
}
```

## build_mod_stmt (current — to be removed)

```rust
fn build_mod_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner().peekable();

    let visibility = if inner.peek().map(|p| p.as_rule()) == Some(Rule::pub_modifier) {
        Some(build_visibility(inner.next().unwrap()))
    } else {
        None
    };
    // ...
    Ok(Statement::ModDecl { name, body, visibility })
}
```

## Locations where pub_modifier is checked (all must change to visibility rule name)

Lines 86, 175, 539 of ast_builder.rs check `Rule::pub_modifier`. After the grammar rename, these must check `Rule::visibility` instead.
