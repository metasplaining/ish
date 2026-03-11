# ish Prototype

A working prototype of the ish language processor proving three mechanisms:

1. **Interpreted execution** — declare functions as AST, execute via tree-walking interpreter
2. **Compiled execution** — same AST → Rust source → native `.so` → dynamically loaded and called
3. **Self-hosting** — code analyzer, Rust code generator, and standard library all written as ish programs running on the interpreter

## Quick Start

```bash
cd proto/

# Run the full end-to-end demonstration (6 verifications)
cargo run -p ish-shell

# Run the complete test suite (45 tests)
cargo test --workspace
```

### Expected output from `ish-shell`

```
=== ish prototype demonstration ===

1. Interpreted factorial(10)... PASS (3628800)
2. Compiled factorial(10)... PASS (3628800)
3. Analyzer detects undeclared variable... PASS (1 warning(s) found)
4. Generator produces compilable Rust... PASS (add(10,32) = 42)
5. Stdlib: abs(-42) and sum(range(5))... PASS (42 + 10 = 52)
6. Consistency: interpreted == compiled for factorial(5,8,12)... PASS

=== Results: 6/6 passed ===
All demonstrations successful! The ish prototype is complete.
```

## Crate Overview

| Crate | Role | Lines |
|-------|------|-------|
| **ish-ast** | AST node types, fluent builder API, display formatting | ~960 |
| **ish-vm** | Tree-walking interpreter, GC-managed values, builtins, AST↔Value reflection | ~2840 |
| **ish-stdlib** | Self-hosted analyzer, Rust generator, and stdlib — all written as ish programs | ~1120 |
| **ish-runtime** | Minimal value type for compiled functions (FFI boundary) | ~50 |
| **ish-codegen** | Compilation driver: temp Cargo project → `cargo build` → load `.so` via libloading | ~300 |
| **ish-shell** | CLI binary running all 6 verification demos | ~320 |

Total: ~5,600 lines of Rust across 16 source files.

## Design Decisions

These were established before implementation. See [`plan-ishPrototype.prompt.md`](../.github/prompts/plan-ishPrototype.prompt.md) for full rationale.

| Decision | Choice |
|----------|--------|
| Project location | `/home/dan/git/ish/proto/` inside the ish repo |
| Compiled function loading | Dynamic linking via `libloading` |
| FFI value passing | Shared `IshValue` enum in `ish-runtime` |
| Numeric types | `i64` + `f64` only (expandable later) |
| AST-as-values representation | Plain Objects with a `kind` string discriminator |
| Error handling | Rust `Result` types — errors bubble up through Rust call stack |
| Namespace model | Flat global namespace |
| Closures | Supported from the start via environment capture |
| Conditional style | Chained if-else (no pattern matching yet) |

## How Programs Are Written

There is **no parser** in the prototype. Programs are constructed in three ways:

### 1. Convenience constructors (direct AST)

```rust
let prog = Program::new(vec![
    Statement::var_decl("x", Expression::int(42)),
    Statement::expr_stmt(Expression::call(
        Expression::ident("println"),
        vec![Expression::call(
            Expression::ident("to_string"),
            vec![Expression::ident("x")],
        )],
    )),
]);
```

### 2. Fluent builder API

```rust
let prog = ProgramBuilder::new()
    .function("factorial", &["n"], |b| {
        b.if_else(
            Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
            |b| b.ret(Expression::int(1)),
            |b| b.ret(Expression::binary(
                BinaryOperator::Mul,
                Expression::ident("n"),
                Expression::call(
                    Expression::ident("factorial"),
                    vec![Expression::binary(
                        BinaryOperator::Sub,
                        Expression::ident("n"),
                        Expression::int(1),
                    )],
                ),
            )),
        )
    })
    .build();
```

### 3. AST factory functions (from within ish)

Ish programs can build AST nodes with factory builtins like `ast_literal`, `ast_binary_op`, `ast_function_call`, etc. This is how self-hosted components construct code.

## The Compilation Pipeline

