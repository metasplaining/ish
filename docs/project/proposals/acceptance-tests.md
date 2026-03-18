---
title: "Proposal: Acceptance Tests"
category: proposal
audience: [all]
status: proposal
last-verified: 2026-03-16
depends-on: [docs/project/rfp/acceptance-tests.md, docs/spec/syntax.md, docs/architecture/shell.md, docs/architecture/overview.md, AGENTS.md, CONTRIBUTING.md]
---

# Proposal: Acceptance Tests

*Generated from [acceptance-tests.md](../rfp/acceptance-tests.md) on 2026-03-16.*

---

## Questions and Answers

### Q: Where should the acceptance tests live?

The RFP specifies `proto/ish-tests`. This directory does not currently exist. The prototype lives entirely under `proto/`, so placing tests there is consistent. However, because these are Bash scripts (not Rust code), they do not participate in the Cargo workspace. `proto/ish-tests` is appropriate — it is inside the prototype tree but clearly separate from the Rust crates.

### Q: How is the ish shell invoked?

The shell binary is at `proto/ish-shell`. It supports three invocation modes:

- **Inline:** `ish -c 'code'` — executes the code string and exits.
- **File:** `ish script.ish` — executes the file and exits.
- **Interactive:** `ish` — launches the Reedline REPL.

For acceptance tests, `-c` and stdin piping are both viable. The binary is built with `cargo build -p ish-shell` and located at `proto/target/debug/ish-shell` (or `release`). Tests will need to either build first or assume the binary is already built.

### Q: What features currently exist to test?

Based on the spec, architecture docs, and existing unit tests, the implemented features include:

- **Expressions:** arithmetic, comparison, logical (`and`/`or`/`not`), unary, string concatenation
- **Variables:** `let`, `let mut`, assignment
- **Control flow:** `if`/`else if`/`else`, `while`, `for..in`, `break`, `continue`
- **Functions:** `fn`, parameters, `return`, default parameters, closures, lambdas (`=>`)
- **Data structures:** object literals, list literals, property access, index access
- **Strings:** single-quoted, double-quoted with interpolation, triple-quoted, char literals, extended delimiters
- **Error handling:** `throw`, `try`/`catch`/`finally`, `with` blocks
- **Comments:** `//`, `#`, `/* */` (nested)
- **Shell mode:** command execution, pipes, redirection, `$()` capture, `>` force-command prefix
- **Compilation:** AST → Rust → `.so` → dynamic load
- **Self-hosting:** analyzer, generator, stdlib written as ish programs
- **Types:** type aliases, object types, union types, tuple types, function types
- **Modules:** `use`, `mod`, visibility modifiers
- **Assurance ledger:** `@standard[]`, `@[entry()]`, `standard` definitions, `entry type` definitions

The existing end-to-end demo (in `ish-shell`) verifies 6 things: factorial interpreted, factorial compiled, analyzer, generator, stdlib, and consistency between interpreted and compiled output.

---

## Feature 1: Bash-Based Acceptance Test Framework

### Issues to Watch Out For

1. **Binary location.** Tests need to find the `ish-shell` binary. Hardcoding `../target/debug/ish-shell` is fragile. Tests should accept an `ISH` environment variable or detect the binary relative to the test directory.

2. **Build dependency.** Should tests build the binary themselves, or require a pre-build step? Building inside each test file would be wasteful. A top-level runner that builds once then runs all tests is better.

3. **Exit code semantics.** The RFP says exit `-1` on failure. In Bash, exit codes are 0–255 (unsigned byte). `exit -1` is interpreted as `exit 255` by most shells. Using `exit 1` is more conventional; `exit 255` is unusual and could confuse CI tools. This needs a decision.

4. **Stdout vs. stderr.** The RFP says to capture stdout. Some ish errors may go to stderr. Tests need to decide whether to capture both or just stdout, and whether to test stderr content separately.

5. **Test isolation.** Bash tests share filesystem state. Tests that write files (e.g., compiled `.so` output) need cleanup. Tests should use temporary directories.

6. **Portability.** Bash is "universally available" on Linux and macOS but not on Windows. If Windows support is ever needed, these tests won't run natively there. This is acceptable for a prototype.

7. **Parallel execution.** A simple recursive runner will run tests sequentially. If the test suite grows large, parallel execution will be wanted. The flat Bash structure makes parallelism harder to add later.

8. **Cross-referencing maintenance burden.** Requiring every doc file to link to its test files and every test file to link to its doc creates a bidirectional coupling that will drift. Agents can maintain this, but it adds complexity to every doc or test change.

