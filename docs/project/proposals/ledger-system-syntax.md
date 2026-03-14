---
title: "Proposal: Ledger System — Naming and Complete Syntax"
category: proposal
audience: [all]
status: accepted
last-verified: 2026-03-14
depends-on: [docs/project/proposals/agreement-metaphor-and-syntax.md, docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/execution.md, docs/spec/memory.md, docs/spec/polymorphism.md, docs/spec/reasoning.md, GLOSSARY.md]
---

# Proposal: Ledger System — Naming and Complete Syntax

> **Status: Accepted.** Superseded by [assurance-ledger-syntax.md](assurance-ledger-syntax.md). Decisions from this proposal were carried forward into the final design.

*Follow-on to [agreement-metaphor-and-syntax.md](agreement-metaphor-and-syntax.md). Generated from `agreement` on 2026-03-14.*

**Decisions carried forward from the prior proposal:**
- Metaphor: **Ledger** (accounting/bookkeeping)
- Rename "encumbrance level": **Yes** (to something more positive)
- Annotation sigil: **Zig-like** `@[...]`
- Block-level and item-level annotations: **Visually differentiated**
- Native syntax features: types (`:`), mutability (`mut`), async (`async`), error declarations (`throws`)
- Syntax approach: **Pragma + Annotation (Approach E)** with native syntax for common features
- The human wants: **"<adjective> Ledger"** to distance from blockchain and improve searchability

---

## Feature: System Name — "<Adjective> Ledger"

### Issues to Watch Out For

1. The adjective must strengthen the metaphor, not dilute it. It should evoke the *checking* or *consistency* aspect, not just generic bookkeeping.
2. The full phrase will appear in docs, error messages, and searches. It must be comfortable at all lengths: the full phrase ("X Ledger"), the shortened form ("the ledger"), and in compound terms ("ledger entry", "ledger audit").
3. The adjective must be searchable — unique enough to return relevant results when combined with "ledger" in a search engine.

### Critical Analysis

#### Alternative A: **Proof Ledger**

Combines the runner-up metaphor (Proof) with the winner (Ledger). Evokes both record-keeping and verification.

