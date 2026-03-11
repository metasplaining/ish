# ish Execution Configurations — Outstanding Issues

Remaining open questions and items that need further work in EXECUTION_CONFIGURATIONS.md.

---

## 1. Thin Shell

- [ ] **Footprint metrics.** What is the actual disk and memory footprint of the thin shell? This should be measured once the prototype is mature enough.
- [ ] **Relationship to streamlined ish.** The thin shell is the natural home for streamlined ish, but the relationship is not formalized. Can encumbered code be used from the thin shell (e.g., if a loaded module was compiled with encumbrance)? Or is the thin shell strictly streamlined?

## 2. Fat Shell — Module System

[MODULES.md](MODULES.md) now describes the module system — package encodings (annotated AST, static/dynamic object code), distribution strategy, and the dynamic linking interface. Some questions here are addressed there; remaining open items are tracked in [MODULES_TODO.md](MODULES_TODO.md).

- [x] **Module format.** [MODULES.md](MODULES.md) specifies three encodings: annotated AST, statically linked object code, and dynamically linked object code.
- [x] **Module distribution.** [MODULES.md](MODULES.md) describes a phased strategy: git-based deps → OCI/ORAS → dedicated registry.
- [ ] **Module generation process.** "Optimized and compiled during the module generation process" — what does this process involve? Is it the same as the compiler configuration (§3), or a separate offline step?
- [ ] **Module compatibility.** How is compatibility ensured between modules compiled at different times or with different versions of ish?

## 3. Compiler Configuration

- [ ] **Which modules constitute the compiler?** The document says "the code analyzer and the compiler" can be loaded as modules — are there others? What is the minimal set of modules needed for each compiler output (local code, module, executable)?
- [ ] **Compilation target.** Does the compiler produce native machine code, Rust source (which is then compiled by `rustc`), LLVM IR, or something else? How does this relate to the Rust generator described in the README?
- [ ] **Local code loading.** When compiled code is "loaded into the current process," how does it interoperate with interpreted code already running in the thin shell? Can compiled and interpreted code call each other?
- [ ] **Incremental compilation.** Is there support for incremental or cached compilation?

## 4. Executable Output

- [ ] **Executable contents.** The executable excludes the thin shell modules (shell, parser, VM, module loader). What does it include? Just the compiled user code and a minimal runtime? Or also the standard library, garbage collector, etc.?
- [ ] **Runtime requirements.** Does the executable have any runtime dependencies (e.g., a runtime library, libc, an allocator)?
- [ ] **Cross-compilation.** Can ish produce executables for platforms other than the host?

## 5. Relationship to Encumbrance

- [ ] **Configuration selection vs. encumbrance level.** The four execution configurations are presented as distinct modes, and the encumbrance system is described separately. How do they interact? For example:
  - Does selecting "executable" output imply a minimum encumbrance level?
  - Can a module compiled with encumbrance be imported into a streamlined thin shell?
  - If different parts of a project have different encumbrance levels, which execution configuration applies?

## 6. Transitions Between Configurations

- [ ] **Fat shell → compiler transition.** Loading the code analyzer and compiler modules is described as a transition from fat shell to compiler configuration. Is this seamless (just an import), or does it change the shell's behavior?
- [ ] **Interactive compilation.** Can the compiler configuration be used interactively (e.g., compile-and-run from the shell), or is it only for batch compilation?
