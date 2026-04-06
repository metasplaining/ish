---
title: "Proposal: Module System"
category: proposal
audience: [ai-dev, human-dev]
status: split
last-verified: 2026-04-05
depends-on: [docs/project/rfp/module-system.md, docs/spec/modules.md, docs/spec/execution.md, GLOSSARY.md]
split-into: [docs/project/proposals/module-system-core.md, docs/project/proposals/module-system-packages.md]
---

# Proposal: Module System

*Version 3 — split accepted; all decisions resolved on 2026-04-05.*

This proposal has been split into two child proposals:

- **[Proposal A — Module Core](module-system-core.md):** Visibility, forward references, module-to-file mapping, import syntax, project directive. Implementable without network dependencies.
- **[Proposal B — Package Management](module-system-packages.md):** Manifest format, versioning, diamond dependencies, proxy/caching, workspace, dependency tooling, traceability, package distribution. Builds on Proposal A.

This document retains the consolidated decision register, the design analysis, and the narrative rationale for the split. The child proposals are authoritative for implementation.

---

## Decision Register

All decisions resolved. The child proposals reference these by number.

| # | Topic | Decision |
|---|-------|---------|
| 1 | Implementation order | Module Core (Proposal A) first; Package Management (Proposal B) second |
| 2 | Version resolution | MVS — no lockfile; manifests are the source of truth |
| 3 | Diamond dependencies | Major version path suffixes (`example.com/foo/v2`); MVS selects one version per major |
| 4 | Visibility | Three levels: `priv`, `pkg` (default), `pub`. Interface files use `.ishi` extension. Auto-generated at low assurance; manually maintained at high assurance |
| 5 | Module identity | Default: one file = one module, path derived from filesystem position. Override: `mod { }` blocks within a file allow multiple modules with arbitrary paths, intended for standalone shell scripts only |
| 6 | Project manifest | JSON (`project.json`). All fields optional. `name` field (formerly `module`). Single `defaults` field for inheritance (formerly `parents` list). `name` is not inheritable from defaults |
| 7 | Workspace | `ish.work` file only. Replace directives deferred |
| 8 | Project resolution | Directive accepts filesystem path, URL path, or inline JSON object. `ISH_PROXY` resolves URL paths. Project file configures dependencies, assurance levels, and publishing metadata — not the module system itself |
| 9 | Defaults inheritance | Single `defaults` field (not a list). Precedence: project's own values > direct defaults > transitive defaults |
| 10 | Import syntax | Hierarchical with aliases: `use foo/bar`, `use foo/bar as b`, `use foo/bar { Type }`, `use foo/bar { Type as T }` |
| 11 | Package distribution transport | Proxy protocol (Go GOPROXY model). `ISH_PROXY` environment variable |
| 12 | Module-to-file mapping | Default: strict file = module. `mod { }` blocks override this within a file (shell-only use case) |
| 13 | Interface file error codes | Compile-time checks. Granular error codes: symbol-not-in-interface, symbol-not-in-implementation, symbol-mismatch, and others as discovered |
| 14 | Assurance and dependency levels | Assurance levels do not restrict which modules a module may depend on |
| 15 | Package format (minimum viable) | Interpreter loads ish packages from source. Interpreter loads Rust packages as shared libraries (`.so`). Rust-based standard library packages build into shared libraries. AST format is a near-term addition |

---

## Design Analysis

*This section responds to the question: what are the most complex or unexpected parts of this module system from a developer perspective, and can they be simplified?*

### 1. Two-Track Module Identity

The most surprising aspect of this design for developers coming from Go, Rust, or Python is the two-track module identity model. The default model (one file = one module, path derived from the filesystem) is straightforward and familiar. The `mod { }` block model (a file contains multiple modules with arbitrary paths) is fundamentally different and applies only to standalone shell scripts.

