---
title: "Proposal: String Syntax"
category: proposal
audience: [all]
status: accepted
last-verified: 2026-03-15
depends-on: [docs/project/rfp/string-syntax.md, docs/spec/syntax.md, docs/spec/types.md, docs/spec/execution.md, docs/project/proposals/language-syntax.md, GLOSSARY.md]
---

# Proposal: String Syntax

*Generated from [string-syntax.md](../rfp/string-syntax.md) on 2026-03-15.*

---

## Questions and Answers

### Q: How should the syntax distinguish `$var` (ish variable) from `$VAR` (environment variable) in interpolated strings?

This is the most consequential design question because it affects both modes and has no established convention across languages. Three viable approaches exist:

1. **Naming convention only.** `$name` always resolves through a unified lookup: check ish scope first, then environment. This is simple but ambiguous — a local variable `HOME` would shadow the environment variable `$HOME`.

2. **Sigil differentiation.** Use `{expr}` for ish expressions and `$VAR` / `${VAR}` for environment variables inside interpolated strings. This mirrors the AST's existing separation (`StringPart::Expr` vs `Expression::EnvVar`).

3. **Explicit namespace.** Use `{expr}` for ish expressions and `{env.HOME}` or `{$HOME}` for environment access. This makes every reference syntactically unambiguous.

**Analysis:** Approach 2 is the strongest fit. It mirrors how Bash works (developers already expect `$HOME` to mean the environment), it matches the existing AST representation, and it avoids shadowing problems. In an interpolated string, `{expr}` evaluates an ish expression, while `$VAR` or `${VAR}` performs environment lookup. Outside of interpolated strings, `$VAR` continues to work as already implemented.

### Q: Can interpolated expressions contain strings? How deep can nesting go?

Yes, interpolated expressions should be able to contain string literals, including nested interpolated strings. Most modern languages (Kotlin, Swift, Python 3.12+) allow this. The AST already supports it — `StringPart::Expr` holds any `Expression`, which can include `StringInterpolation`. There is no need for an artificial nesting limit; the parser handles recursive descent naturally. Example:

```ish
f"Hello, {if name != "" { name } else { "stranger" }}!"
f"result: {f"{x} + {y}" + " = " + f"{x + y}"}"
```

### Q: Are format specifiers supported inside interpolation?

