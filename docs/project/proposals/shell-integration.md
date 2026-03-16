---
title: "Proposal: Shell Integration"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-15
depends-on: [docs/project/rfp/shell-integration.md, docs/spec/execution.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md, docs/architecture/shell.md, docs/project/proposals/language-syntax.md, GLOSSARY.md]
---

# Proposal: Shell Integration

*Generated from [shell-integration.md](../rfp/shell-integration.md) on 2026-03-15.*

---

## Questions and Answers

### Q: Are there alternative Rust libraries available to help with shell implementation? What are the pros and cons of each?

The old prototype used `redox_liner`, a fork of the `liner` crate maintained by the Redox OS project. There are four serious contenders in the Rust readline/line-editor space:

#### 1. Reedline

- **Version:** 0.46.0 (actively developed)
- **Downloads:** ~2M all-time
- **Maintainer:** Nushell team
- **License:** MIT

| Pros | Cons |
|------|------|
| Powers Nushell — proven in a real shell | Younger than rustyline; API still evolving |
| Syntax highlighting, completions, hinter, validator — all trait-based | Larger dependency footprint (crossterm, nu-ansi-term) |
| Fish-style history autosuggestions built in | 18K SLoC — more code to audit |
| Emacs and vi keybinding modes | Not all vi commands fully implemented |
| Clipboard integration | Requires crossterm (no termion backend) |
| Multiline input with validation | Less community documentation than rustyline |
| SQLite-backed history option | |
| Cross-platform (Unix + Windows) | |
| Custom keybinding support | |

#### 2. Rustyline

- **Version:** 17.0.2 (mature, actively maintained)
- **Downloads:** ~29M all-time
- **Maintainer:** Katsu Kawakami
- **License:** MIT

| Pros | Cons |
|------|------|
| Most widely used Rust readline library | No built-in syntax highlighting trait |
| Mature and battle-tested (46 versions) | No fish-style autosuggestion hints |
| Emacs and vi modes with full keybinding set | Completion menu is basic (cycling, not graphical) |
| Unicode support | Multiline requires implementing Validator trait |
| History search (Ctrl-R) | Less shell-oriented than reedline |
| Kill ring, word commands | |
| Cross-platform (Unix + Windows) | |
| 11K SLoC — lighter than reedline | |
| Extensive documentation | |

#### 3. Liner (and redox_liner)

- **Version:** 0.4.4 (stale — last published 8+ years ago)
- **Downloads:** ~74K all-time
- **License:** MIT

| Pros | Cons |
|------|------|
| Simple, minimal API | Unmaintained — 8 years without a release |
| Used in the old ish prototype | Unix-only (no Windows support) |
| Small codebase | No incremental search |
| | Word completion only (not arbitrary) |
| | No syntax highlighting, no hinting |
| | No validators or multiline support |
| | ANSI-only (no terminfo) |

#### 4. Linefeed

- **Version:** 0.6.0 (stale — last published 7 years ago)
- **Downloads:** ~564K all-time
- **License:** MIT/Apache-2.0

| Pros | Cons |
|------|------|
| GNU Readline paradigm — reads inputrc | Unmaintained — 7 years without a release |
| Configurable keybindings | Emacs mode only (no vi) |
| Cross-platform | Smaller community than rustyline/reedline |
| | No syntax highlighting |
| | No fish-style hints |

#### Recommendation: Reedline

**Reedline** is the strongest choice for ish. The rationale:

1. **Shell-first design.** Reedline was built for Nushell — an actual shell. It provides syntax highlighting, tab completion with graphical menus, multiline validation, and fish-style history hints, all as composable traits. These are exactly the features a shell needs.

2. **Active development.** Reedline is actively maintained by the Nushell team with regular releases (48 versions, latest 15 days ago). Rustyline is also active, but reedline's feature set is more aligned with shell use cases.