### Critical Analysis

The RFP proposes a specific structure: Bash scripts, hierarchical directories, ~10 tests per file, ~10 files per directory, recursive runner scripts. Let me evaluate this and alternatives.

#### Alternative A: Pure Bash (as proposed in RFP)

**Pros:**
- Zero dependencies beyond Bash and the ish binary
- Maximum readability for anyone familiar with shell scripting
- Each test file is self-contained and runnable independently
- Hierarchical structure scales naturally
- Test files double as executable documentation
- Easy for AI agents to generate and maintain

**Cons:**
- No built-in test runner features (filtering, parallel execution, timing, TAP/JUnit output)
- Bash string handling is error-prone (quoting, whitespace, special characters)
- No native support for test fixtures, setup/teardown, or test skipping
- Cross-platform concerns (Bash version differences, macOS ships Bash 3.2)
- Verbose — each test requires boilerplate for capture/compare/report
- Difficult to produce machine-readable output for CI integration

#### Alternative B: Bash with a Minimal Test Library

Same as Alternative A, but each test file sources a shared `test_lib.sh` that provides helper functions: `run_ish`, `assert_output`, `assert_exit_code`, `pass`, `fail`, `skip`. The test harness handles formatting, counting, and exit codes.

**Pros:**
- Retains all benefits of pure Bash
- Dramatically reduces boilerplate per test
- Consistent output format across all tests
- Easy to add TAP or JUnit output later by modifying only the library
- Helper functions serve as documentation of the test protocol
- Still zero external dependencies

**Cons:**
- Slightly less self-contained (each file depends on the library)
- The library itself must be maintained
- Still lacks native parallelism and filtering

#### Alternative C: BATS (Bash Automated Testing System)

