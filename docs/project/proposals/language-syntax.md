---
title: "Proposal: Language Syntax"
category: proposal
audience: [all]
status: pending
last-verified: 2026-03-14
depends-on: [docs/project/rfp/language-syntax.md, docs/spec/syntax.md, docs/spec/types.md, docs/spec/execution.md, docs/spec/modules.md, docs/spec/assurance-ledger.md, GLOSSARY.md]
---

# Proposal: Language Syntax

*Generated from [language-syntax.md](../rfp/language-syntax.md) on 2026-03-14.*

---

## Questions and Answers

### Q: Where do the four source syntaxes (Rust, TypeScript, old prototype, existing docs) agree and disagree?

The four sources broadly agree on:

- **C-family block structure:** Braces `{}` for blocks, parentheses `()` for grouping, semicolons as terminators or separators.
- **`let` for variable declaration,** `mut` for mutability (Rust influence adopted by the ish docs).
- **`fn` for function declaration** (Rust keyword, adopted in ish docs).
- **`if`/`else` for conditionals** (universal).
- **Standard arithmetic and comparison operators** (`+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=`).
- **Object literals** with `{ key: value }` syntax (TypeScript/JavaScript, adopted in ish docs and old prototype).
- **Array/List literals** with `[a, b, c]` syntax (universal).
- **Lambda/closure syntax** with `=>` (TypeScript arrow functions, adopted in ish docs).
- **`true`, `false`, `null`** as keywords (universal; old prototype has these).

Key disagreements are covered in the feature sections below.

### Q: How do other languages handle the shell mode vs. programming language tension?

Several languages and shells have tackled this problem:

1. **Nushell** — Takes the approach of being a shell first, with a structured data pipeline model. Programming constructs (`if`, `def`, `for`) are built into the shell grammar. Bare words are treated as external commands. There is no separate "programming mode" — the language is always the shell.

2. **PowerShell** — Uses a cmdlet model where commands are `Verb-Noun` identifiers. Programming constructs are always available. Bare words invoke commands; `$variable` syntax distinguishes variables. The `$` sigil avoids ambiguity.

3. **Fish** — Functions as a shell with programming constructs (`if`, `while`, `function`, etc.) using keyword-delimited blocks (`end` instead of `{}`). Every line is fundamentally a command invocation unless it starts with a recognized keyword.

4. **Xonsh** — Python-based shell that switches between Python mode and subprocess mode. Lines starting with Python syntax (assignments, keywords, etc.) are Python; other lines are subprocess commands. Uses `$()` for subprocess capture in Python mode and `@()` for Python evaluation in subprocess mode.

5. **Oil/Ysh** — Maintains two sub-languages: a shell-compatible command language and an expression language. Uses explicit `var`, `setvar` for programming mode. Bare words are commands. `()` forces expression context.

6. **Elvish** — Uses `{` `}` for closures, `$` for variables, and bare words for commands. Has a clean separation: if a line starts with a known keyword or assignment, it's code; otherwise it's a command.

7. **Raku (Perl 6)** — Not a shell, but its grammar is relevant. It uses a "longest token match" rule and distinctive sigils (`$`, `@`, `%`) to disambiguate how bare words are treated.

The emerging consensus in modern shell-language hybrids is: **bare words default to command invocation, and a small set of reserved keywords switches context to programming mode.** This is essentially the approach described in the RFP.

### Q: How should project definition work in shell mode?

In shell mode, the user needs to import packages and call their functions interactively. Research shows three approaches:

1. **Explicit `use` statements** — The user types `use some::package;` at the shell prompt, and subsequent lines can call functions from that package. This is the simplest and most consistent with the module system spec.

2. **Project file detection** — The shell detects an `ish.toml` or similar project file in the current directory and auto-imports its dependencies. This mirrors how Rust's `cargo` or Node's `package.json` work.

3. **Shell profile** — A `~/.ish/profile.ish` or similar file is executed on shell startup, containing commonly used `use` statements and configurations.

These are not mutually exclusive. The recommendation is to support all three, implementing (1) immediately since it requires no additional infrastructure.

---

## Feature: Core Syntax — Variables, Expressions, and Statements

### Source Agreement

The four sources agree on the following, which can be adopted without alternatives:

```ish
// Variable declaration — immutable by default
let x = 5;
let mut y = 10;
y = 20;

// Type annotation
let x: i32 = 5;

// Object literal
let person = { name: "Alice", age: 30 };

// List literal
let nums = [1, 2, 3];

// Property access
person.name
nums[0]

// Function call (strict invocation)
add(1, 2)

// Binary operators
a + b
a - b
a * b
a / b
a % b
a == b
a != b
a < b
a > b
a <= b
a >= b

// Unary operators
-x
not x
```

### Issues to Watch Out For

