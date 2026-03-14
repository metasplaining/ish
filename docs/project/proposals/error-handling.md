---
title: "Proposal: Error Handling"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-11
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/execution.md, docs/spec/reasoning.md, docs/user-guide/error-handling.md]
---

# Proposal: Error Handling

*Generated from `errors` on 2026-03-11.*

---

## Questions and Answers

### Q: What patterns similar to Java's "closeable" should ish support?

Java's `AutoCloseable` / `try-with-resources` is one instance of a broader family of **resource management patterns**. The key patterns worth considering:

1. **Disposable / close-on-exit** (Java `AutoCloseable`, C# `IDisposable`/`using`, Python `__enter__`/`__exit__`). The resource is acquired, used, and then deterministically released when the block exits — whether normally or via error. This is what the `with` block in the prompt describes.

2. **Flushing / finalizing** (Go `defer`). A broader pattern where any cleanup action is scheduled for block or function exit. Unlike closeable, the deferred action doesn't need to be tied to a resource object — it can be any statement. Go's `defer` stacks deferred calls LIFO. This is strictly more general than closeable.

3. **Scoped access / loan** (Rust's borrow checker performs this statically; some runtimes provide dynamic scoped access). A resource is borrowed for a scope and the borrow is guaranteed to end. Ish could support this via a callback pattern: `resource.withAccess(fn(ref) { ... })`.

4. **Transaction / rollback** (database transactions, software transactional memory). A block of operations either all succeed or all get rolled back. This is a higher-level pattern that could be layered on top of closeable — an object implements both `commit()` and `rollback()`, and the `with` block calls `commit()` on normal exit and `rollback()` on error exit.

**Recommendation:** Support the `with` block (closeable pattern) as the primary mechanism, modeled on Java's `try-with-resources` with the following semantics:
- The object must implement a `close()` method.
- `close()` is called when the block exits, whether normally or via error.
- If both the body and `close()` throw, the body's error takes precedence and the close error is attached as a suppressed error.

Also support **`defer` statements** as a more general mechanism. A `defer` statement registers an action to be executed when the enclosing block exits. This subsumes closeable (you can `defer obj.close()`) and supports arbitrary cleanup.

The transaction/rollback and scoped access patterns can be built as library abstractions on top of `with`/`defer` without language-level support.

### Q: What should the syntax be for suppressing stack frames?

The prompt suggests `stacktrace.suppressFrames(frameCount: u32)`. Several alternatives are worth considering:

1. **Explicit frame count** — `stacktrace.suppressFrames(3)`. Simple but fragile: if the call stack changes (e.g., a helper function is inlined or extracted), the count becomes wrong.

2. **Attribute/annotation on functions** — `@stacktrace.suppress fn internal_helper() { ... }`. The function's frames are always suppressed in stack traces. This is more robust because it's tied to the function declaration, not the call site.

3. **Predicate-based filtering** — `stacktrace.suppressWhere(fn(frame) { frame.module.startsWith("framework.") })`. Most powerful but most complex. Allows suppressing all frames matching a pattern.

4. **Module-level suppression** — `@stacktrace.suppress module framework.internals`. All functions in the module are suppressed. Clean for library authors.

**Recommendation:** Use function-level annotations as the primary mechanism (`@stacktrace.suppress` on a function declaration). This is the most robust and intuitive approach. The annotation could also accept a condition for more nuanced control. The `stacktrace.suppressFrames(n)` imperative API could be kept as an escape hatch for dynamically-determined suppression.

---

## Feature: Streamlined Error Handling (throw / try / catch / finally)

### Issues to Watch Out For

- **Contradiction with existing docs.** The current user guide ([docs/user-guide/error-handling.md](../../user-guide/error-handling.md)) says "errors are values, not exceptions." The prompt describes a throw/catch model for streamlined mode. This needs to be reconciled — the user guide needs a rewrite.
- **Error type identity.** The prompt says the error type is "Error" and is nominally typed. This means `throw { msg: "..." }` wouldn't work unless the object is an instance of `Error`. Need to decide whether arbitrary values can be thrown (like JavaScript) or only `Error` instances (like Java).
- **Interaction with compiled mode.** In compiled mode, exceptions are expensive. The mechanism described in the prompt (throw = return at the boundary, with configurable return handlers) addresses this, but the streamlined developer shouldn't need to think about performance — the language should make it transparent.
- **finally semantics with return.** If a `finally` block contains a `return`, does it override the thrown error? (Java says yes, which is widely considered a design mistake.)

### Critical Analysis

**Alternative A: Conventional throw/catch (as proposed)**
- Pros: Familiar to Java/TypeScript/Python developers. Natural fit for streamlined ish. Low ceremony. Errors propagate automatically until caught.
- Cons: Hidden control flow — a function call might throw and you can't tell from the call site. Performance cost of stack unwinding (mitigated in compiled mode by the handler mechanism). Error types aren't visible in function signatures (in streamlined mode).

**Alternative B: Mandatory Result types (Rust-style) even in streamlined mode**
- Pros: Errors are always visible. No hidden control flow. Function signatures are honest.
- Cons: Boilerplate-heavy. Violates the streamlined goal of minimizing ceremony. Unfamiliar to the target streamlined audience (JavaScript/Python developers).

**Alternative C: Hybrid with sugar — Result types underneath, throw/catch as syntax sugar**
- Pros: Clean mental model for streamlined developers. Result types underneath for encumbered developers. Single underlying mechanism.
- Cons: Two views of the same mechanism could be confusing. The "sugar" needs to be very well designed to avoid leaky abstractions.

The prompt essentially describes Alternative C — throw/catch is the developer-facing API, but under the hood, `throw` returns an error value and a configurable return handler re-throws it. This is the strongest approach because it unifies the streamlined and encumbered models.

### Proposed Implementation

**AST nodes needed:**
- `Throw(expr)` — throw an expression as an error
- `TryCatch { body, catches: Vec<CatchClause>, finally }` — try/catch/finally block
- `CatchClause { param, type_annotation, body }` — a catch clause binding the error

**VM changes:**
- Add a `ControlFlow::Throw(Value)` variant to the interpreter's control flow enum
- `exec_throw` pushes a stack frame onto the stacktrace context parameter, then returns `ControlFlow::Throw`
- `exec_try_catch` runs the body, catches `Throw` control flow, matches against catch clauses, runs finally
- The return handler (already described in prompt) re-throws errors that escape functions

**Builtin changes:**
- `Error` constructor builtin (or nominal type with `new Error(message)`)
- `stacktrace` context parameter (see Feature: Context Parameters below)

**Files affected:**
- `proto/ish-ast/src/lib.rs` — new AST node variants
- `proto/ish-vm/src/interpreter.rs` — execution of throw/try/catch/finally
- `proto/ish-vm/src/error.rs` — reconcile `RuntimeError` with ish `Error` type
- `docs/spec/syntax.md` — syntax specification
- `docs/user-guide/error-handling.md` — complete rewrite

### Decisions

**Decision:** Should arbitrary values be throwable (like JavaScript) or only `Error` instances (like Java)?
--> Only Error.  I need to elaborate more on nominal vs. structural types, which will bear on this question.  Basically, I am thinking that instead of having nominal types, we will have structural types with metadata, one type of metadata is a name.  Another type of metadata is an error annotation.  Essentially, the "throw { msg: "whatever"};" statement from the prompt throws an Error, because part of the throw semantics is to annotate the object being thrown with "Error".

**Decision:** Does `finally` that contains `return` override a thrown error?
--> No return from finally.

**Decision:** Is Alternative C (Result types under the hood with throw/catch as sugar) the correct approach?
--> Yes.

---

## Feature: `with` Block (Closeable Pattern)

### Issues to Watch Out For

- **Interaction with `defer`.** If both `with` and `defer` are supported, developers need guidance on when to use which. `with` is preferable for resource objects; `defer` is preferable for arbitrary cleanup.
- **Multiple resources.** Java's `try-with-resources` allows multiple resources in one statement. Ish should too: `with (a = open("x"), b = open("y")) { ... }`.
- **Suppressed errors.** When both the body and `close()` throw, the suppressed error mechanism needs design. Java attaches suppressed exceptions to the primary exception — ish should do similar.
- **How `close()` is identified.** Is it a named method? An interface/trait? A structural type match? Given ish's structural typing default, any object with a `close()` method should qualify.

### Critical Analysis

**Alternative A: `with` block (as proposed, Java-style)**
- Pros: Dedicated syntax for a common pattern. Clear intent. Compiler can verify the object has `close()`.
- Cons: Another keyword and syntax construct. Limited to the close-on-exit pattern.

**Alternative B: `defer` only (Go-style)**
- Pros: More general. One mechanism instead of two. `defer obj.close()` handles the closeable case.
- Cons: Developer must remember to write the `defer` — no compiler help if they forget. Doesn't enforce the close-on-exit pattern.

**Alternative C: Both `with` and `defer`**
- Pros: `with` for the common case (closeable resources), `defer` for everything else. Each is the right tool for its purpose.
- Cons: Two mechanisms for overlapping use cases. Slightly more language surface area.

**Recommendation:** Alternative C. `with` is a higher-level construct that provides safety guarantees; `defer` is a lower-level escape hatch. They serve different purposes.

### Proposed Implementation

**AST nodes needed:**
- `WithBlock { resources: Vec<(Identifier, Expr)>, body: Block }` — with block
- `Defer { body: Statement }` — defer statement

**Semantics:**
- Resources are initialized in order; if initialization of resource N fails, resources 0..N-1 are closed in reverse order
- On block exit (normal or throw), resources are closed in reverse order
- If close() throws and an error is already in flight, the close error is attached to the primary error via a `suppressedErrors` list
- `defer` statements execute in LIFO order at block exit

**Files affected:**
- `proto/ish-ast/src/lib.rs` — new AST node variants
- `proto/ish-vm/src/interpreter.rs` — with/defer execution
- `docs/spec/syntax.md` — syntax specification

### Decisions

**Decision:** Should `with` use structural typing (any object with `close()`) or require implementing a `Closeable` interface?
--> Probably an annotation on the close method.  Save this as an open question.

**Decision:** Should `defer` be scoped to the enclosing block or the enclosing function (Go scopes it to the function)?
--> I don't have any experience with Go, so I'm not really familiar with defer.  My intuition is to scope it to the enclosing block.  That is what enclosing blocks are for.  But save it as an open question to find out why Go does it the way they do.

---

## Feature: Stack Trace Context Pushing

### Issues to Watch Out For

- **Performance cost.** Every `stacktrace.pushCtx()` allocates a string and modifies the stacktrace. In hot loops, this could be significant. Need a way to make this zero-cost in compiled mode when stacktraces are disabled.
- **Lifetime management.** Context messages are pushed but must be popped when the scope exits. If this is manual (`pushCtx` / `popCtx`), developers will forget. It should be tied to scope exit automatically.
- **String interpolation cost.** The message string `"a[${a}]"` is evaluated even if no error occurs. This is wasteful. Consider lazy evaluation — only interpolating the string when the stacktrace is actually rendered.

### Critical Analysis

**Alternative A: Imperative push/pop (as proposed)**
`stacktrace.pushCtx("a[${a}]")` pushes, with an implicit or explicit pop.
- Pros: Simple API. Direct control.
- Cons: Manual lifecycle. If pop is manual, bugs will occur. If pop is automatic on scope exit, it's really a scoped mechanism wearing an imperative mask.

**Alternative B: Scoped context block**
```
stacktrace.ctx("a[${a}]") {
    inner(a + 1);
}
```
- Pros: Context is scoped. No push/pop lifecycle bugs. Clear visual grouping.
- Cons: Indentation. For functions where the context applies to the entire body, it's just ceremony.

**Alternative C: Function-level annotation**
```
@stacktrace.ctx("a[${a}]")
fn outer(a) {
    inner(a + 1);
}
```
- Pros: Zero indentation overhead. Clean for the common case (context applies to entire function). Parameters are in scope for string interpolation.
- Cons: Less flexible — can't scope context to a sub-block within a function.

**Alternative D: Combined — annotation for functions, block syntax for sub-scopes**
- Pros: Best of both worlds.
- Cons: Two mechanisms.

**Recommendation:** Alternative D. The function-level annotation handles 90% of cases cleanly. The block syntax handles the remaining 10%. Both desugar to the same scoped push/pop mechanism.

**Lazy evaluation:** The context message should capture the interpolation expression but not evaluate it until the stacktrace is printed. This eliminates the cost in the no-error case.

### Proposed Implementation

**Approach:** Context is implemented as part of the stacktrace context parameter mechanism. Each stack frame can carry an optional context string (or lazy context expression).

**AST support:**
- Function annotation: `@stacktrace.ctx(expr)` on function declarations
- Block form: `stacktrace.ctx(expr) { ... }` as a statement

**VM changes:**
- The stack frame representation gains an optional `context: Option<Value>` field
- `stacktrace.pushCtx(msg)` sets the context on the current frame
- When the stacktrace is rendered (on error), context messages are interpolated

**Files affected:**
- `proto/ish-ast/src/lib.rs` — annotation support, context block
- `proto/ish-vm/src/interpreter.rs` — stack frame context
- `proto/ish-vm/src/error.rs` — stacktrace rendering with context

### Decisions

**Decision:** Should context messages be lazily evaluated (only computed when the stacktrace is rendered)?
--> No.  The values used to compute the message could change, introducing subtle bugs.

**Decision:** Should Alternative D (annotation + block syntax) be adopted?
--> None of the above.  It should be a block level annotation.  Since functions (usually) have blocks, that makes it easy to have effectively a function level annotation.  In addition, the push/pop semantics work better with things like loops.  In addition, sometimes a developer needs a few lines to put together the annotation, which makes it important to not only allow it at the function level.

---

## Feature: Stack Frame Suppression

### Issues to Watch Out For

- **Debugging vs. user-facing traces.** Suppressed frames should still be available in a verbose/debug mode. Suppression should affect the default display, not the underlying data.
- **Suppression granularity.** Per-function is the cleanest, but sometimes you need per-module or per-package suppression (e.g., suppress all frames from a test framework).
- **Interaction with context.** If a function is suppressed but has a context message, should the context still appear?

### Critical Analysis

**Alternative A: Imperative frame count (as proposed)**
`stacktrace.suppressFrames(3)`
- Pros: Maximum flexibility.
- Cons: Fragile. Counts break when call stacks change. Hard to maintain.

**Alternative B: Function annotation**
`@stacktrace.suppress fn helper() { ... }`
- Pros: Robust — tied to the function, not a count. Clear intent. Works across refactors.
- Cons: Must be applied function by function.

**Alternative C: Module/package annotation**
`@stacktrace.suppress module framework.internals`
- Pros: One annotation suppresses many functions. Ideal for framework code.
- Cons: Coarse-grained. May suppress frames the developer does want to see.

**Alternative D: Predicate-based filtering**
`stacktrace.filter(fn(frame) { ... })` on the stacktrace context parameter.
- Pros: Maximum flexibility with type safety.
- Cons: Complex. Overkill for most use cases.

**Recommendation:** Primary mechanism is Alternative B (function annotation). Also support Alternative C (module annotation) for convenience. Keep Alternative A as a low-level escape hatch. Do not invest in Alternative D unless a concrete use case demands it.

### Proposed Implementation

**Approach:** Suppression is metadata on the stack frame. When rendering a stacktrace, suppressed frames are hidden by default but available in verbose mode.

**AST support:**
- `@stacktrace.suppress` annotation on function and module declarations

**VM changes:**
- Stack frames carry a `suppressed: bool` flag
- Stacktrace rendering respects the flag

**Files affected:**
- `proto/ish-ast/src/lib.rs` — annotation support
- `proto/ish-vm/src/interpreter.rs` — suppression flag on stack frames
- `proto/ish-vm/src/error.rs` — stacktrace rendering with suppression

### Decisions

**Decision:** Should suppressed frames still be visible in a verbose/debug mode?
--> Yes.  The function that prints a stacktrace should have a parameter that unsuppresses suppressed frames.

**Decision:** If a suppressed function has a context message, should the context still appear in the trace?
--> No. If you want the context, theny ou want the whole stack frame.

---

## Feature: Encumbered Error Handling (Union Return Types)

### Issues to Watch Out For

- **Transition pain.** The prompt says "encumbering existing streamlined code should be as painless as possible." If streamlined code uses throw/catch, encumbering it means changing every function signature to include error types in return unions, and every call site to handle the union. This could be a massive refactor.
- **Type inference support.** The compiler should be able to infer the error union type from the function body. If a function calls three other functions that can return `FileError | NetworkError | ParseError`, the compiler should infer the union automatically, rather than requiring the developer to list them all.
- **Error propagation syntax.** Rust uses `?` to propagate errors. Without an equivalent, encumbered error handling becomes extremely verbose. This is explicitly called out as an open question in the current docs.

### Critical Analysis

**Alternative A: Union return types with mandatory handling (as proposed)**
- Pros: Fully explicit. No hidden control flow. Compiler enforces completeness. Zero runtime overhead for stacktraces when disabled.
- Cons: Verbose without propagation syntax. Requires every variant to be handled at every call site.

**Alternative B: Checked exceptions (Java-style)**
- Pros: Familiar. Compiler enforces handling. Propagation is automatic (just declare `throws`).
- Cons: Widely considered a failure in Java — leads to catch-and-ignore or `throws Exception` everywhere.

**Alternative C: Union return types + `?` propagation operator**
- Pros: Explicit but concise. `let result = readFile(path)?;` — the `?` returns early if the value is an error variant. Type-safe propagation.
- Cons: Another operator to learn. Requires the function's return type to be a compatible union.

The prompt's design (throw = return, configurable return handler) naturally leads to Alternative A/C. The return handler in streamlined mode re-throws; in encumbered mode it's a no-op, leaving the union value for the caller to handle.

**Recommendation:** Alternative C — union return types with a `?` propagation operator. This gives encumbered code both safety and conciseness.

### Proposed Implementation

**Type system support** (mostly exists):
- Union types already specified in [docs/spec/types.md](../../spec/types.md)
- Error types participate in unions: `fn read(path: String) -> String | FileError | PermissionError`
- Type narrowing already specified for unions

**New syntax:**
- `?` operator: `let data = read(path)?;` — if `read` returns an error variant, propagate it as the return value of the enclosing function
- Exhaustive match required on union types when encumbered: `match result { String s -> ..., FileError e -> ... }`

**Compiler support:**
- When "errors thrown by function" is marked, the compiler infers the error union from the function body
- The `?` operator is desugared to a match + early return
- In compiled mode with no-op handlers, no stacktrace code is generated

**Files affected:**
- `proto/ish-ast/src/lib.rs` — `?` operator AST node
- `proto/ish-vm/src/interpreter.rs` — `?` operator execution
- `docs/spec/types.md` — error union type documentation
- `docs/spec/assurance-ledger.md` — "errors thrown by function" entry specification

### Decisions

**Decision:** Should the error propagation operator be `?` (Rust-style), `!` (something novel), or `try` (keyword prefix)?
--> `?` (Rust-style) syntax is semantic sugar for overriding encumberance and using the default error handler, which doesn't return; it throws.

**Decision:** Should the compiler infer error union types automatically, or must the developer declare them?
--> The compiler should infer error union types automatically.  There is a configuration option to mark error return types, forcing the developer to declare them.

---

## Feature: Configurable Error Handling Mechanism

This is the central mechanism that unifies streamlined and encumbered error handling. The prompt describes four configurable components:

1. **Return handler** — processes the return value of each function call
2. **Stackframe pusher** — part of the calling sequence; pushes a frame onto the stacktrace
3. **Stackframe popper** — part of the return sequence; pops the frame
4. **Stacktrace context parameter** — the stacktrace itself, threaded through calls

### Issues to Watch Out For

- **Configurability scope.** Where are these handlers configured? Per-project? Per-module? Per-function? The prompt says "configuration syntax is TBD." This is a critical design decision because it determines how encumbrance boundaries work.
- **Handler performance.** In the interpreteed case, every function call and return invokes these handlers. They must be very fast. In the compiled case, the compiler can inline or eliminate them.
- **Handler composability.** What if two modules want different return handlers? Which wins? Need clear precedence rules.
- **Break from convention.** This mechanism is novel — no mainstream language works this way. It needs exceptional documentation and tooling support so developers can understand what's happening.

### Critical Analysis

**Alternative A: Configurable handlers (as proposed)**
- Pros: Unified mechanism for streamlined and encumbered. Highly flexible. Zero-cost in compiled encumbered mode (no-op handlers eliminated). Enables custom error strategies beyond just "throw" or "return."
- Cons: Complex. Hard to explain. Debugging handler interactions could be nightmarish. Novel — no mainstream precedent.

**Alternative B: Two hardcoded modes — "exceptions" and "result types"**
- Pros: Simple. Well-understood. Each mode has a large existing ecosystem of knowledge.
- Cons: No middle ground. Can't customize. Binary choice doesn't fit the continuum.

**Alternative C: Configurable handlers, but with only two well-documented presets**
- Pros: Flexibility exists for power users. 99% of developers use one of two presets and never think about handlers. Documentation focuses on presets, not the mechanism.
- Cons: Power users may create handler configurations that are hard to understand or debug.

**Recommendation:** Alternative C. The handler mechanism is powerful and elegant, but most developers should never interact with it directly. Ship two presets ("streamlined" and "encumbered") and document them as the standard approaches. The handler mechanism is an advanced feature for framework authors and language experimenters.

### Proposed Implementation

**Context parameter mechanism:**
- `stacktrace` is a context parameter with the default value taken from the caller's stacktrace
- When a function is called, the stackframe pusher fires (default: push a frame onto the stacktrace)
- When a function returns, the stackframe popper fires (default: pop the frame)
- When a function call completes, the return handler processes the result (default: re-throw if error)

**Presets:**
- **Streamlined preset** (default): stackframe pusher pushes, popper pops, return handler re-throws errors
- **Encumbered preset**: stackframe pusher is no-op, popper is no-op, return handler is identity (pass-through)

**Configuration syntax (proposed):**
```
@error.mode("streamlined")   // or "encumbered"
module mymodule;
```
Or at the project level in a config file.

**Implementation order:**
1. Context parameters (language feature — needed first)
2. Stacktrace as a context parameter
3. Return handler mechanism
4. Stackframe pusher/popper
5. Presets
6. Configuration syntax

**Files affected:**
- `proto/ish-ast/src/lib.rs` — context parameters in function signatures
- `proto/ish-vm/src/interpreter.rs` — handler invocation on call/return
- `docs/spec/execution.md` — handler mechanism specification
- `docs/spec/assurance-ledger.md` — error mode as configurable feature

### Decisions

**Decision:** Should handlers be fully user-configurable, or should only the two presets be supported initially?
--> We will start with three presets: the two that are described and no-throw: stackframe pusher pushes, popper pops, return handler is identity (pass-through)

**Decision:** At what granularity can error mode be configured (project / module / function)?
--> All of the above. Encumberance configuration is still TBD, but it will be something in the category of annotations configurably at the project / module / function level

**Decision:** Should the handler mechanism be specified as part of the assurance ledger or as a separate execution concern?
--> As a separate execution concern. It is TBD whether the handler mechanism is an implementation detail or is exposed as part of the public interface.  But we should keep it as a hidden implementation detail until forced to do otherwise.

---

## Feature: Context Parameters

This is a prerequisite for the error handling mechanism. Context parameters are function parameters whose default value comes from the caller's stack frame.

### Issues to Watch Out For

- **Implicit data flow.** Context parameters create invisible data flow between functions. A function's behavior depends on values it didn't explicitly receive. This is powerful but can make code hard to reason about.
- **Interaction with compiled mode.** In compiled mode, the compiler must thread context parameters through the call graph. If a deeply nested function uses `stacktrace`, every function in the call chain must pass it through — unless the compiler can prove it's unused and eliminate it.
- **Not entirely novel.** Scala has implicit parameters. Kotlin has context receivers. These provide precedent but also cautionary tales about complexity.

### Critical Analysis

**Alternative A: Context parameters (as proposed)**
- Pros: Clean mechanism for threading state (like stacktraces) through call chains without polluting every signature. Enables the configurable handler mechanism.
- Cons: Implicit state. Hard to trace data flow. Compiler complexity.

**Alternative B: Thread-local / global state**
- Pros: Simpler implementation. No parameter threading needed.
- Cons: Not composable. Breaks with async/concurrent code. Can't have different stacktraces in different call chains.

**Alternative C: Explicit parameters only**
- Pros: Maximum clarity. No hidden state.
- Cons: Every function in a call chain must explicitly pass the stacktrace. Massive boilerplate.

**Recommendation:** Alternative A. Context parameters are the right abstraction for this use case. The key mitigation for the "implicit data flow" concern is that context parameters should be limited in scope — only a small set of well-known context parameters (like `stacktrace`) should exist, and user-defined context parameters should be clearly documented.

### Proposed Implementation

**Syntax:**
```
fn myFunction(a: i32, context stacktrace: StackTrace = caller.stacktrace) {
    // stacktrace is available here
}
```
The `context` keyword marks a parameter as a context parameter. The default value expression `caller.stacktrace` refers to the caller's context.

**Simplified everyday use:**
Most developers never write context parameters explicitly. The `stacktrace` context parameter is automatically present in every function (unless suppressed). Developers interact with it via `stacktrace.pushCtx()` and similar APIs.

**Files affected:**
- `proto/ish-ast/src/lib.rs` — context parameter in function parameter lists
- `proto/ish-vm/src/interpreter.rs` — context parameter resolution during function calls
- `docs/spec/types.md` — context parameter type documentation

### Decisions

**Decision:** Should user-defined context parameters be supported, or only built-in ones like `stacktrace`?
--> Built ins only for now.  It is important to note that context parameters are intended primarily as a crutch for the interpreter to support lexical scope.

**Decision:** What is the syntax for context parameters?
--> For the moment, there is no syntax for developers to interact directly with context parameters. They are all reached with syntactic sugar.

---

## Implementation Sequencing

The features described in this proposal have dependencies. The recommended implementation order is:

1. **Error type and throw/try/catch/finally** — foundational; everything else builds on this
2. **Stacktrace representation** — needed for context pushing and suppression
3. **Context parameters** — the mechanism that enables configurable handlers
4. **Stack trace context pushing** — uses context parameters
5. **Stack frame suppression** — uses stacktrace representation
6. **`with` blocks and `defer`** — independent of error configuration, but uses try/finally internally
7. **Return handler mechanism** — the configurable handler approach
8. **Encumbered error mode** — built on return handlers and union types
9. **Error propagation operator (`?`)** — requires union types and encumbered mode
10. **Configuration syntax** — ties it all together

Steps 1-2 can be prototyped immediately. Steps 3-6 can proceed in parallel once 1-2 are done. Steps 7-10 are the encumbered story and can follow.

---

## Documentation Updates

The following documentation files will be affected by this proposal:

- [docs/user-guide/error-handling.md](../../user-guide/error-handling.md) — **complete rewrite** needed; currently contradicts the proposed throw/catch model
- [docs/spec/types.md](../../spec/types.md) — Error type specification, union types for error results
- [docs/spec/assurance-ledger.md](../../spec/assurance-ledger.md) — checked exceptions entry, error mode presets
- [docs/spec/execution.md](../../spec/execution.md) — handler mechanism (return handler, stackframe pusher/popper)
- [docs/spec/syntax.md](../../spec/syntax.md) — syntax for throw/try/catch/finally/with/defer/context parameters
- [docs/spec/reasoning.md](../../spec/reasoning.md) — `might_throw` proposition, interaction with error types
- [docs/errors/INDEX.md](../../errors/INDEX.md) — begin populating the error catalog
- [docs/project/open-questions.md](../../project/open-questions.md) — close resolved questions, add new ones from Decisions sections
- [docs/architecture/ast.md](../../architecture/ast.md) — new AST nodes for error handling constructs
- [docs/architecture/vm.md](../../architecture/vm.md) — interpreter changes for handlers and control flow
- [GLOSSARY.md](../../../GLOSSARY.md) — new terms: context parameter, return handler, stackframe pusher, stackframe popper, suppressed error, defer

Remember to update `## Referenced by` sections in all affected files.

--> Consider adding new documentation sections docs/spec/errors.md and/or docs/architecture/errors.md, and possibly others.
---

## History Updates

- [ ] Add `docs/project/history/2026-03-11-error-handling.md` after decisions are made and implementation begins
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
