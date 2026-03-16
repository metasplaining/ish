---
title: "Proposal: Shell Build II"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-15
depends-on: [docs/project/rfp/shell-build-ii.md, docs/project/proposals/shell-build.md, docs/project/proposals/shell-construction.md, docs/spec/syntax.md, docs/architecture/ast.md, GLOSSARY.md]
---

# Proposal: Shell Build II

*Generated from [shell-build-ii.md](../rfp/shell-build-ii.md) on 2026-03-15.*

*Follow-on to [shell-build.md](shell-build.md). Eliminates the tier distinction, removes the bracket-counting validator, and specifies the parser-matches-everything approach.*

---

## Summary of Decisions

| Topic | Decision |
|-------|----------|
| `{ EOI` ambiguity | Always unterminated block (object literals are not top-level) |
| Unterminated single-line strings | Grammar production; REPL treats as error, not incomplete |
| Unterminated block comments | Grammar production (needed for whole-file error messages too) |
| Tier distinction | Eliminated — every delimited construct gets a production |
| Bracket-counting validator | Eliminated — fundamentally broken (`let x = '{'`). Parser only. |
| `has_incomplete` location | `ish-ast` (available to all crates) |

---

## Feature 1: Complete Unterminated Production Inventory

Every delimited grammar rule gets an unterminated sibling. Organized by delimiter type.

### 1a. Brace-delimited (`{` ... `}`)

| # | Rule | Unterminated rule | REPL behavior |
|---|------|-------------------|---------------|
| 1 | `block` | `unterminated_block` | **Wait** — multi-line construct |
| 2 | `object_literal` | `unterminated_object_literal` | **Wait** — multi-line construct |
| 3 | `match_stmt` body | `unterminated_match` | **Wait** — multi-line construct |
| 4 | `entry_type_def` | `unterminated_entry_type_def` | **Wait** — multi-line construct |
| 5 | `object_type` | `unterminated_object_type` | **Wait** — multi-line construct |

### 1b. Bracket-delimited (`[` ... `]`)

| # | Rule | Unterminated rule | REPL behavior |
|---|------|-------------------|---------------|
| 6 | `list_literal` | `unterminated_list_literal` | **Wait** — multi-line construct |
| 7 | `standard_def` | `unterminated_standard_def` | **Wait** — multi-line construct |
| 8 | `standard_annotation` | `unterminated_standard_annotation` | **Wait** — multi-line annotations |
| 9 | `entry_annotation` | `unterminated_entry_annotation` | **Wait** — multi-line annotations |
| 10 | `index_access` | `unterminated_index_access` | **Error** — single-line construct |

### 1c. Parenthesis-delimited (`(` ... `)`)

| # | Rule | Unterminated rule | REPL behavior |
|---|------|-------------------|---------------|
| 11 | grouped expression `(expr)` | `unterminated_paren_expr` | **Wait** — could span lines |
| 12 | `call_args` | `unterminated_call_args` | **Wait** — multi-line function calls |
| 13 | `fn_decl` params | `unterminated_fn_params` | **Wait** — multi-line param lists |
| 14 | `lambda` params | `unterminated_lambda_params` | **Wait** — multi-line param lists |
| 15 | `with_block` resources | `unterminated_with_resources` | **Wait** — multi-line resources |
| 16 | `catch_clause` param | `unterminated_catch_param` | **Error** — single-line construct |
| 17 | `command_substitution` | `unterminated_command_substitution` | **Wait** — multi-line commands |
| 18 | `tuple_type` | `unterminated_tuple_type` | **Wait** — multi-line type |
| 19 | `function_type` params | `unterminated_function_type` | **Error** — single-line construct |

### 1d. String-delimited

| # | Rule | Unterminated rule | REPL behavior |
|---|------|-------------------|---------------|
| 20 | `string_literal` (`'...'`) | `unterminated_string_literal` | **Error** — single-line string |
| 21 | `interp_string` (`"..."`) | `unterminated_interp_string` | **Error** — single-line string |
| 22 | `triple_single_string` (`'''...'''`) | `unterminated_triple_single_string` | **Wait** — multi-line by definition |
| 23 | `triple_double_string` (`"""..."""`) | `unterminated_triple_double_string` | **Wait** — multi-line by definition |
| 24 | `char_literal` (`c'...'`) | `unterminated_char_literal` | **Error** — single character |
| 25 | `extended_double_string` (`~"..."~`) | `unterminated_extended_double_string` | **Error** — single-line form |
| 26 | `extended_single_string` (`~'...'~`) | `unterminated_extended_single_string` | **Error** — single-line form |
| 27 | `extended_triple_double_string` (`~"""..."""~`) | `unterminated_extended_triple_double_string` | **Wait** — multi-line |
| 28 | `extended_triple_single_string` (`~'''...'''~`) | `unterminated_extended_triple_single_string` | **Wait** — multi-line |
| 29 | `shell_quoted_string` (`"..."` in shell) | `unterminated_shell_quoted_string` | **Error** — single-line shell string |
| 30 | `shell_single_string` (`'...'` in shell) | `unterminated_shell_single_string` | **Error** — single-line shell string |

