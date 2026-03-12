---
title: "History: Documentation Conventions Cleanup"
category: history
audience: [human-dev, ai-agent]
status: stable
last-verified: 2026-03-11
depends-on: [CONTRIBUTING.md, AGENTS.md, docs/project/decisions/001-documentation-structure.md]
---

# 2026-03-11: Documentation Conventions Cleanup

A follow-up to the documentation system rollout from the day before, addressing rough edges that emerged once the system was in use.

## What Prompted This

The documentation system created on 2026-03-10 worked well structurally, but a day of use revealed several problems:

1. **The history file was too terse.** The history entry for the documentation system itself read like an agent-generated summary — bullet points and facts, but none of the story. The proposal/response/decision exchange that shaped the documentation system was not captured. Someone reading the history would know *what* was decided, but not *how* the ideas evolved or *why* specific alternatives were chosen.

2. **Some documentation was hallucinated.** When the AI developer created the documentation structure, it generated content for every planned file — including the user guide, AI guide, comparison page, and roadmap. But these files were written without sufficient specification. The content was plausible-sounding but not grounded in actual design decisions. There was no way to distinguish these non-normative placeholder files from legitimate drafts.

3. **No mechanism for structured proposals.** The prompt-file workflow (human writes a file with questions and instructions, AI reads it and acts) worked well, but the AI would jump straight to implementation. There was no step where the AI produced a plan with critical analysis and alternatives for human review before writing code.

## The Changes

### New Document Status: `placeholder`

The existing status values (`draft`, `review`, `stable`) didn't adequately describe files that were generated without specification. A `draft` implies someone started writing real content; these files were closer to hallucinated scaffolding.

A new status was added: **`placeholder`** — *"Generated without sufficient specification. Non-normative — content may be entirely hallucinated. Must be reviewed and rewritten before promotion to `draft`."*

This is deliberately harsher than `draft`. It signals that the content cannot be relied upon at all — it exists only to hold a place in the structure.

Nineteen files were marked `placeholder`:
- All 9 files in `docs/user-guide/` (INDEX + 8 topic files)
- All 7 files in `docs/ai-guide/` (INDEX + 6 topic files)
- `docs/comparison.md`
- `docs/project/roadmap.md`
- `docs/errors/INDEX.md`

The status was added to CONTRIBUTING.md's document statuses table, to the frontmatter template, and to AGENTS.md's conventions section.

### History Writing Convention

The problem with the first history file wasn't just that it was terse — there was no convention saying it shouldn't be. The contributing guidelines said to "add a history file" but didn't specify what should be in it.

Two changes codified the expectation:

1. **CONTRIBUTING.md**, Change Protocol step 6, now reads: *"History files are written for a human audience — use narrative prose that tells the story of what changed and why, showing the proposal → feedback → decision flow where applicable. Do not write terse agent-style summaries."*

2. **AGENTS.md** gained a new convention: *"History files are written for humans. Use narrative prose that tells the story of what happened, including the evolution of ideas through proposal/response/decision exchanges. Do not write terse summaries or bullet-point logs."*

The existing history file (`2026-03-10-documentation-system.md`) was then rewritten to follow this convention, expanding from a terse summary into a full narrative of the proposal/decision exchange.

### The `/propose` Skill

A Copilot skill was created at `.github/skills/propose/SKILL.md` that reads a prompt file and produces a structured proposal. The output includes:

- Answers to any questions in the input file
- For each feature: issues to watch out for, critical analysis with alternatives and pros/cons, proposed implementation details, and a blank decisions section
- Reminders to update documentation and history

Proposals are saved to `docs/project/proposals/`. A proposals INDEX was created to track them.

The skill location was initially requested as `docs/skills/` but was moved to `.github/skills/` after researching Copilot's skill discovery mechanism — only skills under `.github/skills/`, `.agents/skills/`, or `.claude/skills/` are auto-discovered as slash commands.

## Participants

- **Human developer:** Identified the three problems (terse history, hallucinated docs, missing proposal workflow). Specified the requirements for each fix. Made decisions on skill location (`.github/skills/` after the discoverability finding), proposal output location (`docs/project/proposals/`), confirmed including `docs/errors/INDEX.md` in the placeholder batch.

- **AI developer:** Created the plan, researched Copilot skill discovery to correct the skill location, implemented all changes: convention updates to CONTRIBUTING.md and AGENTS.md, status changes across 19 files, history file rewrite, skill creation, and proposals directory setup.

---

## Referenced by

- [docs/project/history/INDEX.md](INDEX.md)
