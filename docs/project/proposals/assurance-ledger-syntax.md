---
title: "Proposal: Assurance Ledger — Standards, Entries, and Complete Syntax"
category: proposal
audience: [all]
status: accepted
last-verified: 2026-03-14
depends-on: [docs/project/proposals/ledger-system-syntax.md, docs/project/proposals/agreement-metaphor-and-syntax.md, docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/execution.md, docs/spec/memory.md, docs/spec/polymorphism.md, docs/spec/reasoning.md, GLOSSARY.md]
---

# Proposal: Assurance Ledger — Standards, Entries, and Complete Syntax

*Follow-on to [ledger-system-syntax.md](ledger-system-syntax.md). Generated from `agreement` on 2026-03-14.*

**Decisions carried forward from prior proposals:**

| Decision | Resolution |
|----------|-----------|
| Metaphor | Ledger (accounting/bookkeeping) |
| System name | **Assurance Ledger** (shortened: "the ledger") |
| Encumbrance continuum | Renamed to **assurance level** |
| Block-scoped annotations | Called **standards**; sigil `@standard[...]` |
| Item-scoped annotations | Called **entries**; sigil `@[entry(params)]` |
| Visual differentiation | Standards and entries use different sigils |
| Annotation parameter syntax | `@[a(b)]` not `@[a: b]` |
| Native syntax features | types (`:`), mutability (`mut`), async (`async`), throws (`throws`) |
| Profiles | **No profiles.** Standards serve that role. |
| Standard states | **Parameterized**, not on/off. Each standard enumerates its valid states. |
| Audit mode | Parenthetical parameter on the standard state |
| Custom entry types | Block syntax (`entry type`); cannot be used as standards |
| Standard definition | `standard name extends base [...]` syntax |

---

## Complete Terminology

Before diving into syntax, here is the final terminology table incorporating all decisions.

| Role | Term | Definition |
|------|------|-----------|
| System name | **Assurance Ledger** | The unified consistency-checking system. Shortened to "the ledger" in running text. |
| Unit of knowledge | **Entry** | A recorded fact about a specific item. Each statement entails entries. |
| Inconsistency | **Discrepancy** | When entries conflict with each other or with the active standard. |
| Checking action | **Audit** | The processor audits the ledger for discrepancies. |
| Block-scoped configuration | **Standard** | A named configuration that governs what the ledger tracks within its scope. Applied with `@standard[name]`. |
| Item-scoped annotation | **Entry annotation** | An annotation that records an entry for a specific item. Applied with `@[entry(params)]`. |
| Declaration-time checking | **Pre-audit** | The ledger is audited when a scope is declared, before any statements execute. |
| Execution-time checking | **Live audit** | The ledger is audited as each statement is executed. |
| Configurable continuum | **Assurance level** | The degree of checking enabled. Streamlined = low assurance. Rigorous = high assurance. |
| User-defined facts | **Custom entry type** | A developer-defined entry type with associated rules. |
| Named standard combination | **Standard** (with `extends`) | A named combination of feature configurations. Defined with the `standard` keyword. |
| Streamlined ish | **Low-assurance ish** | The end of the continuum where code is minimal and dynamically checked. |
| Encumbered ish | **High-assurance ish** | The end of the continuum where code is heavily annotated and statically checked. |

---

## Feature: Standard Definition and Application

### How Standards Work

A **standard** is a named set of feature configurations. Each feature in the standard specifies a **state** — not merely on/off, but a parameterized value from an enumerated set of valid states.

Standards:
- Are defined with the `standard` keyword
- Can extend other standards (inheriting and overriding feature configurations)
- Are applied to lexical scopes with `@standard[name]`
- At inner scopes, override outer-scope standards feature-by-feature
- Are first-class definitions that can be distributed in modules and packages

### Standard Definition Syntax

