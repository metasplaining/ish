---
title: "RFP: Concurrency"
category: rfp
audience: [all]
status: draft
last-verified: 2026-03-23
depends-on: [docs/spec/assurance-ledger.md, docs/spec/execution.md, docs/spec/memory.md]
---

# RFP: Concurrency

ish supports two concurrency mechanisms: cooperative multitasking and parallel multitasking.

---

## Cooperative Multitasking

Cooperative multitasking is supported by an async mechanism similar to other languages, but improved by ish's assurance ledger system.

### Duration Entries

Each statement, block, or function carries an entry or entries describing how long it takes. (We will need to work through a careful analysis to determine how many of these entries a block can have, and whether they are mutually exclusive.)

| Entry | Meaning |
|-------|---------|
| `simple` | The block is guaranteed to execute "quickly." The exact definition of "quickly" is TBD, but something like less than a millisecond or fewer than 100,000 CPU cycles. |
| `complex` | The block is not guaranteed to execute quickly. |
| `yielding` | The block yields the CPU, so it can take an arbitrarily long time to execute. |
| `unyielding` | The block never yields the CPU. |
| `blocking` | The block can cause the whole thread to wait. |

### Await and Promise

A call to a yielding function may either await or accept a promise. If the caller awaits, then they also yield. If the caller accepts a promise, then they do not yield, and they may check the promise later to see if it has been fulfilled.

### Assurance Features

There are multiple assurance features related to cooperative multitasking:

- **Required await/promise** — When enabled, the caller of a yielding function must specify either that they will await or that they will accept a promise. When disabled, calling a yielding function defaults to accepting a promise, and `await` must be explicitly specified.
- **Guaranteed yield** — When enabled, complex unyielding blocks are not allowed. When disabled, they are allowed. To support this feature, system-provided loops support configurable yielding. For example, imagine a set of six nested loops, each iterating through 100 items. Executing the entire set would take 1,000,000,000,000 iterations of the innermost block, so the set of loops would get the `complex` entry. Because guaranteed yield is enabled, the set of loops needs to yield periodically to avoid a discrepancy. Yielding on every iteration of the innermost or second-to-innermost loop would be too frequent, causing excessive yielding overhead. Yielding on every iteration of the third-to-innermost loop might be too infrequent if the innermost block took more than 10 CPU cycles. So, the second-to-innermost loop might be configured to yield every 20 iterations. This would cause a yield to occur once every 2,000 iterations of the innermost block, thus avoiding yielding too frequently or too infrequently. Also, frequently the number of iterations through a loop is not known at build time. But by configuring the loop to yield, it can be guaranteed to yield frequently enough.

### Discrepancy: Blocking in Yielding Context

It is always a discrepancy to call a blocking function from within a yielding function.

### Terminology

We will need to work out the correct terminology, to be consistent with other languages that support async/await.

---

## Parallel Multitasking

Parallel multitasking is supported by assigning a function an operating system thread, allowing it to run on a separate CPU core. In order to implement parallel multitasking in Rust, we will need to comply with Rust's `Send`/`Sync` requirements.

### Design Approach

There are a few initial ideas, but nothing worked out clearly. We should research what other languages do and see if we can find something we can borrow for ish. Our approach should prioritize simplicity and correctness over completeness. We should identify the most common use cases (parallelizing the processing of a large list, reading and writing from queues, etc.) and make sure that it is easy to write them correctly. Less common use cases do not even need to be possible.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
