# Architecture Reference

Detailed reference for the ish prototype's internal structure. For an overview, see [README.md](README.md).

---

## Crate Dependency Graph

```
ish-core (standalone — TypeAnnotation, serde)
  ↑           ↑
ish-ast    ish-runtime ── ish-core, gc, tokio
  ↑           ↑
ish-parser  ish-vm ────── ish-ast, ish-runtime, gc, serde_json, tokio
  ↑           ↑
  └─── ish-codegen ─── ish-ast, ish-vm, ish-runtime, libloading, tempfile
  └─── ish-stdlib ──── ish-ast, ish-vm
  └─── ish-shell (binary)
```

`ish-core` contains shared types (primarily `TypeAnnotation`) used by both `ish-ast` and `ish-runtime`.

`ish-runtime` contains the runtime type system (`Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`) so that compiled packages can link against it without pulling in the full interpreter.

---

## ish-ast

**Purpose:** Define the complete AST as Rust types. All other crates consume this.

### Core Types

```rust
pub struct Program {
    pub statements: Vec<Statement>,
}

pub enum Expression {
    Literal(Literal),               // bool, i64, f64, String, null
    Identifier(String),             // variable reference
    BinaryOp { op, left, right },   // arithmetic, comparison, logical
    UnaryOp { op, operand },        // not, negate
    FunctionCall { callee, args },  // f(a, b, ...)
    ObjectLiteral { pairs },        // { key: val, ... }
    ListLiteral { elements },       // [a, b, ...]
    PropertyAccess { object, property },  // obj.prop
    IndexAccess { object, index },        // list[i]
    Lambda { params, body },        // (x) => { ... }
}

pub enum Statement {
    VariableDecl { name, type_annotation, value },
    Assignment { target: AssignTarget, value },
    Block { statements },
    If { condition, then_block, else_block },
    While { condition, body },
    ForEach { variable, iterable, body },
    Return { value },
    ExpressionStmt(Expression),
    FunctionDecl { name, params, return_type, body },
}

pub enum AssignTarget {
    Variable(String),
    Property { object, property },
    Index { object, index },
}

pub enum Literal { Bool(bool), Int(i64), Float(f64), String(String), Null }
pub enum BinaryOperator { Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, Gt, LtEq, GtEq, And, Or }
pub enum UnaryOperator { Not, Negate }
pub struct Parameter { pub name: String, pub type_annotation: Option<TypeAnnotation> }
```

All types derive `Serialize` + `Deserialize` for JSON round-tripping.

### Convenience Constructors

`Expression` and `Statement` have associated functions that reduce verbosity:

```rust
Expression::int(42)               // Literal(Int(42))
Expression::ident("x")            // Identifier("x".into())
Expression::binary(op, left, right)
Expression::call(callee, args)
Statement::var_decl("x", expr)    // VariableDecl with no type annotation
Statement::ret(expr)              // Return { value: Some(expr) }
```

### Builder API (`builder.rs`)

Fluent builders for constructing programs without deep nesting:

- **`ProgramBuilder`** — top-level: `.function()`, `.var_decl()`, `.stmt()`, `.expr_stmt()`, `.build()`
- **`BlockBuilder`** — block-level: `.var_decl()`, `.assign()`, `.ret()`, `.if_then()`, `.if_else()`, `.while_loop()`, `.for_each()`, `.function()`, `.build()`

The builder closures (`|b| b.ret(...)`) return `&mut BlockBuilder` for chaining.

### Display (`display.rs`)

`Program` implements `fmt::Display`, producing pseudo-code like:

```
fn factorial(n) {
    if (n <= 1) {
        return 1;
    } else {
        return (n * factorial((n - 1)));
    }
}
```

---

## ish-vm

**Purpose:** Execute AST programs via tree-walking interpretation. Value types (`Value`, `Shim`, `RuntimeError`, `ErrorCode`, `IshFunction`) are defined in `ish-runtime` and re-exported.

### Value System

See `ish-runtime` below for the full `Value` enum, `Shim` type alias, `IshFunction` struct, and `RuntimeError`/`ErrorCode` types.

### Environment (`environment.rs`)

Lexical scoping via a chain of GC-managed scopes:

```rust
pub struct Environment {
    inner: Gc<GcCell<Scope>>,
}

struct Scope {
    vars: HashMap<String, Value>,
    parent: Option<Environment>,
}
```