```ish
standard streamlined []

standard cautious [
    types(live),                      // require type annotations; live audit
    null_safety(live),                // require explicit nullability; live audit
    immutability(live),               // require explicit mut/immutable; live audit
]

standard rigorous extends cautious [
    types(pre),                       // upgrade types to pre-audit
    null_safety(pre),                 // upgrade null safety to pre-audit
    immutability(pre),                // upgrade immutability to pre-audit
    overflow(panicking, pre),         // require explicit overflow behavior; pre-audit
    numeric_precision(pre),           // require exact numeric types; pre-audit
    exhaustiveness(pre),              // require exhaustive matches; pre-audit
    unused_variables(pre),            // flag unused variables; pre-audit
    unreachable_statements(pre),      // flag unreachable code; pre-audit
    memory_model(pre),                // require explicit memory model; pre-audit
    polymorphism_strategy(pre),       // require explicit strategy; pre-audit
    checked_exceptions(pre),          // require declared error types; pre-audit
    visibility(pre),                  // require explicit visibility; pre-audit
    open_closed_objects(pre),         // require explicit open/closed; pre-audit
]

// User-defined standard
standard my_team extends cautious [
    overflow(saturating),             // add overflow tracking, default audit mode
    undeclared_errors(any),           // allow any error without declaration
    immutability(pre),                // upgrade immutability to pre-audit
]
```

### Standard Application Syntax

```ish
// At module level
@standard[rigorous]

// At block level
@standard[cautious]
{
    // ...
}

// At function level
@standard[my_team]
fn process(data: Input) -> Output {
    // ...
}
```

### Standard Override Mechanics

When an inner scope applies a standard, its feature configurations override the outer scope's on a per-feature basis. Features not mentioned in the inner standard retain the outer scope's configuration.

```ish
@standard[rigorous]                     // outer: everything pre-audited

fn prototype() {
    @standard[streamlined]              // inner: everything relaxed
    {
        let x = 42;                     // no type required
        x = "changed";                  // no type checking
    }

    // Back to rigorous outside the block
    let y: i32 = 7;                     // type required
}
```

### Individual Feature Override

When a developer needs to adjust a single feature without applying a full standard, they can use `@standard[...]` with just that feature:

```ish
@standard[rigorous]

fn mostly_rigorous() {
    // Relax just one feature
    @standard[types(optional)]          // types become optional in this block
    {
        let x = 42;                     // no type required
        let y: i32 = 7;                 // type still checked if provided
    }
}
```

This works because `@standard[feature(state)]` is syntactically the same as applying a standard — it's just an anonymous inline standard with one feature override. The outer standard's other features remain in effect.

### Issues to Watch Out For

1. **Composability.** When standards extend and override each other, the resolution order must be well-defined and predictable. The rule is: inner scope wins, and within a scope, later `@standard[...]` annotations override earlier ones for the same feature.

2. **Feature state validation.** Not every state is valid for every feature. For example, `overflow(panicking)` is valid but `types(panicking)` is not. The processor must validate feature states at definition time.

3. **Cross-module standard inheritance.** Standards from imported modules should be usable in `extends` clauses. This creates a dependency on the module system.

4. **Error messages.** When a discrepancy is reported, the error must trace back through the standard chain: which standard required the feature, which statement entailed the conflicting entry.

### Critical Analysis

#### Alternative A: Standards as shown above (recommended)

Standards are top-level definitions with `extends` and bracket-list syntax.

**Pros:**
- Clean, dedicated syntax. Impossible to confuse with other language constructs.
- `extends` is familiar from class inheritance in many languages.
- The bracket-list syntax mirrors array/object literals, which developers already know.
- Standard definitions are greppable and easy to find in a codebase.

**Cons:**
- Introduces a new top-level keyword (`standard`) that must be reserved.
- The bracket-list syntax uses `[...]` which could be confused with list literals in some contexts (though the `standard` keyword prefix makes this unambiguous).

#### Alternative B: Standards as annotated objects

```ish
@[standard]
let rigorous = {
    types: pre,
    null_safety: pre,
    immutability: pre,
};
```

**Pros:**
- No new keyword. Standards are just objects with a special annotation.
- Consistent with the language's data-as-code philosophy.

**Cons:**
- The object literal syntax doesn't naturally express `extends`.
- Validation of feature names and states is harder — objects can have arbitrary properties.
- Loses the declarative feel. A standard should feel like a *definition*, not an assignment.

#### Alternative C: Standards as function-like declarations

```ish
standard rigorous(base: cautious) {
    types = pre;
    null_safety = pre;
}
```