### 1e. Comment-delimited

| # | Rule | Unterminated rule | REPL behavior |
|---|------|-------------------|---------------|
| 31 | `block_comment` (`/*...*/`) | `unterminated_block_comment` | **Wait** — multi-line comment |

### 1f. Angle-bracket-delimited

| # | Rule | Unterminated rule | REPL behavior |
|---|------|-------------------|---------------|
| 32 | `generic_params` (`<T, U>`) | `unterminated_generic_params` | **Error** — single-line construct |
| 33 | `generic_type` (`Type<T>`) | `unterminated_generic_type` | **Error** — single-line construct |

**Total: 33 unterminated productions.**

--> The following productions that are defined as **Error** should in fact be **Wait**.  It may be unusual for them to span lines, but it is not an error:
- index_access
- catch_clause
- function_type
- generic_params
- generic_type

---

## Feature 2: REPL Completion Categories

Each `IncompleteKind` is classified into two categories that the REPL uses to decide behavior:

### Wait (continue reading)

The unterminated construct is expected to span multiple lines. REPL shows `...> ` and waits.

```
Block, ObjectLiteral, Match, EntryTypeDef, ObjectType,
ListLiteral, StandardDef, StandardAnnotation, EntryAnnotation,
ParenExpr, CallArgs, FnParams, LambdaParams, WithResources,
CommandSubstitution, TupleType,
TripleSingleString, TripleDoubleString,
ExtendedTripleDoubleString, ExtendedTripleSingleString,
BlockComment
```

### Error (display immediately)

The unterminated construct would not normally span lines. REPL displays an error message.

```
IndexAccess, CatchParam, FunctionType,
StringLiteral, InterpString, CharLiteral,
ExtendedDoubleString, ExtendedSingleString,
ShellQuotedString, ShellSingleString,
GenericParams, GenericType
```

### IncompleteKind enum (expanded)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncompleteKind {
    // Brace-delimited
    Block,
    ObjectLiteral,
    Match,
    EntryTypeDef,
    ObjectType,
    // Bracket-delimited
    ListLiteral,
    StandardDef,
    StandardAnnotation,
    EntryAnnotation,
    IndexAccess,
    // Paren-delimited
    ParenExpr,
    CallArgs,
    FnParams,
    LambdaParams,
    WithResources,
    CatchParam,
    CommandSubstitution,
    TupleType,
    FunctionType,
    // String-delimited
    StringLiteral,
    InterpString,
    TripleSingleString,
    TripleDoubleString,
    CharLiteral,
    ExtendedDoubleString,
    ExtendedSingleString,
    ExtendedTripleDoubleString,
    ExtendedTripleSingleString,
    ShellQuotedString,
    ShellSingleString,
    // Comment-delimited
    BlockComment,
    // Angle-bracket-delimited
    GenericParams,
    GenericType,
}
```

Add a method that expresses the REPL categorization:

```rust
impl IncompleteKind {
    /// Returns true if this kind of incomplete input should cause the REPL
    /// to wait for more input (multiline continuation). Returns false if
    /// it should be reported as an error immediately.
    pub fn is_continuable(&self) -> bool {
        match self {
            // Single-line constructs — error, not continuation
            Self::IndexAccess
            | Self::CatchParam
            | Self::FunctionType
            | Self::StringLiteral
            | Self::InterpString
            | Self::CharLiteral
            | Self::ExtendedDoubleString
            | Self::ExtendedSingleString
            | Self::ShellQuotedString
            | Self::ShellSingleString
            | Self::GenericParams
            | Self::GenericType => false,

            // Everything else — wait for more input
            _ => true,
        }
    }
}
```

---

## Feature 3: REPL Without Bracket Counting

The bracket-counting `IshValidator` from shell-construction Feature 3 is eliminated. The parser is the sole authority.

### Reedline Validator implementation

Reedline's `Validator` trait has a single method: `fn validate(&self, line: &str) -> ValidationResult`. It runs on every Enter keypress. The implementation now invokes the parser:

```rust
// ish-shell/src/validate.rs

