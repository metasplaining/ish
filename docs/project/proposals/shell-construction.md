---
title: "Proposal: Shell Construction"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-15
depends-on: [docs/project/rfp/shell-construction.md, docs/project/proposals/shell-implementation.md, docs/project/proposals/shell-integration.md, docs/spec/execution.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md, docs/architecture/shell.md, docs/project/proposals/language-syntax.md, docs/project/proposals/string-syntax.md, GLOSSARY.md]
---

# Proposal: Shell Construction

*Generated from [shell-construction.md](../rfp/shell-construction.md) on 2026-03-15.*

*Follow-on to [shell-implementation.md](shell-implementation.md), incorporating decisions from review. This is the actionable implementation plan.*

---

## Summary of Decisions

This proposal incorporates the following reviewed decisions from the predecessor proposals:

| Topic | Decision |
|-------|----------|
| Line editor library | Reedline |
| Verification demos | Delete entirely; acceptance tests will replace them separately |
| REPL result echoing | Do not echo. Bare expressions are shell commands. Use `echo "{expr}"` to see values. |
| Parser incomplete vs. error | Yes, enhance. Parser processes whole lines (chunks). |
| Glob expansion location | In the VM |
| `$VAR` in shell args | Resolves to env vars only — not ish scope. `{expr}` is for ish expressions, `$VAR` is for env vars. |
| Exit codes | Set a read-only `$?` variable |
| Syntax highlighting | Regex tokenizer for keystrokes; full parser on submit only |
| Inline execution | Yes, support a `-c` flag (or similar). Also support `ish file.ish`. Positional parameters deferred. |
| History | `--no-history` flag supported |
| Security | `shell.unrestricted` default; `shell.denied` documented; `shell.sandboxed` skipped |

---

## Feature 1: Parser Enhancement — Chunk Parsing and Incomplete Input

The parser currently accepts a full program string via `parse(input: &str) -> Result<Program, Vec<ParseError>>`. For REPL use, the parser needs to distinguish between genuinely invalid input and input that is merely incomplete (e.g., an unclosed brace).

### Issues to Watch Out For

- **Pest limitations.** Pest grammars are PEG-based and match greedily. Detecting "incomplete" typically requires the parser to reach `EOI` prematurely on an otherwise valid prefix. Pest reports this as a parse error like any other.
- **False positives.** Some inputs are ambiguous — `let x =` is incomplete (missing value), but `let x` could be a syntax error rather than an incomplete expression (ish requires initialization). The distinction is sometimes a judgment call.
- **Triple-quote strings.** An unclosed `"""` should be treated as incomplete, not an error. This is harder to detect: the parser sees an open triple-quote and never finds the close.

### Proposed Implementation

Add a new entry point to the parser:

```rust
// ish-parser/src/lib.rs
pub enum ParseResult {
    Complete(Program),
    Incomplete,           // Input looks like the start of valid code but needs more
    Error(Vec<ParseError>),
}

pub fn parse_chunk(input: &str) -> ParseResult
```

**Detection strategy:**

1. First, attempt `parse(input)`. If it succeeds, return `Complete(program)`.
2. If it fails, run a lightweight bracket/quote balance check:
   - Count unmatched `{`, `[`, `(`.
   - Check for unterminated string literals (`"`, `'`, `"""`, `'''`).
   - If any are unbalanced, return `Incomplete`.
3. Otherwise, return `Error(errors)`.

This is a heuristic — it won't catch every case (e.g., `let x =` looks balanced but is incomplete). The REPL's `Validator` provides the first pass (bracket counting on keystrokes), and `parse_chunk` provides the authoritative check on submit.

--> The parser interface should not be changed.  The parser is supposed to be implemented to match invalid input instead of returning an error.  That way, the VM can generate good error messages when the parser matches invalid input, which works better than trying to convert a parser error to a useful error message.  This is something that has not been fully implemented yet.  So, the parser SHOULD have productions like unterminated multi-line string, unterminated list literal, unterminated block, etc.  Fix the parser so that it has these productions.  Then the REPL can have a list of productions that should be treated as incomplete input rather than being treated as complete input or an error.

