---
title: "Plan: Proposal Process Improvements"
category: plan
audience: [ai-dev]
status: completed
last-verified: 2026-03-18
depends-on: [docs/project/proposals/proposal-process-improvements.md, GLOSSARY.md, AGENTS.md, CONTRIBUTING.md]
---

# Plan: Proposal Process Improvements

*Derived from [proposal-process-improvements.md](../proposals/proposal-process-improvements.md) on 2026-03-18.*

## Overview

Implement the three-document proposal lifecycle (RFP → Design Proposal → Implementation Plan) with six supporting agent skills, authority-ordered execution, and cross-session continuity. This replaces the existing single-skill propose workflow with a structured, multi-phase process enforced by agent instructions and skill definitions.

## Requirements

Extracted from the 20 accepted decisions and 7 features. Each is a testable statement.

### Lifecycle (Decisions 1–3)

- R1: Three artifact types exist: RFP, Design Proposal, Implementation Plan.
- R2: Implementation plans are stored in `docs/project/plans/`.
- R3: An RFP always exists. The agent decides whether to produce a design proposal or skip directly to an implementation plan.
- R4: Design proposals iterate via complete replacement. Only one version exists on disk. Prior versions are preserved in the design history directory.

### Authority Order (Decisions 4–5)

- R5: A 12-step authority order is defined and documented.
- R6: Authority order is enforced by agent instructions only (no tooling).

### Terminology (Decision 6)

- R7: Eleven new terms are defined in the glossary: prompt, RFP, design proposal, implementation plan, design history, punch list, decision register, design phase, planning phase, implementation phase, authority order, feature coherence audit.

### Batch Protocol (Decision 7)

- R8: The revise skill produces complete replacement proposals. Prior versions go to the design history directory.
- R9: Both separate-document and inline (`-->`) punch lists are supported.

### Skills (Decisions 8–14, 17)

- R10: Six skills exist: `/propose` (updated), `/revise` (new), `/accept` (new), `/plan` (new), `/implement` (new), `/audit` (new).
- R11: `/propose` includes a Decision Register section in its output.
- R12: `/revise` is a separate skill that handles inline punch lists and removes stale implementation plans.
- R13: `/accept` finalizes proposals — records all decisions, removes open prompts, sets status to accepted.
- R14: `/plan` runs `/accept` first if the proposal is not yet accepted.
- R15: `/implement` follows authority order with checkpoint verification.
- R16: `/audit` uses a hybrid approach (checklist + dependency + search). Reports saved to a file.
- R17: Implementation plans include audit steps.

### Quality Safeguards (Decision 18)

- R18: Every design proposal has a Decision Register section.
- R19: After generating or revising a proposal, the agent scans for internal contradictions.

### Cross-Session Continuity (Decision 19)

- R20: `.github/copilot-instructions.md` instructs agents to resume from the first uncompleted TODO item.
- R21: The `/implement` skill marks TODO items in-progress and completed as it executes.

### Design History (Decision 20)

- R22: Each proposal's history is a directory under `docs/project/history/` containing `summary.md` and version files.

## Authority Order

Update artifacts in this order. The implementing agent must complete each step before moving to the next.

1. GLOSSARY.md (new terms)
2. Roadmap (status → "in progress")
3. Specification docs
4. Architecture docs
5. User guide / AI guide
6. Agent documentation (AGENTS.md, skills, copilot-instructions.md)
7. Acceptance tests
8. Code (implementation)
9. Unit tests
10. Roadmap (status → "completed")
11. History
12. Index files

## TODO

### Phase 1: Glossary and Roadmap

