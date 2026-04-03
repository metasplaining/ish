---
title: ish Glossary
category: project
audience: [all]
status: draft
last-verified: 2026-03-18
depends-on: []
---

# ish Glossary

Canonical definitions of ish-specific terminology. All documentation must use these terms consistently.

---

| Term | Definition |
|------|-----------|
| **Annotated AST** | A parsed and analyzed syntax tree that preserves all metadata. One of the package encoding formats. |
| **Assurance Ledger** | The unified consistency-checking system in ish. Standards configure what the ledger checks within a scope; entries record facts about individual items; the audit detects discrepancies between entries. See [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md). |
| **Assurance level** | The degree to which a developer must explicitly address language concerns (types, memory, polymorphism, etc.). Configurable per-project, per-module, per-function, or per-variable. Replaces the former term "encumbrance." |
| **Async function** | A function declared with `async fn` that can yield (suspend execution cooperatively). At low assurance, async declaration is optional — the runtime infers it. At high assurance, the `async` keyword is required. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Atomic proposition** | A primitive of the reasoning system — an ish function that inspects an AST node and returns a boolean. See [docs/spec/reasoning.md](docs/spec/reasoning.md). |
| **Audit** | The process by which the assurance ledger checks entries for discrepancies. Can be *pre-audit* (build time) or *live audit* (execution time), depending on the active standard's feature states. |
| **Authority order** | The defined sequence in which project artifacts must be updated during implementation, from most authoritative (glossary, roadmap) to least authoritative (index files). |
| **Await** | The `await` keyword that suspends the caller until an awaited future resolves. Makes the caller yielding. At low assurance, await is implicit when calling async functions. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Bare-word invocation** | In shell mode, a line that does not start with a recognized language keyword is parsed as a command invocation. The first word is treated as a command name, and subsequent words are arguments. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Byte buffer** | A single ish type (`ByteBuffer`) representing a contiguous sequence of bytes. Abstracts over Rust's `&[u8]`/`Bytes`/`BytesMut`. Mutability is expressed via the `Mutable` ledger entry. Immutable buffers support O(1) cloning and slicing; mutable buffers support efficient in-place mutation. Freezing (mutable → immutable) is O(1). See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Char literal** | A character value written as `c'A'`. Supports escape sequences: `c'\n'`, `c'\0'`, `c'\u{XXXX}'`. Produces a `Literal::Char` in the AST and a `Value::Char` in the VM. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Codec** | A component that encodes or decodes a stream of bytes into structured values. Used with `Stream<ByteBuffer>` combinators like `.decode(codec)` and `.lines()`. Backed by Tokio's codec system (`tokio_util::codec`). See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **CodedError** | An entry type that extends `Error` and additionally requires a `code: String` property. `CodedError` entries represent errors with machine-readable error codes. See [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md). |
| **Code analyzer** | The component that analyzes ASTs and annotates them with metadata (types, reachability, etc.). |
| **Command substitution** | The `$(cmd)` syntax for capturing the output of a shell command into a variable or expression. Works in both shell mode and programming mode. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Completion signal** | A message sent from the main thread to the shell thread indicating that a submitted task has finished. The completion signal carries no display content — all output is routed through the `ExternalPrinter`. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Compiled function** | An `IshFunction` with a `Compiled(Shim)` implementation instead of an `Interpreted(Statement)` body. Builtins and future stdlib functions are compiled functions. See [docs/architecture/vm.md](docs/architecture/vm.md). |
| **Complexity entry** | A ledger entry type describing whether a block completes quickly (`simple`) or may take a long time (`complex`). Applies to functions and blocks. `complex + unyielding` is a discrepancy when `guaranteed_yield` is enabled. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Compound proposition** | A proposition formed by combining atomic propositions with logical operators (`and`, `or`, `not`). |
| **Concurrent iteration** | Running multiple iterations of a loop body as concurrent tasks on the `LocalSet`. Provided by the `concurrent_map` and `concurrent_for_each` standard library functions, not by syntax. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Context parameter** | A parameter to a function or block that is provided implicitly by the runtime rather than passed explicitly by the caller. Used for stack trace context and other execution metadata. Currently limited to built-ins. |
| **Cooperative multitasking** | A concurrency model where tasks voluntarily yield control at defined points, allowing other tasks to run. In ish, implemented via `async`/`await` on a Tokio `LocalSet`. Contrast with parallel multitasking. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Defer** | A statement that schedules cleanup code to run when the enclosing block exits. Multiple defers execute in LIFO (last-in, first-out) order. See [docs/user-guide/error-handling.md](docs/user-guide/error-handling.md). |
| **Decision register** | A consolidated list of all accepted decisions in a design proposal. Maintained at the top of the proposal as the authoritative reference. |
| **Design history** | Narrative of the deliberation process: what was proposed, what alternatives were considered, what was decided, and by whom. Stored as a directory per proposal under `docs/project/history/`, with a summary file and separate files for each version. |
| **Design phase** | The iterative process of creating and refining a design proposal from an RFP. Ends when the human accepts the design proposal. |
| **Design proposal** | Analysis of an RFP with alternatives, pros/cons, recommendations, and decisions. Output of the design process. May go through multiple iterations. |
| **Discrepancy** | A conflict detected by the assurance ledger — two entries on the same item are incompatible, or a required entry is missing. Replaces the former term "agreement violation." |
| **Domain error subtype** | A specialized entry type that extends `Error`, `CodedError`, or `SystemError` to categorize errors by domain (e.g., `FileError`, `TypeError`, `ArgumentError`). Each domain subtype maps to specific well-known error codes. See [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md). |
| **Entry** | A fact recorded about an item (variable, property, function, type) in the assurance ledger. For example, `@[type(i32)]` or `@[mutable]`. Entries can be created by native syntax (e.g., `mut`, `: i32`) or by explicit annotation (`@[entry(params)]`). |
| **Entry type** | A definition of a kind of entry that can be recorded in the assurance ledger. Built-in entry types include `Type`, `Mutable`, `Open`, `Closed`, `Error`, `CodedError`, and `SystemError`. The error entry types form a hierarchy: `Error` ← `CodedError` ← `SystemError`, with domain subtypes (e.g., `FileError`, `TypeError`). Custom entry types are defined with `entry type name { ... }`. |
| **Execution configuration** | One of the four modes in which ish runs: thin shell, fat shell, compiler, or executable. See [docs/spec/execution.md](docs/spec/execution.md). |
| **Extended delimiter string** | A string wrapped with `~` delimiters to avoid escaping: `~"..."~`, `~'...'~`, `~"""..."""~`, `~'''...'''~`. Content inside extended delimiters is verbatim — no escape processing. Not available in shell mode. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **ExternalPrinter** | Reedline's API for injecting output into the terminal while the prompt is active. In interactive mode, all program output (expression results, `println`, errors, background task output) is routed through the `ExternalPrinter` to prevent prompt corruption. See [docs/spec/concurrency.md](docs/spec/concurrency.md). | `~"..."~`, `~'...'~`, `~"""..."""~`, `~'''...'''~`. Content inside extended delimiters is verbatim — no escape processing. Not available in shell mode. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Fat shell** | The execution configuration where modules have been imported, increasing memory footprint but enabling compiled-speed execution of imported code. |
| **Feature coherence audit** | A cross-check of all project artifacts related to a single feature, verifying consistency. |
| **Feature state** | The configuration of a single feature within a standard. Feature states use separate dimensions: `type_annotations` (`optional` \| `required`), `type_audit` (`runtime` \| `build`), and feature-specific states. See [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md). |
| **Force-command prefix** | The `>` prefix in shell mode that forces a line to be parsed as a command invocation, even if it starts with a recognized language keyword. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Function implementation** | The execution strategy for a function, described by the `FunctionImplementation` enum: `Interpreted(Statement)` for tree-walking execution, or `Compiled(Shim)` for synchronous shim dispatch. See [docs/architecture/vm.md](docs/architecture/vm.md). |
| **Future** | A value of type `Future<T>` representing an eventual result. Created by `spawn`, which starts an async operation and returns a `Future` without suspending. Must be awaited to obtain the result. Dropping a future cancels the underlying task. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **High-assurance ish** | The end of the continuum where code is heavily annotated, compiled, and statically checked. Sits at the intersection of Rust and TypeScript. Formerly called "encumbered ish." |
| **Implementation phase** | The process of executing an implementation plan. Ends when all TODO items are complete. |
| **Implementation plan** | Consolidated, authoritative document derived from the accepted design proposal. Contains the TODO list and file-by-file changes. Input to the implementation process. Stored in `docs/project/plans/`. |
| **Implied await** | When the `await_required` assurance feature is not active, the interpreter automatically awaits a `Future` returned by a bare function call (no explicit `await`/`spawn`). Preserves backward compatibility for parallel builtins at low assurance. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Indeterminate** | The default openness state of a type declaration — neither explicitly open nor explicitly closed. An indeterminate type can be narrowed to open or closed via `@[open]` or `@[closed]` annotations. Object literals, by contrast, default to closed. |
| **Interpolated string** | A double-quoted string (`"..."`) that supports `{expr}` ish expression interpolation and `$VAR` environment variable expansion. Also called an interpolating string. Contrast with literal string. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Intersection type** | A type that represents a value satisfying all of several types simultaneously. Written with `&` (e.g., `Named & Aged`). The intersection of incompatible types produces `never`. For objects, intersection merges property sets; for primitives, incompatible intersections collapse to `never`. See [docs/spec/types.md](docs/spec/types.md). |
| **Literal string** | A single-quoted string (`'...'`) with no interpolation and minimal escapes (`\\` and `\'` only). Also the content model for `'''...'''` triple-quoted strings. Contrast with interpolated string. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Literal type** | A type containing exactly one value (e.g., the type `5` contains only the value `5`). |
| **Live audit** | Audit that occurs at execution time. Features set to `live` in the active standard are checked during live audit. |
| **LocalSet** | A Tokio primitive that runs futures on a single thread without requiring the `Send` trait. All ish user code runs on a `LocalSet` on the main thread. Tasks are spawned with `spawn_local`. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Low-assurance ish** | The end of the continuum where code is minimal, interpreted, and dynamically checked. Approachable by anyone who knows another programming language. Formerly called "streamlined ish." |
| **Main thread** | The thread that runs the Tokio runtime with a `LocalSet`, containing the VM, all `Value` objects, the `Environment`, and GC-managed state. In interactive mode, receives AST submissions from the shell thread and executes them as tasks. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Module** | A unit of code organization. Every `.ish` file defines a module. The module path mirrors the file path. See [docs/spec/modules.md](docs/spec/modules.md). |
| **Nominal typing** | A typing mode where compatibility requires that types be declared as related, not merely that they have the same shape. Contrast with structural typing. |
| **Package** | The distributable artifact produced by building a project. Can be encoded as annotated AST, static object code, or dynamic object code. |
| **Parallel entry** | A ledger entry type that marks a function as executing on a separate Tokio thread pool thread. Implies `@[yielding]`. Functions with this entry are implemented in Rust via parallel shims; ish developers cannot define parallel functions. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Parallel multitasking** | True multi-threaded execution on Tokio's thread pool via `tokio::spawn`. Restricted to Rust-implemented standard library functions via parallel shims. ish user code cannot run in parallel — only concurrently on the `LocalSet`. Contrast with cooperative multitasking. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Parallel shim** | A shim for a parallel builtin or stdlib function. Marshals arguments into `Send`-safe form, spawns work via `spawn_blocking`, uses a `spawn_local` bridge to convert native results back to `Value`, and returns `Value::Future`. Part of the unified `Shim` type — not a separate variant. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Planning phase** | The process of generating an implementation plan from an accepted design proposal. Typically one step, not iterative. |
| **Programming mode** | The default parsing mode for `.ish` source files, where all lines are parsed as language statements. Contrast with shell mode. |
| **Polymorphism strategy** | The implementation technique used for polymorphic dispatch: none, enumerated, monomorphized, virtual method table, or associative array. Normally chosen automatically by the language processor. |
| **Pre-audit** | Audit that occurs at build time (declaration time). Features set to `pre` in the active standard are checked during pre-audit. |
| **Prompt** | Raw input from the human, before any cleanup. Not a project artifact. |
| **Proposition** | A logical assertion about code that can be evaluated by the reasoning system. Can be atomic or compound. |
| **Punch list** | A list of corrections, additions, or changes that the human wants made to a design proposal before accepting it. May be delivered as a separate document or as inline decisions in the proposal itself. |
| **Reader** | An ish type that wraps a Tokio `AsyncRead` source, providing byte-level read access. Methods include `read(n)`, `read_all()`, `stream()`, `seek()`, and `close()`. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Reasoning system** | A shared tool that answers questions about code (reachability, mutation, initialization, etc.). Services the compiler, analyzer, and LSP server. See [docs/spec/reasoning.md](docs/spec/reasoning.md). |
| **Return handler** | A hidden mechanism that intercepts function returns to manage error propagation and stack trace construction. An execution concern, not directly accessible to users. |
| **Shell mode** | The default parsing mode for the interactive shell, where bare-word lines are parsed as command invocations and lines starting with recognized keywords are parsed as language statements. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Shell thread** | In interactive mode, the thread that runs Reedline, collects input, parses it via the ish parser, and submits the resulting `Program` AST to the main thread via a channel. Responsible only for prompts and command line input — never displays program output. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Shim function** | A synchronous Rust closure (`Fn(&[Value]) -> Result<Value, RuntimeError>`) that receives already-validated arguments, performs the function's work (or spawns it), and returns a `Value` (which may be `Value::Future` for yielding/parallel functions). In the compiled function architecture, all builtins are implemented as shims. Also the entry point in a dynamically linked package that marshals calls between the ish runtime and compiled library code. See [docs/architecture/vm.md](docs/architecture/vm.md). |
| **Spawn** | The `spawn` keyword that starts an async operation and returns a `Future<T>` immediately without suspending. The spawned task runs on the `LocalSet` as a `spawn_local` task. Calling `spawn` does not make the caller yielding. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Standard** | A named configuration that sets feature states within a scope. Applied with `@standard[name]`. Defined with `standard name [...]` or `standard name extends base [...]`. Built-in standards include `streamlined`, `cautious`, and `rigorous`. |
| **Stream** | A type `Stream<T>` representing an async sequence of values. `Stream<ByteBuffer>` uses efficient byte-oriented I/O backed by `AsyncRead`/`AsyncWrite`; `Stream<T>` for other types uses channels. Supports combinators like `map`, `filter`, `take`, `zip`, `lines`, and `decode`. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Stream writer** | A type `StreamWriter<T>` representing the write half of a stream — the producer side that sends values into a stream. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Structural typing** | The default typing mode where two types are compatible if they have the same shape (property names and compatible property types), regardless of declaration. |
| **Suppressed error** | An error whose stack frame has been hidden from the default stack trace display. Suppressed frames are still available in verbose/debug mode. Controlled via annotations. |
| **SystemError** | An entry type that extends `CodedError`. `SystemError` entries represent errors generated by the ish runtime itself, with well-known error codes (e.g., `E001`, `E002`). Contrast with user-created `Error` and `CodedError` entries. See [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md). |
| **Thin shell** | The minimal execution configuration — command-line input is parsed and interpreted immediately with no modules loaded. |
| **Triple-quoted string** | A multiline string using `"""..."""` (interpolating) or `'''...'''` (literal). Supports automatic indentation stripping based on the closing delimiter's position. See [docs/spec/syntax.md](docs/spec/syntax.md). |
| **Type widening** | The process by which a literal type is generalized to a broader type (e.g., `5` → `i32`). Rules vary with assurance level. |
| **Type narrowing** | The process by which the ledger maintains refined entry sets through control flow. For example, after an `is_type()` check, the type entry on a variable is narrowed to the checked type within the true branch. Entries are restored or merged after branches converge. See [docs/spec/types.md](docs/spec/types.md). |
| **Value entry** | An entry that records information about a value's type or constraints. There are three kinds: *actual-value entries* (the concrete runtime value, tracked during live audit), *possible-values entries* (the set of values across execution paths, tracked during pre-audit), and *allowed-values entries* (explicit constraints on permitted values, from type annotations). See [docs/spec/assurance-ledger.md](docs/spec/assurance-ledger.md). |
| **With block** | A statement that manages resources requiring cleanup. Initializes resources, executes a body, then closes resources in reverse order on exit. See [docs/user-guide/error-handling.md](docs/user-guide/error-handling.md). |
| **Writer** | An ish type that wraps a Tokio `AsyncWrite` sink, providing byte-level write access. Methods include `write(data)`, `flush()`, and `close()`. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Yield** | The `yield` keyword that explicitly suspends execution, returning control to the Tokio scheduler. A yield-eligible point where the runtime can switch to other tasks. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Yield budget** | A configurable time quantum (default ~1ms) that determines when the runtime inserts automatic yields. At each yield-eligible point, the runtime checks whether the budget is exhausted. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Yield-eligible point** | A location in code where the runtime may insert an automatic yield: loop back-edges (`for`, `while`), function call sites (before the call), and explicit `yield` statements. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |
| **Yielding entry** | A ledger entry type describing whether a block can suspend execution. Values are `yielding` (can suspend) or `unyielding` (never suspends). A function is `yielding` if it contains `await` or `yield`, or calls a `yielding` function with `await`. Distinct from the `async` keyword — a function can be `yielding` without being declared `async` at low assurance. See [docs/spec/concurrency.md](docs/spec/concurrency.md). |

---

## Referenced by

- [AGENTS.md](AGENTS.md)
- [docs/INDEX.md](docs/INDEX.md)
