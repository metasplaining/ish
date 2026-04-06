---
title: Module System — Design History
category: project
audience: [all]
status: current
last-verified: 2026-04-05
depends-on: [docs/project/proposals/module-system.md, docs/project/rfp/module-system.md]
---

# Module System — Design History

*April 5, 2026*

This directory captures the evolution of the module system design proposal. Each version of the proposal is preserved as a separate file, and this summary describes the deliberation that drove changes between versions.

---

## Version 1 — Initial Design Proposal

*Generated on 2026-04-05 from [the RFP](../../rfp/module-system.md).*

The initial proposal was generated in response to an RFP covering eleven problems in the module system design space: physical downloading, versioning, diamond dependencies, encapsulation, forward references, project metadata, workspace flexibility, shell/script integration, dependency simplification, dependency inheritance, and traceability. The agent researched five languages (Go, Rust, OCaml, Haskell, Scheme) and organized the findings into feature-by-feature alternatives analyses.

The proposal left all twelve decision register entries as "pending," with alternatives analyses and recommendations for each. The human then reviewed the proposal and added inline decisions.

Full text: [v1.md](v1.md)

---

## Version 2 — Decisions Incorporated

*Revised on 2026-04-05 from inline decisions in Version 1.*

The human reviewed the initial proposal and made eleven inline decisions (Decision 1 — implementation order — remained pending). The decisions represent significant departures from some of the agent's initial recommendations and reflect a coherent design philosophy: prefer Go-style simplicity where possible, but enrich the model in places where ish's unique properties (assurance continuum, shell/project duality, organizational use cases) create needs that Go's model doesn't address.

### Key decisions and their rationale

**Visibility (D4):** The agent recommended Rust-style annotation-based visibility with three levels (private, pub, pub(global)). The human specified a different three levels — `priv`, `pkg`, `pub` — where `pkg` is the **default**. This inverts the Rust default: items are project-visible unless explicitly restricted. Additionally, the human introduced the interface file concept: generated at low assurance, manually maintained at high assurance. This brings OCaml's `.mli` file discipline into ish's assurance continuum, where higher assurance demands more explicit contracts.

**Forward references (D5):** The agent recommended no cross-module cycles with interface-based decoupling. The human accepted this but clarified the semantics precisely: the compilation unit is the source file (not the project or the directory), mutual recursion within a file is fully supported, and the shell REPL needs an atomic module declaration syntax for interactive use.

**Project manifest format (D6):** The agent recommended a minimal declarative file similar to Go's `go.mod`. The human accepted the minimal declarative approach but specified JSON as the format and added **manifest inheritance**: projects may declare an ordered list of parent projects whose manifests are merged to provide defaults. This addresses an organizational need that Go's module system handles poorly — standardizing dependency versions and assurance configuration across many projects. The materialization tooling idea (generating an annotated manifest showing the source of each inherited value) was added by the human as a debugging affordance.

**Shell-mode / project lookup (D8):** The agent proposed a "script mode" vs. "project mode" distinction based on whether a manifest file was present. The human replaced this with a cleaner model: a `project` directive explicitly names a project file; without one, a directory hierarchy search finds the nearest `project.json`; and the ish installation provides a root default. This eliminates the mode distinction entirely — every file is in some project — while keeping the single-file experience frictionless.

**Dependency inheritance (D9):** The human clarified that project inheritance via `parents` is distinct from package dependencies. Parent projects may contain no code at all and serve purely as BOMs or assurance configuration providers. Because parent projects are regular packages distributed through the same proxy system, they inherit the stability and reproducibility guarantees of the package mechanism. This was a deliberate reuse of the package abstraction rather than introducing a new distribution channel.

**Decisions with minimal deviation:** Decisions 2 (MVS), 3 (major version path suffixes), 7 (workspace files), 10 (hierarchical imports with aliases), 11 (proxy protocol), and 12 (strict file=module) all followed the agent's recommendations closely.

### Gap detection findings

The revision identified five open questions not raised in the initial proposal:

1. **Interface file naming convention** — the decision to add interface files left their file format and naming unstated.
2. **Interface file consistency checking** — what error is produced when a manually maintained interface file contradicts the implementation?
3. **Shell atomic module declaration syntax** — the decision mentions this syntax exists but does not specify it.
4. **Manifest inheritance cycle detection** — if parent projects form a cycle, how and when is this detected?
5. **`project` directive bootstrapping** — if the directive references a package path, the module system is needed to fetch the project file before the module system is configured; the resolution order needs specification.
6. **Assurance level inheritance conflicts** — which parent wins when parents specify conflicting assurance levels?

### Split evaluation

With 14 independent implementation steps, the proposal exceeds the 10-step threshold for a split recommendation. The natural split separates **Module Core** (in-process module system: visibility, forward references, file mapping, import syntax, project directive lookup) from **Package Management** (cross-project dependency management: manifest format, versioning, diamond resolution, proxy, workspace, tooling, traceability, distribution). The split also resolves the pending Decision 1 (implementation order) by making Module Core the first phase.

The split decision is presented to the human as a decision point in the proposal body.

Full text: [v2.md](v2.md)

---

## Version 3 — Split Accepted; Design Analysis Added

*Revised on 2026-04-05 from inline decisions in Version 2.*

Version 2 left fourteen inline decisions unanswered. The human reviewed the full proposal and resolved all of them in a single pass. The decisions span every feature section and collectively produce a significantly more specific design. A split of the proposal into two child proposals — Module Core and Package Management — was also accepted.

### Key decisions and their rationale

**Module identity revised (D5, D12):** The earlier design said "file is the compilation unit" without specifying how the shell REPL would support interactive mutual recursion. The human resolved this with a `mod { }` block syntax: a file with no `mod` directives defines one module at the filesystem-derived path; a file with `mod` directives defines multiple modules at arbitrary paths. Critically, this revision changes the earlier "strict file = module" rule — files are still the dominant model, but `mod` blocks are a sanctioned escape hatch for standalone shell scripts. The assurance system can flag `mod` directives in multi-file projects as a discrepancy, providing enforcement without completely prohibiting them at low assurance.

**Interface files specified (D4, D13):** The human resolved the two remaining open questions about interface files. The extension is `.ishi`. Consistency checks are performed at compile time, not lazily at the point of use. Error codes must be granular — separate codes for symbol-not-in-interface, symbol-not-in-implementation, and symbol-mismatch rather than a single "interface mismatch" code.

**Import syntax examples corrected (D10):** The examples in v2 incorrectly used full external-package paths for within-project imports; this was a leftover from an earlier design iteration. The corrected form uses module-relative paths for within-project imports.

**Project directive expanded (D8):** The bootstrapping open question in v2 (how does a `project` directive reference a package before the module system is initialized?) was resolved by clarifying that project files do not configure the module system. They configure dependencies, assurance levels, and publishing metadata. Fetching a project file via URL requires only `ISH_PROXY`, a plain environment variable. The `project` directive now accepts three forms: a filesystem path to a JSON file, a URL to a JSON file, or an inline JSON object.

**Manifest simplified (D6):** Three changes to the manifest format. First, all fields are optional — not just the metadata fields. A project manifest may be an empty object. Second, the `module` field is renamed `name`. Third, the `parents` list (multiple inheritance) is replaced by a single `defaults` field (single inheritance). The simplification eliminates the "which parent wins" ambiguity and makes the inheritance chain a linear sequence rather than a tree.

**`name` not inheritable (D6):** The `name` field is the one field that cannot be inherited from a `defaults` package. Each project must declare its own name, or be anonymous.

**Defaults precedence (D9):** With multiple `parents` removed, the precedence rule became: the project's own values take precedence over the direct `defaults` package's values, which take precedence over transitive defaults values.

**Assurance levels and dependencies (D14):** The v2 design included an assurance integration section in the Traceability feature that said high-assurance modules cannot depend on low-assurance packages without explicit boundary annotations. The human struck this design as incorrect — assurance levels do not restrict which modules a module may depend on. The acceptance test for this rule was also struck.