use reedline::{ValidationResult, Validator};

pub struct IshValidator;

impl Validator for IshValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        match ish_parser::parse(line) {
            Ok(program) => {
                if ish_ast::has_incomplete_continuable(&program) {
                    ValidationResult::Incomplete
                } else {
                    ValidationResult::Complete
                }
            }
            Err(_) => {
                // If the parser still returns errors for some inputs,
                // treat them as complete (submit and show the error).
                ValidationResult::Complete
            }
        }
    }
}
```

### Issues to Watch Out For

**Performance.** The parser now runs on every Enter keypress, not just on final submit. For typical REPL input (1–20 lines), pest parsing is sub-millisecond. This is not a concern.

For pathological input (thousands of lines pasted), the parse could take noticeable time. Mitigation: cap the validator at a reasonable input size (e.g., 100KB) and return `Complete` for anything larger, deferring full parsing to the submit path.

**Keystroke feel.** With bracket counting, insertion of a newline on Enter was instantaneous. With parser invocation, there's a parse on each Enter. Users will not notice the difference for normal input sizes. If latency becomes measurable, the parser can be called in a background thread with a timeout — but this is premature optimization.

### Critical Analysis

**Alternative: Keep the bracket-counting validator as a pre-filter.**
- Pro: Handles the common "unbalanced brace" case without parsing.
- Con: Fundamentally broken. `let x = '{'` produces a false incomplete. The validator would need to understand string quoting, comments, and escape sequences — at which point it's duplicating the parser. The user explicitly rejected this.

**Alternative: Parse on every keystroke.**
- Pro: Always accurate.
- Con: Wasteful. Only Enter keypress matters for the Validator.

**Chosen approach: Parse on Enter only (via Validator trait).**
- Pro: Accurate. Simple. One code path.
- Con: Slightly slower than bracket counting for the common case (parsing vs. character scan). Not measurably different.

### Decisions

**Decision:** Should the validator cap input size and skip parsing for very large pastes?
--> No.  We should prioritize correctness over performance.

---

## Feature 4: `has_incomplete` API in `ish-ast`

### Public API

```rust
// ish-ast/src/lib.rs (or a new ish-ast/src/analysis.rs module)

impl Program {
    /// Returns true if the AST contains any Incomplete node whose kind
    /// is continuable (multi-line construct).
    pub fn has_incomplete_continuable(&self) -> bool {
        self.statements.iter().any(|s| s.has_incomplete_continuable())
    }

    /// Returns true if the AST contains any Incomplete node (continuable or error).
    pub fn has_any_incomplete(&self) -> bool {
        self.statements.iter().any(|s| s.has_any_incomplete())
    }
}

impl Statement {
    pub fn has_incomplete_continuable(&self) -> bool {
        match self {
            Statement::Incomplete { kind, .. } => kind.is_continuable(),
            // Recurse into all compound variants:
            Statement::Block { statements } =>
                statements.iter().any(|s| s.has_incomplete_continuable()),
            Statement::If { condition, then_block, else_block } =>
                condition.has_incomplete_continuable()
                    || then_block.has_incomplete_continuable()
                    || else_block.as_ref().map_or(false, |b| b.has_incomplete_continuable()),
            Statement::While { condition, body } =>
                condition.has_incomplete_continuable()
                    || body.has_incomplete_continuable(),
            Statement::ForEach { iterable, body, .. } =>
                iterable.has_incomplete_continuable()
                    || body.has_incomplete_continuable(),
            Statement::FunctionDecl { body, .. } =>
                body.has_incomplete_continuable(),
            Statement::TryCatch { body, catches, finally } =>
                body.has_incomplete_continuable()
                    || catches.iter().any(|c| c.body.has_incomplete_continuable())
                    || finally.as_ref().map_or(false, |b| b.has_incomplete_continuable()),
            Statement::WithBlock { resources, body } =>
                resources.iter().any(|(_, e)| e.has_incomplete_continuable())
                    || body.has_incomplete_continuable(),
            Statement::Defer { body } =>
                body.has_incomplete_continuable(),
            Statement::ExpressionStmt(expr) =>
                expr.has_incomplete_continuable(),
            Statement::VariableDecl { value, .. } =>
                value.has_incomplete_continuable(),
            Statement::Assignment { value, .. } =>
                value.has_incomplete_continuable(),
            Statement::Return { value } =>
                value.as_ref().map_or(false, |e| e.has_incomplete_continuable()),
            Statement::Throw { value } =>
                value.has_incomplete_continuable(),
            Statement::Match { subject, arms } =>
                subject.has_incomplete_continuable()
                    || arms.iter().any(|a| a.body.has_incomplete_continuable()),
            Statement::ShellCommand { .. } => false, // leaf
            Statement::Annotated { inner, .. } =>
                inner.has_incomplete_continuable(),
            Statement::ModDecl { body, .. } =>
                body.as_ref().map_or(false, |b| b.has_incomplete_continuable()),
            // Leaf statements with no sub-expressions/statements:
            Statement::TypeAlias { .. }
            | Statement::Use { .. }
            | Statement::StandardDef { .. }
            | Statement::EntryTypeDef { .. } => false,
        }
    }

