---
title: "History: Error Handling Implementation"
category: history
audience: [human-dev, ai-agent]
status: stable
last-verified: 2026-03-11
depends-on: [docs/project/proposals/error-handling.md]
---

# 2026-03-11: Error Handling Implementation

The first implementation of error handling in the ish prototype, covering throw/try/catch/finally, with blocks, defer, and error builtins.

## Background

The ish prototype had no error handling beyond `RuntimeError` — a Rust-side error type used for runtime failures like division by zero or argument count mismatches. There was no way for ish programs to throw, catch, or handle errors. The user-guide error handling page was a placeholder that mentioned result types, contradicting the spec's mention of thrown exceptions.

The human developer created an `errors` prompt file containing a detailed design for error handling, covering: the streamlined-to-encumbered continuum for errors, throw/try/catch syntax, closeable resources (with blocks), stack trace context, frame suppression, context parameters, and return handlers.

## The Proposal

The `/propose` skill processed the `errors` file and produced a structured proposal at `docs/project/proposals/error-handling.md`. The proposal analyzed seven features:

1. **Error type and throw/catch mechanism** — should errors be structural objects or nominal types? Alternative approaches included Java-style class hierarchy, Go-style error interface, and Rust-style Result types.

2. **Finally blocks** — should `return` from `finally` be allowed? The analysis noted that Java allows it but it's widely considered a bad practice.

3. **With blocks (resource management)** — how to identify the close method: annotation, naming convention, or interface. How to handle close failures.

4. **Defer** — Go-style deferred cleanup. Should it be function-scoped (Go) or block-scoped?

5. **Stack trace context and annotations** — block-level vs. function-level context annotations, suppressed frame visibility.

6. **Context parameters** — implicit parameters for runtime metadata. Whether to expose developer-facing syntax.

7. **Return handler mechanism** — the hidden implementation detail that manages error propagation and stack traces.

The proposal included 12 decision prompts, a 10-step implementation sequencing plan, and documentation update requirements.

## The Decisions

The human developer filled in all 12 decision prompts:

- **Error objects:** Only Error objects should be throwable, using structural types with metadata (annotations on the thrown value).
- **Finally returns:** No return from finally — confirmed.
- **Error model:** Alternative C (Result types under the hood with throw/catch sugar) — a significant choice that aligns ish's error model with both exception-oriented and result-oriented paradigms.
- **With block close identification:** Probably annotation on close method (left as open question).
- **Defer scope:** Block-scoped (with a note to research Go's approach — this is an open question).
- **Context messages:** Not lazily evaluated — values could change between throw and catch.
- **Stack trace context:** Block-level annotation (not function-level or combined).
- **Suppressed frames in verbose mode:** Yes.
- **Suppressed function context:** No — if you want context, you want the whole frame.
- **`?` operator:** Rust-style, sugar for overriding encumbrance to use default handler.
- **Error type inference:** Compiler should infer error union types automatically (config option to force declaration).
- **Error mode presets:** Three presets: streamlined, encumbered, and no-throw. Configurable at project/module/function level.
- **Handler mechanism:** Separate execution concern, hidden implementation detail.
- **Context parameters:** Built-ins only for now (crutch for interpreter's lack of lexical scope access). No developer syntax yet.

Several decisions were explicitly left as open questions for future work: close method identification, defer scoping, and the `?` operator syntax.

## The Implementation

The implementation covered steps 1–6 of the proposal's 10-step sequencing plan (the foundational runtime mechanics). Steps 7–10 (return handler mechanism, encumbered mode enforcement, `?` operator, configuration syntax) were deferred — they require type system infrastructure that doesn't exist yet.

### AST Changes (ish-ast)

Four new `Statement` variants were added:
- `Throw { value: Expression }` — raises a thrown value
- `TryCatch { body, catches: Vec<CatchClause>, finally }` — structured error handling
- `WithBlock { resources: Vec<(String, Expression)>, body }` — resource management
- `Defer { body }` — deferred cleanup

A new `CatchClause` struct supports future type-based catch matching via an optional type annotation.

Convenience constructors, builder methods (`BlockBuilder::throw()`, `.try_catch()`, `.defer()`), and display formatting were added for all four.

### Interpreter Changes (ish-vm)

The `ControlFlow` enum gained a `Throw(Value)` variant. This was the most consequential design decision in the implementation — thrown values travel through the control flow system rather than through Rust's `Result` type.

**Throw mechanics:** A throw unwinds through blocks and loops until it hits a try/catch or a function boundary. At a function boundary (`call_function`), `ControlFlow::Throw(v)` is converted to `Err(RuntimeError::thrown(v))`, preserving the thrown value. This means throws don't silently cross function boundaries — they become explicit errors.

**TryCatch execution:** The initial implementation only caught `ControlFlow::Throw` from within the try body. This failed for throws from function calls (which become `RuntimeError`). The fix: `TryCatch` also catches `RuntimeError`s with `thrown_value`, re-extracting the original value. This allows try/catch to work seamlessly across function call boundaries.

**With blocks:** Resources are initialized in declaration order. If a later initialization fails, earlier resources are closed in reverse. The body executes in a child scope. On exit (normal or throw), all resources are closed in reverse order. Body errors take precedence over close errors. A `try_close` helper looks for a `close` method on Object values.

**Defer:** During block execution, `Defer` statements are collected into a vector rather than executed immediately. When the block exits — normally, via return, or via throw — deferred statements execute in LIFO order.

### Error Builtins

Three new built-in functions: `new_error(message)` creates an error object with `message` and `__is_error__` metadata, `is_error(value)` checks for the metadata flag, and `error_message(error)` extracts the message.

### Reflection

The reflection system (AST↔Value bidirectional conversion) was extended with `stmt_to_value`/`value_to_stmt` cases for all four new statement types, plus three new AST factory builtins: `ast_throw`, `ast_try_catch`, `ast_defer`.

### Test Results

14 new tests were added to the interpreter module, covering: basic throw, try/catch, try/catch with return, finally on normal exit, finally on throw, throw across function boundaries, throw from function caught by caller, with blocks (normal and throw), defer execution, defer LIFO order, error builtins, error message roundtrip through throw/catch, and try without throw.

All 59 workspace tests pass (up from 45). All 6 end-to-end shell demos continue to pass.

## Documentation Updates

Ten documentation files were updated:
- **User guide error handling** — complete rewrite from placeholder to draft, covering throw/try/catch/finally, error type, with blocks, defer, and encumbrance continuum
- **Architecture AST** — new nodes documented with CatchClause struct
- **Architecture VM** — control flow, throw mechanics, with blocks, defer, error builtins, test counts updated
- **Spec types** — expanded Error Handling section, closed Error Types open questions
- **Spec agreement** — added error mode preset as a configurable marked feature
- **Spec execution** — added section on error handling across configurations and the return handler mechanism
- **Error catalog** — populated with 6 initial error codes
- **Open questions** — closed error handling question, added new questions from proposal decisions
- **Glossary** — added: context parameter, defer, return handler, suppressed error, with block

## What's Left

The remaining proposal steps require type system infrastructure:
- Step 7: Return handler mechanism (needs stack frames)
- Step 8: Encumbered mode enforcement (needs function signatures with error types)
- Step 9: `?` operator (needs result type inference)
- Step 10: Configuration syntax for error mode presets

Open questions from the decisions: close method identification mechanism, whether defer should be function-scoped or block-scoped, `?` operator exact semantics, configuration syntax.

## Participants

- **Human developer:** Created the `errors` design file with comprehensive requirements. Filled in all 12 decision prompts in the proposal, providing direction on error model philosophy (Result types under the hood), scope decisions (block-scoped defer), and which features to defer to future work.

- **AI developer:** Created the proposal via `/propose`, implemented all AST, interpreter, reflection, and builtin changes, wrote 14 tests, updated 10 documentation files, and created this history file.

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
