---
title: "RFP: Concurrency Correctness Fixes"
category: rfp
audience: [all]
status: draft
last-verified: 2026-04-01
depends-on: [docs/spec/concurrency.md, docs/architecture/vm.md, docs/spec/assurance-ledger.md]
---

# RFP: Concurrency Correctness Fixes

Three issues were identified in the concurrency prototype implementation that need correction.

---

## 1. FutureRef Equality Should Be Identity-Based

The `PartialEq` implementation for `Value::Future` currently returns `false` for all comparisons. The comment states this is because futures are "identity-based, not value-based," but the implementation does not actually perform identity comparison. Two futures should be equal if they are references to the same `Rc<RefCell<...>>` — that is what identity-based equality means.

---

## 2. Awaiting a Non-Future Must Always Be an Error

The interpreter currently handles `await` on a non-future value by silently returning that value. This is incorrect behavior. Regardless of assurance level, awaiting a non-future is an error:

- At low assurance, it should throw at runtime.
- At high assurance, it should throw at build time.

The concurrency design proposal was silent on this point. All appropriate documentation must be updated to specify this behavior.

---

## 3. Blocking Builtins Should Become Parallel Functions

Some builtins (`print`, `println`, `read_file`, etc.) perform blocking I/O. These should be converted to parallel functions that run on a blocking threadpool. Other builtins that perform pure computation (`len`, `type_of`, `keys`, etc.) do not block and should remain as they are.

ish should now have two kinds of builtins: **simple** (synchronous, pure computation) and **parallel** (run on `tokio::task::spawn_blocking`, perform I/O or other blocking operations).

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
