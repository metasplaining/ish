# ish Reasoning System — Outstanding Issues

Remaining open questions, proposed interfaces, implementation challenges, example use cases, and an overall evaluation of the reasoning system described in REASONING.md.

---

## 1. Undefined or Under-Specified Aspects

### 1.1 Plugin Interface

- [ ] **Atomic proposition plugin signature.** The spec says plugins are "ish functions which take an AST node and perhaps some state as input and return a boolean result." This needs to be made precise:
  - What is the type of the AST node parameter? Is it a single node, a subtree, or the entire AST?
  - What is "some state"? Is it a mutable context object? An immutable snapshot of the analysis so far? A symbol table?
  - Can a plugin access the results of other propositions, or only inspect the raw AST?
  - Can a plugin be stateful across multiple invocations (e.g., accumulating information across an entire module)?

### 1.2 Annotation Syntax

- [ ] **How developers annotate code.** The spec says developers can "annotate their code with arbitrary assertions and queries." No syntax is proposed:
  - Are annotations attributes/decorators (e.g., `@assert(reachable)`)? Inline expressions? Special comments?
  - Where can annotations appear — on statements, expressions, declarations, blocks?
  - How does a developer distinguish between an assertion (must hold or the build fails) and a query (report the result without failing)?

### 1.3 Interaction with Encumbrance

- [ ] **How the reasoning system varies with encumbrance level.** The README describes a streamlined ↔ encumbered continuum, but REASONING.md does not address it:
  - In streamlined mode, are reasoning annotations ignored, deferred to runtime, or still evaluated?
  - In encumbered mode, can the developer require that certain propositions hold at build time?
  - Can the level of reasoning strictness be independently configured, like other encumbrance features?

### 1.4 Interaction with the Agreement System

- [ ] **Relationship to agreement checks.** The agreement system is now defined in [AGREEMENT.md](AGREEMENT.md) — agreement is a linguistic concept where marked language features must be present and consistent. Reasoning propositions and agreement checks seem closely related:
  - Are agreements implemented as reasoning propositions? (e.g., is type agreement a proposition like `assignable_to(expr, Type)`?)
  - Is `assert(reachable)` an agreement, a reasoning annotation, or both?
  - How are pre-conditions and post-conditions expressed in terms of propositions?
  - When a marked feature is present, does the reasoning system enforce the agreement check?

### 1.5 Compound Proposition Semantics

- [ ] **Logical operations beyond `and` / `or`.** The spec mentions `and` and `or`. Other important operations are not addressed:
  - Is `not` supported?
  - Is implication (`implies`) supported, or must it be expressed as `not(A) or B`?
  - Can propositions be quantified (e.g., "for all variables in this scope, X holds")?
  - Can propositions reference specific variables or expressions, or only the annotated AST node?

### 1.6 Error Reporting

- [ ] **What happens when a proposition is violated.** The spec does not describe the developer experience when a reasoning assertion fails:
  - What does the error message look like?
  - Can plugins provide custom error messages?
  - Does the system explain *why* a proposition failed (e.g., showing the code path that makes a block reachable)?

### 1.7 Plugin Registration and Discovery

- [ ] **How plugins are registered.** "The language provides an interface for defining new atomic proposition plugins" — but:
  - Is registration declarative (e.g., a special export or annotation on a function) or imperative (calling a registration API)?
  - Are plugins scoped to a module, a project, or global?
  - How are naming conflicts between plugins resolved?
  - Can third-party packages ship reasoning plugins?

### 1.8 Proposition Evaluation Order and Fixpoints

- [ ] **Evaluation model.** Some propositions depend on others (e.g., reachability depends on mutation analysis). The evaluation strategy is not described:
  - Are propositions evaluated in a fixed order, or does the system compute a fixpoint?
  - How are circular dependencies between propositions handled?
  - Is evaluation lazy or eager?

### 1.9 Scope of Reasoning

- [ ] **Inter-procedural vs. intra-procedural.** The spec does not state whether reasoning is limited to a single function body or can span call boundaries:
  - Can a proposition assert something about a called function's behavior (e.g., "this function never throws")?
  - How does inter-procedural analysis interact with separate compilation?

---

## 2. Proposed Interfaces

### 2.1 Atomic Proposition Plugin Interface

```
// A plugin is an ish function with this signature:
fn proposition_name(node: AstNode, context: ReasoningContext) -> bool

// ReasoningContext provides access to analysis state:
interface ReasoningContext {
    symbol_table: SymbolTable,          // Variable bindings and types in scope
    query(prop: Proposition) -> bool,   // Evaluate another proposition
    parent_node: AstNode?,              // Enclosing AST node
    encumbrance: EncumbranceLevel,      // Current encumbrance configuration
}
```

