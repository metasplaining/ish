# Phase 6: Unit Tests and Final Verification

*Part of: [module-system-core-a1/overview.md](overview.md)*

## Context Files

None required — test content is fully specified here.

## Requirements

- ish-ast has unit tests for: `Visibility` JSON round-trip, `Statement::Use` (plain, aliased, selective), `Statement::DeclareBlock` construction.
- ish-parser `phase5.rs` has the full test suite for new module constructs (if not already written in Phase 4, task 16).
- `cargo test --workspace` passes with zero failures.
- `cargo run -p ish-shell` runs the 6 end-to-end demo verifications successfully.

## Tasks

### proto/ish-ast/src/lib.rs — add tests in the `#[cfg(test)]` block

- [x] 1. Add `test_visibility_roundtrip` — `proto/ish-ast/src/lib.rs`

  ```rust
  #[test]
  fn test_visibility_roundtrip() {
      for vis in [Visibility::Priv, Visibility::Pkg, Visibility::Pub] {
          let json = serde_json::to_string(&vis).unwrap();
          let parsed: Visibility = serde_json::from_str(&json).unwrap();
          assert_eq!(vis, parsed);
      }
  }
  ```

- [x] 2. Add `test_use_plain_roundtrip` — `proto/ish-ast/src/lib.rs`

  ```rust
  #[test]
  fn test_use_plain_roundtrip() {
      let stmt = Statement::Use {
          module_path: vec!["net".into(), "http".into()],
          alias: None,
          selective: None,
      };
      let json = serde_json::to_string(&stmt).unwrap();
      let parsed: Statement = serde_json::from_str(&json).unwrap();
      assert_eq!(stmt, parsed);
  }
  ```

- [x] 3. Add `test_use_aliased_roundtrip` — `proto/ish-ast/src/lib.rs`

  ```rust
  #[test]
  fn test_use_aliased_roundtrip() {
      let stmt = Statement::Use {
          module_path: vec!["net".into(), "http".into()],
          alias: Some("h".into()),
          selective: None,
      };
      let json = serde_json::to_string(&stmt).unwrap();
      let parsed: Statement = serde_json::from_str(&json).unwrap();
      assert_eq!(stmt, parsed);
  }
  ```

- [x] 4. Add `test_use_selective_roundtrip` — `proto/ish-ast/src/lib.rs`

  ```rust
  #[test]
  fn test_use_selective_roundtrip() {
      let stmt = Statement::Use {
          module_path: vec!["net".into(), "http".into()],
          alias: None,
          selective: Some(vec![
              SelectiveImport { name: "Client".into(), alias: None },
              SelectiveImport { name: "Request".into(), alias: Some("Req".into()) },
          ]),
      };
      let json = serde_json::to_string(&stmt).unwrap();
      let parsed: Statement = serde_json::from_str(&json).unwrap();
      assert_eq!(stmt, parsed);
  }
  ```

- [x] 5. Add `test_declare_block_construction` — `proto/ish-ast/src/lib.rs`

  ```rust
  #[test]
  fn test_declare_block_construction() {
      let block = Statement::DeclareBlock {
          body: vec![
              Statement::function_decl("even", vec![Parameter::new("n")], Statement::block(vec![])),
              Statement::function_decl("odd", vec![Parameter::new("n")], Statement::block(vec![])),
          ],
      };
      let json = serde_json::to_string(&block).unwrap();
      let parsed: Statement = serde_json::from_str(&json).unwrap();
      assert_eq!(block, parsed);
  }
  ```

### proto/ish-parser/tests/phase5.rs — verify or write all tests

- [x] 6. Verify that `phase5.rs` contains all 16 tests specified in Phase 4, task 16. If any are missing, add them now.

  Full test list:
  - `parse_priv_fn`: `parse("priv fn f() {}").unwrap()` → `FunctionDecl { visibility: Some(Visibility::Priv), .. }`
  - `parse_pkg_fn`: `parse("pkg fn f() {}").unwrap()` → `FunctionDecl { visibility: Some(Visibility::Pkg), .. }`
  - `parse_pub_fn`: `parse("pub fn f() {}").unwrap()` → `FunctionDecl { visibility: Some(Visibility::Pub), .. }`
  - `parse_no_visibility_fn`: `parse("fn f() {}").unwrap()` → `FunctionDecl { visibility: None, .. }`
  - `parse_priv_let`: `parse("priv let x = 1").unwrap()` → `VariableDecl { visibility: Some(Visibility::Priv), .. }`
  - `parse_pub_type`: `parse("pub type Id = int").unwrap()` → `TypeAlias { visibility: Some(Visibility::Pub), .. }`
  - `parse_use_plain`: `parse("use foo/bar").unwrap()` → `Use { module_path: ["foo", "bar"], alias: None, selective: None }`
  - `parse_use_aliased`: `parse("use foo/bar as b").unwrap()` → `Use { alias: Some("b"), .. }`
  - `parse_use_selective`: `parse("use foo/bar { Type }").unwrap()` → `Use { selective: Some([SelectiveImport { name: "Type", alias: None }]), .. }`
  - `parse_use_selective_rename`: `parse("use foo/bar { Type as T }").unwrap()` → selective entry with `alias: Some("T")`
  - `parse_use_external`: `parse("use example.com/foo/bar").unwrap()` → `Use { module_path: ["example.com", "foo", "bar"], .. }` (or however domain_segment is serialized)
  - `parse_declare_block`: `parse("declare {\n  fn a() {}\n  fn b() {}\n}").unwrap()` → `DeclareBlock` with 2 children
  - `parse_bootstrap_path`: `parse("bootstrap 'path/to/cfg.json'").unwrap()` → `Bootstrap { source: BootstrapSource::Path("path/to/cfg.json") }`
  - `parse_bootstrap_url`: `parse("bootstrap 'https://example.com/cfg.json'").unwrap()` → `Bootstrap { source: BootstrapSource::Url("https://example.com/cfg.json") }`
  - `parse_bootstrap_inline`: `parse(r#"bootstrap {"ish": ">=1.0"}"#).unwrap()` → `Bootstrap { source: BootstrapSource::Inline(r#""ish": ">=1.0""#) }` (inner content only)
  - `parse_incomplete_declare`: `parse("declare {").unwrap()` → `Incomplete { kind: IncompleteKind::DeclareBlock }`

## Final Verification

Run: `cd proto && cargo test --workspace 2>&1`

Check: All tests pass. Test count should be at least as high as before (317+) given new tests were added.

Run: `cd proto && cargo run -p ish-shell 2>&1`

Check: Six end-to-end demo verifications all pass. No panics.

Run: `cd proto && bash ish-tests/run_all.sh 2>&1 | tail -5`

Check: 255 acceptance tests pass (or current baseline count).

Run: `grep -r "ModDecl\|pub_modifier\|PubScope\|Visibility::Public\|Visibility::Private" proto/ish-ast proto/ish-parser proto/ish-vm proto/ish-stdlib proto/ish-codegen 2>&1`

Check: No output — none of these identifiers exist anywhere in the workspace.

Invoke: `/verify module-system-core-a1/phase-6.md`
