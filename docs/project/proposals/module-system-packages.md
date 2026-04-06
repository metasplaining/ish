---
title: "Proposal B: Package Management"
category: proposal
audience: [ai-dev, human-dev]
status: proposal
last-verified: 2026-04-05
depends-on: [docs/project/proposals/module-system.md, docs/project/proposals/module-system-core.md, docs/project/rfp/module-system.md, GLOSSARY.md]
---

# Proposal B: Package Management

*Derived from [module-system.md](module-system.md) v3 on 2026-04-05.*

Package Management covers cross-project dependency management: manifest format, versioning, diamond resolution, proxy downloading, workspace overrides, dependency tooling, defaults inheritance, traceability, and package distribution. It builds on [Proposal A — Module Core](module-system-core.md).

---

## Decision Register

Decisions from the parent proposal relevant to this proposal. See [module-system.md](module-system.md) for the full register.

| # | Topic | Decision |
|---|-------|---------|
| 1 | Implementation order | Package Management (this proposal) second, after Module Core |
| 2 | Version resolution | MVS — no lockfile; manifests are the source of truth |
| 3 | Diamond dependencies | Major version path suffixes; MVS selects one version per major |
| 6 | Project manifest | JSON (`project.json`). All fields optional. `name` field. Single `defaults` field for inheritance. `name` is not inheritable |
| 7 | Workspace | `ish.work` file only. Replace directives deferred |
| 9 | Defaults inheritance | Single `defaults` field. Precedence: project's own values > direct defaults > transitive defaults |
| 11 | Package distribution transport | Proxy protocol. `ISH_PROXY` environment variable |
| 14 | Assurance and dependency levels | Assurance levels do not restrict which modules a module may depend on |
| 15 | Package format (minimum viable) | Interpreter loads ish packages from source. Interpreter loads Rust packages as shared libraries (`.so`). Rust standard library packages build into shared libraries. AST format is a near-term addition |

---

## Feature: Project Manifest Format

### Issues to Watch Out For

- The manifest format sets the tone for the developer experience — too much required metadata discourages casual use.
- ish has multiple execution configurations that may need representation.
- The `name` field is optional but not inheritable — this asymmetry needs clear documentation.

### Design

The project manifest is a **JSON file** named `project.json`. All fields are optional (D6). A valid manifest may be an empty JSON object `{}`.

```json
{
  "name": "example.com/myproject",
  "ish": "0.1.0",
  "defaults": "example.com/org-defaults v1.0.0",
  "dependencies": {
    "example.com/lib": "v1.2.3",
    "example.com/util": "v0.4.0"
  },
  "retract": ["v0.3.0", "v0.3.1"],
  "description": "...",
  "license": "Apache-2.0",
  "authors": ["..."],
  "repository": "https://..."
}
```

**Field notes:**

- `name`: The package path for this project (e.g., `example.com/myproject`). Optional — unnamed projects can be developed locally but cannot be published to a proxy. **Not inheritable from `defaults`**: each project must declare its own name.
- `ish`: The minimum ish version required.
- `defaults`: A single package reference (e.g., `example.com/org-defaults v1.0.0`) whose manifest provides default values for fields not set in this manifest. See Dependency Inheritance feature.
- `dependencies`: A map of package paths to version constraints.
- `retract`: Versions of this package that are retracted and should not be selected by dependents.

**Inheritance precedence (D9):** A field's value is taken from the first source that specifies it:
1. This manifest's own value
2. The direct `defaults` package's manifest
3. Transitive defaults (the defaults of the defaults, and so on)

The `name` field is excluded from inheritance: it is always local.

**Materialization tooling:** `ish mod materialize` generates a fully resolved, annotated version of the manifest showing the source of each inherited field. This is generated on demand for debugging and auditing; it is not stored.

### Acceptance Tests