**Package distribution constraints (D15):** The v2 three-phase distribution evolution was replaced by a more grounded specification of the minimum viable packaging system. The critical insight is that ish packages come in two fundamentally different forms: ish source packages and Rust compiled packages. The Rust-based standard library (I/O and other low-level modules) cannot be distributed as ish source. The minimum viable system handles this: the interpreter loads ish source directly and Rust packages as shared libraries (`.so`). The AST format is identified as a near-term addition once the minimum viable system works. The three-phase distribution evolution is deferred for full specification later.

**Split accepted:** The split evaluation from v2 was accepted. The proposal is marked `status: split` in its frontmatter and replaced by two child proposals. The split also resolves Decision 1 (implementation order): Module Core first, then Package Management.

### Design analysis added

In addition to resolving the inline decisions, v3 includes a design analysis section responding to the question: *What are the most complex or unexpected parts of the module system from a developer perspective, and can they be simplified?*

The analysis identifies five areas of complexity:

1. **Two-track module identity** — the file-based and `mod`-block-based models must be framed as distinct modes, not blended. The recommendation is to call them "the module system" and "shell script mode" respectively in the user guide.

2. **Interface file lifecycle** — the auto-generated → manually-maintained transition is underspecified. A presence-based model (committed `.ishi` file = authoritative; absent = auto-generated) was proposed as a simplification, with an `ish interface freeze` command to initiate the transition. This is left as an open question for a future revision.

3. **`project` directive with inline JSON** — inline JSON inside ish source files is unusual and creates parser complexity. An alternative using ish's own object literal syntax was proposed as the better long-term design. Left as an open question.

4. **MVS without a lockfile** — a deliberate trade-off that will surprise developers from Cargo or npm backgrounds. No change recommended; the user guide should explain the trade-off explicitly.

5. **Package format duality** — inherent in ish's design. `ish publish` should determine the correct format automatically so package authors do not need to understand the details.

Full text: [v3 — module-system.md](../../proposals/module-system.md), [module-system-core.md](../../proposals/module-system-core.md), [module-system-packages.md](../../proposals/module-system-packages.md)

---

## Version 4 — Core Proposal Revised: Interface Freeze, Declare Blocks, Export Alternatives

*Revised on 2026-04-05 from inline decisions in [module-system-core.md](../../proposals/module-system-core.md).*

The human reviewed the first version of the Module Core child proposal and made two inline decisions, asked for an alternatives analysis on a third point, and flagged one further design question. The changes affect two features substantially and introduce a new open question about package-level exports.

### Interface file generation revised (D4 update)

The earlier design described a two-track interface file model: auto-generated at low assurance, manually maintained at high assurance. The human replaced this with a simpler presence-based model. Interface files are never generated automatically. The developer explicitly generates them by running `ish interface freeze [module_name]`. Once a `.ishi` file exists in the project, the compiler enforces it: all `pub` declarations in the `.ishi` file must match the implementation exactly. Regenerating the file with `ish interface freeze` overwrites the existing one.

The significance of the change is that the auto-generation / manual-maintenance distinction disappears. The question is simply whether a `.ishi` file is present. This eliminates the need for the compiler to consult the assurance level when deciding whether to enforce the interface file. The assurance system continues to play a role in whether the project *should* have interface files (high-assurance modules are expected to), but that is an advisory concern, not a mechanical one.

The interface file content is now specified precisely: it contains only the `pub` declarations from the implementation. The earlier design was vaguer about what the generated file would contain.

### `mod { }` blocks replaced by `declare { }` blocks (D5 update)

The earlier design introduced `mod { }` blocks as a way for single-file shell scripts to define multiple named modules at arbitrary paths. The human discarded this design entirely and replaced it with `declare { }` blocks, which are a simpler and fundamentally different construct.

`declare { }` blocks are anonymous — they do not introduce module paths. They are declarations-only groupings: function and type definitions are permitted inside; top-level command invocations and function calls are not. Their primary purpose is to allow mutually recursive declarations to be stated as a unit, both in source files and interactively in the REPL.

