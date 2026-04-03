---
title: "Architecture: ish-shell"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-31
depends-on: [docs/architecture/overview.md, docs/spec/syntax.md, docs/spec/concurrency.md]
---

# ish-shell

**Source:** `proto/ish-shell/src/`

Interactive shell and script runner for the ish language. Supports three execution modes: interactive REPL, file execution, and inline execution.

---

## Execution Modes

| Mode | Invocation | Description |
|------|-----------|-------------|
| Interactive | `ish` | Reedline-based REPL with syntax highlighting, multiline input, and history |
| File | `ish file.ish` | Execute an ish source file (shebang-aware) |
| Inline | `ish -c 'code'` | Execute a code string from the command line |

### Flags

- `--no-history` — disable history file in interactive mode

---

## Module Structure

| File | Purpose |
|------|---------|
| `main.rs` | CLI argument dispatch: REPL, file, or inline mode |
| `repl.rs` | Interactive REPL loop, file/inline execution entries |
| `validate.rs` | `IshValidator` — parser-based multiline detection |
| `highlight.rs` | `IshHighlighter` — regex-based syntax coloring |

---

## REPL Architecture

The REPL uses Reedline with three custom components:

### Validator (`IshValidator`)

Implements Reedline's `Validator` trait. On every Enter keypress, invokes `ish_parser::parse()` and checks `program.has_incomplete_continuable()`. If the AST contains continuable `Incomplete` nodes (unclosed braces, brackets, parentheses, multi-line strings), Reedline inserts a newline and shows the continuation prompt. Non-continuable incomplete input (e.g., unterminated single-line strings) submits immediately and reports an error.

### Highlighter (`IshHighlighter`)

Implements Reedline's `Highlighter` trait. A character-by-character tokenizer that applies ANSI colors:
- **Keywords** (blue bold): `let`, `fn`, `if`, `else`, `while`, `for`, `match`, etc.
- **Literals** (cyan bold): `true`, `false`, `null`
- **Numbers** (cyan): integer and float literals
- **Strings** (green): single-quoted and double-quoted
- **Comments** (dark gray): `//` and `#` to end of line
- **Operators** (yellow): `+`, `-`, `*`, `/`, `==`, `!=`, etc.

### Prompt

Uses Reedline's `DefaultPrompt` showing `ish> `. Continuation lines show `...> `.

### History

File-backed history at `~/.ish_history` (1000 entries). Disabled with `--no-history`.

---

## Parser-Matches-Everything Philosophy

The parser always succeeds. Incomplete or malformed input produces `Incomplete` AST nodes instead of parse errors. This enables:
1. **Multiline detection** — the REPL checks for continuable `Incomplete` nodes to decide whether to request more input
2. **Error localization** — non-continuable `Incomplete` nodes indicate syntax errors at specific positions
3. **Uniform handling** — no separate bracket-counting or heuristic pre-filter needed

---

## Shell Command Execution

Shell commands (`ShellCommand` AST nodes) are executed by the VM interpreter:

- **Builtins:** `cd` (change directory), `pwd` (print working directory), `exit` (terminate)
- **External commands:** executed via `tokio::process::Command` (async)
- **Pipes:** command chaining via stdin/stdout piping
- **Redirections:** `>`, `>>`, `2>`, `2>&1`, `&>`
- **Globs:** expanded before command execution
- **`$?`:** synthetic read-only variable holding the last command's exit code

---

## Two-Thread Architecture

In interactive mode, the shell uses two threads to allow the Reedline line editor and the Tokio-based VM to run concurrently:

### Shell Thread

Runs the Reedline event loop and the parser. When the user submits input, the shell thread parses it into a `Program` AST (which is `Send`) and sends it to the main thread via a channel. The shell thread then waits for a completion signal before showing the next prompt.

**Parser placement rationale:** The parser is stateless and produces a `Send`-safe AST. Running it on the shell thread avoids blocking the `LocalSet` during parsing and keeps the main thread focused on execution.

Parse errors are displayed directly on the shell thread — they never reach the main thread.

### Main Thread

Runs the Tokio runtime with a `LocalSet`. Receives `Program` AST from the shell thread, executes it via the async interpreter, and sends a completion signal back when the top-level task finishes. Spawned futures survive after the submitting task completes — they continue running on the `LocalSet`.

Runtime errors are formatted to strings on the main thread before being sent to output.

### Communication Channels

| Channel | Direction | Payload |
|---------|-----------|--------|
| Program submission | Shell → Main | `Program` AST (`Send`) |
| Completion signal | Main → Shell | Unit signal (no display content) |
| Output | Main → Shell | Strings via Reedline `ExternalPrinter` |

### ExternalPrinter Integration

All program output — expression results, `println`, errors, and background task output — routes through Reedline's `ExternalPrinter` in interactive mode. The `ExternalPrinter` writes to a channel that the Reedline event loop reads, ensuring output does not interleave with the prompt or user input.

In non-interactive mode (file execution, inline execution), there is no shell thread. The main thread parses and executes directly, and output goes to OS stdout/stderr.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reedline` | Line editor with validation, highlighting, history, hints |
| `nu-ansi-term` | ANSI color styling for the highlighter |
| `tokio` | Async runtime (`LocalSet`, `spawn_local`, `process::Command`) |
| `ish-parser` | Source parsing for validation and execution |
| `ish-vm` | Tree-walking interpreter |
| `ish-stdlib` | Standard library functions |

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
- [docs/architecture/vm.md](vm.md)
- [docs/spec/concurrency.md](../spec/concurrency.md)
