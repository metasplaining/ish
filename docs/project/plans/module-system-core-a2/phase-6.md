---
title: "Plan Phase 6: Shell — Interface Freeze and Project Root"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-06
depends-on: [docs/project/plans/module-system-core-a2/overview.md, docs/project/plans/module-system-core-a2/phase-3.md, docs/project/plans/module-system-core-a2/phase-5.md]
---

# Phase 6: Shell — Interface Freeze and Project Root

*Part of: [module-system-core-a2/overview.md](overview.md)*

Add the `ish interface freeze` subcommand and project root discovery to `ish-shell`. Depends on Phase 3 (module_loader) and Phase 5 (ProjectContext on IshVm).

## Context Files

- [context/shell-changes.md](context/shell-changes.md) — full spec for interface_cmd.rs, project root discovery, main.rs dispatch pattern

## Requirements

- `ish interface freeze` with no target argument walks `src/` under the current project root, producing `.ishi` files for all `.ish` files.
- `ish interface freeze <module_name>` processes a single module.
- Each `.ishi` file contains only `pub` function signatures and `pub type` aliases, one per line.
- Project root discovery runs before REPL launch and before file execution.
- `ProjectContext` is passed to the VM before any program runs.
- `cd proto && cargo build -p ish-shell` passes.

## Tasks

- [x] 1. Create `proto/ish-shell/src/interface_cmd.rs`:

  ```rust
  use std::path::{Path, PathBuf};
  use ish_ast::{Statement, Visibility};

  /// Run `ish interface freeze [target]`.
  ///
  /// If `target` is None, walks `<project_root>/src/` and processes all .ish files.
  /// If `target` is Some(module_name), resolves the name to a single .ish file.
  /// For each file: parse, collect pub FunctionDecl and TypeAlias, write .ishi sibling.
  pub fn freeze(target: Option<String>, project_root: &Path) {
      let src_root = project_root.join("src");
      if !src_root.exists() {
          eprintln!("error: no src/ directory found under {}", project_root.display());
          std::process::exit(1);
      }
      let files: Vec<PathBuf> = if let Some(ref mod_name) = target {
          // Resolve module_name (slash-separated) to src_root/a/b/c.ish or index.ish.
          let parts: Vec<&str> = mod_name.split('/').collect();
          let mut candidate = src_root.clone();
          for part in &parts {
              candidate.push(part);
          }
          candidate.set_extension("ish");
          if candidate.exists() {
              vec![candidate]
          } else {
              // Try index.ish
              candidate.set_extension("");
              candidate.push("index.ish");
              if candidate.exists() {
                  vec![candidate]
              } else {
                  eprintln!("error: module '{}' not found", mod_name);
                  std::process::exit(1);
              }
          }
      } else {
          // Walk src/ recursively for all .ish files.
          collect_ish_files(&src_root)
      };

      for file in files {
          process_file(&file, &src_root);
      }
  }

  fn collect_ish_files(dir: &Path) -> Vec<PathBuf> {
      // Walk dir recursively. Collect all entries with .ish extension.
      // Use std::fs::read_dir + recursion. No external crate needed.
      let mut result = Vec::new();
      if let Ok(entries) = std::fs::read_dir(dir) {
          for entry in entries.flatten() {
              let path = entry.path();
              if path.is_dir() {
                  result.extend(collect_ish_files(&path));
              } else if path.extension().map_or(false, |e| e == "ish") {
                  result.push(path);
              }
          }
      }
      result
  }

  fn process_file(file: &Path, src_root: &Path) {
      let content = match std::fs::read_to_string(file) {
          Ok(c) => c,
          Err(e) => { eprintln!("error reading {}: {}", file.display(), e); return; }
      };
      let program = match ish_parser::parse(&content) {
          Ok(p) => p,
          Err(e) => { eprintln!("error parsing {}: {}", file.display(), e); return; }
      };

      // Collect pub FunctionDecl and TypeAlias statements.
      let pub_decls: Vec<&Statement> = program.statements.iter().filter(|s| {
          match s {
              Statement::FunctionDecl { visibility, .. } => matches!(visibility, Some(ish_ast::Visibility::Pub)),
              Statement::TypeAlias { visibility, .. } => matches!(visibility, Some(ish_ast::Visibility::Pub)),
              _ => false,
          }
      }).collect();

      // Format as .ishi content using Display on each statement.
      let mut content = String::new();
      for decl in &pub_decls {
          // Use ish_ast display formatting.
          // FunctionDecl: write signature only (no body).
          // TypeAlias: write as-is.
          content.push_str(&format!("{}\n", format_decl(decl)));
      }

      // Write to sibling .ishi file.
      let ishi_path = file.with_extension("ishi");
      match std::fs::write(&ishi_path, &content) {
          Ok(()) => {
              // Compute module path for display.
              let rel = file.strip_prefix(src_root).unwrap_or(file);
              println!("Wrote {}", rel.with_extension("ishi").display());
          }
          Err(e) => eprintln!("error writing {}: {}", ishi_path.display(), e),
      }
  }

  fn format_decl(stmt: &Statement) -> String {
      // For FunctionDecl: emit "pub fn name(params) -> RetType" (no body).
      // For TypeAlias: emit the full type alias (it has no body).
      // Use the ish_ast display formatter (Display impl on Statement) but strip
      // the body from FunctionDecl.
      // Simplest approach: match on the statement kind and build the string manually
      // from the AST fields.
      match stmt {
          Statement::FunctionDecl { name, params, return_type, visibility, type_params, .. } => {
              let vis = "pub ";
              let tparams = if type_params.is_empty() { String::new() }
                  else { format!("<{}>", type_params.join(", ")) };
              let params_str = params.iter().map(|p| {
                  if let Some(ref ann) = p.type_annotation {
                      format!("{}: {}", p.name, ann)
                  } else {
                      p.name.clone()
                  }
              }).collect::<Vec<_>>().join(", ");
              let ret = if let Some(ref rt) = return_type {
                  format!(" -> {}", rt)
              } else { String::new() };
              format!("{}fn {}{}({}){}", vis, name, tparams, params_str, ret)
          }
          Statement::TypeAlias { name, definition, visibility, .. } => {
              format!("pub type {} = {}", name, definition)
          }
          _ => String::new(),
      }
  }
  ```

