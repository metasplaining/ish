---
title: ish Polymorphism Strategies
category: spec
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md]
---

# ish Polymorphism Strategies

Existing languages implement polymorphism in several ways. Ordered roughly from most performant / most constrained to least performant / least constrained:

| Strategy              | Description                                                                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------|
| None                  | Data is stored in a fixed format determined at build time.                                                                                       |
| Enumerated            | Data can be one of several variant formats, each determined at build time. Code matches on the variant and executes case logic.                   |
| Monomorphized         | Code is written against interfaces. The build tool generates a specialized (monomorphized) variant of each function for each conforming format.   |
| Virtual method table  | Code is written against interfaces. The build tool attaches metadata to data records, enabling functions to interpret the data at runtime.        |
| Associative array     | Each record is stored as a hash table, allowing an arbitrary set of properties per object at runtime.                                            |

ish supports all of these strategies. In general, the implementation detail is hidden from the developer. The ish language processor chooses the highest-performing strategy for which all constraints are met. For example, interpreted ish always uses associative arrays because there is no build step at which to choose a more constrained option. The active standard can require a specific strategy and produce a discrepancy if the constraints for that strategy are not met.

---

## Open Questions

Open questions for polymorphism. See also [docs/project/open-questions.md](../project/open-questions.md#polymorphism) for a consolidated view.

- [ ] **Developer-facing interface.** How does a developer define an interface/trait/protocol? How does a developer implement it for a type? What does polymorphic code look like syntactically?
- [ ] **Strategy selection rules.** What exactly are the constraints for each strategy? Can the developer override the automatic selection?
- [ ] **Interaction with type system.** How do generics / type parameters work? Are there trait bounds?

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
- [docs/spec/memory.md](memory.md)
- [GLOSSARY.md](../../GLOSSARY.md)
