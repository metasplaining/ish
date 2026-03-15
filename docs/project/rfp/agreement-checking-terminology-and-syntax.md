---
title: "RFP: Agreement Checking — Terminology and Syntax"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-14
depends-on: [docs/spec/types.md, docs/spec/execution.md, GLOSSARY.md]
---

# RFP: Agreement Checking — Terminology and Syntax

*Converted from `agreement` on 2026-03-14.*

---

## Background

Many features in ish are built on top of agreement checking.

From a developer's perspective, agreement checking works as follows. The ish language processor maintains a set of facts. Each statement entails additional facts, so before a statement there is one set of facts, and after the statement there is a new, slightly different set of facts. Before processing a statement, the language processor checks whether the new set of facts will be internally consistent — or, expressed differently, it checks whether all of the statements agree with one another. If there is any disagreement, instead of executing the statement, the language processor throws an agreement error. The error describes the disagreement and indicates each of the statements that entailed facts involved in it. Configuring a language feature as marked will generally increase the number of facts entailed by statements.

### Example

```
[@immutable_by_default] {
    let x = 7;
    x = false;
}
```

The block declaration entails the fact `@immutable_by_default(block1)`. This fact means that whenever a variable is declared, it is immutable unless the declaring statement explicitly includes the `mut` keyword. The statement `let x = 7` entails the facts `@int(x), @value(x,7), @immutable(x)`. These facts mean that the variable `x` has a structural type that is one of the integer types, the current value of `x` is 7, and `x` is immutable. The statement `x = false` entails the facts `@bool(x), @value(x,false)`. Before executing the statement `x = false`, the ish language processor checks the new facts against the existing facts and detects that they disagree.

This example highlights the problem that a single statement can introduce multiple disagreements. The variable `x` has the `@immutable` fact, yet its value is being mutated. It also has the `@int` fact, and the value is set to a bool. A strategy is needed for determining which agreement error will be thrown. This strategy is TBD, but for this example, suppose the error is that there was an attempt to mutate an immutable variable. The error will reference all three statements, because:

1. The variable `x` was declared as immutable. (Statement 2)
2. The reason that Statement 2 makes the variable immutable is that immutable is the default. (Statement 1)
3. The attempt to mutate the variable comes from Statement 3.

### Fact Types and Rules

There is a defined set of possible fact types and associated rules. For example, `@int` and `@bool` are fact types, and `@int` implies `!@bool` is an associated rule. The set of fact types is extensible, as ish supports user-defined annotations, and annotations are implemented as agreement facts. All fact types and rules must follow a set of constraints that ensure they are resolvable efficiently at build time. These constraints include:

1. Every fact is attached to a single item (project, module, function, block, statement, variable, or other).
2. The kinds of items that each fact type may be attached to is restricted. For example, `@bool` may only be applied to a variable.
3. The rules for detecting disagreements are restricted to a specific set of information available at build time. (This set will evolve over time as needed. Based just on the example above, it must include the set of facts attached to the variable that the new fact is being attached to, and the set of facts attached to the lexical scope of the statement entailing the new fact.)
4. Facts may take parameters other than the item they are attached to, but those parameters must be defined at build time.

### Subsumed Language Features

The agreement checking language feature subsumes many other language features:

1. Types — Types are facts about variables.
2. Annotations — Annotations attach facts to items.
3. Configurable marking of language features — Facts attached to lexical scopes.
4. Mutability/Immutability — Facts attached to variables.
5. Visibility — Facts attached to variables.
6. Sync/Async — Facts attached to blocks and functions.
7. Blocking wait/No blocking wait — Facts attached to blocks and functions.
8. State mutator/Not state mutator — Facts attached to functions.
9. Unused variable checking — Facts attached to variables.
10. Unreachable statement checking — Facts attached to statements.
11. Null dereferences.
12. Checked exceptions — Facts attached to functions.

There are likely others.

### Execution-Time vs. Declaration-Time Checking

There are two approaches to agreement checking:

1. **Execution time** — The approach shown above. Each statement is checked for disagreement as it is executed.
2. **Declaration time** — An entire lexical scope (project, module, function, block) is checked as it is declared.

Declaration-time agreement checking throws an error if any reachable path through a function or block could possibly throw an agreement error. Declaration-time checking is configurable on a feature-by-feature basis. For example, unused variable checking might be configured to be checked at declaration time, while null dereferences are configured to be checked at execution time. In this case, an attempt to declare a function with unused variables would throw an agreement exception when the function was declared, but an attempt to declare a function that dereferences null pointers would not throw an agreement exception until a null pointer is actually dereferenced at runtime.

Note that although execution-time agreement checking is well suited to interpreted mode, and declaration-time checking is well suited to compiled mode, they are not coupled. Both kinds of agreement checking are supported by both the compiler and the interpreter.

### Presets

Combinations of facts can be defined as presets, so that all of the facts can be attached to an item with a single annotation.

### Relationship to Structural and Nominal Typing

The type specification says that ish supports hybrid structural/nominal typing. This can be clarified in the context of agreement checking. ish supports structural types, both inferred and declared. Additionally, a developer could:

1. Annotate a variable with a custom annotation.
2. Annotate a function to require the custom annotation.
3. Implement rules for the custom annotation to require other facts, including structural typing, mutability, not throwing exceptions, etc.

---

## Requests

### 1. Terminology: Research and Recommend a Metaphor

The current terminology uses "agreement," based on a natural linguistics metaphor. An alternative under consideration is "contract," based on a legal contracts metaphor.

The natural linguistics metaphor is problematic because it relates to grammar, and this language feature is much more about semantics than syntax. The legal contract metaphor is appealing because developers are already familiar with the design-by-contract pattern.

The term "facts" comes from a propositional logic metaphor, which is good because a computer science education includes propositional logic and developers are already familiar with it. However, it mixes terminology from a natural language metaphor with terminology from a propositional logic metaphor. If switching to a contract metaphor, "facts" would become "terms." This extends the contract metaphor, but the word "term" could easily be confused as meaning "word" instead of "a point of specification in a contract."

**Research what terminology other languages use for these features.** Is there a better fit for what ish is trying to do? Also, propose some alternative metaphors. A good metaphor will have these properties:

1. The metaphor itself relates to consistency checking and will be easily understood by software developers.
2. The terminology related to the metaphor is distinctive, so that when readers see a term they will instantly recognize it as pertaining to the metaphor and the particular role in the metaphor being referenced.
3. The terminology of the metaphor does not have multiple meanings and is therefore unlikely to be misunderstood by readers.

For each alternative metaphor proposed, also propose terminology. List the pros and cons of each alternative metaphor. Recommend a metaphor and terminology.

### 2. Syntax: Propose Alternatives and Recommend

Propose alternative syntaxes for this feature, with pros and cons for each. Recommend a syntax. Consider as alternatives:

- A unified "everything is an annotation" approach to syntax.
- An "agreement is completely hidden and every language feature has its own dedicated syntax, even though it is all handled by the same agreement checker under the covers" approach.
- Hybrid approaches.
- Any other approaches not listed here.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
- [docs/project/proposals/agreement-metaphor-and-syntax.md](../proposals/agreement-metaphor-and-syntax.md)
- [docs/project/proposals/ledger-system-syntax.md](../proposals/ledger-system-syntax.md)
- [docs/project/proposals/assurance-ledger-syntax.md](../proposals/assurance-ledger-syntax.md)