- `Environment::new()` — root scope
- `env.child()` — create child scope (for blocks, function calls)
- `env.define(name, value)` — bind in current scope
- `env.get(name)` — walk chain upward
- `env.set(name, value)` — update existing binding (walks chain)

Closures capture an `Environment` at definition time. When a closure is called, a child of the captured environment becomes the function's local scope.

### Interpreter (`interpreter.rs`)

```rust
pub struct IshVm {
    pub global_env: Environment,
    pub defer_stack: Vec<Vec<(Statement, Environment)>>,
    pub ledger: LedgerState,
}
```

The VM is accessed via `Rc<RefCell<IshVm>>`. Methods are associated functions that take `vm: &Rc<RefCell<IshVm>>` rather than `&mut self`.

**Key methods:**

| Method | Description |
|--------|-------------|
| `IshVm::new()` | Creates VM, registers all builtins + AST factory functions |
| `IshVm::run(vm, &Program)` | Execute a program, return last expression's value |
| `IshVm::eval_expression(vm, &Expression, &Environment)` | Evaluate a single expression |
| `IshVm::call_function(vm, &Value, &[Value])` | Call a function value with arguments |

All functions — builtins and interpreted — are called uniformly via `(func.shim)(args)`. There is no dispatch on implementation variant.

**Internal control flow** uses a `ControlFlow` enum:
- `ControlFlow::None` — continue to next statement
- `ControlFlow::Return(Value)` — propagate return value up the call stack
- `ControlFlow::ExprValue(Value)` — capture an expression statement's value without re-evaluating

**Short-circuit evaluation:** `And` and `Or` operators only evaluate the right operand when needed.

**Division by zero** returns a `RuntimeError` rather than panicking.

### Builtins (`builtins.rs`)

Rust-native functions registered at VM startup. All take `&[Value]` and return `Result<Value, RuntimeError>`.

Registration groups:

| Group | Functions |
|-------|-----------|
| I/O | `print`, `println`, `read_file`, `write_file` |
| Strings | `str_concat`, `str_length`, `str_slice`, `str_contains`, `str_starts_with`, `str_replace`, `str_split`, `str_to_upper`, `str_to_lower`, `str_char_at`, `str_trim` |
| Lists | `list_push`, `list_pop`, `list_length`, `list_get`, `list_set`, `list_slice`, `list_join` |
| Objects | `obj_get`, `obj_set`, `obj_has`, `obj_keys`, `obj_values`, `obj_remove` |
| Types | `type_of`, `is_type` |
| Conversion | `to_string`, `to_int`, `to_float` |

### Reflection (`reflection.rs`)

Bidirectional conversion between Rust AST types and ish Values (Objects with `"kind"` discriminators):

**AST → Value:**
```rust
pub fn program_to_value(program: &Program) -> Value     // { kind: "program", statements: [...] }
pub fn stmt_to_value(stmt: &Statement) -> Value          // { kind: "var_decl"|"if"|..., ... }
pub fn expr_to_value(expr: &Expression) -> Value         // { kind: "literal"|"binary_op"|..., ... }
```

**Value → AST:**
```rust
pub fn value_to_program(value: &Value) -> Result<Program, RuntimeError>
pub fn value_to_stmt(value: &Value) -> Result<Statement, RuntimeError>
pub fn value_to_expr(value: &Value) -> Result<Expression, RuntimeError>
```

**AST factory builtins** — 22 functions callable from ish programs:

`ast_program`, `ast_literal`, `ast_identifier`, `ast_binary_op`, `ast_unary_op`,
`ast_function_call`, `ast_block`, `ast_return`, `ast_var_decl`, `ast_if`, `ast_while`,
`ast_function_decl`, `ast_expr_stmt`, `ast_lambda`, `ast_property_access`,
`ast_index_access`, `ast_object_literal`, `ast_list_literal`, `ast_param`,
`ast_assignment`, `ast_assign_target_var`, `ast_for_each`

These allow ish programs to construct new AST nodes as values.

#### Value representation of AST nodes

Every node is an Object with a `"kind"` field. Examples:

```json
{ "kind": "literal", "literal_type": "int", "value": 42 }
{ "kind": "identifier", "name": "x" }
{ "kind": "binary_op", "op": "add", "left": {...}, "right": {...} }
{ "kind": "var_decl", "name": "x", "value": {...} }
{ "kind": "function_decl", "name": "factorial", "params": [...], "body": {...} }
{ "kind": "if", "condition": {...}, "then_block": {...}, "else_block": null }
{ "kind": "block", "statements": [...] }
{ "kind": "return", "value": {...} }
```