- [x] 1. **Add glossary terms** — `GLOSSARY.md`
  Add 11 new terms in alphabetical order within the existing table:
  - **Authority order** — The defined sequence in which project artifacts must be updated during implementation, from most authoritative (glossary, roadmap) to least authoritative (index files).
  - **Decision register** — A consolidated list of all accepted decisions in a design proposal. Maintained at the top of the proposal as the authoritative reference.
  - **Design history** — Narrative of the deliberation process: what was proposed, what alternatives were considered, what was decided, and by whom. Stored as a directory per proposal under `docs/project/history/`, with a summary file and separate files for each version.
  - **Design phase** — The iterative process of creating and refining a design proposal from an RFP. Ends when the human accepts the design proposal.
  - **Design proposal** — Analysis of an RFP with alternatives, pros/cons, recommendations, and decisions. Output of the design process. May go through multiple iterations.
  - **Feature coherence audit** — A cross-check of all project artifacts related to a single feature, verifying consistency.
  - **Implementation phase** — The process of executing an implementation plan. Ends when all TODO items are complete.
  - **Implementation plan** — Consolidated, authoritative document derived from the accepted design proposal. Contains the TODO list and file-by-file changes. Input to the implementation process. Stored in `docs/project/plans/`.
  - **Planning phase** — The process of generating an implementation plan from an accepted design proposal. Typically one step, not iterative.
  - **Prompt** — Raw input from the human, before any cleanup. Not a project artifact.
  - **Punch list** — A list of corrections, additions, or changes that the human wants made to a design proposal before accepting it. May be delivered as a separate document or as inline decisions in the proposal itself.
  Update `last-verified` date.
  Verify: all 11 terms present, alphabetically sorted within the table.

- [x] 2. **Add roadmap entry** — `docs/project/roadmap.md`
  Add under "In Progress":
  `- [ ] Proposal process improvements (three-document lifecycle, six skills, authority order)`
  Verify: entry appears under "In Progress".

---

**CHECKPOINT A:** Glossary terms exist and roadmap shows "in progress." Read both files and verify against R5, R7.

---

### Phase 2: Project Documentation

- [x] 3. **Update CONTRIBUTING.md** — `CONTRIBUTING.md`
  Changes needed:
  - In the existing "RFPs" subsection under Documentation Conventions, expand to describe the full three-document lifecycle (RFP → Design Proposal → Implementation Plan).
  - Add a new subsection "Authority Order" documenting the 12-step sequence.
  - Add a new subsection "Proposals and Plans" describing:
    - Design proposals in `docs/project/proposals/`
    - Implementation plans in `docs/project/plans/`
    - Design history directories in `docs/project/history/`
    - The iteration protocol (complete replacement, decision register, self-consistency check)
  - Update terminology: use "design proposal" where "proposal" is used alone, where the context is about the three-document lifecycle.
  - Update the history file convention to describe directories (not single files).
  Verify: search CONTRIBUTING.md for "authority order", "implementation plan", "design proposal", "design history" — all present.

- [x] 4. **Update AI guide orientation** — `docs/ai-guide/orientation.md`
  Changes needed:
  - Add a section describing the proposal process lifecycle.
  - Reference the six skills.
  - Add "Proposal Process" to the "Where to Find Things" table.
  Verify: orientation.md mentions the three-document lifecycle.

---

**CHECKPOINT B:** CONTRIBUTING.md and orientation.md describe the new process. Verify R1, R2, R5 are documented.

---

### Phase 3: Agent Documentation and Skills

- [x] 5. **Create `.github/copilot-instructions.md`** — `.github/copilot-instructions.md` (new file)
  Content as specified in the proposal's Feature 5 "Copilot Instructions File" section. Must include:
  - Proposal process overview (4-step summary)
  - Full 12-step authority order
  - Implementation discipline rules
  - Resuming implementation instructions
  Verify: file exists, contains "Authority Order" with all 12 steps.

- [x] 6. **Update `/propose` skill** — `.github/skills/propose/SKILL.md`
  Changes needed:
  - Update description to reference "design proposal" terminology.
  - Add Decision Register section to the output template (after the version/reference line, before features).
  - Update the History Updates section to reference the design history directory convention (directory per proposal, summary.md + version files).
  - Update step 7 to describe the directory-based history convention.
  Verify: output template includes `## Decision Register` section.

