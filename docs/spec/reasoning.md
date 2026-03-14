---
title: ish Reasoning System
category: spec
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/types.md, docs/spec/assurance-ledger.md]
---

# ish Reasoning System

## Overview

Several parts of the ish system need to reason about code: the compiler, the static code analyzer, and the LSP server. Rather than each component implementing its own analysis logic, ish provides a single shared reasoning tool that services all of them.

## What the Reasoning Tool Analyzes

The reasoning tool answers questions about code such as:

1. Is a statement reachable?
2. Is a variable mutated?
3. Does a block of code throw errors?
4. Is a variable guaranteed to be initialized?

## Exposing the Reasoning Tool to the Language

It is proposed to expose the reasoning tool's interface to the language itself. This would allow developers to annotate their code with arbitrary assertions and queries, which the language processor would evaluate. It would also support plugin developers who want to extend the analyzer.

## Propositions

The building blocks of the reasoning system are logic propositions. Two kinds are supported:

### Atomic Propositions

Atomic propositions are the primitives of the reasoning system. Each is defined by a plugin — an ish function that takes an AST node (and perhaps some state) as input and returns a boolean result. The language provides an interface for defining new atomic proposition plugins.

An initial set of built-in atomic propositions includes:

1. Statement reachable
2. Variable mutated
3. Block might throw
4. Variable guaranteed to be initialized

### Compound Propositions

Compound propositions are formed by applying logical operations (`and`, `or`, `not`) to atomic or other compound propositions.

## Relationship to the Type System

One possibility is to implement the entire type system on top of this reasoning tool. The interfaces of the type system would be unified with those of the reasoning tool — effectively, the type system would be a special case of a more general facility for reasoning about code.

---

## Proposed Interfaces

### Atomic Proposition Plugin Interface

```
fn proposition_name(node: AstNode, context: ReasoningContext) -> bool

interface ReasoningContext {
    symbol_table: SymbolTable,
    query(prop: Proposition) -> bool,
    parent_node: AstNode?,
    assurance_level: AssuranceLevel,
}
```

### Proposition Annotation Syntax (Strawman)

```
@reason.assert(reachable)
let x = compute();

@reason.query(might_throw)
do_something();

@reason.assert(initialized(x) and not mutated(y))
let z = x + y;

@reason.assert(not might_throw) {
    parse(data);
    validate(data);
}
```

### Plugin Registration (Strawman)

```
@reason.plugin("my_custom_check")
export fn my_custom_check(node: AstNode, ctx: ReasoningContext) -> bool {
    // ... analysis logic ...
}
```

### Type System Integration Interface (Strawman)

```
@reason.assert(assignable_to(x, i32))
let y: i32 = x;
```

---

## Example Use Cases

### Compile-Time Safety Assertions

```
@reason.assert(not might_throw)
fn transfer_funds(from: Account, to: Account, amount: f64) {
    from.balance -= amount;
    to.balance += amount;
}
```

### Dead Code Detection

```
fn process(input: Input) {
    if (input.kind == "a") {
        handle_a(input);
    } else if (input.kind == "b") {
        handle_b(input);
    } else {
        @reason.assert(reachable)
        handle_unknown(input);
    }
}
```

### Custom Domain-Specific Analysis

```
@reason.plugin("sql_safe")
export fn sql_safe(node: AstNode, ctx: ReasoningContext) -> bool { ... }

@reason.assert(sql_safe)
let query = build_query(user_input);
db.execute(query);
```

---

## Open Questions

Open questions for the reasoning system. See also [docs/project/open-questions.md](../project/open-questions.md#reasoning-system) for a consolidated view.

### Plugin Interface

- [ ] **Atomic proposition plugin signature.** What is the type of the AST node parameter? What is "some state"? Can a plugin access the results of other propositions? Can a plugin be stateful across multiple invocations?

### Annotation Syntax

- [ ] **How developers annotate code.** Are annotations attributes/decorators, inline expressions, or special comments? Where can annotations appear? How does a developer distinguish between an assertion and a query?

### Interaction with Assurance Levels

- [ ] **How the reasoning system varies with assurance level.** In low-assurance mode, are annotations ignored, deferred to runtime, or evaluated? Can the level of reasoning strictness be independently configured?

### Interaction with the Assurance Ledger

- [ ] **Relationship to assurance ledger checks.** Are ledger entries implemented as reasoning propositions? How are pre-conditions and post-conditions expressed?

### Compound Proposition Semantics

- [ ] **Logical operations beyond `and` / `or`.** Is implication supported? Can propositions be quantified? Can propositions reference specific variables?

### Error Reporting

- [ ] **What happens when a proposition is violated.** Error message format? Custom error messages from plugins? Explanation of *why* a proposition failed?

### Plugin Registration and Discovery

- [ ] **How plugins are registered.** Declarative or imperative? Plugin scope? Naming conflicts? Can third-party packages ship plugins?

### Proposition Evaluation Order and Fixpoints

- [ ] **Evaluation model.** Fixed order or fixpoint computation? How are circular dependencies between propositions handled? Lazy or eager evaluation?

### Scope of Reasoning

- [ ] **Inter-procedural vs. intra-procedural.** Can a proposition assert something about a called function's behavior? How does inter-procedural analysis interact with separate compilation?

### Implementation Challenges

- [ ] **Performance.** Bounding analysis time with extensible plugins.
- [ ] **Soundness.** Policy for undecidable properties. Over-approximate vs. under-approximate.
- [ ] **Plugin sandboxing.** Termination guarantees. Side effect restrictions.
- [ ] **Bootstrapping.** Boundary between built-in propositions (Rust) and plugins (ish).
- [ ] **LSP integration.** Incremental analysis strategy. Partial results during editing.

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
