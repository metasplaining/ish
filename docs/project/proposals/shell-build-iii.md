---
title: "Proposal: Shell Build III"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-15
depends-on: [docs/project/rfp/shell-build-iii.md, docs/project/proposals/shell-build-ii.md, docs/spec/syntax.md, docs/architecture/ast.md, GLOSSARY.md]
---

# Proposal: Shell Build III

*Generated from [shell-build-iii.md](../rfp/shell-build-iii.md) on 2026-03-15.*

*Follow-on to [shell-build-ii.md](shell-build-ii.md). Records decisions, corrects errors. No new features.*

---

## Summary of Decisions from Shell Build II

| Topic | Decision |
|-------|----------|
| `IndexAccess`, `CatchParam`, `FunctionType`, `GenericParams`, `GenericType` | Reclassified from **Error** to **Wait** |
| Validator input size cap | No — prioritize correctness |
| `has_incomplete` API style | Inherent methods |
| Non-delimiter error productions | Out of scope |

---

## Correction 1: REPL Completion Categories

### Updated Wait list (26 variants)

All unterminated constructs except single-line strings cause the REPL to wait for more input:

```
Block, ObjectLiteral, Match, EntryTypeDef, ObjectType,
ListLiteral, StandardDef, StandardAnnotation, EntryAnnotation,
IndexAccess,
ParenExpr, CallArgs, FnParams, LambdaParams, WithResources,
CatchParam, CommandSubstitution, TupleType, FunctionType,
TripleSingleString, TripleDoubleString,
ExtendedTripleDoubleString, ExtendedTripleSingleString,
BlockComment,
GenericParams, GenericType
```

### Updated Error list (7 variants)

Only single-line string types are treated as errors — unterminated single-line strings are always wrong, never a continuation:

```
StringLiteral, InterpString, CharLiteral,
ExtendedDoubleString, ExtendedSingleString,
ShellQuotedString, ShellSingleString
```

### Updated `is_continuable()` method

```rust
impl IncompleteKind {
    /// Returns true if this kind of incomplete input should cause the REPL
    /// to wait for more input (multiline continuation). Returns false if
    /// it should be reported as an error immediately.
    pub fn is_continuable(&self) -> bool {
        match self {
            // Unterminated single-line strings — error, not continuation
            Self::StringLiteral
            | Self::InterpString
            | Self::CharLiteral
            | Self::ExtendedDoubleString
            | Self::ExtendedSingleString
            | Self::ShellQuotedString
            | Self::ShellSingleString => false,

            // Everything else — wait for more input
            _ => true,
        }
    }
}
```

### Updated Feature 1 table entries

The following rows from shell-build-ii Feature 1 change from **Error** to **Wait**:

| # | Rule | Unterminated rule | REPL behavior (corrected) |
|---|------|-------------------|--------------------------|
| 10 | `index_access` | `unterminated_index_access` | **Wait** |
| 16 | `catch_clause` param | `unterminated_catch_param` | **Wait** |
| 19 | `function_type` params | `unterminated_function_type` | **Wait** |
| 32 | `generic_params` (`<T, U>`) | `unterminated_generic_params` | **Wait** |
| 33 | `generic_type` (`Type<T>`) | `unterminated_generic_type` | **Wait** |

All other rows are unchanged.

---

## Correction 2: Remove Stale Bracket-Counting Text

Shell-build-ii Feature 5 (REPL Incomplete Detection from the predecessor shell-build) contained text describing a "two-layer approach" with a bracket-counting `IshValidator` as a "fast path" and the parser as an "authoritative fallback." This contradicts the decision from shell-build that bracket counting is fundamentally broken and eliminated.

The corrected design is: **the parser is the only layer.** There is no bracket-counting state machine. The reedline `Validator` invokes the full parser on every Enter keypress.

The following text from shell-build-ii Feature 3 is the sole authority on how the validator works:

> Reedline's `Validator` trait has a single method: `fn validate(&self, line: &str) -> ValidationResult`. It runs on every Enter keypress. The implementation now invokes the parser.

There is no fast path, no pre-filter, no keystroke-level scanner. The parser handles everything.

---

## Correction 3: Test Reclassification

The test in shell-build-ii Feature 8 that checks `unterminated_string_is_error` is correct — single-line strings are indeed non-continuable errors. But the comment in `unterminated_string_inside_list` needs updating:

```rust
#[test]
fn unterminated_string_inside_list() {
    let result = ish_parser::parse("let x = [\"hello").unwrap();
    assert!(result.has_any_incomplete());
    // The unterminated string is not continuable (single-line string error),
    // but the unterminated list IS continuable.
    // has_incomplete_continuable checks the whole tree — the list's
    // unterminated_list_literal makes this continuable overall.
    assert!(result.has_incomplete_continuable());
}
```

This is a subtle but important case: a non-continuable `Incomplete` node nested inside a continuable one. The tree-walking `has_incomplete_continuable` finds the continuable list and returns `true`. The REPL waits for more input. This is correct behavior — the user is still building the list; the string error will surface when the list is eventually closed.

---

## No Other Changes

All grammar rules, AST types, builder functions, implementation sequence, files affected, deferred items, and documentation updates from shell-build-ii remain unchanged. This proposal only records decisions and corrects the errors noted above.

---

## Documentation Updates

No additional documentation changes beyond those specified in shell-build-ii.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-shell-build-iii.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
