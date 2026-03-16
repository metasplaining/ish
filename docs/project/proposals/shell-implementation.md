---
title: "Proposal: Shell Implementation"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-15
depends-on: [docs/project/rfp/shell-implementation.md, docs/project/proposals/shell-integration.md, docs/spec/execution.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md, docs/architecture/shell.md, docs/project/proposals/language-syntax.md, docs/project/proposals/string-syntax.md, GLOSSARY.md]
---

# Proposal: Shell Implementation

*Generated from [shell-implementation.md](../rfp/shell-implementation.md) on 2026-03-15.*

*Follow-on to [shell-integration.md](shell-integration.md), incorporating decisions made during review.*

---

## Correction: Parser Status

The original shell integration proposal repeatedly stated that the parser does not yet exist and deferred several features on that basis (syntax highlighting, pipes, redirection, globbing, shell scripting). This was incorrect.

The `ish-parser` crate is fully implemented:

- **Grammar:** 650+ lines of PEG rules in [ish.pest](../../proto/ish-parser/src/ish.pest), covering all language features and shell integration.
- **AST builder:** 1500+ lines in [ast_builder.rs](../../proto/ish-parser/src/ast_builder.rs), transforming parse trees to ish AST nodes.
- **Public API:** `ish_parser::parse(input: &str) -> Result<Program, Vec<ParseError>>`.
- **Tests:** 150+ tests across 9 test files (phases 1–8 plus string syntax).
- **Shell support in grammar:** `ShellCommand`, `ShellPipeline`, `ShellArg` (bare, quoted, glob, env var, command sub), `Redirection` (stdout write/append, stderr, combined), background execution, force-command prefix.

However, the parser is **isolated** — no consumer crate depends on it. The `ish-shell` binary builds ASTs via the builder API. The VM stubs out `ShellCommand` and `CommandSubstitution` with error messages.

This proposal accounts for the parser being available, which upgrades several features from "deferred" to "implement now."

---

## Feature: Parser Integration

Wire the parser into the shell and VM so that ish source text can be parsed et executed end-to-end.

### Issues to Watch Out For

- **Dependency direction.** The parser depends on `ish-ast`. Both `ish-vm` and `ish-shell` depend on `ish-ast`. Adding `ish-parser` as a dependency of `ish-shell` is straightforward. The VM itself does not need to depend on the parser — it receives ASTs.
- **Error display.** `ParseError` has `start`, `end`, and `message` fields. The REPL needs to display these errors clearly, ideally with a caret pointing to the error location in the input.
- **Incremental parsing.** The current parser takes a full program string. For REPL use, single-line or multiline snippets need to parse correctly. The parser should handle partial programs (e.g., a single expression or statement) without requiring a full program wrapper.
- **Multiline coordination.** The reedline `Validator` determines whether input is complete. If the parser could indicate "incomplete input" vs. "parse error," the REPL could decide whether to prompt for continuation or display an error. Currently, `ParseError` does not distinguish these cases.

### Critical Analysis

**Alternative A: Parser in ish-shell only (recommended)**
- Pros: Clean separation. The VM stays parser-agnostic — it interprets ASTs from any source (parser, builder API, compiled). The shell is the only crate that needs to parse text.
- Cons: Any future crate that needs parsing (e.g., a language server) would need its own dependency on `ish-parser`.

**Alternative B: Parser in ish-vm**
- Pros: The VM could offer a `run_source(text)` convenience method.
- Cons: Couples the VM to the parser needlessly. Violates the current architecture's clean AST-as-interface boundary.

### Proposed Implementation

1. Add `ish-parser` to `ish-shell/Cargo.toml`:
   ```toml
   ish-parser = { path = "../ish-parser" }
   ```

2. In the REPL loop, call `ish_parser::parse(&input)` on each submitted line/block.

3. On success, pass the resulting `Program` to `vm.run(&program)`.

