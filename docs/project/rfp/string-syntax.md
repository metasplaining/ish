---
title: "RFP: String Syntax"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-15
depends-on: [docs/spec/syntax.md, docs/spec/types.md, docs/spec/execution.md, docs/project/proposals/language-syntax.md, GLOSSARY.md]
---

# RFP: String Syntax

*Generated from the language syntax proposal decisions on 2026-03-15.*

---

## Overview

The language syntax proposal established double-quoted strings as the baseline for the initial syntax phase. All other string-related syntax was deferred to this follow-on because string handling in a shell-language hybrid is complex enough to warrant dedicated analysis.

ish needs a string syntax that serves two audiences simultaneously: programmers writing `.ish` source files and interactive users typing commands in the shell. The syntax must handle quoting, interpolation, multiline content, and raw strings while remaining clean and conventional.

## Current State

The current syntax supports only double-quoted strings with `+` for concatenation:

```ish
let name = "Alice"
let greeting = "Hello, " + name + "!"
```

No interpolation, raw strings, multiline strings, or char literals are specified. The following decisions from the language syntax proposal are marked TBD:

- Interpolation syntax
- Raw string syntax
- Quote style (single, double, or both)
- Char literal syntax

## Requirements

### 1. String Literals Containing Quotes

It must be possible to write string literals that contain the ordinary quote characters (`"` and `'`) without requiring escape sequences. This is a common need in both programming and shell usage — shell commands frequently contain both kinds of quotes, and requiring escapes is error-prone and ugly.

Approaches to explore:

- Bash-style: unescaped single quotes inside double-quoted strings and vice versa
- Heredoc or multi-character delimiter syntax where the string doesn't end until the chosen delimiter is seen
- Triple-quote or similar block syntax (Python, Kotlin, Swift)
- Backtick or other alternate delimiter

### 2. Multiline Strings

It must be possible to write string literals that span multiple lines without escape sequences like `\n`. Multiline strings are essential for embedding templates, SQL queries, shell scripts, and formatted text.

Consider:

- How is leading whitespace/indentation handled?
- Is there a dedent mechanism (Kotlin `trimMargin`, Swift indentation stripping)?
- Can multiline strings be interpolated?

### 3. String Interpolation

It must be possible to embed expressions inside string literals. This is one of the most-used features in modern languages and is essential for ish.

Specific questions to address:

- **Syntax:** `f"...{expr}..."` (Python), `` `...${expr}...` `` (TypeScript/JavaScript), `"...${expr}..."` (Kotlin, Dart), `"...\(expr)..."` (Swift), or something else?
- **Nesting:** Can interpolated expressions contain strings? How deep can nesting go?
- **Format specifiers:** Are format specifiers supported inside interpolation (e.g., `f"{value:.2f}"` in Python)?
- **Environment variables:** Interpolation must support both ish variables and environment variables (`$HOME`, `${PATH}`) in both shell mode and programming mode. How does the syntax distinguish between them?

### 4. Shell Mode Strings

In shell mode, unquoted strings are the norm for command arguments. The string syntax must account for:

- Bare words as command arguments (`ls -la foo.txt`)
- Glob patterns (`*.rs`, `src/**/*.ish`)
- Environment variable expansion (`$HOME`, `${PATH}`)
- Quoting rules for arguments containing spaces or special characters
- How interpolation works in shell mode vs. programming mode

### 5. Consistency Between Modes

There should be as much consistency as possible between programming mode and shell mode string handling. A developer should not have to learn two different quoting systems. Where the modes must differ (e.g., bare words in shell mode), the differences should be minimal and unsurprising.

### 6. Char Literals

ish needs a `char` type and literal syntax. However, char literals are rare in practice, so the syntax can be "clunky" — it should not steal one of the useful quote characters (`"` or `'`) that would be better used for strings.

Approaches to explore:

- Rust-style `'c'` (but this steals single quotes from strings)
- Named syntax like `char('c')` or `c'A'`
- Unicode escape like `\u{41}`
- Some prefix or suffix notation

### 7. Combinatorial Coverage

The syntax must support all combinations of:

- **Interpolated or not** — plain string vs. string with embedded expressions
- **Multiline or not** — single-line vs. multi-line content
- **Quote-containing or not** — content that includes quote characters without escapes

This gives eight use cases (2×2×2) that all need clean solutions.

### 8. Clean and Conventional

Despite the complexity of the requirements, the syntax should be clean, readable, and recognizable to developers coming from other languages. Exotic or novel syntax should be avoided unless it provides clear advantages over established conventions.

## Research Scope

The proposal should survey string handling across a broad range of languages — not just Rust and TypeScript, but also:

- **Python** — f-strings, triple-quoted strings, raw strings
- **Kotlin** — `"${expr}"`, triple-quoted multiline with `trimMargin()`/`trimIndent()`
- **Swift** — `"\(expr)"`, multi-line strings with indentation stripping, extended delimiters (`#"..."#`)
- **Dart** — `"${expr}"`, single and double quotes, triple quotes for multiline, raw strings `r"..."`
- **Ruby** — `"#{expr}"`, `'...'` for non-interpolated, heredocs, `%q` and `%Q` delimiters
- **Raku (Perl 6)** — extensive quoting system with `Q`, `q`, `qq`, user-chosen delimiters, heredocs
- **Nushell** — string handling in a structured shell context
- **Bash** — single quotes (literal), double quotes (interpolating), `$'...'` (escape sequences), heredocs
- **PowerShell** — single quotes (literal), double quotes (interpolating), here-strings
- **Julia** — `"$(expr)"`, triple-quoted strings, raw strings
- **Zig** — multiline strings with `\\`, no string interpolation (uses `std.fmt`)
- **Nim** — raw strings, triple-quoted strings, `fmt"{expr}"` via library

For each language, note:

- What syntax is used and how well-received it is by the community
- Known pain points or gotchas
- How interpolation and quoting interact

## Deferred Decisions

The following decisions from the language syntax proposal must be resolved:

1. **Quote styles** — double-only, both single and double, or more options?
2. **Interpolation syntax** — `f"...{expr}..."`, `` `...${expr}...` ``, `"...{expr}..."`, or other?
3. **Raw strings** — `r"..."` (Rust/Python-style), or some other mechanism?
4. **Multiline strings** — triple quotes, heredocs, indentation stripping, or other?
5. **Char literals** — what syntax, given that stealing `'` may conflict with string goals?
6. **Environment variable interpolation** — how to distinguish `$var` (ish variable) from `$VAR` (environment variable) in interpolated strings?

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