    pub fn has_any_incomplete(&self) -> bool {
        // Same structure, but returns true for ALL IncompleteKind variants.
        match self {
            Statement::Incomplete { .. } => true,
            // ... same recursion as above ...
            _ => false,
        }
    }
}

impl Expression {
    pub fn has_incomplete_continuable(&self) -> bool {
        match self {
            Expression::Incomplete { kind, .. } => kind.is_continuable(),
            Expression::BinaryOp { left, right, .. } =>
                left.has_incomplete_continuable() || right.has_incomplete_continuable(),
            Expression::UnaryOp { operand, .. } =>
                operand.has_incomplete_continuable(),
            Expression::FunctionCall { callee, args } =>
                callee.has_incomplete_continuable()
                    || args.iter().any(|a| a.has_incomplete_continuable()),
            Expression::ObjectLiteral(pairs) =>
                pairs.iter().any(|(_, e)| e.has_incomplete_continuable()),
            Expression::ListLiteral(items) =>
                items.iter().any(|e| e.has_incomplete_continuable()),
            Expression::PropertyAccess { object, .. } =>
                object.has_incomplete_continuable(),
            Expression::IndexAccess { object, index } =>
                object.has_incomplete_continuable() || index.has_incomplete_continuable(),
            Expression::Lambda { body, .. } =>
                body.has_incomplete_continuable(),
            Expression::StringInterpolation(parts) =>
                parts.iter().any(|p| match p {
                    StringPart::Expr(e) => e.has_incomplete_continuable(),
                    _ => false,
                }),
            Expression::CommandSubstitution(stmt) =>
                stmt.has_incomplete_continuable(),
            // Leaves: no sub-expressions
            Expression::Literal(_)
            | Expression::Identifier(_)
            | Expression::EnvVar(_) => false,
        }
    }

    pub fn has_any_incomplete(&self) -> bool {
        match self {
            Expression::Incomplete { .. } => true,
            // ... same recursion ...
            _ => false,
        }
    }
}
```

### Issues to Watch Out For

**Exhaustiveness.** Every `Statement` and `Expression` variant must be covered in both `has_incomplete_continuable` and `has_any_incomplete`. Adding a new variant to either enum requires updating both methods. Consider using a `#[deny(non_exhaustive_patterns)]` or relying on the default `_ => false` arm with a comment explaining the obligation.

**Derive vs. manual.** A derive macro could auto-generate the recursion. But introducing proc macros for this single use case is over-engineering. The manual implementation is readable and ensures each variant is handled intentionally.

### Decisions

**Decision:** Should `has_any_incomplete` and `has_incomplete_continuable` be inherent methods on `Program`/`Statement`/`Expression`, or free functions in a separate `analysis` module?
--> Inherent methods.

---

## Feature 5: Parser API Evolution

### The question

The parser currently returns `Result<Program, Vec<ParseError>>`. With the "parser matches everything" philosophy, the parser should ideally always return `Ok(Program)`. Does the API change?

### Proposed approach: Gradual evolution

The API signature stays `Result<Program, Vec<ParseError>>` for now. The unterminated productions make the parser match *more* input, but there will always be edge cases where pest itself fails to match (e.g., truly malformed byte sequences, or constructs not yet covered by unterminated rules). As coverage increases, `Err` becomes rarer until eventually the return type could simplify to just `Program`.

The practical impact:
- **Today:** `parse()` returns `Err` for some badly formed input.
- **After this proposal:** `parse()` returns `Err` for fewer inputs. Most "bad" input now produces an AST with `Incomplete` or (future) `Error` nodes.
- **Eventually:** When all error-recovery productions are in place, `parse()` always returns `Ok`. At that point, change the API to return `Program` directly.

The REPL handles both cases: `Ok(program)` → check for incomplete nodes; `Err(errors)` → display errors. No behavioral change needed in the REPL when the parser's `Err` rate decreases.

### Issues to Watch Out For