---

## ish-stdlib

**Purpose:** Self-hosted components — all written as ish programs (ASTs built using the Rust builder API).

### Entry Point

```rust
pub fn load_all(vm: &mut IshVm) {
    stdlib::register_stdlib(vm);      // abs, max, min, range, etc.
    analyzer::register_analyzer(vm);  // analyze(), collect_declarations(), etc.
    generator::register_generator(vm); // generate_rust(), generate_expr(), etc.
}
```

Each `register_*` function builds an AST using `ProgramBuilder`, then executes it on the VM via `vm.run()`. This defines the ish functions in the VM's global environment.

### Analyzer (`analyzer.rs`)

Ish functions that inspect AST-as-values and report issues:

| Function | Signature | Description |
|----------|-----------|-------------|
| `collect_declarations` | `(node, declared) → list` | Walk AST, collect names from var_decl, function_decl, for_each, and function params |
| `collect_references` | `(node, refs) → list` | Walk AST, collect all Identifier name references |
| `list_contains` | `(lst, item) → bool` | Linear search helper |
| `check_undeclared` | `(declared, referenced) → list` | Compare declared vs referenced names, return undeclared ones |
| `check_returns` | `(node) → bool` | Check if a block contains a return statement |
| `is_constant_expr` | `(node) → bool` | Check if an expression is a literal constant |
| `analyze` | `(program_node) → object` | Main entry: returns `{ warnings: [...], declared_count, reference_count }` |

### Generator (`generator.rs`)

Ish functions that produce Rust source code from AST-as-values:

| Function | Signature | Description |
|----------|-----------|-------------|
| `rust_op` | `(op) → string` | Map ish operator names to Rust symbols (`"add"` → `"+"`) |
| `generate_expr` | `(node) → string` | Generate Rust for an expression node |
| `generate_stmt` | `(node, indent) → string` | Generate Rust for a statement node (with indentation) |
| `generate_block` | `(node, indent) → string` | Generate Rust for `{ ... }` blocks |
| `generate_rust` | `(node) → string` | Main entry: handles `function_decl` and `program` nodes |

**Supported constructs:** literals (with Rust suffixes like `_i64`), identifiers, binary/unary ops, function calls, var declarations, assignments, returns, if/else, while loops, blocks.

**Generated output example** (for `factorial(n)`):

```rust
fn factorial(n: i64) -> i64 {
    if (n <= 1_i64) {
        return 1_i64;
    } else {
        return (n * factorial((n - 1_i64)));
    }
}
```

### Stdlib (`stdlib.rs`)

Higher-level functions, also defined as ish programs:

| Function | Signature | Description |
|----------|-----------|-------------|
| `abs` | `(x) → int` | Absolute value |
| `max` | `(a, b) → val` | Larger value |
| `min` | `(a, b) → val` | Smaller value |
| `range` | `(n) → list` | `[0, 1, ..., n-1]` |
| `sum` | `(lst) → int` | Sum of list elements |
| `map` | `(lst, f) → list` | Apply f to each element |
| `filter` | `(lst, pred) → list` | Keep elements where pred returns true |
| `assert` | `(cond, msg) → bool` | Print error if false |
| `assert_eq` | `(a, b, msg) → bool` | Check equality |

---

## ish-runtime

**Purpose:** Runtime types shared between the interpreter and compiled packages.

**Dependencies:** `ish-core` (for `TypeAnnotation`), `gc` (GC-managed values), `tokio` (async runtime for `FutureRef`).

### Value System (`value.rs`)

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

pub type Shim = Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>;

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

### Error Handling (`error.rs`)

`RuntimeError` with `message` and optional `thrown_value`. `ErrorCode` enum with 13 variants (E001–E013) for type-safe error construction.

---

## ish-codegen

**Purpose:** Compile generated Rust source into dynamically loadable shared libraries.

### CompilationDriver

```rust
pub struct CompilationDriver {
    runtime_path: PathBuf,  // absolute path to ish-runtime crate
}
```

**Pipeline:**
1. Create a temp directory via `tempfile::tempdir()`
2. Write a `Cargo.toml` (cdylib crate type, depends on `ish-runtime`)
3. Write `src/lib.rs` with the generated Rust source
4. Run `cargo build --release`
5. Find the `.so`/`.dylib` in `target/release/`
6. Load via `libloading::Library::new()`
7. Look up function symbol and return a callable function pointer

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `compile(source)` | `(CompiledLibrary, PathBuf)` | Compile and return the library + .so path |
| `compile_function_1(source, name)` | `(CompiledLibrary, fn(i64) → i64)` | Compile and look up a 1-arg function |
| `compile_function_2(source, name)` | `(CompiledLibrary, fn(i64, i64) → i64)` | Compile and look up a 2-arg function |

