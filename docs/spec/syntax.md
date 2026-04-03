---
title: ish Syntax
category: spec
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/types.md, docs/spec/assurance-ledger.md, docs/spec/modules.md, docs/spec/execution.md, docs/project/proposals/language-syntax.md]
---

# ish Syntax

ish is a C-family language with braces for blocks, `fn` for functions, and `let` for variables. It operates in two modes: **programming mode** (the default for `.ish` source files) and **shell mode** (the default for the interactive shell). The syntax is designed to serve both modes with minimal friction.

All syntax decisions are documented in the [language syntax proposal](../project/proposals/language-syntax.md).

---

## Comments

Both `//` and `#` are accepted as line comment starters. Block comments use `/* */` with nesting support.

```ish
// This is a line comment
# This is also a line comment (shell-style)

/*
  This is a block comment.
  /* Nested block comments are allowed. */
*/
```

Documentation should prefer `//` by convention.

---

## Statements and Semicolons

Statements are newline-terminated. Semicolons are optional — used to separate multiple statements on one line or for explicit termination. Multiline expressions continue when the line ends with an operator, open bracket, or explicit continuation (`\`).

```ish
let a = 1; let b = 2   // two statements on one line
let c = some_long_expression
    + more_stuff        // continuation: line ends with operator
```

---

## Variables and Expressions

Variables are immutable by default. Use `mut` for mutable variables.

```ish
let x = 5
let mut y = 10
y = 20

// Type annotation
let z: i32 = 42
```

### Operators

```ish
// Arithmetic
a + b
a - b
a * b
a / b
a % b

// Comparison
a == b    // structural equality (single kind; no ===)
a != b
a < b
a > b
a <= b
a >= b

// Logical
a and b
a or b
not a

// Unary
-x
```

Logical operators are `and`, `or`, `not` — not `&&`/`||`. This avoids conflicts with shell command chaining in shell mode. `+` is used for string concatenation.

---

## Data Structures

```ish
// Object literal
let person = { name: "Alice", age: 30 }

// List literal
let nums = [1, 2, 3]

// Property access
person.name

// Index access
nums[0]
```

---

## Strings

ish uses **shell-convention** quoting: single-quoted strings are literal (no interpolation), double-quoted strings support interpolation and escapes. This matches the universal shell expectation that `'...'` is raw and `"..."` processes special characters.

The full string syntax was designed in the [string syntax proposal](../project/proposals/string-syntax.md).

### Single-Quoted Strings (Literal)

Single-quoted strings are literal — no interpolation, no escape processing except `\\` and `\'`:

```ish
let raw = 'no {interpolation} or $expansion here'
let escaped = 'it\'s fine'
let backslash = 'path\\to\\file'
let regex = '(\d+)\s+'
```

### Double-Quoted Strings (Interpolating)

Double-quoted strings support ish expression interpolation with `{expr}` and environment variable expansion with `$VAR`:

```ish
let name = "Alice"
let greeting = "Hello, {name}!"              // -> "Hello, Alice!"
let calc = "Result: {2 + 2}"                 // -> "Result: 4"
let home = "Home is $HOME"                   // -> "Home is /home/alice"
let path = "Path is: ${PATH}"                // -> "Path is: /usr/bin:..."
```

**Escape sequences** in double-quoted strings: `\n` (newline), `\t` (tab), `\r` (carriage return), `\\` (backslash), `\"` (double quote), `\{` (literal brace), `\}` (literal brace), `\$` (literal dollar), `\0` (null), `\u{XXXX}` (Unicode scalar).

### Triple-Quoted Strings (Multiline)

Triple-quoted strings support multiline content with automatic indentation stripping based on the closing delimiter's position:

```ish
let sql = """
    SELECT *
    FROM users
    WHERE active = true
    """

let template = '''
    No {interpolation} or $expansion here.
    Everything is literal.
    '''
```

`"""..."""` supports interpolation (same as `"..."`). `'''...'''` is literal (same as `'...'`).

### Char Literals

Char literals use the `c'...'` prefix syntax:

```ish
let ch = c'A'
let newline = c'\n'
let null_char = c'\0'
```

The `char()` builtin constructs a char from a single-character string or Unicode code point:

```ish
let c = char("A")     // from string
let c = char(65)      // from code point
```

### Extended Delimiter Strings

For strings containing quotes or other special characters, extended delimiters wrap the string with `~`:

```ish
let s = ~"She said "hello" and it's fine"~
let t = ~'Contains both "quotes" and apostrophes'~
```

Extended delimiters are available for all four quote styles: `~"..."~`, `~'...'~`, `~"""..."""~`, `~'''...'''~`. Content inside extended delimiters is verbatim — no escape processing.

Extended delimiters are not available in shell mode. If you need an extended delimiter string as a shell argument, assign it to a variable first and interpolate it.

---

## Control Flow

Parentheses around conditions are **prohibited** (not optional — they are not allowed). Braces are required for all blocks.

```ish
// Conditional
if x > 0 {
    println("positive")
} else if x == 0 {
    println("zero")
} else {
    println("negative")
}

// While loop
while condition {
    // body
}

// For-each loop
for item in collection {
    println(item)
}

// Break and continue
while true {
    if done {
        break
    }
    if skip {
        continue
    }
}
```

There is no C-style `for (init; cond; step)` loop. The `match` keyword is reserved for a future pattern-matching feature. There is no `loop` keyword — use `while true` for infinite loops.

---

## Functions and Closures

Functions are declared with `fn`. Named functions require explicit `return`. Return type is declared with `-> Type`.

```ish
// Named function
fn add(a: i32, b: i32) -> i32 {
    return a + b
}

// Untyped (low-assurance)
fn greet(name) {
    println("Hello, " + name + "!")
}

// Default parameters
fn connect(host: String, port: i32 = 8080) {
    // ...
}
```

### Lambdas

TypeScript-style arrow functions with both expression and block bodies. Expression-body lambdas use implicit return (last expression). Block-body lambdas require explicit `return`.

```ish
// Expression body (implicit return)
let double = (x) => x * 2

// Block body (explicit return)
let process = (x) => {
    let y = transform(x)
    return y
}
```

### Closures

Functions capture variables from their enclosing scope:

```ish
fn make_counter() {
    let mut count = 0
    return () => {
        count = count + 1
        return count
    }
}
```

### Function Types

Function type syntax uses `fn(Args) -> Ret`:

```ish
type Handler = fn(Request) -> Response
```

---

## Error Handling

```ish
// Throw
throw new_error("something went wrong")

// Try/catch/finally
try {
    let data = read_file("config.json")
} catch (e) {
    println("Error: " + error_message(e))
} finally {
    cleanup()
}

// Typed catch clauses
try {
    risky_operation()
} catch (e: NotFoundError) {
    handle_not_found(e)
} catch (e: PermissionError) {
    handle_permission(e)
} catch (e) {
    handle_unknown(e)
}

// ? operator (sugar for throw-on-error)
let data = read_file("config.json")?

// With blocks (resource management)
with (f = open_file("data.txt")) {
    let contents = f.read()
}

// Defer (function-scoped cleanup)
fn process() {
    let conn = connect_to_db()
    defer conn.disconnect()
    // ...
}
```

See [docs/user-guide/error-handling.md](../user-guide/error-handling.md) for full details.

---

## Concurrency

### Async Functions

Functions are declared async with the `async` keyword before `fn`. At low assurance, async declaration is optional — the runtime infers it.

```ish
// Explicit async function
async fn fetch_data(url: String) -> String {
    let response = await http.get(url)
    return response.body
}

// At low assurance, async keyword is optional
fn fetch_data(url) {
    let response = http.get(url)    // implicitly awaited
    return response.body
}
```

### Await

The `await` keyword suspends the caller until the awaited future resolves. `await` syntactically requires a function call — `await expr` where `expr` is not a call is a parse error (produces an `Incomplete` node). At low assurance, `await` is implicit when calling functions that return futures (implied await).

```ish
let result = await some_async_function()
```

### Spawn

The `spawn` keyword starts an async operation and returns a `Future<T>` immediately without suspending the caller. Like `await`, `spawn` syntactically requires a function call.

```ish
let future = spawn do_something()
// ... do other work ...
let result = await future
```

### Yield

The `yield` keyword explicitly gives up control to the Tokio scheduler.

```ish
yield    // explicit cooperative yield point
```

#### `yield every` (Loop Syntax)

Statement-count-based yielding for loops (available at higher assurance levels):

```ish
for item in items yield every 500 {
    process(item)
}
```

#### Yield Budget Annotation

Custom time-based yield threshold:

```ish
@[yield_budget(500us)]
for item in items {
    process(item)
}
```

#### Unyielding Annotation

Suppress yielding for a block:

```ish
@[unyielding]
for item in items {
    process(item)
}
```

See [docs/spec/concurrency.md](concurrency.md) for full concurrency semantics.

---

## Type Declaration Syntax

```ish
// Type alias
type Name = String

// Union type
type Result = Success | Failure
type Direction = "north" | "south" | "east" | "west"

// Object type
type Person = {
    name: String,
    age?: i32,          // optional
    mut score: f64,     // mutable property
}

// Tuple type
type Point = (f64, f64)

// Function type
type Handler = fn(Request) -> Response
```

Nominal typing is handled through entries/annotations in the assurance ledger, not through a `nominal type` keyword. See [docs/spec/assurance-ledger.md](assurance-ledger.md).

---

## Visibility

All symbols default to `pub(self)` (visible only within the current module). The default visibility is configurable via a standard.

```ish
fn internal_helper() { ... }         // pub(self) — default
pub fn exported() { ... }            // pub(global)
pub(super) fn parent_only() { ... }  // visible to parent module
pub(project) fn project_wide() { ... } // visible within the project
```

Bare `pub` means `pub(global)`.

---

## Assurance Ledger Syntax Constructs

See [docs/spec/assurance-ledger.md](assurance-ledger.md) for full details.

| Construct | Syntax | Scope |
|-----------|--------|-------|
| Apply standard to scope | `@standard[name]` | block, function, module |
| Inline feature override | `@standard[feature(state)]` | block, function, module |
| Multi-feature override | `@standard[feat1(state), feat2(state)]` | block, function, module |
| Apply entry to item | `@[entry(params)]` | variable, property, function, type, statement |
| Define a standard | `standard name [...]` | module, function, block |
| Extend a standard | `standard name extends base [...]` | module, function, block |
| Define an entry type | `entry type name { ... }` | module level |

---

## Modules

```ish
use std::io
use mylib::utils

mod helpers
pub mod api
```

See [docs/spec/modules.md](modules.md) for full details.

---

## Shell Mode

In the interactive shell, lines are parsed as **command invocations** unless they begin with a recognized language keyword or have unambiguous language syntax (assignment, type annotation, etc.).

**Recognized keywords** (non-exhaustive): `let`, `mut`, `fn`, `if`, `else`, `while`, `for`, `match`, `return`, `use`, `mod`, `pub`, `type`, `standard`, `entry`, `try`, `catch`, `finally`, `throw`, `with`, `defer`, `break`, `continue`.

```ish
// Command invocations (bare words)
git status
ls -la *.rs
cat file.txt | grep "hello"
cargo build 2>&1

// Language statements
let result = $(ls -la)          // capture command output
fn deploy() { ... }
if file_exists("x") { ... }
use std::io
```

### Shell-Specific Syntax

```ish
// Pipe
ls -la | grep ".rs"

// Redirection
cargo build > build.log 2>&1

// Background
long_running_task &

// Command substitution
let files = $(ls -la)

// Environment variables
echo $HOME
echo ${PATH}

// Glob expansion
ls *.rs

// Command chaining
cargo build && cargo test      // && means "and then if success" in shell mode
cargo build; cargo test        // unconditional chaining
```

### Force-Command Prefix

Use `>` to force a line to be parsed as a command invocation when it would otherwise be parsed as a language statement:

```ish
> some_function_name arg1 arg2   // force command mode
```

### Shell Mode Restrictions

A standard can prevent the use of shell mode. Developers can configure this in their project file to ensure source code never invokes shell commands as a result of a syntax error. The `$(cmd)` command substitution syntax also has a standard to restrict its use.

### Project Definition in Shell Mode

```ish
// Interactive import
use mylib::utils

// Project detection: if ish.toml exists in the current directory,
// its dependencies are available automatically.

// Shell profile: ~/.ish/profile.ish is executed on shell startup.
```

---

## Naming Conventions

ish uses consistent naming conventions across all code and declarations.

| Kind | Convention | Examples |
|------|-----------|----------|
| Variables | `snake_case` | `user_name`, `total_count` |
| Functions | `snake_case` | `get_user`, `is_type`, `validate` |
| Types | `PascalCase` | `String`, `Person`, `HttpResponse` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_PORT` |
| Modules | `snake_case` | `std::io`, `http_client` |
| Entry types | `PascalCase` | `Error`, `CodedError`, `Mutable`, `Type` |
| Standards | `snake_case` | `streamlined`, `cautious`, `rigorous` |
| Keywords | `lowercase` | `let`, `fn`, `if`, `mut` |

These conventions match the existing prototype and Rust conventions. Built-in functions use `snake_case` (e.g., `is_type`, `error_message`, `type_of`). Built-in entry types use `PascalCase` (e.g., `Error`, `CodedError`, `SystemError`, `Mutable`, `Type`, `Open`, `Closed`).

---

## Parser

The parser uses [pest](https://pest.rs/) (PEG parser generator) with an error-accepting grammar. For each construct, the grammar contains rules for both valid and invalid forms, allowing the parser to always succeed and produce a parse tree. Error reporting walks the tree and generates diagnostics for invalid nodes.

The grammar is structured in layers: lexer rules (keywords, operators, literals) → expression rules → statement rules → top-level rules (language statement vs. shell command).

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/concurrency.md](concurrency.md)
- [docs/project/proposals/language-syntax.md](../project/proposals/language-syntax.md)
- [docs/user-guide/language-basics.md](../user-guide/language-basics.md)
