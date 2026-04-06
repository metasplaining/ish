# Phase 4: ish-parser — Grammar and AST Builder

*Part of: [module-system-core-a1/overview.md](overview.md)*

## Context Files

- [context/grammar-modules-current.md](context/grammar-modules-current.md) — current grammar module rules
- [context/ast-builder-current.md](context/ast-builder-current.md) — current build_visibility, build_use_stmt, build_mod_stmt

## Requirements

- Grammar uses `visibility` rule (not `pub_modifier`). `priv`, `pkg`, `pub` are all valid.
- Module paths use `/` separator. External paths use domain segment (`foo.bar/...`).
- `use_stmt` parses all four forms (plain, aliased, selective, selective-with-rename).
- `declare_block` rule exists. `mod_stmt` does not exist.
- `bootstrap_stmt` rule exists for string-literal and inline-JSON forms.
- `declare_block` and `bootstrap_stmt` are in the `statement` rule.
- `mod_stmt` is removed from `statement` and `annotated_stmt`.
- Keywords include `priv`, `pkg`, `declare`, `bootstrap`. `mod` is not a keyword.
- `IncompleteKind::DeclareBlock` is produced for unterminated `declare {`.
- `build_visibility` maps string → correct `Visibility` variant.
- `build_use_stmt` produces `Statement::Use` with all three fields.
- `build_mod_stmt` is removed. `build_declare_block` and `build_bootstrap_stmt` are added.
- All `pub_modifier` rule checks in `ast_builder.rs` are changed to `visibility` rule checks.
- `phase5.rs` tests for `ModDecl` and old `Visibility` are removed and replaced with correct tests for new constructs.

## Tasks

### proto/ish-parser/src/ish.pest

- [x] 1. Replace `pub_modifier` with `visibility` rule — `proto/ish-parser/src/ish.pest`

  Remove:
  ```pest
  pub_modifier = { "pub" ~ ("(" ~ identifier ~ ")")? }
  ```

  Replace with:
  ```pest
  visibility = { "priv" | "pkg" | "pub" }
  ```

- [x] 2. Update `let_stmt`, `fn_decl`, `type_alias` to use `visibility?` — `proto/ish-parser/src/ish.pest`

  In each of these three rules, replace `pub_modifier?` with `visibility?`.

  ```pest
  let_stmt = { visibility? ~ "let" ~ mut_kw? ~ identifier ~ (":" ~ type_annotation)? ~ "=" ~ expression }

  fn_decl = {
      visibility? ~ async_kw? ~ "fn" ~ identifier ~ generic_params? ~ "(" ~ param_list? ~ ")" ~ ("->" ~ type_annotation)? ~ block
  }

  type_alias = { visibility? ~ "type" ~ identifier ~ "=" ~ type_annotation }
  ```

- [x] 3. Replace the `module_path` rule — `proto/ish-parser/src/ish.pest`

  Remove:
  ```pest
  module_path = { identifier ~ ("::" ~ identifier)* }
  ```

  Replace with:
  ```pest
  module_path = {
      domain_segment ~ ("/" ~ path_segment)* |
      path_segment ~ ("/" ~ path_segment)*
  }
  domain_segment = @{ identifier ~ "." ~ identifier ~ ("." ~ identifier)* }
  path_segment   = @{ ".." | "." | identifier }
  ```

- [x] 4. Replace `use_stmt` rule — `proto/ish-parser/src/ish.pest`

  Remove:
  ```pest
  use_stmt = { "use" ~ module_path }
  ```

  Replace with:
  ```pest
  use_stmt = {
      "use" ~ module_path ~ "{" ~ selective_import_list ~ "}" |
      "use" ~ module_path ~ ("as" ~ identifier)?
  }
  selective_import_list = { selective_import ~ ("," ~ selective_import)* ~ ","? }
  selective_import      = { identifier ~ ("as" ~ identifier)? }
  ```

  Note: The selective form must come first (before the plain/aliased form) because PEG parsers are ordered-choice — both begin with `"use" ~ module_path`, but the `{` distinguishes the selective form. Placing it first ensures correct disambiguation.

- [x] 5. Replace `mod_stmt` with `declare_block` — `proto/ish-parser/src/ish.pest`

  Remove:
  ```pest
  mod_stmt = { pub_modifier? ~ "mod" ~ identifier ~ block? }
  ```

  Replace with:
  ```pest
  declare_block           = { "declare" ~ "{" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)* ~ stmt_sep?)? ~ "}" }
  unterminated_declare_block = { "declare" ~ "{" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)* ~ stmt_sep?)? ~ EOI }
  ```

