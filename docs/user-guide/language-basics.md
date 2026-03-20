---
title: Language Basics
category: user-guide
audience: [human-dev]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/syntax.md, docs/spec/types.md]
---

# Language Basics

ish is a C-family language with braces, `fn` for functions, and `let` for variables. Statements are newline-terminated; semicolons are optional.

---

## Variables

```ish
let x = 5           // immutable
let mut y = 10      // mutable
y = 20              // OK — y is mutable
// x = 10           // ERROR — x is immutable

// Type annotation
let z: i32 = 42
```

## Expressions

Standard arithmetic, comparison, and logical operators:

```ish
let sum = a + b
let bigger = x > y
let both = a and b
let either = a or b
let negated = not a
```

## Control Flow

Parentheses around conditions are not used:

```ish
if condition {
    // ...
} else {
    // ...
}

while condition {
    // ...
}

for item in collection {
    println(item)
}
```

## Comments

```ish
// Line comment
# Shell-style line comment

/* Block comment */
```

## Functions

```ish
fn greet(name) {
    println("Hello, " + name + "!")
}

fn add(a: i32, b: i32) -> i32 {
    return a + b
}
```

For more detail, see [Functions](functions.md) and the [type system specification](../spec/types.md).

---

## Naming Conventions

ish uses consistent naming conventions:

| Kind | Convention | Examples |
|------|-----------|----------|
| Variables | `snake_case` | `user_name`, `total_count` |
| Functions | `snake_case` | `get_user`, `is_type`, `validate` |
| Types | `PascalCase` | `String`, `Person`, `HttpResponse` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_PORT` |
| Modules | `snake_case` | `std::io`, `http_client` |
| Entry types | `PascalCase` | `Error`, `CodedError`, `Mutable` |
| Standards | `snake_case` | `streamlined`, `cautious`, `rigorous` |

For full details, see the [naming conventions specification](../spec/syntax.md#naming-conventions).

---

## Referenced by

- [docs/user-guide/INDEX.md](INDEX.md)
