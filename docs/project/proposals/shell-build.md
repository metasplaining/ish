---
title: "Proposal: Shell Build"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-15
depends-on: [docs/project/rfp/shell-build.md, docs/project/proposals/shell-construction.md, docs/spec/syntax.md, docs/architecture/ast.md, GLOSSARY.md]
---

# Proposal: Shell Build

*Generated from [shell-build.md](../rfp/shell-build.md) on 2026-03-15.*

*Follow-on to [shell-construction.md](shell-construction.md), focusing on incomplete-input detection via grammar productions. Incorporates all previous decisions.*

---

## Summary

The previous proposal planned to detect incomplete REPL input via a `parse_chunk()` heuristic (bracket-counting after a failed parse). That approach was rejected. Instead:

1. The parser's existing API (`parse(input) -> Result<Program, Vec<ParseError>>`) is **unchanged**.
2. The pest grammar gains **unterminated productions** — rules that accept `EOI` where a closing delimiter is expected.
3. The AST gains an **`Incomplete` variant** on both `Statement` and `Expression` to represent these partial constructs.
4. The REPL inspects the parsed AST: if any `Incomplete` node is present, the input needs more lines; otherwise it is complete (or erroneous, and the error is in the AST as well).

The parser never fails — it always produces an AST. Bad input matches error/unterminated productions rather than causing pest to reject the input.

---

## Feature 1: Inventory of Delimited Constructs

Every grammar rule that opens a delimiter and expects a corresponding close is a candidate for an unterminated production. Here is the complete inventory:

### Brace-delimited (`{` ... `}`)

| Rule | Close | Used by |
|------|-------|---------|
| `block` | `}` | `fn_decl`, `if_stmt`, `while_stmt`, `for_stmt`, `try_catch`, `catch_clause`, `with_block`, `defer_stmt`, `lambda`, `mod_stmt` |
| `object_literal` | `}` | `primary` |
| `match_stmt` | `}` | `statement` |
| `entry_type_def` | `}` | `statement` |
| `object_type` | `}` | `type_annotation` |

### Bracket-delimited (`[` ... `]`)

| Rule | Close | Used by |
|------|-------|---------|
| `list_literal` | `]` | `primary` |
| `standard_def` | `]` | `statement` (`standard name [features]`) |
| `standard_annotation` | `]` | `annotation` (`@standard[...]`) |
| `entry_annotation` | `]` | `annotation` (`@[...]`) |
| `index_access` | `]` | `postfix_op` |

### Parenthesis-delimited (`(` ... `)`)

| Rule | Close | Used by |
|------|-------|---------|
| grouped expression | `)` | `primary` (`( expression )`) |
| `call_args` | `)` | `postfix_op` |
| `fn_decl` params | `)` | `fn_decl` (`fn name(params)`) |
| `lambda` params | `)` | `lambda` (`(params) => body`) |
| `with_block` resources | `)` | `with_block` (`with(bindings)`) |
| `catch_clause` param | `)` | `catch_clause` (`catch(err)`) |
| `command_substitution` | `)` | `shell_word` (`$(...)`) |
| `tuple_type` | `)` | `type_annotation` |
| `function_type` | `)` | `type_annotation` (`fn(...) -> T`) |

### String-delimited

| Rule | Open | Close | Interpolation? |
|------|------|-------|---------------|
| `string_literal` | `'` | `'` | No |
| `interp_string` | `"` | `"` | Yes (`{expr}`, `$VAR`) |
| `triple_single_string` | `'''` | `'''` | No |
| `triple_double_string` | `"""` | `"""` | Yes |
| `char_literal` | `c'` | `'` | No |
| `extended_double_string` | `~"` | `"~` | No |
| `extended_single_string` | `~'` | `'~` | No |
| `extended_triple_double_string` | `~"""` | `"""~` | No |
| `extended_triple_single_string` | `~'''` | `'''~` | No |
| `shell_quoted_string` | `"` | `"` (no newline) | No |
| `shell_single_string` | `'` | `'` (no newline) | No |

### Comment-delimited

| Rule | Open | Close |
|------|------|-------|
| `block_comment` | `/*` | `*/` |

---

## Feature 2: Grammar Productions — Unterminated Rules

### Design principle

For each delimited construct, add a sibling rule that matches the same opening but accepts `EOI` instead of the closing delimiter. PEG ordered choice (`|`) ensures the complete rule is tried first. Only if the close is missing does the unterminated rule match.