The `CompiledLibrary` holds both the `libloading::Library` and the `TempDir` — dropping it unloads the library and cleans up the temp directory.

### Templates (`template.rs`)

- `cargo_toml(runtime_path)` — generates `Cargo.toml` with `crate-type = ["cdylib"]`
- `lib_rs(source)` — wraps source with `#![allow(...)]` pragmas

---

## ish-shell

**Purpose:** CLI binary demonstrating the 6 end-to-end verifications.

Each demo is independent and self-contained. They share a single `IshVm` instance with stdlib loaded. The compilation demos (2, 4, 6) use `CompilationDriver` with a relative path to `ish-runtime`.

---

## Key Implementation Patterns

### Pattern: ref on GC values

Because `Value` implements `Drop` (via the gc crate), pattern matching must use `ref` to borrow rather than move:

```rust
match value {
    Value::Object(ref obj) => { ... }  // borrow the Gc pointer
    Value::String(ref s) => { ... }    // borrow the Rc<String>
    _ => { ... }
}
```

### Pattern: ControlFlow::ExprValue

The interpreter needs to return the last expression statement's value from `run()` without re-evaluating it. `ControlFlow::ExprValue(value)` carries this through `exec_statement` back to `run()`.

### Pattern: self-hosted programs via builder API

Each self-hosted component (analyzer, generator, stdlib) follows the same pattern:

```rust
pub fn register_X(vm: &mut IshVm) {
    let program = build_X();          // Build ish AST via ProgramBuilder
    vm.run(&program).unwrap();        // Execute it → defines functions in global env
}
```

After `register_X`, the ish functions are available by name in the VM and can be called from other ish programs or from Rust via `vm.call_function()`.

### Pattern: AST-as-values for self-hosting

Self-hosted tools receive AST nodes as ish Objects (via `program_to_value()`), walk them by reading the `"kind"` field with `obj_get()`, and recursively process child nodes. This means the analyzer and generator are themselves ish programs that manipulate data — no special AST visitor infrastructure needed.

---

## File Index

```
proto/
├── Cargo.toml                          Workspace: 6 members
├── README.md                           This overview
├── ARCHITECTURE.md                     This detailed reference
│
├── ish-ast/
│   ├── Cargo.toml                      deps: serde, serde_json
│   └── src/
│       ├── lib.rs                      AST types, convenience constructors (8 tests)
│       ├── builder.rs                  ProgramBuilder, BlockBuilder (2 tests)
│       └── display.rs                  fmt::Display for AST (1 test)
│
├── ish-vm/
│   ├── Cargo.toml                      deps: ish-ast, gc, serde_json
│   └── src/
│       ├── lib.rs                      Module declarations
│       ├── value.rs                    Value enum, ObjectRef, ListRef, FunctionRef
│       ├── environment.rs             Lexical scope chain
│       ├── interpreter.rs             IshVm, eval, exec, call (8 tests)
│       ├── builtins.rs                45 built-in functions (6 tests)
│       ├── reflection.rs              AST↔Value conversion, AST factories (4 tests)
│       └── error.rs                   RuntimeError type
│
├── ish-stdlib/
│   ├── Cargo.toml                      deps: ish-ast, ish-vm
│   └── src/
│       ├── lib.rs                      load_all() entry point
│       ├── analyzer.rs                Self-hosted code analyzer (4 tests)
│       ├── generator.rs               Self-hosted Rust generator (3 tests)
│       └── stdlib.rs                  abs, max, min, range, etc. (6 tests)
│
├── ish-runtime/
│   ├── Cargo.toml                      deps: serde, serde_json
│   └── src/
│       └── lib.rs                      IshValue enum (1 test)
│
├── ish-codegen/
│   ├── Cargo.toml                      deps: ish-ast, ish-vm, ish-runtime, libloading, tempfile
│   └── src/
│       ├── lib.rs                      CompilationDriver (2 tests)
│       └── template.rs                Cargo.toml + lib.rs templates (2 tests)
│
└── ish-shell/
    ├── Cargo.toml                      deps: ish-ast, ish-vm, ish-stdlib, ish-codegen
    └── src/
        └── main.rs                     6 end-to-end verification demos
```