4. On parse error, format and display the error with location context:
   ```
   ish> let x = 
   error: unexpected end of input
     let x = 
             ^
   ```

5. For multiline detection, implement a lightweight bracket/brace/paren counter in the `Validator` trait — do not invoke the full parser on every keystroke. The validator determines if the input *might* be incomplete; the parser gives the authoritative result on submission.

6. Update the roadmap to move the parser from "Future" to "Completed."

**Files affected:**
- `proto/ish-shell/Cargo.toml` — add `ish-parser` dependency
- `proto/ish-shell/src/main.rs` — import and use `ish_parser::parse`
- `docs/project/roadmap.md` — move parser to completed

### Decisions

**Decision:** Should the parser be enhanced to distinguish "incomplete input" from "parse error" to improve the REPL multiline experience?
--> Yes.  The old prototype dealt with this problem by parsing "chunks", where a chunk might be a whole file, and it might be a single line from stdin.  Also, note that the shell never needs to process anything smaller than a whole line.  A line is not submitted to the VM for processing until the user presses return, even when that line contains multiple statements separated by a semicolon.

---

## Feature: REPL with Reedline

Replace the current verification demo binary with a proper interactive REPL using reedline as the line editor.

### Issues to Watch Out For

- **Removing verification demos.** The decision is to delete them entirely, not preserve them. This means the 6 end-to-end verifications that currently validate the prototype will be lost. These should be migrated to integration tests (`cargo test`) before deletion so the project doesn't lose test coverage.
- **Startup time.** Loading reedline and stdlib should be fast. If stdlib loading is slow, consider lazy-loading.
- **History file location.** `~/.ish_history` is the natural location. On systems without `$HOME`, fall back to a temporary location or disable history.

### Proposed Implementation

#### Step 1: Migrate verification demos to tests

Before deleting the demos from `main.rs`, create an integration test file that exercises the same 6 verifications:

- Create `proto/ish-shell/tests/verification.rs` (or `proto/ish-vm/tests/integration.rs`).
- Each verification becomes a `#[test]` function.
- Run `cargo test --workspace` to confirm all pass.

#### Step 2: Replace main.rs with REPL entry point

```rust
// proto/ish-shell/src/main.rs (sketch)

mod repl;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // File execution: ish script.ish
        let filename = &args[1];
        repl::run_file(filename);
    } else {
        // Interactive REPL
        repl::run_interactive();
    }
}
```

Add `--no-history` handling via argument parsing (keep it simple — no clap dependency needed for two flags).

#### Step 3: Implement REPL module

Create `proto/ish-shell/src/repl.rs`:

1. **`run_interactive()`:**
   - Initialize `IshVm`, load stdlib.
   - Create `Reedline::create()` with:
     - `FileBackedHistory` pointing to `~/.ish_history` (unless `--no-history`).
     - `DefaultHinter` for fish-style history autosuggestions.
     - `IshValidator` for multiline bracket counting.
     - `IshHighlighter` for syntax highlighting.
     - Emacs keybindings (default).
   - Custom prompt: `IshPrompt` implementing reedline's `Prompt` trait.
   - Loop on `read_line()`:
     - `Signal::Success(input)` → `process_input(&input, &mut vm)`.
     - `Signal::CtrlC` → clear, continue.
     - `Signal::CtrlD` → break, exit.

2. **`process_input(input, vm)`:**
   - Call `ish_parser::parse(input)`.
   - On parse error → print formatted error, return.
   - On success → call `vm.run(&program)`.
   - On runtime value → print it (suppress `Null`).
   - On runtime error → print formatted error.

3. **`run_file(filename)`:**
   - Read the file contents.
   - Parse with `ish_parser::parse(&contents)`.
   - Execute with `vm.run(&program)`.
   - Exit with appropriate status code.

#### Step 4: Implement reedline traits