[BATS](https://github.com/bats-core/bats-core) is a TAP-compliant testing framework for Bash. Tests are written as Bash functions with `@test` annotations.

**Pros:**
- Mature, well-maintained ecosystem
- Built-in TAP output for CI integration
- Supports setup/teardown, test filtering, parallel execution
- Active community and good documentation

**Cons:**
- External dependency (must be installed)
- BATS-specific syntax (`@test`, `run`, `[ "$status" -eq 0 ]`) is less immediately readable to non-BATS users
- Adds a layer between the reader and the actual ish invocations
- Conflicts with the "maximum readability" goal — readers need to understand BATS conventions

#### Alternative D: Rust Integration Tests

Write acceptance tests as Rust integration tests in a `proto/ish-tests` crate within the Cargo workspace. Each test spawns the ish-shell binary and checks output.

**Pros:**
- Native to the existing build system
- Parallel execution built in (`cargo test` runs tests in parallel by default)
- Rich assertion library (`assert_eq!`, custom matchers)
- Runs via the same `cargo test --workspace` command
- Strong string handling and no quoting issues
- Easy to produce JUnit XML for CI

**Cons:**
- **Violates readability goal** — Rust test boilerplate obscures the ish programs being tested
- Less approachable for non-Rust developers
- Compilation overhead for the test crate itself
- Harder for AI agents to quickly scan and understand what ish feature is being tested
- Does not serve as "executable documentation" the way a Bash script does

#### Alternative E: ish Self-Testing

Write acceptance tests in ish itself, testing language features by running ish programs that verify their own output using builtins.

**Pros:**
- Dogfooding — the language tests itself
- Maximum demonstration of language capabilities
- No external dependencies at all

**Cons:**
- **Circular dependency** — a bug in the interpreter could make tests pass incorrectly
- Cannot test the shell invocation modes (the test is already inside the shell)
- Cannot test compilation pipeline or error handling of invalid input
- Not viable until the language is more mature

#### Recommendation

**Alternative B (Bash with a minimal test library)** best balances the RFP's goals. It preserves the readability and self-contained nature of pure Bash while eliminating the boilerplate that would make tests tedious to write and maintain. The shared library is small (likely under 100 lines) and easy to understand.

Alternative D (Rust integration tests) could complement Bash tests for edge-case testing that benefits from programmatic string construction, but it should not replace the Bash tests as the primary acceptance test suite.

### Proposed Implementation

#### Directory Structure

```
proto/ish-tests/
├── run_all.sh                  # Top-level runner: builds ish, runs all groups
├── lib/
│   └── test_lib.sh             # Shared test helpers
├── basics/
│   ├── run_group.sh            # Runs all tests in this group
│   ├── variables.sh            # ~10 tests: let, let mut, assignment
│   ├── arithmetic.sh           # ~10 tests: +, -, *, /, %
│   ├── comparison.sh           # ~10 tests: ==, !=, <, >, <=, >=
│   ├── logical.sh              # ~10 tests: and, or, not
│   ├── strings.sh              # ~10 tests: single/double/triple/char/extended
│   ├── comments.sh             # ~5 tests: //, #, /* */
│   ├── data_structures.sh      # ~10 tests: objects, lists, access
│   └── type_declarations.sh    # ~10 tests: type aliases, object types, unions
├── control_flow/
│   ├── run_group.sh
│   ├── if_else.sh
│   ├── while_loop.sh
│   ├── for_loop.sh
│   └── break_continue.sh
├── functions/
│   ├── run_group.sh
│   ├── basic_functions.sh
│   ├── closures.sh
│   ├── lambdas.sh
│   └── default_params.sh
├── error_handling/
│   ├── run_group.sh
│   ├── throw_catch.sh
│   ├── try_finally.sh
│   └── with_blocks.sh
├── shell_mode/
│   ├── run_group.sh
│   ├── commands.sh
│   ├── pipes_redirection.sh
│   ├── command_substitution.sh
│   └── env_variables.sh
└── compilation/
    ├── run_group.sh
    ├── compile_and_run.sh
    └── self_hosting.sh
```

#### Test Library (`lib/test_lib.sh`)

```bash
#!/usr/bin/env bash
# Test library for ish acceptance tests.
# Source this file at the top of each test script.

TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Find the ish binary
ISH="${ISH:-$(dirname "${BASH_SOURCE[0]}")/../../../target/debug/ish-shell}"

assert_output() {
    local name="$1" expected="$2" actual="$3"
    TESTS_RUN=$((TESTS_RUN + 1))
    if [[ "$actual" == "$expected" ]]; then
        echo "  pass: $name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  FAIL: $name"
        echo "    expected: $(printf '%q' "$expected")"
        echo "    actual:   $(printf '%q' "$actual")"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

assert_exit_code() {
    local name="$1" expected="$2" actual="$3"
    TESTS_RUN=$((TESTS_RUN + 1))
    if [[ "$actual" -eq "$expected" ]]; then
        echo "  pass: $name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo "  FAIL: $name (expected exit $expected, got $actual)"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

run_ish() {
    "$ISH" -c "$1" 2>&1
}

run_ish_stdin() {
    "$ISH" <<'ISH_EOF'
$1
ISH_EOF
}

finish() {
    echo "$TESTS_PASSED/$TESTS_RUN passed"
    if [[ $TESTS_FAILED -gt 0 ]]; then
        exit 1
    fi
    exit 0
}
```

#### Example Test File (`basics/variables.sh`)

```bash
#!/usr/bin/env bash
# ---
# feature: Variables and Assignment
# docs: docs/spec/syntax.md#variables-and-expressions
# ---
# Tests variable declaration with let, mutable variables with let mut,
# and variable reassignment. Verifies that ish correctly handles
# immutable and mutable bindings.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../lib/test_lib.sh"

echo "=== Variables and Assignment ==="

# Test: let binding
output=$(run_ish 'let x = 5; println(x)')
assert_output "let binding" "5" "$output"

# Test: let mut and reassignment
output=$(run_ish 'let mut y = 10; y = 20; println(y)')
assert_output "let mut reassignment" "20" "$output"

# ... more tests ...

finish
```

#### Runner Script (`run_all.sh`)

```bash
#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../.."

echo "Building ish-shell..."
cargo build -p ish-shell

echo ""
echo "Running acceptance tests..."
TOTAL_PASS=0
TOTAL_FAIL=0

for group_runner in "$SCRIPT_DIR"/*/run_group.sh; do
    echo ""
    echo "--- $(basename "$(dirname "$group_runner")") ---"
    if bash "$group_runner"; then
        TOTAL_PASS=$((TOTAL_PASS + 1))
    else
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
done

echo ""
echo "=== Summary ==="
echo "Groups passed: $TOTAL_PASS"
echo "Groups failed: $TOTAL_FAIL"

if [[ $TOTAL_FAIL -gt 0 ]]; then
    exit 1
fi
```

#### Group Runner (`basics/run_group.sh`)

```bash
#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FAILURES=0

for test_file in "$SCRIPT_DIR"/*.sh; do
    [[ "$(basename "$test_file")" == "run_group.sh" ]] && continue
    echo ""
    if bash "$test_file"; then
        :
    else
        FAILURES=$((FAILURES + 1))
    fi
done

if [[ $FAILURES -gt 0 ]]; then
    exit 1
fi
```

### Decisions

**Decision:** Which alternative to adopt? (A) Pure Bash, (B) Bash with test library, (C) BATS, (D) Rust integration tests, (E) ish self-testing?
--> B

**Decision:** Exit code on failure: `exit 1` (conventional) or `exit 255` / `exit -1` (as stated in the RFP)?
--> exit 1

**Decision:** Should tests capture stderr separately from stdout, or merge them with `2>&1`?
--> merge them

**Decision:** Should `run_all.sh` build the binary automatically, or require a separate build step?
--> Build it automatically

---

## Feature 2: Hierarchical Directory Organization

### Issues to Watch Out For

1. **Premature hierarchy.** With the current feature set, there are roughly 6–8 feature groups. The "~10 files per directory, ~10 tests per file" guideline works well at this scale, giving a two-level hierarchy. A third level is not yet needed and should not be created speculatively.

2. **Group boundaries.** Some features cut across groups (e.g., string interpolation involves both strings and expressions). The test should live in the most specific group, with a comment noting the cross-cutting concern.

3. **Naming conventions.** File names should be `snake_case.sh`. Group directory names should be `snake_case`. Runner scripts should have a consistent name (`run_group.sh`).

### Critical Analysis

The proposed hierarchy is reasonable for the current project size. The key insight is that the hierarchy should grow organically — start with the groups that correspond to the spec's major sections, and split only when a group exceeds ~10 files.

An alternative would be a flat directory with a naming convention (`basics_variables.sh`, `basics_arithmetic.sh`, etc.) — simpler but harder to navigate at scale.

**Recommendation:** Adopt the hierarchical structure as proposed, starting with 6–7 groups matching the spec sections.

### Proposed Implementation

Initial groups, derived from the spec:

| Group | Covers | Estimated Files |
|-------|--------|-----------------|
| `basics/` | Variables, arithmetic, comparison, logical, strings, comments, data structures, types | 8 |
| `control_flow/` | if/else, while, for, break/continue | 4 |
| `functions/` | Functions, closures, lambdas, default params | 4 |
| `error_handling/` | throw/catch, try/finally, with blocks, defer | 4 |
| `shell_mode/` | Commands, pipes, redirection, capture, env vars | 5 |
| `compilation/` | Compile-and-run, self-hosting verification | 2–3 |

Total: ~27–30 test files, ~250–300 individual tests.

### Decisions

**Decision:** Adopt these initial groups, or reorganize differently?
--> Yes

---

## Feature 3: Metadata and Cross-Referencing

### Issues to Watch Out For

1. **Bidirectional link maintenance.** This is the highest-risk part of the proposal. Every spec doc change must update test links; every test change must update doc links. This will drift without tooling enforcement.

2. **Frontmatter format in Bash.** The RFP says "not actual frontmatter, since this needs to be in a Bash comment." A convention is needed. Using `# ---` delimiters inside a Bash comment block is the most readable approach.

3. **Link format.** Test files link to docs using relative paths. Docs files link to tests. The paths are in different parts of the tree, making relative links long and fragile.

### Critical Analysis

#### Alternative A: Manual Bidirectional Links (as proposed)

Both test files and doc files contain links to each other.

**Pros:**
- Self-documenting — looking at either file tells you the relationship
- No tooling needed to discover connections

**Cons:**
- High maintenance burden — changes to one side require updating the other
- Links will drift and become stale
- Increases the diff size for every test or doc change

#### Alternative B: Test → Doc Links Only (Unidirectional)

Test files link to their doc, but doc files do not link back to specific tests. Instead, a generated index (or a script that scans test metadata) produces the reverse mapping on demand.

**Pros:**
- Half the maintenance burden
- Doc files stay clean — no test infrastructure leaking into spec docs
- Reverse mapping is always accurate because it's generated from source of truth (the test metadata)

**Cons:**
- Requires running a script to find tests for a given doc
- Not visible when just reading the doc file

#### Alternative C: Mapping File

A single `test_map.md` or `test_map.json` file maintains the authoritative mapping between doc sections and test files. Both test metadata and doc cross-references are derived from (or verified against) this file.

**Pros:**
- Single source of truth
- Easy to audit completeness (every doc section should have ≥1 test)
- Can generate both directions programmatically

**Cons:**
- One more file to keep in sync
- Indirect — must look up the map to find connections

#### Recommendation

**Alternative B (unidirectional test → doc)** with a verification script. Each test file declares its doc reference in metadata. A script (similar to the existing `check-links.sh`) scans test metadata and verifies the referenced doc files exist. A second mode generates the reverse index. This keeps doc files clean and avoids the bidirectional drift problem.

### Proposed Implementation

#### Test Metadata Format

```bash
#!/usr/bin/env bash
# ---
# feature: Variables and Assignment
# docs: docs/spec/syntax.md
# section: Variables and Expressions
# ---
```

#### Verification Script (`docs/scripts/check-test-refs.sh`)

Scans all `proto/ish-tests/**/*.sh` files, extracts `# docs:` lines, and verifies the referenced doc files exist. Reports orphaned tests (no doc ref) and untested docs (no test references them).

### Decisions

**Decision:** Bidirectional links (A), unidirectional test→doc (B), or mapping file (C)?
--> Bidirectional links.

**Decision:** Should the doc files have a `## Acceptance Tests` section listing related tests, or should this be generated/optional?
--> The doc files have a `## Acceptance Tests` section listing related tests.  It is important for humans to have a convenient way to find the tests related to a feature.

---

## Feature 4: Documentation and Tooling Updates

### Issues to Watch Out For

1. **AGENTS.md update.** The build/test commands in AGENTS.md need to include acceptance test execution.
2. **CONTRIBUTING.md update.** Contributors need guidance on writing and maintaining acceptance tests.
3. **Agent skills.** A new skill or playbook may be needed for "add acceptance test for feature X."

### Critical Analysis

This is straightforward. The key files to update are:

- `AGENTS.md` — add `proto/ish-tests` to the project structure table; add a run command for acceptance tests
- `CONTRIBUTING.md` — add a section on acceptance test conventions
- `docs/architecture/overview.md` or a new `docs/architecture/testing.md` — describe the test infrastructure

A new agent skill (`add-acceptance-test`) would be valuable but should be deferred until the test framework is established and the patterns are clear.

### Proposed Implementation

1. Add to `AGENTS.md` project structure table:

   | `proto/ish-tests/` | Bash acceptance test suite |

2. Add to `AGENTS.md` build & test section:

   ```bash
   cd proto && bash ish-tests/run_all.sh   # Run acceptance tests
   ```

3. Add to `CONTRIBUTING.md`:
   - Acceptance test conventions (metadata format, naming, organization)
   - Requirement to add/update acceptance tests when adding features or changing behavior

4. Add `## Acceptance Tests` section to spec doc files that have corresponding tests (if bidirectional linking is chosen).

### Decisions

**Decision:** Create a new `docs/architecture/testing.md` or add a testing section to `docs/architecture/overview.md`?
--> Create a new `docs/architecture/testing.md`

**Decision:** Create an agent skill for acceptance test maintenance now or defer?
--> Create a skill for adding an acceptance test, and also a skill for an acceptance test audit, which 1. Runs all the tests to see which ones are failing; 2. Checks all test -> doc links for orphans; 3. Checks all do -> test links for orphans; 4. Checks for untested features in the docs; and 5. Checks for discrepancies between documented behavior and tested behavior.

---

## Documentation Updates

The following documentation files will be affected by this proposal:

- [AGENTS.md](../../AGENTS.md) — project structure table, build/test commands
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — testing conventions
- [docs/architecture/overview.md](../architecture/overview.md) — or new `testing.md`
- [docs/spec/syntax.md](../spec/syntax.md) — if bidirectional test links are adopted
- [docs/spec/types.md](../spec/types.md) — if bidirectional test links are adopted
- [docs/spec/modules.md](../spec/modules.md) — if bidirectional test links are adopted
- [docs/spec/execution.md](../spec/execution.md) — if bidirectional test links are adopted
- [docs/project/roadmap.md](roadmap.md) — acceptance test milestone
- [docs/project/maturity.md](maturity.md) — testing maturity update

Remember to update `## Referenced by` sections in all modified files.

---

## History Updates

- [ ] Add `docs/project/history/2026-03-16-acceptance-tests.md`
- [ ] Update `docs/project/history/INDEX.md`

---

## Referenced by

- [docs/project/proposals/INDEX.md](INDEX.md)
- [docs/project/rfp/acceptance-tests.md](../rfp/acceptance-tests.md)