- An empty `{}` is a valid project manifest.
- A manifest with only `name` is valid.
- A manifest with a `defaults` package that declares `dep/foo v2.0.0` uses that version if the child does not specify `dep/foo`.
- A manifest that declares `dep/foo v1.5.0` overrides the defaults package's declaration of the same.
- The `name` field from a `defaults` package is ignored; the child project's own `name` is used.
- An unresolvable path in `dependencies` produces a `package/not-found` error.
- `ish mod materialize` shows the source of each field in the resolved manifest.

---

## Feature: Dependency Inheritance / Defaults

### Issues to Watch Out For

- Defaults inheritance is distinct from package dependencies: defaults packages contribute configuration, not code.
- The single-defaults model eliminates diamond conflicts in the inheritance chain.
- Transitive defaults must be fetched to compute inherited values; this requires tracking the fetch chain.

### Design

The `defaults` field names a single package that acts as a Bill of Materials (BOM) and configuration provider for the project. The defaults package is a regular package distributed through the proxy system. It typically contains only a `project.json` — no ish source files.

**Use cases:**
- **Organization BOM:** An organization publishes a defaults package that declares approved versions of common dependencies. All projects in the organization list this as their `defaults`.
- **Assurance configuration:** A defaults package declares assurance level settings for the organization.
- **Toolchain defaults:** A defaults package specifies the default ish version and execution configuration.

**Inheritance chain:** A defaults package may itself declare a `defaults` field. The chain is traversed to resolve values not found in earlier entries. Cycle detection is performed during traversal: if a package appears more than once in the defaults chain, a `defaults/cycle` error is produced with the full cycle path.

**Single defaults (not a list):** The change from multiple `parents` to a single `defaults` field (D6, D9) eliminates the ambiguity of "which parent wins" for conflicting values. Organizational defaults hierarchies are expressed as a chain (each level declares one `defaults`), not a tree.

### Acceptance Tests

- A project with `defaults` inherits a dependency version not declared in its own manifest.
- A project explicitly declaring a dependency version overrides the inherited value.
- A defaults chain A → B → C where C declares a value and A does not: A inherits from C.
- A defaults chain A → B → A produces a `defaults/cycle` error naming the cycle.

---

## Feature: Versioning (MVS)

### Issues to Watch Out For

- Pre-1.0 versions need different compatibility expectations; the major version path model applies to v0 packages as well.
- Version resolution must be deterministic across machines without a lockfile.
- Retracted versions must be excluded without complicating resolution logic.

### Design

All packages use **semantic versioning** (major.minor.patch). Version resolution uses **Minimal Version Selection (MVS)**: across the transitive dependency graph, select the minimum version of each package that satisfies all constraints. Since each constraint is a minimum requirement, the selected version is always the highest minimum — making resolution deterministic without a lockfile.

**Why no lockfile:** The manifest files are the complete specification of the build. MVS selects a deterministic version from them. Two machines with the same manifests select the same versions. This eliminates lockfile merge conflicts. The trade-off is that builds can change as upstream packages raise their minimum versions; this is a deliberate design choice.

**Major version path suffixes:** Major version changes require a new package path (e.g., `example.com/foo/v2`). Within a major version, exactly one version is in the build. This prevents type identity problems from having two copies of the same major version.

**`retract` field:** Package authors use the `retract` field to mark versions that contain serious bugs or were published in error. MVS skips retracted versions during resolution.

### Acceptance Tests

- Two dependencies requiring `v1.2.0` and `v1.3.0` of a common package resolve to `v1.3.0`.
- A retracted version is not selected even if it would otherwise satisfy all constraints.
- Given identical manifest files on two machines, resolution selects identical versions.

---

## Feature: Diamond Dependencies / Multiple Versions

### Issues to Watch Out For

- Type identity: values of the same type from two different major versions are not compatible.
- Multiple major versions of the same package bloat the build.

### Design

Within a major version, MVS guarantees exactly one version is in the build. Different major versions are treated as **distinct packages** with distinct paths. A project may simultaneously depend on `example.com/foo v1.5.0` and `example.com/foo/v2 v2.1.0` — they have different paths and there is no conflict.

