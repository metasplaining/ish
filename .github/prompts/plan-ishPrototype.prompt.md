# Plan: ish Language Processor Prototype

## TL;DR

Build a fresh prototype proving three mechanisms: (1) interpreted function declaration and invocation, (2) compiled function declaration (AST → Rust → binary → callable), and (3) self-hosting: code analyzer, Rust generator, and standard library all written as ish programs running on the interpreter. The approach is **AST-first**: define comprehensive AST node types in Rust, build an interpreter to execute them, then write the self-hosted components as ish programs that manipulate AST-as-values. Syntax is deferred — programs are built via Rust builder APIs and a Rust macro DSL for ergonomics.

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                       ish-shell                          │
│   CLI binary + optional REPL, orchestrates everything    │
└──────┬──────────────────────────┬────────────────────────┘
       │                          │
       ▼                          ▼
┌──────────────┐      ┌────────────────────────┐
│  ish-parser  │      │      ish-linker        │
│  (deferred — │      │  Loads compiled .so     │
│   stub or    │      │  and ish stdlib ASTs    │
│   minimal)   │      └───────────┬────────────┘
└──────┬───────┘                  │
       │                          │
       ▼                          ▼
┌──────────────────────────────────────────────────────────┐
│                        ish-ast                           │
│  AST node types, builder APIs, Rust macro DSL,           │
│  JSON serde, AST ↔ ish Value round-tripping              │
└──────┬──────────────────────────┬────────────────────────┘
       │                          │
       ▼                          ▼
┌──────────────────┐   ┌──────────────────────────────────┐
│     ish-vm       │   │         ish-codegen              │
│  Tree-walking    │   │  Infrastructure for compiling    │
│  interpreter,    │   │  generated Rust: temp project,   │
│  GC, values,     │   │  cargo invocation, .so loading   │
│  call stack,     │   │  via libloading                  │
│  built-in fns    │   └──────────────────────────────────┘
└──────────────────┘
```

**Self-hosted components** (ish programs, running on ish-vm):
- **Code Analyzer**: receives AST-as-ish-values, annotates with metadata, returns enriched AST
- **Rust Generator**: receives analyzed AST-as-ish-values, produces Rust source code strings
- **Standard Library**: common functions defined as ish ASTs, loaded at startup

### Key Design Decisions

1. **AST-first, syntax-deferred**: Programs are constructed via Rust builder APIs, a `ish_ast!` proc-macro DSL, or JSON deserialization. A parser can be added later once syntax is designed.

2. **AST-as-values**: Every AST node can be converted to/from an ish Object value. This is critical for self-hosting — the analyzer and generator are ish programs that manipulate AST objects. Each node is an Object with a `kind: String` discriminator and node-specific properties.

3. **Single function declaration, dual execution**: Functions are declared the same way in the AST. They can be interpreted (run by the VM) or compiled (translated to Rust, compiled, dynamically loaded). The prototype demonstrates both paths from the same declaration.

4. **Built-in functions**: Rust-implemented functions registered in the VM under a known namespace. These provide I/O, string ops, list ops, object introspection, process execution, and AST construction — the primitives that ish programs need.

5. **Garbage collection**: Reuse the `gc` crate approach from the existing prototype for interpreter memory management.

6. **No type/serialization dead end**: Skip the TypeRepository/deserialization pattern from ish-workspace. Types are AST metadata annotations, not a runtime deserialization framework.

---

## Phases

### Phase 1: AST Foundation
*No dependencies. Start here.*

Define comprehensive AST node types as Rust enums/structs with builder pattern.

**AST nodes needed:**

Expressions:
- `Literal` — bool, integer (i64), float (f64), string, null
- `Identifier` — variable/function name reference
- `BinaryOp` — arithmetic (+, -, *, /, %), comparison (==, !=, <, >, <=, >=), logical (and, or)
- `UnaryOp` — not, negate
- `FunctionCall` — callee expression + argument list
- `ObjectLiteral` — list of key-value pairs
- `ListLiteral` — list of expressions
- `PropertyAccess` — object.property
- `IndexAccess` — list[index]
- `Lambda` — anonymous function (params, body)
- `MatchExpr` — pattern matching expression (for AST traversal in analyzer/generator)

Statements:
- `VariableDecl` — let name [: type] = expr
- `Assignment` — name = expr (or property/index assignment)
- `Block` — sequence of statements, optional result expression
- `If` — condition, then-block, optional else-block
- `While` — condition, body-block
- `ForEach` — variable, iterable expression, body-block
- `Return` — optional expression
- `ExpressionStmt` — expression evaluated for side effects
- `FunctionDecl` — name, parameters (with optional type annotations), return type, body block

Top-level:
- `Program` — list of top-level declarations/statements

Metadata (optional annotations):
- `TypeAnnotation` — type info on a node
- `AnalyzerNote` — metadata added by the code analyzer

**Files to create:**
- `ish-ast/src/lib.rs` — core types, Expression/Statement/Program enums
- `ish-ast/src/builder.rs` — fluent builder API for constructing ASTs
- `ish-ast/src/display.rs` — Debug/Display impls for readable AST dumps
- `ish-ast/Cargo.toml`

**Verification:**
- Unit tests: construct every AST node type via builder, assert structure
- Round-trip: AST → JSON → AST for all node types

---

### Phase 2: Runtime Values & Interpreter Core
*Depends on Phase 1.*

Implement the interpreter's value system and basic statement execution.

**Value types:**
```
Value = Bool | Int(i64) | Float(f64) | String(Rc<String>) | Null 
      | Object(ObjectRef) | List(ListRef) | Function(FunctionRef)
      | BuiltinFunction(BuiltinRef) | CompiledFunction(CompiledRef)
