---
title: "Architecture: ish-ast"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md]
---

# ish-ast

**Source:** `proto/ish-ast/src/`

Defines the complete AST as Rust types. All other crates consume this.

---

## Core Types

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

---

## Convenience Constructors

```rust
Expression::int(42)               // Literal(Int(42))
Expression::ident("x")            // Identifier("x".into())
Expression::binary(op, left, right)
Expression::call(callee, args)
Statement::var_decl("x", expr)    // VariableDecl with no type annotation
Statement::ret(expr)              // Return { value: Some(expr) }
```

---

## Builder API (`builder.rs`)

Fluent builders for constructing programs without deep nesting:

- **`ProgramBuilder`** — top-level: `.function()`, `.var_decl()`, `.stmt()`, `.expr_stmt()`, `.build()`
- **`BlockBuilder`** — block-level: `.var_decl()`, `.assign()`, `.ret()`, `.if_then()`, `.if_else()`, `.while_loop()`, `.for_each()`, `.function()`, `.build()`

The builder closures (`|b| b.ret(...)`) return `&mut BlockBuilder` for chaining.

---

## Display (`display.rs`)

`Program` implements `fmt::Display`, producing pseudo-code:

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

## Tests

- `lib.rs`: 8 tests (AST construction and serialization)
- `builder.rs`: 2 tests (builder API)
- `display.rs`: 1 test (display formatting)

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/vm.md](vm.md)