3. **Trait-based architecture.** The `Highlighter`, `Completer`, `Hinter`, and `Validator` traits allow ish to plug in language-aware behavior incrementally. Start with defaults, then replace them with ish-specific implementations as the language matures:
   - `Highlighter` → ish syntax highlighting
   - `Completer` → ish variable/function/command completion
   - `Validator` → multiline expression detection (e.g., open braces)
   - `Hinter` → history-based suggestions

4. **Leverage existing work.** The RFP says "take advantage of as much functionality provided by the shell helper library as possible." Reedline provides significantly more out-of-the-box functionality than rustyline — especially syntax highlighting, the hint system, and the completion menu — reducing the amount of custom code ish needs to write.

5. **Nushell precedent.** Nushell is a language-shell hybrid similar in spirit to ish. Their line editor needs are closely aligned with ours. Reedline has been stress-tested against those needs.

Rustyline would be the second choice — mature and lightweight, but ish would need to build many shell features (highlighting, hints, graphical completion menus) from scratch. Liner/linefeed are unmaintained and should not be considered.

---

## Feature: Shell REPL Integration

This feature covers transforming the current `ish-shell` verification demo binary into an interactive REPL (read-eval-print loop) using reedline.

### Issues to Watch Out For

- **Dual-mode parsing.** The language syntax proposal establishes shell mode vs. programming mode. The REPL needs a mode-detection heuristic: does this line start with a keyword (`let`, `fn`, `if`, etc.) or is it a bare-word command invocation? The parser does not yet exist, so the REPL will need a temporary heuristic that can be replaced later.
- **Multiline input.** Expressions with unclosed braces `{`, brackets `[`, or parentheses `(` need to continue on the next line. Reedline's `Validator` trait handles this, but ish needs to implement a bracket-counting validator.
- **Existing verification demos.** The current `ish-shell` main.rs runs 6 verification demos. These should be preserved (perhaps behind a `--verify` flag) rather than replaced.
- **History isolation.** The ish history file should be stored in a sensible location (e.g., `~/.ish_history`) and not conflict with other shells.
- **Signal handling.** Ctrl-C should cancel the current input (not exit), Ctrl-D on an empty line should exit. Reedline handles this via `Signal::CtrlC` and `Signal::CtrlD`.

### Critical Analysis

**Alternative A: Reedline-based REPL (recommended)**
- Pros: Rich out-of-the-box experience (highlighting, hints, completions, multiline). Trait-based design allows incremental enhancement. Proven in Nushell.
- Cons: Larger dependency. API may change (pre-1.0). Ties ish to reedline's design patterns.

**Alternative B: Rustyline-based REPL**
- Pros: Mature, stable, widely used. Smaller dependency. Better documentation.
- Cons: Missing built-in syntax highlighting, fish-style hints, and graphical completion menus. More custom code needed for shell UX.

**Alternative C: Custom line editor on crossterm**
- Pros: Full control. No readline library dependency. Exactly the features ish needs and nothing more.
- Cons: Enormous implementation effort. Reinventing the wheel. Distraction from the language design focus.

### Proposed Implementation

**Phase 1: Basic REPL (implement now)**

1. Add `reedline` dependency to `ish-shell/Cargo.toml`.
2. Restructure `main.rs`:
   - `--verify` flag runs the existing 6 verification demos.
   - Default (no args) enters the interactive REPL.
3. REPL loop:
   - Create `Reedline::create()` with file-backed history (`~/.ish_history`).
   - Custom prompt showing `ish> ` (or `...> ` for continuation lines).
   - On `Signal::Success(line)`: dispatch to the VM via a new `process_line()` function.
   - On `Signal::CtrlC`: clear current input.
   - On `Signal::CtrlD`: exit.
4. `process_line()` initially:
   - Parse the line as an ish expression/statement (once the parser exists).
   - For now, as a placeholder, treat input as ish AST builder calls or a simple expression evaluator.