**Files affected:**
- `proto/ish-parser/src/lib.rs` — add `ParseResult` enum and `parse_chunk()` function

### Decisions

**Decision:** Should the inline execution flag be `-c` (bash convention) or `-e` (perl convention)?
--> `-c` (bash convention)

---

## Feature 2: REPL with Reedline

Delete the verification demos and replace `ish-shell` with an interactive REPL.

### Proposed Implementation

#### main.rs

```rust
mod repl;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let no_history = args.iter().any(|a| a == "--no-history");

    // Filter out flags to get positional args
    let positional: Vec<&str> = args[1..].iter()
        .filter(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .collect();

    if let Some(idx) = args.iter().position(|a| a == "-c") {
        // Inline execution: ish -c 'code'
        let code = args.get(idx + 1).expect("missing argument to -c");
        repl::run_inline(code);
    } else if let Some(filename) = positional.first() {
        // File execution: ish script.ish
        repl::run_file(filename);
    } else {
        // Interactive REPL
        repl::run_interactive(no_history);
    }
}
```

Argument handling is deliberately simple — no external arg-parsing crate.

#### repl.rs

**`run_interactive(no_history: bool)`:**

1. Create `IshVm`, load stdlib via `ish_stdlib::load_all(&mut vm)`.
2. Set up reedline:
   - `FileBackedHistory::with_file(1000, history_path)` unless `no_history`.
   - `DefaultHinter::default().with_style(gray italic)` — fish-style history hints.
   - `IshValidator` — multiline bracket counting (see Feature 3).
   - `IshHighlighter` — regex-based syntax coloring (see Feature 4).
   - `IshPrompt` — shows `ish> ` or `...> `.
3. REPL loop:
   - `Signal::Success(input)` → `process_input(&input, &mut vm)`.
   - `Signal::CtrlC` → clear, continue.
   - `Signal::CtrlD` → exit.

**`process_input(input: &str, vm: &mut IshVm)`:**

1. Call `ish_parser::parse(input)`.
2. On parse error → format error with caret, print to stderr, return.
3. On success → `vm.run(&program)`.
4. On runtime error → format and print to stderr.
5. Do **not** echo result values — the decision is that bare expressions are shell commands and there's no expression-statement access from the shell.

**`run_file(filename: &str)`:**

1. `std::fs::read_to_string(filename)`.
2. Strip shebang line if first line starts with `#!`.
3. `ish_parser::parse(&contents)`.
4. On parse error → print errors with filename and position, `std::process::exit(1)`.
5. `vm.run(&program)`.
6. On runtime error → print, exit 1.
7. On success → exit 0.

**`run_inline(code: &str)`:**

1. `ish_parser::parse(code)`.
2. On error → print, exit 1.
3. `vm.run(&program)`.
4. On error → print, exit 1.
5. On success → exit 0.

**`IshPrompt`:**

Implements reedline's `Prompt` trait. Returns `"ish> "` as the prompt indicator. The continuation prompt for multiline input shows `"...> "`. Reedline handles the continuation case when the `Validator` returns `Incomplete`.

**Files affected:**
- `proto/ish-shell/Cargo.toml` — add `reedline`, `ish-parser` dependencies; remove `ish-codegen` (no longer needed without demos)
- `proto/ish-shell/src/main.rs` — replace entirely
- `proto/ish-shell/src/repl.rs` — new file

---

## Feature 3: Multiline Validator

Implements reedline's `Validator` trait to detect incomplete input before submission.

### Proposed Implementation

Create `proto/ish-shell/src/validate.rs`:

The validator counts unmatched delimiters. It must handle:

- **Braces:** `{` / `}` — code blocks
- **Brackets:** `[` / `]` — list literals
- **Parentheses:** `(` / `)` — grouping, function calls
- **Double-quoted strings:** Skip content inside `"..."` (handle `\"` escapes)
- **Single-quoted strings:** Skip content inside `'...'` (handle `\'` escapes)
- **Triple-quoted strings:** `"""..."""` and `'''...'''` — multiline contexts
- **Comments:** Skip content inside `// ...` (to end of line) and `/* ... */`

