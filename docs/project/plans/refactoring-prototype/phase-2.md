# Phase 2: M3 — Extract `scan_to_close_brace`

*Part of: [refactoring-prototype/overview.md](overview.md)*

## Context Files
- [context/helper-signatures.md](context/helper-signatures.md) — `scan_to_close_brace` definition and call-site notes

## Requirements
- `fn scan_to_close_brace(chars: &[char], start: usize) -> Option<(String, usize)>` exists as
  a free function near `interpolate_shell_quoted` in `interpreter.rs`.
- The `${var}` brace-scanning loop is replaced with a call to `scan_to_close_brace(&chars, i + 2)`.
- The `{var}` brace-scanning loop is replaced with a call to `scan_to_close_brace(&chars, i + 1)`.
- The `name.chars().next().unwrap()` call in the `{var}` branch is replaced with
  `name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')`.
- No behaviour change for any input.

## Tasks

- [x] 1. In `proto/ish-vm/src/interpreter.rs`, add the free function `scan_to_close_brace`
  immediately before or after `interpolate_shell_quoted` (around line 3200). Use the exact
  body from [context/helper-signatures.md](context/helper-signatures.md) §M3.

- [x] 2. Locate the `${var}` branch in `interpolate_shell_quoted`. It currently reads:
  ```rust
  if i + 1 < chars.len() && chars[i + 1] == '{' {
      let mut j = i + 2;
      while j < chars.len() && chars[j] != '}' {
          j += 1;
      }
      if j < chars.len() {
          let name: String = chars[i + 2..j].iter().collect();
          out.push_str(&resolve_shell_var(&name, env));
          i = j + 1;
          continue;
      }
  }
  ```
  Replace the inner `let mut j` / `while` / `if j < chars.len()` block with:
  ```rust
  if let Some((name, next_i)) = scan_to_close_brace(&chars, i + 2) {
      out.push_str(&resolve_shell_var(&name, env));
      i = next_i;
      continue;
  }
  ```

- [x] 3. Locate the `{var}` branch in `interpolate_shell_quoted`. It currently reads:
  ```rust
  if c == '{' {
      let mut j = i + 1;
      while j < chars.len() && chars[j] != '}' {
          j += 1;
      }
      if j < chars.len() {
          let name: String = chars[i + 1..j].iter().collect();
          if !name.is_empty()
              && (name.chars().next().unwrap().is_ascii_alphabetic() || name.starts_with('_'))
              && name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
          {
              out.push_str(&resolve_shell_var(&name, env));
              i = j + 1;
              continue;
          }
      }
  }
  ```
  Replace with:
  ```rust
  if c == '{' {
      if let Some((name, next_i)) = scan_to_close_brace(&chars, i + 1) {
          if !name.is_empty()
              && name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
              && name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
          {
              out.push_str(&resolve_shell_var(&name, env));
              i = next_i;
              continue;
          }
      }
  }
  ```

## Verification

Run: `cd proto && cargo test --workspace`
Check: all tests pass; specifically the `interpolate_shell_quoted` unit tests inside `interpreter.rs` and any acceptance tests covering shell variable substitution (look in `proto/ish-tests/`).
Invoke: `/verify refactoring-prototype/phase-2.md`