**Pros:**
- Familiar if you think of `base` as a parameter.
- Could support conditional logic inside the body (e.g., platform-specific standards).

**Cons:**
- Over-complicated for what is essentially a static configuration.
- Mixes imperative and declarative styles.
- The `base:` parameter is non-obvious compared to `extends`.

### Recommendation: Alternative A

The dedicated `standard` keyword with `extends` and bracket-list syntax (Alternative A) is the clearest and most appropriate for a static configuration construct.

### Decisions

**Decision:** Standard definition syntax — dedicated keyword with brackets (A, recommended), annotated objects (B), or function-like declarations (C)?
--> A - dedicated keyword with brackets

**Decision:** Can standards be defined inside functions/blocks, or only at module level?
--> inside functions/blocks

**Decision:** Should the language ship with built-in standards (streamlined, cautious, rigorous) or should these be defined in the standard library?
--> Standard library.

---

## Feature: Parameterized Feature States

### The State Model

Each feature that participates in the assurance ledger has a set of **valid states**. These are not merely on/off; they describe *how* and *when* the feature is checked.

The three base states common to most features are:

| State | Meaning |
|-------|---------|
| `optional` | The feature is not required. If present, it is checked. If absent, no discrepancy. |
| `live` | The feature is required. Checked during live audit (execution time). |
| `pre` | The feature is required. Checked during pre-audit (declaration time). |

Some features have additional feature-specific states that supplement the base state. These are passed as additional parameters.

### Feature-Specific States

#### Overflow behavior

The `overflow` feature requires a behavior parameter in addition to the audit mode:

```ish
overflow(wrapping)             // require overflow annotation; wrapping is the default; default audit
overflow(wrapping, pre)        // wrapping default; pre-audit
overflow(panicking, live)      // panicking default; live audit
overflow(saturating, pre)      // saturating default; pre-audit
overflow(optional)             // do not require overflow annotation
```

When `overflow` is active with a default behavior (e.g., `wrapping`), variables that don't explicitly specify overflow behavior are assumed to use the default. Variables that do specify a different behavior are checked as entries.

#### Error declarations

```ish
undeclared_errors(pre)         // functions must declare thrown errors; pre-audit
undeclared_errors(live)        // functions must declare thrown errors; live audit
undeclared_errors (optional)   // error declarations not required
```

#### Implicit conversions

```ish
implicit_conversions(allow)    // safe widening conversions happen implicitly
implicit_conversions(deny)     // all conversions must be explicit
implicit_conversions(deny, pre)  // explicit conversions required; pre-audit
```

### How Parameterized States Compose in `extends`

When a standard extends another and overrides a feature, the new state completely replaces the old:

```ish
standard cautious [
    overflow(wrapping, live),    // wrapping, live audit
]

standard stricter extends cautious [
    overflow(panicking, pre),    // completely replaces: now panicking, pre-audit
]
```

When a standard extends another and does *not* mention a feature, the parent's state is inherited unchanged.

### Issues to Watch Out For

1. **State validation.** The processor must know which states are valid for each feature. Invalid combinations (e.g., `types(wrapping)`) must produce clear errors at standard definition time.

2. **Default audit mode.** When a feature state omits the audit mode (e.g., `types(live)` vs just `types`), what is the default? The proposal uses the convention that a bare feature name (e.g., `types` with no parenthetical) means the feature's natural default audit mode (which should be specified per-feature).

3. **User-defined features.** Custom entry types define their own valid states. For simple markers (like `@[validated]`), only `optional`/`live`/`pre` may apply. For parameterized markers, the entry type definition must enumerate valid states.

### Complete Feature State Table

The following table lists every built-in ledger feature, its valid states, and its default behavior in the `streamlined` and `rigorous` built-in standards.

