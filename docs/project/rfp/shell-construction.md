---
title: "RFP: Shell Construction"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/project/proposals/shell-implementation.md, docs/project/rfp/shell-implementation.md, docs/spec/execution.md, docs/spec/syntax.md, docs/architecture/shell.md, GLOSSARY.md]
---

# RFP: Shell Construction

*Converted from decisions in [shell-implementation.md](../proposals/shell-implementation.md) on 2026-03-15.*

---

## Context

The [shell implementation proposal](../proposals/shell-implementation.md) was reviewed and decisions were made. This RFP captures those decisions and requests a final implementation proposal that can be acted on directly.

## Decisions from the Shell Implementation Proposal

### Parser Enhancement

The parser should be enhanced to distinguish "incomplete input" from "parse error." The old prototype handled this by parsing "chunks" — a chunk could be a whole file or a single line from stdin. The shell never needs to process anything smaller than a whole line. A line is not submitted to the VM for processing until the user presses return, even when that line contains multiple statements separated by a semicolon.

### Verification Demos

Delete them entirely. Do not migrate them to tests. Comprehensive acceptance tests will be implemented separately and will completely replace the old demos.

### REPL Result Echoing

Do not echo statement results. There is no way to execute an expression statement directly from the shell — the parser will treat bare expressions as shell commands. To see an expression evaluate, use `echo "{expression}"` which interpolates the expression result into a string and then the `echo` executable prints it to stdout.

### Glob Expansion

Glob expansion should happen in the VM, where the command is executed.

### Variable Interpolation in Shell Args

The original proposal's `ShellArg::EnvVar` resolution was incorrect. Interpolated strings have separate syntaxes for ish expressions and environment variables:
- `{expr}` resolves as an ish expression.
- `$VAR` / `${VAR}` resolves as an environment variable.

In the example string `"ish: {var1} env: ${var1}"`, the first `var1` is an ish expression and the second is an environment variable. `$VAR` in a shell argument should resolve only against environment variables, not ish scope. This is the standard shell convention and matches the string syntax proposal.

### Exit Codes

Non-zero exit codes should set a `$?` variable. This must be implemented as a special value that can be read but not written from the shell.

### Syntax Highlighting

Use a regex-based tokenizer for keystroke-level highlighting. Only use the full parser on submit.

### Inline Execution

The shell should accept:
1. **No args** — interactive mode.
2. **First arg is a filename** — execute that file. Subsequent args should be positional parameters (defer positional parameters for now; add to todos).
3. **An inline execution flag** — accept a flag for inline code execution (like bash `-c` or python `-c`). The specific flag name should be determined.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
