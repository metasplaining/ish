# Phase 6: H1 — Extract Six Interpreter Helpers

*Part of: [refactoring-prototype/overview.md](overview.md)*

## Context Files
- [context/helper-signatures.md](context/helper-signatures.md) — all six helper function bodies

## Requirements
- Six free functions exist in `interpreter.rs` below `impl IshVm { }` and above `#[cfg(test)]`:
  `eval_literal`, `eval_unary_op`, `apply_property_read`, `apply_index_read`,
  `apply_property_write`, `apply_index_write`.
- `eval_expression_yielding`: `Expression::Literal` arm calls `eval_literal`; `Expression::UnaryOp`
  inner match calls `eval_unary_op`; `Expression::PropertyAccess` calls `apply_property_read`;
  `Expression::IndexAccess` calls `apply_index_read`.
- `eval_expression_unyielding`: same four replacements.
- `exec_statement_yielding`: `AssignTarget::Property` calls `apply_property_write`;
  `AssignTarget::Index` calls `apply_index_write`.
- `exec_statement_unyielding`: same two replacements.
- Behaviour is unchanged for all inputs. All existing tests pass.

## Important: Apply After Phase 1 and Phase 2

Phases 1 and 2 also modify `interpreter.rs`. Complete them first to avoid merge conflicts.

## Tasks

- [x] 1. Add all six helper functions to `proto/ish-vm/src/interpreter.rs`. Place them
  between the closing `}` of `impl IshVm` and the `#[cfg(test)]` section. Use the exact
  bodies from [context/helper-signatures.md](context/helper-signatures.md) §H1.

- [x] 2. In `eval_expression_yielding` (~line 944), find the `Expression::Literal(lit)` arm:
  ```rust
  Expression::Literal(lit) => Ok(match lit {
      Literal::Bool(b) => Value::Bool(*b),
      // ... six arms ...
      Literal::Null => Value::Null,
  }),
  ```
  Replace with:
  ```rust
  Expression::Literal(lit) => Ok(eval_literal(lit)),
  ```

- [x] 3. In `eval_expression_unyielding` (~line 2622), find the same `Expression::Literal(lit)`
  arm and apply the same replacement.

- [x] 4. In `eval_expression_yielding` (~line 979), find the `Expression::UnaryOp { op, operand }` arm.
  After `let val = Self::eval_expression_yielding(...).await?;`, the current code is a `match op { ... }` block.
  Replace the `match op { ... }` block with:
  ```rust
  eval_unary_op(op, val)
  ```
  (The arm body becomes: `let val = ...; eval_unary_op(op, val)`)

- [x] 5. In `eval_expression_unyielding` (~line 2654), find the `Expression::UnaryOp { op, operand }` arm
  and apply the same replacement (using the sync evaluator for `val`).

- [x] 6. In `eval_expression_yielding` (~line 1070), find the `Expression::PropertyAccess { object, property }` arm.
  After `let obj = Self::eval_expression_yielding(...).await?;`, replace the `match obj { ... }` block with:
  ```rust
  apply_property_read(obj, property)
  ```

- [x] 7. In `eval_expression_unyielding` (~line 2701), find the `Expression::PropertyAccess` arm
  and apply the same replacement.

- [x] 8. In `eval_expression_yielding` (~line 1085), find the `Expression::IndexAccess { object, index }` arm.
  After evaluating `obj` and `idx`, replace the `match (&obj, &idx) { ... }` block with:
  ```rust
  apply_index_read(obj, idx)
  ```

- [x] 9. In `eval_expression_unyielding` (~line 2716), find the `Expression::IndexAccess` arm
  and apply the same replacement.

- [x] 10. In `exec_statement_yielding` (~line 253), find `Statement::Assignment { target, value }`.
  Locate the `AssignTarget::Property { object, property }` arm. After evaluating `obj`, replace:
  ```rust
  if let Value::Object(ref obj_ref) = obj {
      obj_ref.borrow_mut().insert(property.clone(), val);
  } else {
      return Err(RuntimeError::system_error(format!(...), ErrorCode::TypeMismatch));
  }
  ```
  With:
  ```rust
  apply_property_write(&obj, property, val)?;
  ```

- [x] 11. In `exec_statement_yielding`, locate the `AssignTarget::Index { object, index }` arm.
  After evaluating `obj` and `idx`, replace the entire `if let Value::List ... else if let Value::Object ... else { Err(...) }` block with:
  ```rust
  apply_index_write(&obj, &idx, val)?;
  ```

- [x] 12. In `exec_statement_unyielding` (~line 2117), find the same `Statement::Assignment`
  match. Apply the same two replacements (tasks 10 and 11) for the unyielding variant.

## Verification

Run: `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`
Check: all tests pass (317 unit tests + 255 acceptance tests). Spot-check that the four
`eval_expression_*` / `exec_statement_*` functions are visibly shorter and reference the
helper names.
Invoke: `/verify refactoring-prototype/phase-6.md`