| Feature | Valid states | Default in `streamlined` | Default in `rigorous` |
|---------|-------------|--------------------------|----------------------|
| `types` | `optional`, `live`, `pre` | `optional` | `pre` |
| `null_safety` | `optional`, `live`, `pre` | `optional` | `pre` |
| `immutability` | `optional`, `live`, `pre` | `optional` | `pre` |
| `overflow` | `optional`, `wrapping`, `panicking`, `saturating` × `live`/`pre` | `optional` | `panicking(pre)` |
| `numeric_precision` | `optional`, `live`, `pre` | `optional` | `pre` |
| `implicit_conversions` | `allow`, `deny`, `deny(pre)` | `allow` | `deny(pre)` |
| `checked_exceptions` | `optional`, `live`, `pre` | `optional` | `pre` |
| `undeclared_errors` | `any`, `typed`, `none` | `any` | `none` |
| `exhaustiveness` | `optional`, `live`, `pre` | `optional` | `pre` |
| `unused_variables` | `optional`, `live`, `pre` | `optional` | `pre` |
| `unreachable_statements` | `optional`, `live`, `pre` | `optional` | `pre` |
| `memory_model` | `optional`, `gc`, `rc`, `owned`, `stack`, `auto` × `live`/`pre` | `optional` | `auto(pre)` |
| `polymorphism_strategy` | `optional`, `auto`, `none`, `enum`, `mono`, `vtable`, `assoc` × `live`/`pre` | `optional` | `auto(pre)` |
| `open_closed_objects` | `optional`, `live`, `pre` | `optional` | `pre` |
| `visibility` | `optional`, `live`, `pre` | `optional` | `pre` |
| `sync_async` | `optional`, `live`, `pre` | `optional` | `pre` |
| `blocking` | `optional`, `allow`, `deny` × `live`/`pre` | `optional` | `deny(pre)` |
| `pure_functions` | `optional`, `live`, `pre` | `optional` | `pre` |

### Decisions

**Decision:** What is the default audit mode when a feature is specified without a parenthetical? (e.g., bare `types` in a standard definition — is it `live` or `pre`?)
--> All features default to live, except features that are only allowed on a function declaration, which default to pre.

**Decision:** Should `undeclared_errors` and `checked_exceptions` be separate features or combined into one feature with more states?
--> undeclared_errors should replace checked exceptions.  Note that it should take a list of allowed entry types.  For example, `@standard[@undeclared_errors(@Error)]` specifies that functions may return any value that has the `@Error` annotation without declaring it, while  `@standard[@undeclared_errors(@name(my-error))]` specifies that functions may return any value that has the `@name(my-error)` annotation without declaring it.

--> Note that I have not reviewed the complete feature state table, and it should be considered a placeholder.
---

## Feature: Entry Annotation Syntax

### Item-Level Entries

Entry annotations attach entries to specific items. They use the `@[name(params)]` sigil.

```ish
@[overflow(wrapping)] let z: u8 = 255;
@[memory(stack)] let buffer: List<u8> = alloc(1024);
@[nullable] let name: String? = null;
@[polymorphism(vtable)] type Shape = { area: () -> f64 };
```

When native syntax exists for a feature, entries can be expressed either way:

```ish
// These are equivalent:
let mut x: i32 = 7;
@[mutable] @[type(i32)] let x = 7;
```

Native syntax is preferred for readability. The annotation form exists for programmatic generation and for features that lack native syntax.

### Entries Are Cumulative

Multiple entries on an item accumulate. They never override each other. Conflicting entries produce a discrepancy.

```ish
let mut x: i32 = 7;           // entries: mutable, type(i32)
@[overflow(wrapping)] let y: u8 = 0;  // entries: type(u8), overflow(wrapping)

// ERROR: Discrepancy — conflicting overflow entries
// @[overflow(wrapping)] @[overflow(panicking)] let z: u8 = 0;
```

### Entries on Object Properties

```ish
type DatabaseRecord = {
    id: i64,
    mut name: String,                      // mutable (native syntax)
    email: String,                         // immutable by default under immutability standard
    @[nullable] age: i32?,                 // explicit nullability entry
    @[overflow(saturating)] retries: u8,   // overflow behavior for this property
};
```

### Entries on Function Signatures

```ish
@[pure]
fn calculate(x: f64, y: f64) -> f64 {
    return x * y + y;
}

@[blocking(deny)]
async fn fetch_data(url: String) -> Response throws NetworkError {
    // ...
}
```

### Custom Entries

User-defined entry types are created with `entry type` blocks:

```ish
entry type validated {
    applies_to: [variable, property],
}

entry type sanitized {
    applies_to: [variable, property],
    requires: @[type(_)],              // must have a known type
}

entry type thread_safe {
    applies_to: [variable, property],
    implies: [@[immutable]],           // thread_safe implies immutable
    conflicts: [@[object_type(open)]],  // cannot be on open objects
}
```