5. Print results: display the return value of each evaluated expression (suppressing `null` returns).

**Phase 2: Enhanced UX (implement now with stubs)**

6. Implement `Validator` trait — count open braces/brackets/parens for multiline.
7. Implement `Highlighter` trait — stub that returns unhighlighted text (placeholder for later).
8. Implement `Completer` trait — stub returning empty completions (placeholder for later).
9. Implement `Hinter` trait — delegate to reedline's `DefaultHinter` for history-based suggestions.

**Phase 3: Language-aware features (defer)**

10. Syntax-aware highlighting (requires parser).
11. Context-aware completion (variables in scope, builtins, filesystem paths, commands on PATH).
12. Error display formatting (structured error messages with source location).

**Files affected:**
- `proto/ish-shell/Cargo.toml` — add `reedline` dependency
- `proto/ish-shell/src/main.rs` — restructure into REPL + verify mode
- `proto/ish-shell/src/repl.rs` — new file: REPL loop, prompt, signal handling
- `proto/ish-shell/src/validate.rs` — new file: `Validator` implementation
- `proto/ish-shell/src/highlight.rs` — new file: stub `Highlighter`
- `proto/ish-shell/src/complete.rs` — new file: stub `Completer`
- `docs/architecture/shell.md` — update to document REPL architecture

### Decisions

**Decision:** Reedline vs. rustyline vs. custom line editor — which library should ish use?
--> Reedline

**Decision:** Should the existing verification demos be behind `--verify` or in a separate binary?
--> Delete them entirely.

**Decision:** Should the REPL support a `--no-history` flag for scripting/testing contexts?
--> Yes.


---

## Feature: Shell Feature Assessment

This section evaluates standard shell features by complexity and recommends whether each should be implemented now or deferred.

### Issues to Watch Out For

- **Parser dependency.** Most shell features depend on having a parser. The parser is listed as "Future" on the roadmap. The REPL can handle a subset of features without a full parser (e.g., external command execution, history), but features like piping and globbing need at least a tokenizer.
- **Security surface.** Every feature that executes external commands or accesses the filesystem expands the security surface. The assurance ledger is not yet implemented, so each feature should be evaluated for what security controls it will eventually need.
- **Scope creep.** The RFP explicitly says "shell features that add significant complexity should be deferred." The bar should be high for inclusion.

### Critical Analysis

