---
title: "Proposal: Agreement Metaphor and Syntax"
category: proposal
audience: [all]
status: accepted
last-verified: 2026-03-14
depends-on: [docs/spec/assurance-ledger.md, docs/spec/types.md, docs/spec/execution.md, docs/spec/reasoning.md, GLOSSARY.md]
---

# Proposal: Agreement Metaphor and Syntax

> **Status: Accepted.** Superseded by [assurance-ledger-syntax.md](assurance-ledger-syntax.md). Decisions from this proposal were carried forward into the final design.

*Generated from [agreement-checking-terminology-and-syntax.md](../rfp/agreement-checking-terminology-and-syntax.md) on 2026-03-14.*

---

## Questions and Answers

### Q: What terminology do other languages use for consistency-checking features similar to ish's agreement system?

The ish agreement system is unusual in that it **unifies** what most languages treat as separate features (type checking, mutability enforcement, null safety, overflow behavior, etc.) under a single consistency-checking mechanism. No mainstream language has a single unified term for this composite concept, but many languages have related terminology for subsets:

**Type checking / Type systems:**
- Nearly all languages call this "type checking" or "type inference". The terminology is universal: "types", "type annotations", "type errors".

**Design by Contract (DbC):**
- **Eiffel** (originator): `require` (preconditions), `ensure` (postconditions), `invariant` (class invariants). Collected under the term **"contracts"**.
- **D**: `in` / `out` / `invariant` blocks, also called **"contracts"**.
- **Ada/SPARK**: `Pre` / `Post` aspects, called **"contracts"** or **"aspects"**.
- **Kotlin**: `kotlin.contracts` — compiler-visible **"contracts"** that declare behavioral guarantees.
- **Racket**: A rich **"contract system"** that wraps values with runtime-checked behavioral obligations.
- **Clojure**: `clojure.spec` — **"specs"** that declare structural and behavioral properties, checked at boundaries.

**Attributes / Annotations:**
- **Java**: `@Override`, `@NonNull`, `@Nullable` — **"annotations"** attached to declarations that may or may not be checked.
- **C#**: `[NotNull]`, `[Pure]` — **"attributes"**. Microsoft also shipped **"Code Contracts"** (now deprecated) using `Contract.Requires()` / `Contract.Ensures()`.
- **Python**: **"type hints"** (PEP 484), checked externally by tools like mypy.

**Trait / Protocol / Interface conformance:**
- **Rust**: **"trait bounds"** — a type must implement a trait to satisfy a generic constraint. The checking is called "the borrow checker" (for lifetimes/borrowing) and "type checking" (for traits).
- **Swift**: **"protocol conformance"** — types declare they conform to a protocol and the compiler verifies the required members exist.
- **Haskell**: **"type class constraints"** — a polymorphic function declares what type classes its parameters must be instances of.

**Assertions / Verification:**
- **SPARK/Ada**: Formal **"proof obligations"** — the compiler generates mathematical proof obligations and an external prover checks them.
- **Dafny**: **"verification conditions"** — preconditions, postconditions, loop invariants, all expressed in the language and checked by an SMT solver.

**Key observation:** The closest analogue to ish's unified agreement system is **Design by Contract** (Eiffel/D/Racket), because DbC also treats consistency checking as a first-class, configurable, cross-cutting concern rather than a fixed feature of the type system. However, DbC systems typically operate at function boundaries (pre/postconditions), whereas ish's agreement system operates at every statement.

### Q: Is there a better fit metaphor than "agreement" (natural linguistics) or "contract" (legal)?

Yes. The prompt correctly identifies problems with both:

- **Agreement** (linguistics): The metaphor is about grammar/syntax, but ish's feature is about semantics. "Agreement" is an extremely common English word, making it non-distinctive.
- **Contract** (legal): "Terms" is ambiguous (word vs. clause). "Contract" is already heavily used in programming (Eiffel, D, Kotlin, Racket, C#). It wouldn't feel distinctive to ish.

The ideal metaphor should: (1) relate to consistency checking, (2) use distinctive terminology, and (3) avoid ambiguous terms. Several alternatives are analyzed below in the Feature section.

---

## Feature: Metaphor and Terminology for the Consistency-Checking System

### Issues to Watch Out For

1. **Terminology lock-in.** Once a metaphor is chosen and propagated through docs, specs, error messages, compiler output, and user muscle memory, changing it later is extremely costly. This decision should be made carefully and early.

2. **Mixing metaphors.** The current system already mixes "agreement" (linguistics) with "facts" (propositional logic) and "marked features" (linguistics). Any new metaphor must provide replacements for *all* of these terms to avoid continued mixing.

3. **Collision with existing programming terminology.** Many appealing terms ("contract", "constraint", "assertion", "protocol", "invariant") are already heavily used in programming. Using them for ish's unique unified system creates confusion about whether ish means the same thing as other languages.

4. **Ergonomics in error messages.** The chosen terminology will appear in every error message from the consistency checker. Terms must read naturally in sentences like "X violates Y" or "X and Y are inconsistent".

5. **The term for "facts" is particularly important.** This is the most-used concept: every statement entails them, every check examines them. The replacement must be short, clear, and unambiguous.

6. **Community perception.** An overly academic or unusual metaphor may make the language feel inaccessible. An overly common one makes it feel generic.

### Critical Analysis

Below are eight candidate metaphors evaluated against the three stated criteria and practical considerations. For each, the full terminology mapping is:

- **System name** — what you call the whole feature
- **Unit of knowledge** — replaces "fact" (what each statement entails)
- **Inconsistency** — what you call a failure
- **Checking action** — what the processor does
- **Scope decorator** — replaces "marked feature" (what configures checking)
- **Configuration** — replaces "encumbrance level" (or works alongside it)

---

#### Alternative 1: Agreement (status quo — linguistics metaphor)

| Role | Term |
|------|------|
| System | Agreement |
| Unit of knowledge | Fact |
| Inconsistency | Disagreement / Agreement error |
| Checking action | Check for agreement |
| Scope decorator | Marked feature |
| Configuration | Encumbrance level |

**Pros:**
- Already in use across all docs and the glossary. No migration cost.
- "Marked feature" is a genuinely good term borrowed well from linguistics.
- The linguistics analogy (subject-verb agreement) is intellectually interesting.

**Cons:**
- "Agreement" is one of the most overloaded words in English (legal agreement, social agreement, nodding in agreement). Not distinctive.
- The linguistics analogy is about surface syntax/grammar, but the feature is about deep semantics. This mismatch may mislead.
- "Facts" comes from propositional logic, creating a mixed metaphor (linguistics + logic).
- "Disagreement" as an error term is soft — it sounds like a polite difference of opinion, not a program defect.

---

#### Alternative 2: Contract (legal metaphor)

| Role | Term |
|------|------|
| System | Contract |
| Unit of knowledge | Term / Clause |
| Inconsistency | Breach / Violation |
| Checking action | Enforce |
| Scope decorator | Stipulation |
| Configuration | Strictness level |

**Pros:**
- Developers already know "Design by Contract" from Eiffel. Immediate conceptual familiarity.
- "Breach" and "violation" are strong error terms that convey seriousness.
- "Enforce" is a clear verb for what the processor does.

**Cons:**
- "Term" is deeply ambiguous (word *or* contractual clause). This is a serious problem.
- "Clause" is better but still somewhat ambiguous (SQL clause, grammar clause).
- "Contract" is already used by Eiffel, D, Kotlin, Racket, and C#. Not distinctive to ish.
- The metaphor implies two parties making a deal. In ish, it's one program checked against itself — the "parties" analogy doesn't hold cleanly.

---

#### Alternative 3: Constraint (mathematical/CSP metaphor)

| Role | Term |
|------|------|
| System | Constraint system |
| Unit of knowledge | Constraint / Property |
| Inconsistency | Conflict / Violation |
| Checking action | Solve / Verify |
| Scope decorator | Required constraint |
| Configuration | Constraint level |

**Pros:**
- CS-familiar. Constraint satisfaction problems (CSPs) are part of every CS curriculum.
- "Conflict" is a clear and unambiguous error term.
- Maps well to the actual implementation: the processor is effectively solving a constraint satisfaction problem.

**Cons:**
- "Constraint" is ubiquitous in programming (database constraints, layout constraints, generic constraints). Not distinctive.
- "Constraint system" sounds like an internal implementation detail, not a user-facing language concept.
- "Solve" implies an optimization search, which is misleading — ish's checking is deterministic, not search-based.
- No natural term for "marked feature". "Required constraint" is clunky.

---

#### Alternative 4: Ledger (accounting/bookkeeping metaphor)

| Role | Term |
|------|------|
| System | Ledger |
| Unit of knowledge | Entry |
| Inconsistency | Discrepancy |
| Checking action | Audit |
| Scope decorator | Tracked feature |
| Configuration | Scrutiny level |

**Pros:**
- **Highly distinctive.** No mainstream language uses accounting terminology. When a developer sees "ledger", "entry", "audit", or "discrepancy" in ish docs, they will instantly recognize it as pertaining to this system.
- The metaphor maps well: each statement adds "entries" to a "ledger"; the processor "audits" for "discrepancies"; the level of auditing is configurable ("scrutiny level").
- "Discrepancy" is an excellent error term — it means exactly "things that should be consistent but aren't".
- "Audit" naturally encompasses both real-time checking (auditing each transaction as it's recorded) and batch checking (auditing a complete set of books), mapping to execution-time vs. declaration-time checking.
- "Entry" is short, clear, and unambiguous in this context.
- "Tracked feature" is a natural replacement for "marked feature" — a feature that the ledger tracks.

**Cons:**
- Accounting is not traditionally associated with programming. Some developers may find it unexpected.
- "Ledger" has recent associations with blockchain/cryptocurrency, which could be distracting.
- "Entry" has some ambiguity (entry point, dictionary entry), though in context it should be clear.
- The metaphor doesn't have a natural analogue for user-defined annotations creating new entry types. You'd need to extend it (e.g., "custom ledger columns").

---

#### Alternative 5: Proof (formal methods metaphor)

| Role | Term |
|------|------|
| System | Proof system |
| Unit of knowledge | Proposition / Assertion |
| Inconsistency | Contradiction |
| Checking action | Verify |
| Scope decorator | Required proof |
| Configuration | Rigor level |

**Pros:**
- "Contradiction" is the most precisely correct term for what happens when facts/entries are inconsistent.
- "Proposition" maps perfectly to what the reasoning spec already calls these.
- CS-aligned: formal methods, program verification, and proof assistants are well-known in PL theory.
- "Rigor level" is an excellent replacement for "encumbrance level" — it's more positive-sounding.

**Cons:**
- "Proof system" implies formal verification (Coq, Lean, Dafny), setting expectations that ish doesn't meet. Developers may expect mathematical proofs.
- "Proof" and "proposition" are already used in the reasoning spec for a related but different subsystem, creating internal naming collisions.
- Heavy/academic terminology may intimidate developers coming from the streamlined end of the spectrum.
- "Required proof" as a scope decorator is awkward.

---

#### Alternative 6: Accord (harmony/music metaphor)

| Role | Term |
|------|------|
| System | Accord |
| Unit of knowledge | Note |
| Inconsistency | Discord / Dissonance |
| Checking action | Harmonize |
| Scope decorator | Voiced feature |
| Configuration | Fidelity level |

**Pros:**
- "Discord" and "dissonance" are vivid, memorable error terms that evoke exactly the right feeling.
- "Accord" itself means agreement/harmony without the linguistic-grammar baggage.
- Musical terminology is distinctive — no mainstream language uses it.
- "Harmonize" is a pleasant verb for what the processor does.

**Cons:**
- "Note" is ambiguous (musical note, annotation note, comment note). This is a significant problem for the most frequently used term.
- "Voiced feature" is a stretch — it tries to extend the music metaphor but sounds forced.
- The music metaphor may feel whimsical or unserious for a critical language feature.
- "Fidelity level" doesn't obviously map to "how much checking is turned on".
- "Discord" is now strongly associated with the chat application.

---

#### Alternative 7: Assay (metallurgy/chemistry metaphor)

| Role | Term |
|------|------|
| System | Assay |
| Unit of knowledge | Trait (in the metallurgical sense: a measured property) |
| Inconsistency | Impurity / Defect |
| Checking action | Assay |
| Scope decorator | Assayed property |
| Configuration | Purity level |

**Pros:**
- **Extremely distinctive.** "Assay" is rarely used in programming.
- The metaphor maps well: an assay tests a material for specific properties, just as the system checks a program for specific properties.
- "Purity level" is a nice term for the encumbrance continuum (more checking = higher purity).
- "Defect" is a strong, familiar error term.

**Cons:**
- "Trait" collides with Rust traits. Fatal for a language that compiles to Rust.
- "Assay" is an uncommon English word. Many developers won't immediately know what it means.
- "Impurity" may feel judgmental (your code is impure!), which conflicts with the philosophy that streamlined code is perfectly valid.
- The metaphor implies testing a finished product, not checking as statements are processed.

---

#### Alternative 8: Canon (authoritative standard metaphor)

| Role | Term |
|------|------|
| System | Canon |
| Unit of knowledge | Dictum (pl: dicta) |
| Inconsistency | Violation / Heresy |
| Checking action | Adjudicate |
| Scope decorator | Canonical feature |
| Configuration | Strictness level |

**Pros:**
- "Canon" means an authoritative set of rules, which is exactly what the system maintains.
- "Dictum" is distinctive and unambiguous — a proclaimed rule or pronouncement.
- "Canonical" is already well-known in CS ("canonical form", "canonical URL").
- "Adjudicate" maps well: the processor is a judge deciding whether statements comply.

**Cons:**
- "Canon" has strong religious connotations (biblical canon, church canon law). "Heresy" amplifies this.
- "Dictum" may feel obtuse or pretentious. Its plural "dicta" is unfamiliar to most English speakers.
- "Adjudicate" is verbose for frequent use in error messages.
- The religious overtones could make the language feel dogmatic or unwelcoming.

---

### Comparative Summary

| Criterion | Agreement | Contract | Constraint | Ledger | Proof | Accord | Assay | Canon |
|-----------|:---------:|:--------:|:----------:|:------:|:-----:|:------:|:-----:|:-----:|
| Relates to consistency checking | ★★★ | ★★★ | ★★★ | ★★★ | ★★★ | ★★★ | ★★☆ | ★★★ |
| Distinctive terminology | ★☆☆ | ★☆☆ | ★☆☆ | ★★★ | ★★☆ | ★★★ | ★★★ | ★★☆ |
| Unambiguous terminology | ★★☆ | ★☆☆ | ★★☆ | ★★☆ | ★★☆ | ★☆☆ | ★☆☆ | ★☆☆ |
| Natural in error messages | ★★☆ | ★★★ | ★★★ | ★★★ | ★★★ | ★★☆ | ★★☆ | ★★☆ |
| Developer familiarity | ★★☆ | ★★★ | ★★★ | ★★☆ | ★★★ | ★☆☆ | ★☆☆ | ★★☆ |
| No term collisions | ★★★ | ★☆☆ | ★☆☆ | ★★☆ | ★☆☆ | ★☆☆ | ★☆☆ | ★★☆ |

### Recommendation: Ledger (Alternative 4)

**Recommended metaphor: Ledger**, with the following terminology:

| Role | Term | Rationale |
|------|------|-----------|
| System name | **Ledger** | Immediately evokes record-keeping and consistency. Distinctive. |
| Unit of knowledge | **Entry** | Short, clear. Each statement adds entries to the ledger. |
| Inconsistency | **Discrepancy** | Means precisely "things that should match but don't". |
| Checking action | **Audit** | The processor audits the ledger for discrepancies. |
| Scope decorator | **Tracked feature** | A feature whose entries are tracked in the ledger. |
| Declaration-time checking | **Pre-audit** | The ledger is audited before the scope is entered. |
| Execution-time checking | **Live audit** | The ledger is audited as each entry is recorded. |
| Configurable presets | **Ledger profile** | A named combination of tracked features. |
| User-defined rules | **Custom entry type** | A developer-defined entry type with associated rules. |

**Why Ledger wins:**
1. **Distinctiveness.** No mainstream language uses accounting terminology. Developers reading ish docs will never confuse "ledger" terminology with concepts from other languages.
2. **Metaphor coherence.** The bookkeeping analogy holds at every level: entries are recorded, books are audited, discrepancies are found, and the level of scrutiny is configurable. This maps one-to-one onto ish's system.
3. **Natural language quality.** Error messages read well: *"Discrepancy: entry @immutable(x) from line 2 conflicts with mutation of x at line 3. Audit trail: @immutable_by_default(block) at line 1."*
4. **Execution-time vs. declaration-time** maps naturally to **live audit** (real-time bookkeeping) vs. **pre-audit** (reviewing all books before proceeding).

**Mitigations for cons:**
- The blockchain association is fading and the word "ledger" existed for centuries before cryptocurrency. Context will make the meaning clear.
- "Entry" ambiguity is resolved by context — in the ledger system, "entry" always means a recorded fact.

**Runner-up: Proof (Alternative 5)** is the strongest conceptual fit, but the collision with the reasoning spec's existing use of "proposition" and the risk of setting formal-verification expectations make it less practical. However, one term from the Proof metaphor — **"rigor level"** — is worth borrowing regardless of the chosen metaphor. It is a more positive framing than "encumbrance level" for describing how much checking is enabled.

### Decisions

**Decision:** Which metaphor should ish adopt for its consistency-checking system? Alternatives: Agreement (status quo), Contract, Constraint, Ledger (recommended), Proof, Accord, Assay, Canon.
--> Ledger

**Decision:** Should "encumbrance level" be renamed to "rigor level" (or similar) regardless of which metaphor is chosen? The current term "encumbrance" has negative connotations — it suggests that static checking is a burden.
--> Yes

**Decision:** If not Ledger, is there a hybrid approach — e.g., keeping "agreement" as the system name but replacing "facts" with a better term like "entry" or "property"?
--> No.  I like the accounting metaphor.  But we should explore alternative terminology for the system name itself.  If we replace "Ledger" with "<adjective> Ledger", it will both help distance the terminology from blockchain's distributed ledger, and make the feature more searchable.  Please propose alternatives for an appropriate adjective.

---

## Feature: Syntax for the Consistency-Checking System

### Issues to Watch Out For

1. **The syntax must work across the entire encumbrance continuum.** In streamlined mode, a developer typing `let x = 7` shouldn't see any consistency-checking syntax at all. In encumbered mode, a developer should be able to configure detailed checking behavior. The syntax must gracefully span this range.

2. **Block-scoped configuration is already implied.** The prompt's example uses `[@immutable_by_default] { ... }` — a block-level annotation. This pattern is already in the conceptual design. Any syntax proposal must account for this.

3. **User-defined annotations must feel first-class.** The prompt describes a system where developers can create custom annotations with custom rules. If annotations use one syntax and built-in features use another, user-defined annotations feel second-class.

4. **Readability at scale.** In large codebases, the syntax for consistency checking will appear on a significant fraction of lines (in encumbered mode). Verbose syntax creates visual noise. Terse syntax creates a learning curve.

5. **IDE integration.** Whatever syntax is chosen must be easy for syntax highlighters, formatters, and LSP servers to handle.

### Critical Analysis

Five approaches are analyzed below, plus a recommended hybrid.

---

#### Approach A: Unified Annotation Syntax ("Everything is an annotation")

Every consistency-checking feature uses the same `[@...]` syntax, whether it's types, mutability, overflow behavior, or user-defined properties.

```ish
[@immutable_by_default] {
    [@type(int)] let x = 7;
    [@mutable] [@type(bool)] let y = true;
    [@overflow(wrapping)] [@type(u8)] let z = 255;
}
```

**Pros:**
- Maximum consistency. One syntax to learn, one mental model.
- User-defined annotations are syntactically indistinguishable from built-in ones, making them truly first-class.
- Makes the unified nature of the system visible — developers see that types, mutability, and overflow are all "the same kind of thing".
- Easy to implement: the processor treats all annotations uniformly.

**Cons:**
- Extremely verbose for common operations. `[@type(int)] let x = 7` is much worse than `let x: int = 7`.
- Alien-looking. Developers from any other language will find `[@type(int)]` bizarre when they're used to `x: int`.
- Fails the streamlined goal: even in encumbered mode, basic type annotations shouldn't feel foreign.
- Makes ish look like a research language, not a practical one.
- Searching for "all typed variables" requires parsing annotations rather than recognizing a `:` token.

---

#### Approach B: Hidden Native Syntax ("Dedicated syntax per feature")

Every consistency-checking feature gets its own purpose-built syntax. The unified system is an implementation detail that developers never see.

```ish
let x: int = 7;            // type annotation
let mut y: bool = true;    // mutability keyword
let z: wrapping u8 = 255;  // overflow modifier on type
async fn fetch(): string { ... }  // async keyword
fn process(): string throws IOError { ... }  // throws clause
```

**Pros:**
- Maximally familiar. Developers from TypeScript, Rust, Kotlin, Swift, etc. will recognize every construct immediately.
- Concise. Common features have minimal syntactic overhead.
- Each feature's syntax can be optimized for that feature's ergonomics.
- Streamlined code looks just like JavaScript/TypeScript. Encumbered code looks just like Rust/Kotlin. This is exactly the continuum ish aims for.

**Cons:**
- Each feature has different syntax, creating more to learn.
- The unified nature of the system is invisible. Developers won't realize that types, mutability, and overflow are all checked by the same mechanism — which means they won't intuit that user-defined annotations can do the same things.
- User-defined annotations necessarily use a different syntax (they can't get their own keywords), creating a visible gap between built-in and user-defined features.
- Adding new built-in features requires designing new syntax each time.
- Harder to implement: the processor must map many syntactic forms to the unified internal representation.

---

#### Approach C: Hybrid with Threshold ("Common features get native syntax, uncommon get annotations")

A defined set of common features (types, mutability, async, errors) get native syntax. Everything else (overflow behavior, polymorphism strategy, memory model, user-defined properties) uses annotation syntax.

```ish
let x: int = 7;                           // type: native
let mut y: bool = true;                   // mutability: native
async fn fetch(): string throws IOError { ... }  // async, throws: native

[@overflow(wrapping)] let z: u8 = 255;    // overflow: annotation
[@polymorphism(monomorphized)]
fn process(item: Processable) { ... }     // polymorphism: annotation
[@my_custom_rule] let w = compute();      // user-defined: annotation
```

**Pros:**
- Common features are familiar and concise.
- Uncommon features and user-defined properties share the same syntax, making user-defined annotations feel on par with uncommon built-in features.
- Developers learn native syntax first (streamlined), then annotations later (encumbered). Natural learning curve.

**Cons:**
- Inconsistent. Why does mutability get a keyword but overflow doesn't? The threshold is arbitrary and must be documented/justified.
- User-defined annotations still can't achieve keyword-level ergonomics.
- The system's unified nature is partially hidden, partially exposed — which might be more confusing than fully hiding or fully exposing it.

---

#### Approach D: Block Configuration + Native Feature Syntax

All consistency-checking configuration happens at block/module level via a `config` or `use` declaration. Individual features use native syntax. No item-level annotations.

```ish
use strict(types, mutability, null_safety, overflow);

let x: int = 7;         // type required because of strict(types)
let mut y: bool = true;  // mutability required because of strict(mutability)
let z: u8 = 255;         // overflow checking active because of strict(overflow)
```

**Pros:**
- Code body is clean — no annotations clutter individual lines.
- Configuration is centralized and easy to review.
- Familiar to developers who know JavaScript's `"use strict"` or Perl's `use strict`.
- Makes the encumbrance continuum very visible: the `use strict(...)` line is the control panel.

**Cons:**
- Per-variable configuration is impossible — you can't make just one variable immutable in a block where mutability isn't tracked.
- "All or nothing" at the block level doesn't support the per-variable encumbrance the spec envisions.
- User-defined annotations still need a separate mechanism for item-level attachment.
- Doesn't solve the syntax question for the features themselves (types, mutability, etc.) — you still need Approach B or C for those.

---

#### Approach E: Pragma + Annotation ("Layers")

Block-scoped pragmas configure *which* features are tracked. Annotations configure *per-item properties* when the developer wants to override defaults or attach metadata.

```ish
// Block-level: which features are tracked
[@track(types, mutability)] {
    let x: int = 7;                    // type: native syntax
    let mut y: bool = true;            // mutability: native keyword
    
    // Override per-variable
    [@overflow(wrapping)] let z: u8 = 255;
    
    // User-defined
    [@my_rule] let w = compute();
}
```

This is actually what the prompt's original example does — `[@immutable_by_default]` is a block-level pragma, and individual items use native syntax.

**Pros:**
- Clean separation between scope configuration (pragmas) and item decoration (annotations and native syntax).
- Supports per-variable overrides through annotations.
- User-defined annotations work at both levels (block-scoped rules AND item-level markers).
- Block pragmas can compose: `[@track(types)] [@overflow(wrapping)]` means "track types in this block and default to wrapping overflow".

**Cons:**
- Two syntactic mechanisms to learn (pragmas and annotations use the same `[@...]` syntax but serve different roles — this could be clarified or made confusing depending on docs).
- The difference between a pragma and an annotation may confuse beginners.

---

#### Approach F (not in original list): Expression-Level Ascription

Rather than annotations, some features could use inline ascription operators that feel like native syntax:

```ish
let x = 7 as int;           // type ascription
let y = value as! NonNull;   // null check ascription
let z = 255 as wrapping u8;  // overflow ascription
```

**Pros:**
- Looks like type casting, which developers already understand.
- Position-flexible: ascriptions can go on any expression, not just declarations.

**Cons:**
- Conflates checking with casting. `7 as int` looks like a conversion, not an annotation.
- Doesn't address block-level feature configuration at all.
- Verbose for declarations (the whole point of annotations is to be attached to declarations).
- Not suitable as a primary syntax — at best complementary.

---

### Comparative Summary

| Criterion | A: All Annotation | B: All Native | C: Hybrid Threshold | D: Block Config | E: Pragma+Annotation | F: Ascription |
|-----------|:-:|:-:|:-:|:-:|:-:|:-:|
| Familiar syntax for common features | ★☆☆ | ★★★ | ★★★ | ★★★ | ★★★ | ★★☆ |
| User-defined annotations feel first-class | ★★★ | ★☆☆ | ★★☆ | ★☆☆ | ★★☆ | ★☆☆ |
| Conciseness | ★☆☆ | ★★★ | ★★★ | ★★★ | ★★☆ | ★★☆ |
| Unified system is visible | ★★★ | ★☆☆ | ★★☆ | ★★☆ | ★★★ | ★☆☆ |
| Supports encumbrance continuum | ★★☆ | ★★★ | ★★★ | ★★☆ | ★★★ | ★★☆ |
| Supports per-variable configuration | ★★★ | ★★★ | ★★★ | ★☆☆ | ★★★ | ★★★ |
| Minimal learning surface | ★★★ | ★☆☆ | ★★☆ | ★★☆ | ★★☆ | ★★☆ |
| Extensible to new features | ★★★ | ★☆☆ | ★★☆ | ★☆☆ | ★★★ | ★☆☆ |

### Recommendation: Approach E (Pragma + Annotation), with native syntax for common features

The recommended syntax approach is **Approach E with elements of C**: block-level pragmas configure what is tracked, common features use native syntax (types via `:`, mutability via `mut`, async via `async`, throws via `throws`), and everything else — including user-defined annotations — uses `[@...]` syntax at item level.

```ish
// Project or module level: set the default profile
[@profile(encumbered)]

// Block-level: override specific tracking
[@track(overflow)] {
    let x: int = 7;                        // type: native (:)
    let mut y: bool = true;                // mutability: native (mut)
    async fn fetch(): string throws IOError {  // async, throws: native
        ...
    }
    
    [@overflow(wrapping)] let z: u8 = 255; // overflow: annotation
    
    // User-defined rule at item level
    [@my_domain.validated] let input = parse(raw);
}

// Streamlined mode: just write code
let x = 7;
x = "hello";  // Fine in streamlined — no discrepancy
```

**Key design points:**
1. **Common features never require `[@...]` syntax.** Types, mutability, async, and throws all have dedicated native syntax that every developer already knows.
2. **Uncommon features and user-defined rules use `[@...]`.** This is explicitly the extensibility mechanism.
3. **Block-level `[@track(...)]` and `[@profile(...)]` configure** what the system checks in that scope.
4. **The same `[@...]` syntax works at both block and item level,** but the semantics differ: block-level annotations configure tracking, item-level annotations attach entries.
5. **Profiles are presets** that bundle common configurations: `[@profile(streamlined)]` turns off most tracking, `[@profile(encumbered)]` turns on everything.

### Decisions

**Decision:** Which syntax approach should ish adopt? Alternatives: A (all annotation), B (all native), C (hybrid threshold), D (block config only), E (pragma + annotation, recommended), F (ascription). Or a combination?
--> Recommended

**Decision:** Should `[@...]` be the annotation syntax, or should ish use a different sigil? Alternatives: `@[...]` (Zig-like), `#[...]` (Rust-like), `@name` (Java-like), `[name]` (C#-like).
--> Zig-like.  Repeating the '@' on every element is ugly.  `#` is too comment-like. `[name]` is too array-like.

**Decision:** Should the pragma syntax (block-level) and the item-level annotation syntax be visually differentiated, or should they use the same `[@...]` form and rely on context to distinguish them?
--> They should be visually differentiated.  There are not many annotations that could be applied to either a block or a variable.

**Decision:** What is the exact set of features that should have native syntax vs. annotation syntax? The recommended cut: types (`:`), mutability (`mut`), async (`async`), error declaration (`throws`) get native syntax; everything else is annotation-based.
--> Let's start with that.

--> We need to consider syntax more carefully.
--> Annotations at the lexical scope override annotations at the outer lexical scope.  What does that look like?  Consider both more streamlined and more rigorous overrides.
--> Annotations on variables do not override.  They are cummulative.  Discrepancies cause an error to be thrown.
--> I want to see some example object type definitions that include entries for things other than structural types.
--> I want to see some example custom annotation definitions.
--> We need distinct terminology for block scoped and type scoped annotations.
--> I want to see some example preset definitions
--> Please build a table of all language features that will need annotations, and a proposed annotation for each.
--> Please build a complete syntax proposal along the lines covered here, with alternatives and decision points for the most controversial or problematic items.

---

## Documentation Updates

The following documentation files would be affected if these proposals are accepted:

- [docs/spec/agreement.md](docs/spec/agreement.md) — Core spec for the system. Would need full rewrite with new terminology and syntax.
- [GLOSSARY.md](GLOSSARY.md) — All agreement-related terms need updating: "Agreement", "Marked feature", "Encumbrance"/"Encumbered ish"/"Streamlined ish" (if "rigor level" is adopted).
- [docs/spec/types.md](docs/spec/types.md) — References to agreement and marked features throughout.
- [docs/spec/reasoning.md](docs/spec/reasoning.md) — The relationship between reasoning propositions and entries/facts.
- [docs/spec/execution.md](docs/spec/execution.md) — Declaration-time vs. execution-time checking terminology.
- [docs/spec/modules.md](docs/spec/modules.md) — Module-boundary interaction with the checking system.
- [docs/spec/polymorphism.md](docs/spec/polymorphism.md) — Polymorphism strategy as a tracked feature.
- [docs/user-guide/encumbrance.md](docs/user-guide/encumbrance.md) — User-facing explanation of the continuum.
- [docs/user-guide/types.md](docs/user-guide/types.md) — Type syntax examples.
- [docs/ai-guide/orientation.md](docs/ai-guide/orientation.md) — AI developer orientation.
- [docs/ai-guide/playbook-encumbered.md](docs/ai-guide/playbook-encumbered.md) — Encumbered mode playbook.
- [docs/ai-guide/playbook-streamlined.md](docs/ai-guide/playbook-streamlined.md) — Streamlined mode playbook.
- [docs/ai-guide/playbook-mixed.md](docs/ai-guide/playbook-mixed.md) — Mixed mode playbook.
- [docs/project/open-questions.md](docs/project/open-questions.md) — Several open questions would be answered or reframed.
- [AGENTS.md](AGENTS.md) — Key concepts section references agreement.

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-14-agreement-metaphor-and-syntax.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
