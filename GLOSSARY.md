---
title: ish Glossary
category: project
audience: [all]
status: draft
last-verified: 2026-03-11
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
| **Atomic proposition** | A primitive of the reasoning system — an ish function that inspects an AST node and returns a boolean. See [docs/spec/reasoning.md](docs/spec/reasoning.md). |
| **Audit** | The process by which the assurance ledger checks entries for discrepancies. Can be *pre-audit* (build time) or *live audit* (execution time), depending on the active standard's feature states. |
| **Code analyzer** | The component that analyzes ASTs and annotates them with metadata (types, reachability, etc.). |
| **Compound proposition** | A proposition formed by combining atomic propositions with logical operators (`and`, `or`, `not`). |
| **Context parameter** | A parameter to a function or block that is provided implicitly by the runtime rather than passed explicitly by the caller. Used for stack trace context and other execution metadata. Currently limited to built-ins. |
| **Defer** | A statement that schedules cleanup code to run when the enclosing block exits. Multiple defers execute in LIFO (last-in, first-out) order. See [docs/user-guide/error-handling.md](docs/user-guide/error-handling.md). |
| **Discrepancy** | A conflict detected by the assurance ledger — two entries on the same item are incompatible, or a required entry is missing. Replaces the former term "agreement violation." |
| **Entry** | A fact recorded about an item (variable, property, function, type) in the assurance ledger. For example, `@[type(i32)]` or `@[mutable]`. Entries can be created by native syntax (e.g., `mut`, `: i32`) or by explicit annotation (`@[entry(params)]`). |
| **Entry type** | A definition of a kind of entry that can be recorded in the assurance ledger. Built-in entry types include `type`, `mutable`, `overflow`, `throws`. Custom entry types are defined with `entry type name { ... }`. |
| **Execution configuration** | One of the four modes in which ish runs: thin shell, fat shell, compiler, or executable. See [docs/spec/execution.md](docs/spec/execution.md). |
| **Fat shell** | The execution configuration where modules have been imported, increasing memory footprint but enabling compiled-speed execution of imported code. |
| **High-assurance ish** | The end of the continuum where code is heavily annotated, compiled, and statically checked. Sits at the intersection of Rust and TypeScript. Formerly called "encumbered ish." |
| **Literal type** | A type containing exactly one value (e.g., the type `5` contains only the value `5`). |
| **Live audit** | Audit that occurs at execution time. Features set to `live` in the active standard are checked during live audit. |
| **Low-assurance ish** | The end of the continuum where code is minimal, interpreted, and dynamically checked. Approachable by anyone who knows another programming language. Formerly called "streamlined ish." |
| **Module** | A unit of code organization. Every `.ish` file defines a module. The module path mirrors the file path. See [docs/spec/modules.md](docs/spec/modules.md). |
| **Nominal typing** | A typing mode where compatibility requires that types be declared as related, not merely that they have the same shape. Contrast with structural typing. |
| **Package** | The distributable artifact produced by building a project. Can be encoded as annotated AST, static object code, or dynamic object code. |
| **Polymorphism strategy** | The implementation technique used for polymorphic dispatch: none, enumerated, monomorphized, virtual method table, or associative array. Normally chosen automatically by the language processor. |
| **Pre-audit** | Audit that occurs at build time (declaration time). Features set to `pre` in the active standard are checked during pre-audit. |
| **Proposition** | A logical assertion about code that can be evaluated by the reasoning system. Can be atomic or compound. |
| **Reasoning system** | A shared tool that answers questions about code (reachability, mutation, initialization, etc.). Services the compiler, analyzer, and LSP server. See [docs/spec/reasoning.md](docs/spec/reasoning.md). |
| **Return handler** | A hidden mechanism that intercepts function returns to manage error propagation and stack trace construction. An execution concern, not directly accessible to users. |
| **Shim function** | The entry point in a dynamically linked package that marshals calls between the ish runtime and compiled library code. |
| **Standard** | A named configuration that sets feature states within a scope. Applied with `@standard[name]`. Defined with `standard name [...]` or `standard name extends base [...]`. Built-in standards include `streamlined`, `cautious`, and `rigorous`. |
| **Structural typing** | The default typing mode where two types are compatible if they have the same shape (property names and compatible property types), regardless of declaration. |
| **Suppressed error** | An error whose stack frame has been hidden from the default stack trace display. Suppressed frames are still available in verbose/debug mode. Controlled via annotations. |
| **Thin shell** | The minimal execution configuration — command-line input is parsed and interpreted immediately with no modules loaded. |
| **Type widening** | The process by which a literal type is generalized to a broader type (e.g., `5` → `i32`). Rules vary with assurance level. |
| **With block** | A statement that manages resources requiring cleanup. Initializes resources, executes a body, then closes resources in reverse order on exit. See [docs/user-guide/error-handling.md](docs/user-guide/error-handling.md). |

---

## Referenced by

- [AGENTS.md](AGENTS.md)
- [docs/INDEX.md](docs/INDEX.md)
