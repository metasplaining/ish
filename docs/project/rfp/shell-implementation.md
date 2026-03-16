---
title: "RFP: Shell Implementation"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/project/proposals/shell-integration.md, docs/project/rfp/shell-integration.md, docs/spec/execution.md, docs/spec/assurance-ledger.md, docs/spec/syntax.md, docs/architecture/shell.md, GLOSSARY.md]
---

# RFP: Shell Implementation

*Converted from decisions in [shell-integration.md](../proposals/shell-integration.md) on 2026-03-15.*

---

## Context

The [shell integration proposal](../proposals/shell-integration.md) was reviewed and decisions were made. This RFP captures those decisions and requests a follow-on implementation proposal.

The original proposal incorrectly assumed the parser had not yet been implemented. In fact, the `ish-parser` crate is fully functional — a PEG-based parser (using `pest`) with a comprehensive grammar covering all language features including shell integration (commands, pipes, redirections, globs, environment variables, command substitution, background execution, and the force-command prefix). The parser has 150+ tests across 9 test files. However, it is not yet integrated into any consumer crate: `ish-shell` builds ASTs directly via the builder API, and the VM stubs out `ShellCommand` and `CommandSubstitution` execution.

## Decisions from the Original Proposal

### Library Choice

**Reedline.** Use reedline as the line editor library.

### Verification Demos

**Delete entirely.** The existing 6 verification demos in `ish-shell/src/main.rs` should be removed, not preserved behind a flag. The shell binary should become a proper REPL.

### History Flag

**Yes.** Support a `--no-history` flag for scripting and testing contexts.

### Shell Features

**Implement now:**
- External command execution
- Command history (file-backed)
- Line editing (emacs/vi) — provided by reedline
- Fish-style history hints — provided by reedline
- Multiline input (open brace continuation)
- Prompt customization
- Variable interpolation in commands — implement immediately, not deferred
- Environment variable access
- Exit / quit
- `cd`
- Syntax highlighting — implement now (the parser exists)
- Pipes — implement now (the parser supports them)
- Redirection — implement now (the parser supports them)
- Globbing — implement now (the parser supports them)
- Shell scripting (`.ish` file execution) — implement now (the parser exists)

**Defer:**
- Background jobs (requires job control, process groups, signal handling)
- Command substitution (requires subshell / streams)
- Piping into ish functions (requires streams)
- Aliases
- Signal handling beyond Ctrl-C/D
- Startup files (`~/.ishrc`) — defer until shell is more stable

### Pipe Syntax

**Defer to parser.** The parser already uses `|` for pipes in shell mode. The question of `|` vs `|>` and its interaction with union types is a parser concern to be resolved separately.

### Security Standards

- **Default standard:** `shell.unrestricted` is the default.
- **Skip `shell.sandboxed` for now.** The most common use case is that an unrestricted shell uses `shell.denied` packages. Sandboxing is not needed yet.
- **Implement `shell.unrestricted` and `shell.denied` only.** Document interfaces as before.

### Subshell

**Deferred.** Subshell return type (`$(cmd)`) is TBD.

## Requirements

Create an implementation proposal that:

1. Accounts for the parser being available and fully functional.
2. Integrates `ish-parser` into `ish-shell` and the VM.
3. Implements shell command execution in the VM interpreter (currently stubbed out).
4. Replaces the verification demos with a proper reedline-based REPL.
5. Implements all the "implement now" features listed above.
6. Documents deferred features in the roadmap.
7. Updates the roadmap to reflect that the parser is complete.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
