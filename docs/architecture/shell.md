---
title: "Architecture: ish-shell"
category: architecture
audience: [all]
status: draft
last-verified: 2026-03-10
depends-on: [docs/architecture/overview.md, docs/spec/syntax.md]
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
- **External commands:** executed via `std::process::Command`
- **Pipes:** command chaining via stdin/stdout piping
- **Redirections:** `>`, `>>`, `2>`, `2>&1`, `&>`
- **Globs:** expanded before command execution
- **`$?`:** synthetic read-only variable holding the last command's exit code

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reedline` | Line editor with validation, highlighting, history, hints |
| `nu-ansi-term` | ANSI color styling for the highlighter |
| `ish-parser` | Source parsing for validation and execution |
| `ish-vm` | Tree-walking interpreter |
| `ish-stdlib` | Standard library functions |

---

## Referenced by

- [docs/architecture/INDEX.md](INDEX.md)
- [docs/architecture/overview.md](overview.md)