```
 ish AST (Program)
     │
     ▼ program_to_value()
 AST-as-Values (ish Objects with "kind" fields)
     │
     ▼ analyze()          ← ish program running on ish-vm
 Analysis result { warnings, declared_count, reference_count }
     │
     ▼ generate_rust()    ← ish program running on ish-vm
 Rust source code string
     │
     ▼ CompilationDriver
 Temp Cargo project written to /tmp/ish-compiled-XXXX/
     │
     ▼ cargo build --release
 Shared library (.so / .dylib)
     │
     ▼ libloading::Library::new()
 Loaded function pointer (extern "C" fn)
     │
     ▼ callable from Rust
 Result matches interpreted execution
```

## Built-in Functions

45 built-in functions registered in the VM at startup:

| Category | Functions |
|----------|-----------|
| **I/O** | `print`, `println`, `read_file`, `write_file` |
| **Strings** | `str_concat`, `str_length`, `str_slice`, `str_contains`, `str_starts_with`, `str_replace`, `str_split`, `str_to_upper`, `str_to_lower`, `str_char_at`, `str_trim` |
| **Lists** | `list_push`, `list_pop`, `list_length`, `list_get`, `list_set`, `list_slice`, `list_join` |
| **Objects** | `obj_get`, `obj_set`, `obj_has`, `obj_keys`, `obj_values`, `obj_remove` |
| **Types** | `type_of`, `is_type` |
| **Conversion** | `to_string`, `to_int`, `to_float` |
| **AST factories** | `ast_program`, `ast_literal`, `ast_identifier`, `ast_binary_op`, `ast_unary_op`, `ast_function_call`, `ast_block`, `ast_return`, `ast_var_decl`, `ast_if`, `ast_while`, `ast_function_decl`, `ast_expr_stmt`, `ast_lambda`, `ast_property_access`, `ast_index_access`, `ast_object_literal`, `ast_list_literal`, `ast_param`, `ast_assignment`, `ast_assign_target_var`, `ast_for_each` |

## Stdlib (ish-defined)

These functions are themselves ish programs (ASTs built via the ProgramBuilder) that run on the interpreter:

| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `max(a, b)` | Larger of two values |
| `min(a, b)` | Smaller of two values |
| `range(n)` | List `[0, 1, ..., n-1]` |
| `sum(lst)` | Sum of list elements |
| `map(lst, f)` | Apply function to each element |
| `filter(lst, pred)` | Keep elements satisfying predicate |
| `assert(cond, msg)` | Print error if condition is false |
| `assert_eq(a, b, msg)` | Check equality with error message |

## Test Summary

```
ish-ast       8 tests   AST construction, JSON round-trip, builder, display
ish-vm       19 tests   Interpreter, builtins, reflection round-trip
ish-stdlib   13 tests   Analyzer, generator, stdlib functions
ish-codegen   4 tests   Compilation, template generation
ish-runtime   1 test    Value conversions
─────────────────────
Total        45 tests
```

## What This Proves

- **Interpreted and compiled execution produce identical results** for the same AST
- **Self-hosted tools work**: the analyzer and generator are ish programs that run on the interpreter and operate on AST-as-values
- **The compilation pipeline is real**: generated Rust is compiled by `cargo`, loaded as a `.so` at runtime, and called via FFI — producing correct results
- **The AST-first approach works**: without any parser, the full language processing pipeline operates via builder APIs and AST manipulation
- **The value system supports self-hosting**: Objects, Lists, and Functions are sufficient to represent and manipulate AST nodes as data

## What Comes Next

The prototype defers or stubs several components that a full implementation would need:

- **Parser** — translate ish syntax into AST (the prototype uses builder APIs instead)
- **Type system** — the AST includes `TypeAnnotation` but the VM ignores it
- **Richer compiled function signatures** — current FFI is `i64 → i64`; needs IshValue marshaling
- **Error handling in ish** — currently Rust `Result` only; no try/catch in the language
- **Module system** — everything lives in a flat global namespace
- **ish-runtime expansion** — String, Object, List types for compiled code
- **Reasoning system** — formal proposition-based analysis (see `REASONING.md`)
