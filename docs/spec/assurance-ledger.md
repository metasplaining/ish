---
title: Assurance Ledger
category: spec
audience: [all]
status: draft
last-verified: 2026-03-19
depends-on: [docs/project/proposals/assurance-ledger-syntax.md, docs/spec/types.md]
---

# Assurance Ledger

The assurance ledger is ish's unified consistency-checking system. It manages the full spectrum from low-assurance code (minimal annotations, dynamic checking) to high-assurance code (extensive annotations, static checking). The system has four core concepts:

- **Standards** configure what the ledger checks within a scope.
- **Entries** record facts about individual items (variables, properties, functions, types).
- **Audits** check entries for consistency — either at build time or at execution time.
- **Discrepancies** are conflicts detected by the audit — two entries are incompatible, or a required entry is missing.

The ledger is ish's central mechanism for managing code quality. Features traditionally considered part of the "type system" — type checking, null safety, mutability, overflow behavior — are all ledger features. The type system (see [docs/spec/types.md](types.md)) specifies *what values are valid*; the ledger specifies *what checks are performed, when, and how strictly*.

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
        let mut sum: f64 = 0.0
        // ...
    }
}
```

### Entries

An **entry** is a fact recorded about an item in the ledger. Entries describe types, behaviors, and constraints on values, properties, functions, and types.

Entries can be created by native syntax or by explicit annotation. These are equivalent:

```ish
let mut x: i32 = 7
@[mutable] @[type(i32)] let x = 7
```

Native syntax is preferred for readability. The annotation form exists for programmatic generation and for features that lack native syntax.

#### Value Entries

Three special entry kinds track type information on values:

- **Actual-value entry** (`actual_value`): The exact value at a given point in execution. Replaces the concept of literal types — instead of a value "having" a literal type, the ledger records what value it actually holds.
- **Possible-values entry** (`possible_values`): The set of values a variable might hold at a given point. Updated by inference, narrowing, and control flow analysis.
- **Allowed-values entry** (`allowed_values`): The set of values a variable is *permitted* to hold, as declared by a type annotation. Used for type compatibility checking.

```ish
let x: i32 = 5
// Entries on x:
//   actual_value(5)
//   possible_values(i32)
//   allowed_values(i32)    — from the : i32 annotation

