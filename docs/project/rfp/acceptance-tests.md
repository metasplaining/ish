---
title: "RFP: Acceptance Tests"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-16
depends-on: [docs/architecture/shell.md, docs/spec/syntax.md, AGENTS.md, CONTRIBUTING.md]
---

# RFP: Acceptance Tests

*Cleaned-up version of the `acceptance_tests` prompt file.*

---

## Proposed Test Infrastructure

ish should have acceptance tests in the `proto/ish-tests` directory.

The acceptance tests should be written in Bash. Bash is universally available and is an adequate test harness for running ish.

### Test Organization

The tests should be organized such that each test file contains approximately ten tests, testing one feature. Test files should be grouped into directories each containing approximately ten test files, for a group of related features. This hierarchical grouping continues upward until there are enough directories for all the tests needed.

### Test Structure

Each test should be structured as follows:

1. Docs
2. Invoke ish shell, passing either an inline program with the `-c` argument or a program to stdin with a here document
3. Capture stdout and return code
4. Check stdout and return code against the expected value
5. Print the test name and either "pass" or "fail"

The test file should exit with `0` if all tests pass, or `-1` if any fail.

### Runner Scripts

Each non-leaf directory should have a script that runs all the test files in its subdirectories.

### Metadata and Cross-Referencing

The test file should contain a metadata section with frontmatter-like information (not actual YAML frontmatter, since this needs to be in a Bash comment) as well as a link to the relevant file in `/docs` for the feature under test. The docs section of each test should briefly describe the feature being tested and the expected behavior. Files in `/docs` should have a link to each of the test files associated with them.

### Use Cases

This test structure supports several use cases:

1. Run all the tests, determine which features are not working. For non-working features, find the documentation of their expected behavior.
2. Examine all the documented features. For each feature, inspect the test to see if it agrees with the docs.
3. Read the docs on a specific feature. Run the associated test to see that feature in action.

### Readability Goal

Generally speaking, the acceptance tests should be written for maximum readability. They are intended to demonstrate all the features and show that the features are actually implemented and that the behavior matches the documentation. They are not intended to be exhaustive tests of every edge case.

### Documentation and Tooling Updates

The project documentation should be updated to explain the project structure. The relevant skills and agent documentation should be updated so that agents keep the acceptance tests, and the doc-to-test mapping, up to date whenever either the documentation or code changes.

---

## Request for Alternatives

Propose alternative acceptance test structures, and how they might better accomplish the acceptance test objectives.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