Package authors are responsible for updating the package path on major version bumps. This is an explicit burden on the author, not the consumer. The benefit is that every `use` statement is unambiguous about which major version it imports.

### Acceptance Tests

- A project depending on both `example.com/foo` (v1.x) and `example.com/foo/v2` (v2.x) compiles without conflict.
- Two transitive dependencies requiring different minor versions of the same major version resolve to the higher minor version (MVS).

---

## Feature: Physical Downloading and Caching

### Issues to Watch Out For

- Builds must work offline after initial fetch.
- Downloaded packages must be verified against tampering.
- Private packages must bypass public checksum servers.

### Design

ish uses a **proxy protocol** for package distribution, modeled on Go's GOPROXY (D11). Package paths are URL-like; the tool resolves them through a configurable proxy chain.

- **Cache directory:** `~/.ish/cache/` stores downloaded packages as content-addressed archives.
- **Checksum verification:** All downloaded packages are verified against a checksum database. The local checksum file records hashes of all dependencies for the project.
- **`ISH_PROXY`:** Configures the proxy chain (e.g., `https://proxy.ish.example.com,direct`). `direct` means VCS resolution. `off` disables network access entirely.
- **`ISH_PRIVATE`:** Configures package path prefixes that bypass the proxy and checksum server (for private packages).
- **Vendoring:** `ish mod vendor` copies all dependencies into a `vendor/` directory. When a `vendor/` directory is present, ish uses it exclusively; no network access occurs.

### Acceptance Tests

- A package downloaded once is served from cache on subsequent builds without network access.
- A package whose downloaded content does not match the recorded checksum produces a `package/checksum-mismatch` error.
- With `ISH_PROXY=off`, no network access occurs; resolution uses cache and vendor only.
- `ish mod vendor` produces a `vendor/` directory; subsequent builds use it.

---

## Feature: Developer Workspace Flexibility

### Issues to Watch Out For

- Workspace overrides must not leak into published packages.
- Workspace files must be safe to commit to version control for team use, or safe to gitignore for personal use.

### Design

A workspace file (`ish.work`) lists local module directories to use instead of fetching from the proxy:

```json
{
  "use": [
    "./mylib",
    "./myapp"
  ]
}
```

When `ish.work` is present, ish uses the local filesystem paths for the listed modules. The published `project.json` of those modules is unaffected.

The workspace file is added to `.gitignore` by default. Teams that want a shared workspace file may commit it; the workspace file contains no version information and does not affect reproducibility.

Replace directives (overriding a single dependency in the manifest) are deferred to a future revision.

### Acceptance Tests

- With `ish.work` pointing `example.com/mylib` to `../mylib`, `use example.com/mylib` resolves to the local directory.
- Removing `ish.work` reverts to proxy resolution.
- The published manifest of `mylib` is unaffected by the workspace override.

---

## Feature: Dependency Management Tools

These tools are emergent properties of MVS, the proxy protocol, and the manifest format. No independent design decisions are required.

| Command | Purpose |
|---------|---------|
| `ish mod tidy` | Adds missing and removes unused dependencies from the manifest |
| `ish mod add <pkg>[@version]` | Adds a dependency to the manifest |
| `ish mod remove <pkg>` | Removes a dependency from the manifest |
| `ish mod graph` | Prints the full transitive dependency graph |
| `ish mod why <pkg>` | Explains why a package is in the dependency graph |
| `ish mod verify` | Verifies checksums of all downloaded packages |
| `ish mod materialize` | Generates an annotated manifest showing the source of each inherited field |
| `ish mod vendor` | Copies all dependencies into `vendor/` for hermetic builds |

---

## Feature: Traceability

### Issues to Watch Out For

- Traceability must work across the assurance continuum.
- Build provenance must be embeddable in compiled artifacts.
- Assurance levels do not restrict which dependencies a module may use (D14).

### Design

Three traceability mechanisms:

1. **Checksum file** — records cryptographic hashes of all resolved dependencies. Stored in the project directory alongside the manifest. Used by `ish mod verify` for tamper detection and reproducibility verification.

2. **Embedded build info** — compiled artifacts embed the package path, version, and dependency hashes of everything in the build. This enables auditing any deployed artifact by inspecting its metadata. At high assurance, embedded build info is required; at low assurance, it is optional.

3. **Dependency graph tools** — `ish mod graph`, `ish mod why`, and `ish mod verify` (see Dependency Management Tools).

Assurance levels affect the requirement for embedded build info, not which dependencies may be used.

### Acceptance Tests

- A compiled artifact includes the list of dependencies and their versions in its metadata.
- `ish mod verify` detects a package whose downloaded content has changed since last verification.

---

## Feature: Package Distribution

### Issues to Watch Out For

- Packages may contain ish source, compiled Rust code, or both.
- The Rust-based standard library (I/O and other low-level packages) cannot be distributed as ish source.
- The interpreter and the compiler have different loading requirements.

### Design

**Package format constraints (D15):**

| Runtime | ish source package | Rust package |
|---------|-------------------|--------------|
| Interpreter | Loads directly from source (`.ish` files) | Requires shared library (`.so`) |
| Compiler | Compiles from source | Requires static library (`.a`) |

**Minimum viable packaging system:**

The minimum viable system targets the interpreter and source-based packages, plus Rust shared libraries for the standard library:

1. The interpreter loads ish packages directly from source files.
2. The interpreter loads Rust-based packages from shared libraries (`.so`).
3. The Rust-based standard library is built and distributed as shared libraries.

This supports the full interpreter experience, including I/O and other stdlib functionality. Compiled execution of Rust-based packages requires `.a` format and is a subsequent step.

**AST package format:** The AST format (pre-parsed, pre-analyzed package content) is a near-term addition to the minimum viable system. It reduces startup time for interpreted execution and provides an intermediate step between source distribution and compiled distribution.

**Distribution evolution:**

1. **Phase 1 (current target):** Source packages distributed via the proxy protocol. Rust packages distributed as shared libraries. Proxy serves zip archives containing source files and `project.json`.
2. **Phase 2:** AST format packages. Precompiled artifacts for ish packages.
3. **Phase 3 (future):** Dedicated registry with search, documentation, and discovery.

**`ish publish`:** Should determine the correct package format automatically based on the package contents. Package authors should not need to know about `.so` vs. source vs. AST distinctions for typical use cases.

**Open Question:** What is the on-disk layout of a shared library package? Does it follow the same `project.json` + source layout with a compiled artifact alongside, or is it a separate format entirely?
-->

### Acceptance Tests

- A pure ish source package is interpreted without any compilation step.
- A Rust-based package (`.so`) is loaded by the interpreter.
- A package distributed as source via the proxy is downloaded, cached, and loaded by the interpreter.
- `ish mod verify` works for packages in all supported formats.

---

## Documentation Updates

| File | Change |
|------|--------|
| `docs/spec/modules.md` | Add package management, versioning, proxy, manifest sections |
| `docs/spec/execution.md` | Note that package format affects execution configuration options |
| `GLOSSARY.md` | Add/update: project manifest, `defaults` package, BOM, MVS, proxy, `ISH_PROXY`, `ISH_PRIVATE`, workspace, vendor, checksum file, retract, shared library package |
| `docs/architecture/vm.md` | Module loading for source and shared library packages |
| `docs/architecture/codegen.md` | Package loading for compiled execution |
| `docs/user-guide/modules.md` | Package management section; "Why no lockfile?" explanation |
| `docs/errors/INDEX.md` | Add: `package/not-found`, `package/checksum-mismatch`, `defaults/cycle`, `package/version-retracted` |

---

## Referenced by

- [docs/project/proposals/module-system.md](module-system.md)
- [docs/project/proposals/INDEX.md](INDEX.md)
