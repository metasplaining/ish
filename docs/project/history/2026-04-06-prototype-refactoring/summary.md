---
title: "History: Prototype Code Quality Refactoring"
category: history
audience: [ai-dev, human-dev]
---

# History: Prototype Code Quality Refactoring

## v1 → v2 (2026-04-06)

The initial proposal (v1) was generated directly from the `/propose-refactoring` scan of the
full `proto/` codebase. It identified seven findings — two High, three Medium, and two Low —
and presented alternatives and decision prompts for each.

The human reviewed the proposal and entered inline decisions on the same day. Every decision
was resolved:

- **H1 (interpreter helpers):** Approved. The three helpers named in the proposal
  (`eval_literal`, `apply_property_write`, `apply_index_write`) were confirmed. The human
  also asked for further research into additional candidates; the `/revise` pass identified
  three more (`eval_unary_op`, `apply_property_read`, `apply_index_read`) that are equally
  clean extractions.

- **H2 (ast_builder unwraps):** Conservative path chosen — fix only the non-grammar-structural
  panic sites (numeric parsing overflow, `lines.last()`, the `value.unwrap()` in
  `build_var_decl`), not all 102 `.next().unwrap()` calls. Appropriate for prototype stage.

- **M1 (register_ast_builtins):** `simple_ast_builtin` helper approved. Naming-convention
  dependency (`ast_<kind>`) accepted.

- **M2 (builtins.rs boilerplate):** `arity` helper and `new_builtin` wrapper approved.

- **M3 (brace-scanning):** `scan_to_close_brace` extraction approved, motivated primarily
  by the removal of the inline `.unwrap()` panic site.

- **L1 (exit-code constant):** Approved.

- **L2 (Display split):** Deferred.

The v2 revision removed all pending decision prompts, expanded H1 with the six-helper
catalogue, added testability statements and concurrency notes to each feature section, and
updated the decision register to reflect all resolved outcomes.
