---
title: Language Syntax Design
category: project
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/project/proposals/language-syntax.md, docs/spec/syntax.md]
---

# Language Syntax Design

*March 14, 2026*

## The Starting Point

Until this point, ish had no formal syntax. The prototype constructed programs as ASTs directly in Rust using builder APIs and convenience constructors — there was no parser, no grammar, and no specification for what ish source code should look like. The documentation showed code examples in an informal notation that drew loosely from Rust and TypeScript, but the examples were inconsistent with each other and no decisions had been formalized.

Four sources of syntactic influence existed: the Rust-flavored AST node types and builder APIs in the prototype, the TypeScript-inspired examples scattered through the documentation, the old pest grammar from an earlier prototype in the ish-workspace, and the existing spec documents which had established certain conventions for assurance ledger syntax, error handling, and type declarations. The task was to reconcile these four sources into a coherent, complete syntax specification.

## The Tension Between Shell and Language

The central design challenge was that ish is not just a programming language — it is also a shell. The thin shell execution configuration accepts command-line input and processes it immediately, which means the syntax must support both structured programming and bare-word command invocation. This is a tension that has plagued every language-shell hybrid: how do you tell whether `git status` is a function call or a command invocation?

The proposal surveyed seven approaches from existing languages and shells. Nushell treats everything as a structured data pipeline. PowerShell uses `$` sigils for variables to avoid ambiguity. Xonsh uses heuristic detection to guess whether a line is Python or a shell command. Fish and Elvish use keyword-based rules — if a line starts with a recognized keyword, it's code; otherwise it's a command.

The keyword-based approach won. A line is parsed as a language statement if it begins with a recognized keyword (`let`, `fn`, `if`, `while`, `for`, `use`, `type`, etc.) or has unambiguous language syntax (assignment, type annotation). Otherwise, it is parsed as a command invocation. For the rare edge case where a command name coincides with a keyword, the `>` prefix forces command mode. This approach is simple, predictable, and has precedent in Elvish and Fish.

A critical addition was the decision that a standard can prohibit shell mode entirely. This means project source code can be configured to never invoke shell commands — preventing accidental command invocations caused by syntax errors.

## Choosing Between Rust and TypeScript Conventions

Many individual syntax decisions required choosing between Rust and TypeScript conventions. The choices were guided by a few principles: shell-friendliness (avoiding syntax that conflicts with shell operators), readability for both low-assurance and high-assurance code, and consistency with decisions already made in the documentation.

**Semicolons** were the first major decision. Rust requires them; TypeScript uses automatic semicolon insertion (ASI), which is widely considered a mistake. The modern trend (Go, Kotlin, Swift) is toward newline-terminated statements. The decision was newline-terminated with optional semicolons — newlines end statements, semicolons are available for multiple statements on one line, and multiline expressions continue when a line ends with an operator or open bracket.

**Logical operators** posed a real conflict. Rust and TypeScript use `&&`/`||`, but `&&` has meaning in shells (command chaining, "and then if success"). The ish documentation already used `and`/`or`/`not`, following Python's convention. The decision preserved `and`/`or`/`not` for logical operations and reserved `&&` for shell-mode command chaining. This means `&&` has different semantics in shell mode (sequential execution) versus programming mode (where it is not valid — use `and` instead).

**Condition parentheses** were another Rust-vs-TypeScript split. Rust and Go prohibit parentheses around `if`/`while` conditions; TypeScript and C require them. The initial proposal recommended optional parentheses, but the decision went further: parentheses are **prohibited**. This is a stronger position than most languages take, but it enforces a consistent visual style and avoids the inconsistency that optional parentheses create in codebases.

**Lambda syntax** had a clear winner. Rust's `|x| body` syntax clashes with the shell pipe operator `|`, which would cause real confusion in a shell-language hybrid. TypeScript's arrow syntax `(x) => expr` has no such conflict and was already established in the ish documentation. The decision supported both expression-body lambdas (`(x) => x * 2` with implicit return) and block-body lambdas (`(x) => { ... }` with explicit `return`). Named functions always require explicit `return` — no Rust-style last-expression returns, which would interact badly with optional semicolons.

**For loops** adopted Rust/Python's `for x in iter` syntax, dropping C-style `for (init; cond; step)` entirely. The `match` keyword was reserved for future pattern matching but not specified, and there is no `loop` keyword — `while true` suffices.

## Functions and Error Handling

**Default parameters** were adopted (`fn connect(host: String, port: i32 = 8080)`), departing from Rust's approach of using `Option` types. This was motivated by ish's goal of being approachable in low-assurance mode — requiring `Option<T>` for simple defaults is hostile to that goal.

**The `?` operator** was initially proposed for deferral, but the decision was to implement it immediately. It is syntactic sugar for detecting if a function's return value is an error type and throwing it if it is. This connects to a broader correction: the proposal identified that the assurance ledger documentation had been using `fn get_user(id: i64) -> User throws NotFoundError` as an example, but this is wrong. ish functions never throw in their signatures — they return union types. The correct form is `fn get_user(id: i64) -> User | NotFoundError`. Throwing is a behavior of the *calling* function, not the callee. The `?` operator on the caller side is what converts an error return value into a throw.

**Function types** use `fn(Args) -> Ret` syntax rather than TypeScript's `(Args) => Ret`, maintaining consistency with function declarations.

## Strings, Types, and Visibility

**String syntax** proved complex enough to warrant its own follow-on proposal. The decision for the current phase was double-quoted strings only, but the discussion identified requirements that the follow-on must address: string literals containing quote characters without escapes, multiline strings, string interpolation that works with both ish variables and environment variables, consistency between programming mode and shell mode, and a `char` literal syntax that doesn't steal a useful quote character. An RFP was generated for this topic.

**Type declaration syntax** was largely settled by the existing documentation. The one significant change was the removal of `nominal type` — nominal typing is now handled through entries/annotations in the assurance ledger, not through a dedicated keyword.

**Visibility** settled on bare `pub` meaning `pub(global)`, departing from a proposal that suggested `pub(project)` as the default. The reasoning was that having `pub` mean anything other than "fully public" would be deeply unintuitive. Instead, the default visibility (when no `pub` is written) is `pub(self)`, but this default is configurable via a standard.

## Comments and Parser Strategy

**Comments** accept both `//` (programmer convention) and `#` (shell convention) as line comment starters. Block comments use `/* */` with Rust-style nesting support. The cost of accepting both is minimal, and the benefit to shell users who instinctively type `#` is significant.

**Parser technology** selected pest (PEG parser generator) with an error-accepting grammar. The approach is to write rules for both valid and invalid forms of each construct, so the parser always succeeds. Error reporting then walks the parse tree and generates diagnostics for invalid nodes. This was chosen over hand-written recursive descent (more code to maintain), tree-sitter (FFI complexity, deferred for IDE support), and parser combinators (less declarative). The grammar is structured in layers: lexer rules → expression rules → statement rules → top-level rules.

## Implementation Phases

The syntax features were organized into eight implementation phases, progressing from a minimal viable language (variables, arithmetic, control flow, functions) through data structures, error handling, type system surface, modules, shell integration, assurance ledger annotations, and finally advanced features (pattern matching, generics, async/await). All eight phases have been implemented, and the documentation has been updated to reflect the final syntax across the specification, user guide, and architecture documents.

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