The naming convention: `unterminated_` prefix.

### Priority tiers

Not all constructs need unterminated productions immediately. Tier 1 covers constructs likely to span multiple REPL lines. Tier 2 covers constructs that are theoretically possible but unlikely to be typed incrementally.

**Tier 1 — Implement now:**

| Construct | Rationale |
|-----------|-----------|
| Block (`{ }`) | Multi-line functions, if/while/for bodies |
| List literal (`[ ]`) | Multi-line list construction |
| Object literal (`{ }`) | Multi-line objects |
| Triple-quoted strings (`"""`, `'''`) | Multi-line strings by definition |
| Block comment (`/* */`) | Multi-line comments |
| Match body (`match expr { }`) | Multi-line match arms |
| Parenthesized expression | Multi-line chained expressions |
| Call args | Multi-line function calls |
| Command substitution (`$(...)`) | Multi-line embedded commands |

**Tier 2 — Defer:**

| Construct | Rationale |
|-----------|-----------|
| Single-line strings (`'...'`, `"..."`) | Shell strings explicitly reject `\n`; language strings on a single line are obviously wrong if unterminated |
| Extended strings (`~"..."~`, etc.) | Uncommon in REPL use |
| Index access (`[expr]`) | One-line construct |
| Annotation brackets (`@standard[...]`, `@[...]`) | Uncommon in REPL use |
| Standard def brackets | Uncommon in REPL use |
| Type annotations (`{...}`, `(...)` in types) | Rarely typed in REPL |
| `catch_clause` param, `with_block` resources | Sub-parts of larger blocks — the unterminated_block catches these |
| `fn_decl` param list, `lambda` param list | Generally one-line; unterminated_block catches the body |

### Concrete grammar additions (Tier 1)

```pest
// --- Unterminated productions (incomplete input) ---
// These match when input ends (EOI) before the closing delimiter.
// PEG ordered choice ensures the complete rule is always tried first.

unterminated_block = {
    "{" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)* ~ stmt_sep?)? ~ EOI
}

unterminated_list_literal = {
    "[" ~ NEWLINE* ~ (expression ~ ("," ~ NEWLINE* ~ expression)* ~ ","? ~ NEWLINE*)? ~ EOI
}

unterminated_object_literal = {
    "{" ~ NEWLINE* ~ (object_pair ~ (object_sep ~ object_pair)* ~ object_sep?)? ~ NEWLINE* ~ EOI
}

unterminated_triple_double_string = ${
    "\"\"\"" ~ NEWLINE? ~ triple_double_part* ~ EOI
}

unterminated_triple_single_string = ${
    "'''" ~ NEWLINE? ~ triple_single_inner ~ EOI
}

unterminated_block_comment = @{
    "/*" ~ (block_comment | (!"*/" ~ ANY))* ~ EOI
}

unterminated_match = {
    "match" ~ expression ~ "{" ~ NEWLINE* ~
    (match_arm ~ (("," | NEWLINE) ~ NEWLINE* ~ match_arm)* ~ ("," | NEWLINE)? ~ NEWLINE*)? ~ EOI
}

unterminated_paren_expr = {
    "(" ~ expression? ~ EOI
}

unterminated_call_args = {
    "(" ~ arg_list? ~ EOI
}

unterminated_command_substitution = {
    "$(" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)*)? ~ EOI
}
```

### Integration into existing rules

The unterminated variants must be added as alternatives within the rules that reference the complete forms. For example:

```pest
// Before:
block = { "{" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)* ~ stmt_sep?)? ~ "}" }

// After:
block = { "{" ~ NEWLINE* ~ (statement ~ (stmt_sep ~ statement)* ~ stmt_sep?)? ~ "}" }
block_or_unterminated = _{ block | unterminated_block }
```

Then every rule that currently references `block` — `fn_decl`, `if_stmt`, `while_stmt`, etc. — changes to reference `block_or_unterminated`. This is a mechanical substitution.

Similarly for other constructs:

```pest
// Expressions
primary = {
    lambda |
    "(" ~ expression ~ ")" | unterminated_paren_expr |
    object_literal | unterminated_object_literal |
    list_literal | unterminated_list_literal |
    // ... literals ...
    triple_double_string | unterminated_triple_double_string |
    triple_single_string | unterminated_triple_single_string |
    // ... remaining ...
    command_substitution | unterminated_command_substitution |
    env_var |
    identifier
}
```

