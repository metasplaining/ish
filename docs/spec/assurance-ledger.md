---
title: Assurance Ledger
category: spec
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/project/proposals/assurance-ledger-syntax.md]
---

# Assurance Ledger

The assurance ledger is ish's unified consistency-checking system. It manages the full spectrum from low-assurance code (minimal annotations, dynamic checking) to high-assurance code (extensive annotations, static checking). The system has four core concepts:

- **Standards** configure what the ledger checks within a scope.
- **Entries** record facts about individual items (variables, properties, functions, types).
- **Audits** check entries for consistency — either at build time (pre-audit) or at execution time (live audit).
- **Discrepancies** are conflicts detected by the audit — two entries are incompatible, or a required entry is missing.

---

## Concepts

### Standards

A **standard** is a named configuration that sets feature states within a scope. Standards are applied with `@standard[name]` and defined with `standard name [...]` or `standard name extends base [...]`. Each feature in a standard has a state that determines whether and when it is checked.

Standards can be applied at module, function, or block scope. When a standard is applied, it governs all code within that scope. Inner scopes can apply different standards or override individual features.

```ish
@standard[cautious]
fn process(data: List<String>) -> Result {
    @standard[rigorous]
    {
        // This block is checked under the rigorous standard
        let mut sum: f64 = 0.0;
        // ...
    }
}
```

### Entries

An **entry** is a fact recorded about an item in the ledger. For example, `@[type(i32)]` records that a variable has type `i32`; `@[mutable]` records that a variable is mutable.

Entries can be created by native syntax or by explicit annotation. These are equivalent:

```ish
let mut x: i32 = 7
@[mutable] @[type(i32)] let x = 7
```

Native syntax is preferred for readability. The annotation form exists for programmatic generation and for features that lack native syntax.

### Audit

The **audit** is the process by which the ledger checks entries for consistency. There are two audit modes:

- **Pre-audit** — occurs at build time (declaration time). Features set to `pre` in the active standard are checked during pre-audit.
- **Live audit** — occurs at execution time. Features set to `live` in the active standard are checked during live audit.

Features set to `optional` are not required. If an entry for an optional feature is present, it is still checked; if absent, no discrepancy is raised.

### Discrepancies

A **discrepancy** is a conflict detected by the audit. Two entries on the same item are incompatible, or a required entry is missing. Discrepancy reporting includes an audit trail tracing back through the chain of standards and statements that contributed to the conflict.

---

## Standard Definition Syntax

Standards are defined with the `standard` keyword:

```ish
standard cautious [
    types(live),
    null_safety(live),
    immutability(live),
]

standard api_safety extends cautious [
    checked_exceptions(pre),
    null_safety(pre),
    undeclared_errors(typed),
]
```

Standards support single inheritance via `extends`. When a standard extends another, it inherits all feature states from the parent and can override individual features. Features not mentioned are inherited unchanged.

Standards can be defined at module level or inside functions and blocks.

---

## Standard Application Syntax

Standards are applied with `@standard[name]`:

```ish
@standard[cautious]

// Inline feature override:
@standard[overflow(saturating), checked_exceptions(live)]

// Apply a named standard to a function:
@standard[api_safety]
fn get_user(id: UserId) -> User | NotFoundError {
    // ...
}
```

Multiple `@standard[...]` annotations on the same scope are cumulative — feature states from later annotations override those from earlier ones.

---

## Entry Annotation Syntax

Entries are applied with `@[entry(params)]`:

```ish
@[overflow(wrapping)] let z: u8 = 255
@[memory(stack)] let buffer: List<u8> = alloc(1024)
@[nullable] let name: String? = null
@[pure]
fn calculate(x: f64, y: f64) -> f64 {
    return x * y + y
}
```

Multiple entries on an item accumulate. Conflicting entries produce a discrepancy.

### Custom Entries

```ish
@[validated] @[sanitized] let user_input: String = clean(raw)
@[thread_safe] let config: AppConfig = load_config()
```

---

## Entry Type Definition Syntax

Custom entry types are defined with `entry type` blocks:

```ish
entry type validated {
    applies_to: [variable, property],
}

entry type sanitized {
    applies_to: [variable, property],
    requires: @[type(_)],
}

entry type thread_safe {
    applies_to: [variable, property],
    implies: [@[immutable]],
    conflicts: [@[object_type(open)]],
}
```

Entry types support inheritance via `extends`.

---

## Native Syntax Equivalence

Native syntax and entry annotations are fully interchangeable. The following table shows the mapping:

| Native syntax | Entry annotation | Applies to |
|--------------|-----------------|------------|
| `: T` | `@[type(T)]` | variable, parameter, property |
| `-> T` | `@[return_type(T)]` | function |
| `?` suffix | `@[nullable]` | variable, property |
| `mut` | `@[mutable]` | variable, property |
| `async` | `@[async]` | function, block |
| Error union types | `@[throws(E)]` | function |
| `pub(...)` | `@[visibility(pub(...))]` | variable, function, type, module |

---

## Parameterized Feature States

Each feature that participates in the assurance ledger has a set of valid states. These are not merely on/off; they describe *how* and *when* the feature is checked.

The three base states common to most features are:

| State | Meaning |
|-------|---------|
| `optional` | The feature is not required. If present, it is checked. If absent, no discrepancy. |
| `live` | The feature is required. Checked during live audit (execution time). |
| `pre` | The feature is required. Checked during pre-audit (declaration time). |

Some features have additional feature-specific states. For example:

- `overflow` takes a behavior parameter: `wrapping`, `panicking`, or `saturating`
- `implicit_conversions` takes `allow` or `deny`
- `undeclared_errors` takes a list of allowed entry types (e.g., `@standard[@undeclared_errors(@Error)]`)

When a feature appears in a standard without a parenthetical, it defaults to `live`, except for features that only apply to function declarations, which default to `pre`.

When a standard extends another and overrides a feature, the new state completely replaces the old.

---

## Feature State Table

> **Note:** This table is a placeholder. The valid states and defaults have not been fully reviewed.

| Feature | Standard state | Entry annotation | Native syntax | Applies to |
|---------|---------------|-----------------|---------------|------------|
| Type annotations | `types(optional\|live\|pre)` | `@[type(T)]` | `: T` | variable, parameter, property |
| Return type | (part of `types`) | `@[return_type(T)]` | `-> T` | function |
| Null safety | `null_safety(optional\|live\|pre)` | `@[nullable]`, `@[non_null]` | `?` suffix | variable, property |
| Mutability | `immutability(optional\|live\|pre)` | `@[mutable]`, `@[immutable]` | `mut` keyword | variable, property |
| Numeric overflow | `overflow(optional\|wrapping\|panicking\|saturating × live\|pre)` | `@[overflow(wrapping)]`, etc. | — | variable, property |
| Numeric precision | `numeric_precision(optional\|live\|pre)` | `@[numeric(exact)]` | explicit type | variable, property |
| Implicit conversions | `implicit_conversions(allow\|deny × live\|pre)` | — | — | scope-level only |
| Undeclared errors | `undeclared_errors(any\|typed\|none)` | — | — | scope-level only |
| Exhaustive matching | `exhaustiveness(optional\|live\|pre)` | — | — | scope-level only |
| Unused variables | `unused_variables(optional\|live\|pre)` | `@[allow_unused]` | `_` prefix | variable |
| Unreachable code | `unreachable_statements(optional\|live\|pre)` | `@[allow_unreachable]` | — | statement |
| Memory model | `memory_model(optional\|gc\|rc\|owned\|stack\|auto × live\|pre)` | `@[memory(stack)]`, etc. | — | variable |
| Polymorphism | `polymorphism_strategy(optional\|auto\|none\|enum\|mono\|vtable\|assoc × live\|pre)` | `@[polymorphism(vtable)]`, etc. | — | type, function |
| Open/closed objects | `open_closed_objects(optional\|live\|pre)` | `@[object_type(open)]`, `@[object_type(closed)]` | — | type, variable |
| Visibility | `visibility(optional\|live\|pre)` | `@[visibility(pub(...))]` | `pub(...)` | variable, function, type, module |
| Sync/Async | `sync_async(optional\|live\|pre)` | `@[sync]`, `@[async]` | `async` keyword | function, block |
| Blocking | `blocking(optional\|allow\|deny × live\|pre)` | `@[blocking(allow)]`, `@[blocking(deny)]` | — | function, block |
| Pure functions | `pure_functions(optional\|live\|pre)` | `@[pure]`, `@[mutates_state]` | — | function |

