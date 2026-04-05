---
name: audit
description: 'Run a feature coherence audit across all project artifacts. Use when: you need to verify that all documentation, tests, and code are consistent for a given feature. Trigger words: audit, coherence check, consistency check, verify artifacts.'
argument-hint: 'Feature name or path to implementation plan'
---

# Audit

Run a feature coherence audit — a cross-check of all project artifacts related to a single feature, verifying consistency.

## When to Use

- After implementing a feature, to verify all artifacts are consistent
- When a user suspects documentation drift or inconsistencies
- As part of an implementation plan's checkpoint steps
- The user invokes `/audit <feature-name>`

## Procedure

1. **Identify the feature to audit** (from user input or implementation plan).

2. **For each artifact type in the checklist:**
   a. Search for files related to the feature (keyword search, `depends-on` traversal).
   b. Read relevant sections.
   c. Extract claims about the feature (behavior, status, syntax, etc.).

3. **Cross-check claims across artifact types:**
   a. Spec vs. acceptance tests: do tests cover all specified behaviors?
   b. Spec vs. code: does the implementation match the spec?
   c. Roadmap vs. actual: is the status accurate?
   d. User guide vs. spec: does the guide accurately describe the feature?
   e. AI guide vs. spec: same check.
   f. Glossary: are all terms used consistently?
   g. AGENTS.md / skills: is the feature reflected in agent documentation?

4. **Distinguish between** "not yet implemented" (expected for planned features) and "described differently in two places" (an inconsistency).

5. **Report inconsistencies and missing coverage.**

6. **Save the audit report** to a file that the human can review and delete.

## Report Format

```markdown
# Feature Coherence Audit: <Feature Name>

*Audited on <date>.*

## Summary
- Artifacts checked: <N>
- Consistent: <N>
- Inconsistent: <N>
- Missing coverage: <N>

## Glossary
**Status:** ✅ Consistent / ⚠️ Inconsistent / ❌ Missing
<details>

## Roadmap
**Status:** ✅ / ⚠️ / ❌
<details>

## Specification
**Status:** ✅ / ⚠️ / ❌
<details>

## Architecture
**Status:** ✅ / ⚠️ / ❌
<details>

## User Guide
**Status:** ✅ / ⚠️ / ❌
<details>

## AI Guide
**Status:** ✅ / ⚠️ / ❌
<details>

## Agent Documentation
**Status:** ✅ / ⚠️ / ❌
<details>

## Acceptance Tests
**Status:** ✅ / ⚠️ / ❌
<details>

## Code
**Status:** ✅ / ⚠️ / ❌
<details>
```