- [x] 7. **Create `/revise` skill** — `.github/skills/revise/SKILL.md` (new file)
  YAML frontmatter: `name: revise`, description with trigger words ("revise", "punch list", "update proposal"), argument-hint for proposal path.
  9-step procedure as specified in the proposal's Feature 5 `/revise` section.
  Verify: file exists with correct procedure.

- [x] 8. **Create `/accept` skill** — `.github/skills/accept/SKILL.md` (new file)
  YAML frontmatter: `name: accept`, description with trigger words ("accept", "finalize proposal"), argument-hint for proposal path.
  10-step procedure as specified in the proposal's Feature 5 `/accept` section.
  Verify: file exists with correct procedure.

- [x] 9. **Create `/plan` skill** — `.github/skills/plan/SKILL.md` (new file)
  YAML frontmatter: `name: plan`, description with trigger words ("plan", "implementation plan", "ready to implement"), argument-hint for proposal path.
  8-step procedure as specified in the proposal's Feature 5 `/plan` section.
  Include the implementation plan output template from Feature 1.
  Verify: file exists with correct procedure and output template.

- [x] 10. **Create `/implement` skill** — `.github/skills/implement/SKILL.md` (new file)
  YAML frontmatter: `name: implement`, description with trigger words ("implement", "build", "execute plan"), argument-hint for plan path.
  7-step procedure as specified in the proposal's Feature 5 `/implement` section.
  Verify: file exists with correct procedure.

- [x] 11. **Create `/audit` skill** — `.github/skills/audit/SKILL.md` (new file)
  YAML frontmatter: `name: audit`, description with trigger words ("audit", "coherence check", "consistency check"), argument-hint for feature name or plan path.
  6-step procedure and audit report format as specified in the proposal's Feature 5 `/audit` section.
  Verify: file exists with correct procedure and report template.

- [x] 12. **Update AGENTS.md** — `AGENTS.md`
  Changes needed:
  - Add new task playbooks to the Task Playbooks table:
    - "Creating a design proposal" → load `.github/skills/propose/SKILL.md`
    - "Revising a design proposal" → load `.github/skills/revise/SKILL.md`
    - "Accepting a design proposal" → load `.github/skills/accept/SKILL.md`
    - "Creating an implementation plan" → load `.github/skills/plan/SKILL.md`
    - "Implementing a plan" → load `.github/skills/implement/SKILL.md`
    - "Running a feature audit" → load `.github/skills/audit/SKILL.md`
  - Add authority order reference under Key Concepts or a new section.
  - Update terminology to use "design proposal" and "implementation plan."
  Verify: all six skills listed in Task Playbooks table.

---

**CHECKPOINT C:** All six skills exist. copilot-instructions.md exists. AGENTS.md references all skills. Verify R10–R21 by reading each skill file and confirming procedures match the proposal.

---

### Phase 4: Feature Coherence Audit

- [x] 13. **Run feature coherence audit** — (no file; output to review file)
  Audit the "Proposal Process Improvements" feature across all artifacts:
  - Glossary: all 11 terms present and consistent?
  - Roadmap: status correct?
  - CONTRIBUTING.md: lifecycle, authority order, terminology consistent with proposal?
  - AI guide: references correct?
  - All 6 skill files: procedures match proposal?
  - copilot-instructions.md: authority order matches?
  - AGENTS.md: playbooks cover all skills?
  Save report. Fix any inconsistencies found.

---

**CHECKPOINT D:** Audit report shows all artifacts consistent. Fix any issues before proceeding.

---

### Phase 5: Finalization

- [x] 14. **Update roadmap to completed** — `docs/project/roadmap.md`
  Move the proposal process entry from "In Progress" to "Completed":
  `- [x] Proposal process improvements (three-document lifecycle, six skills, authority order)`
  Verify: entry appears under "Completed".

- [x] 15. **Update history** — `docs/project/history/2026-03-18-proposal-process-improvements/summary.md`
  Append an implementation narrative describing what was done.

