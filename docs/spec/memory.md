---
title: ish Memory Management
category: spec
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/spec/agreement.md, docs/spec/polymorphism.md]
---

# ish Memory Management

ish supports four memory management models, ordered from most performant / most constrained to least:

| Model              | Description                                                                                                                                                     |
|--------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Stack              | A fixed-size slot is allocated in the function's stack frame. The variable is deallocated when the function returns.                                             |
| Heap (owned)       | Space is allocated on the heap with exactly one owning pointer. The variable is deallocated when the pointer goes out of scope.                                  |
| Reference counted  | Space is allocated on the heap. A reference count tracks pointers; the variable is deallocated when the count reaches zero.                                      |
| Garbage collected  | Space is allocated on the heap. A mark-and-sweep garbage collector deallocates unreachable variables.                                                            |

The ish language processor chooses the highest-performing model for which all constraints are met. For example, interpreted ish always uses garbage collection. The stack is used when a variable's size is known at build time and the variable only exists for the lifetime of a single function call. The build can be encumbered to fail with a descriptive error message if the constraints for a particular model are not met.

---

## Open Questions

Open questions for memory management. See also [docs/project/open-questions.md](../project/open-questions.md#memory-management) for a consolidated view.

- [ ] **Developer-facing controls.** Can a developer explicitly choose a memory management model for a variable? Or is it always inferred? What annotations or syntax exist?
- [ ] **Ownership and borrowing rules.** Are there borrowing / lifetime annotations? Is there a borrow checker in encumbered mode? Rules for passing owned values to functions?
- [ ] **Reference cycle handling.** Reference counting cannot collect cycles. Is this addressed (e.g., weak references)?

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
- [GLOSSARY.md](../../GLOSSARY.md)