The risk is cognitive: a developer reading a shell script with `mod { }` blocks must hold a completely different mental model than they use for project code. This is not avoidable — the shell use case genuinely requires in-file mutual recursion across module boundaries, and the file-based model alone cannot satisfy this. However, the two models must be presented as distinct in the user guide. The assurance system's ability to flag `mod` directives as a discrepancy in project code provides a technical enforcement mechanism.

**Recommendation for the user guide:** Frame the file-based model as "the module system" and frame `mod { }` blocks as "shell script mode" — a separate, named mode with its own section. Developers should not encounter the two-track model as a single blended concept.

### 2. Interface File Lifecycle

The auto-generated → manually-maintained interface file transition is underspecified. When does a project "graduate" from generating `.ishi` files automatically to treating them as authoritative contracts? How does a developer initiate this transition?

The simplest model is presence-based: if a `.ishi` file exists in version control, the compiler treats it as authoritative regardless of assurance level settings. If it does not exist, tooling generates it on demand. This eliminates the need for an explicit mode switch and makes the behavior deterministic from the file system state alone.

A command like `ish interface freeze` (or `ish mod freeze-interfaces`) could generate all `.ishi` files and print guidance about committing them. This gives developers a clear action to take when they want to lock down a module's public contract.

**Open Question:** Should the `.ishi` file presence-based model replace the assurance-level-gated model, or complement it?
-->

### 3. The `project` Directive with Inline JSON

Allowing inline JSON objects inside ish source files is unusual. The use case is valid (standalone scripts that need a specific dependency without a separate file), but the implementation has non-trivial implications for the parser: it must recognize the boundary between the JSON object and the surrounding ish code.

An alternative that fits ish's own syntax better would be an inline ish object literal:

```ish
project { name: "myscript", dependencies: { "example.com/http": "v1.2.3" } }
```

This uses ish's own data syntax rather than embedding a foreign language. It would also compose naturally with ish's type system if project manifests are ever typed.

The inline JSON form is acceptable for the prototype since the parser already handles JSON-adjacent syntax. The inline ish object form is the better long-term design.

**Open Question:** Should the inline `project` directive use JSON object syntax or ish object literal syntax?
-->

### 4. MVS Without a Lockfile

Developers from Cargo or npm backgrounds expect a lockfile. MVS without a lockfile means builds are deterministic given the same manifest files, but builds can silently change when upstream packages release new higher minimum versions. The checksum file verifies integrity but does not pin versions.

This is a deliberate Go-inspired trade-off. It eliminates lockfile merge conflicts and keeps the manifest as the single source of truth. The trade-off is real and should be stated explicitly in the user guide early, not discovered by accident.

The design is correct. No change recommended. The user guide should include a "Why no lockfile?" section that explains the MVS guarantee and contrasts it with lockfile-based systems.

### 5. Package Format Duality

Ish packages can be ish source (interpreted or compiled) or Rust shared libraries. This is a necessary complexity: the I/O standard library is Rust-based and cannot be distributed as ish source. The minimum viable packaging system (D15) handles this correctly: the interpreter loads ish source directly and Rust packages as `.so` files.

The complexity is inherent and not simplifiable. What can be simplified is the developer-facing story: `ish publish` should determine the correct package format automatically based on whether the package contains ish source, a Rust crate, or both. Package authors should not need to know about `.so` vs. source vs. AST format distinctions unless they are building hybrid packages.

---

## Child Proposals

- **[Module Core](module-system-core.md)** — Visibility/Encapsulation, Forward References / Mutual Recursion, Module-to-File Mapping, Import Syntax / Use Directive, Project Directive and Hierarchy Lookup.
- **[Package Management](module-system-packages.md)** — Project Manifest Format, Versioning (MVS), Diamond Dependencies, Physical Downloading and Caching, Developer Workspace Flexibility, Dependency Management Tools, Dependency Inheritance, Traceability, Package Distribution.

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
- [docs/project/proposals/module-system-core.md](module-system-core.md)
- [docs/project/proposals/module-system-packages.md](module-system-packages.md)
