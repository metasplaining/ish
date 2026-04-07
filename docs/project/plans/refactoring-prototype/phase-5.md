# Phase 5: H2 — Fix Non-Structural Panics in ast_builder.rs

*Part of: [refactoring-prototype/overview.md](overview.md)*

## Context Files
- [context/helper-signatures.md](context/helper-signatures.md) — H2 fixes and `ParseError` struct note

## Requirements
- Integer literal parse in `build_match_pattern` returns `ParseError` instead of panicking on overflow.
- Float literal parse in `build_match_pattern` returns `ParseError` instead of panicking on overflow.
- `lines.last()` in `strip_triple_quote_literal` uses `.unwrap_or` instead of `.unwrap()`.
- `value.unwrap()` in `build_var_decl` is replaced with `ok_or_else(...)?` propagation.
- A new acceptance test verifies that `let x = 99999999999999999999` produces an error
  containing "overflows".

## Important: ParseError struct

`ParseError` in `proto/ish-parser/src/error.rs` is:
```rust
pub struct ParseError {
    pub start: usize,
    pub end: usize,
    pub message: String,
}
```
Use `ParseError::new(0, 0, message)` for all new error sites (no `span` field exists).

## Tasks

- [x] 1. In `proto/ish-parser/src/ast_builder.rs`, locate `build_match_pattern`
  (~line 1555). In the `Rule::integer_literal` arm, replace:
  ```rust
  let n: i64 = inner.as_str().parse().unwrap();
  ```
  With:
  ```rust
  let s = inner.as_str();
  let n: i64 = s.parse().map_err(|_| ParseError::new(0, 0,
      format!("integer literal '{}' overflows i64", s)
  ))?;
  ```

- [x] 2. In the same function, in the `Rule::float_literal` arm (~line 1568), replace:
  ```rust
  let n: f64 = inner.as_str().parse().unwrap();
  ```
  With:
  ```rust
  let s = inner.as_str();
  let n: f64 = s.parse().map_err(|_| ParseError::new(0, 0,
      format!("float literal '{}' is not a valid f64", s)
  ))?;
  ```

- [x] 3. Locate `strip_triple_quote_literal` (~line 1438). Replace:
  ```rust
  let last_line = lines.last().unwrap();
  ```
  With:
  ```rust
  let last_line = lines.last().copied().unwrap_or("");
  ```

- [x] 4. Locate `build_var_decl` (~line 75). Find the struct literal field `value: value.unwrap()`.
  Replace it with:
  ```rust
  value: value.ok_or_else(|| ParseError::new(0, 0,
      "variable declaration missing value".to_string()
  ))?,
  ```
  Confirm the enclosing function signature already returns `Result<Statement, ParseError>`.

- [x] 5. Add a new acceptance test for integer literal overflow. Create or append to
  `proto/ish-tests/basics/literals.sh` (or the most appropriate existing file):
  ```bash
  # Integer literal overflow
  output=$(run_ish 'let x = 99999999999999999999' 2>&1)
  assert_contains "integer literal overflow" "overflows" "$output"
  ```
  Follow the existing test file convention (check `proto/ish-tests/basics/arithmetic.sh`
  for the exact `assert_contains` helper name used in this test suite — use `assert_fail`
  or `assert_contains` as appropriate).

## Verification

Run: `cd proto && cargo test --workspace && bash ish-tests/run_all.sh`
Check: all tests pass; the new integer overflow test passes; no grammar-structural `unwrap()` calls were accidentally removed.
Invoke: `/verify refactoring-prototype/phase-5.md`