Usage:

```ish
@[validated] @[sanitized] let user_input: String = clean(raw);
@[thread_safe] let config: AppConfig = load_config();
```

### Issues to Watch Out For

1. **Native syntax vs. entry annotation equivalence.** The spec must define precisely which native syntax forms produce which entries. For example, `mut` produces `@[mutable]`, `: i32` produces `@[type(i32)]`, `throws NetworkError` produces `@[throws(NetworkError)]`.

2. **Entry parameter types.** Some entries take type parameters (`@[type(i32)]`), some take enum values (`@[overflow(wrapping)]`), some take no parameters (`@[pure]`). The parser must handle all forms.

3. **Custom entry type validation.** The `requires`, `implies`, and `conflicts` clauses reference other entries. These must be validated when the entry type is defined — not deferred to usage time.

### Decisions

**Decision:** Should native syntax and entry annotation forms be fully interchangeable, or should native syntax be the "canonical" form with entry annotations as a secondary mechanism?
--> Fully interchangeable

**Decision:** Should `entry type` blocks support inheritance (`extends`), similar to standards?
--> Support inheritance

---

## Feature: Complete Syntax for the Assurance Ledger

This section consolidates all syntax into a unified reference, incorporating all decisions.

### Syntax Summary Table

| Construct | Syntax | Scope |
|-----------|--------|-------|
| Apply standard to scope | `@standard[name]` | block, function, module |
| Inline feature override | `@standard[feature(state)]` | block, function, module |
| Multi-feature override | `@standard[feat1(state), feat2(state)]` | block, function, module |
| Apply entry to item | `@[entry(params)]` | variable, property, function, type, statement |
| Define a standard | `standard name [...]` | module level |
| Extend a standard | `standard name extends base [...]` | module level |
| Define an entry type | `entry type name { ... }` | module level |
| Type annotation (native) | `let x: T = ...` | variable, parameter, property |
| Mutability (native) | `let mut x = ...` | variable, property |
| Async (native) | `async fn ...` | function, block |
| Error declaration (native) | `fn f() throws E` | function |
| Nominal type (native) | `nominal type Name = T` | type |

### Complete Feature Annotation Reference

For each feature, the table shows: the standard state syntax, the entry annotation syntax, equivalent native syntax (if any), and what items the entry applies to.

| Feature | Standard state | Entry annotation | Native syntax | Applies to |
|---------|---------------|-----------------|---------------|------------|
| Type annotations | `types(optional\|live\|pre)` | `@[type(T)]` | `: T` | variable, parameter, property |
| Return type | (part of `types`) | `@[return_type(T)]` | `-> T` | function |
| Null safety | `null_safety(optional\|live\|pre)` | `@[nullable]`, `@[non_null]` | `?` suffix | variable, property |
| Mutability | `immutability(optional\|live\|pre)` | `@[mutable]`, `@[immutable]` | `mut` keyword | variable, property |
| Numeric overflow | `overflow(optional\|wrapping\|panicking\|saturating` | `@[overflow(wrapping)]`, etc. | — | variable, property |
| | `× live\|pre)` | | | |
| Numeric precision | `numeric_precision(optional\|live\|pre)` | `@[numeric(exact)]` | explicit type | variable, property |
| Implicit conversions | `implicit_conversions(allow\|deny` | — | — | — (scope-level only) |
| | `× live\|pre)` | | | |
| Checked exceptions | `checked_exceptions(optional\|live\|pre)` | `@[throws(ErrorType)]` | `throws E` | function |
| Undeclared errors | `undeclared_errors(any\|typed\|none)` | — | — | — (scope-level only) |
| Exhaustive matching | `exhaustiveness(optional\|live\|pre)` | — | — | — (scope-level only) |
| Unused variables | `unused_variables(optional\|live\|pre)` | `@[allow_unused]` | `_` prefix | variable |
| Unreachable code | `unreachable_statements(optional\|live\|pre)` | `@[allow_unreachable]` | — | statement |
| Memory model | `memory_model(optional\|gc\|rc\|owned\|stack\|auto` | `@[memory(stack)]`, etc. | — | variable |
| | `× live\|pre)` | | | |
| Polymorphism | `polymorphism_strategy(optional\|auto\|none` | `@[polymorphism(vtable)]`, etc. | — | type, function |
| | `\|enum\|mono\|vtable\|assoc × live\|pre)` | | | |
| Open/closed objects | `open_closed_objects(optional\|live\|pre)` | `@[object_type(open)]`, `@[object_type(closed)]` | — | type, variable |
| Visibility | `visibility(optional\|live\|pre)` | `@[visibility(pub(...))]` | `pub(...)` | var, fn, type, module |
| Sync/Async | `sync_async(optional\|live\|pre)` | `@[sync]`, `@[async]` | `async` keyword | function, block |
| Blocking | `blocking(optional\|allow\|deny × live\|pre)` | `@[blocking(allow)]`, `@[blocking(deny)]` | — | function, block |
| Pure functions | `pure_functions(optional\|live\|pre)` | `@[pure]`, `@[mutates_state]` | — | function |
| Nominal typing | — | `@[nominal(Name)]` | `nominal type` | type |