- [x] 16. **Update index files** — Multiple files
  - `docs/project/history/INDEX.md` — verify entry is current.
  - `docs/project/proposals/INDEX.md` — verify status is "accepted".
  - `docs/ai-guide/INDEX.md` — add entry if new AI guide files were created.
  - `docs/INDEX.md` — add `docs/project/plans/` to the documentation map if not already present.
  - `docs/project/plans/INDEX.md` — create index for the plans directory (new file).
  Update `## Referenced by` sections in all files that gained new inbound references.
  Verify: all index files reference the new artifacts.

---

**CHECKPOINT E (Final):** All TODO items complete. Run final audit. All artifacts consistent. Implementation plan status set to "completed".

---

## Reference

### Existing File Locations

Files that will be modified (current paths for the implementing agent):

| File | Current State |
|------|---------------|
| `GLOSSARY.md` | 89 terms in alphabetical table. New terms insert alphabetically. |
| `CONTRIBUTING.md` | Has "RFPs" subsection under Documentation Conventions. No authority order or plans section. |
| `AGENTS.md` | Has Task Playbooks table with 8 entries. Key Concepts section with 5 bullets. |
| `.github/skills/propose/SKILL.md` | 8-step procedure. Output template has no Decision Register section. History section references single file, not directory. |
| `docs/ai-guide/orientation.md` | Status: placeholder. Has "Common AI Tasks" section. No proposal process section. |
| `docs/project/roadmap.md` | Status: placeholder. "In Progress" section has 3 items. |
| `docs/project/proposals/INDEX.md` | 14 entries. This proposal is listed as "accepted". |
| `docs/project/history/INDEX.md` | Lists history entries. |

### Files That Will Be Created

| File | Purpose |
|------|---------|
| `.github/copilot-instructions.md` | Always-loaded agent context with authority order and implementation discipline. |
| `.github/skills/revise/SKILL.md` | Revise skill definition. |
| `.github/skills/accept/SKILL.md` | Accept skill definition. |
| `.github/skills/plan/SKILL.md` | Plan skill definition. |
| `.github/skills/implement/SKILL.md` | Implement skill definition. |
| `.github/skills/audit/SKILL.md` | Audit skill definition. |
| `docs/project/plans/INDEX.md` | Index for the plans directory. |
| `docs/project/plans/proposal-process-improvements.md` | This file. |

### Glossary Term Insertion Points

New terms must be inserted alphabetically in the GLOSSARY.md table. The approximate insertion points (by preceding existing term):

| New Term | Insert After |
|----------|-------------|
| Authority order | Audit |
| Decision register | Defer |
| Design history | Decision register (new) |
| Design phase | Design history (new) |
| Design proposal | Design phase (new) |
| Feature coherence audit | Execution configuration |
| Implementation phase | (near "I" section — after any existing I terms) |
| Implementation plan | Implementation phase (new) |
| Planning phase | (near "P" section — after Package) |
| Prompt | (near "P" section — after Planning phase) |
| Punch list | Prompt (new) |

### Decision-to-Requirement Traceability

| Decision | Requirements |
|----------|-------------|
| D1 (lifecycle) | R1, R4 |
| D2 (plans directory) | R2 |
| D3 (lightweight path) | R3 |
| D4 (authority order) | R5 |
| D5 (enforcement) | R6 |
| D6 (terminology) | R7 |
| D7 (batch protocol) | R8, R9 |
| D8 (revise skill) | R12 |
| D9 (skills decomposition) | R10 |
| D10 (accept skill) | R13 |
| D11 (plan precondition) | R14 |
| D12 (revise cleanup) | R12 |
| D13 (copilot instructions) | R20 |
| D14 (implement skill) | R15, R21 |
| D15 (audit approach) | R16 |
| D16 (audit report storage) | R16 |
| D17 (audit in implementation) | R17 |
| D18 (quality safeguards) | R18, R19 |
| D19 (cross-session continuity) | R20, R21 |
| D20 (design history) | R22 |

---

## Referenced by

- [docs/project/proposals/proposal-process-improvements.md](../proposals/proposal-process-improvements.md)
