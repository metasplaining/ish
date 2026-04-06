---
title: "RFP: Module System"
category: rfp
audience: [ai-dev, human-dev]
status: draft
last-verified: 2026-04-05
depends-on: [docs/spec/modules.md, docs/spec/execution.md, GLOSSARY.md]
---

# RFP: Module System

The next features to implement are modules, projects, packages, lexical scopes, and the use directive, in some order. We would like to implement these one at a time, but first we need to figure out a natural order to implement them.

## Research Request

For the problems described below, research how each of the following languages solves them: **OCaml**, **Rust**, **Go**, **Scheme**, and **Haskell**. Existing decisions already made in ish should not constrain the research. We want a best-in-class module system. At the same time, no ish-specific innovations are planned — the goal is to find out what the best existing languages do and imitate them. Undoubtedly there are tradeoffs involved, and we want to identify what they are and make them intentionally rather than accidentally. What are the key issues and tradeoffs? What approaches have these projects taken? Are there any pitfalls or techniques that we specifically need to be aware of?

## Problems to Solve

1. **Physical downloading and caching of external code.**
2. **Versioning of code and versioning dependencies.**
3. **Different parts of the code depend on different versions of external code.**
4. **Encapsulation** — Support reasoning about code by limiting the set of things that could possibly interact.
5. **Forward references** — By designating a set of code that must be defined concurrently and possibly recursively.
6. **Specifying project metadata and dependencies.**
7. **Supporting different physical dependencies by developers using the same code.**
   - Developers working on both the code and its dependencies get both from their workspace.
   - Developers not working on the dependencies get them from a remote repo.
   - Different developers may use different repos for the same code, for trust and caching reasons.
8. **Mixed programming language/shell nature** — Support a strategy or combination of strategies that is convenient for the mixed programming language/shell nature of ish.
9. **Simplify dependency management.**
10. **Inherited dependency management** — Avoid repeating ourselves in every project.
11. **Traceability** — Make code easily traceable. It should be obvious how to trace dependencies.

## Open Question

What other problems should we be aware of while implementing modules, projects, packages, and lexical scopes?

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