They should be deferred to a future proposal. Format specifiers (Python's `f"{value:.2f}"`) add complexity to the parser and are better handled through a format function or method initially. This matches ish's incremental approach — start simple, add specifiers when the need is proven. The interpolation syntax should be designed so that format specifiers *could* be added later (e.g., `{expr:spec}`) without breaking existing code.

### Q: What syntax is used in each surveyed language, and how well-received is it?

**Survey of string handling across languages:**

| Language | Quote types | Interpolation | Multiline | Raw strings | Char literal | Reception |
|----------|------------|---------------|-----------|------------|-------------|-----------|
| **Python** | `"..."`, `'...'` (identical) | `f"...{expr}..."` | `"""..."""`, `'''...'''` | `r"..."` | N/A (single-char string) | f-strings are beloved; triple-quote well-established |
| **Kotlin** | `"..."` | `"...${expr}..."` or `"...$name..."` | `"""..."""` with `trimIndent()`/`trimMargin()` | N/A (use `"""..."""`) | `'c'` | String templates praised; `trimIndent()` is ergonomic |
| **Swift** | `"..."` | `"...\(expr)..."` | `"""..."""` with indentation stripping | `#"..."#` (extended delimiters) | N/A (Character type, no literal syntax) | Extended delimiters widely praised; `\()` slightly unusual but liked |
| **Dart** | `"..."`, `'...'` (identical) | `"...${expr}..."` or `"...$name..."` | `"""..."""`, `'''...'''` | `r"..."` | N/A | Clean design; well-received |
| **Ruby** | `"..."` (interpolating), `'...'` (literal) | `"...#{expr}..."` | Heredocs (`<<~HEREDOC`) | `'...'` is already raw-ish | N/A | Heredocs powerful; `#{}` familiar to Rubyists |
| **Raku** | `Q`, `q`, `qq` + user-chosen delimiters | `qq"...{expr}..."` or `"...$var..."` | Built into all forms | `Q[...]` | N/A | Extremely flexible but steep learning curve |
| **Nushell** | `"..."` (interpolating), `'...'` (literal) | `$"...($expr)..."` | Via `"..."` with newlines | `'...'` | N/A | Clean; practical; well-liked by shell users |
| **Bash** | `"..."` (interpolating), `'...'` (literal) | `"...$var..."` or `"...${expr}..."` | Heredocs (`<<EOF`, `<<-EOF`, `<<'EOF'`) | `'...'` | N/A | Universal; heredocs essential but syntax dated |
| **PowerShell** | `"..."` (interpolating), `'...'` (literal) | `"...$var..."` or `"...$($expr)..."` | Here-strings (`@"..."@`, `@'...'@`) | `'...'` | N/A | Here-strings useful; `$()` for expressions is clear |
| **Julia** | `"..."` | `"...$(expr)..."` or `"...$name..."` | `"""..."""` | `raw"..."` | `'c'` | Clean; triple-quote standard |
| **Zig** | `"..."` | None (use `std.fmt`) | `\\` line prefix | N/A | N/A | Deliberate simplicity; multiline syntax divisive |
| **Nim** | `"..."` | `fmt"{expr}"` (library macro) | `"""..."""` | `r"..."` | N/A | Library approach is flexible but less ergonomic |
| **Rust** | `"..."` | None (use `format!()` macro) | Just include newlines | `r"..."`, `r#"..."#` | `'c'` | `r#` delimiters solve quoting; no interpolation is a pain point |
| **TypeScript** | `"..."`, `'...'`, `` `...` `` | `` `...${expr}...` `` | `` `...` `` (template literals) | N/A (no built-in) | N/A | Template literals universally loved; having 3 quote types is mixed |

**Key observations from the survey:**

1. **`${}` is the most common interpolation syntax** — used by Kotlin, Dart, Bash, PowerShell, Julia, TypeScript (in template literals). Python's `f"{}"` is the second most common.
2. **Triple quotes dominate multiline** — Python, Kotlin, Dart, Julia, Nim all use `"""..."""`. Heredocs are the shell tradition but feel dated in a programming context.
3. **Two quote types with different semantics** is extremely common in shells (Bash, PowerShell, Nushell, Ruby). Single-quoted = literal, double-quoted = interpolating.
4. **Extended delimiters** (Swift's `#"..."#`, Rust's `r#"..."#`) are the most elegant solution for strings containing quotes.
5. **Raw strings** with `r"..."` prefix are well-established (Python, Rust, Dart, Nim).
6. **Char literals** using `'c'` exist in Rust, Kotlin, and Julia, but most languages (Python, TypeScript, Swift, Dart, Ruby) simply use single-char strings.

---

## Feature 1: Quote Styles

### Issues to Watch Out For

- **Single quotes are premium real estate.** Assigning them to char literals (as the language syntax proposal tentatively suggested) permanently removes them from string duty. Given that char literals are rare and strings-containing-quotes are common, this may be the wrong allocation.
- **Shell mode expectations.** Shell users strongly expect `'...'` to be a literal (non-interpolating) string. All major shells (Bash, PowerShell, Nushell, Fish) work this way.
- **Breaking the symmetry.** If single and double quotes have *different* semantics, this must be clearly communicated. If they are synonyms (Python, Dart), the language gains little.
- **The old prototype** used both single and double quotes as synonyms. The language syntax proposal moved to double-only with single reserved for chars.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Double quotes only** | Simplest; no semantic difference to learn; char literals get `'c'` | Cannot write `"She said \"hello\""` cleanly; shell users lose single-quote-literal convention |
| **B: Both quotes, identical semantics** (Python/Dart) | Easy to embed one quote type inside the other; familiar to Python/Dart devs | Does not solve the interpolation-vs-literal distinction; char literals need different syntax; shell users may expect single = literal |
| **C: Shell convention — `'...'` literal, `"..."` interpolating** (Bash/PowerShell/Nushell) | Shell users feel at home; covers the quotes-inside-strings case; natural semantic distinction | Diverges from the language syntax proposal's tentative decision; single quotes are no longer available for chars; may surprise pure-programming users who expect `'...'` == `"..."` |
| **D: Double quotes for plain strings, `f"..."` for interpolation, single quotes for chars** (language syntax proposal's tentative pick) | Explicit interpolation; char support; familiar `f` prefix from Python | Cannot embed double quotes without escaping or raw strings; wastes `'` on rare char literals; shell users lose `'...'` convention |
| **E: Shell convention + extended delimiters for edge cases** | Combines C's benefits with Swift/Rust escape hatch for pathological strings | Slightly more syntax to learn (extended delimiters are rarely needed) |

**Analysis:**

Alternative C/E is the strongest fit for ish's dual nature as a shell *and* a programming language. The shell convention of `'literal'` vs `"interpolating"` is one of the most deeply ingrained patterns in computing. Ish is a shell-language hybrid — fighting this convention creates friction for the primary audience.

Alternative D (the tentative pick from the language syntax proposal) sacrifices single quotes for char literals, which are rare in practice. Most modern languages (Python, TypeScript, Swift, Dart, Ruby) don't even have a distinct char literal syntax and manage fine. The ish type system defines `char`, but the literals can use a less common syntax.

**Recommendation:** Alternative E — shell convention with extended delimiters.

- `"..."` — interpolating string (ish variables via `{expr}`, env vars via `$VAR`)
- `'...'` — literal string (no interpolation, no escape processing except `\\` and `\'`)
- Extended delimiters for strings containing both quote types (see Feature 5)

### Proposed Implementation

```ish
// Double-quoted: supports interpolation and escape sequences
let greeting = "Hello, {name}!"
let path = "Home is $HOME"
let escaped = "She said \"hello\""

// Single-quoted: literal, no processing
let raw_regex = '(\d+)\s+'
let sql = 'SELECT * FROM users WHERE name = "Alice"'
let json_template = '{"key": "value"}'

// Edge case: string containing both quote types
// (handled by extended delimiters — see Feature 5)
```

### Decisions

**Decision:** Adopt shell convention (`'literal'`, `"interpolating"`) or keep double-only with `f"..."` prefix?
--> Adopt shell convention

**Decision:** If shell convention is adopted, should `'...'` process *any* escapes (like `\'` and `\\`), or be truly raw (Bash-style, where `'` can never appear inside `'...'`)?
--> Support `\\` and `\'`

---

## Feature 2: String Interpolation

### Issues to Watch Out For

- **Ambiguity with block syntax.** If `{expr}` is the interpolation delimiter inside `"..."`, then a string containing a literal `{` needs escaping. This is manageable (use `\{` or `{{`) but must be specified.
- **Environment variable collision.** `$name` inside `"..."` could mean either an ish variable or an environment variable. The syntax must make the distinction clear.
- **Shell mode compatibility.** Shell users expect `$VAR` to expand environment variables inside double quotes, just like Bash.
- **Parser complexity.** Implicit interpolation in all double-quoted strings (Kotlin/Dart-style) is more complex to parse than prefix-triggered interpolation (Python's `f"..."`).

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `f"...{expr}..."` prefix** (Python-style) | Explicit opt-in; plain strings need no special parsing; familiar from Python | Extra prefix for every interpolated string; `f` may look odd to non-Pythonistas |
| **B: `"...{expr}..."` implicit** (always interpolating) | No prefix needed; cleaner for the common case; resembles Kotlin/Dart (but with `{}` not `${}`) | Literal `{` needs escaping even in plain strings; breaks expectation from C-family languages that `{` is not special in strings |
| **C: `"...${expr}..."` with `$name` shorthand** (Kotlin/Dart/Bash-style) | Familiar to shell users and Kotlin/Dart devs; `$` signals interpolation; `{}` only needed for complex expressions | Every `$` in a string needs escaping; collides with environment variable syntax `$HOME` |
| **D: `"...\(expr)..."` ** (Swift-style) | No collision with `$` (env vars) or `{` (JSON); `\` already signals "special" | Unusual to most developers; `\(` looks like an escape rather than interpolation |
| **E: Hybrid — `{expr}` for ish expressions, `$VAR`/`${VAR}` for env vars** | Clean separation; `$` means environment (as in shell tradition); `{` means expression | Literal `{` needs escaping; two interpolation mechanisms in one string type |
| **F: `f"..."` prefix + `{expr}` for ish expressions + `$VAR` for env vars** | Combines Python's explicitness with shell's `$VAR` convention; plain `"..."` strings need no parsing for `{` or `$` | Verbose for simple cases |

**Analysis:**

The critical constraint is that ish is a shell hybrid. Shell users expect `"...$HOME..."` to expand environment variables. Any syntax that repurposes `$` for ish variable interpolation (Alternative C) creates a direct collision with this deeply-ingrained expectation.

Alternative E provides the cleanest separation: `{expr}` is ish expression interpolation, `$VAR` is environment variable expansion, and they coexist naturally in the same string. This matches the existing AST structure (`StringPart::Expr` vs `Expression::EnvVar`).

The question then is whether interpolation is implicit in all `"..."` strings (Alternative E) or opt-in via `f"..."` (Alternative F).

Arguments for implicit (always-interpolating `"..."`):
- More concise — no prefix on the most common case
- Matches Kotlin, Dart, Bash, PowerShell, Nushell
- ish already has single-quoted `'...'` for literal strings

Arguments for explicit (`f"..."`):
- Plain `"..."` can safely contain `{` without escaping — important for JSON, format strings, regex with quantifiers like `{3}`
- Matches Python, which has the most popular interpolation syntax
- The prefix makes intent explicit — a reader knows immediately that a string contains expressions

**Recommendation:** If the shell convention (Feature 1 Alternative E) is adopted for quote styles, use **implicit interpolation** in double-quoted strings (Alternative E). The rationale: with `'...'` available for literal strings, the need for non-interpolating double-quoted strings drops significantly. Users who need literal braces or dollars in non-interpolating contexts use `'...'`. For the occasional literal `{` in an interpolated string, use `\{`.

If double-only quotes are kept (Feature 1 Alternative A/D), use **explicit `f"..."` prefix** (Alternative F), because without single-quoted literal strings, every double-quoted string risks unintended interpolation.

### Proposed Implementation (assuming shell convention for quotes)

```ish
// Ish expression interpolation
let name = "Alice"
let greeting = "Hello, {name}!"              // -> "Hello, Alice!"
let calc = "Result: {2 + 2}"                 // -> "Result: 4"
let cond = "{if x > 0 { "positive" } else { "negative" }}"

// Environment variable expansion
let home = "$HOME"                            // -> "/home/alice"
let path = "Path is: ${PATH}"                // -> "Path is: /usr/bin:..."

// Both in one string
let msg = "User {user.name} home: $HOME"

// Escaping literal braces and dollars
let json = "key: \{not interpolated\}"        // -> "key: {not interpolated}"
let price = "Cost: \$50"                      // -> "Cost: $50"

// Literal strings — no interpolation at all
let pattern = '{name}: $HOME'                 // -> "{name}: $HOME"
```

### Decisions

**Decision:** Implicit interpolation in `"..."` (Kotlin/Bash-style) or explicit `f"..."` prefix (Python-style)?
--> Implicit interpolation in `"..."` (Kotlin/Bash-style)

**Decision:** Escape syntax for literal braces — `\{` or `{{`?
-->`\{`

**Decision:** Escape syntax for literal dollar sign in double-quoted strings — `\$` or `$$`?
-->`\$`

---

## Feature 3: Multiline Strings

### Issues to Watch Out For

- **Indentation handling.** Multiline strings in source code are indented to match the surrounding code, but the string content should not include that indentation. Without a dedent mechanism, developers must left-align multiline strings, breaking code formatting.
- **Trailing newlines.** Should the string include a trailing newline after the closing delimiter?
- **Interpolation in multiline strings.** Multiline strings should support the same interpolation as single-line strings.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Triple-quoted `"""..."""`** (Python/Kotlin/Dart/Julia) | Widely established; immediately recognizable; can contain unescaped `"` | Need a dedent mechanism; three characters to open/close |
| **B: Heredocs** (Bash/Ruby) | Shell heritage; can choose arbitrary delimiter; indented form (`<<~`) handles dedent | Awkward in expressions; feels dated in a modern language; multi-line syntax split across two locations |
| **C: Backtick-delimited** (TypeScript template literals) | Familiar to web devs; clean look; built-in multiline | Steals backtick for other uses (shell command substitution?); TypeScript-specific |
| **D: `"""..."""` with automatic indentation stripping** (Swift/Kotlin `trimIndent()`) | Clean; handles indentation naturally; familiar | Must specify precise stripping rules |

**Analysis:**

Triple-quoted strings (`"""..."""`) are the clear winner. They are used by Python, Kotlin, Dart, Julia, Nim, and Swift (with adaptations). Heredocs are the shell tradition but are awkward in a programming context — the delimiter appears on a different line from the string content, making them hard to use inside expressions.

The key design question is indentation handling. Three approaches:

1. **No automatic stripping** (Python before `textwrap.dedent`): The string includes all leading whitespace exactly as written. Simple but forces ugly formatting.
2. **Library function** (Kotlin `trimIndent()`/`trimMargin()`): The string is raw; a method call strips indentation. Explicit but verbose.
3. **Automatic stripping based on closing delimiter position** (Swift): The closing `"""` determines the indentation baseline. Any whitespace at the start of each line up to that column is stripped. Elegant and requires no function call.

**Recommendation:** Alternative D — triple-quoted strings with Swift-style automatic indentation stripping based on closing delimiter position.

The closing `"""` must appear on its own line. Its column position defines the "base indentation." Each content line must start with at least that much whitespace, which is stripped. Lines with less indentation than the closing delimiter are a compile error. The string does not include a leading or trailing newline from the delimiter lines.

### Proposed Implementation

```ish
// Basic multiline
let sql = """
    SELECT *
    FROM users
    WHERE active = true
    """
// Result: "SELECT *\nFROM users\nWHERE active = true\n"
// (4-space indent stripped based on closing """ position)

// Interpolation in multiline
let query = """
    SELECT *
    FROM {table_name}
    WHERE id = {id}
    """

// Containing double quotes — no escaping needed
let json = """
    {
        "name": "Alice",
        "age": 30
    }
    """
// Note: literal { and " inside triple-quoted strings present a parsing
// question — see Decisions below.

// Single-line triple-quoted (unusual but legal)
let s = """hello"""
```

The triple-quoted form also applies to single-quoted delimiters for literal multiline strings:

```ish
let template = '''
    No {interpolation} or $expansion here.
    Everything is literal.
    '''
```

### Decisions

**Decision:** Triple-quoted strings with Swift-style indentation stripping, or Kotlin-style `trimIndent()` library function?
--> Triple-quoted strings with Swift-style indentation stripping

**Decision:** Should `{` inside triple-quoted double-quoted strings trigger interpolation? (If yes, `\{` to escape. If no, interpolation only in `f"""..."""`.)
--> Yes

**Decision:** Include trailing newline before closing `"""`?
--> Yes

**Decision:** Allow `'''...'''` for multiline literal strings?
--> Yes

---

## Feature 4: Char Literals

### Issues to Watch Out For

- **Frequency.** Char literals are rare in application code. The syntax should not steal valuable characters from more common constructs.
- **Type system requirement.** The type spec defines `char` as "a single Unicode scalar value." The AST currently has no `Char` literal variant — it uses `Literal::String` for single characters. The VM's `Value` enum has no `Char` variant either.
- **Single quote contention.** If single quotes are used for literal strings (Feature 1), they cannot be used for char literals.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `'c'`** (Rust/Kotlin/Julia) | Clean; familiar to C-family devs; established convention | Steals single quotes from string use; conflicts with Feature 1's recommendation |
| **B: `c'A'`** (prefixed) | Preserves `'...'` for strings; obvious intent; prefix is mnemonic | Novel syntax; no established precedent |
| **C: `char('A')` or `char("A")`** (function-style) | No new syntax; works with existing parser; clear intent | Verbose; technically a function call, not a literal; optimizer must constant-fold |
| **D: `c"A"`** (prefixed double-quote) | Preserves single quotes; uses existing double-quote infrastructure; similar to Rust `b"byte string"` | Novel; easily confused with string |
| **E: No char literal syntax** (Python/TypeScript/Swift approach) | Simplest; single-char strings coerce to `char` where needed | Loses type safety at the literal level; `char` type exists but has no literal |

**Analysis:**

If Feature 1 adopts the shell convention (single quotes for literal strings), Alternative A is eliminated. The type system requires `char`, but the *literal syntax* can be chunky because char literals are rare.

Alternative C (`char("A")`) is the most pragmatic: it requires no new parser syntax, it's readable, and the analyzer can constant-fold it at compile time. "A" is a string literal that the `char()` constructor narrows to a `char` value.

Alternative E is also viable — many successful languages (Python, TypeScript, Swift) don't have char literals. A single-character string is simply typed as `char` when the context requires it. The type annotation `let c: char = "A"` is clear and requires no new syntax.

**Recommendation:** Alternative E as the primary mechanism (type-directed coercion) with Alternative C as a convenience function.

```ish
let c: char = "A"           // string literal coerced to char by type annotation
let c = char("A")           // explicit char construction
```

Both forms are clean, require no new syntax, and don't steal quote characters. The analyzer verifies that the string contains exactly one Unicode scalar value.

### Proposed Implementation

1. **No new literal syntax.** Char values are created from single-character string literals via type coercion or the `char()` constructor.
2. **Add `Literal::Char(char)` to the AST** — the analyzer lowers a `Literal::String("A")` to `Literal::Char('A')` when the target type is `char`. Alternatively, skip the AST variant and handle it entirely in the type checker.
3. **Add `Value::Char(char)` to the VM** — or continue using `Value::String` for single characters and let the type system enforce the constraint.
4. **Add `char()` builtin** — takes a single-character string or integer (Unicode code point) and returns a `char`.

### Decisions

**Decision:** Dedicated char literal syntax (which?), or type-directed coercion from string literals + `char()` function?
--> `c'A'` - Intent is obvious.  Syntax is clean.  Novelty is offset by infrequent usage.

**Decision:** Add `Literal::Char` to AST and `Value::Char` to VM, or handle char as a type-level constraint on `String`?
--> Add `Literal::Char` to AST and `Value::Char` to VM

---

## Feature 5: Strings Containing Both Quote Types (Extended Delimiters)

### Issues to Watch Out For

- **Complexity budget.** Extended delimiters are a power feature. They must be simple enough that developers can read and write them without looking up the syntax.
- **Rare usage.** Most strings contain at most one type of quote. Extended delimiters are for edge cases — the syntax should not be optimized at the expense of everyday readability.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: Swift-style `#"..."#`** | Elegant; `#` count can increase (`##"..."##`); disables escape interpretation unless `\#(...)` is used | Novel to many devs; the `#` count mechanism is unusual |
| **B: Rust-style `r#"..."#`** | Established (Rust devs know it); raw by definition; `#` count scales | Only raw — no interpolation; the `r` prefix is specific to Rust |
| **C: User-chosen delimiter** (Raku, Perl) | Maximum flexibility; handles any content | Complex; hard to scan visually; steep learning curve |
| **D: Triple-quoted strings are sufficient** | No new syntax needed; `"""` can contain `"`, `'''` can contain `'` | Cannot contain `"""` or `'''` respectively; rare edge case |
| **E: Swift-style extended delimiters with interpolation support** | Handles arbitrary content *and* interpolation; `#"..."#` for raw, `#"...\#(expr)..."#` for interpolated | Full generality; slightly complex but rarely needed |

**Analysis:**

Triple-quoted strings (Feature 3) already handle most cases: `"""` can contain unescaped `"` and `'`, and `'''` can contain unescaped `'` and `"`. The only remaining case is a string that contains `"""` itself, which is vanishingly rare.

Swift-style extended delimiters (Alternative A/E) are the most elegant solution for the rare case. They scale naturally: `#"..."#` handles one level, `##"..."##` handles two levels. They are also the only surveyed mechanism that maintains interpolation support while suppressing normal escape processing.

**Recommendation:** Alternative E — Swift-style extended delimiters with interpolation support, but classified as a **Phase 2** feature. Triple-quoted strings handle 99% of cases. Extended delimiters can wait until a real need emerges.

### Proposed Implementation (Phase 2)

```ish
// Extended delimiter — contains unescaped " and '
let s = ~"She said "hello" and it's fine"~

// Extended delimiter with interpolation (future)
let s = ~"She said "hello" to \~(name)"~

// Extended triple-quoted
let s = ~"""
    Contains """ and "
    """~
```

### Decisions

**Decision:** Include extended delimiters in Phase 1, or defer to Phase 2?
--> Phase 1

**Decision:** If included, Swift-style `#"..."#` or a different mechanism?
--> Tilde-delimited `~"..."~`, for all four quote styles: non-interpolated/single line, non-interpolated/multiline, interpolated/single line, interpolated/multiline. The original proposal recommended Swift-style `#"..."#`, but `#` conflicts with `#` line comments in the ish grammar. `~` was chosen as the delimiter character — unused in the grammar, easy to type, and visually clean. Extended delimiters are not available in shell mode.

---

## Feature 6: Raw Strings

### Issues to Watch Out For

- **Overlap with single-quoted strings.** If single-quoted strings are literal (no interpolation, no escape processing), they *are* raw strings. A separate `r"..."` prefix becomes redundant.
- **Shell expectations.** Shell users expect `'...'` to be the raw/literal form. Adding another raw string mechanism is confusing.

### Critical Analysis

| Alternative | Pros | Cons |
|-------------|------|------|
| **A: `r"..."` prefix** (Python/Rust) | Well-known; explicit intent | Redundant if `'...'` is already literal |
| **B: `'...'` is the raw string** (shell convention) | No new syntax; shell-compatible; consistent with Feature 1 | Not labeled "raw" — may not be obvious to non-shell devs |
| **C: Both `'...'` and `r"..."`** | Belt-and-suspenders; accommodates both traditions | Two ways to do the same thing; violates "one obvious way" |
| **D: Extended delimiters serve as raw strings** | `#"..."#` is inherently raw (escapes disabled unless `\#()` used) | Only available if extended delimiters are adopted |

**Analysis:**

If the shell convention is adopted (Feature 1), single-quoted strings *are* raw strings. There is no need for a separate `r"..."` syntax. This follows the Bash/PowerShell/Nushell tradition and reduces the number of string forms.

If single quotes are not adopted for literal strings (i.e., Feature 1 Alternative A or D is chosen), then `r"..."` is needed and should follow the Python/Rust convention.

**Recommendation:** Conditional on Feature 1:
- If `'...'` = literal: No `r"..."` prefix needed. `'...'` is the raw string.
- If double-only quotes: Add `r"..."` for raw strings.

### Decisions

**Decision:** If single-quoted literal strings are adopted, is a separate `r"..."` syntax still needed?
--> Extended delimiters serve as raw strings.  It is worth explicitly calling out that escape sequences are disabled for all four extended delimiter types.

---

## Feature 7: Shell Mode String Handling

### Issues to Watch Out For

- **Bare words are the default.** In shell mode, `ls -la foo.txt` treats each space-separated token as a string argument. These are not quoted strings — they are bareword tokens.
- **Glob expansion.** `*.rs` must not be treated as a syntax error or a string — it's a glob pattern that expands to file matches.
- **Quoting rules.** When a shell argument contains spaces, it must be quoted: `echo "hello world"`. The quoting rules must be consistent with programming mode.
- **Mixed mode.** A line like `let result = $(curl -s "https://api.example.com/{endpoint}")` combines programming mode (variable assignment), command substitution, and string interpolation.

### Critical Analysis

The existing shell mode spec (syntax.md) already establishes the framework:
- Bare words for command arguments
- `$VAR` / `${VAR}` for environment variables
- `$(cmd)` for command substitution
- `"..."` for quoted arguments
- Glob patterns in shell mode

The string syntax proposal needs to ensure consistency between shell mode and programming mode. The key principle: **quoting rules are the same in both modes.** A double-quoted string in shell mode follows the same interpolation rules as in programming mode. A single-quoted string in shell mode is literal, just as in programming mode.

### Proposed Implementation

```ish
// Shell mode: bare words
ls -la foo.txt

// Shell mode: environment variables (no quotes needed for single words)
echo $HOME
cd $HOME/projects

// Shell mode: quoted arguments with spaces
grep "hello world" file.txt

// Shell mode: interpolation in quoted arguments
let file = "output.txt"
grep "pattern" {file}

// Shell mode: single-quoted literal arguments
grep 'no $expansion' file.txt

// Shell mode: command substitution with interpolation
let count = $(wc -l < "{file}")

// Shell mode: glob patterns (unquoted — expanded by the shell layer)
ls *.rs
rm -f build/**/*.o

// Programming mode: same quoting rules
let msg = "Hello, {name}! Path: $HOME"
let literal = 'No {interpolation} here'
```

### Decisions

**Decision:** Are quoting rules identical between shell mode and programming mode (recommended), or are there mode-specific differences?
--> Quoting rules are almost identical, with one difference: Extended delimiters are not available in shell mode.  Bareword tokens would create a grammatical ambiguity with extended delimiters.  If someone **really** wants extended delimiters for a string literal in the shell, they can just assign the string literal to a variable, which is a programming mode statement, and then interpolate the variable into a shell command.

---

## Feature 8: Combinatorial Coverage Verification

The RFP requires that all 8 combinations of (interpolated × multiline × contains-quotes) are cleanly supported. Here is the coverage matrix under the recommended syntax:

| Interpolated | Multiline | Contains quotes | Recommended syntax | Example |
|---|---|---|---|---|
| No | No | No | `'hello'` | `let s = 'hello'` |
| No | No | Yes | `'She said "hi"'` | `let s = 'He said "it''s fine"'` or extended delimiters |
| No | Yes | No | `'''...'''` | `let s = '''`<br>`    line 1`<br>`    line 2`<br>`    '''` |
| No | Yes | Yes | `'''...'''` | Content can include `"` freely |
| Yes | No | No | `"Hello, {name}"` | `let s = "Hello, {name}!"` |
| Yes | No | Yes | `"She said 'hello'"` | `let s = "She said 'hello'"` |
| Yes | Yes | No | `"""..."""` | `let s = """`<br>`    Hello, {name}`<br>`    """` |
| Yes | Yes | Yes | `"""..."""` | Content can include `'` and `"` freely |

**Gap analysis:**

- **Non-interpolated, single-line, contains both `"` and `'`**: This is the edge case that single-quoted or double-quoted strings alone cannot handle. Solutions: (a) escape one of them (`\'` in single-quoted or `\"` in double-quoted), (b) use extended delimiters (`~"..."~`), or (c) use triple-quoted form even for single-line content. This is rare enough that escaping is acceptable in Phase 1; extended delimiters solve it cleanly in Phase 2.
- **All other combinations** are covered cleanly by the recommended syntax.

### Decisions

**Decision:** Is the gap (both quote types, non-interpolated, single-line) acceptable to solve with escaping in Phase 1?
--> Since I did not decide to go with the recommended solution, the coverage matrix is different.  Re-check to make sure the decided solution has no gaps, and all combinations are cleanly covered by the decided syntax.

---

## Summary of Recommended Syntax

| Form | Semantics | Interpolation | Escapes |
|------|-----------|---------------|---------|
| `"..."` | Standard string | Yes: `{expr}` for ish, `$VAR` for env | Yes: `\n`, `\t`, `\\`, `\"`, `\{`, `\$` |
| `'...'` | Literal string | No | Minimal: `\\`, `\'` only |
| `"""..."""` | Multiline interpolating | Yes | Yes, same as `"..."` |
| `'''...'''` | Multiline literal | No | Minimal, same as `'...'` |
| `~"..."~` | Extended delimiter | Yes, via `\~(expr)` (future) | Verbatim — no escape processing |
| `char("A")` | Char construction | N/A | N/A |

**Escape sequences in double-quoted strings:** `\n` (newline), `\t` (tab), `\r` (carriage return), `\\` (backslash), `\"` (double quote), `\{` (literal brace), `\$` (literal dollar), `\0` (null), `\u{XXXX}` (Unicode scalar).

**Escape sequences in single-quoted strings:** `\\` (backslash), `\'` (single quote). All other characters are literal.

---

## Phasing

### Phase 1 (Implement Now)

- Double-quoted strings with `{expr}` interpolation and `$VAR` environment expansion
- Single-quoted literal strings
- Triple-quoted multiline strings (`"""..."""` and `'''...'''`) with automatic indentation stripping
- `char()` builtin function
- Type-directed coercion from single-character string to `char`
- Escape sequences as specified above

### Phase 2 (Deferred)

- Extended delimiters (`~"..."~`, `~"""..."""~`) — implemented ahead of schedule in Phase 1
- Format specifiers inside interpolation (`{expr:format}`)
- Heredoc syntax (if shell users demand it)

---

## Documentation Updates

The following documentation files will need updates when this proposal is implemented:

- [docs/spec/syntax.md](../../spec/syntax.md) — § Strings: replace the current stub with the full string syntax specification
- [docs/spec/types.md](../../spec/types.md) — § Char: update char literal examples to reflect the chosen syntax; update string literal type examples if needed
- [docs/spec/execution.md](../../spec/execution.md) — § Shell: ensure shell mode string handling is documented
- [docs/architecture/ast.md](../../architecture/ast.md) — Update AST node documentation for `StringInterpolation`, `StringPart`, `EnvVar`; add `Literal::Char` if adopted
- [docs/architecture/vm.md](../../architecture/vm.md) — Update builtins section for `char()`; update value representation if `Value::Char` is added
- [docs/user-guide/getting-started.md](../../user-guide/getting-started.md) — Update string examples
- [docs/user-guide/functions.md](../../user-guide/functions.md) — Update any string-related function examples
- [docs/ai-guide/patterns.md](../../ai-guide/patterns.md) — Update patterns if string syntax changes affect AI usage
- [docs/project/proposals/language-syntax.md](language-syntax.md) — Update the deferred string decisions to reference this proposal
- [GLOSSARY.md](../../../GLOSSARY.md) — Add entries for: interpolated string, literal string, extended delimiter, triple-quoted string, bare word (verify existing)

Remember to update `## Referenced by` sections in all modified files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-15-string-syntax.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
- [docs/project/rfp/string-syntax.md](../rfp/string-syntax.md)
