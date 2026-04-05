---
title: "Plan: Agent Pipeline Skills"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-04-04
depends-on:
  - docs/project/proposals/agent-pipeline-skills.md
  - .claude/skills/revise/SKILL.md
  - .claude/skills/plan-implementation/SKILL.md
  - .claude/skills/implement/SKILL.md
  - GLOSSARY.md
  - docs/ai-guide/orientation.md
---

# Plan: Agent Pipeline Skills

*Derived from [agent-pipeline-skills.md](../../proposals/agent-pipeline-skills.md) on 2026-04-04.*

## Overview

This plan implements the eight features from the Agent Pipeline Skills proposal: gap detection and split evaluation in the Revise skill; phase-directory output and context files in the Plan skill; guard rails in the Implement skill; three new skills (Bug Fix, Troubleshoot, Verify); a backchannel clarification-request mechanism; and terminology additions to the glossary.

All changes are to documentation and skill procedure files — no code changes.

## Requirements

1. The Revise skill checks each feature section against the gap detection checklist after incorporating the punch list, injecting open questions where gaps are found.
2. The Revise skill evaluates whether a revised proposal should be split (threshold: ≥ 10 independently implementable steps) and presents a split decision if so.
3. The Plan skill produces a directory with phase files when the plan has > 5 implementation steps; a single file for ≤ 5 steps.
4. Each phase file contains context files (verbatim extracts), tasks, and a verification step.
5. Context files contain verbatim extracts — no summarizing or paraphrasing.
6. Before saving, the Plan skill scrutinizes each phase for completeness; if incomplete, it writes a clarification request and stops.
7. The Implement skill stops at the first unclear or impossible task and writes a clarification request.
8. The Implement skill handles both single-file and directory plan formats.
9. The Implement skill invokes `/verify` after each phase in a directory plan.
10. The Bug Fix skill (new) fixes a confirmed bug across all artifact types, proceeding only after root cause is identified.
11. The Troubleshoot skill (new) identifies root cause without making behavioral changes.
12. The Verify skill (new) checks changed artifacts for internal consistency after each phase or bug fix.
13. Blocked agents write clarification requests to `docs/project/clarifications/<date>-<slug>.md`.
14. An index for clarification requests exists at `docs/project/clarifications/INDEX.md`.
15. All skills append a status line to output: `Status: completed | blocked (see clarifications/<date>-<topic>.md)`.
16. GLOSSARY.md contains six new terms: Role, Skill, Agent, Backchannel, Clarification request, Phase.
17. `docs/ai-guide/orientation.md` reflects the eight-role pipeline and current skill file locations.

## Phase Dependency Graph

```
Phase 1 (Vocabulary) → Phase 2 (Revise) → Phase 6 (Wrap-up)
                     → Phase 3 (Plan)   ↗
                     → Phase 4 (Implement) ↗
                     → Phase 5 (New skills) ↗
```

Phases 2–5 are independent of each other. All depend on Phase 1 (GLOSSARY must exist before skills reference new terms). Phase 6 depends on all prior phases.

## Implementation Order

1. [Phase 1: Vocabulary and Documentation](phase-1.md) — GLOSSARY.md, AI guide
2. [Phase 2: Revise Skill](phase-2.md) — gap detection, split evaluation
3. [Phase 3: Plan Skill](phase-3.md) — phase directories, context files, scrutiny
4. [Phase 4: Implement Skill](phase-4.md) — guard rails, phase-directory handling, Verify invocation
5. [Phase 5: New Skills and Infrastructure](phase-5.md) — Bug Fix, Troubleshoot, Verify skills; clarifications index
6. [Phase 6: Wrap-up](phase-6.md) — roadmap, history, index files

## Context Files

- [context/terminology.md](context/terminology.md) — Terminology Canonicalization feature (verbatim)
- [context/revise-skill.md](context/revise-skill.md) — Revise Skill feature (verbatim)
- [context/plan-skill.md](context/plan-skill.md) — Plan Skill feature (verbatim)
- [context/implement-skill.md](context/implement-skill.md) — Implement Skill feature (verbatim)
- [context/bug-fix-skill.md](context/bug-fix-skill.md) — Bug Fix Skill feature (verbatim)
- [context/troubleshoot-skill.md](context/troubleshoot-skill.md) — Troubleshoot Skill feature (verbatim)
- [context/verify-skill.md](context/verify-skill.md) — Verify Skill feature (verbatim)
- [context/backchannel.md](context/backchannel.md) — Backchannel Communication feature (verbatim)

## Reference

- Skills are in `.claude/skills/` (not `.agents/skills/` — that directory does not exist).
- The six-skill list in `docs/ai-guide/orientation.md` (line 86) reads: `/propose`, `/revise`, `/accept`, `/plan-implementation`, `/implement`, `/audit`. It must be updated to eight skills: add `/bug-fix`, `/troubleshoot`, `/verify`. Also update `.github/skills/` reference to `.claude/skills/`.
- Clarifications directory `docs/project/clarifications/` does not yet exist.
- GLOSSARY.md table is alphabetically sorted — insert the six new terms in the correct alphabetical positions.
- Roadmap entry for this work: check `docs/project/roadmap.md` for any matching entry; if none, none needs to be added (this is a meta-project change, not a language feature).