### 2.2 Proposition Annotation Syntax (Strawman)

```
// Assertion — build fails if the proposition does not hold
@reason.assert(reachable)
let x = compute();

// Query — reports the result without failing
@reason.query(might_throw)
do_something();

// Compound proposition
@reason.assert(initialized(x) and not mutated(y))
let z = x + y;

// Scoped to a block
@reason.assert(not might_throw) {
    parse(data);
    validate(data);
}
```

### 2.3 Plugin Registration (Strawman)

```
// A plugin is registered by exporting a function with the @reason.plugin annotation:
@reason.plugin("my_custom_check")
export fn my_custom_check(node: AstNode, ctx: ReasoningContext) -> bool {
    // ... analysis logic ...
}
```

### 2.4 Type System Integration Interface (Strawman)

If the type system is built on the reasoning tool, type checks become propositions:

```
// Built-in type-checking propositions:
//   assignable_to(expr, Type)    — is the expression assignable to the type?
//   narrows_to(variable, Type)   — after this point, is the variable's type narrowed?

@reason.assert(assignable_to(x, i32))
let y: i32 = x;
```

---

## 3. Implementation Challenges

### 3.1 Performance

- **Analysis time.** A plugin-based reasoning system is inherently extensible, which makes it hard to bound analysis time. A project with many plugins could have unacceptable build times.
- **Fixpoint computation.** If propositions depend on each other, the system may need iterative fixpoint computation, which can be expensive on large codebases.
- **Inter-procedural analysis.** Reasoning across function boundaries requires whole-program analysis or summary-based approaches, both of which are complex and expensive.

### 3.2 Soundness and Completeness

- **Plugin correctness.** The system's guarantees are only as strong as its plugins. A buggy plugin can produce unsound results (claiming something is true when it is not), which could lead to miscompilation or missed errors.
- **Undecidability.** Some properties of programs are undecidable in general (e.g., halting). The system needs a clear policy for what happens when a proposition cannot be determined — is the result `true`, `false`, or an error?
- **Approximation.** Practical static analysis is approximate. The system must define whether propositions are over-approximate (may report false positives) or under-approximate (may miss true positives), and whether this is configurable per proposition.

### 3.3 Plugin Sandboxing

- **Termination.** A plugin is an arbitrary ish function. It could loop forever. The system needs a strategy for detecting or preventing non-terminating plugins (timeouts, resource limits, or restricting the plugin language to a decidable subset).
- **Side effects.** Plugins should be pure (no I/O, no mutation of shared state), but enforcing this needs either language-level purity guarantees or a sandbox.

### 3.4 Bootstrapping

- **Chicken-and-egg problem.** Plugins are ish functions, but the reasoning system is needed to compile ish. The initial set of built-in propositions must be implemented in Rust, with the plugin system bootstrapped later. The boundary between "built-in" and "plugin" needs to be clearly defined.

### 3.5 Composability with the Type System

- **Unifying two complex systems.** If the type system is implemented as a special case of the reasoning system, the reasoning system must be powerful enough to express all type-system concepts (subtyping, generics, variance, type narrowing). This is a large design surface and risks making the reasoning system overly complex.
- **Error quality.** Type errors are among the most common errors developers encounter. If type checking is routed through a generic proposition system, the error messages must still be as clear and specific as those from a purpose-built type checker.

### 3.6 LSP Integration

- **Incremental analysis.** The LSP server must respond to edits in real time. The reasoning system needs an incremental evaluation strategy — re-evaluating only the propositions affected by a change, not the entire program.
- **Partial results.** During editing, code is often syntactically or semantically incomplete. The reasoning system must degrade gracefully rather than failing entirely.

---

## 4. Example Use Cases

### 4.1 Compile-Time Safety Assertions

A developer writing critical code can assert that a section never throws, ensuring the compiler verifies this statically:

```
@reason.assert(not might_throw)
fn transfer_funds(from: Account, to: Account, amount: f64) {
    // The compiler guarantees no exception can escape this function.
    // If a future edit introduces a throwing call, the build fails.
    from.balance -= amount;
    to.balance += amount;
}
```

### 4.2 Dead Code Detection

A library author can mark code that should always be reachable, catching dead code introduced by refactoring:

```
fn process(input: Input) {
    if (input.kind == "a") {
        handle_a(input);
    } else if (input.kind == "b") {
        handle_b(input);
    } else {
        @reason.assert(reachable)  // Fails if the analyzer proves this branch is dead
        handle_unknown(input);
    }
}
```

### 4.3 Initialization Guarantees

In performance-critical code, a developer can assert that a variable is always initialized before use, avoiding runtime null checks:

```
let result: i32;
if (condition) {
    result = compute_fast();
} else {
    result = compute_slow();
}
@reason.assert(initialized(result))
use(result);  // No runtime check needed — the compiler proved initialization
```

### 4.4 Custom Domain-Specific Analysis

A web framework can ship a plugin that checks for SQL injection vulnerabilities:

```
// In the framework's plugin package:
@reason.plugin("sql_safe")
export fn sql_safe(node: AstNode, ctx: ReasoningContext) -> bool {
    // Returns true if the expression is a parameterized query,
    // false if it contains string concatenation with user input.
    // ...
}

// In application code:
@reason.assert(sql_safe)
let query = build_query(user_input);
db.execute(query);
```

### 4.5 Immutability Verification

A developer can assert that a shared data structure is never mutated after construction:

```
let config = load_config();
@reason.assert(not mutated(config))
start_server(config);
// If any code path between here and the end of scope mutates config, the build fails.
```

### 4.6 Query Mode for Exploration

During development, a programmer can query the reasoning system to understand code behavior without failing the build:

```
@reason.query(might_throw)      // IDE shows: "yes — parse() on line 12 may throw"
fn process(data: String) {
    let parsed = parse(data);
    validate(parsed);
}
```

---

## 5. Evaluation

### 5.1 Pros

1. **Unification.** A single reasoning tool serving the compiler, analyzer, and LSP avoids duplicating analysis logic across components. This reduces bugs and keeps behavior consistent.
2. **Extensibility.** The plugin system allows developers and framework authors to define domain-specific analyses without modifying the language itself. This is a powerful differentiator.
3. **Developer empowerment.** Exposing the reasoning tool to the language gives developers direct access to the same analysis the compiler uses. Assertions like "this code is reachable" or "this variable is initialized" are verifiable at build time, not just trusted comments.
4. **Alignment with ish philosophy.** The streamlined ↔ encumbered continuum is about making tradeoffs configurable. Reasoning annotations are a natural extension — developers opt in to the level of static verification they want.
5. **Type system unification potential.** If the type system can be implemented as a special case, the language's conceptual surface area shrinks: there is one system for reasoning about code, not two.
6. **Ecosystem value.** Third-party reasoning plugins (security checks, framework-specific validations, coding standards enforcement) could form a rich ecosystem similar to linting rules, but with deeper semantic understanding.

### 5.2 Cons

1. **Complexity.** The reasoning system is the most ambitious component in ish. Making it general enough to subsume the type system, performant enough for interactive use, and correct enough to trust for compilation is a very high bar.
2. **Performance risk.** Extensible static analysis is expensive. Without careful design, build times and LSP responsiveness could suffer, especially with third-party plugins.
3. **Soundness risk.** Trusting user-written plugins for compiler-level decisions (e.g., optimizations based on "this never throws") is dangerous. A buggy plugin could cause silent miscompilation.
4. **Bootstrapping burden.** The built-in propositions must be implemented in Rust before ish can compile itself. This increases the amount of Rust code needed for bootstrapping, working against the stated goal of minimizing it.
5. **Learning curve.** Developers must understand propositions, plugins, and the annotation syntax in addition to the standard language features. For a language aiming to be "approachable by anyone who knows at least one other programming language," this adds conceptual weight.
6. **Unclear precedent.** No mainstream language has a user-extensible static reasoning system at this level of generality. This is novel, which means there is no existing ecosystem of tools, patterns, or best practices to draw on. The design risk is high.
7. **Error message quality.** Routing all analysis through a generic proposition system risks producing abstract, hard-to-understand error messages. Purpose-built analyzers (type checkers, linters) can produce tailored messages because they understand the specific domain.

### 5.3 Overall Evaluation

The reasoning system is the most distinctive and ambitious design element in ish. Its core insight — that type checking, reachability analysis, mutation tracking, and other forms of static reasoning are all special cases of a single mechanism — is intellectually compelling and, if executed well, could significantly reduce the complexity of the language's internals.

However, the gap between the current sketch and a workable design is large. The proposal needs concrete answers to fundamental questions: What is the evaluation model? How is soundness maintained when plugins are user-written? How does performance scale? The risk of the reasoning system becoming an over-generalized abstraction that does nothing well is real.

A pragmatic path forward would be to:

1. **Implement the built-in propositions first** as a conventional static analyzer, without the plugin system.
2. **Validate the proposition model** by expressing the type system's existing analyses (reachability, narrowing, initialization) as propositions internally.
3. **Design the plugin interface** based on the patterns that emerge from step 2, rather than designing it speculatively.
4. **Expose the annotation syntax** only after the internal model is stable.

This staged approach reduces the risk of over-engineering while preserving the option to realize the full vision.