If any delimiter count is positive (more opens than closes), return `ValidationResult::Incomplete`. Otherwise return `Complete`.

This is deliberately simple — a character-by-character state machine, not a parser invocation. Speed matters here since this runs on every keystroke.

**Files affected:**
- `proto/ish-shell/src/validate.rs` — new file

---

## Feature 4: Syntax Highlighting

Regex-based keyword/token highlighter for keystroke-level speed.

### Proposed Implementation

Create `proto/ish-shell/src/highlight.rs`:

The highlighter splits input into tokens and applies ANSI colors via `nu_ansi_term` (already a dependency of reedline — no additional crate):

| Token type | Color | Pattern |
|-----------|-------|---------|
| Keywords | Bold blue | `let`, `mut`, `fn`, `if`, `else`, `while`, `for`, `in`, `return`, `throw`, `try`, `catch`, `finally`, `with`, `defer`, `match`, `use`, `mod`, `pub`, `type`, `and`, `or`, `not` |
| Boolean/null literals | Bold cyan | `true`, `false`, `null` |
| Numbers | Cyan | `[0-9]+(\.[0-9]+)?` |
| Double-quoted strings | Green | `"..."` (including `\"` escapes) |
| Single-quoted strings | Green | `'...'` (including `\'` escapes) |
| Comments | Dark gray | `//...` or `#...` to end of line |
| Operators | Yellow | `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<=`, `>=`, `<`, `>`, `=` |
| Everything else | Default | Pass through unstyled |

The highlighter walks the input character by character, building styled spans. String and comment content is treated as a single span (not tokenized further). Keywords are matched as whole words (word boundary check).

The implementation returns reedline's `StyledText` type.

**Files affected:**
- `proto/ish-shell/src/highlight.rs` — new file

---

## Feature 5: Shell Command Execution in the VM

Replace the `ShellCommand` and `CommandSubstitution` stubs in the interpreter with working implementations.

### Proposed Implementation

#### 5a. Shell builtins

Intercept before external dispatch:

| Builtin | Behavior |
|---------|----------|
| `cd [dir]` | `std::env::set_current_dir(dir)`. No args → `$HOME`. Sets `$?` to `0` on success, `1` on error. |
| `exit [code]` | `std::process::exit(code)`. Default code `0`. |
| `pwd` | Print `std::env::current_dir()` to stdout. |

#### 5b. Argument resolution

For each `ShellArg` in the AST:

| Variant | Resolution |
|---------|-----------|
| `Bare(s)` | Use as-is |
| `Quoted(s)` | Use as-is (quotes stripped by parser) |
| `Glob(pattern)` | Expand via `glob::glob(pattern)`. Collect matches. If no matches, pass pattern literally (POSIX convention). Each match becomes a separate argument. |
| `EnvVar(name)` | `std::env::var(name).unwrap_or_default()`. This resolves **only** against OS environment variables — not ish scope. This matches the string syntax: `$VAR` is always an env var, `{expr}` is always an ish expression. |
| `CommandSub(cmd)` | Print warning: "command substitution not yet supported — requires streams". Return empty string. |

#### 5c. Simple command execution

```rust
// TODO(standard:shell): Check active shell standard before dispatching.
// When the assurance ledger is implemented, shell.denied will block execution here.

let mut cmd = std::process::Command::new(&command);
cmd.args(&resolved_args);
// Apply redirections...
let status = cmd.status()?;
let code = status.code().unwrap_or(-1) as i64;
// Update $? (see Feature 6)
Ok(ControlFlow::None)
```

On `std::io::ErrorKind::NotFound` → print `ish: command not found: {command}` and set `$?` to `127` (standard shell convention).

#### 5d. Redirection

For each `Redirection` in the AST:

| Kind | Implementation |
|------|---------------|
| `StdoutWrite` (`>`) | `File::create(target)` → `cmd.stdout(Stdio::from(file))` |
| `StdoutAppend` (`>>`) | `OpenOptions::new().append(true).create(true).open(target)` → `cmd.stdout(...)` |
| `StderrWrite` (`2>`) | `File::create(target)` → `cmd.stderr(Stdio::from(file))` |
| `StderrAndStdout` (`2>&1`) | `cmd.stderr(Stdio::from(cmd.stdout.try_clone()))` — redirect stderr to wherever stdout goes |
| `AllWrite` (`&>`) | Open file, redirect both stdout and stderr to it |

#### 5e. Pipe execution

For pipelines (`cmd1 | cmd2 | cmd3`):

1. Build a `Vec<Command>` from the first command + pipeline stages.
2. Spawn the first command with `stdout(Stdio::piped())`.
3. For each middle stage: `stdin(prev.stdout.take())`, `stdout(Stdio::piped())`.
4. For the last stage: `stdin(prev.stdout.take())`, inherit terminal stdout.
5. Wait for all children. Set `$?` to the exit code of the last command.

Apply redirections to the appropriate stage (redirections on the first command apply before piping; redirections on later stages apply to those stages).

#### 5f. Background execution

The parser supports `background: true`. Background jobs are deferred. If encountered, print `ish: background execution not yet supported` and run the command in the foreground.

**Files affected:**
- `proto/ish-vm/src/interpreter.rs` — replace `ShellCommand` stub with full execution
- `proto/ish-vm/Cargo.toml` — add `glob = "0.3"` dependency

---

## Feature 6: Exit Code Variable (`$?`)

Shell commands set a `$?` variable that can be read but not written from ish code.

### Issues to Watch Out For

- **Special variable semantics.** `$?` is not a regular ish variable — it cannot be assigned, only read. It must be updated after every shell command execution.
- **Grammar interaction.** `$?` uses the `$` prefix which the parser treats as an env var. One option: `$?` is an env var that the VM sets. Another: it's a special ish variable that the VM manages.
- **Naming.** `$?` is the bash convention. Alternatives: `$STATUS`, `$EXIT_CODE`, `last_exit_code`.

### Proposed Implementation

Implement `$?` as a synthetic environment variable that the VM sets after each shell command:

```rust
// After executing a shell command:
std::env::set_var("?", code.to_string());
```

This way, `$?` in shell args or `$?` in interpolated strings (`"exit code: $?"`) resolves naturally via the existing `EnvVar` mechanism — no parser changes needed. The VM sets the variable after each `ShellCommand` execution.

The parser already handles `$?` because `env_var` matches `$` followed by identifier-like characters. However, `?` is not an identifier character in the current grammar. The grammar rule needs a small extension:

```pest
env_var = @{
    "$" ~ "?" |   // Special: $?
    "$" ~ "{" ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* ~ "}" |
    "$" ~ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")*
}
```

**Files affected:**
- `proto/ish-vm/src/interpreter.rs` — set `$?` after shell command execution
- `proto/ish-parser/src/ish.pest` — extend `env_var` rule to accept `$?`

### Decisions

**Decision:** Should `$?` be implemented as a synthetic env var (simplest) or as a special ish variable with custom read/write semantics?
--> synthetic env var

---

## Feature 7: Shell Scripting and Inline Execution

Support executing `.ish` files and inline code from the command line.

### Proposed Implementation

#### File execution

`ish script.ish` runs the file. Implementation is in `repl::run_file()` (see Feature 2).

**Shebang support:** The parser needs to skip `#!` lines. Since `#` is already a comment character in ish, a shebang line (`#!/usr/bin/env ish`) will be parsed as a comment by default — no grammar change needed in the current pest grammar. The `#` comment rule handles it:

```pest
line_comment = @{ ("//" | "#") ~ (!NEWLINE ~ ANY)* }
```

So `#!/usr/bin/env ish` is already a valid comment. No parser change required.

#### Inline execution

`ish -c 'let x = 5; echo "{x}"'` executes the inline code. Implementation is in `repl::run_inline()` (see Feature 2).

#### Positional parameters (deferred)

`ish script.ish arg1 arg2` — the args after the filename should be available as positional parameters (`$1`, `$2`, etc. or a `$ARGS` list). This is deferred.

