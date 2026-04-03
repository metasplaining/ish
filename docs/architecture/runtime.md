---
title: "Architecture: ish-runtime"
category: architecture
audience: [all]
status: draft
last-verified: 2026-04-02
depends-on: [docs/architecture/overview.md, docs/spec/types.md, docs/spec/errors.md, docs/spec/concurrency.md]
---

# ish-runtime

**Source:** `proto/ish-runtime/src/`

Standalone runtime types shared between the interpreter (`ish-vm`) and compiled packages. All compiled ish packages depend on this crate without pulling in the full interpreter.

**Dependencies:** `ish-core` (for `TypeAnnotation`), `gc` (GC-managed values), `tokio` (async runtime for `FutureRef`).

---

## Value System (`value.rs`)

```rust
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Rc<String>),
    Char(char),
    Null,
    Object(ObjectRef),              // Gc<GcCell<HashMap<String, Value>>>
    List(ListRef),                  // Gc<GcCell<Vec<Value>>>
    Function(FunctionRef),          // Gc<IshFunction>
    Future(FutureRef),              // Rc<RefCell<Option<JoinHandle>>>
}
```

- **GC-managed:** Objects, Lists, and Functions use the `gc` crate (v0.5)
- **Strings** use `Rc<String>` (cheap cloning, no GC overhead for immutable data)
- **`PartialEq`** supports cross-type Int/Float comparison, Char equality, and Future identity equality (`Rc::ptr_eq`)
- **`is_truthy()`** — false for `false`, `0`, `0.0`, `Null`; true for everything else (including `Char`)
- **No `BuiltinFunction` variant** — all builtins are `Function` values with compiled shims

### Shim

```rust
pub type Shim = Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>;
```

The universal function calling convention. All functions — builtins, interpreted, and compiled — are invoked through a `Shim`. Interpreted functions have shims created by the VM that capture the body statement, closure environment, and VM reference.

### IshFunction

```rust
pub struct IshFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub param_types: Vec<Option<TypeAnnotation>>,
    pub return_type: Option<TypeAnnotation>,
    pub shim: Shim,
    pub is_async: bool,
    pub has_yielding_entry: Option<bool>,
}
```

**Shim types** (behavioral, not structural — all use the same `Shim` type):
- **Unyielding shims** (`len`, `type_of`, etc.) — call logic directly, return a plain `Value`.
- **Yielding shims** — spawn work via `spawn_local`, return `Value::Future`.
- **Parallel shims** (`print`, `read_file`, etc.) — marshal args to `Send`-safe form, `spawn_blocking` + `spawn_local` bridge, return `Value::Future`.

### Constructors

| Function | Description |
|----------|-------------|
| `new_compiled_function(name, params, param_types, return_type, shim, is_async, has_yielding_entry)` | Create an `IshFunction` with a directly-provided shim |
| `new_object(map)` | Wrap a `HashMap<String, Value>` in GC |
| `new_list(vec)` | Wrap a `Vec<Value>` in GC |
| `empty_object()` | Create a `Value::Object` with an empty map |

---

## Error Handling (`error.rs`)

### RuntimeError

```rust
pub struct RuntimeError {
    pub message: String,
    pub thrown_value: Option<Value>,
}
```

- `RuntimeError::new(message)` — create a runtime error with a message
- `RuntimeError::thrown(value)` — create a runtime error from a thrown value, preserving it for the caller's try/catch
- `RuntimeError::system_error(message, code)` — create a system error with an `ErrorCode`

### ErrorCode

```rust
pub enum ErrorCode {
    UnhandledThrow,       // E001
    DivisionByZero,       // E002
    ArgumentCountMismatch,// E003
    TypeMismatch,         // E004
    UndefinedVariable,    // E005
    NotCallable,          // E006
    IndexOutOfBounds,     // E007
    IoError,              // E008
    NullUnwrap,           // E009
    ShellError,           // E010
    AsyncError,           // E011
    AwaitUnyielding,      // E012
    SpawnUnyielding,      // E013
}
```

Type-safe error code enum replacing string literal codes. Implements `Display` (e.g., `"E001"`) and `as_str()`.

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/codegen.md](codegen.md)
- [docs/architecture/vm.md](vm.md)
