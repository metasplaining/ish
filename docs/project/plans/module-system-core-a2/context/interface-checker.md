*Extracted verbatim from [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §`ish-vm` — New module: interface_checker.rs.*

---

**New module: `proto/ish-vm/src/interface_checker.rs`**

This module handles interface file (`.ishi`) consistency checking:

```
check_interface(
    module_file: &Path,
    pub_declarations: &[Declaration],
) -> Vec<InterfaceError>
    Locate the sibling .ishi file at the same path with .ishi extension.
    If no .ishi file exists: return empty (no enforcement).
    Parse the .ishi file for its pub declarations (function signatures and type definitions).
    Compare against pub_declarations.
    Emit:
      InterfaceError::SymbolNotInImplementation for each symbol in .ishi absent from the .ish file
      InterfaceError::SymbolNotInInterface for each pub symbol in .ish absent from the .ishi file
      InterfaceError::SymbolMismatch for each symbol present in both with different signatures
```

The `.ishi` file format is a subset of ish source: function signatures (`fn name(params) -> RetType`) and type aliases (`type Name = Definition`), each with `pub` keyword, one per line. No function bodies. The `ish-parser` crate is used to parse `.ishi` files (they are valid ish source).

**Error codes for interface errors:**

| Code | ErrorCode Variant | Summary |
|------|------------------|---------|
| E022 | `InterfaceSymbolNotInImplementation` | `.ishi` declares a symbol absent from the `.ish` file |
| E023 | `InterfaceSymbolNotInInterface` | `.ish` has a `pub` symbol not declared in `.ishi` |
| E024 | `InterfaceSymbolMismatch` | Symbol present in both `.ishi` and `.ish` with mismatched signatures |