**Files affected:**
- `proto/ish-shell/src/main.rs` — `-c` flag handling
- `proto/ish-shell/src/repl.rs` — `run_file()`, `run_inline()`

---

## Feature 8: Security Standards Documentation

Document `shell.unrestricted` and `shell.denied` in the assurance ledger spec and error catalog. No code changes.

### Proposed Implementation

1. **Add to `docs/spec/assurance-ledger.md`** — a new "Shell Standards" section:

   > ### Shell Standards
   >
   > **`shell.unrestricted`** — the default standard for all execution configurations. All external commands are permitted. All environment variables are accessible.
   >
   > **`shell.denied`** — disables external command execution entirely. Any attempt to execute a shell command throws a `ShellDenied` error (E007). Only ish builtins and ish-native functions are available. Suitable for library modules and untrusted code.

2. **Add to `docs/errors/INDEX.md`**:

   > | *E007* | Runtime | ShellDenied — attempted to execute an external command while `shell.denied` standard is active |

3. **Add to `docs/project/roadmap.md`**:
   - Move "Parser (PEG or other approach)" from "Future" to "Completed."
   - Add "Shell REPL (reedline)" to "In Progress."
   - Add deferred items to "Future."

**Files affected:**
- `docs/spec/assurance-ledger.md` — Shell Standards section
- `docs/errors/INDEX.md` — E007
- `docs/project/roadmap.md` — update milestones

---

## Implementation Sequence

```
1. Parser enhancement: parse_chunk() and $? grammar
   ↓
2. VM: Shell command execution (builtins, args, globs, env vars, pipes, redirections, $?)
   ↓
3. REPL: main.rs, repl.rs, prompt (Reedline integration, parse → VM → output)
   ↓
4. Validator: validate.rs (multiline bracket counting)
   ↓
5. Highlighter: highlight.rs (regex tokenizer)
   ↓    ↓
6a. File execution (.ish)     6b. Inline execution (-c)
   ↓
7. Documentation updates
```

Steps 4 and 5 can be done in parallel. Steps 6a and 6b are independent of each other and can be done in parallel after step 3. Step 7 accompanies each step.

---

## Deferred Items

Add these to the roadmap as "Future":

| Feature | Blocked By | Notes |
|---------|-----------|-------|
| Background jobs (`&`, `jobs`, `fg`, `bg`) | Job control, process groups | Parser already supports `background: true` |
| Command substitution (`$(cmd)`) | Streams | Parser already supports `CommandSubstitution` |
| Piping into ish functions | Streams | |
| Positional parameters (`$1`, `$2`, `$ARGS`) | Design decision needed | For `ish file.ish arg1 arg2` |
| Aliases | Not essential | ish functions suffice |
| Signal handling (SIGTSTP, SIGHUP) | Complexity | |
| Startup files (`~/.ishrc`) | Shell stability | |
| Tab completion (context-aware) | Scope tracking, PATH scanning | Stub `Completer` included |
| `shell.denied` enforcement | Assurance ledger | Standard documented, not enforced |
| `shell.sandboxed` design | Assurance ledger | Skipped entirely for now |
| Acceptance tests | Separate effort | Replace old verification demos |

---

## Documentation Updates

| File | Update |
|------|--------|
| `docs/architecture/shell.md` | Rewrite: REPL architecture, reedline, parser integration, command dispatch, pipe/redirect execution |
| `docs/spec/execution.md` | Add REPL behavior to thin-shell configuration |
| `docs/spec/assurance-ledger.md` | Add `shell.unrestricted` and `shell.denied` standard definitions |
| `docs/errors/INDEX.md` | Add `E007: ShellDenied` |
| `docs/project/roadmap.md` | Parser → Completed. Shell REPL → In Progress. Deferred shell features → Future. |
| `docs/user-guide/getting-started.md` | Document REPL usage, `ish file.ish`, `ish -c` |
| `GLOSSARY.md` | Verify/add entries for `shell standard`, `subshell` |
| `proto/ARCHITECTURE.md` | Update crate dependency graph to show `ish-parser → ish-shell` link |

Remember to update `## Referenced by` sections in all affected files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-shell-construction.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
