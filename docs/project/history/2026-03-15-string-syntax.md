---
title: String Syntax Design and Implementation
category: project
audience: [all]
status: draft
last-verified: 2026-03-15
depends-on: [docs/project/proposals/string-syntax.md, docs/project/rfp/string-syntax.md, docs/spec/syntax.md]
---

# String Syntax Design and Implementation

*March 15, 2026*

## The Problem

The language syntax proposal had deliberately deferred string syntax to a follow-on proposal. The interim decision was double-quoted strings only — functional, but known to be inadequate. The deferral documented a rich set of requirements: string literals that can contain quote characters without escapes, multiline strings, string interpolation that handles both ish variables and environment variables, consistency between programming and shell modes, and a char literal syntax that does not steal a valuable quote character. An RFP had been generated covering all of these requirements.

The RFP led to a comprehensive proposal that surveyed string handling across fourteen languages — Python, Kotlin, Swift, Dart, Ruby, Raku, Nushell, Bash, PowerShell, Julia, Zig, Nim, Rust, and TypeScript — examining not just their syntax but how well-received each approach was by its community. From this survey, clear patterns emerged that guided the design.

## The Central Decision: Shell Convention

The most consequential decision was adopting the shell convention for quote styles: single-quoted strings (`'...'`) are literal with no interpolation, and double-quoted strings (`"..."`) support interpolation and escapes. This is one of the most deeply ingrained patterns in computing — Bash, PowerShell, Nushell, Fish, and Ruby all follow it. Since ish is a shell-language hybrid, fighting this convention would create friction for the primary audience.

The alternative — the tentative pick from the language syntax proposal — was double-quoted strings with an `f"..."` prefix for interpolation and single-quoted chars. This allocated the premium single-quote character to char literals, which are vanishingly rare in practice. Most modern languages (Python, TypeScript, Swift, Dart, Ruby) don't even have distinct char literal syntax and manage fine. Giving single quotes to strings and using a prefix syntax (`c'A'`) for the rare char literal was the clear practical choice.

The decision also meant that double-quoted strings are always interpolating — no `f` prefix needed. With single-quoted strings available for literal text, the risk of accidental interpolation drops significantly. Users who need literal braces or dollar signs in an interpolating context use `\{` and `\$`. Users who want no interpolation at all use single quotes.

## Interpolation Design

The interpolation syntax combined two mechanisms: `{expr}` for ish expression interpolation and `$VAR` for environment variable expansion. This separation mirrors the AST's existing structure (`StringPart::Expr` vs `Expression::EnvVar`) and matches how shell users think — `$HOME` means an environment variable, not an ish variable.

The question of `$var` (ish variable) versus `$VAR` (environment variable) was resolved cleanly by this split. There is no `$var` syntax for ish variables inside strings — ish variables use `{expr}`. Environment variables use `$VAR`. The two are syntactically distinct and the parser can produce the correct AST node without ambiguity.

Escaped braces (`\{`, `\}`) and escaped dollars (`\$`) prevent interpolation when the literal characters are needed in an interpolating string. The full escape set for double-quoted strings is: `\n`, `\t`, `\r`, `\\`, `\"`, `\{`, `\}`, `\$`, `\0`, `\u{XXXX}`.

## Multiline Strings

Triple-quoted strings (`"""..."""` for interpolating, `'''...'''` for literal) follow the convention established by Python, Kotlin, Dart, and Julia. The key design question was indentation handling — without automatic dedent, developers must break code formatting to avoid unwanted whitespace in the string content.

Swift's approach won: the closing delimiter's column position defines the baseline indentation. Each content line's leading whitespace up to that column is stripped. This is elegant, requires no function call (unlike Kotlin's `trimIndent()`), and handles the common case where multiline strings are indented to match surrounding code.

## Char Literals

The proposal analyzed five approaches to char literals. With single quotes claimed by literal strings, the traditional `'c'` syntax was unavailable. The options ranged from a `c'A'` prefix to no char literal syntax at all (relying on type-directed coercion from single-character strings).