let y = 5
// Entries on y:
//   actual_value(5)
//   possible_values: deferred (resolved on demand)
//   no allowed_values entry (no annotation)
```

### Audit

The **audit** is the process by which the ledger checks entries for consistency. There are two audit modes:

- **Runtime audit** — occurs at execution time. Features with `type_audit` set to `runtime` in the active standard are checked during runtime audit.
- **Build audit** — occurs at build time (declaration time). Features with `type_audit` set to `build` in the active standard are checked during build audit.

Feature annotations have a separate dimension controlling whether they are required:

- `type_annotations` set to `optional` — the annotation is not required. If an entry for the feature is present, it is still checked; if absent, no discrepancy is raised.
- `type_annotations` set to `required` — the annotation is required. A missing entry produces a discrepancy.

Type checking (type compatibility on assignment, function call, and return) is integrated into the audit. The VM always checks type compatibility; the standard determines *when* (runtime vs. build) and *how strictly* (optional vs. required annotations).

### Discrepancies

A **discrepancy** is a conflict detected by the audit. Two entries on the same item are incompatible, or a required entry is missing. Discrepancy reporting includes an audit trail tracing back through the chain of standards and statements that contributed to the conflict.

---

## Entry Types

Entry types define the schema for entries — what an entry means, what items it can apply to, and what other entries it implies or conflicts with.

### Built-In Entry Types

The following entry types are pre-registered at startup:

| Entry type | Parent | Required properties | Description |
|-----------|--------|-------------------|-------------|
| `Error` | — | `message: String` | Error entry — marks a value as an error |
| `CodedError` | `Error` | `code: String` | Error with a well-known code |
| `SystemError` | `CodedError` | — | Interpreter-generated error |
| `TypeError` | `CodedError` | — | Type mismatch or type system violation |
| `ArgumentError` | `CodedError` | — | Incorrect argument count or type |
| `FileError` | `CodedError` | — | File system operation failure |
| `FileNotFoundError` | `FileError` | — | File does not exist |
| `PermissionError` | `FileError` | — | Permission denied |
| `Mutable` | — | — | Marks a variable/property as mutable |
| `Type` | — | — | Structural type entry |
| `Open` | — | — | Object is open to extra properties |
| `Closed` | — | — | Object has exactly declared properties |

Entry types support inheritance via `extends`. The `CodedError` entry type extends `Error`, inheriting its `message: String` requirement and adding `code: String`. `SystemError` extends `CodedError`. Domain error subtypes (`TypeError`, `ArgumentError`, `FileError`, etc.) extend `CodedError` to classify errors by source. See [docs/spec/errors.md](errors.md) for the full error hierarchy and throw audit semantics.

### Custom Entry Types

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

---

## Type Checking as a Ledger Feature

Type checking is not a separate system — it is a feature managed by the assurance ledger. The audit checks type compatibility on three kinds of statements:

- **Assignment**: The value's type must be compatible with the variable's `allowed_values` entry (the declared type annotation).
- **Function call**: Each argument's type must be compatible with the corresponding parameter's declared type.
- **Return statement**: The returned value's type must be compatible with the function's declared return type.

The active standard's `types` feature controls the strictness:

| `type_annotations` | `type_audit` | Behavior |
|-------|------|----------|
| `optional` | `runtime` | Type annotations are optional. When present, checked at runtime. |
| `required` | `runtime` | Type annotations are required (omitting one is a discrepancy). Checked at runtime. |
| `required` | `build` | Type annotations are required and checked at build time. |

### Type Compatibility Rules

The type compatibility checker determines when one type is assignable to another:

| Type category | Compatibility rule |
|--------------|-------------------|
| Simple types | Match by name (e.g., `i32` to `i32`) |
| Union types | Value matches if it matches any member of the union |
| Optional types | Inner type or `null` |
| Object types | Structural — required properties present with compatible types |
| Function types | Parameter count matches, parameter types compatible, return type compatible |
| List types | Element type compatible |
| Tuple types | Same length, position-by-position type compatibility |
| Intersection types | Value satisfies all constituent types |

---

## Type Narrowing as Entry Maintenance

Type narrowing is not a separate pass — it is the natural consequence of the ledger maintaining entry sets through control flow. After every statement, the ledger produces revised entry sets that reflect the information gained from that statement.

### Narrowing Rules

| Condition | True branch | False branch |
|-----------|------------|-------------|
| `is_type(x, T)` | `x` narrowed to `T` | `T` excluded from `x` |
| `x != null` | `null` excluded from `x` | `x` narrowed to `null` |
| `x == null` | `x` narrowed to `null` | `null` excluded from `x` |

### Branch Merge

When branches converge (after if/else, at the end of a loop, etc.), the ledger **unions** the entry sets from all branches. This means:

- If `x` was narrowed to `String` in the true branch and `i32` in the false branch, after the if/else, `x`'s possible-values entry is `String | i32`.
- If `x` was narrowed to `String` in the true branch and the false branch returns/throws, `x`'s possible-values entry remains `String` after the if.

### Entry Restoration

On branch exit, entries are restored to their pre-branch state, then updated with the merged result. This prevents narrowing from one branch from leaking into code after the branch.

### Nested Narrowing

Narrowing composes: narrowing inside a nested branch further refines the outer branch's narrowing. The ledger maintains a stack of saved entry states for nested branches.

---

## Standard Definition Syntax

Standards are defined with the `standard` keyword:

```ish
standard cautious [
    types(required, runtime),
    null_safety(required, runtime),
    immutability(required, runtime),
]