| Feature | Complexity | Recommendation | Rationale |
|---------|-----------|----------------|-----------|
| **External command execution** | Low | **Implement** | Core purpose of a shell. `std::process::Command` handles this. The old prototype already had this. |
| **Command history** (file-backed) | Low | **Implement** | Reedline provides this out of the box. Minimal code needed. |
| **Line editing** (emacs/vi) | Low | **Implement** | Provided by reedline. Zero ish code needed. |
| **Fish-style history hints** | Low | **Implement** | Reedline's `DefaultHinter`. One line of setup. |
| **Multiline input** (open brace continuation) | Low | **Implement** | Requires a simple bracket-counter `Validator`. Essential for entering blocks. |
| **Prompt customization** | Low | **Implement** | Custom prompt struct implementing reedline's `Prompt` trait. Needed for continuation lines. |
| **Variable interpolation in commands** | Medium | **Implement** | Essential for `let dir = "/tmp"; ls $dir`. Requires string interpolation in shell mode. |
| **Environment variable access** (`$HOME`, `$PATH`) | Low | **Implement** | `std::env::var()`. Already needed for string interpolation. |
| **Exit / quit command** | Low | **Implement** | Trivial. |
| **`cd` (change directory)** | Low | **Implement** | Shell builtin, not external command. `std::env::set_current_dir()`. |
| **Syntax highlighting** | Medium | **Defer** | Requires a parser or at least a lexer. Stub the trait now, implement when the parser exists. |
| **Tab completion** (commands, paths, variables) | Medium | **Defer** | Path completion is straightforward but command/variable completion requires scope awareness. Stub now, implement incrementally. |
| **Pipes** (`cmd1 \| cmd2`) | Medium–High | **Defer** | Requires connecting stdout/stdin between processes. Needs a tokenizer to parse the pipe operator. The `|` character also has meaning in ish's type syntax (unions). Interaction needs careful design. |
| **Redirection** (`>`, `>>`, `<`, `2>`) | Medium | **Defer** | Requires tokenizer, file handle management. The `>` character is also the force-command prefix in ish shell mode. Disambiguation needed. |
| **Globbing** (`*.txt`, `src/**/*.rs`) | Medium | **Defer** | Requires a glob expansion pass before command execution. Library support exists (`glob` crate), but integrating it into the execution pipeline adds complexity. |
| **Background jobs** (`cmd &`, `jobs`, `fg`, `bg`) | High | **Defer** | Requires job control, process groups, signal handling. Significant complexity. Not needed for the language design phase. |
| **Command substitution** (`$(cmd)`) | High | **Defer** | Requires subshell execution and output capture. The RFP explicitly defers subshells until streams are implemented. |
| **Piping into ish functions** | High | **Defer** | Requires streams (the RFP defers this). |
| **Aliases** | Medium | **Defer** | Useful but not essential. Can be simulated with ish functions once those work in shell mode. |
| **Shell scripting** (`.ish` file execution) | Medium | **Defer** | Requires the parser. The REPL is interactive-first for now. |
| **Signal handling** (beyond Ctrl-C/D) | Medium | **Defer** | SIGTSTP (Ctrl-Z), SIGHUP, etc. Complex, not needed for language design phase. |
| **Startup files** (`~/.ishrc`) | Low–Medium | **Defer** | Requires file execution, which requires the parser. |

### Proposed Implementation

Implement the features marked "Implement" above as part of the REPL integration (Feature 1). The deferred features should be documented as future work in the roadmap and architecture docs.

For external command execution specifically:

1. If the input line does not start with an ish keyword, treat it as a command invocation.
2. Split the line on whitespace (simple tokenization for now).
3. The first token is the command name. Look it up on `$PATH` via `std::process::Command`.
4. Remaining tokens are arguments.
5. Run the command, inheriting stdin/stdout/stderr.
6. Report the exit status.

Special builtins (`cd`, `exit`) are intercepted before external command dispatch.

### Decisions

**Decision:** Should variable interpolation in shell-mode arguments (`ls $dir`) be implemented immediately, or deferred until the string interpolation system from the string syntax proposal is fully implemented?
--> Immediately.

**Decision:** For pipe syntax (`|`), should ish use `|` (conflicting with union types) or an alternative syntax (e.g., `|>`)? Or should this decision be deferred?
--> See parser.

--> Implement all of the features marked implement, also syntax highlighting, pipes, redirection, globbing, and shell scripting.

--> Document the deferred features in the roadmap.

---

## Feature: Security Standards for Shell Execution

This feature proposes assurance ledger standards that govern what the shell is allowed to execute. Per the RFP, these standards are deferred (the assurance ledger is not yet implemented), but their interfaces and intended behavior should be documented now.

### Issues to Watch Out For

- **Arbitrary code execution.** A shell that can run any command is inherently dangerous. The assurance ledger's purpose is to gate dangerous operations behind explicit authorization. Without it, the shell is operating in a fully permissive mode — which is acceptable for the prototype, but must be documented as such.
- **Trojan programs.** A malicious ish script could invoke destructive external commands (`rm -rf /`). Standards need to control what commands can be executed and with what arguments.
- **Environment leakage.** Shell commands inherit the user's environment. Sensitive environment variables (API keys, tokens) could leak to child processes.
- **Path traversal.** Commands like `cd ../../..` or file arguments with `../` could access files outside the intended scope.
- **Network access.** External commands may make network requests. Standards may need to gate network-capable commands.
- **Interaction with compiled mode.** Compiled ish programs (`.so` files) loaded via `CompilationDriver` already have full system access. The security standards for the shell need to be consistent with the standards for compiled execution.