### Issues to Watch Out For

**Ambiguity between `unterminated_block` and `unterminated_object_literal`.** Both match `{ ... EOI`. The grammar needs to distinguish them. Currently, `block` appears in statements and `object_literal` appears in expressions, so they exist in different contexts. But when used as alternatives within `primary`, the unterminated variants need the same disambiguation. The solution: `unterminated_object_literal` requires at least one `object_pair` (key `:` value), while `unterminated_block` contains statements. Since pest tries alternatives in order and `object_literal` / `unterminated_object_literal` come before `block` / `unterminated_block` in `primary`, the distinction should hold. However, `{ EOI` (brace followed immediately by end of input) is ambiguous — it could be an empty block or an empty object. Default to treating it as an unterminated block.

**Recursive unterminated rules.** An `unterminated_block` contains statements, which may themselves contain expressions, which may contain `unterminated_list_literal`, etc. This is fine — pest handles recursion. The AST tree will contain `Incomplete` nodes at whatever depth the input ran out.

**Performance.** Adding unterminated alternatives does not meaningfully affect parse performance. Pest's PEG engine tries the complete rule first; the unterminated alternative is only attempted after the complete rule fails at that position. The extra backtracking is proportional to the size of the input, not exponential.

### Critical Analysis

**Alternative: Continue using the heuristic approach.**
- Pro: No grammar changes. No AST changes. Simpler overall.
- Con: Heuristics are fragile. `let x =` is missed. Nested constructs (e.g., unterminated string inside a block) produce confusing errors. Cannot distinguish "needs more input" from "genuinely wrong."

**Alternative: Make all closers optional with `?`.**
- Pro: Even simpler grammar changes (just `"}"?` instead of `"}"`).
- Con: Loses the distinction between complete and unterminated. The AST can't tell whether `{ x }` or `{ x` was parsed — both produce a block. The REPL can't detect incomplete input by inspecting the AST.

**Chosen approach (explicit unterminated productions):**
- Pro: Clear signal in the AST. Parser never fails. VM can produce targeted error messages for each kind of unterminated construct. REPL has an authoritative, parser-derived answer about completeness.
- Con: More grammar rules to maintain. Each new delimited construct needs a corresponding unterminated rule.

### Decisions

**Decision:** Should `{ EOI` (open brace, immediate end of input) be treated as an unterminated block or an unterminated object literal?
--> This issue should not arise.  Object literals should not be a top level production.  It should be treated as an unterminated block.

**Decision:** Should unterminated single-line strings (`"hello` without closing quote) match an unterminated production or remain a parse error? They're unlikely in multiline REPL input but could occur in `-c` inline code.
--> (Almost) nothing should be a parse error.  The grammar should match on almost every input.  It may match on something that system as a whole treats as an error (like an unterminated string) but the parser should treat it as a successful match.  The parser should match unterminated single line strings as unterminated single line strings.  The shell should treate that production as an error, not as still waiting for input.

---

## Feature 3: AST Additions

### New AST variants

Add an `Incomplete` variant to both `Statement` and `Expression`:

```rust
// ish-ast/src/lib.rs

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncompleteKind {
    Block,
    ListLiteral,
    ObjectLiteral,
    TripleDoubleString,
    TripleSingleString,
    BlockComment,
    Match,
    ParenExpr,
    CallArgs,
    CommandSubstitution,
}

// Add to Expression enum:
pub enum Expression {
    // ... existing variants ...

    /// Parsed from an unterminated production. Contains partial content.
    Incomplete {
        kind: IncompleteKind,
        partial: Box<Expression>,  // Whatever was parsed before EOI
    },
}

// Add to Statement enum:
pub enum Statement {
    // ... existing variants ...

    /// Parsed from an unterminated production. Contains partial content.
    Incomplete {
        kind: IncompleteKind,
        partial_statements: Vec<Statement>,  // Statements parsed before EOI
    },
}
```

### Design rationale

Using a single `IncompleteKind` enum (rather than separate variants per construct) keeps the AST diff small and makes it easy to add new kinds later. The `partial` / `partial_statements` fields preserve whatever was successfully parsed before input ended — this is useful for error messages and for the REPL to display partial results.

### Issues to Watch Out For

**Where does `Incomplete` live — Statement or Expression?** It depends on the construct:

| Unterminated construct | AST location | Why |
|-----------------------|-------------|-----|
| `unterminated_block` | `Statement::Incomplete` | Blocks are statements |
| `unterminated_list_literal` | `Expression::Incomplete` | Lists are expressions |
| `unterminated_object_literal` | `Expression::Incomplete` | Objects are expressions |
| `unterminated_triple_double_string` | `Expression::Incomplete` | Strings are expressions |
| `unterminated_triple_single_string` | `Expression::Incomplete` | Strings are expressions |
| `unterminated_block_comment` | Neither — comments are discarded | See below |
| `unterminated_match` | `Statement::Incomplete` | Match is a statement |
| `unterminated_paren_expr` | `Expression::Incomplete` | Parens are expressions |
| `unterminated_call_args` | `Expression::Incomplete` | Calls are expressions |
| `unterminated_command_substitution` | `Expression::Incomplete` | Command subs are expressions |

**Block comments.** Pest's `COMMENT` rule is implicit — matching comments are silently discarded. An `unterminated_block_comment` can't produce an AST node through the normal `COMMENT` mechanism. Options:

1. Make `unterminated_block_comment` a statement-level rule rather than a comment rule. Add it to `statement` alternatives. The AST builder maps it to `Statement::Incomplete { kind: BlockComment, ... }`.
2. Accept that unterminated block comments remain parse errors. The REPL validator (bracket-counting state machine) catches `/*` without `*/` at the keystroke level.

Option 2 is simpler and sufficient. Block comments in REPL input are rare. The fast-path validator already handles this case.

### Critical Analysis

**Alternative: A generic `Partial(String)` node instead of typed `IncompleteKind`.**
- Pro: One variant, stores raw text.
- Con: Loses information about what construct was unterminated. REPL can't distinguish "unterminated string" from "unterminated block" for continuation prompts or error messages.

**Alternative: No AST change — use a side-channel flag.**
- Pro: AST stays clean. Parser sets a `has_incomplete: bool` flag alongside the `Program`.
- Con: Loses location information. Can't tell which part of the input is incomplete. Harder for the VM to produce good error messages for intentionally unterminated constructs.

**Chosen approach (typed Incomplete variants):**
- Pro: Full information preserved. REPL and VM can pattern-match on `IncompleteKind`. Good error messages. Extensible.
- Con: Two new enum variants to handle everywhere the AST is matched.

### Decisions

**Decision:** Should unterminated block comments be handled via a grammar production (option 1) or remain a parse error caught by the keystroke validator (option 2)?
--> It should be a grammar production. Although we are only interested in the shell right now, all of these incomplete productions are also needed in order to generate good error messages when a whole file is being read.

---

## Feature 4: AST Builder Mappings

Each new grammar rule needs a corresponding builder function.

### Statement-level builders

```rust
// ast_builder.rs additions

fn build_unterminated_block(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut statements = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::EOI => break,
            _ => statements.push(build_statement(inner)?),
        }
    }
    Ok(Statement::Incomplete {
        kind: IncompleteKind::Block,
        partial_statements: statements,
    })
}

fn build_unterminated_match(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let mut inner = pair.into_inner();
    let _subject = build_expression(inner.next().unwrap())?;
    let mut statements = Vec::new();
    for arm_pair in inner {
        match arm_pair.as_rule() {
            Rule::match_arm => { /* parse arm, push to list */ }
            Rule::EOI => break,
            _ => {}
        }
    }
    Ok(Statement::Incomplete {
        kind: IncompleteKind::Match,
        partial_statements: statements,
    })
}
```

### Expression-level builders