- **Semicolons:** Rust requires them; TypeScript/JavaScript uses ASI (automatic semicolon insertion). The ish docs show semicolons in examples. The old pest grammar uses `;` as a statement separator. Shell mode needs newlines to terminate statements.
- **Logical operators:** Rust uses `&&`/`||`; TypeScript uses `&&`/`||`; the ish docs use `and`/`or`. The old prototype does not define these.
- **String concatenation:** The ish docs use `+` for string concatenation (TypeScript-style). Rust uses `format!` or `.push_str()`. This matters for a scripting language.
- **Equality:** Should ish have both `==`/`===` (TypeScript) or only `==` (Rust)? Since ish has a strong type system, JavaScript's type coercion problem doesn't apply.

### Critical Analysis

#### Semicolons

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Required semicolons** (Rust) | Unambiguous parsing; simple grammar; consistent with compiled-language feel | Annoying in shell mode; extra typing for interactive use |
| **B: Optional semicolons with ASI** (TypeScript/Go) | Friendly for shell/REPL use; familiar to JS/TS devs | ASI edge cases cause subtle bugs; grammar complexity |
| **C: Newline-terminated with optional semicolons** | Best for shell mode; semicolons available for multi-statement lines | Multiline expressions need continuation rules |

**Internet consensus:** Modern languages increasingly use newline-terminated statements (Go, Kotlin, Swift, Python). ASI (JavaScript-style) is widely considered a mistake.