### Built-In Standards

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
    checked_exceptions(pre),
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

### Full Example: A Complete Module

```ish
// ── Module-level standard ──────────────────────────
@standard[cautious]

// Override specific features at module level
@standard[overflow(saturating), checked_exceptions(live)]

// ── Standard definition ────────────────────────────
standard api_safety extends cautious [
    checked_exceptions(pre),
    null_safety(pre),
    undeclared_errors(typed),
]

// ── Custom entry types ─────────────────────────────
entry type validated {
    applies_to: [variable, property],
}

entry type rate_limited {
    applies_to: [function],
}

// ── Type definitions ───────────────────────────────
type Coordinate = {
    x: f64,
    y: f64,
};

type BoundedCoordinate = {
    @[overflow(saturating)] x: u16,
    @[overflow(saturating)] y: u16,
};

nominal type UserId = i64;
nominal type SessionId = String;

type User = {
    id: UserId,
    mut name: String,
    email: String,
    @[nullable] phone: String?,
};

// ── Functions ──────────────────────────────────────

// Function uses a named standard for its scope
@standard[api_safety]
@[rate_limited]
fn get_user(id: UserId) -> User throws NotFoundError {
    let raw = db.query("SELECT * FROM users WHERE id = ?", id);
    let user = validate(User, raw);
    return user;
}

// Inline standard override: relax types for a quick helper
fn quick_format(data) {
    @standard[types(optional)]
    {
        let result = "";
        for item in data {
            result = result + " " + item;
        }
        return result;
    }
}

// High-assurance block inside a moderate-assurance module
fn critical_calculation(values: List<f64>) -> f64 {
    @standard[rigorous]
    {
        let mut sum: f64 = 0.0;
        let mut count: u64 = 0;
        for v in values {
            sum = sum + v;
            count = count + 1;
        }
        return sum / count.to_f64();
    }
}

// Entry annotations on variables
fn process_input(raw: String) -> String {
    @[validated] @[sanitized] let clean: String = sanitize(validate(raw));
    return transform(clean);
}
```

### Error Message Examples

When the ledger detects a discrepancy, the error message should trace the full provenance:

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
Discrepancy: immutable variable mutated

  Entry @immutable(x) at line 5 conflicts with mutation at line 7.

  Audit trail:
    - @standard[rigorous] at line 1 requires immutability(pre)
    - `let x: i32 = 7` at line 5 entailed entry @immutable(x) (default under immutability standard)
    - `x = 42` at line 7 attempts to mutate x
```

```
Discrepancy: undeclared error type

  Function `fetch` at line 12 may throw NetworkError, but the active standard
  requires checked_exceptions(pre) and no @[throws(NetworkError)] declaration was found.

  Audit trail:
    - @standard[api_safety] at line 1 requires checked_exceptions(pre)
    - `fetch` calls `http.get()` which throws NetworkError
    - No `throws NetworkError` on `fetch` signature