The more consequential part of the change is the *implicit* declare wrapping rule: when a file is pulled into compilation via `use`, the compiler implicitly wraps the file's contents in a `declare { }` block. Any top-level command or function call in the file causes a `module/script-not-importable` error. This creates a clean, enforceable distinction between runnable scripts (which may have top-level commands) and importable modules (which may not). The same file can be both if it contains only declarations.

The strict file=module mapping rule, which previously had an exception for `mod`-directive files, is now truly without exception. `declare { }` blocks are anonymous and do not affect the module namespace.

### Top-level package exports (D15 — new open question)

The human raised a new design problem: how should a package expose a curated top-level API so that callers can `use example.com/http { Client }` rather than `use example.com/http/client { Client }`? Currently, a directory path does not resolve to any module because no file is named after the directory.

Rather than making a decision, the human asked for an alternatives analysis. The revised proposal presents five alternatives:

- **Option A (`public.ish`):** The user's own suggestion. A file named `public.ish` in directory `foo/` maps to module path `foo`. Explicit intent, readable, but reserves the filename `public` within any directory.
- **Option B (`index.ish`):** Same semantics, Node.js convention. Familiar to JS developers, less inherently readable.
- **Option C (`pkg.ish`):** Same semantics, name drawn from ish's `pkg` visibility keyword. Arguably consistent with the vocabulary, but the overloading of `pkg` at two levels of abstraction is a concern.
- **Option D (manifest entry point):** The project manifest declares an `entry` field pointing to the module to use as the top-level API. No magic filenames; maximum flexibility; slightly more friction for callers.
- **Option E (no mechanism):** Callers always use full sub-module paths. Zero complexity, poor ergonomics for large packages.

The proposal notes that Options A and D are the strongest candidates and leaves D15 as an open question for the human to decide.

Full text: [v4 snapshot](v4.md), revised proposal at [module-system-core.md](../../proposals/module-system-core.md)

---

## Version 5 — `index.ish` Adopted; Cross-Tree Project Question Raised

*Revised on 2026-04-05 from inline decisions in [module-system-core.md](../../proposals/module-system-core.md).*

The human reviewed the previous revision and resolved the top-level package export question, added a clarification on the escape mechanism, and raised a new open question about cross-tree project membership.

### D15 resolved: `index.ish`

From five alternatives presented in v4, the human chose `index.ish`. A file named `index.ish` in any directory maps to the module path of that directory rather than `<dir>/index`. The convention applies at every level of the directory hierarchy, not just the project root — `src/net/index.ish` defines module `net`, and `src/net/http/index.ish` defines module `net/http`. `index` is a reserved filename with no escape mechanism: there is simply no way to have a module named `foo/index`.

The conflict rule is also specified: if both `foo.ish` and `foo/index.ish` exist in the source tree, the build fails for both with `module/path-conflict`. The prohibition is symmetric — neither file wins; the author must resolve the conflict.

The decision was incorporated into D12 (module-to-file mapping) and D15 (top-level exports), and the import syntax section was updated to show `use net` resolving via `index.ish`.

### Escape mechanism question resolved

Version 4 included an open question under the (eventually unchosen) Option A analysis: "What if someone legitimately needs a module named `foo/public`?" The human answered "an escape mechanism is not needed," which establishes the general principle: reserved filenames are simply unavailable as module names. Since `index.ish` was chosen, the operative reserved name is `index`, and the same principle applies. The proposal does not discuss escape mechanisms.

### New open question: cross-tree project membership (D16)

The human raised a new design question not previously addressed: if an ish file uses the `project` directive to reference a project file located elsewhere on the filesystem, what happens? The directive was designed for the case where a standalone script in an isolated directory needs to specify which project it belongs to. But it also creates the possibility of a file inserting itself into a project whose source root does not contain it.