### Critical Analysis

Three proposed standards, from least to most restrictive:

#### Standard: `shell.unrestricted`

Allows all shell operations without restriction. This is the default for the prototype and for low-assurance ish usage.

- **When active:** All external commands can be executed. All environment variables are accessible. No path restrictions. No network restrictions.
- **Purpose:** Development, scripting, trusted environments.

| Pros | Cons |
|------|------|
| Zero friction — behaves like a normal shell | No protection against malicious or accidental damage |
| Appropriate for low-assurance ish | Cannot be used in shared or production environments |
| Simple to implement (it's the default) | |

#### Standard: `shell.sandboxed`

Restricts shell operations to a declared set of allowed commands and paths.

- **Configuration interface:**
  ```ish
  @[standard(shell.sandboxed)]
  @[shell.allow_commands("ls", "cat", "grep", "find", "echo")]
  @[shell.allow_paths("/home/user/project", "/tmp")]
  @[shell.deny_env("AWS_SECRET_KEY", "DATABASE_URL")]
  ```
- **When active:** Only whitelisted commands can be executed. File arguments are validated against allowed paths. Specified environment variables are hidden from child processes.
- **Purpose:** Semi-trusted environments, CI/CD, shared machines.

| Pros | Cons |
|------|------|
| Granular control over what the shell can do | Maintaining allowlists is tedious |
| Environment variable protection | May break workflows that need unlisted commands |
| Path restriction prevents traversal attacks | Allowlist approach is brittle — new commands require updates |

#### Standard: `shell.denied`

Completely disables external command execution from ish code.

- **When active:** Any attempt to invoke an external command throws a `ShellDenied` error. Only ish builtins and ish-native functions are available. Environment variables are read-only and filtered.
- **Purpose:** High-assurance contexts, library modules, untrusted code execution.

| Pros | Cons |
|------|------|
| Maximum security — no external process execution | Cannot function as a shell in this mode |
| Appropriate for pure-ish library code | Too restrictive for interactive use |
| Simple to implement — just block the command dispatch | |

#### Controversial Question: Granularity of `shell.sandboxed`

The sandboxed standard could be implemented at different granularities:

**Option 1: Command-level allowlist (as described above)**
- Pros: Simple, easy to understand, easy to audit.
- Cons: Coarse — `ls` is allowed everywhere or nowhere. Cannot restrict `rm` to only certain directories at the command level.

**Option 2: Command + argument pattern matching**
```ish
@[shell.allow("ls", "-la", "/home/user/**")]
@[shell.allow("rm", "/tmp/**")]
@[shell.deny("rm", "-rf", "/**")]
```
- Pros: Fine-grained control. Can allow `rm` in `/tmp` but deny it elsewhere.
- Cons: Complex pattern language. Hard to get right. Patterns may not cover all argument orderings.

**Option 3: Capability-based (acquire a handle to perform operations)**
```ish
let fs = acquire(capability.filesystem, path: "/home/user/project")
fs.exec("ls", "-la")  // OK — scoped to project
exec("ls", "-la")     // ERROR — no ambient authority
```
- Pros: Cleanest security model. No ambient authority. Auditable.
- Cons: Highest complexity. Requires rethinking how the shell dispatch works. Significant departure from traditional shell UX.

**Recommendation:** Start with **Option 1** (command-level allowlist) for the initial implementation. It covers the most common use cases with the least complexity. Document Option 3 (capability-based) as a future direction in the roadmap — it aligns with ish's assurance philosophy but requires more language infrastructure (e.g., the module system, the assurance ledger) to implement properly.

### Proposed Implementation

Since these standards are deferred, the implementation for now is documentation-only:

1. **Document the three standards** (`shell.unrestricted`, `shell.sandboxed`, `shell.denied`) in the assurance ledger spec as shell-related standards.
2. **Document the entry syntax** for shell-related ledger entries (`@[shell.allow_commands(...)]`, `@[shell.allow_paths(...)]`, `@[shell.deny_env(...)]`).
3. **Add a `ShellDenied` error** to the error catalog as a placeholder.
4. **Mark the shell dispatch function** in the REPL implementation with a comment indicating where the standard check will be inserted:
   ```rust
   // TODO(standard:shell): Check active shell standard before dispatching.
   // When the assurance ledger is implemented, this is where
   // shell.sandboxed and shell.denied will gate execution.
   ```
5. **Add roadmap items** for implementing each standard.

**Files affected:**
- `docs/spec/assurance-ledger.md` — add shell standards section
- `docs/errors/INDEX.md` — add `ShellDenied` error
- `docs/project/roadmap.md` — add shell security standards milestones
- `proto/ish-shell/src/repl.rs` — placeholder comment at command dispatch point

### Decisions

**Decision:** Should `shell.unrestricted` be the default standard, or should the default depend on the execution configuration (e.g., thin shell defaults to unrestricted, fat shell defaults to sandboxed)?
--> `shell.unrestricted` should be the default.

**Decision:** Should `shell.sandboxed` use a command-level allowlist (Option 1), command+argument patterns (Option 2), or capabilities (Option 3)?
--> Let's skip sandboxed for now.  The most common use case is that an unrestricted shell uses denied packages.

**Decision:** Should environment variable filtering in `shell.sandboxed` use an allowlist (only listed variables are visible) or a denylist (only listed variables are hidden)?
--> No sandboxed for now.

---

## Feature: Subshell (Deferred)

Per the RFP, the subshell is explicitly deferred until streams are implemented. This section documents the intended interface for future reference.

### Issues to Watch Out For

- **Stream dependency.** Command substitution (`$(cmd)`) captures the stdout of a subprocess as a string or stream. This requires the stream abstraction to be in place.
- **Nesting.** Subshells can nest: `echo $(cat $(find . -name "*.txt"))`. The evaluator needs to handle recursive subshell expansion.
- **Error propagation.** If the inner command fails, how does the error surface? Does the outer command receive an empty string, or does the error propagate?

### Proposed Interface (for documentation only)

```ish
// Command substitution — captures stdout as a string
let files = $(ls -la)

// Nested substitution
let count = $(wc -l $(find . -name "*.txt"))

// In interpolated strings
let msg = "There are {$(wc -l data.txt)} lines"
```

The subshell should respect the active shell standard — if `shell.denied` is active, `$(...)` should also be denied.

### Decisions

**Decision:** Should `$(cmd)` return a string (trimmed stdout) or a structured value (exit code + stdout + stderr)?
--> TBD

---

## Documentation Updates

The following documentation files will need updates when this proposal is implemented:

| File | Update |
|------|--------|
| `docs/architecture/shell.md` | Rewrite: document REPL architecture, reedline integration, command dispatch |
| `docs/spec/execution.md` | Add detail to thin-shell configuration about REPL behavior |
| `docs/spec/assurance-ledger.md` | Add shell security standards (`shell.unrestricted`, `shell.sandboxed`, `shell.denied`) |
| `docs/spec/syntax.md` | May need shell-mode parsing details as they are implemented |
| `docs/errors/INDEX.md` | Add `ShellDenied` error |
| `docs/project/roadmap.md` | Add shell integration milestones and deferred items |
| `docs/user-guide/getting-started.md` | Update to reflect how to launch and use the ish shell |
| `GLOSSARY.md` | May need entries for `shell standard`, `command dispatch`, `subshell` if not already present |
| `docs/project/proposals/language-syntax.md` | Cross-reference this proposal for shell mode implementation details |

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-shell-integration.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
