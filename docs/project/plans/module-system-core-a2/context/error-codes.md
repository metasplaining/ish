*Extracted verbatim from [module-system-core-a2.md](../../../proposals/module-system-core-a2.md) §Error Codes.*

---

All new error codes are production sites in `ish-vm`. Add to `docs/errors/INDEX.md` and add variants to `ErrorCode` in `ish-runtime/src/error.rs`:

| Code | ErrorCode Variant | Structural Type | Summary | Production site |
|------|------------------|-----------------|---------|----------------|
| E016 | `ModuleNotFound` | `CodedError` | `use` path has no matching `.ish` file | `module_loader::resolve_module_path` |
| E017 | `ModuleCycle` | `CodedError` | Circular `use` dependency detected | `interpreter.rs` — Use evaluation |
| E018 | `ModuleScriptNotImportable` | `CodedError` | File imported via `use` contains top-level commands | `interpreter.rs` — Use evaluation |
| E019 | `ModulePathConflict` | `CodedError` | Both `foo.ish` and `foo/index.ish` exist | `module_loader::resolve_module_path` |
| E020 | `ModuleDeclareBlockCommand` | `CodedError` | `declare { }` block contains a non-declaration statement | `interpreter.rs` — DeclareBlock evaluation |
| E021 | `ModuleBootstrapInProject` | `CodedError` | `bootstrap` used inside a project hierarchy | `interpreter.rs` — Bootstrap evaluation |
| E022 | `InterfaceSymbolNotInImplementation` | `CodedError` | `.ishi` declares a symbol absent from the `.ish` file | `interface_checker.rs` |
| E023 | `InterfaceSymbolNotInInterface` | `CodedError` | `.ish` has a `pub` symbol not declared in `.ishi` | `interface_checker.rs` |
| E024 | `InterfaceSymbolMismatch` | `CodedError` | Symbol present in both `.ishi` and `.ish` with mismatched signatures | `interface_checker.rs` |

---

**How to extend `ish-runtime/src/error.rs`:**

The `ErrorCode` enum currently has E001–E015. Add variants after `UnyieldingViolation` (E015):

```rust
ModuleNotFound,             // E016
ModuleCycle,                // E017
ModuleScriptNotImportable,  // E018
ModulePathConflict,         // E019
ModuleDeclareBlockCommand,  // E020
ModuleBootstrapInProject,   // E021
InterfaceSymbolNotInImplementation, // E022
InterfaceSymbolNotInInterface,      // E023
InterfaceSymbolMismatch,            // E024
```

Add `as_str` arms:

```rust
ErrorCode::ModuleNotFound => "E016",
ErrorCode::ModuleCycle => "E017",
ErrorCode::ModuleScriptNotImportable => "E018",
ErrorCode::ModulePathConflict => "E019",
ErrorCode::ModuleDeclareBlockCommand => "E020",
ErrorCode::ModuleBootstrapInProject => "E021",
ErrorCode::InterfaceSymbolNotInImplementation => "E022",
ErrorCode::InterfaceSymbolNotInInterface => "E023",
ErrorCode::InterfaceSymbolMismatch => "E024",
```