```

--> Note that references must include project and file, if the conflicting statement does not come from the same project or file.

---

## Feature: Cross-Scope Standard Interactions

### Question: What happens at module boundaries?

When a high-assurance module calls a function from a low-assurance module (or vice versa), the ledger must manage the boundary. Several approaches:

#### Alternative A: Caller's standard governs (recommended)

The calling code's active standard determines what is checked. If the caller requires `types(pre)`, then the function being called must have type information available — either declared or inferrable from its package metadata.

**Pros:**
- Simple mental model: "my standard applies to my code."
- High-assurance code can safely call low-assurance libraries as long as type information is available.
- No need for the callee to know about the caller's standard.

**Cons:**
- If the called module provides no type information (pure streamlined), the caller's `types(pre)` standard cannot be satisfied, producing a discrepancy.
- This could prevent high-assurance code from using streamlined libraries.

#### Alternative B: Explicit boundary annotation

A special entry marks a boundary where standards change:

```ish
@[assurance_boundary(trust)]
let result = streamlined_lib.compute(data);
```

**Pros:**
- Explicit and auditable. Every boundary crossing is visible in the code.
- The developer chooses whether to trust the external code.

**Cons:**
- Verbose. Every call to a lower-assurance module needs an annotation.
- "Trust" is a loaded term in security contexts.

#### Alternative C: Standard includes boundary policy

The standard definition specifies how to handle cross-assurance boundaries:

```ish
standard my_team extends cautious [
    types(pre),
    boundary(trust_lower),     // allow calls to lower-assurance code
]
```

**Pros:**
- Configured once per standard, not per call site.
- Teams can choose their boundary policy centrally.

**Cons:**
- Coarse-grained — all lower-assurance modules are treated the same.
- Hides individual boundary decisions.

### Decisions

**Decision:** How should cross-assurance-level boundaries be handled? Alternatives: caller's standard governs (A), explicit boundary annotations (B), or standard-level boundary policy (C)?
--> The standards of a module apply to statements within that module's lexical scope, and therefore govern what entries are entailed to items by statements in that module.  When a statement in module A attempts to pass a variable V that it declared to a function F declared in module B, V is going to have those entries entailed that A chose.  The formal parameters of F are going to require those entries that B chose.  The audit will verify that there is no discrepancy between V's entries and the entries that F is expecting.  So modules of different assurance levels can call each other, as long as there are no discrepancies between what is being passed and what is being expected. There are currently no standards defined to prevent high assurance modules from calling low assurance modules, but we should add that eventually.

---

## Feature: Relationship Between Standards and Entry Types

### Clarification: Standards vs. Entry Types

Standards and entry types serve different roles:

| | Standard | Entry type |
|-|----------|-----------|
| Purpose | Configure what the ledger checks within a scope | Define a new kind of entry that can be recorded |
| Applied with | `@standard[name]` | `@[name(params)]` |
| Scope | Block, function, or module | Individual item |
| Effect | Sets feature states (optional/live/pre + params) | Records a fact about an item |
| Definition | `standard name [...]` | `entry type name { ... }` |
| Can extend? | Yes (`extends`) | TBD (see decision below) |
| Built-in examples | `streamlined`, `cautious`, `rigorous` | `type`, `mutable`, `overflow`, `throws` |
| Custom examples | `my_team`, `api_safety` | `validated`, `sanitized`, `thread_safe` |

An entry type **cannot** be used as a standard (per prior decision). Standards configure the ledger's behavior; entry types define what facts can be recorded.

However, a standard can reference entry types in its feature list — for example, a standard could require that all functions have a `@[rate_limited]` or `@[unmetered]` entry. This is how custom entry types integrate with the standard system.

### Custom Entry Types in Standards

```ish
// Define entry types
entry type rate_limited {
    applies_to: [function],
}

entry type unmetered {
    applies_to: [function],
}

// A standard can require that one of these entries be present
standard api_production extends api_safety [
    rate_limiting(pre),            // built-in feature: require rate_limited or unmetered
]
```

This raises the question of how custom entry types register as features that standards can track.

### Issues to Watch Out For

1. **The gap between built-in features and custom entry types.** Built-in features like `types` and `immutability` have well-defined states. Custom entry types (like `@[validated]`) are simpler — typically just present/absent. How does a standard express "require this custom entry"?

2. **Registering custom entries as trackable features.** If a team defines `@[validated]`, they should be able to write a standard that requires it. But the feature name in the standard (`validated(pre)`) must be connected to the entry type definition.

### Proposed Approach

Entry types that declare `trackable: true` can be referenced in standard definitions:

```ish
entry type validated {
    applies_to: [variable, property],
    trackable: true,                    // can be required by a standard
}

