---
title: "RFP: Implementation Audit — Types, Errors, and Assurance Ledger"
category: rfp
audience: [ai-dev]
status: rfp
last-verified: 2026-03-20
depends-on: [docs/spec/assurance-ledger.md, docs/spec/errors.md, docs/spec/types.md, docs/project/plans/types-errors-assurance-consistency.md]
---

# RFP: Implementation Audit — Types, Errors, and Assurance Ledger

During implementation of the types, errors, and assurance ledger feature, the
implementing agent appears to have become confused and forgotten or
misunderstood the requirements. This RFP describes the known issues and
requests a proposal for fixing them.

---

## Issue 1: VM Incorrectly Gates Type Narrowing on `types` Feature

In `proto/ish-vm/src/interpreter.rs`, in the `exec_statement` function, in the
case that processes `if` statements, the VM checks whether the `types` feature
is enabled and applies type narrowing only if it is. If the feature is
disabled, the VM skips type narrowing entirely.

This is broken in at least two ways:

1. **It is not the VM's job to apply standards.** This is the ledger's job. If
   this logic is needed at all, it should be in the ledger.
2. **This logic shows a fundamental misunderstanding of how standards work.**
   The `types` feature controls whether explicit types are required to be
   specified when declaring variables. It does not control whether discrepancy
   checking is enabled. Discrepancy checking is always enabled. By skipping
   the application of type narrowing in this case, this logic corrupts the
   ledger state for the entire `if` block.

## Issue 2: Acceptance Tests for Type Narrowing Are Useless

The acceptance tests for type narrowing appear to have mostly correct
conditional logic to trigger type narrowing, but they have no way of asserting
ledger state once they have triggered it. They just print constants. A builtin
should be added to construct a ledger state object, and the assurance tests
should be fixed to call and check that instead — something like
`println(ledger_state(x))`.

## Issue 3: Error Specification — Structural Error Hierarchy Misunderstanding

There is a misunderstanding in the error specification that has been partially
corrected but needs more work.

`CodedError` and the types in the `SystemError` hierarchy should not each have
their own `EntryType`. The error hierarchy should work as follows:

- **Error** is both nominal and structural. To be an Error, an object must have
  the `@Error` annotation and a `message` property of type `String`.
- **CodedError** is purely structural. To be a `CodedError`, an object must be
  an Error and have a `code` property of type `String`.
- **Leaf errors** (e.g., `NoSuchFileError`) are purely structural. To be a
  `NoSuchFileError`, an object must be a `CodedError`, and the `code` property
  must have the value of the appropriate error code (e.g., `"E004"`).
- **Domain union errors** (e.g., `FileError`) are a union of all the file-related
  leaf errors.

The system is structured this way so that:

1. It is easy to create and throw any type of error.
2. It is easy to catch the kind of errors you want. `catch(e: Error)`,
   `catch(e: FileError)`, and `catch(e: NoSuchFileError)` all do the right
   thing.
3. The `throw` statement does not do a lot of special-case entry management
   logic. It adds one annotation, the `@Error` annotation.
4. Error handling code can be written fearlessly in a low-assurance
   environment: `catch(e) { println("Error: {e.message}") }` is guaranteed to
   catch everything that can possibly be thrown and not fail with a type error
   or null dereference.

## Issue 4: `defer` Scoping Regressed in Error Spec

The error specification now says `defer` is block-scoped. `defer` is supposed
to be function-scoped. A separate proposal (`defer-scoping.md`) resolved this
question in favor of function scoping, and the implementation correctly uses
function-scoped defer stacks. The spec wording regressed during the
consistency rewrite.

## Issue 5: Spec Rewrite May Have Introduced Other Meaning Changes

Check the specifications against the git history to see if the process of spec
rewriting introduced any other changes of meaning that were not specifically
requested in the proposal.

## Issue 6: Check Assurance Ledger Spec Clarity

Check the specifications to see if they can be improved to more clearly
specify that the VM should not do ledger checking and that ledger checking and
updating are always performed (not gated on feature enablement).

## Issue 7: Comprehensive Audit Request

Fix the broken interpreter code. Specifically check the interpreter, ledger
implementation, and acceptance tests for other similar issues. Perform a
feature consistency audit on the ledger, types, and errors.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