**Pros:**
- "Proof" immediately signals that this is about verification/checking, not mere bookkeeping.
- "Proof ledger" is a real accounting term (a ledger that serves as a record of correctness), giving the phrase historical grounding.
- Extremely searchable. "proof ledger ish" will return nothing but ish-related results.
- Borrows the best term from the Proof metaphor without the downsides (doesn't promise formal verification when used as an adjective).

**Cons:**
- Could still evoke formal methods associations in some CS readers.
- "Proof" slightly clashes if the reasoning spec keeps using "proposition" — though the two systems are different enough that this may be fine.

#### Alternative B: **Tally Ledger**

Evokes the physical act of counting and recording marks.

**Pros:**
- "Tally" has a satisfying simplicity — it means to count up, to keep score. Every statement "tallies" its entries.
- Distinctly non-technical. No CS term collisions.
- "Tally" can be used as a verb: "the processor tallies each statement's entries."

**Cons:**
- "Tally" evokes counting/summing, not consistency checking. A tally doesn't inherently check for contradictions.
- "Tally ledger" is somewhat redundant — a tally *is* a kind of ledger.

#### Alternative C: **Binding Ledger**

Evokes the sense that entries are *bound* — once recorded, they constrain what follows.

**Pros:**
- "Binding" has a legal connotation (binding agreement, binding contract) that reinforces seriousness.
- In programming, "binding" means associating a name with a value — which is exactly what ledger entries do.
- "Binding" correctly implies that entries have *force* — they restrict future statements.

**Cons:**
- "Binding" already has a heavily-used meaning in programming (variable binding, name binding, data binding). Not distinctive.
- "Binding ledger" could be misread as "a ledger of variable bindings."

#### Alternative D: **Assurance Ledger**

Evokes the idea that the ledger provides *assurance* — confidence that the program is consistent.

**Pros:**
- "Assurance" is the correct word for what the system provides — it's the positive outcome of auditing.
- "Quality assurance" is a well-known concept, and this is analogous: the ledger is the QA system.
- Professional-sounding. Doesn't evoke any competing CS concept.
- "Assurance level" is an excellent replacement for "encumbrance level" / "rigor level" — it's positive (more assurance = more checking) without being judgy.

**Cons:**
- "Assurance ledger" is a long phrase (5 syllables). Slightly heavy in frequent use.
- "Assurance" doesn't directly evoke *how* the checking works — it describes the outcome rather than the mechanism.

#### Alternative E: **Ruling Ledger**

Evokes a judge's ruling — each entry is a ruling about a property of the code.

**Pros:**
- "Ruling" correctly implies authority and finality — once an entry is recorded, it rules.
- "Ruling" can be used as a noun (an individual decision) and maps well to "entry": each entry is a ruling.
- Extends the audit metaphor: auditors discover discrepancies, rulings resolve them.

**Cons:**
- "Ruling" sounds authoritarian. Could feel heavy-handed for a feature that's meant to be configurable and progressive.
- Less searchable — "ruling ledger" could return results about court rulings.

#### Alternative F: **Source Ledger**

The ledger records facts about source code. Simple, literal, unambiguous.

**Pros:**
- "Source" is immediately understood — it's about source code.
- "Source ledger" is unique and highly searchable.
- Semantically accurate: it is a ledger of entries derived from the source.

**Cons:**
- Bland. Doesn't evoke checking, verification, or consistency. It's just descriptive.
- "Source" is extremely overloaded (source code, event sourcing, source control, data source).

### Recommendation: **Proof Ledger** (Alternative A)

**Recommended name: Proof Ledger.**

The phrase combines the best of both finalist metaphors. "Proof" signals that this is a verification system, while "ledger" signals that it operates by maintaining a structured record. "Proof ledger" is also a real term in accounting — a control ledger used to verify that accounts balance correctly — which grounds the metaphor in both the accounting and verification domains simultaneously.

When shortened, "the ledger" is natural and sufficient. In compound terms: "ledger entry", "ledger audit", "proof ledger profile" all read well.

**Derived terminology update:**

| Role | Term |
|------|------|
| System name | **Proof Ledger** (shortened: **the ledger**) |
| Unit of knowledge | **Entry** |
| Inconsistency | **Discrepancy** |
| Checking action | **Audit** |
| Scope decorator (block-level) | **Ruling** (a block-level annotation that rules what is tracked) |
| Item decorator | **Entry annotation** (an item-level annotation that records an entry) |
| Declaration-time checking | **Pre-audit** |
| Execution-time checking | **Live audit** |
| Configurable presets | **Ledger profile** |
| User-defined rules | **Custom entry type** |
| Encumbrance continuum | **Assurance level** (streamlined = low assurance, encumbered = high assurance) |

**Note:** "Ruling" is introduced here as the term for block-level annotations (see the terminology distinction section below). This borrows from Alternative E specifically for the scope decorator role, because block-level annotations "rule" what the ledger tracks within their scope.

### Decisions

**Decision:** What should the full name be? Alternatives: Proof Ledger (recommended), Tally Ledger, Binding Ledger, Assurance Ledger, Ruling Ledger, Source Ledger.
--> Assurance Ledger.  It inspires confidence while also being searchable.  Dovetails nicely with assurance level.

**Decision:** Should "encumbrance" be renamed to "assurance level"? (The prior proposal approved renaming it; this proposes the specific replacement.)
--> Yes.

---

## Feature: Terminology for Block-Scoped vs. Item-Scoped Annotations

### Issues to Watch Out For

1. Block-scoped and item-scoped annotations have fundamentally different semantics despite sharing the same underlying system. Block-scoped annotations *configure* what the ledger tracks; item-scoped annotations *record entries* for specific items.
2. The terminology must make this distinction intuitive without requiring deep knowledge of the system.
3. The terms must work naturally in both documentation and error messages.

### Critical Analysis

#### Alternative A: **Ruling** (block) / **Entry** (item)

A **ruling** declares what the ledger is obligated to track within a scope. An **entry** records a specific fact about a specific item.

```ish
@[ruling: immutable_by_default]       // ruling: configures the ledger for this block
{
    @[overflow: wrapping] let z: u8 = 255;  // entry: records overflow behavior for z
}
```

**Pros:**
- "Ruling" clearly implies authority and scope — it governs everything beneath it.
- "Entry" is already the established term for the unit of knowledge.
- The distinction is sharp: rulings configure, entries record.
- Error messages read well: *"Ruling @[ruling: immutable_by_default] at line 1 caused entry @immutable(x) at line 2, which conflicts with mutation at line 3."*

**Cons:**
- "Ruling" is a new term that must be learned. It's intuitive but not immediately obvious.
- Some annotations could arguably be either (e.g., a block-level default that also applies to a specific variable).

#### Alternative B: **Scope annotation** (block) / **Item annotation** (item)

Purely descriptive. Named after where they attach.

**Pros:**
- Maximum clarity. No metaphor to learn.
- "Scope annotation" and "item annotation" need no explanation.

**Cons:**
- Boring. Doesn't extend the ledger metaphor.
- "Scope annotation" is generic — it could apply to any annotation system.
- Long for frequent use.

#### Alternative C: **Policy** (block) / **Entry** (item)

A **policy** declares the rules for a scope. An **entry** records a specific fact.

**Pros:**
- "Policy" is well-understood in enterprise software (security policies, access policies).
- The distinction between policy and entry is immediately clear.
- "Policy" implies that it can be overridden or inherited, which matches the scoping behavior.

**Cons:**
- "Policy" has enterprise/bureaucratic connotations that may clash with the language's accessibility goals.
- Mixes metaphors (accounting + governance).

### Recommendation: **Ruling** (block) / **Entry** (item) (Alternative A)

Rulings govern scopes. Entries record facts about items. This extends the accounting metaphor naturally: in auditing, a *ruling* is a standard or regulation that determines what must be recorded, and an *entry* is an individual record.

### Decisions

**Decision:** What terms should distinguish block-scoped from item-scoped annotations? Alternatives: Ruling/Entry (recommended), Scope/Item annotation, Policy/Entry.
--> Standard/Entry.  The arguments for Ruling/Entry are strong, but the correct accounting term for the concept is "Accounting standard", not "Ruling".  Also dovetails nicely with the existing concept of coding standards.

---

## Feature: Complete Annotation Syntax

This section addresses all the follow-on syntax questions from the prior proposal's decisions.

### Annotation Sigil

Per the prior decision, the sigil is Zig-like: `@[...]`.

- **Rulings** (block-scoped): `@[ruling: ...]`
- **Entries** (item-scoped): `@[...]`

The `ruling:` prefix visually differentiates the two forms. This is explicit, self-documenting, and machine-parseable.

### Scope Override: How Rulings Work

Rulings at inner lexical scopes override rulings at outer scopes for the same feature. Here is how both directions work — increasing and decreasing assurance.

#### Increasing assurance (low → high)

```ish
// Project default: streamlined (low assurance)
// No rulings needed — everything is inferred and dynamic

fn process(data) {
    // Developer decides this function needs type safety
    @[ruling: types]
    @[ruling: null_safety]
    {
        let x: i32 = data.count;        // type required by ruling
        let name: String = data.name;    // type required by ruling
        let label: String? = data.label; // nullability explicit (?), required by ruling

        // Nested scope: add immutability on top of existing rulings
        @[ruling: immutability]
        {
            let y: i32 = x + 1;          // immutable by default (ruling)
            let mut z: i32 = 0;           // must say 'mut' explicitly
            z = y;                        // ok — z is mutable
            // y = 5;                     // DISCREPANCY: y is immutable
        }
    }
}
```

#### Decreasing assurance (high → low)

```ish
// Project default: high assurance
@[ruling: types, null_safety, immutability, overflow(panicking)]

fn quick_prototype() {
    // Relax rules for rapid iteration
    @[ruling: !types]
    @[ruling: !null_safety]
    {
        let x = 42;              // no type required (ruling overridden)
        let y = null;             // no null tracking
        x = "changed my mind";   // allowed — types aren't tracked here
    }
}
```

#### Override syntax: the `!` negation prefix

`@[ruling: !feature]` disables a ruling inherited from an outer scope. This mirrors boolean negation and reads naturally: "no types", "no null safety".

Multiple features can be grouped in one ruling:

```ish
@[ruling: types, null_safety, immutability]     // enable three features
@[ruling: !types, !null_safety]                  // disable two features
```

### Item-Level Entries Are Cumulative

Annotations on items (variables, functions, properties) do not override each other. They accumulate. If the accumulated entries contain a discrepancy, the processor throws an error.

```ish
@[ruling: types]
{
    let x: i32 = 7;                // entry: @type(x, i32)
    @[overflow: wrapping] let y: u8 = 255;  // entries: @type(y, u8), @overflow(y, wrapping)

    // Discrepancy — conflicting entries on the same item:
    // @[overflow: wrapping] @[overflow: panicking] let z: u8 = 0;
    //   ^^ ERROR: Discrepancy: conflicting overflow entries for z
}
```

### Object Type Definitions with Ledger Entries

Object types can include entries for properties beyond just structural types. This is how nominal-like behavior, mutability, and custom annotations are attached to object properties.

```ish
type DatabaseRecord = {
    id: i64,                           // structural type only
    mut name: String,                  // mutable property (native syntax)
    email: String,                     // immutable by default
    @[null_safety: strict] age: i32?,  // explicit null safety on this property
    @[overflow: saturating] retries: u8,  // overflow behavior for this property
};

// An object type with custom entries
type ApiResponse = {
    status: i32,
    body: String,
    @[validated]                       // custom entry: this field has been validated
    @[sanitized]                       // custom entry: this field has been sanitized
    user_input: String,
};

// Nominal/branded type using a custom entry
type UserId = {
    @[nominal: UserId]                 // this entry makes UserId nominally distinct
    value: i64,
};

type ProductId = {
    @[nominal: ProductId]
    value: i64,
};
// UserId and ProductId are structurally identical but nominally distinct
```

### Custom Entry Type Definitions

Developers can define new entry types with associated rules. A custom entry type specifies:
1. What items it can be applied to (variable, function, block, property, etc.)
2. What other entries it implies
3. What rules detect discrepancies

```ish
// Define a custom entry type @[validated]
entry type validated {
    applies_to: [variable, property],

    // No implied entries — this is a pure marker
}

// Define a custom entry type @[sanitized] with rules
entry type sanitized {
    applies_to: [variable, property],

    rules: {
        // A sanitized value must have a known type
        requires: @[type: _],
    },
}

// Define a custom entry type @[thread_safe] with compound rules
entry type thread_safe {
    applies_to: [variable, property],

    implies: [
        @[immutable],          // thread safe implies immutable
    ],

    rules: {
        // Cannot be applied to open objects
        conflicts: @[object_type: open],
    },
}

// Define a custom entry type usable as a ruling
entry type strict_mode {
    applies_to: [block, function, module],

    implies_rulings: [
        @[ruling: types],
        @[ruling: null_safety],
        @[ruling: immutability],
        @[ruling: overflow(panicking)],
    ],
}

// Usage:
@[ruling: strict_mode] {
    let x: i32 = 7;      // all strict_mode rulings are active
}
```

### Preset (Ledger Profile) Definitions

Presets are named combinations of rulings that configure the ledger in one annotation. They are the primary mechanism for the assurance continuum.

```ish
// Built-in profiles (defined by the language)
ledger profile streamlined {
    // Minimal tracking — dynamic checking only
    rulings: []                          // no rulings active
    audit_mode: live                     // check at execution time
}

ledger profile cautious {
    // Moderate tracking
    rulings: [types, null_safety, immutability]
    audit_mode: live
}

ledger profile rigorous {
    // Full tracking — static checking
    rulings: [
        types,
        null_safety,
        immutability,
        overflow(panicking),
        numeric_precision,
        exhaustiveness,
        unused_variables,
        unreachable_statements,
        memory_model,
        polymorphism_strategy,
        checked_exceptions,
        visibility,
        open_closed_objects,
    ]
    audit_mode: pre                       // check at declaration time
}

// User-defined profile
ledger profile my_team_standard {
    extends: cautious                     // inherit from built-in
    rulings: [
        +overflow(saturating),            // add overflow tracking
        +checked_exceptions,              // add exception tracking
        -immutability,                    // remove immutability tracking
    ]
    audit_mode: pre
}

// Usage at module level:
@[profile: rigorous]

// Usage at block level:
@[profile: streamlined] {
    let x = 42;          // no checking — rapid prototyping
}
```

### Complete Feature Annotation Table

The following table lists every language feature that participates in the ledger, its ruling form (block-scoped), its entry form (item-scoped), and the native syntax (if any).

| Feature | Ruling form | Entry form | Native syntax | Applies to |
|---------|-------------|------------|---------------|------------|
| Variable type | `@[ruling: types]` | `@[type: T]` | `: T` after name | variable, parameter, property |
| Function return type | `@[ruling: types]` | `@[return_type: T]` | `-> T` or `: T` after params | function |
| Mutability | `@[ruling: immutability]` | `@[mutable]` / `@[immutable]` | `mut` keyword | variable, property |
| Immutable by default | — | — | — | (implied by `@[ruling: immutability]`) |
| Async mode | `@[ruling: async_mode]` | `@[async: wait]` / `@[async: defer]` | `async` keyword | function, block |
| Thrown errors | `@[ruling: checked_exceptions]` | `@[throws: ErrorType]` | `throws ErrorType` | function |
| Error mode preset | `@[ruling: error_mode(preset)]` | — | — | block, function, module |
| Numeric precision | `@[ruling: numeric_precision]` | `@[numeric: exact]` | (explicit type like `i32`) | variable, property |
| Numeric overflow | `@[ruling: overflow(behavior)]` | `@[overflow: wrapping]` / `panicking` / `saturating` | — | variable, property, block |
| Implicit conversions | `@[ruling: !implicit_conversions]` | — | — | block, module |
| Null safety | `@[ruling: null_safety]` | `@[nullable]` / `@[non_null]` | `?` suffix on type | variable, property |
| Memory model | `@[ruling: memory_model]` | `@[memory: stack]` / `heap` / `rc` / `gc` | — | variable |
| Polymorphism strategy | `@[ruling: polymorphism_strategy]` | `@[polymorphism: none]` / `enum` / `mono` / `vtable` / `assoc` | — | type, function |
| Open/closed objects | `@[ruling: open_closed_objects]` | `@[object_type: open]` / `@[object_type: closed]` | — | type, variable |
| Visibility | `@[ruling: visibility]` | `@[visibility: pub(...)]` | `pub(...)` keyword | variable, function, type, module |
| Unused variables | `@[ruling: unused_variables]` | `@[allow_unused]` | `_` prefix on name | variable |
| Unreachable statements | `@[ruling: unreachable_statements]` | `@[allow_unreachable]` | — | statement |
| Sync/Async enforcement | `@[ruling: sync_async]` | `@[sync]` / `@[async]` | `async` keyword | block, function |
| Blocking wait | `@[ruling: blocking]` | `@[blocking: allow]` / `@[blocking: deny]` | — | block, function |
| State mutation | `@[ruling: pure_functions]` | `@[pure]` / `@[mutates_state]` | — | function |
| Exhaustiveness | `@[ruling: exhaustiveness]` | — | — | block (match/switch) |
| Nominal typing | — | `@[nominal: Name]` | `nominal type` keyword | type |

### Syntax for Setting Audit Mode Per Feature

Some features should be pre-audited (declaration-time) while others are live-audited (execution-time). This is configurable per feature.

```ish
// At module level: set default audit modes
@[ruling: types(pre)]                    // types checked at declaration time
@[ruling: null_safety(pre)]              // null safety checked at declaration time
@[ruling: overflow(panicking, live)]     // overflow checked at execution time
@[ruling: unused_variables(pre)]         // unused vars checked at declaration time

// A profile can set audit modes for all features
ledger profile production {
    rulings: [
        types(pre),
        null_safety(pre),
        immutability(pre),
        overflow(panicking, pre),
        checked_exceptions(pre),
        unused_variables(pre),
        unreachable_statements(pre),
    ]
    audit_mode: pre   // default for any ruling not explicitly specified
}
```

### Full Syntax Example: A Complete Module

Putting it all together — a complete module showing the syntax at every level.

```ish
// Module-level profile
@[profile: cautious]

// Module-level ruling overrides
@[ruling: overflow(saturating)]
@[ruling: checked_exceptions]

// Type definitions with entries
type Coordinate = {
    x: f64,
    y: f64,
};

type BoundedCoordinate = {
    @[overflow: saturating] x: u16,
    @[overflow: saturating] y: u16,
};

nominal type UserId = i64;
nominal type SessionId = String;

// Custom entry type for domain validation
entry type authenticated {
    applies_to: [variable, property],
    requires: @[type: _],
}

entry type rate_limited {
    applies_to: [function],
}

// Function with mixed native and entry syntax
@[rate_limited]
fn get_user(id: UserId) -> ApiResponse throws NotFoundError {
    let raw = db.query("SELECT * FROM users WHERE id = ?", id);
    let user = validate(User, raw);

    @[authenticated] let session = get_session(user);

    return { status: 200, body: serialize(user) };
}

// Streamlined helper — relax assurance in local scope
fn quick_format(data) {
    @[ruling: !types]
    {
        let result = "";
        for item in data {
            result = result + " " + item;   // no type checking here
        }
        return result;
    }
}

// High-assurance block inside a moderate-assurance module
fn critical_calculation(values: List<f64>) -> f64 {
    @[profile: rigorous]
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
```

### Syntax Alternatives and Decision Points

#### Decision Point 1: Ruling syntax — prefix vs. named parameter

**Alternative 1a (recommended):** `@[ruling: feature]`
```ish
@[ruling: types]
@[ruling: !null_safety]
```

**Alternative 1b:** `@[track: feature]` (uses "track" instead of "ruling")
```ish
@[track: types]
@[track: !null_safety]
```

**Alternative 1c:** Separate sigil for rulings — `@{...}` for rulings, `@[...]` for entries
```ish
@{types, null_safety}         // ruling
@[overflow: wrapping] let z;  // entry
```

| | 1a: `@[ruling:]` | 1b: `@[track:]` | 1c: `@{...}` |
|-|:-:|:-:|:-:|
| Self-documenting | ★★★ | ★★★ | ★★☆ |
| Conciseness | ★★☆ | ★★☆ | ★★★ |
| Extends metaphor | ★★★ | ★★☆ | ★☆☆ |
| Visual distinction from entries | ★★★ | ★★★ | ★★★ |
| Parseable without context | ★★★ | ★★★ | ★★☆ |

#### Decision Point 2: Negation syntax for ruling overrides

**Alternative 2a (recommended):** `!` prefix — `@[ruling: !types]`
**Alternative 2b:** `no_` prefix — `@[ruling: no_types]`
**Alternative 2c:** Separate keyword — `@[unruling: types]`

| | 2a: `!` | 2b: `no_` | 2c: `@[unruling:]` |
|-|:-:|:-:|:-:|
| Familiar to programmers | ★★★ | ★★☆ | ★★☆ |
| Concise | ★★★ | ★★☆ | ★☆☆ |
| Readable | ★★☆ | ★★★ | ★★★ |
| Learnable | ★★★ | ★★★ | ★★☆ |

#### Decision Point 3: Profile application syntax

**Alternative 3a (recommended):** `@[profile: name]`
**Alternative 3b:** Dedicated keyword — `use profile name;`
**Alternative 3c:** Special ruling — `@[ruling: profile(name)]`

| | 3a: `@[profile:]` | 3b: `use profile` | 3c: `@[ruling: profile()]` |
|-|:-:|:-:|:-:|
| Consistent with annotation system | ★★★ | ★☆☆ | ★★★ |
| Familiar | ★★☆ | ★★★ | ★☆☆ |
| Concise | ★★☆ | ★★★ | ★☆☆ |
| Self-documenting | ★★★ | ★★★ | ★★☆ |

#### Decision Point 4: Audit mode specification

**Alternative 4a (recommended):** Parenthetical — `@[ruling: types(pre)]`
**Alternative 4b:** Separate annotation — `@[ruling: types] @[audit: pre]`
**Alternative 4c:** Profile-only — audit mode can only be set in ledger profile definitions, not inline

| | 4a: parenthetical | 4b: separate | 4c: profile-only |
|-|:-:|:-:|:-:|
| Concise | ★★★ | ★☆☆ | ★★★ |
| Flexible | ★★★ | ★★★ | ★☆☆ |
| Clear | ★★☆ | ★★★ | ★★★ |
| No ambiguity | ★★☆ | ★★★ | ★★★ |

#### Decision Point 5: Custom entry type definition syntax

**Alternative 5a (recommended):** `entry type` block (see examples above)
**Alternative 5b:** Function-based — define entry types as functions returning rules
```ish
fn entry_validated(item) -> EntryRules {
    return { applies_to: [variable, property] };
}
```

**Alternative 5c:** Declarative annotation on a type alias
```ish
@[entry_type(applies_to: [variable, property])]
type validated;
```

| | 5a: `entry type` | 5b: function-based | 5c: type alias |
|-|:-:|:-:|:-:|
| Readable | ★★★ | ★★☆ | ★★☆ |
| Extensible | ★★★ | ★★★ | ★★☆ |
| Consistent with rest of language | ★★☆ | ★★★ | ★★☆ |
| Self-documenting | ★★★ | ★★☆ | ★★☆ |

#### Decision Point 6: Ledger profile definition syntax

**Alternative 6a (recommended):** `ledger profile` block (see examples above)
**Alternative 6b:** Object literal — profiles are just objects
```ish
let my_profile: LedgerProfile = {
    rulings: [types, null_safety],
    audit_mode: pre,
};
```

**Alternative 6c:** Annotation on a name
```ish
@[profile_definition(rulings: [types, null_safety], audit: pre)]
profile my_team_standard;
```

| | 6a: `ledger profile` | 6b: object literal | 6c: annotation |
|-|:-:|:-:|:-:|
| Readable | ★★★ | ★★☆ | ★☆☆ |
| Composable | ★★★ | ★★★ | ★★☆ |
| Feels like a definition | ★★★ | ★★☆ | ★★☆ |
| Consistent with `entry type` | ★★★ | ★☆☆ | ★☆☆ |

### Decisions

**Decision:** Ruling syntax — `@[ruling: ...]` (1a), `@[track: ...]` (1b), or `@{...}` (1c)?
--> `@standard[...]`

**Decision:** Negation syntax — `!` prefix (2a), `no_` prefix (2b), or `@[unruling:]` (2c)?
--> `@[immutability(optional)]

**Decision:** Profile application — `@[profile: name]` (3a), `use profile name` (3b), or `@[ruling: profile()]` (3c)?
--> There's no such thing as profiles.  Developers can define a standard, which is the same as any other standard, not a special profile.
--> A standard declaration looks like:
standard my_team_standard extends cautious [
    overflow(saturating),             // add overflow tracking
    undeclared_errors(any),           // Allow any kind of error to be returned without specifying it in the function signature
    immutability(pre),                // add immutability tracking during pre audit
]

**Decision:** Audit mode specification — parenthetical (4a), separate annotation (4b), or profile-only (4c)?
--> 4a parenthetical

**Decision:** Custom entry type definition — `entry type` block (5a), function-based (5b), or type alias annotation (5c)?
--> 5a block, except entry types can't be useed as standards.  See decision 3 for standard definition.

**Decision:** Ledger profile definition — `ledger profile` block (6a), object literal (6b), or annotation (6c)?
--> There's no such thing as a ledger profile.  See decision 3.

--> In general, the syntax `@[a: b]` should be replaced with `@[a(b)]`

--> Not every standard is limited to on/off.  For example, some support live (check for explicit value during live audit), pre (check for explicit value during pre audit), and optional (Do not require explicit value).  As such, possible states should be enumerated with a parameter rather than an on/off marker.  

---

## Documentation Updates

All files from the prior proposal's documentation update list remain affected. Additional files:

- [docs/spec/syntax.md](docs/spec/syntax.md) — Currently a placeholder. This proposal provides substantial syntax that should be integrated once the syntax spec is written.
- [docs/spec/memory.md](docs/spec/memory.md) — Memory model annotations (`@[memory: stack]`, etc.) are now specified.
- [docs/spec/polymorphism.md](docs/spec/polymorphism.md) — Polymorphism strategy annotations are now specified.
- [docs/user-guide/functions.md](docs/user-guide/functions.md) — Function-level annotations (`@[pure]`, `@[rate_limited]`, etc.).
- [docs/user-guide/objects.md](docs/user-guide/objects.md) — Object type definitions with entries.
- [docs/project/proposals/agreement-metaphor-and-syntax.md](agreement-metaphor-and-syntax.md) — Predecessor proposal; update status to indicate this follow-on exists.

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-14-ledger-system-naming-and-syntax.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
- [docs/project/proposals/agreement-metaphor-and-syntax.md](agreement-metaphor-and-syntax.md)