**Beyond unterminated: other error productions.** The "parser matches everything" philosophy implies more than just unterminated rules. Future work should add productions for common syntax errors — e.g., `let = 5` (missing variable name), `fn {}` (missing function name), `if {}` (missing condition). This proposal only covers unterminated (missing close delimiter) productions. General error-recovery productions are a separate effort.

### Decisions

**Decision:** Should the proposal include a concrete plan for non-delimiter error productions (e.g., `let = 5`), or is that out of scope?
--> That is out of scope.

---

## Feature 6: Complete Grammar Additions

Here are all 33 unterminated productions. Grouped by the section of the grammar they modify.

### 6a. Block and block-like constructs

```pest
unterminated_block = {
    "{" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)* ~ stmt_sep?)? ~ EOI
}

unterminated_match = {
    "match" ~ expression ~ "{" ~ NEWLINE* ~
    (match_arm ~ (("," | NEWLINE) ~ NEWLINE* ~ match_arm)* ~
     ("," | NEWLINE)? ~ NEWLINE*)? ~ EOI
}

unterminated_entry_type_def = {
    "entry" ~ "type" ~ identifier ~ "{" ~ NEWLINE* ~
    (entry_type_field ~ ("," ~ NEWLINE* ~ entry_type_field)* ~
     ","? ~ NEWLINE*)? ~ EOI
}
```

Modify rules that reference `block`:

```pest
// All these change from `block` to `block_or_unterminated`:
fn_decl = {
    pub_modifier? ~ "fn" ~ identifier ~ generic_params? ~
    "(" ~ param_list? ~ ")" ~ ("->" ~ type_annotation)? ~ block_or_unterminated
}
if_stmt = {
    "if" ~ expression ~ block_or_unterminated ~
    (NEWLINE* ~ "else" ~ (if_stmt | block_or_unterminated))?
}
while_stmt = { "while" ~ expression ~ block_or_unterminated }
for_stmt = { "for" ~ identifier ~ "in" ~ expression ~ block_or_unterminated }
try_catch = { "try" ~ block_or_unterminated ~ catch_clause+ ~ ("finally" ~ block_or_unterminated)? }
catch_clause = { "catch" ~ "(" ~ identifier ~ (":" ~ type_annotation)? ~ ")" ~ block_or_unterminated }
with_block = { "with" ~ "(" ~ resource_binding ~ ("," ~ resource_binding)* ~ ","? ~ ")" ~ block_or_unterminated }
defer_stmt = { "defer" ~ (block_or_unterminated | expression_stmt) }
mod_stmt = { pub_modifier? ~ "mod" ~ identifier ~ block_or_unterminated? }
lambda = { "(" ~ param_list? ~ ")" ~ "=>" ~ (block_or_unterminated | expression) }

// The silent wrapper:
block_or_unterminated = _{ block | unterminated_block }
```

Similarly for match and entry_type_def — replace inline in `statement`:

```pest
statement = _{
    // ... existing alternatives ...
    match_stmt | unterminated_match |
    entry_type_def | unterminated_entry_type_def |
    // ...
}
```

### 6b. Expressions (list, object, parens)

```pest
unterminated_object_literal = {
    "{" ~ NEWLINE* ~ (object_pair ~ (object_sep ~ object_pair)* ~ object_sep?)? ~
    NEWLINE* ~ EOI
}

unterminated_list_literal = {
    "[" ~ NEWLINE* ~ (expression ~ ("," ~ NEWLINE* ~ expression)* ~
    ","? ~ NEWLINE*)? ~ EOI
}

unterminated_paren_expr = {
    "(" ~ expression? ~ EOI
}

unterminated_call_args = {
    "(" ~ arg_list? ~ EOI
}

unterminated_index_access = {
    "[" ~ expression? ~ EOI
}

unterminated_command_substitution = {
    "$(" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)*)? ~ EOI
}
```

Update `primary`:

```pest
primary = {
    lambda |
    "(" ~ expression ~ ")" | unterminated_paren_expr |
    object_literal | unterminated_object_literal |
    list_literal | unterminated_list_literal |
    // ... all literals ...
    triple_double_string | unterminated_triple_double_string |
    triple_single_string | unterminated_triple_single_string |
    extended_triple_double_string | unterminated_extended_triple_double_string |
    extended_triple_single_string | unterminated_extended_triple_single_string |
    extended_double_string | unterminated_extended_double_string |
    extended_single_string | unterminated_extended_single_string |
    char_literal | unterminated_char_literal |
    interp_string | unterminated_interp_string |
    string_literal | unterminated_string_literal |
    command_substitution | unterminated_command_substitution |
    env_var |
    identifier
}
```