---

## Built-In Standards

The built-in standards are defined in the standard library, not hardcoded into the language:

```ish
standard streamlined []

standard cautious [
    types(live),
    null_safety(live),
    immutability(live),
]

standard rigorous extends cautious [
    types(pre),
    null_safety(pre),
    immutability(pre),
    overflow(panicking, pre),
    numeric_precision(pre),
    implicit_conversions(deny, pre),
    undeclared_errors(none),
    exhaustiveness(pre),
    unused_variables(pre),
    unreachable_statements(pre),
    memory_model(auto, pre),
    polymorphism_strategy(auto, pre),
    open_closed_objects(pre),
    visibility(pre),
    sync_async(pre),
    blocking(deny, pre),
    pure_functions(pre),
]
```

---

## Cross-Scope Standard Interactions

The standards of a module apply to statements within that module's lexical scope, and therefore govern what entries are entailed to items by statements in that module.

When a statement in module A passes a variable V to a function F declared in module B:
- V carries the entries that A's standards entailed.
- F's formal parameters require the entries that B's standards entailed.
- The audit verifies that there is no discrepancy between V's entries and the entries F expects.

Modules of different assurance levels can call each other as long as there are no discrepancies between what is passed and what is expected.

---

## Discrepancy Reporting

When the ledger detects a discrepancy, the error message includes an audit trail:

```
Discrepancy: type mismatch

  Entry @type(x, i32) at line 8 conflicts with @type(x, bool) at line 10.

  Audit trail:
    - @standard[cautious] at line 1 requires types(live)
    - `let x: i32 = 7` at line 8 entailed entry @type(x, i32)
    - `x = false` at line 10 entailed entry @type(x, bool)
    - @type(i32) and @type(bool) are incompatible
```

```
Discrepancy: undeclared error type

  Function `fetch` at line 12 may throw NetworkError, but the active standard
  requires undeclared_errors(none) and no @[throws(NetworkError)] declaration was found.

  Audit trail:
    - @standard[api_safety] at line 1 requires undeclared_errors(none)
    - `fetch` calls `http.get()` which throws NetworkError
    - No `throws NetworkError` on `fetch` signature
```

References in discrepancy messages include project and file when the conflicting statement originates from a different project or file.

---

## Complete Example

```ish
// ── Module-level standard ──────────────────────────
@standard[cautious]
@standard[overflow(saturating), undeclared_errors(typed)]

// ── Standard definition ────────────────────────────
standard api_safety extends cautious [
    undeclared_errors(none),
    null_safety(pre),
]

// ── Custom entry types ─────────────────────────────
entry type validated {
    applies_to: [variable, property],
}

entry type rate_limited {
    applies_to: [function],
}

// ── Type definitions ───────────────────────────────
type User = {
    id: i64,
    mut name: String,
    email: String,
    @[nullable] phone: String?,
};

// ── Functions ──────────────────────────────────────
@standard[api_safety]
@[rate_limited]
fn get_user(id: i64) -> User | NotFoundError {
    let raw = db.query("SELECT * FROM users WHERE id = ?", id)
    return validate(User, raw)
}

fn critical_calculation(values: List<f64>) -> f64 {
    @standard[rigorous]
    {
        let mut sum: f64 = 0.0
        let mut count: u64 = 0
        for v in values {
            sum = sum + v
            count = count + 1
        }
        return sum / count.to_f64()
    }
}
```

---

## Open Questions

- [ ] **Custom entry type trackability.** Custom entry types that should be required by standards need a mechanism to register as trackable features. The specific mechanism is TBD.
- [ ] **Custom entry discrepancy messages.** When a standard requires a custom entry and a variable/function doesn't have it, should the discrepancy message differ from built-in feature discrepancies? Format TBD.

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
- [docs/spec/reasoning.md](reasoning.md)
- [GLOSSARY.md](../../GLOSSARY.md)
- [docs/project/proposals/assurance-ledger-syntax.md](../project/proposals/assurance-ledger-syntax.md)
