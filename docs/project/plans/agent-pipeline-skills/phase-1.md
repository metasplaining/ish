---
title: "Phase 1: Vocabulary and Documentation"
category: plan-phase
audience: [ai-dev]
status: ready
last-verified: 2026-04-04
---

# Phase 1: Vocabulary and Documentation

*Part of: [agent-pipeline-skills/overview.md](overview.md)*

## Context Files

- [context/terminology.md](context/terminology.md) — six new GLOSSARY terms with definitions

## Requirements

- GLOSSARY.md contains six new terms: Role, Skill, Agent, Backchannel, Clarification request, Phase — each with the exact definition from the proposal.
- `docs/ai-guide/orientation.md` describes eight pipeline roles (not six) and references `.claude/skills/` (not `.github/skills/`).

## Tasks

- [ ] 1. Add six terms to GLOSSARY.md — `GLOSSARY.md`

  Insert alphabetically into the existing table. Exact definitions in [context/terminology.md](context/terminology.md).

  Insertion positions (all alphabetically between existing entries):
  - **Agent** — between "Audit" and "Authority order"
  - **Backchannel** — between "Await" and "Bare-word invocation"
  - **Clarification request** — between "Codec" and "CodedError"
  - **Phase** — between "Parallel shim" and "Planning phase"
  - **Role** — between "Runtime crate" and "Shell mode" (note: alphabetically R comes before S)
  - **Skill** — between "Shell thread" and "Spawn"

- [ ] 2. Update `docs/ai-guide/orientation.md` — `docs/ai-guide/orientation.md`

  a. In the §Proposal Process section (around line 86), update the skill list from six skills to eight:
     - Change: `/propose`, `/revise`, `/accept`, `/plan-implementation`, `/implement`, `/audit`
     - To: `/propose`, `/revise`, `/accept`, `/plan-implementation`, `/implement`, `/bug-fix`, `/troubleshoot`, `/verify`, `/audit`

  b. In the same sentence, change `.github/skills/` to `.claude/skills/`.

  c. Update "Six skills support this lifecycle" to "Nine skills support this lifecycle" (or reword to avoid the count).

## Verification

Run: `grep -n "Role\|Backchannel\|Clarification request\|Phase\b" /home/dan/git/ish/GLOSSARY.md`
Check: All six terms appear with their definitions.

Run: `grep -n "bug-fix\|troubleshoot\|verify\|\.claude/skills" /home/dan/git/ish/docs/ai-guide/orientation.md`
Check: New skills are listed; `.claude/skills/` appears instead of `.github/skills/`.

Invoke: `/verify agent-pipeline-skills/phase-1.md`

Status: completed | blocked (see clarifications/<date>-<topic>.md)