| Trait | File | Behavior |
|-------|------|----------|
| `Prompt` | `repl.rs` (inline) | Shows `ish> ` for normal input, `...> ` for continuation lines |
| `Validator` | `validate.rs` | Counts unmatched `{`, `[`, `(`, and triple-quote opens. Returns `Incomplete` if unbalanced. |
| `Highlighter` | `highlight.rs` | Tokenizes input and applies colors: keywords (blue), strings (green), numbers (cyan), comments (gray), operators (yellow). Uses the parser's pest tokenizer or a simpler regex-based approach for keystroke-level speed. |
| `Completer` | `complete.rs` | Stub for now — returns empty completions. Future: complete builtins, variables in scope, filesystem paths, commands on PATH. |

**Files affected:**
- `proto/ish-shell/Cargo.toml` — add `reedline` dependency
- `proto/ish-shell/src/main.rs` — replace with REPL entry point
- `proto/ish-shell/src/repl.rs` — new: REPL loop, prompt, process_input, run_file
- `proto/ish-shell/src/validate.rs` — new: `Validator` implementation
- `proto/ish-shell/src/highlight.rs` — new: `Highlighter` implementation  
- `proto/ish-shell/src/complete.rs` — new: stub `Completer`
- `proto/ish-shell/tests/verification.rs` — new: migrated verification tests

### Decisions

**Decision:** Should the verification demos be migrated to `ish-shell/tests/` or `ish-vm/tests/`? The VM tests would be more appropriate since the demos exercise the VM, not the shell.
--> The verification demos should be deleted entirely.  I plan to implement comprehensive acceptance tests next, which will completely replace the old demos. I don't want to waste time figuring out how to migrate them.

**Decision:** Should the REPL print the result of every expression, or only when the user types an expression-statement (not `let`, `fn`, etc.)?
--> There is no way to execute an expression statement directly from the shell.  The parser will treat it as a shell command.  Don't bother to echo statement results at all.  If I want to see an expression evaluate, I can run `echo "{expression}`.  This will interpolate the results of the expression into a string, and then the echo executable will print the string to stdout.

---

## Feature: Shell Command Execution in the VM

The VM currently stubs out `ShellCommand` and `CommandSubstitution` with error messages. To support the REPL, the interpreter must execute shell commands.

### Issues to Watch Out For

- **Security.** The RFP specifies `shell.unrestricted` as the default and `shell.denied` for packages. Until the assurance ledger is implemented, all execution is unrestricted. The dispatch point must be clearly marked for future standard checks.
- **Glob expansion.** The parser produces `ShellArg::Glob(pattern)` nodes. The VM needs to expand these against the filesystem before passing arguments to the command. The `glob` crate handles this.
- **Environment variable expansion in args.** `ShellArg::EnvVar(name)` needs to be resolved to the variable's value via `std::env::var()`.
- **Pipe execution.** The parser produces `ShellPipeline` chains. The VM needs to connect stdout of one command to stdin of the next using `std::process::Stdio::piped()`.
- **Redirection.** The parser produces `Redirection` nodes. The VM needs to open files and redirect stdout/stderr accordingly.
- **Exit code.** Shell commands return exit codes. The VM should represent these as `Value::Int(code)` so users can check `$?` behavior (future feature). For now, a non-zero exit code should print a message but not throw an error (matching shell convention).
- **Background execution.** The parser supports `background: true`. The decision is to defer background jobs. If the parser produces a background command, the VM should print a "background execution not yet supported" warning and run the command in the foreground.
- **Command not found.** If the command is not on `$PATH`, print a clear error: `ish: command not found: xyz`.

### Proposed Implementation

Add shell command execution to `proto/ish-vm/src/interpreter.rs`. Replace the current stub:

```rust
Statement::ShellCommand { .. } => {
    Err(RuntimeError::new("Shell commands are not supported in this execution mode"))
}
```

with a full implementation:

#### 1. Shell builtins

Before attempting external execution, check for shell builtins:

| Builtin | Implementation |
|---------|---------------|
| `cd <dir>` | `std::env::set_current_dir(dir)`. No args → `$HOME`. |
| `exit` / `quit` | `std::process::exit(code)`. Default code 0. |
| `pwd` | `std::env::current_dir()` → print. |

#### 2. Argument resolution

For each `ShellArg`:
- `Bare(s)` → use as-is.
- `Quoted(s)` → use as-is (quotes already stripped by parser).
- `Glob(pattern)` → expand via `glob::glob(pattern)`, collecting matches into multiple arguments. If no matches, pass the pattern literally (POSIX convention).
- `EnvVar(name)` → `std::env::var(name)`, or empty string if unset.
- `CommandSub(cmd)` → defer (print warning: "command substitution not yet supported").

#### 3. Variable interpolation

The RFP decision says variable interpolation in shell-mode arguments should be implemented immediately. Currently, shell args can include `$var` references to ish variables (not just env vars). This requires the VM to check the ish environment first, then fall back to `std::env::var()`:

```rust
ShellArg::EnvVar(name) => {
    // Check ish scope first, then OS environment
    if let Ok(val) = env.get(name) {
        val.to_display_string()
    } else {
        std::env::var(name).unwrap_or_default()
    }
}
```

#### 4. Simple command execution

```rust
let mut cmd = std::process::Command::new(&command);
cmd.args(&resolved_args);

// Apply redirections
for redir in redirections {
    match redir.kind {
        RedirectKind::StdoutWrite => {
            let file = File::create(&redir.target)?;
            cmd.stdout(Stdio::from(file));
        }
        RedirectKind::StdoutAppend => {
            let file = OpenOptions::new().append(true).create(true).open(&redir.target)?;
            cmd.stdout(Stdio::from(file));
        }
        RedirectKind::StderrWrite => {
            let file = File::create(&redir.target)?;
            cmd.stderr(Stdio::from(file));
        }
        // ... other redirection kinds
    }
}

// TODO(standard:shell): Check active shell standard before dispatching.
// When the assurance ledger is implemented, shell.denied will block here.
let status = cmd.status()?;
Ok(Value::Int(status.code().unwrap_or(-1) as i64))
```

#### 5. Pipe execution

For pipelined commands (`ls | grep foo | wc -l`):

1. Spawn the first command with `stdout(Stdio::piped())`.
2. For each subsequent pipeline stage, spawn with `stdin` connected to the previous command's `stdout` and `stdout(Stdio::piped())` (except the last stage, which inherits the terminal's stdout).
3. Wait for all commands to complete.
4. Return the exit code of the last command.

#### 6. Glob expansion

Add `glob` as a dependency to `ish-vm`:

```toml
glob = "0.3"
```

Expand glob patterns before passing args to `Command`:

```rust
ShellArg::Glob(pattern) => {
    let matches: Vec<String> = glob::glob(pattern)
        .map(|paths| paths.filter_map(|p| p.ok())
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
        .unwrap_or_default();
    if matches.is_empty() {
        vec![pattern.clone()] // No match → literal (POSIX)
    } else {
        matches
    }
}
```

**Files affected:**
- `proto/ish-vm/src/interpreter.rs` — implement `ShellCommand` execution, pipe handling, redirection, glob expansion, builtins
- `proto/ish-vm/Cargo.toml` — add `glob` dependency
- `proto/ish-vm/src/builtins.rs` — optionally register `cd`, `pwd` as ish builtins (or handle inline in the interpreter)

### Decisions

**Decision:** Should glob expansion happen in the VM (where the command is executed) or in a pre-processing step in the shell?
--> VM

**Decision:** Should ish variable interpolation in shell args (`ls $dir`) check ish scope first then env vars, or env vars first then ish scope? The proposed order (ish first, env fallback) means an ish variable can shadow an env var.
--> That is not how variable interpolation works.  Interpolated strings have separate syntaxes for ish expressions and environment variables.  In the example string `"ish: {var1} env: ${var1}"` the first var1 is resolved as an ish expression, and the second var1 is resolved as an environment variable.