```rust
fn build_unterminated_list(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut elements = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::EOI => break,
            _ => elements.push(build_expression(inner)?),
        }
    }
    Ok(Expression::Incomplete {
        kind: IncompleteKind::ListLiteral,
        partial: Box::new(Expression::ListLiteral(elements)),
    })
}

fn build_unterminated_object(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut pairs_vec = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::object_pair => {
                // Parse key-value pair
                let mut kv = inner.into_inner();
                let key = kv.next().unwrap().as_str().to_string();
                let value = build_expression(kv.next().unwrap())?;
                pairs_vec.push((key, value));
            }
            Rule::EOI => break,
            _ => {}
        }
    }
    Ok(Expression::Incomplete {
        kind: IncompleteKind::ObjectLiteral,
        partial: Box::new(Expression::ObjectLiteral(pairs_vec)),
    })
}

fn build_unterminated_triple_double(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    // Collect parts parsed before EOI
    let mut parts = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::triple_double_text => parts.push(StringPart::Text(inner.as_str().to_string())),
            Rule::triple_double_interp => {
                let expr = build_expression(inner.into_inner().next().unwrap())?;
                parts.push(StringPart::Expr(expr));
            }
            Rule::triple_double_env => {
                let name = &inner.as_str()[1..]; // strip $
                parts.push(StringPart::Expr(Expression::EnvVar(name.to_string())));
            }
            Rule::EOI => break,
            _ => {}
        }
    }
    Ok(Expression::Incomplete {
        kind: IncompleteKind::TripleDoubleString,
        partial: Box::new(Expression::StringInterpolation(parts)),
    })
}

fn build_unterminated_triple_single(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let content = pair.into_inner()
        .take_while(|p| p.as_rule() != Rule::EOI)
        .map(|p| p.as_str())
        .collect::<String>();
    Ok(Expression::Incomplete {
        kind: IncompleteKind::TripleSingleString,
        partial: Box::new(Expression::Literal(Literal::String(content))),
    })
}

fn build_unterminated_paren(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let inner_expr = pair.into_inner()
        .find(|p| p.as_rule() != Rule::EOI)
        .map(|p| build_expression(p))
        .transpose()?
        .unwrap_or(Expression::Literal(Literal::Null));
    Ok(Expression::Incomplete {
        kind: IncompleteKind::ParenExpr,
        partial: Box::new(inner_expr),
    })
}

fn build_unterminated_call(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut args = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::EOI => break,
            _ => args.push(build_expression(inner)?),
        }
    }
    Ok(Expression::Incomplete {
        kind: IncompleteKind::CallArgs,
        partial: Box::new(Expression::ListLiteral(args)), // reuse list to carry partial args
    })
}

fn build_unterminated_cmd_sub(pair: Pair<Rule>) -> Result<Expression, ParseError> {
    let mut statements = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::EOI => break,
            _ => statements.push(build_statement(inner)?),
        }
    }
    Ok(Expression::Incomplete {
        kind: IncompleteKind::CommandSubstitution,
        partial: Box::new(Expression::CommandSubstitution(Box::new(
            Statement::Block { statements },
        ))),
    })
}
```

### Statement dispatcher update

```rust
fn build_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    match pair.as_rule() {
        // ... existing arms ...
        Rule::block => build_block(pair),
        Rule::unterminated_block => build_unterminated_block(pair),
        Rule::unterminated_match => build_unterminated_match(pair),
        // ...
    }
}
```

---

## Feature 5: REPL Incomplete Detection

### How it works

After `parse(input)` succeeds (the parser now always succeeds), the REPL walks the AST looking for `Incomplete` nodes:

```rust
// ish-shell/src/repl.rs

fn has_incomplete(program: &Program) -> bool {
    program.statements.iter().any(|s| stmt_is_incomplete(s))
}

fn stmt_is_incomplete(stmt: &Statement) -> bool {
    match stmt {
        Statement::Incomplete { .. } => true,
        Statement::Block { statements } => statements.iter().any(stmt_is_incomplete),
        Statement::If { condition, then_block, else_block } => {
            expr_is_incomplete(condition)
                || stmt_is_incomplete(then_block)
                || else_block.as_ref().map_or(false, |b| stmt_is_incomplete(b))
        }
        // ... recurse into all statement variants that contain sub-statements/expressions
        _ => false, // leaf statements
    }
}

fn expr_is_incomplete(expr: &Expression) -> bool {
    match expr {
        Expression::Incomplete { .. } => true,
        Expression::BinaryOp { left, right, .. } => {
            expr_is_incomplete(left) || expr_is_incomplete(right)
        }
        Expression::FunctionCall { callee, args } => {
            expr_is_incomplete(callee) || args.iter().any(expr_is_incomplete)
        }
        // ... recurse into all expression variants
        _ => false,
    }
}
```

### REPL control flow

```
Input submitted
    ↓
parse(input) → Program
    ↓
has_incomplete(&program)?
    ├─ yes → append newline, show "...> " prompt, continue reading
    └─ no → vm.run(&program)
              ├─ If the AST contains error nodes → VM formats error messages
              └─ Otherwise → normal execution
```

### Interaction with the multiline validator (Feature 3 from shell-construction)