The revised proposal analyzes this scenario in a new open question section (D16). Three implications are identified. First, module path derivation fails: the cross-tree file is not under the source root, so no module path can be assigned. The file can reference the project but cannot be imported by other project modules. Second, visibility: if the file is accepted as a project member, it gains `pkg` access to the entire project's internals — this breaks the assumption that `pkg` visibility is controlled by the project author. Third, security: write access to any directory is sufficient to create a file that claims membership in a trusted project and reads its `pkg` API. This is a nonzero expansion of the attack surface.

Three options are presented. Option A prohibits cross-tree membership entirely with error `project/out-of-source-root`. Option B allows it but restricts the cross-tree file to `pub` visibility only (it can use the project's exported API but not its internals). Option C allows it without restriction. The proposal recommends Option A on grounds that the security concern is real and the legitimate use cases are well-served by placing the file inside the source root or using an independent inline project directive.

D16 remains open pending a decision.

Full text: [v5 snapshot](v5.md), revised proposal at [module-system-core.md](../../proposals/module-system-core.md)

---

## Version 6 — Project Membership Clarified; Two New Open Questions

*Revised on 2026-04-05 from inline decisions in [module-system-core.md](../../proposals/module-system-core.md).*

The human reviewed the cross-tree project question (D16) from version 5 and resolved it directionally, then raised two new open questions that had been quietly implied by the existing design but never surfaced explicitly.

### D16 resolved directionally: project membership is directory-only

The earlier design had a `project` directive that let a file name the project it belonged to. The human replaced this entirely with a cleaner rule: a file is a member of a project if and only if it is located under that project's source root, discovered by walking up the directory tree. There is no directive that confers project membership. This eliminates the cross-tree vulnerability — a file outside a project's source root simply cannot claim `pkg` access to its internals.

The `project` directive survives in a reduced form, renamed to something that does not imply membership (name still pending, D16-name). Its sole purpose is to provide dependency configuration to standalone scripts that are not under any `project.json`. It grants access to the `pub` APIs of listed dependencies and nothing more. If a file uses the renamed directive while also being under a `project.json` in its hierarchy, that is an error — project members get their configuration from their project, not from a directive.

The proposal presents six naming candidates: `bootstrap` (user suggestion), `dependencies`, `standalone`, `env`, `runtime`, and `require`. The recommendation leans toward `bootstrap` or `standalone`, but the decision is left open (D16-name).

### New open question D17: script vs. module distinction

The discussion of project membership forced the observation that scripts in a project are not available to consumers via `use` — they occupy the source tree without contributing to the library surface. This raised the question of whether the script/module distinction should be made explicit at the file level, rather than being discovered implicitly through the implicit declare-wrapping rule.

Four options are presented:

- Option A (user suggestion): Convention — scripts use shebang + no `.ish` extension; modules use `.ish` + no shebang. Unenforced.
- Option B: Extension-enforced — only `.ish` files can be imported; non-`.ish` files are always scripts regardless of content.
- Option C: No explicit distinction — the current content-based rule continues; documentation recommends conventions.
- Option D: Explicit `script` or `module` keyword in file frontmatter; the compiler enforces declared intent.

The recommendation notes that Options A and B combine well: the shebang convention is familiar and low-friction; the extension rule gives the compiler a reliable importability signal without content inspection.

### New open question D18: project deployment models and script exposure

The human observed that the design assumes projects are libraries. But projects also distribute executables (CLI tools), and some scripts are for public consumption while others are internal. No mechanism exists for a project to declare "these scripts are distributed as part of my public interface."

Three deployment shapes are named — library, executable, bundle — and four options for script exposure are presented:

- Option A (user suggestion): Conventional `scripts/` directory for source-form public scripts; `bin/` for compiled versions with matching names. `ish publish` handles compilation.
- Option B: Manifest `executables` field listing which files are public executables.
- Option C: File-level `pub script` annotation extending the visibility system.
- Option D: Defer to Proposal B (Package Management).

The recommendation is to defer (Option D) until D17 is resolved, because the right exposure mechanism depends on how scripts are identified in the first place.

Full text: [v6 snapshot](v6.md), revised proposal at [module-system-core.md](../../proposals/module-system-core.md)

---

## Version 7 — Bootstrap Scope, D17 and D18 Resolved, src/ Required

*Revised on 2026-04-05 from inline decisions in [module-system-core.md](../../proposals/module-system-core.md).*

The human made four decisions in this revision, resolving three open questions and expanding the scope of a fourth.

### `bootstrap` named and scoped (D16 finalized)

The standalone dependency directive is named `bootstrap`. Its scope was also expanded beyond what v6 had specified. Version 6 described it as granting access to `pub` APIs of listed packages. The human clarified that it should do everything a `project.json` does for a project member: setting the required ish version, defining dependencies, configuring standards and assurance levels. Publishing metadata is the only field excluded, since that concept applies to distributed projects and not standalone scripts. The directive now parallels the full `project.json` feature set for a single file.

The three accepted forms (filesystem path, URL, inline JSON) are unchanged.

### D17 resolved: `.ish` extension required for importable modules

The script vs. module distinction was settled precisely. The compiler's rule is simple: `use` only locates files with the `.ish` extension. A file without `.ish` is never found by `use`, regardless of its content. This is the only enforced distinction.

The interpreter is deliberately more forgiving: any input it receives — a file path, inline code, a REPL session — is treated as a script. There is no "script mode" declaration.

The unenforced convention was also confirmed: executable scripts intended for direct execution should use a shebang line and omit the `.ish` extension. This is a convention for human readability and OS integration (shebangs make files directly executable), not a compiler rule.

### D18 partially resolved: `scripts/` and `tools/` directories

The question of how projects expose executable scripts to consumers was partially resolved. Two conventional directories are established: `scripts/` for scripts intended for public distribution with the package, and `tools/` for internal scripts (build tools, test runners, dev utilities) that are not distributed. These are build tooling conventions — the tooling knows where to look based on directory — rather than language rules. How scripts in `scripts/` are actually distributed to consumers (installation, PATH, packaging format) is deferred to Package Management.

### New D19: `src/` directory required

A new structural decision was added: projects must have a `src/` directory. All importable `.ish` module files are nested under it. The `src/` prefix is not part of any module path in `use` statements — `src/net/http.ish` is imported as `use net/http`. A project without a `src/` directory is malformed and causes a tool error.

This decision was added as D19 in the decision register and reflected in the new "Project Layout" feature section, which consolidates the layout rules for `src/`, `scripts/`, and `tools/` in one place.

Full text: [v7 snapshot](v7.md), revised proposal at [module-system-core.md](../../proposals/module-system-core.md)

---

## Version 8 — `pkg` Visibility Extended to Project-Contained Scripts

*Revised on 2026-04-05 from a single inline decision in [module-system-core.md](../../proposals/module-system-core.md).*

This revision makes one targeted change: the scope of `pkg` visibility is widened to include script files, not just module files.

### What changed

Prior to this revision, `pkg` visibility was described as visible "anywhere within the project," but the only project members discussed were modules under `src/`. The human clarified that `pkg` visibility should extend to any script file physically located anywhere under the project root — in `scripts/`, in `tools/`, or anywhere else under the directory that contains `project.json`. The location within the project does not matter; the determining criterion is whether the source file path falls under the project root at the time of compilation or execution.

Two things explicitly do not qualify: inline scripts (e.g., `ish -e "..."`) and interactive shells that have merely changed directory into the project root. Both of these lack a file path under the project root. This is a natural consequence of the file-containment rule, not a special case.

### The entry point pattern

The human described the primary motivation for this change: a project may define many modules with entirely `pkg`-level declarations (no `pub` surface at all), then provide a single script physically inside the project as the executable entry point. The script calls into the project's internals through `pkg` access and becomes the runnable artifact that users interact with. External consumers cannot `use` the modules, but they can run the script. This "dark library + script entry point" pattern is now a named first-class use case in the proposal.

The change was propagated into D4 (decision register), the visibility section design, the acceptance tests, and the project layout section, which was updated to note that `pkg` access is granted regardless of which subdirectory the script occupies.

Full text: [v8 snapshot](v8.md), revised proposal at [module-system-core.md](../../proposals/module-system-core.md)

---

## Version 9 — Prototype and Documentation Changes Added

*Revised on 2026-04-05 from inline decision in [module-system-core.md](../../proposals/module-system-core.md).*

The human observed that the proposal was silent on how the prototype gets updated and too brief on documentation changes. This revision adds two major sections to the proposal and raises a split question.

### What the proposal was missing

Through eight revisions, the proposal had developed a complete and correct description of the language behavior — visibility, module mapping, imports, project membership, `declare { }` blocks, and the `bootstrap` directive — without specifying any of the implementation work needed to realize that behavior in the prototype. The documentation table was a list of file names with one-line descriptions, not a guide that an implementing agent could act on.

### Prototype Changes section added

An audit of the existing codebase revealed that the prototype has partial, incorrect scaffolding for modules:

- The `Visibility` enum has variants `Private`, `Public`, and `PubScope(String)` — none of which match the language's `priv`/`pkg`/`pub` vocabulary.
- `Statement::Use` exists but stores `path: Vec<String>` with `::` separators (left over from an earlier design). It has no alias or selective import fields.
- `Statement::ModDecl` exists and represents the old `mod { }` block design that was replaced by `declare { }` blocks in v4. It is silently ignored in the interpreter.
- No `Statement::DeclareBlock` or `Statement::Bootstrap` exists anywhere in the AST.
- The grammar `use_stmt` uses `::` as the path separator instead of `/`.
- The interpreter stubs out `Statement::Use` and `Statement::ModDecl` as no-ops.
- No module loading, project root discovery, access control, cycle detection, or interface file checking exists.

The new Prototype Changes section specifies all required changes crate by crate: `ish-ast` (revised `Visibility`, new `Use` fields, new `DeclareBlock` and `Bootstrap` variants, removal of `ModDecl`), `ish-parser` (grammar updates for visibility, module paths, `declare`, `bootstrap`), `ish-vm` (two new modules — `module_loader.rs` and `access_control.rs` — plus full implementations of `Use`, `DeclareBlock`, and `Bootstrap` evaluation, interface file checking), `ish-codegen` (module resolution in the compiled path), and `ish-shell` (the `ish interface freeze` subcommand and project context discovery at startup). Error codes are tabulated with their production sites.

### Documentation Updates section expanded

The sparse table of file names was replaced with a per-file breakdown specifying exactly what must be added, changed, or removed in each document. This includes the full list of GLOSSARY terms to add or update (with definitions), the complete list of topics that `docs/spec/modules.md` must cover (it requires a full rewrite), the new "Module Directives" section for `docs/spec/syntax.md`, and what the user guide must contain for each workflow (getting started, visibility, entry point pattern, `bootstrap`, interface files, common mistakes).

### Split offered

Counting independent implementation steps yields more than ten: AST revisions, grammar changes, two new VM modules, VM interpreter changes across six statement types, codegen changes, a new shell subcommand, nine new error codes, five architecture and spec doc updates, a new user guide, and a full acceptance test file. A split is presented as a decision point.

The natural boundary is between language representation (changes to `ish-core`, `ish-ast`, `ish-parser`, grammar, GLOSSARY, `docs/spec/syntax.md`, `docs/architecture/ast.md`) and execution and tooling (changes to `ish-vm`, `ish-codegen`, `ish-shell`, error catalog, user guide, remaining architecture docs, and acceptance tests). The representation changes are prerequisites for the execution changes; implementing them first yields a complete, compilable parser and AST with correct node types.

Full text: [v9 snapshot](v9.md), revised proposal at [module-system-core.md](../../proposals/module-system-core.md)

---

## Version 10 — Implementation Decisions and Split into A-1 / A-2

*Revised on 2026-04-05 from inline decisions in [module-system-core.md](../../proposals/module-system-core.md).*

The human reviewed the Prototype Changes and Documentation Updates sections added in version 9 and made six inline decisions. The revision also acted on the split decision offered at the end of v9, splitting the proposal into two child proposals.

### Implementation decisions incorporated

**Bootstrap deferral (D20):** The `bootstrap` directive's full implementation is deferred. Only step 1 — verifying that the caller is not under a `project.json` hierarchy and returning E021 if it is — is implemented now. Config parsing, the application of the resulting configuration to the file's context, and URL fetching are deferred. `ISH_PROXY` is not yet fully specified, and the bootstrap feature as a whole is not yet actionable beyond the containment check.

**VM module decomposition (D21):** The human observed that `interpreter.rs` is already too large, and that adding all of the new module-loading and interface-checking behavior directly to it would make it worse. The decision establishes three new dedicated modules: `module_loader.rs` for filesystem and project-structure concerns, `access_control.rs` for visibility enforcement, and `interface_checker.rs` for `.ishi` consistency checking. The interpreter calls into these; it does not contain the logic itself. This is a structural constraint on the A-2 implementation, not just a refactoring suggestion.

**Analyzer update for declare blocks (D22):** Since `declare { }` blocks now support mutually recursive functions, the code analyzer needs to reason about yielding across a mutual-recursion group. The rules are: if any function in a cycle is yielding (by any criterion), all functions in the cycle become yielding; if no function in the cycle has any yielding criterion other than the cyclic call itself, all are unyielding. This update is part of A-2.

**Defer ish-codegen (D23):** Changes to `ish-codegen` are deferred until the interpreter-based module loading is validated. The v9 codegen section remains as reference material but is not in scope for A-2.

**Error code assignment (D24):** The nine new module/interface error conditions are assigned codes E016–E024, with `ErrorCode` variants following the existing naming convention (`ModuleNotFound`, `ModuleCycle`, `InterfaceSymbolMismatch`, etc.). These replace the string-code-only table from v9.

**AGENTS.md vs CLAUDE.md (D25):** The documentation update that v9 placed under a "`CLAUDE.md` task table" heading is redirected. `CLAUDE.md` and any copilot instructions files are symlinks to `AGENTS.md`; they must never be updated directly. All agent-facing documentation changes — including new task playbook rows — go to `AGENTS.md`. The note about this symlink relationship is also added to `AGENTS.md` as persistent guidance.

### Split into A-1 and A-2

The split proposed at the end of v9 was accepted. The proposal is now split into:

- **[Proposal A-1: Module Core — Language Representation](../../proposals/module-system-core-a1.md):** Changes to `ish-ast` (`Visibility`, `Statement::Use`, `DeclareBlock`, `Bootstrap`, removal of `ModDecl`), `ish-parser` (grammar for visibility, module paths, `declare`, `bootstrap`), GLOSSARY.md, `docs/spec/syntax.md`, and `docs/architecture/ast.md`. No runtime dependencies; can be implemented and merged first.

- **[Proposal A-2: Module Core — Execution and Tooling](../../proposals/module-system-core-a2.md):** The VM (`module_loader.rs`, `access_control.rs`, `interface_checker.rs`, interpreter changes, analyzer updates), `ish-shell` (`ish interface freeze` subcommand, project context discovery), error codes (E016–E024 added to `ish-runtime/src/error.rs` and `docs/errors/INDEX.md`), architecture docs, user guide, acceptance tests, and `AGENTS.md` updates. Depends on A-1.

The parent proposal (`module-system-core.md`) is marked `status: split` and serves as the language design reference. Its feature sections and decision register (D1–D25) remain authoritative. Implementation work proceeds through the child proposals.

Full text: [v10 snapshot](v10.md), split into [module-system-core-a1.md](../../proposals/module-system-core-a1.md) and [module-system-core-a2.md](../../proposals/module-system-core-a2.md)

---

## Referenced by

- [docs/project/proposals/module-system.md](../../proposals/module-system.md)
- [docs/project/history/INDEX.md](../INDEX.md)
