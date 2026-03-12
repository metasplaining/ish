---
title: ish Agreement and Marked Language Features
category: spec
audience: [all]
status: draft
last-verified: 2026-03-11
depends-on: []
---

# ish Agreement and Marked Language Features

Agreement and marked language features are concepts from linguistics. In natural language, certain words in a sentence need to agree with each other in order for the sentence to be grammatical. For example, the sentence "He are here." is ungrammatical because the word "He" (singular) does not agree with the word "are" (plural). "Grammatical number" (singular vs. plural) is considered to be a marked feature in the English language — it is something that must be expressed in order to speak grammatically. Other marked features in English include tense and gender. It is important to note that different language features are marked in different languages. For example, in Turkish whether you saw something yourself versus heard from someone else is a marked language feature. Even languages that mark the same feature sometimes do it differently. For example, in English gender is marked in the third person pronoun (he/she) but not in the second person pronoun (you). In Spanish, on the other hand, gender is marked in the second person pronoun (nosotros/nosotras) but not in the third person pronoun (su).

In ish, what language features are marked is configurable. In streamlined ish, practically nothing is marked. But ish can be encumbered to mark a large number of language features. When one of these features is marked, the language requires that it be present. When the feature is present, it is checked for agreement. Note that even in streamlined ish, it is still allowed for any of these features to be present, and when that happens, the language will check it for agreement.

## Configurable Marked Features

Features that can be configured to be marked or unmarked in ish:

- Variable type
- Function parameter type
- Function return type
- Async type (wait or defer)
- Errors thrown by function
- Error mode preset (streamlined, encumbered, or no-throw)
- Mutability (whether a variable or property is mutable or immutable)
- Numeric type (exact numeric type such as `i32` vs. `f64`, rather than a default)
- Numeric overflow behavior (wrapping, panicking, or saturating)
- Implicit numeric conversions (whether safe widening conversions are allowed implicitly)
- Nullability (whether a variable can hold `null`)
- Memory management model (stack, heap/owned, reference counted, or garbage collected)
- Polymorphism strategy (none, enumerated, monomorphized, virtual method table, or associative array)
- Open vs. closed object types (whether an object may have undeclared properties)

---

## Open Questions

Open questions for the agreement system. See also [docs/project/open-questions.md](../project/open-questions.md#agreement-system) for a consolidated view.

- [ ] **What happens when an agreement is violated at build time vs. runtime?**
- [ ] **What is the syntax for marking/unmarking features at the project, file, function, or variable level?**
- [ ] **How does agreement interact with boundaries between differently-encumbered code?**

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
- [docs/spec/reasoning.md](reasoning.md)
- [GLOSSARY.md](../../GLOSSARY.md)
