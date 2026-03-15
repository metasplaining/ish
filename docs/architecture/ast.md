---
title: "Architecture: ish-ast"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/architecture/overview.md, docs/spec/syntax.md]
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
    Literal(Literal),               // bool, i64, f64, String, char, null
    Identifier(String),             // variable reference
    BinaryOp { op, left, right },   // arithmetic, comparison, logical
    UnaryOp { op, operand },        // not, negate
    FunctionCall { callee, args },  // f(a, b, ...)
    ObjectLiteral { pairs },        // { key: val, ... }
    ListLiteral { elements },       // [a, b, ...]
    PropertyAccess { object, property },  // obj.prop
    IndexAccess { object, index },        // list[i]
    Lambda { params, body },        // (x) => { ... }
    StringInterpolation(Vec<StringPart>), // "hello {name}!"
    EnvVar(String),                 // $HOME
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
    Throw { value: Expression },
    TryCatch { body, catches: Vec<CatchClause>, finally },
    WithBlock { resources: Vec<(String, Expression)>, body },
    Defer { body },
}

pub struct CatchClause {
    pub param: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub body: Box<Statement>,
}

pub enum AssignTarget {
    Variable(String),
    Property { object, property },
    Index { object, index },
}

pub enum Literal { Bool(bool), Int(i64), Float(f64), String(String), Char(char), Null }
pub enum BinaryOperator { Add, Sub, Mul, Div, Mod, Eq, NotEq, Lt, Gt, LtEq, GtEq, And, Or }
pub enum UnaryOperator { Not, Negate }
pub struct Parameter { pub name: String, pub type_annotation: Option<TypeAnnotation> }
```

All types derive `Serialize` + `Deserialize` for JSON round-tripping.

---

## Convenience Constructors

```rust
Expression::int(42)               // Literal(Int(42))
Expression::string("hello")       // Literal(String("hello".into()))
Expression::char_lit('A')         // Literal(Char('A'))
Expression::ident("x")            // Identifier("x".into())
Expression::binary(op, left, right)
Expression::call(callee, args)
Statement::var_decl("x", expr)    // VariableDecl with no type annotation
Statement::ret(expr)              // Return { value: Some(expr) }
Statement::throw(expr)            // Throw { value: expr }
Statement::try_catch(body, catches, finally)
Statement::with_block(resources, body)
Statement::defer(body)            // Defer { body }
CatchClause::new(param, body)     // untyped catch
CatchClause::typed(param, type_annotation, body)
```

---

## Builder API (`builder.rs`)

Fluent builders for constructing programs without deep nesting:

- **`ProgramBuilder`** — top-level: `.function()`, `.var_decl()`, `.stmt()`, `.expr_stmt()`, `.build()`
- **`BlockBuilder`** — block-level: `.var_decl()`, `.assign()`, `.ret()`, `.if_then()`, `.if_else()`, `.while_loop()`, `.for_each()`, `.function()`, `.throw()`, `.try_catch()`, `.defer()`, `.build()`

The builder closures (`|b| b.ret(...)`) return `&mut BlockBuilder` for chaining.

---

## Display (`display.rs`)

`Program` implements `fmt::Display`, producing pseudo-code:

```ish
fn factorial(n) {
    if n <= 1 {
        return 1
    } else {
        return n * factorial(n - 1)
    }
}
```

---

## Tests

- `lib.rs`: 8 tests (AST construction and serialization)
- `builder.rs`: 2 tests (builder API)
- `display.rs`: 1 test (display formatting)

---

## Error Handling Nodes

The following AST nodes support error handling:

- **`Throw { value }`** — Raises the evaluated expression as a thrown error.
- **`TryCatch { body, catches, finally }`** — Executes the body; if a throw occurs, matches against catch clauses; always executes the finally block.
- **`CatchClause { param, type_annotation, body }`** — Binds the thrown value to `param` and executes the body. The `type_annotation` enables type-based catch matching (not yet implemented).
- **`WithBlock { resources, body }`** — Initializes resources, executes the body, then closes resources in reverse order.
- **`Defer { body }`** — Schedules the body to execute when the enclosing function exits (function-scoped, not block-scoped).

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/vm.md](vm.md)