- [x] 2. Edit `proto/ish-shell/src/main.rs` — add `mod interface_cmd;` at the top.

- [x] 3. Edit `proto/ish-shell/src/main.rs` — add the `interface freeze` dispatch before the existing `-c` check:

  ```rust
  if positional.first() == Some(&"interface") {
      if positional.get(1) == Some(&"freeze") {
          let target = positional.get(2).map(|s| s.to_string());
          let cwd = std::env::current_dir().expect("cannot determine cwd");
          interface_cmd::freeze(target, &cwd);
          return;
      }
      eprintln!("unknown interface subcommand: {:?}", positional.get(1));
      std::process::exit(1);
  }
  ```

- [x] 4. Edit `proto/ish-shell/src/repl.rs` (or `main.rs`) — add project root discovery before VM creation:

  In `run_file(filename)` and `run_inline(code)` in `repl.rs`, before creating the VM:

  ```rust
  let project_context = {
      let start = if filename_mode {
          std::path::Path::new(filename).parent().unwrap_or(std::path::Path::new("."))
      } else {
          std::path::Path::new(".")
      };
      let project_root = ish_vm::module_loader::find_project_root(start);
      let src_root = project_root.as_ref().map(|r| r.join("src"));
      ish_vm::access_control::ProjectContext { project_root, src_root }
  };
  // Set on VM after creation:
  vm.borrow_mut().project_context = project_context;
  ```

  For `run_interactive`, use `std::env::current_dir()` as the start directory.

- [x] 5. Add `ish-parser` to `proto/ish-shell/Cargo.toml` under `[dependencies]` if not already present (already present):

  ```toml
  ish-parser = { path = "../ish-parser" }
  ```

## Verification

Run: `cd proto && cargo build -p ish-shell`
Check: No errors.

Run: Create a temp project with `src/`, a `project.json`, and a `src/utils.ish` file containing `pub fn greet(name: String) -> String { "hello" }`. Then run `ish interface freeze`. Check that `src/utils.ishi` is created containing `pub fn greet(name: String) -> String`.

Run: `cd proto && cargo run -p ish-shell -- interface freeze 2>&1 | head -5` (from a directory without a src/ dir)
Check: Error message about missing src/ directory.

Invoke: `/verify module-system-core-a2/phase-6.md`