**Decision:** Non-zero exit codes — should they print a warning, set a `$?` variable, or be silently ignored?
--> They should set a `$?` variable.  This needs to be implemented as a special value that can be read but not written from the shell.

---

## Feature: Syntax Highlighting

Since the parser is available, syntax highlighting can be implemented now rather than deferred.

### Issues to Watch Out For

- **Performance.** The reedline `Highlighter` is called on every keystroke. Running the full PEG parser on every keystroke may be too slow for large inputs. A simpler regex-based tokenizer may be needed for highlighting, with the full parser reserved for submission.
- **Partial input.** During typing, the input is usually incomplete (e.g., half a string, unclosed brace). The highlighter must handle invalid input gracefully — never crash, and ideally highlight what it can.
- **Color scheme.** Need to choose colors that work on both light and dark terminals.

### Proposed Implementation

Implement a lightweight keyword/token highlighter in `proto/ish-shell/src/highlight.rs`:

1. **Do not** use the full PEG parser for keystroke-level highlighting. Instead, use a simple regex-based tokenizer that recognizes:
   - Keywords: `let`, `mut`, `fn`, `if`, `else`, `while`, `for`, `in`, `return`, `throw`, `try`, `catch`, `finally`, `with`, `defer`, `match`, `use`, `mod`, `pub`, `type`, `true`, `false`, `null`, `and`, `or`, `not` → **bold blue**
   - Strings: `"..."`, `'...'` → **green**
   - Numbers: integer/float literals → **cyan**
   - Comments: `//...`, `#...` → **dark gray**
   - Shell commands: first word of a bare-word line → **bold white**
   - Operators: `+`, `-`, `*`, `/`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=` → **yellow**

2. Use `nu_ansi_term` (reedline's own dependency) for color styling — no additional crate needed.

3. The highlighter returns a `StyledText` (reedline's colored string type) for each call.

**Files affected:**
- `proto/ish-shell/src/highlight.rs` — regex-based syntax highlighter

### Decisions

**Decision:** Should the highlighter use the full parser or a lightweight regex tokenizer? (Recommended: regex for keystrokes, full parser only on submit.)
--> regex for keystrokes, full parser only on submit.

---

## Feature: Shell Scripting (`.ish` file execution)

Since the parser is available, executing `.ish` files from the command line can be implemented now.

### Issues to Watch Out For

- **Shebang.** `.ish` files should support `#!/usr/bin/env ish` as the first line. The parser should skip shebang lines.
- **Exit codes.** A script's exit code should be the exit code of the last statement, or the code passed to `exit`.
- **Error reporting.** Parse errors and runtime errors should include the filename and line numbers.

### Proposed Implementation

1. In `main.rs`, if a filename argument is provided, call `repl::run_file(filename)`.
2. `run_file()`:
   - Read the file with `std::fs::read_to_string()`.
   - Strip shebang line if present (`#!` on line 1).
   - Parse with `ish_parser::parse(&contents)`.
   - On parse error, print errors with filename and line/column numbers, exit with code 1.
   - On success, run with `vm.run(&program)`.
   - On runtime error, print with filename context, exit with code 1.
   - On success, exit with code 0 (or the code from `exit` if called).

3. Update the parser's `ParseError` display to optionally include a filename.

**Files affected:**
- `proto/ish-shell/src/main.rs` — file execution dispatch
- `proto/ish-shell/src/repl.rs` — `run_file()` implementation
- `proto/ish-parser/src/ish.pest` — add shebang rule (skip `#!` line at start)
- `proto/ish-parser/src/error.rs` — optional filename in error display

### Decisions

