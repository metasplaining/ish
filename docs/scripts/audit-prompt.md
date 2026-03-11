---
title: Documentation Audit Prompt
category: project
audience: [ai-agent]
status: draft
last-verified: 2026-03-10
depends-on: [GLOSSARY.md, CONTRIBUTING.md]
---

# Documentation Audit Prompt

Use this prompt (or adapt it) when asking an AI agent to audit the ish documentation.

---

## Full Audit Prompt

```
Perform a documentation audit of the ish project. Check the following:

1. **Broken links**: Run `bash docs/scripts/check-links.sh` and fix any broken cross-references.

2. **Frontmatter**: Run `bash docs/scripts/check-frontmatter.sh` and fix any missing or malformed YAML frontmatter.

3. **Stale docs**: Run `bash docs/scripts/check-stale.sh 90` and review any documents not verified in the last 90 days. Update `last-verified` if the content is still accurate, or flag for revision.

4. **Glossary coverage**: Run `bash docs/scripts/check-glossary.sh` and add any missing domain terms to GLOSSARY.md.

5. **Single source of truth**: Search for duplicated normative claims (definitions, rules, constraints) that appear in more than one file. Each normative claim should appear in exactly one spec file.

6. **Backward references**: Check that every file's `## Referenced by` section is accurate. If file A links to file B, then file B's Referenced-by section should list file A.

7. **Open question consistency**: Verify that every open question in `docs/project/open-questions.md` has a corresponding entry in the relevant spec file, and vice versa.

8. **500-line limit**: Check for files exceeding 500 lines and flag them for splitting.

Report findings as a list of issues with file paths and recommended fixes.
```

---

## Quick Audit (Links + Frontmatter Only)

```
Run these commands and fix any issues:
  bash docs/scripts/check-links.sh
  bash docs/scripts/check-frontmatter.sh
```

---

## Referenced by

- [docs/INDEX.md](../INDEX.md)
- [CONTRIBUTING.md](../../CONTRIBUTING.md)
