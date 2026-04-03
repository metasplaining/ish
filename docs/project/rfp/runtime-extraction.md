---
title: "RFP: Runtime Extraction"
category: rfp
audience: [all]
status: draft
last-verified: 2026-04-02
depends-on: [docs/architecture/runtime.md, docs/architecture/vm.md, docs/architecture/codegen.md]
---

# RFP: Runtime Extraction

Extract the runtime from the VM to set conditions for package implementation.

---

## 1. Compiled Package Architecture

A compiled ish package is a shared library that defines a catalog and a set of shims. In order to build a package, the Rust compiler needs the shim definition. We don't want every ish package to depend on the `ish-vm` crate, so we need to move the definition of `Shim` to the `ish-runtime` crate. The `ish-vm` and all compiled ish packages will depend on this crate.

---

## 2. Move Value to ish-runtime

We will also have to move `Value` to the `ish-runtime` crate, because shims depend on it. This is a good thing. We will be able to build compiled ish packages that use garbage-collected, dynamically typed `Value`s internally. This is the compiler's base-case output. The compiler produces statically typed, reference-counted code only as an optimization when the necessary constraints are met.

---

## 3. Other Types to Move

We should determine what other types should be moved to the runtime. Error codes are a good choice. Are there any other types that would be useful to compiled functions in ish packages?

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