The decision favored `c'A'` as a dedicated syntax. While novel, its intent is immediately obvious, the syntax is clean, and its infrequent usage means that novelty is not a significant cost. The proposal had recommended type coercion as primary with `char()` as a convenience function, but the decision went with a dedicated literal syntax plus the `char()` builtin for programmatic construction.

In the AST, `Literal::Char(char)` was added. In the VM, `Value::Char(char)` was added with support for char-char concatenation (producing a string), char-string concatenation, and char comparison. The `char()` builtin accepts a single-character string, an integer code point, or a `Value::Char` identity.

## Extended Delimiters

For the rare case where a string contains both quote types and escaping is inconvenient, extended delimiters wrap any string form with `~`: `~"..."~`, `~'...'~`, `~"""..."""~`, `~'''...'''~`. Content inside extended delimiters is verbatim — no escape processing.

The proposal had initially recommended deferring extended delimiters to a later phase, but the decision was to include them immediately for all four quote styles. The rationale was that the grammar additions are small, the implementation is straightforward (extended content is just verbatim text), and having them available from the start avoids a painful gap where developers encounter strings they cannot express cleanly.

The proposal's original design followed Swift's `#"..."#` convention, but implementation revealed a conflict: the ish grammar accepts both `//` and `#` as line comment starters (shell convention), and pest's implicit COMMENT rule would consume `#"hello"#` as a comment before the parser could recognize it as an extended delimiter. Rather than hack the comment rule, `~` was chosen as the delimiter character — it is unused elsewhere in the grammar, easy to type, and visually unobtrusive. Extended delimiters are not available in shell mode to avoid ambiguity.

## Coverage Verification

The proposal formally verified that the chosen syntax covers all eight combinations of (interpolated × multiline × contains-quotes). The coverage matrix:

- Non-interpolated, single-line, no quotes: `'hello'`
- Non-interpolated, single-line, with quotes: `'She said "hello"'` or extended delimiters
- Non-interpolated, multiline, no quotes: `'''...'''`
- Non-interpolated, multiline, with quotes: `'''...'''` (content includes `"` freely)
- Interpolated, single-line, no quotes: `"Hello, {name}!"`
- Interpolated, single-line, with quotes: `"She said 'hello'"`
- Interpolated, multiline, no quotes: `"""..."""`
- Interpolated, multiline, with quotes: `"""..."""` (content includes `'` and `"` freely)

No gaps were identified.

## Implementation

The implementation touched all six prototype crates. In ish-ast, `Literal::Char(char)` was added to the enum and `Expression::char_lit()` was added as a convenience constructor. The display layer was updated so that `Literal::String` renders with single quotes (`'hello'`), `Literal::Char` renders as `c'A'`, and `StringInterpolation` no longer produces an `f` prefix.

The parser grammar was substantially rewritten. The old `string_literal` rule (double-quoted) and `f_string` rule were replaced with eight new string forms: `string_literal` (single-quoted), `interp_string` (double-quoted with `{expr}` and `$VAR`), `triple_double_string`, `triple_single_string`, `char_literal`, plus four extended delimiter variants. The `primary` rule was ordered to try extended triples first (longest match), then extended singles, then regular triples, then char literals, then interpolating strings, then literal strings.

The AST builder gained new functions for each string form, with distinct escape processing: `unescape_single_string` handles only `\\` and `\'`, while `unescape_double_string` handles the full escape set. Triple-quoted strings gained indentation stripping logic based on the closing delimiter's position. Extended delimiter content is passed through verbatim.

In the VM, `Value::Char(char)` was added with `is_truthy` (always true), `type_name` ("char"), display, and equality. The interpreter evaluates `Literal::Char` to `Value::Char`, supports char-char and char-string concatenation via the `+` operator, and supports char comparison. The `char()` builtin was added to the conversion function group.

The reflection system was updated for `Literal::Char` round-tripping through the AST factory, and the self-hosted Rust code generator was updated to emit char literals as single-quoted Rust characters.

Twenty-nine new tests were added across the parser and VM test suites, covering all string forms, escape sequences, interpolation, char operations, and display round-trips. All 224 tests pass, and all six shell verification demos continue to pass.

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
