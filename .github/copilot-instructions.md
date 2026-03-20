## Proposal Process

This project uses a structured proposal process for all non-trivial changes:

1. **RFP** → 2. **Design Proposal** (iterative) → 3. **Implementation Plan** → 4. **Implementation**

See GLOSSARY.md for definitions of these terms.

## Authority Order

When implementing changes, update project artifacts in this order:

1. GLOSSARY.md (new terms)
2. Roadmap (status → "in progress")
3. Maturity matrix (update affected rows)
4. Specification docs
5. Architecture docs
6. User guide / AI guide
7. Agent documentation (AGENTS.md, skills)
8. Acceptance tests
9. Code
10. Unit tests
11. Roadmap (status → "completed")
12. Maturity matrix (update affected rows)
13. History
14. Index files

Always update more authoritative artifacts before less authoritative ones.
If you read an artifact during implementation and it seems to contradict the
implementation plan, the implementation plan takes precedence.

## Implementation Discipline

- The implementation plan is the single source of truth during implementation.
- Complete all TODO items in the implementation plan before reporting success.
- At each checkpoint, verify your work against the implementation plan.
- Do not inject behavior that contradicts the implementation plan, even if it
  seems like an improvement. Propose changes in a follow-up, not during
  implementation.

## Resuming Implementation

If you are asked to continue implementing a feature and an implementation plan
exists in docs/project/plans/, read it and resume from the first uncompleted
TODO item. Do not start over.