**Recommendation:** **Alternative C** — newlines terminate statements; semicolons are optional (used to separate multiple statements on one line or for explicit termination). Multiline expressions continue when the line ends with an operator, open bracket, or explicit continuation (`\`). This serves both shell mode (no semicolons needed) and file mode (semicolons available for clarity).

#### Logical Operators

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `&&` / `||`** (Rust, TypeScript) | Familiar to most programmers; established convention | `&&` has meaning in shells (command chaining); conflicts with shell mode |
| **B: `and` / `or` / `not`** (ish docs, Python) | Readable; no shell conflict; already in ish docs | Less familiar to C-family devs; looks like a scripting language |
| **C: Both** (Perl, Raku — different precedence) | Maximum flexibility | Complexity; two ways to do the same thing; precedence confusion |

**Internet consensus:** `and`/`or`/`not` is gaining popularity in modern languages (Python, Ruby, Lua, Nim). The `&&` conflict with shell command chaining is a real concern for shell-hybrid languages.

**Recommendation:** **Alternative B** — `and`/`or`/`not`. These are already established in the ish docs, avoid shell conflicts, and are more readable. Short-circuit semantics as expected.

#### String Concatenation

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `+` operator** (TypeScript, Java, Python) | Familiar; already in ish docs | Overloading `+` for strings is controversial; implicit type coercion issues |
| **B: String interpolation only** (Rust-ish: `f"hello {name}"`) | No operator overloading ambiguity; powerful | Need a separate operator for concatenating string variables |
| **C: `+` for concat, plus string interpolation** | Best of both; familiar and powerful | Slight complexity |

**Recommendation:** **Alternative C** — support both `+` for string concatenation and string interpolation syntax (see Strings section below). The `+` operator is already in the docs and is expected by most developers.

### Proposed Implementation

Adopt the following core syntax:

```ish
// Variables
let x = 5
let mut y = 10
let z: i32 = 42

// Assignments
y = 20

// Logical
let result = a and b
let either = a or b
let negated = not a

// Comparison
a == b   // structural equality (single kind)

// Newline termination; optional semicolons
let a = 1; let b = 2   // two statements on one line
let c = some_long_expression
    + more_stuff        // continuation: line ends with operator
```

### Decisions

**Decision:** Semicolons — required, ASI, or newline-terminated with optional semicolons?
--> newline-terminated with optional semicolons

**Decision:** Logical operators — `&&`/`||` or `and`/`or`/`not`?
--> `and`/`or`/`not`

**Decision:** Allow `+` for string concatenation, or string interpolation only?
--> Both.  Ease of string manipulation is important for any language, but especially for a shell.

**Decision:** Single equality operator `==` (no `===`)?
--> Yes. (no `===`)

---

## Feature: Comments

### Source Agreement

Rust and TypeScript both use `//` for line comments and `/* */` for block comments. The old prototype does not define comment syntax. The ish docs do not specify comment syntax but use `//` in examples.

### Issues to Watch Out For

- Shell-style `#` comments are common in shell languages and expected by shell users.
- `//` conflicts with nothing in shell mode (it's not a valid path start on its own).
- Nested block comments are supported by Rust but not by TypeScript. Nested comments are useful for commenting out blocks that already contain comments.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `//` and `/* */`** (Rust/TS) | Familiar to most programmers; already used in ish docs | Unfamiliar to shell users |
| **B: `#` and `/* */`** (shell + C) | Natural for shell mode; `#` is universally recognized as comment in shells | Unfamiliar for Rust/TS developers; `#` may conflict with future syntax (e.g., attributes, macros) |
| **C: Both `//` and `#`** for line comments, `/* */` for block | Maximum compatibility | Two ways to do same thing; mild complexity |

**Internet consensus:** Languages that bridge shell and programming (Nushell, Oil/Ysh, Julia) tend to use `#` for comments. Compiled languages use `//`.

**Recommendation:** **Alternative C** — accept both `//` and `#` as line comment starters. This serves both audiences: programmers use `//`, shell users use `#`. Block comments use `/* */` with nesting support (Rust-style). The cost is minimal and the benefit to shell users is significant. Documentation should prefer `//` by convention.

### Proposed Implementation

```ish
// This is a line comment
# This is also a line comment (shell-style)

/*
  This is a block comment.
  /* Nested block comments are allowed. */
*/
```

### Decisions

**Decision:** Accept both `//` and `#` for line comments, or pick one?
--> Both

**Decision:** Support nested block comments (Rust-style)?
--> Yes

---

## Feature: Control Flow

### Source Agreement

All sources agree on `if`/`else` with braces. The old prototype has `if`/`else` with braces. The ish docs show `if`/`else` and `while`. The AST includes `If`, `While`, `ForEach`, and `Return`.

### Issues to Watch Out For

- **`for` loop syntax** — Rust uses `for x in iter`, TypeScript uses `for (x of iter)` and C-style `for`. The ish AST has `ForEach { variable, iterable, body }`.
- **`while` loop** — Universal agreement on `while (condition) { body }`.
- **Condition parentheses** — Rust does not require parens around `if`/`while` conditions; TypeScript does. The ish docs show parens.
- **`match`/`switch`** — Rust uses `match` with pattern matching; TypeScript uses `switch`. The ish AST does not include a match/switch node, but pattern matching is listed as an open question.
- **`loop`** — Rust has `loop` for infinite loops; TypeScript uses `while (true)`.
- **`break`/`continue`** — Universal in both Rust and TypeScript.

### Critical Analysis

#### Condition Parentheses

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Required parens** `if (x > 0) {}` (TypeScript, C, Java) | Familiar; clear visual separation between condition and body | Extra noise; Rust community considers them unnecessary |
| **B: No parens** `if x > 0 {}` (Rust, Go) | Cleaner; less typing; the braces already delimit the condition | Can look odd with complex conditions; unfamiliar to JS/TS devs |
| **C: Optional parens** (Swift, Kotlin) | Flexible; no wrong answer | Inconsistency in codebases |

**Internet consensus:** The trend in modern languages (Go, Rust, Swift, Kotlin) is away from required parentheses. However, the ish docs currently show parentheses.

**Recommendation:** **Alternative C** — optional parentheses. This respects both Rust (no parens) and TypeScript (parens) conventions, avoids forcing a choice, and makes the parser more forgiving. Linting or formatting tools can enforce a project-level preference.

#### For Loop Syntax

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `for x in iter`** (Rust, Python) | Clean; reads naturally; no parentheses needed | No C-style `for (init; cond; step)` |
| **B: `for (x of iter)`** (TypeScript) | Familiar to TS devs | Parens are noise; `of` vs `in` confusion (TS has both with different semantics) |
| **C: `for x in iter` plus `for (init; cond; step)`** | Full C-family compatibility | Two loop forms; C-style `for` is falling out of favor |

**Recommendation:** **Alternative A** — `for x in iter`. The C-style `for` loop is deprecated in practice (Rust dropped it; Kotlin/Swift don't use it). `while` covers the remaining cases. `in` is more readable than `of`.

#### Pattern Matching

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `match` with patterns** (Rust) | Powerful; safe; exhaustiveness checking; aligns with ish's type system | Complex to implement; not needed in phase 1 |
| **B: `switch`/`case`** (TypeScript, C) | Familiar | Fallthrough bugs; less powerful than Rust `match` |
| **C: `match` with Rust-style patterns, defer to later phase** | Can be designed properly; not rushed | Need a placeholder for the AST |

**Recommendation:** **Alternative C** — reserve the `match` keyword and design it in a later phase. It's a significant feature that deserves careful design to integrate with the ish type system (literal types, structural typing, union types). For now, `if`/`else if`/`else` chains suffice.

### Proposed Implementation

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

// Loop with break/continue
while true {
    if done {
        break
    }
    if skip {
        continue
    }
}

// Return
fn compute() -> i32 {
    return 42
}
```

### Decisions

**Decision:** Condition parentheses — required, prohibited, or optional?
--> prohibited

**Decision:** For loop — `for x in iter` or `for (x of iter)` or both?
--> `for x in iter`

**Decision:** Reserve `match` keyword for a later phase, or implement `switch`/`case` now?
--> Reserve `match` keyword for a later phase

**Decision:** Include a `loop` keyword for infinite loops (Rust-style)?
--> No.

---

## Feature: Functions and Closures

### Source Agreement

All sources agree on `fn` for function declaration. The ish docs show both named functions and lambdas with `=>`. The AST has `FunctionDecl` and `Lambda`.

### Issues to Watch Out For

- **Return type syntax** — Rust uses `-> Type` (after params); TypeScript uses `: Type` (after params). The ish docs use `-> Type`.
- **Lambda syntax** — The ish docs use `(params) => { body }` (TypeScript arrow syntax). Rust uses `|params| body` or `|params| { body }`.
- **Implicit return** — Rust returns the last expression if no semicolon; TypeScript requires `return`. This interacts with the semicolon decision.
- **Default parameters** — TypeScript supports them (`fn f(x = 5)`); Rust does not (uses `Option` instead).
- **Rest/spread parameters** — TypeScript has `...args`; Rust has no direct equivalent.
- **Generic/type parameters** — Both Rust and TypeScript use `<T>`. This can be deferred.

### Critical Analysis

#### Lambda Syntax

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `(x) => { body }`** (TypeScript) | Familiar to TS/JS devs; already in ish docs; clear visual distinction from function calls | Verbose for simple expressions; `=>` is an additional operator |
| **B: `\|x\| body`** (Rust) | Concise; no `=>` needed; established in Rust | `|` pipes conflict with shell mode; unfamiliar to non-Rust devs |
| **C: `(x) => expr` or `(x) => { body }`** (both forms) | Single-expression lambdas are concise; block form for multi-statement | Mild complexity; two forms |

**Recommendation:** **Alternative C** — TypeScript-style arrow functions with both expression and block bodies. The `|x|` syntax has a real conflict with shell pipe operators and would be confusing in shell mode. Arrow functions are already established in the ish docs.

```ish
let double = (x) => x * 2          // expression body
let process = (x) => {             // block body
    let y = x * 2
    return y + 1
}
```

#### Implicit Return

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Explicit `return` only** (TypeScript, C) | Unambiguous; clear intent | Verbose for short functions |
| **B: Last expression is return value** (Rust) | Concise; elegant for short functions | Interacts badly with optional semicolons; subtle bugs if you accidentally leave off semicolon |
| **C: Explicit `return` for functions, last-expression for lambdas** | Functions are clear; lambdas are concise | Inconsistency between two function forms |

**Recommendation:** **Alternative C** — explicit `return` for named functions, last-expression return allowed for lambda expression bodies (the `(x) => x * 2` form is an implicit return). Block-body lambdas use explicit `return`. This avoids the semicolon interaction problem while keeping lambdas concise.

#### Default Parameters

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Support defaults** `fn f(x: i32 = 5)` (TypeScript) | Convenient; reduces boilerplate | Complicates overload resolution; adds complexity |
| **B: No defaults; use `Option`** (Rust) | Simpler; explicit | Verbose; unfriendly in low-assurance mode |
| **C: Support defaults in later phase** | Avoids over-specifying now; can learn from usage | Feature gap in early phases |

**Recommendation:** **Alternative A** — support default parameters. This is important for a scripting-friendly language. Low-assurance ish should be approachable, and requiring `Option<T>` for simple defaults is hostile to that goal.

### Proposed Implementation

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

// Lambda — expression body
let double = (x) => x * 2

// Lambda — block body
let process = (x) => {
    let y = transform(x)
    return y
}

// Closure
fn make_counter() {
    let mut count = 0
    return () => {
        count = count + 1
        return count
    }
}
```

### Decisions

**Decision:** Lambda syntax — TypeScript arrow `(x) => expr` or Rust closure `|x| expr`?
--> `(x) => expr`

**Decision:** Implicit return — explicit-only, Rust-style last-expression, or hybrid?
--> Hybrid:  Explicit `return` for named functions, last-expression return allowed for lambda expression bodies (the `(x) => x * 2` form is an implicit return). Block-body lambdas use explicit `return`

**Decision:** Default parameters — support now, defer, or never?
--> Support now.

---

## Feature: Strings and String Interpolation

### Issues to Watch Out For

- The old prototype supports both single-quoted and double-quoted strings.
- Rust uses only double-quoted strings (single quotes are for `char`).
- TypeScript supports single, double, and backtick template literals.
- Shell mode benefits from unquoted strings for arguments.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Double quotes only; `f"..."` for interpolation** (Rust-ish) | Simple; char literals with `'`; clear distinction | Breaking change from old prototype; annoying for simple strings |
| **B: Both `"..."` and `'...'`; backtick templates** (TypeScript) | Familiar to web devs; flexible | Three string forms; `char` type needs different syntax |
| **C: `"..."` for strings, `'c'` for chars, `"...{expr}..."` for interpolation** | Clean; distinct char type; interpolation without extra prefix | Interpolation syntax `{expr}` may conflict with object literals inside strings |
| **D: `"..."` for plain strings, `f"...{expr}..."` for interpolation, `'c'` for chars** | Explicit interpolation mode; no ambiguity; char type preserved | Slightly more typing for interpolation; `f` prefix may look odd |

**Recommendation:** **Alternative D** — double-quoted strings, `f"..."` for interpolation, single-quoted char literals. Shell mode inherits unquoted strings from the shell grammar (see Shell Mode section). The `f` prefix is familiar from Python and becoming common.

### Proposed Implementation

```ish
let name = "Alice"
let greeting = f"Hello, {name}!"           // interpolation
let multiline = "line one\nline two"
let ch = 'A'                                // char literal
let raw = r"no \escapes \here"             // raw string (no escape processing)
```

### Decisions

**Decision:** String quote styles — double-only, or both single and double?
--> **Addressed in the [string syntax proposal](string-syntax.md).** Shell convention adopted: `'...'` for literal strings, `"..."` for interpolating strings, `"""..."""` and `'''...'''` for multiline, `c'A'` for char literals, `~"..."~` for extended delimiters.

**Decision:** Interpolation syntax — `f"...{expr}..."`, `` `...${expr}...` ``, or `"...{expr}..."`?
--> **Addressed in the [string syntax proposal](string-syntax.md).** Implicit interpolation in `"..."` with `{expr}` for ish expressions and `$VAR` for environment variables.

**Decision:** Raw strings — `r"..."` (Rust-style)?
--> **Addressed in the [string syntax proposal](string-syntax.md).** No `r"..."` prefix — single-quoted strings serve as literal (raw) strings, and extended delimiters (`~"..."~`) handle edge cases.

---

## Feature: Type Declaration Syntax

### Source Agreement

The ish docs already establish:

```ish
type Direction = "north" | "south" | "east" | "west"   // union of literals
nominal type UserId = i64                                // nominal wrapper
type Person = { name: String, age?: i32 }               // object type
type StringMap = { [key: String]: i32 }                  // index signature
```

### Issues to Watch Out For

- **Union type syntax** — both Rust and TypeScript use `|` but with different semantics. Rust enums are tagged unions; TypeScript unions are structural. The ish docs use `|` (TypeScript-style).
- **Tuple syntax** — Rust uses `(T1, T2)`, TypeScript uses `[T1, T2]`. The ish docs use `(T1, T2)` (Rust-style).
- **Generic syntax** — Both use `<T>`. Can be deferred.
- **The `|` operator in shell mode** — pipe operator `|` conflicts with union type `|`. This needs careful grammar design.

### Critical Analysis

The `|` conflict between union types and shell pipes is real but manageable. In programming-language context (after `type`, in type annotations), `|` means union. In shell context (between commands), `|` means pipe. The grammar mode switch handles this naturally.

### Proposed Implementation

Adopt the syntax already established in the ish documentation. No alternatives needed since the docs already specify this clearly.

```ish
// Type alias
type Name = String

// Union type
type Result = Success | Failure
type Direction = "north" | "south" | "east" | "west"

// Nominal type
nominal type UserId = i64

// Object type
type Person = {
    name: String,
    age?: i32,          // optional
    mut score: f64,     // mutable property
}

// Tuple type
type Point = (f64, f64)

// Function type (proposed)
type Handler = fn(Request) -> Response
```

### Decisions

**Decision:** Function type syntax — `fn(Args) -> Ret` or `(Args) => Ret`?
--> fn(Args) -> Ret

--> The nominal type syntax is out of date and should be removed from the type specification.  Nominal typing is now handled through entries/annotations.  See the assurance ledger spec.

---

## Feature: Visibility

### Source Agreement

The module spec defines visibility as: `pub(self)` (default), `pub(super)`, `pub(in path)`, `pub(project)`, `pub(global)`. This is Rust-style with extensions. TypeScript uses `export`.

### Issues to Watch Out For

- The `pub(...)` syntax is verbose for low-assurance code.
- Shell mode doesn't need visibility modifiers (everything is local).

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `pub(...)` as specified** (Rust-style) | Already in the module spec; granular control; consistent | Verbose |
| **B: `pub` as shorthand for `pub(project)`, full form available** | Less typing for common case; still granular when needed | Different default from Rust's `pub` (which means fully public) |
| **C: `export` keyword** (TypeScript-style) | Familiar to TS devs; simpler model | Doesn't support fine-grained visibility; not in the module spec |

**Recommendation:** **Alternative B** — bare `pub` defaults to `pub(project)` (the most common use case). Full `pub(scope)` syntax available for fine-grained control. This is already essentially what the module spec describes.

### Proposed Implementation

```ish
fn internal_helper() { ... }       // pub(self) — default, no keyword needed
pub fn api_function() { ... }      // pub(project) — visible within the project
pub(super) fn parent_only() { ... } // visible to parent module
pub(global) fn exported() { ... }   // visible to external consumers
```

### Decisions

**Decision:** Bare `pub` means `pub(project)` or `pub(global)`?
--> `pub(global)`.  It would be very uninituitive to have pub mean pub(project).  Instead, we can have the default visibility be pub(project), although the default visibility should be specifiable via a standard.

---

## Feature: Shell Mode and Command-Line Mode

This is the most novel and challenging aspect of the ish syntax design.

### Issues to Watch Out For

- **Ambiguity:** `git status` — is this calling function `git` with argument `status`, or invoking the `git` executable with argument `status`?
- **Quoting:** Shell commands often include unquoted arguments with special characters (`*.txt`, `~/docs`, `$HOME`).
- **Pipes and redirection:** `cmd1 | cmd2 > file` — must coexist with programming operators.
- **Globs:** `*.rs` needs to work in shell mode without being a syntax error.
- **Background processes:** `cmd &` — conflicts with `&` as a reference or bitwise operator.
- **Exit codes:** Shell commands return integers; ish functions return values.
- **Environment variables:** `$HOME` vs variable access.

### Critical Analysis

#### Overall Shell/Language Integration Strategy

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Reserved-word mode switch** (RFP's current approach) | Simple model; a few keywords switch to "language mode" | Every new keyword is a potential conflict with a command name; how to exit "language mode"? |
| **B: Shell-first with explicit programming blocks** | Natural shell experience; `{ }` or `do/end` for multi-statement programming | Programming feels second-class; verbose for mixed usage |
| **C: Language-first with shell escape** | Programming feels natural; shell commands via prefix (`!`, `$`, or backtick) | Shell experience suffers; common operations need prefix |
| **D: Context-sensitive parsing (Xonsh model)** | Automatic detection; minimal ceremony | Complex; surprising misdetections; hard to reason about |
| **E: Dual-mode with explicit toggle** | Clear; no ambiguity; user controls mode | Friction switching; easy to forget which mode you're in |
| **F: Bare-word command invocation with sigil-free language integration** | Lines starting with known keywords or containing `=` or `:` are language; others are commands | Simple rule; predictable; works like refined Elvish/Fish model |

**Recommendation:** **Alternative F**, refined as follows:

**The Rule:** A line is parsed as a *language statement* if it begins with a recognized keyword or has unambiguous language syntax (assignment, type annotation, etc.). Otherwise, it is parsed as a *command invocation*.

**Recognized keywords** (non-exhaustive): `let`, `mut`, `fn`, `if`, `else`, `while`, `for`, `match`, `return`, `use`, `mod`, `pub`, `type`, `nominal`, `standard`, `entry`, `try`, `catch`, `finally`, `throw`, `with`, `defer`, `break`, `continue`, `loop`.

**Command invocation:**
```
git status                    // invokes git with arg "status"
ls -la *.rs                   // shell glob and flags
cat file.txt | grep "hello"  // pipe
cargo build 2>&1              // redirection
```
**Language statements:**
```
let result = git status       // captures command output into variable
fn deploy() { ... }           // function declaration
if file_exists("x") { ... }  // conditional
use std::io                   // module import
```

**Explicit mode markers for edge cases:**
```
> some_function_name arg1 arg2   // force command mode (if function name matches a command)
! let                            // force command mode for a keyword-named command (rare)
```

This approach:
- Makes the common case natural in both modes
- Only requires disambiguation for rare edge cases
- Has precedent in Elvish and Fish
- Avoids the complexity of Xonsh's heuristic detection

### Shell-Specific Syntax Elements

```ish
// Pipe
ls -la | grep ".rs"

// Redirection
cargo build > build.log 2>&1

// Background
long_running_task &

// Capture command output to variable
let files = $(ls -la)          // $() for command substitution
let count = $(wc -l < file.txt)

// Environment variables in shell mode
echo $HOME
echo ${PATH}

// Glob expansion (shell mode only)
ls *.rs

// Command chaining
cargo build && cargo test      // in shell mode, && means "and then if success"
cargo build; cargo test        // unconditional chaining
```

### Project Definition in Shell Mode

```ish
// Interactive import
use mylib::utils               // loads the package, makes utils available

// Project detection (automatic)
// If ish.toml exists in current directory, its dependencies are available

// Shell profile (~/.ish/profile.ish)
// Executed on shell startup; can contain use statements and configuration
```

### Proposed Implementation Phases

1. **Phase 1:** Language keywords recognized; all other lines are command invocations. No pipes or redirection yet — commands are simple bare-word invocations.
2. **Phase 2:** Add pipes (`|`), redirection (`>`, `>>`, `2>&1`), command substitution (`$()`), and background (`&`).
3. **Phase 3:** Add glob expansion, environment variable interpolation `$VAR`, and shell profile support.
4. **Phase 4:** Project file detection and auto-import.

### Decisions

**Decision:** Shell/language integration strategy — reserved-word mode switch, context-sensitive parsing, or keyword-based rule?
--> keyword-based rule.  But also there should be a standard that prevents the use of shell mode.  Developers can put this in their project file, and prevent the project's source code from ever invoking shell commands as a result of a syntax error.

**Decision:** Command substitution syntax — `$(cmd)`, `` `cmd` ``, or other?
--> `$(cmd)` works in both modes, and in interpolated strings.  There needs to be a standard to restrict its use.

**Decision:** Should `&&` in shell mode mean "and then" (shell convention) while `and` means logical-and (programming convention)?
--> Yes

**Decision:** Force-command prefix — `>`, `!`, `$`, or other?
--> `>`

---

## Feature: Error Handling Syntax

### Source Agreement

The ish docs and AST already establish error handling syntax. This is largely settled:

```ish
// Try/catch/finally
try {
    let data = read_file("config.json")
} catch (e) {
    println("Error: " + error_message(e))
} finally {
    cleanup()
}

// With blocks
with (f = open_file("data.txt")) {
    let contents = f.read()
}

// Defer
fn process() {
    let conn = connect_to_db()
    defer conn.disconnect()
    // ...
}

// Throw
throw new_error("something went wrong")
```

### Issues to Watch Out For

- **`?` operator** — listed as an open question. Rust uses `?` for error propagation. This is extremely convenient and well-loved.
- **Typed catch clauses** — multiple `catch` blocks with type annotations are mentioned but not yet specified.
- **`throws` declaration** — shown in the assurance-ledger docs: `fn get_user(id: i64) -> User throws NotFoundError`.

### Critical Analysis

#### `?` Operator

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Support `?` operator** (Rust-style) | Concise; beloved by Rust community; reduces boilerplate | Another way to do error handling; interacts with `throws` declaration |
| **B: No `?` operator** | Simpler; only one error handling mechanism | Verbose; lots of try/catch boilerplate for "just propagate" cases |
| **C: Defer `?` to later phase** | Avoids premature commitment | Missing a beloved feature |

**Recommendation:** **Alternative C** — defer `?` to a later phase. The try/catch/throw mechanism is sufficient for initial phases, and `?` interacts with the return type system in ways that need careful design (should `?` return a `Result` type? How does it interact with `throws`?).

#### Typed Catch Clauses

```ish
try {
    risky_operation()
} catch (e: NotFoundError) {
    handle_not_found(e)
} catch (e: PermissionError) {
    handle_permission(e)
} catch (e) {
    handle_unknown(e)
}
```

This follows TypeScript's pattern and is consistent with the existing ish syntax. Recommend adopting it.

### Proposed Implementation

Adopt the existing syntax from the ish docs. Add typed catch clauses. Reserve `?` for a future phase.

### Decisions

**Decision:** Support `?` operator now, defer, or never?
--> Now.  The `?` operator should have been fully specified in the error documentation.  Please check if it was not.  It is syntactic sugar for detecting if the return value of a function is an error type, and throwing it if it is.

--> Also, this example in the assurance ledger docs is wrong `fn get_user(id: i64) -> User throws NotFoundError`.  The correct syntax is: `fn get_user(id: i64) -> User | NotFoundError`.  ish functions never throw, the always return.  Throwing is a behavior of the calling function.

**Decision:** Typed catch clauses — adopt TypeScript-style syntax?
--> Yes.

---

## Feature: Assurance Ledger Syntax Integration

The assurance-ledger syntax is already well-specified in the ledger spec. The key syntax elements integrate into the broader language as annotations:

```ish
@standard[cautious]
@[mutable] @[type(i32)]
@standard[overflow(saturating)]
standard name extends base [ ... ]
entry type name { ... }
```

No alternatives are needed — this is settled. The only consideration is ensuring the `@` prefix for annotations doesn't conflict with other syntax (it doesn't in any of the four source languages).

---

## Feature: Parser Strategy

### Issues to Watch Out For

- **Error recovery** — a grammar that accepts malformed input must carefully track *where* the input is malformed to generate good error messages.
- **Performance** — PEG parsers (pest) can have poor worst-case performance on some grammar structures (exponential backtracking).
- **Maintenance** — pest grammars can become complex and difficult to debug.
- **Shell mode parsing** — shell command invocation (bare words, globs, pipes) is very different from programming language parsing. The parser needs to handle both.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: pest with error-accepting grammar** (RFP's approach) | Pest is proven in the prototype; PEG is declarative and maintainable; error-accepting rules give control over error messages; Rust-native | PEG backtracking can cause performance issues; error recovery rules add grammar complexity; pest has limited error recovery primitives compared to hand-written parsers |
| **B: Hand-written recursive descent parser** | Maximum control over error recovery; can produce the best error messages (Rust's own parser, GCC); predictable performance; straightforward shell mode integration | More code to write and maintain; easier to introduce bugs; no declarative grammar spec |
| **C: Tree-sitter** | Excellent error recovery built-in; incremental parsing (great for IDE); established ecosystem | Generates C code; FFI complexity; grammar language is JavaScript; less control over error message specifics |
| **D: LALR/LR parser (lalrpop)** | Strong theoretical foundation; efficient; Rust-native | Poor error recovery; not well-suited for PEG-like flexibility; rigid grammar constraints |
| **E: pest for grammar specification, hand-written for error recovery** | Declarative grammar as spec; pest handles happy path; hand-written code handles error cases | Two parsing systems to maintain; complexity |
| **F: Chumsky** | Rust-native parser combinator library; excellent error recovery support; composable | Younger ecosystem; parser combinators can be hard to read; less declarative than PEG |

**Internet consensus:** The Rust community is moving toward either hand-written parsers (for maximum quality, e.g., rust-analyzer) or parser combinator libraries (winnow, chumsky) for new projects. Pest remains popular for simpler grammars. Tree-sitter dominates the IDE space.

**Recommendation:** **Alternative A (pest) with refinements.** The rationale:

1. **Proven in the prototype** — the team has experience with pest and the existing grammar provides a starting point.
2. **Error-accepting grammar works** — the approach of writing rules that "recognize" bad input is sound and matches pest's PEG model well. For each construct, write a `valid_X` rule and an `invalid_X` rule, and combine them into an `X` rule. The parser always succeeds; the error reporting phase walks the parse tree and generates errors for `invalid_*` nodes.
3. **Shell mode integration** — pest's PEG model handles the mode-switching approach naturally. A top-level rule can try language keywords first and fall back to a shell command rule.
4. **Performance is manageable** — the main risk (exponential backtracking) is avoidable with careful grammar design. The ish grammar is not pathological (no deeply nested ambiguous alternatives).
5. **Tree-sitter can be added later** for IDE support, potentially auto-generated from the pest grammar.

**Refinements to the approach:**
- Structure the grammar in layers: lexer rules (keywords, operators, literals) → expression rules → statement rules → top-level rules (language statement vs. shell command).
- Use pest's `COMMENT` and `WHITESPACE` built-in rules.
- Write comprehensive test cases for each error-accepting rule to ensure error messages are accurate.
- Consider generating a tree-sitter grammar from the pest grammar for future IDE integration.

### Decisions

**Decision:** Parser technology — pest, hand-written recursive descent, tree-sitter, chumsky, or hybrid?
--> pest for now.  We may have to re-visit this.  But I'd rather start off with a parser that is auto-generated from a formal grammar and go from there.

**Decision:** Should a tree-sitter grammar be planned from the start for IDE support?
--> No, that is for a later phase.

---

## Implementation Phases

The syntax features should be implemented in the following priority order:

### Phase 1: Minimal Viable Language
- Comments (`//`, `#`)
- Primitive literals (numbers, strings, booleans, null)
- Variable declaration (`let`, `let mut`)
- Assignment
- Arithmetic and comparison operators
- `if`/`else`
- `while` loop
- Function declaration (`fn`) and invocation
- `return`
- Block scoping (`{ }`)
- Newline-terminated statements with optional semicolons

### Phase 2: Data Structures and Closures
- Object literals (`{ key: value }`)
- List literals (`[a, b, c]`)
- Property access (`obj.prop`)
- Index access (`list[i]`)
- Lambda / arrow functions (`(x) => expr`)
- Closures
- String interpolation (`f"...{expr}..."`)
- `for x in iter` loop

### Phase 3: Error Handling
- `throw`
- `try`/`catch`/`finally`
- `with` blocks
- `defer`
- Typed catch clauses

### Phase 4: Type System Surface
- Type annotations on variables and parameters
- Type alias (`type Name = ...`)
- Union types (`A | B`)
- Nominal types (`nominal type`)
- Object type declarations
- Tuple syntax

### Phase 5: Modules and Visibility
- `use` statements
- `mod` declarations
- `pub` / `pub(scope)` visibility
- Module path resolution

### Phase 6: Shell Integration
- Bare-word command invocation
- Pipes (`|`)
- Redirection (`>`, `>>`, `2>&1`)
- Command substitution (`$(...)`)
- Glob expansion
- Environment variable access (`$VAR`)
- Background execution (`&`)

### Phase 7: Assurance Ledger Surface
- `@standard[name]` annotations
- `@[entry(params)]` annotations
- `standard` definitions
- `entry type` definitions

### Phase 8: Advanced Features
- Pattern matching (`match`)
- `?` operator
- Generics (`<T>`)
- `async`/`await`
- Default and rest parameters

---

## Documentation Updates

The following documentation files will need updates as syntax is implemented:

- [docs/spec/syntax.md](../../spec/syntax.md) — **Major rewrite** replacing the placeholder with the full syntax specification.
- [docs/spec/types.md](../../spec/types.md) — Update syntax examples to match final decisions.
- [docs/spec/modules.md](../../spec/modules.md) — Update import/visibility syntax examples.
- [docs/spec/execution.md](../../spec/execution.md) — Add shell mode syntax details.
- [docs/spec/assurance-ledger.md](../../spec/assurance-ledger.md) — Verify annotation syntax consistency.
- [docs/user-guide/language-basics.md](../../user-guide/language-basics.md) — Rewrite with final syntax.
- [docs/user-guide/functions.md](../../user-guide/functions.md) — Update function syntax examples.
- [docs/user-guide/error-handling.md](../../user-guide/error-handling.md) — Verify error handling syntax consistency.
- [docs/user-guide/getting-started.md](../../user-guide/getting-started.md) — Update examples.
- [docs/architecture/ast.md](../../architecture/ast.md) — Update AST to match new syntax constructs.
- [docs/project/open-questions.md](../../project/open-questions.md) — Close resolved syntax questions.
- [GLOSSARY.md](../../../GLOSSARY.md) — Add any new terms (e.g., "bare-word invocation", "command substitution").

Remember to update `## Referenced by` sections in all modified files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-14-language-syntax.md` — narrative account of the syntax design process, the sources consulted, the key disagreements between Rust and TypeScript conventions, the shell mode design challenge, and the decisions made.
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