Update `postfix_op` to include unterminated alternatives:

```pest
postfix_op = _{
    call_args | unterminated_call_args |
    dot_access |
    index_access | unterminated_index_access |
    try_op
}
```

### 6c. String literals

```pest
// Single-line strings — unterminated means EOI before close quote

unterminated_string_literal = ${
    "'" ~ single_string_inner ~ EOI
}

unterminated_interp_string = ${
    "\"" ~ interp_string_part* ~ EOI
}

unterminated_char_literal = ${
    "c'" ~ char_literal_inner? ~ EOI
}

// Triple-quoted strings — unterminated means EOI before close triple-quote

unterminated_triple_double_string = ${
    "\"\"\"" ~ NEWLINE? ~ triple_double_part* ~ EOI
}

unterminated_triple_single_string = ${
    "'''" ~ NEWLINE? ~ triple_single_inner ~ EOI
}

// Extended strings

unterminated_extended_double_string = ${
    "~\"" ~ extended_double_inner ~ EOI
}

unterminated_extended_single_string = ${
    "~'" ~ extended_single_inner ~ EOI
}

unterminated_extended_triple_double_string = ${
    "~\"\"\"" ~ NEWLINE? ~ extended_triple_double_inner ~ EOI
}

unterminated_extended_triple_single_string = ${
    "~'''" ~ NEWLINE? ~ extended_triple_single_inner ~ EOI
}

// Shell strings

unterminated_shell_quoted_string = ${
    "\"" ~ shell_quoted_inner ~ EOI
}

unterminated_shell_single_string = ${
    "'" ~ shell_single_inner ~ EOI
}
```

Update `shell_word`:

```pest
shell_word = {
    command_substitution | unterminated_command_substitution |
    env_var |
    shell_quoted_string | unterminated_shell_quoted_string |
    shell_single_string | unterminated_shell_single_string |
    shell_bare_word
}
```

### 6d. Parenthesized sub-constructs

```pest
unterminated_fn_params = {
    "(" ~ param_list? ~ EOI
}

unterminated_lambda_params = {
    "(" ~ param_list? ~ EOI
}

unterminated_with_resources = {
    "(" ~ resource_binding ~ ("," ~ resource_binding)* ~ ","? ~ EOI
}

unterminated_catch_param = {
    "(" ~ identifier? ~ (":" ~ type_annotation)? ~ EOI
}
```

These integrate into their parent rules:

```pest
fn_decl = {
    pub_modifier? ~ "fn" ~ identifier ~ generic_params_or_unterminated? ~
    ("(" ~ param_list? ~ ")" | unterminated_fn_params) ~
    ("->" ~ type_annotation)? ~ block_or_unterminated
}

lambda = {
    ("(" ~ param_list? ~ ")" | unterminated_lambda_params) ~
    "=>" ~ (block_or_unterminated | expression)
}

with_block = {
    "with" ~ ("(" ~ resource_binding ~ ("," ~ resource_binding)* ~ ","? ~ ")" | unterminated_with_resources) ~
    block_or_unterminated
}

catch_clause = {
    "catch" ~ ("(" ~ identifier ~ (":" ~ type_annotation)? ~ ")" | unterminated_catch_param) ~
    block_or_unterminated
}
```

### 6e. Type annotations

```pest
unterminated_object_type = {
    "{" ~ NEWLINE* ~ (object_type_field ~ ("," ~ NEWLINE* ~ object_type_field)* ~ ","?)? ~
    NEWLINE* ~ EOI
}

unterminated_tuple_type = {
    "(" ~ type_annotation ~ ("," ~ type_annotation)* ~ ","? ~ EOI
}

unterminated_function_type = {
    "fn" ~ "(" ~ type_list? ~ EOI
}
```

Update `primary_type`:

```pest
primary_type = {
    list_type |
    object_type | unterminated_object_type |
    function_type | unterminated_function_type |
    tuple_type | unterminated_tuple_type |
    generic_type | unterminated_generic_type |
    simple_type
}
```

### 6f. Angle brackets (generics)

```pest
unterminated_generic_params = {
    "<" ~ identifier ~ ("," ~ identifier)* ~ EOI
}

unterminated_generic_type = {
    identifier ~ "<" ~ type_annotation ~ ("," ~ type_annotation)* ~ EOI
}

generic_params_or_unterminated = _{ generic_params | unterminated_generic_params }
```

### 6g. Annotations