- [x] 6. Add `bootstrap_stmt` rule — `proto/ish-parser/src/ish.pest`

  Add after `declare_block`:
  ```pest
  bootstrap_stmt     = { "bootstrap" ~ (string_literal | inline_json_object) }
  inline_json_object = { "{" ~ json_content ~ "}" }
  json_content       = @{ (!("{" | "}") ~ ANY | "{" ~ json_content ~ "}")* }
  ```

  Note: `json_content` is recursive to handle nested JSON objects. The `@` makes the inner content atomic.

  Correction: `json_content` must be non-atomic to allow recursion. Use:
  ```pest
  bootstrap_stmt     = { "bootstrap" ~ (string_literal | inline_json_object) }
  inline_json_object = { "{" ~ json_content ~ "}" }
  json_content       = { (!("{" | "}") ~ ANY | inline_json_object)* }
  ```

- [x] 7. Update `statement` rule — remove `mod_stmt`, add `declare_block` and `bootstrap_stmt` — `proto/ish-parser/src/ish.pest`

  Replace the `statement` rule body so that:
  - `mod_stmt |` is removed
  - `declare_block |` is added (place before `assign_stmt`)
  - `bootstrap_stmt |` is added (place before `assign_stmt`)
  - `unterminated_declare_block` is added alongside `unterminated_block`

  Updated `statement` rule:
  ```pest
  statement = _{
      annotated_stmt |
      standard_def |
      entry_type_def |
      fn_decl |
      let_stmt |
      if_stmt |
      while_stmt |
      for_stmt |
      return_stmt |
      yield_stmt |
      throw_stmt |
      try_catch |
      with_block |
      defer_stmt |
      match_stmt |
      type_alias |
      use_stmt |
      declare_block |
      bootstrap_stmt |
      assign_stmt |
      expression_stmt |
      shell_command |
      unterminated_declare_block |
      unterminated_block
  }
  ```

- [x] 8. Update `annotated_stmt` — remove `mod_stmt` reference — `proto/ish-parser/src/ish.pest`

  ```pest
  annotated_stmt = { (annotation ~ NEWLINE*)+ ~ (fn_decl | let_stmt | type_alias | block | while_stmt | for_stmt) }
  ```

- [x] 9. Update `keyword` rule — `proto/ish-parser/src/ish.pest`

  Remove `"mod"`. Add `"priv"`, `"pkg"`, `"declare"`, `"bootstrap"`.

  ```pest
  keyword = {
      ("let" | "mut" | "fn" | "if" | "else" | "while" | "for" | "in" |
       "return" | "true" | "false" | "null" | "and" | "or" | "not" |
       "match" | "use" | "pub" | "pkg" | "priv" | "type" | "standard" | "entry" |
       "try" | "catch" | "finally" | "throw" | "with" | "defer" |
       "async" | "await" | "spawn" | "yield" | "declare" | "bootstrap" |
       "break" | "continue") ~ !(ASCII_ALPHANUMERIC | "_")
  }
  ```

### proto/ish-parser/src/ast_builder.rs

- [x] 10. Rewrite `build_visibility` — `proto/ish-parser/src/ast_builder.rs`

  Replace the function body:
  ```rust
  fn build_visibility(pair: Pair<Rule>) -> Visibility {
      match pair.as_str() {
          "priv" => Visibility::Priv,
          "pkg"  => Visibility::Pkg,
          "pub"  => Visibility::Pub,
          other  => panic!("unexpected visibility keyword: {}", other),
      }
  }
  ```

- [x] 11. Update all `Rule::pub_modifier` checks to `Rule::visibility` — `proto/ish-parser/src/ast_builder.rs`

  There are three locations (lines 86, 175, 539 in the original file). In each, change:
  ```rust
  if inner.peek().map(|p| p.as_rule()) == Some(Rule::pub_modifier) {
  ```
  to:
  ```rust
  if inner.peek().map(|p| p.as_rule()) == Some(Rule::visibility) {
  ```

- [x] 12. Rewrite `build_use_stmt` — `proto/ish-parser/src/ast_builder.rs`

  ```rust
  fn build_use_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
      let mut inner = pair.into_inner().peekable();
      let path_pair = inner.next().unwrap(); // module_path
      let module_path: Vec<String> = path_pair
          .into_inner()
          .map(|p| p.as_str().to_string())
          .collect();

      let mut alias = None;
      let mut selective = None;

      if let Some(next) = inner.peek() {
          if next.as_rule() == Rule::selective_import_list {
              let list_pair = inner.next().unwrap();
              selective = Some(
                  list_pair
                      .into_inner()
                      .map(|si| {
                          let mut parts = si.into_inner();
                          let name = parts.next().unwrap().as_str().to_string();
                          let alias = parts.next().map(|p| p.as_str().to_string());
                          SelectiveImport { name, alias }
                      })
                      .collect(),
              );
          } else if next.as_rule() == Rule::identifier {
              alias = Some(inner.next().unwrap().as_str().to_string());
          }
      }

      Ok(Statement::Use { module_path, alias, selective })
  }
  ```