// Now this works:
standard strict_data extends cautious [
    validated(pre),                     // all variables/properties must have @[validated]
]
```

For entry types that are **not** trackable (the default), they are purely informational — they record facts but standards cannot require their presence.

### Decisions

**Decision:** Should custom entry types be trackable by standards? If so, is `trackable: true` the right mechanism, or should all entry types be trackable by default?
--> Yes.  Mechanism TBD.

**Decision:** When a standard requires a custom entry, and a variable/function doesn't have that entry, what is the discrepancy message? Should it differ from built-in feature discrepancies?
--> TBD

---

## Open Questions Resolved by This Proposal

The following open questions from [docs/spec/agreement.md](docs/spec/agreement.md) and [docs/project/open-questions.md](docs/project/open-questions.md) would be answered if this proposal is accepted:

| Open question | Proposed answer |
|--------------|----------------|
| What is the syntax for marking/unmarking features? | `@standard[name]` at scope level; `@[entry(params)]` at item level. Standards define feature states. |
| What happens when an agreement is violated at build time vs. runtime? | Depends on the feature's state: `pre` = build-time discrepancy, `live` = runtime discrepancy, `optional` = no check unless entry is present. |
| How does agreement interact with boundaries between differently-encumbered code? | TBD — three alternatives proposed (caller governs, boundary annotation, standard policy). |

---

## Documentation Updates

All files from the prior proposals' documentation update lists remain affected. Additional/updated:

- [docs/spec/agreement.md](docs/spec/agreement.md) — Would be substantially rewritten or replaced with a new spec based on the assurance ledger model. "Agreement" becomes "assurance ledger"; "marked features" become "standards"; "facts" become "entries".
- [GLOSSARY.md](GLOSSARY.md) — Major updates: add Assurance Ledger, Standard, Entry, Discrepancy, Audit, Pre-audit, Live audit, Assurance level. Remove or redirect: Agreement, Marked feature, Encumbrance, Encumbered ish, Streamlined ish (→ High/Low-assurance ish).
- [docs/spec/syntax.md](docs/spec/syntax.md) — The `standard`, `entry type`, `@standard[...]`, and `@[...]` constructs are now defined.
- [docs/spec/types.md](docs/spec/types.md) — Type annotations are now entries in the ledger. Nominal typing uses `@[nominal(Name)]`.
- [docs/spec/memory.md](docs/spec/memory.md) — Memory model is a ledger feature with states.
- [docs/spec/polymorphism.md](docs/spec/polymorphism.md) — Polymorphism strategy is a ledger feature with states.
- [docs/spec/execution.md](docs/spec/execution.md) — Pre-audit maps to declaration-time checking; live audit maps to execution-time.
- [docs/spec/modules.md](docs/spec/modules.md) — Cross-module standard boundaries.
- [docs/user-guide/encumbrance.md](docs/user-guide/encumbrance.md) — Would be rewritten as "Assurance Levels" user guide.
- [docs/ai-guide/orientation.md](docs/ai-guide/orientation.md) — Update terminology throughout.
- [docs/ai-guide/playbook-encumbered.md](docs/ai-guide/playbook-encumbered.md) — Rename and update for high-assurance.
- [docs/ai-guide/playbook-streamlined.md](docs/ai-guide/playbook-streamlined.md) — Update for low-assurance.
- [docs/ai-guide/playbook-mixed.md](docs/ai-guide/playbook-mixed.md) — Update for mixed-assurance.
- [AGENTS.md](AGENTS.md) — Update key concepts.
- [docs/project/open-questions.md](docs/project/open-questions.md) — Mark answered questions; add new ones from this proposal.

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-14-assurance-ledger-standards-and-syntax.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
- [docs/project/proposals/ledger-system-syntax.md](ledger-system-syntax.md)
- [docs/project/proposals/agreement-metaphor-and-syntax.md](agreement-metaphor-and-syntax.md)