standard api_safety extends cautious [
    checked_exceptions(required, build),
    null_safety(required, build),
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
@standard[overflow(saturating), checked_exceptions(required, runtime)]

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

Each feature that participates in the assurance ledger has states along two independent dimensions:

### Annotation Dimension (`type_annotations`)

Controls whether annotations are required:

| State | Meaning |
|-------|---------|
| `optional` | The annotation is not required. If present, it is checked. If absent, no discrepancy. |
| `required` | The annotation is required. A missing annotation produces a discrepancy. |

### Audit Dimension (`type_audit`)

Controls when checking occurs:

| State | Meaning |
|-------|---------|
| `runtime` | Checked during runtime audit (execution time). |
| `build` | Checked during build audit (declaration time / compile time). |

Some features have additional feature-specific states. For example:

- `overflow` takes a behavior parameter: `wrapping`, `panicking`, or `saturating`
- `implicit_conversions` takes `allow` or `deny`
- `undeclared_errors` takes a list of allowed entry types (e.g., `@standard[@undeclared_errors(@Error)]`)

When a feature appears in a standard without explicit annotation/audit dimensions, it defaults to `type_annotations(required)` and `type_audit(runtime)`, except for features that only apply to function declarations, which default to `type_audit(build)`.

When a standard extends another and overrides a feature, the new state completely replaces the old.

---

## Feature State Table

> **Note:** This table is a placeholder. The valid states and defaults have not been fully reviewed. Feature states use two dimensions: `type_annotations` (`optional` | `required`) and `type_audit` (`runtime` | `build`).

| Feature | Standard state | Entry annotation | Native syntax | Applies to |
|---------|---------------|-----------------|---------------|------------|
| Type annotations | `types(optional\|required, runtime\|build)` | `@[type(T)]` | `: T` | variable, parameter, property |
| Return type | (part of `types`) | `@[return_type(T)]` | `-> T` | function |
| Null safety | `null_safety(optional\|required, runtime\|build)` | `@[nullable]`, `@[non_null]` | `?` suffix | variable, property |
| Mutability | `immutability(optional\|required, runtime\|build)` | `@[mutable]`, `@[immutable]` | `mut` keyword | variable, property |
| Open/closed objects | `open_closed_objects(optional\|required, runtime\|build)` | `@[Open]`, `@[Closed]` | — | type, variable |
| Numeric overflow | `overflow(optional\|wrapping\|panicking\|saturating, runtime\|build)` | `@[overflow(wrapping)]`, etc. | — | variable, property |
| Numeric precision | `numeric_precision(optional\|required, runtime\|build)` | `@[numeric(exact)]` | explicit type | variable, property |
| Implicit conversions | `implicit_conversions(allow\|deny, runtime\|build)` | — | — | scope-level only |
| Undeclared errors | `undeclared_errors(any\|typed\|none)` | — | — | scope-level only |
| Exhaustive matching | `exhaustiveness(optional\|required, runtime\|build)` | — | — | scope-level only |
| Unused variables | `unused_variables(optional\|required, runtime\|build)` | `@[allow_unused]` | `_` prefix | variable |
| Unreachable code | `unreachable_statements(optional\|required, runtime\|build)` | `@[allow_unreachable]` | — | statement |
| Memory model | `memory_model(optional\|gc\|rc\|owned\|stack\|auto, runtime\|build)` | `@[memory(stack)]`, etc. | — | variable |
| Polymorphism | `polymorphism_strategy(optional\|auto\|none\|enum\|mono\|vtable\|assoc, runtime\|build)` | `@[polymorphism(vtable)]`, etc. | — | type, function |
| Visibility | `visibility(optional\|required, runtime\|build)` | `@[visibility(pub(...))]` | `pub(...)` | variable, function, type, module |
| Sync/Async | `sync_async(optional\|required, runtime\|build)` | `@[sync]`, `@[async]` | `async` keyword | function, block |
| Blocking | `blocking(optional\|allow\|deny, runtime\|build)` | `@[blocking(allow)]`, `@[blocking(deny)]` | — | function, block |
| Pure functions | `pure_functions(optional\|required, runtime\|build)` | `@[pure]`, `@[mutates_state]` | — | function |

---

## Built-In Standards

The built-in standards are defined in the standard library, not hardcoded into the language:

```ish
standard streamlined []

standard cautious [
    types(required, runtime),
    null_safety(required, runtime),
    immutability(required, runtime),
]

standard rigorous extends cautious [
    types(required, build),
    null_safety(required, build),
    immutability(required, build),
    overflow(panicking, build),
    numeric_precision(required, build),
    implicit_conversions(deny, build),
    undeclared_errors(none),
    exhaustiveness(required, build),
    unused_variables(required, build),
    unreachable_statements(required, build),
    memory_model(auto, build),
    polymorphism_strategy(auto, build),
    open_closed_objects(required, build),
    visibility(required, build),
    sync_async(required, build),
    blocking(deny, build),
    pure_functions(required, build),
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
    - @standard[cautious] at line 1 requires types(required, runtime)
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
    null_safety(required, build),
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
- [ ] **Entry type definition syntax.** The `entry type` block syntax shown above is provisional. The exact fields (`applies_to`, `requires`, `implies`, `conflicts`) and their semantics need to be formalized.

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/types.md](types.md)
- [docs/spec/errors.md](errors.md)
- [docs/architecture/vm.md](../architecture/vm.md)
- [docs/user-guide/assurance-levels.md](../user-guide/assurance-levels.md)
- [docs/ai-guide/orientation.md](../ai-guide/orientation.md)
