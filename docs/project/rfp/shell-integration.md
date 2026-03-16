---
title: "RFP: Shell Integration"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/spec/execution.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md, docs/architecture/shell.md, GLOSSARY.md]
---

# RFP: Shell Integration

*Converted from `shell` on 2026-03-15.*

---

## Context

It is time to integrate shell functionality into ish. The old prototype in `ish-workspace` serves as a reference — it is incomplete, but demonstrates how the pieces were envisioned fitting together using `redox_liner`.

## Questions

1. Are there alternative Rust libraries available to help with shell implementation? What are the pros and cons of each? Make a recommendation.

## Requirements

### Library Usage

In this version, ish should take advantage of as much functionality provided by the shell helper library as possible. The focus right now is not on the shell experience, and shell features that add significant complexity should be deferred.

### Shell Feature Assessment

Consider various shell features. Estimate how much complexity each one adds to the solution, and make a recommendation about whether it should be deferred.

### Security Standards

We will need standards (some already mentioned in previous RFPs and proposals) to protect against arbitrary shell execution. These standards need to be deferred since the assurance ledger is not yet implemented. However, we should still document their interfaces and functionality as part of implementing the shell. They should be clearly documented as deferred and added to the roadmap. Propose standards for this purpose. If any of them is ambiguous or controversial, propose alternatives with pros, cons, and a recommendation.

### Subshell

We will leave the subshell unimplemented for now, since implementing it requires that we first implement streams.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
