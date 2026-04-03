---
title: ish Execution Configurations
category: spec
audience: [all]
status: draft
last-verified: 2026-03-14
depends-on: [docs/spec/modules.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md]
---

# ish Execution Configurations

ish has several execution configurations, serving different purposes. Each represents a different tradeoff between footprint, startup speed, and capability.

---

## 1. Thin Shell

When ish is started without any configuration or arguments, it runs as a **thin shell**. This is the smallest and most lightweight configuration.

- Accepts command-line input, interprets it, and processes it immediately.
- The shell, parser, virtual machine, and module loader are all loaded, but these are small, leaving ish with a minimal disk and memory footprint.
- In shell mode, bare-word lines are parsed as command invocations; lines starting with recognized keywords (`let`, `fn`, `if`, etc.) are parsed as language statements. See [docs/spec/syntax.md § Shell Mode](syntax.md#shell-mode) for the full shell syntax.
- Function declaration is supported, but lightweight — no code analysis or generation is performed. No semantic checking is done when functions are declared.
- This is the entry point for low-assurance ish (see [assurance-ledger.md](assurance-ledger.md)).

## 2. Fat Shell

When `use` statements are invoked from the thin shell, ish downloads modules and loads them into memory.

- Imported modules increase ish's memory footprint.
- Because modules have been optimized and compiled during the module generation process, once loaded they execute very quickly.
- The fat shell bridges the gap between interactive use and compiled performance: the shell itself is interpreted, but imported code runs at compiled speed.

## 3. Compiler

Some of the modules that can be loaded are the **code analyzer** and the **compiler**. Once these have been loaded, ish can optimize and compile code, producing three kinds of output:

1. **Local code** — loaded into the current process for immediate use.
2. **Packages** — saved for later import in a different process.
3. **Executables** — saved as standalone programs.

This is the configuration used to build high-assurance ish (see [assurance-ledger.md](assurance-ledger.md)).

## 4. Executable

When ish generates an executable, it is compiled down to just the modules the program needs. Typically, this means the thin shell modules — shell, parser, virtual machine, and module loader — are excluded. The result is a standalone executable with no ish interpreter overhead.

---

## Error Handling Across Configurations

All execution configurations support ish's error handling mechanisms (throw/try/catch/finally, with blocks, defer). The **return handler** mechanism — which intercepts function returns to manage error propagation and stack trace construction — is an implementation detail hidden from the user. It operates as a separate execution concern. See [docs/user-guide/error-handling.md](../user-guide/error-handling.md) for user-facing documentation.

---

## Concurrency Runtime

All execution configurations integrate with a Tokio async runtime to support cooperative multitasking. The runtime model varies by configuration.

### Thin Shell and Fat Shell (Interactive Mode)

In interactive mode, the shell uses a **two-thread architecture**:

- **Shell thread:** Runs Reedline (blocking `read_line()`), collects input, parses it via the ish parser, and submits the resulting `Program` AST to the main thread via a channel. The shell thread is responsible only for prompts and command line input.
- **Main thread:** Runs the Tokio runtime with a `LocalSet`. The VM lives here. All `Value` objects, the `Environment`, and GC-managed state are confined to this thread. Spawned futures survive between shell submissions.

All program output (expression results, `println`, errors, background task output) goes through stdout/stderr via Reedline's `ExternalPrinter` in interactive mode. The main thread sends only a completion signal (not display content) back to the shell thread.

Shell command execution uses `tokio::process::Command` instead of `std::process::Command` to avoid blocking the `LocalSet` thread.

### Non-Interactive Mode (File/Inline Execution)

When ish is started with a file or inline code, there is no shell thread. The main thread parses the input, creates a Tokio runtime and `LocalSet`, runs the program to completion, and exits. Output goes directly to OS stdout/stderr.

### Compiler and Executable

The compiler configuration operates within the same Tokio runtime model. Generated executables include the Tokio runtime as part of their standard runtime dependencies.

See [docs/spec/concurrency.md](concurrency.md) for the full concurrency model.

---

## Open Questions

Open questions for execution configurations. See also [docs/project/open-questions.md](../project/open-questions.md#execution-configurations) for a consolidated view.

### Thin Shell

- [ ] **Footprint metrics.** What is the actual disk and memory footprint? Measure once the prototype is mature.
- [ ] **Relationship to low-assurance ish.** Can high-assurance code be used from the thin shell? Or is the thin shell strictly low-assurance?

### Fat Shell — Module System

- [ ] **Module generation process.** What does optimization and compilation involve? Same as the compiler configuration, or a separate offline step?
- [ ] **Module compatibility.** How is compatibility ensured between modules compiled at different times or with different versions of ish?

### Compiler Configuration

- [ ] **Which modules constitute the compiler?** Minimal set needed for each compiler output?
- [ ] **Compilation target.** Native machine code, Rust source (compiled by `rustc`), LLVM IR, or something else?
- [ ] **Local code loading.** How does compiled code interoperate with interpreted code in the thin shell?
- [ ] **Incremental compilation.** Is there support for incremental or cached compilation?

### Executable Output

- [ ] **Executable contents.** What is included beyond compiled user code? Standard library? Garbage collector?
- [ ] **Runtime requirements.** Runtime dependencies (runtime library, libc, allocator)?
- [ ] **Cross-compilation.** Can ish produce executables for platforms other than the host?

### Relationship to Assurance Levels

- [ ] **Configuration selection vs. assurance level.** Does selecting "executable" imply a minimum assurance level? Can a compiled module be imported into a low-assurance thin shell? What happens with mixed assurance levels?

### Transitions Between Configurations

- [ ] **Fat shell → compiler transition.** Seamless import, or does it change the shell's behavior?
- [ ] **Interactive compilation.** Can the compiler configuration be used interactively?

---

## Referenced by

- [docs/spec/INDEX.md](INDEX.md)
- [docs/spec/concurrency.md](concurrency.md)
- [docs/spec/modules.md](modules.md)
- [GLOSSARY.md](../../GLOSSARY.md)