The keystroke-level `IshValidator` (bracket-counting state machine) is **still useful** as a fast path:

| Layer | Runs | Speed | Accuracy |
|-------|------|-------|----------|
| `IshValidator` | Every keystroke (reedline calls `validate()`) | O(n) character scan, no allocation | Approximate — counts brackets, tracks string state |
| `parse()` + `has_incomplete()` | On submit (Enter pressed while validator says "complete") | Full parse | Authoritative |

The validator prevents premature submission: if brackets are unbalanced, reedline inserts a newline instead of submitting. Once the validator says "complete," the input is submitted and the full parser runs. If the parser finds an `Incomplete` node that the validator missed (e.g., `let x =` — balanced but incomplete), the REPL appends and continues.

This two-layer approach means the common case (typing a multi-line block) never invokes the parser until the user finishes typing — the validator handles continuation. The parser is the authoritative fallback for edge cases.

--> Counting brackets is fundamentally broken.  An input statement like `let x ='{'` fails the bracket count and causes a bug. There should be NO bracket counting.

### Decisions

**Decision:** Should the `has_incomplete` / `stmt_is_incomplete` / `expr_is_incomplete` functions live in `ish-shell` (REPL-specific), or in `ish-ast` (available to all crates)?
--> ish-ast

---

## Feature 6: The `$?` Grammar Extension

As decided in the predecessor proposal, `$?` is implemented as a synthetic env var. The grammar needs to accept `?` after `$`:

```pest
// Before:
env_var = @{
    "$" ~ "{" ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* ~ "}" |
    "$" ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")*
}

// After:
env_var = @{
    "$" ~ "?" |
    "$" ~ "{" ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* ~ "}" |
    "$" ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")*
}
```

The `$?` alternative comes first so it matches before the general `$IDENT` pattern.

**Files affected:**
- `proto/ish-parser/src/ish.pest` — extend `env_var` rule

---

## Feature 7: Inline Execution with `-c`

As decided, the shell accepts `ish -c 'code'`. No further design changes from the shell-construction proposal.

**Files affected:**
- `proto/ish-shell/src/main.rs` — as specified in shell-construction.md Feature 2

---

## Implementation Sequence

```
1. AST: Add IncompleteKind, Statement::Incomplete, Expression::Incomplete
   ↓
2. Grammar: Add unterminated_* productions (Tier 1)
   ↓
3. Grammar: Integrate unterminated rules into existing rules
   ↓
4. AST builder: Add build_unterminated_* functions
   ↓
5. AST builder: Wire new rules into dispatchers
   ↓
6. Tests: Parser tests for each unterminated construct
   ↓
7. REPL: has_incomplete() detection
   ↓
8. Grammar: $? extension to env_var
```

Steps 1 and 2 can proceed in parallel. Steps 6 should accompany each grammar/builder change. Step 8 is independent.

---

## Files Affected

| File | Changes |
|------|---------|
| `proto/ish-ast/src/lib.rs` | Add `IncompleteKind` enum, `Incomplete` variant to `Statement` and `Expression` |
| `proto/ish-parser/src/ish.pest` | Add ~10 `unterminated_*` rules, modify existing rules to use `_or_unterminated` alternatives, extend `env_var` for `$?` |
| `proto/ish-parser/src/ast_builder.rs` | Add ~10 `build_unterminated_*` functions, update dispatchers |
| `proto/ish-shell/src/repl.rs` | Add `has_incomplete()` / `stmt_is_incomplete()` / `expr_is_incomplete()` |
| `proto/ish-parser/tests/` | Add tests for each unterminated production |

---

## Deferred Items

| Feature | Rationale |
|---------|-----------|
| Tier 2 unterminated productions (single-line strings, extended strings, annotations, type annotations) | Uncommon in REPL; add if users request |
| Unterminated block comment grammar production | Handled by keystroke validator |
| VM error messages for incomplete constructs | VM currently doesn't execute `Incomplete` nodes — separate effort |

---

## Documentation Updates

| File | Update |
|------|--------|
| `docs/architecture/ast.md` | Document `IncompleteKind`, `Statement::Incomplete`, `Expression::Incomplete` |
| `docs/architecture/shell.md` | Two-layer validation: validator (keystrokes) + parser (submit) |
| `docs/spec/syntax.md` | Note that the grammar matches incomplete input rather than failing |
| `proto/ARCHITECTURE.md` | Update parser description |

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-shell-build.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