```pest
unterminated_standard_annotation = {
    "@standard[" ~ annotation_args? ~ EOI
}

unterminated_entry_annotation = {
    "@[" ~ (entry_item ~ ("," ~ entry_item)* ~ ","?)? ~ EOI
}

unterminated_standard_def = {
    "standard" ~ identifier ~ ("extends" ~ identifier)? ~
    "[" ~ NEWLINE* ~ (feature_spec ~ ("," ~ NEWLINE* ~ feature_spec)* ~ ","? ~ NEWLINE*)? ~ EOI
}
```

### 6h. Block comments

`unterminated_block_comment` cannot be part of the implicit `COMMENT` rule (pest discards those silently). Instead, make it a statement-level rule:

```pest
unterminated_block_comment = @{
    "/*" ~ (block_comment | (!"*/" ~ ANY))* ~ EOI
}
```

Add to `statement`:

```pest
statement = _{
    unterminated_block_comment |   // before other alternatives
    // ... existing ...
}
```

The AST builder maps it to `Statement::Incomplete { kind: IncompleteKind::BlockComment, partial_statements: vec![] }`.

---

## Feature 7: AST Builder — Complete Additions

### New builder functions needed

| Grammar rule | Builder function | Returns |
|-------------|-----------------|---------|
| `unterminated_block` | `build_unterminated_block` | `Statement::Incomplete` |
| `unterminated_match` | `build_unterminated_match` | `Statement::Incomplete` |
| `unterminated_entry_type_def` | `build_unterminated_entry_type_def` | `Statement::Incomplete` |
| `unterminated_block_comment` | `build_unterminated_block_comment` | `Statement::Incomplete` |
| `unterminated_list_literal` | `build_unterminated_list` | `Expression::Incomplete` |
| `unterminated_object_literal` | `build_unterminated_object` | `Expression::Incomplete` |
| `unterminated_paren_expr` | `build_unterminated_paren` | `Expression::Incomplete` |
| `unterminated_call_args` | `build_unterminated_call` | `Expression::Incomplete` |
| `unterminated_index_access` | `build_unterminated_index` | `Expression::Incomplete` |
| `unterminated_command_substitution` | `build_unterminated_cmd_sub` | `Expression::Incomplete` |
| `unterminated_string_literal` | `build_unterminated_string` | `Expression::Incomplete` |
| `unterminated_interp_string` | `build_unterminated_interp` | `Expression::Incomplete` |
| `unterminated_char_literal` | `build_unterminated_char` | `Expression::Incomplete` |
| `unterminated_triple_double_string` | `build_unterminated_triple_double` | `Expression::Incomplete` |
| `unterminated_triple_single_string` | `build_unterminated_triple_single` | `Expression::Incomplete` |
| `unterminated_extended_double_string` | `build_unterminated_ext_double` | `Expression::Incomplete` |
| `unterminated_extended_single_string` | `build_unterminated_ext_single` | `Expression::Incomplete` |
| `unterminated_extended_triple_double_string` | `build_unterminated_ext_triple_double` | `Expression::Incomplete` |
| `unterminated_extended_triple_single_string` | `build_unterminated_ext_triple_single` | `Expression::Incomplete` |
| `unterminated_shell_quoted_string` | `build_unterminated_shell_quoted` | `Expression::Incomplete` |
| `unterminated_shell_single_string` | `build_unterminated_shell_single` | `Expression::Incomplete` |
| `unterminated_fn_params` | handled inline in `build_fn_decl` | sets flag |
| `unterminated_lambda_params` | handled inline in `build_lambda` | sets flag |
| `unterminated_with_resources` | handled inline in `build_with_block` | sets flag |
| `unterminated_catch_param` | handled inline in `build_try_catch` | sets flag |
| `unterminated_standard_annotation` | `build_unterminated_std_annotation` | `Statement::Incomplete` |
| `unterminated_entry_annotation` | `build_unterminated_entry_annotation` | `Statement::Incomplete` |
| `unterminated_standard_def` | `build_unterminated_standard_def` | `Statement::Incomplete` |
| `unterminated_object_type` | handled inline in type builder | `Expression::Incomplete` |
| `unterminated_tuple_type` | handled inline in type builder | `Expression::Incomplete` |
| `unterminated_function_type` | handled inline in type builder | `Expression::Incomplete` |
| `unterminated_generic_params` | handled inline in `build_fn_decl` | sets flag |
| `unterminated_generic_type` | handled inline in type builder | `Expression::Incomplete` |

Most builders share a pattern: iterate inner pairs, collect until `EOI`, wrap in `Incomplete`. The sub-construct builders (fn_params, lambda_params, with_resources, catch_param, generic_params, type variants) are handled inline in their parent statement builders.

---

