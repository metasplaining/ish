*Extracted verbatim from [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §`ish-shell`.*

---

**New subcommand: `ish interface freeze`**

Add an `interface` subcommand to the shell binary (`ish-shell/src/main.rs`). Implement in a new file `ish-shell/src/interface_cmd.rs`:

```
interface_freeze(target: Option<String>, project_root: &Path)
    If target is None: walk src/ and process all .ish files.
    If target is Some(module_name): resolve module_name to a .ish file path.
    For each .ish file:
        Parse the file.
        Collect all FunctionDecl and TypeAlias nodes with Visibility::Pub.
        Format them as a .ishi declaration file (signatures only, no bodies).
        Write to the sibling .ishi path, overwriting any existing file.
        Print: "Wrote <module_path>.ishi"
```

The `.ishi` file format is a subset of ish source: function signatures (`fn name(params) -> RetType`) and type aliases (`type Name = Definition`), each with `pub` keyword, one per line. No function bodies.

**Project root discovery at startup**

In `ish-shell/src/main.rs`, before launching the REPL or executing a file, determine the `ProjectContext`:

1. If executing a file: call `module_loader::find_project_root` from the file's directory.
2. If in REPL mode: call `find_project_root` from the current working directory.
3. Store the `ProjectContext` and pass it to the interpreter.

---

**Current `main.rs` structure** (proto/ish-shell/src/main.rs):

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let no_history = args.iter().any(|a| a == "--no-history");
    let positional: Vec<&str> = args[1..]
        .iter()
        .filter(|a| !a.starts_with("--"))
        .filter(|a| a.as_str() != "-c")
        .map(|s| s.as_str())
        .collect();

    if let Some(idx) = args.iter().position(|a| a == "-c") {
        let code = args.get(idx + 1).expect("missing argument to -c");
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(repl::run_inline(code));
    } else if let Some(filename) = positional.first() {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(repl::run_file(filename));
    } else {
        repl::run_interactive(no_history);
    }
}
```

The `interface` subcommand is triggered when `positional[0] == "interface"` and `positional[1] == "freeze"`. No Tokio runtime is needed for `interface freeze` (it is synchronous filesystem work).

**Subcommand dispatch pattern to add:**

```
if positional.first() == Some(&"interface") && positional.get(1) == Some(&"freeze") {
    let target = positional.get(2).map(|s| s.to_string());
    let project_root = std::env::current_dir().expect("cannot determine cwd");
    interface_cmd::freeze(target, &project_root);
} else if ... // existing file / inline / repl dispatch
```