**Decision:** Should `ish` accept `-e 'code'` for inline execution (like `bash -c` or `python -c`)?
--> Why `-e`, why not `-c` like bash or python.  ish should definitely accept:
--> 1. No args - interactive mode
--> 2. Args - first arg is the name of a file to execute.  Subsequent args should be positional parameters.  We'll defer that for now, we should add it to the todos.
--> 3. An inline execution option. 

---

## Feature: Security Standards (Documentation Only)

Per the original proposal decisions: implement `shell.unrestricted` (default) and `shell.denied` only. Skip `shell.sandboxed` for now — the most common use case is an unrestricted shell that consumes `shell.denied` packages.

### Proposed Implementation

This is documentation-only for now:

1. **Add to assurance ledger spec** (`docs/spec/assurance-ledger.md`):
   - `shell.unrestricted` — default standard for shell mode. All external commands permitted.
   - `shell.denied` — no external command execution. Suitable for library modules and untrusted code.

2. **Add `ShellDenied` error** to `docs/errors/INDEX.md`.

3. **Mark the dispatch point** in `interpreter.rs`:
   ```rust
   // TODO(standard:shell): Check active shell standard before dispatching.
   // When the assurance ledger is implemented, shell.denied will block execution here.
   ```

4. **Add roadmap items** for:
   - `shell.denied` enforcement (requires assurance ledger)
   - `shell.sandboxed` design (deferred)

**Files affected:**
- `docs/spec/assurance-ledger.md` — shell standards section
- `docs/errors/INDEX.md` — `ShellDenied` error
- `docs/project/roadmap.md` — shell security milestones
- `proto/ish-vm/src/interpreter.rs` — TODO comment at dispatch point

---

## Implementation Sequence

The features have dependencies that dictate ordering:

```
1. Migrate verification demos to tests
   ↓
2. Parser integration (add ish-parser to ish-shell)
   ↓
3. Shell command execution in VM (builtins, args, globs, pipes, redirection)
   ↓
4. REPL with reedline (main.rs rewrite, repl.rs, prompt, validator)
   ↓
5. Syntax highlighting (highlight.rs)
   ↓
6. Shell scripting (.ish file execution)
   ↓
7. Documentation updates (roadmap, architecture, assurance ledger, errors)
```

Steps 5 and 6 can be done in parallel after step 4. Step 7 should accompany each step.

---

## Deferred Features (Add to Roadmap)

The following features are explicitly deferred and should be added as "Future" items on the roadmap:

| Feature | Blocked By |
|---------|-----------|
| Background jobs (`&`, `jobs`, `fg`, `bg`) | Job control, process groups, signal handling |
| Command substitution (`$(cmd)`) | Streams |
| Piping into ish functions | Streams |
| Aliases | Not essential; ish functions suffice |
| Signal handling (SIGTSTP, SIGHUP) | Complex, not needed for design phase |
| Startup files (`~/.ishrc`) | Shell stability |
| Tab completion (context-aware) | Scope tracking, PATH scanning |
| `shell.sandboxed` standard | Assurance ledger implementation |
| `shell.denied` enforcement | Assurance ledger implementation |

---

## Documentation Updates

| File | Update |
|------|--------|
| `docs/architecture/shell.md` | Rewrite: REPL architecture, reedline integration, parser integration, command dispatch, pipe/redirect execution |
| `docs/spec/execution.md` | Add REPL behavior to thin-shell configuration |
| `docs/spec/assurance-ledger.md` | Add `shell.unrestricted` and `shell.denied` standard definitions |
| `docs/errors/INDEX.md` | Add `ShellDenied` error |
| `docs/project/roadmap.md` | Move parser to "Completed." Add shell integration to "In Progress." Add deferred shell features to "Future." |
| `docs/user-guide/getting-started.md` | Update to document REPL usage and `.ish` file execution |
| `GLOSSARY.md` | Verify entries for `shell standard`, `subshell`; add if missing |
| `proto/ARCHITECTURE.md` | Update crate dependency graph to show ish-parser → ish-shell link |

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-shell-implementation.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