```

- `ObjectRef` = `Gc<GcCell<HashMap<String, Value>>>` — GC-managed mutable object
- `ListRef` = `Gc<GcCell<Vec<Value>>>` — GC-managed mutable list
- `FunctionRef` = `Gc<IshFunction>` where `IshFunction { params, body: Block, closure_env }`
- `BuiltinRef` = `Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>`
- `CompiledRef` = wrapper around dynamically loaded function pointer

**Interpreter (IshVm):**
- `Environment` — chain of scopes (HashMap<String, Value>), parent pointer for lexical scoping
- `eval_expression(expr, env) → Result<Value, RuntimeError>`
- `exec_statement(stmt, env) → Result<Option<Value>, RuntimeError>` (Option for return values)
- `exec_block(block, env) → Result<Option<Value>, RuntimeError>`

**Implement these statement types first:**
- VariableDecl, Assignment, Block, If, While, ExpressionStmt, Return
- FunctionDecl (creates FunctionRef, binds in environment)

**Implement these expression types first:**
- Literal, Identifier, BinaryOp, UnaryOp, FunctionCall, ObjectLiteral, ListLiteral, PropertyAccess, IndexAccess

**Files to create:**
- `ish-vm/src/lib.rs` — IshVm entry point
- `ish-vm/src/value.rs` — Value enum, ObjectRef, ListRef, FunctionRef
- `ish-vm/src/environment.rs` — Environment (scoped variable storage)
- `ish-vm/src/interpreter.rs` — eval_expression, exec_statement
- `ish-vm/src/error.rs` — RuntimeError with source location
- `ish-vm/Cargo.toml`

**Verification:**
- Test: declare function, call it, verify return value
- Test: closures capture enclosing scope
- Test: recursive function (factorial)
- Test: objects and lists in functions
- Test: nested blocks with scoping

---

### Phase 3: Built-in Functions
*Depends on Phase 2.*

Implement Rust-native functions that ish programs can call. These are the primitives that self-hosted components need.

**Categories:**

*String operations:*
- `str_concat(a, b)`, `str_length(s)`, `str_slice(s, start, end)`, `str_contains(s, substr)`
- `str_starts_with(s, prefix)`, `str_replace(s, from, to)`, `str_split(s, delimiter)`
- `str_to_upper(s)`, `str_to_lower(s)`, `str_char_at(s, i)`, `str_trim(s)`

*List operations:*
- `list_push(list, value)`, `list_pop(list)`, `list_length(list)`, `list_get(list, i)`
- `list_set(list, i, value)`, `list_slice(list, start, end)`
- `list_map(list, fn)`, `list_filter(list, fn)`, `list_fold(list, init, fn)`
- `list_join(list, separator)` (list of strings → string)

*Object operations:*
- `obj_get(obj, key)`, `obj_set(obj, key, value)`, `obj_has(obj, key)`
- `obj_keys(obj)`, `obj_values(obj)`, `obj_remove(obj, key)`

*I/O:*
- `print(value)`, `println(value)`, `read_file(path)`, `write_file(path, content)`

*Type checking:*
- `type_of(value)` → string ("bool", "int", "float", "string", "object", "list", "function", "null")
- `is_type(value, type_name)` → bool

*Conversion:*
- `to_string(value)`, `to_int(value)`, `to_float(value)`

*Process execution (for compilation):*
- `exec(command, args)` → { stdout, stderr, exit_code }

**Registration mechanism:**
- `BuiltinRegistry` struct with `register(name, fn)` method
- Interpreter preloads all builtins into the root environment

**Files to create/modify:**
- `ish-vm/src/builtins.rs` — all built-in function implementations
- `ish-vm/src/builtins/strings.rs`, `lists.rs`, `objects.rs`, `io.rs`, `types.rs` (or a single file if small)

**Verification:**
- Test each built-in category independently
- Integration test: ish program that uses string manipulation to build output

---

### Phase 4: AST-as-Values (Reflection Layer)
*Depends on Phase 2. Parallel with Phase 3.*

Enable AST nodes to be represented as ish Object values, and vice versa. This is the critical bridge for self-hosting.

**Design:**
Each AST node → ish Object with:
- `kind: String` — discriminator (e.g., "if", "function_decl", "binary_op", "literal")
- Node-specific properties as Object fields
- Child nodes are nested Objects (recursive structure)

Example:
```
// AST: if (x > 0) { return x; } else { return -x; }
// As ish Object:
{
  kind: "if",
  condition: {
    kind: "binary_op",
    op: ">",
    left: { kind: "identifier", name: "x" },
    right: { kind: "literal", value: 0, literal_type: "int" }
  },
  then_branch: {
    kind: "block",
    statements: [
      { kind: "return", value: { kind: "identifier", name: "x" } }
    ]
  },
  else_branch: {
    kind: "block",
    statements: [
      { kind: "return", value: { kind: "unary_op", op: "negate", operand: { kind: "identifier", name: "x" } } }
    ]
  }
}
```

**Implementation:**
- `ast_to_value(node: &AstNode) -> Value` — Rust AST → ish Object (recursive)
- `value_to_ast(value: &Value) -> Result<AstNode, ConversionError>` — ish Object → Rust AST

**Built-in AST factory functions** (callable from ish):
- `ast_literal(value)`, `ast_identifier(name)`, `ast_binary_op(op, left, right)`
- `ast_if(condition, then_branch, else_branch)`, `ast_while(condition, body)`
- `ast_function_decl(name, params, body)`, `ast_function_call(callee, args)`
- `ast_block(statements)`, `ast_return(value)`, `ast_var_decl(name, value)`
- `ast_object_literal(pairs)`, `ast_list_literal(elements)`
- `ast_property_access(object, property)`, `ast_index_access(object, index)`
- `ast_lambda(params, body)`, `ast_match(value, arms)`
- `ast_program(declarations)`

These factory functions create ish Object values with the correct `kind` and properties.

**Files to create:**
- `ish-ast/src/reflection.rs` — ast_to_value, value_to_ast conversions
- `ish-vm/src/builtins/ast_factory.rs` — built-in AST construction functions

**Verification:**
- Round-trip: Rust AST → ish Value → Rust AST, assert equality
- Test: ish program constructs an AST using factory functions, convert back, execute it

---

### Phase 5: Code Analyzer in ish
*Depends on Phases 3 and 4.*

Write a basic code analyzer as an ish program. It receives an AST (as ish Object values), traverses it, and returns an annotated version. For the prototype, the analyzer performs simple analyses to prove the mechanism works.

**Prototype analyses:**
1. **Variable usage check**: walk the AST, track declared vs. referenced variables, flag undeclared references
2. **Return path check**: verify that all branches of a function end with a return
3. **Constant folding annotation**: identify `BinaryOp` nodes where both operands are literals, annotate with computed value

**Structure of the analyzer (ish program):**
```
function analyze(ast_node) {
  let kind = obj_get(ast_node, "kind");
  if kind == "function_decl" {
    let body = obj_get(ast_node, "body");
    let params = obj_get(ast_node, "params");
    let declared = collect_declarations(body, params);
    let referenced = collect_references(body);
    check_undeclared(declared, referenced);
    // ... annotate and return modified AST
  }
  // ... handle other node kinds
}
```

The analyzer is defined as an ish AST (built using Rust builder APIs or the macro DSL) and executed by the interpreter.

**Files to create:**
- `ish-stdlib/src/analyzer.rs` — Rust code that builds the analyzer's AST using the builder API
- Or: `ish-stdlib/analyzer.json` — the analyzer program as serialized AST JSON

**Verification:**
- Test: analyzer detects undeclared variable, returns annotation
- Test: analyzer detects missing return path
- Test: analyzer annotates constant expression

---

### Phase 6: Rust Generator in ish
*Depends on Phases 3 and 4. Parallel with Phase 5.*

Write a basic Rust code generator as an ish program. It takes an AST (as ish Object values) and produces Rust source code as a string.

**Scope for prototype:**
Generate valid Rust for functions containing:
- Integer/float/bool/string literals
- Variable declarations (let)
- Arithmetic and comparison operators
- If/else
- While loops
- Function calls (to other generated functions or known Rust functions)
- Return

**Structure of the generator (ish program):**
```
function generate_rust(ast_node) {
  let kind = obj_get(ast_node, "kind");
  if kind == "function_decl" {
    let name = obj_get(ast_node, "name");
    let params = obj_get(ast_node, "params");
    let body = obj_get(ast_node, "body");
    let param_str = generate_params(params);
    let body_str = generate_block(body);
    return str_concat("fn ", name, "(", param_str, ") -> IshValue {\n", body_str, "\n}");
  }
  // ... handle other node kinds
}
```

**Files to create:**
- `ish-stdlib/src/generator.rs` — Rust code that builds the generator's AST

**Verification:**
- Test: generate Rust for a simple function (add two numbers), verify output compiles
- Test: generate Rust for a function with if/else, verify correctness
- Test: generate + compile + run, compare output with interpreted execution

---

### Phase 7: Compiled Function Mechanism
*Depends on Phase 6.*

Infrastructure to compile generated Rust source into a dynamically loadable library and call it from the interpreter.

**Pipeline:**
1. Rust generator (ish program) produces Rust source string
2. `ish-codegen` writes source to a temp Cargo project
3. `ish-codegen` invokes `cargo build --release` on the temp project
4. `ish-codegen` loads the resulting `.so`/`.dylib` via `libloading`
5. Function pointer wrapped as `CompiledRef` value in the VM
6. Compiled function is callable just like interpreted functions

**Temp project structure:**
```
/tmp/ish-compiled-XXXX/
  Cargo.toml          (depends on ish-runtime for IshValue type)
  src/lib.rs          (generated Rust code with #[no_mangle] pub extern "C" fn ...)
```

**ish-runtime crate** (minimal dependency for compiled code):
- Defines `IshValue` enum matching the interpreter's `Value`
- Provides conversion functions
- No GC dependency — compiled functions work with owned values

**Files to create:**
- `ish-codegen/src/lib.rs` — CompilationDriver: write temp project, invoke cargo, load .so
- `ish-codegen/src/template.rs` — Cargo.toml template, boilerplate for generated lib.rs
- `ish-runtime/src/lib.rs` — minimal IshValue type for compiled functions
- `ish-runtime/Cargo.toml`
- `ish-codegen/Cargo.toml`

**Verification:**
- End-to-end test: define function AST → analyze (ish) → generate Rust (ish) → compile → load → call compiled function → verify output matches interpreted execution
- Test: compiled function receives arguments and returns values correctly

---

### Phase 8: Standard Library in ish
*Depends on Phases 3, 4. Parallel with Phases 5-7.*

Implement standard library functions as ish programs (ASTs built in Rust, or loaded from JSON).

**Prototype stdlib functions:**
- `abs(x)` — absolute value
- `max(a, b)`, `min(a, b)` — comparison
- `range(start, end)` — returns list of integers
- `map(list, fn)`, `filter(list, fn)`, `reduce(list, init, fn)` — higher-order (if not built-in)
- `assert(condition, message)` — testing helper
- `assert_eq(a, b, message)` — equality assertion

**Files to create:**
- `ish-stdlib/src/lib.rs` — loads all stdlib functions into the VM
- `ish-stdlib/src/math.rs` — math stdlib functions as ASTs
- `ish-stdlib/src/collections.rs` — collection helpers as ASTs
- `ish-stdlib/src/testing.rs` — assert functions as ASTs
- `ish-stdlib/Cargo.toml`

**Verification:**
- Test: call stdlib `abs(-5)` from an ish program, verify result is 5
- Test: `map(range(1, 5), fn(x) { x * x })` returns `[1, 4, 9, 16]`
- Test: stdlib function works both interpreted and compiled

---

### Phase 9 (Optional): Macro DSL for Ergonomic AST Construction
*Parallel with any phase. Quality-of-life improvement.*

Create a Rust procedural macro that allows writing ish-like syntax in Rust code, which expands to AST builder calls. This dramatically reduces the verbosity of writing test programs and self-hosted components.

```rust
let program = ish_ast! {
    fn factorial(n) {
        if n <= 1 {
            return 1;
        }
        return n * factorial(n - 1);
    }
};
```

Expands to builder API calls generating the AST.

**Files to create:**
- `ish-macros/src/lib.rs` — proc-macro implementation
- `ish-macros/Cargo.toml`

**Verification:**
- Test: macro-generated AST matches hand-built AST

---

## Crate Dependency Graph

```
ish-macros (proc-macro, optional)
     │
     ▼
  ish-ast ◄──────────────────────────────┐
     │                                    │
     ├──────────┐                         │
     ▼          ▼                         │
  ish-vm    ish-codegen ──► ish-runtime   │
     │          │                         │
     ▼          │                         │
  ish-stdlib ◄──┘                         │
     │                                    │
     ▼                                    │
  ish-shell ──────────────────────────────┘
```

New workspace root: **To be decided** (see Question 1).

---

## Relevant Files (from existing prototype — reference only)

- `ish-workspace/ish-parser/src/lib.rs` — reference for Span-based tracking, Expression/Statement enums, parser pipeline (parse → AST). Reuse the span concept.
- `ish-workspace/ish-parser/src/ish.pest` — reference for pest grammar structure. Not reusing directly since syntax is deferred.
- `ish-workspace/ish-vm/src/lib.rs` — reference for `IshVm.process_chunk()`, `resolve_expression()`, `process_statement()` flow. Reuse the tree-walking approach.
- `ish-workspace/ish-vm/src/value.rs` — reference for `Value` enum with GC types. Reuse the `Gc<GcCell<...>>` pattern for Object/List; skip the serialization/type-hint machinery.
- `ish-workspace/ish-vm/src/object.rs` — reference for Object trait + `IVMObject` HashMap impl. Simplify: use `HashMap<String, Value>` directly instead of trait objects.
- `ish-workspace/ish-vm/src/list.rs` — reference for List trait. Simplify similarly.
- `ish/README.md` — architecture overview, module responsibilities, optimization philosophy
- `ish/TYPES.md` — type system design (reference for future phases, not prototype)
- `ish/REASONING.md` — reasoning system design (reference for analyzer design)

---

## Verification (End-to-End)

The prototype is **complete** when these demonstrations work:

1. **Interpreted function**: Define `factorial(n)` as AST → execute on VM → returns correct result for `factorial(10)`
2. **Compiled function**: Same `factorial(n)` AST → run through ish-written analyzer → run through ish-written Rust generator → compile to .so → load → call → returns same result as interpreted
3. **Self-hosted analyzer**: Code analyzer written as ish AST program detects undeclared variable in a test AST
4. **Self-hosted generator**: Rust generator written as ish AST program produces compilable Rust for a test function
5. **Standard library in ish**: Call `abs(-42)` and `map(range(1,5), double)` where these are ish-defined functions
6. **Consistency**: For a set of test functions, interpreted output == compiled output

---

## Questions to Answer

### Critical (blocking — needed before implementation starts)

1. **Project location**: Where should the new prototype live? Options:
   - (A) New directory alongside existing: `/home/dan/ish-proto/`
   - (B) New directory inside ish repo: `/home/dan/git/ish/proto/`
   - (C) Replace ish-workspace (user said no)
   - **Recommendation**: (A) — clean separation from both docs and old prototype
   - **Answer**: (B) — new `proto/` directory inside ish repo for easy code sharing and future integration

2. **Compiled function loading**: How should compiled ish functions be invoked at runtime?
   - (A) **Dynamic linking** via `libloading` — compile to `.so`, load at runtime, call via FFI. Most impressive demo, but adds complexity (ABI, platform differences).
   - (B) **Separate process** — generate a complete Rust program, compile and run it, capture output. Simpler but doesn't show interop.
   - (C) **Build-time integration** — generate Rust code that's compiled into the main binary in a two-pass build. Demonstrates compilation but not runtime dynamism.
   - **Recommendation**: (A) for the full demo, with (B) as a simpler first milestone
   - **Answer**: (A) — dynamic linking is the most compelling demonstration of the compiled function mechanism, and it's feasible with careful design of the FFI boundary.

3. **Value representation across the FFI boundary**: When a compiled function is called from the interpreter (or vice versa), how do values cross the boundary?
   - (A) **Shared `IshValue` enum** — both interpreter and compiled code use the same Rust type (requires `ish-runtime` crate as shared dependency)
   - (B) **JSON serialization** — marshal values as JSON across the boundary (simple but slow)
   - (C) **C-compatible repr** — define a C ABI for values (most portable but verbose)
   - **Recommendation**: (A) — simplest for a prototype, ish-runtime is a thin shared type
  - **Answer**: (A) — using a shared `IshValue` enum in the `ish-runtime` crate allows for efficient and type-safe value passing between the interpreter and compiled functions, without the overhead of serialization.

4. **Numeric representation in the prototype**: The full language has i8-i128, u8-u128, f32, f64. For the prototype:
   - (A) **Full numeric tower** — all types from day one
   - (B) **Simplified** — i64 + f64 only, expand later
   - **Recommendation**: (B) — matches the "streamlined defaults to f64" philosophy and reduces boilerplate
   - **Answer**: (B) — starting with just `i64` and `f64` simplifies the implementation while still demonstrating the core mechanisms. Additional numeric types can be added in future iterations.
### Important (needed before Phase 4-5)

5. **AST node representation as ish values**: How should AST nodes appear to ish programs?
   - (A) **Plain Objects** — `{ kind: "if", condition: {...}, ... }`. Simple, flexible, no special types needed.
   - (B) **Tagged Objects** — Objects with a hidden `__ast_type` field and validation on construction. Safer but more complex.
   - (C) **Dedicated AST value type** — Add `Value::AstNode(AstNode)` variant. Type-safe but breaks the "implement in ish" goal.
   - **Recommendation**: (A) — simplest, matches the "objects are the universal data structure" philosophy
   - **Answer**: (A) — representing AST nodes as plain Objects with a `kind` field is the most straightforward approach, allowing ish programs to manipulate ASTs without needing special types or validation logic.

6. **Error handling model for the prototype**: How do runtime errors propagate?
   - (A) **Rust Result types only** — errors bubble up through the Rust call stack, printed at top level
   - (B) **ish-level try/catch** — implement exception mechanism in the interpreter
   - (C) **Result values** — errors are special values that ish code checks explicitly
   - **Recommendation**: (A) for now — sufficient for prototype, defer ish-level error handling
   - **Answer**: (A) — using Rust's `Result` types for error handling keeps the implementation simpler for the prototype. ish-level error handling can be added in future iterations once the core mechanisms are in place.

7. **Module/import system for the prototype**: The analyzer and generator are separate "programs." How are they loaded?
   - (A) **No modules** — everything in a flat global namespace, stdlib pre-loaded
   - (B) **Simple file-based loading** — `load("analyzer.ish")` reads and evaluates a file
   - (C) **AST-based modules** — modules are ASTs loaded programmatically
   - **Recommendation**: (A) or (C) — since programs are ASTs anyway, they're loaded programmatically. No file-based module system needed for prototype.
   - **Answer**: (A) — for the prototype, a simple flat global namespace with pre-loaded stdlib functions is sufficient. The analyzer and generator can be defined as ASTs in Rust and loaded directly into the VM without needing a file-based module system.

### Nice to Have (can be deferred)

8. **Closures in interpreter**: Should interpreted functions capture their lexical environment?
   - **Recommendation**: Yes — closures are essential for higher-order functions like `map`/`filter`. Implement from the start via environment chains.
    - **Answer**: Yes — implementing closures from the start allows for more expressive ish programs and is essential for demonstrating the full capabilities of the interpreter, especially for the self-hosted components.

9. **Pattern matching vs. chained if-else**: The analyzer and generator need to dispatch on AST node kinds. Options:
   - (A) `match` expression in the AST — cleaner but requires implementing pattern matching
   - (B) Chained `if (kind == "x") { ... } else if (kind == "y") { ... }` — works with existing features
   - **Recommendation**: Start with (B), add match expression if time permits. The analyzer can work fine with chained conditionals.
  - **Answer**: (B) — starting with chained `if-else` statements is simpler and sufficient for the prototype. A more elegant pattern matching mechanism can be added in future iterations once the core functionality is established.

10. **Macro DSL priority**: Is the `ish_ast!` macro essential or nice-to-have?
    - **Recommendation**: Nice-to-have. Builder API is sufficient. Macro improves DX significantly but can be added anytime.
    - **Answer**: Nice-to-have — the builder API allows for constructing ASTs without the macro, so it can be deferred until after the core mechanisms are in place. The macro will enhance developer experience but is not critical for demonstrating the prototype's goals.

---

## Decisions Already Made (from documentation analysis)

- Interpreter uses garbage collection (`gc` crate) — confirmed by existing prototype
- Compilation target is Rust — confirmed by README
- AST is the central representation — confirmed by architecture docs
- Type system is set-of-values semantic — defer full implementation, but keep in mind for AST design
- Polymorphism/memory strategies are compiler concerns — not needed for prototype
- Reasoning system is aspirational — the code analyzer prototype is a stepping stone

---

## Scope Boundaries

**In scope:**
- AST node definitions for all expression/statement types listed
- Tree-walking interpreter with GC, closures, call stack
- Built-in functions for strings, lists, objects, I/O, type checking
- AST ↔ ish Value reflection layer
- Code analyzer as an ish program (basic analyses)
- Rust generator as an ish program (basic function generation)
- Compiled function mechanism (generate → compile → load → call)
- Standard library functions as ish programs
- End-to-end demonstration of all three proof-of-concept goals

**Out of scope (explicitly deferred):**
- Parser / syntax design
- Full type system implementation
- Type checking / inference
- Reasoning system
- Memory management strategies beyond GC
- Polymorphism strategies beyond associative arrays
- Concurrency / async
- Module/import system beyond programmatic loading
- Package manager
- LSP / editor integration
- Optimization
- Agreement/encumbrance system
- Production error messages