## Feature 8: Test Plan

### Test structure

Add a new test file `proto/ish-parser/tests/unterminated.rs` containing:

```rust
// For each unterminated production:

#[test]
fn unterminated_block_empty() {
    // Just an open brace
    let result = ish_parser::parse("{").unwrap();
    assert!(result.has_incomplete_continuable());
}

#[test]
fn unterminated_block_with_content() {
    // Open brace with statements
    let result = ish_parser::parse("{ let x = 5").unwrap();
    assert!(result.has_incomplete_continuable());
}

#[test]
fn unterminated_list_literal() {
    let result = ish_parser::parse("let x = [1, 2, 3").unwrap();
    assert!(result.has_incomplete_continuable());
}

#[test]
fn unterminated_string_is_error() {
    // Unterminated single-line string: NOT continuable
    let result = ish_parser::parse("let x = \"hello").unwrap();
    assert!(result.has_any_incomplete());
    assert!(!result.has_incomplete_continuable());
}

#[test]
fn unterminated_triple_string_is_continuable() {
    let result = ish_parser::parse("let x = \"\"\"hello").unwrap();
    assert!(result.has_incomplete_continuable());
}

#[test]
fn complete_input_has_no_incomplete() {
    let result = ish_parser::parse("let x = 5").unwrap();
    assert!(!result.has_any_incomplete());
}

// ... one test per production, plus nested cases
```

### Nested incomplete tests

```rust
#[test]
fn unterminated_list_inside_block() {
    let result = ish_parser::parse("{ let x = [1, 2").unwrap();
    assert!(result.has_incomplete_continuable());
}

#[test]
fn unterminated_string_inside_list() {
    let result = ish_parser::parse("let x = [\"hello").unwrap();
    assert!(result.has_any_incomplete());
    // The string is not continuable, but the expression still has incomplete
}
```

---

## Implementation Sequence

```
1. AST: Add IncompleteKind (33 variants), Statement::Incomplete, Expression::Incomplete
   ↓
2. AST: Add is_continuable(), has_incomplete_continuable(), has_any_incomplete()
   ↓
3. Grammar: Add all 33 unterminated_* rules
   ↓
4. Grammar: Integrate into existing rules (block_or_unterminated, primary, postfix_op, etc.)
   ↓
5. AST builder: Add builder functions, update dispatchers
   ↓
6. Tests: unterminated.rs with tests for each production
   ↓
7. REPL validator: Replace bracket counter with parser-based IshValidator
   ↓
8. Grammar: $? extension (from shell-build)
```

Steps 1 and 2 can proceed in parallel with 3 and 4. Step 8 is independent.

---

## Files Affected

| File | Changes |
|------|---------|
| `proto/ish-ast/src/lib.rs` | `IncompleteKind` (33 variants), `Statement::Incomplete`, `Expression::Incomplete`, `is_continuable()`, `has_incomplete_continuable()`, `has_any_incomplete()` |
| `proto/ish-parser/src/ish.pest` | 33 `unterminated_*` rules, `block_or_unterminated` wrapper, modified `primary`, `postfix_op`, `statement`, `shell_word`, `primary_type`, `fn_decl`, `if_stmt`, `while_stmt`, `for_stmt`, `try_catch`, `catch_clause`, `with_block`, `defer_stmt`, `mod_stmt`, `lambda`, `match_stmt`, `entry_type_def`, `standard_def`, `annotation` |
| `proto/ish-parser/src/ast_builder.rs` | ~25 `build_unterminated_*` functions, updated dispatchers |
| `proto/ish-parser/tests/unterminated.rs` | ~40+ tests |
| `proto/ish-shell/src/validate.rs` | Replace bracket counter with parser-based `IshValidator` |

---

## Deferred Items

| Feature | Rationale |
|---------|-----------|
| Non-delimiter error productions (`let = 5`, `fn {}`, `if {}`) | Separate effort; this proposal covers only unterminated delimiters |
| Change `parse()` return type to `Program` | Wait until error-recovery coverage is complete |
| VM error messages for `Incomplete` nodes | Separate effort; VM currently skips `Incomplete` |

---

## Documentation Updates

| File | Update |
|------|--------|
| `docs/architecture/ast.md` | Document all 33 `IncompleteKind` variants and the `has_incomplete_*` API |
| `docs/architecture/shell.md` | Parser-only validation (no bracket counting). REPL control flow. |
| `docs/spec/syntax.md` | Parser-matches-everything philosophy. Unterminated productions. |
| `proto/ARCHITECTURE.md` | Update parser description: never-fail goal |

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-shell-build-ii.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