- [x] 13. Remove `build_mod_stmt` — `proto/ish-parser/src/ast_builder.rs`

  Delete the entire `build_mod_stmt` function. Remove the `Rule::mod_stmt => build_mod_stmt(pair),` dispatch entry from `build_statement`.

- [x] 14. Add `build_declare_block` — `proto/ish-parser/src/ast_builder.rs`

  ```rust
  fn build_declare_block(pair: Pair<Rule>) -> Result<Statement, ParseError> {
      let mut body = Vec::new();
      for inner in pair.into_inner() {
          body.push(build_statement(inner)?);
      }
      Ok(Statement::DeclareBlock { body })
  }
  ```

  Add dispatch in `build_statement`:
  ```rust
          Rule::declare_block => build_declare_block(pair),
          Rule::unterminated_declare_block => Ok(Statement::Incomplete { kind: IncompleteKind::DeclareBlock }),
  ```

- [x] 15. Add `build_bootstrap_stmt` — `proto/ish-parser/src/ast_builder.rs`

  ```rust
  fn build_bootstrap_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
      let inner = pair.into_inner().next().unwrap();
      let source = match inner.as_rule() {
          Rule::string_literal => {
              let s = inner.into_inner().next().unwrap().as_str().to_string();
              if s.starts_with("https://") {
                  BootstrapSource::Url(s)
              } else {
                  BootstrapSource::Path(s)
              }
          }
          Rule::inline_json_object => {
              let content = inner.into_inner().next().unwrap().as_str().to_string();
              BootstrapSource::Inline(content)
          }
          rule => {
              let span = inner.as_span();
              return Err(ParseError::new(
                  span.start(), span.end(),
                  format!("unexpected bootstrap source rule: {:?}", rule),
              ));
          }
      };
      Ok(Statement::Bootstrap { source })
  }
  ```

  Add dispatch in `build_statement`:
  ```rust
          Rule::bootstrap_stmt => build_bootstrap_stmt(pair),
  ```

### proto/ish-parser/tests/phase5.rs

- [x] 16. Replace `phase5.rs` — `proto/ish-parser/tests/phase5.rs`

  The existing file tests old constructs (`ModDecl`, `Visibility::Public`, `PubScope`, `path` field, `::` separators). Replace the entire file with tests for the new constructs. Keep the file named `phase5.rs`.

  The new file must contain these tests (see Phase 6 for full test content — add them here directly so they live in phase5.rs):

  - `parse_priv_fn` — `priv fn f() {}` → `Visibility::Priv`
  - `parse_pkg_fn` — `pkg fn f() {}` → `Visibility::Pkg`
  - `parse_pub_fn` — `pub fn f() {}` → `Visibility::Pub`
  - `parse_no_visibility_fn` — `fn f() {}` → `visibility == None`
  - `parse_priv_let` — `priv let x = 1` → `Visibility::Priv`
  - `parse_pub_type` — `pub type Id = int` → `Visibility::Pub`
  - `parse_use_plain` — `use foo/bar` → `module_path == ["foo", "bar"]`, alias None, selective None
  - `parse_use_aliased` — `use foo/bar as b` → alias == Some("b")
  - `parse_use_selective` — `use foo/bar { Type }` → selective with one entry
  - `parse_use_selective_rename` — `use foo/bar { Type as T }` → selective entry with alias
  - `parse_use_external` — `use example.com/foo/bar` → module_path includes domain segments
  - `parse_declare_block` — `declare { fn a() {} fn b() {} }` → `DeclareBlock` with two FunctionDecl children
  - `parse_bootstrap_path` — `bootstrap 'path/to/cfg.json'` → `Bootstrap { source: BootstrapSource::Path(...) }`
  - `parse_bootstrap_url` — `bootstrap 'https://example.com/cfg.json'` → `Bootstrap { source: BootstrapSource::Url(...) }`
  - `parse_bootstrap_inline` — `bootstrap { "ish": ">=1.0" }` → `Bootstrap { source: BootstrapSource::Inline(...) }`
  - `parse_incomplete_declare` — unterminated `declare {` → `Statement::Incomplete { kind: IncompleteKind::DeclareBlock }`

## Verification

Run: `cd proto && cargo build -p ish-parser 2>&1`

Check: Build succeeds with zero errors. No `pub_modifier`, `mod_stmt`, `ModDecl`, or `PubScope` references remain.

Run: `cd proto && cargo test -p ish-parser 2>&1`

Check: All tests pass including the new phase5 tests.

Invoke: `/verify module-system-core-a1/phase-4.md`
