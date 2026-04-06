*Extracted verbatim from [proto/ish-vm/src/](../../../../proto/ish-vm/src/) — all ModDecl match arms that must be removed when the variant is deleted from ish-ast.*

## proto/ish-vm/src/analyzer.rs line 234

```rust
        | Statement::ModDecl { .. }
```

This is in a terminal arm (returns without recursing). After removal, the arm is simply deleted.

## proto/ish-vm/src/interpreter.rs line 716

```rust
            Statement::ModDecl { .. } => {
```

This is a stub that logs a no-op. The entire arm must be deleted.

## proto/ish-vm/src/interpreter.rs line 2208

```rust
            Statement::ModDecl { .. } => Ok(ControlFlow::None),
```

This is a terminal no-op arm. The entire arm must be deleted.

## proto/ish-vm/src/reflection.rs line 143

```rust
        Statement::ModDecl { name, body, .. } => {
```

This is in a reflection function that converts AST nodes to ish Values. The entire arm must be deleted.

## New arms required

After adding `Statement::DeclareBlock` and `Statement::Bootstrap`, every exhaustive match on `Statement` in these VM files must add arms for the new variants. For A-1 scope, add unreachable/no-op arms only:

- `Statement::DeclareBlock { .. } => { /* handled in A-2 */ }` (interpreter) or `| Statement::DeclareBlock { .. }` (terminal arms)
- `Statement::Bootstrap { .. } => { /* handled in A-2 */ }` (interpreter) or `| Statement::Bootstrap { .. }` (terminal arms)
