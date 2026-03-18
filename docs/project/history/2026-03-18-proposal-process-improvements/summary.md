---
title: Proposal Process Improvements — Design History
category: project
audience: [all]
status: current
last-verified: 2026-03-18
depends-on: [docs/project/proposals/proposal-process-improvements.md, docs/project/rfp/proposal-process-improvements.md]
---

# Proposal Process Improvements — Design History

*March 18, 2026*

This directory captures the evolution of the proposal process improvements design. Each version of the design proposal is preserved as a separate file, and this summary describes the deliberation narrative that drove changes between versions.

---

## Version 1 — Initial Design Proposal

*Generated on 2026-03-18 from [the RFP](../../rfp/proposal-process-improvements.md).*

The initial proposal was generated in response to an RFP that identified four failure modes in the existing proposal process: contradictory information in follow-on proposals, poor implementation readability, specification drift during implementation, and incomplete task execution. The RFP also requested research into industry best practices for agent-assisted design work.

### Research Findings

The agent surveyed five industry patterns for agent-driven design:

1. **Specification → Plan → Execute (SPE)** — used by Devin and Cursor Composer. Plans are opaque to the user.
2. **Iterative Refinement** — used by Anthropic's workflows. Conversational, but final state is spread across a thread.
3. **Document-Driven Development (DDD)** — adapted from Amazon's "working backwards." All artifacts are documents, not conversation messages.
4. **Dual-Track Development** — separate discovery and delivery tracks with different rhythms.
5. **Authoritative Document Pattern** — emerging best practice. One designated source of truth; all iterative discussion in other documents.

Five best practices for agent context management emerged: single source of truth, authority ordering, explicit state tracking, checkpoint-based execution, and batch communication.

### Design Alternatives Explored

**Feature 1: Revised Proposal Lifecycle** — four alternatives were considered:
- *Alternative A: Three-Document Lifecycle (RFP → Design Proposal → Implementation Plan)* — separates design deliberation from implementation instructions
- *Alternative B: Single Evolving Document with Sections* — uses a "Final State" section at top, history below
- *Alternative C: Two-Document with In-Place Consolidation* — rewrites proposal into plan on acceptance
- *Alternative D: Current Process with Stricter Instructions* — "try harder" approach

**Feature 4: Batch-Oriented Conversation Protocol** — three alternatives were considered:
- *Alternative A: In-Place Replacement* — each iteration overwrites the proposal, history captures evolution
- *Alternative B: Versioned Files* — proposal-v1.md, proposal-v2.md, etc.
- *Alternative C: Single File with Append-Only History Section* — body is current state, bottom has iteration history

**Feature 5: Skills Decomposition** — initially proposed five skills: propose, revise, plan, implement, audit.

**Feature 6: Feature Coherence Audit** — three approaches:
- *Alternative A: Checklist-Based* — fixed checklist of artifact types
- *Alternative B: Dependency-Driven* — follows `depends-on` metadata
- *Alternative C: Hybrid* — checklist for coverage, search for discovery, cross-check for consistency

**Feature 7: Proposal Quality** — two safeguards proposed: decision register and self-consistency check.

### Decisions Made in Version 1

The human reviewed the initial proposal and made decisions inline:

1. **Lifecycle:** Three-document (Alternative A). The process always begins with a prompt, which the agent formats as an RFP. The agent then decides whether to proceed to a design proposal or directly to an implementation plan based on whether the RFP needs further elaboration.

2. **Plans directory:** Implementation plans live in `docs/project/plans/` (new directory).

3. **Lightweight path:** There should always be at least an RFP. The agent decides whether a design proposal is needed.

4. **Authority order:** Accept the 12-step default. Enforce by convention (agent instructions) only, not tooling.

5. **Terminology:** Accept all proposed terms.

6. **Conversation protocol:** In-place replacement (Alternative A). History should be verbose — containing every version of every proposal, the associated punch list, and a narrative explaining how the proposal evolved between versions. This means decisions-not-taken and questions-already-answered can be removed from subsequent versions, because the information is still available in history.

7. **Revise skill:** A separate skill, not a mode of propose.

8. **Skills decomposition:** Modified. The skills as listed did not account for two things:
   - The punch list might be delivered as a separate document, but is more likely to be delivered as decisions made inline in the document.
   - The final version of the proposal will likely still contain decision points and a punch list, though the punch list may be trivially "accept everything."
   
   The revise skill should explicitly allow for inline punch lists. An **accept** skill should be added. The accept skill's purpose is to revise the proposal one last time so that all decisions are recorded, the content is consistent with all decisions, and no new decisions are requested. The accept skill should not result in surprises.
   
   The plan skill should first run the accept skill if the proposal hasn't been accepted yet. The human can send the proposal back after the plan step. Removing the old implementation plan should be part of the revise skill.

9. **Audit approach:** Hybrid (Alternative C). Reports saved to a file the human can delete. Audits should be part of the implementation plan.

10. **Proposal quality:** Both decision register and self-consistency check.

11. **Cross-session continuity:** Accept the proposed approach (copilot instructions for resuming from implementation plan).

Full text: [v1.md](v1.md)

---

## Version 2 — Decisions Incorporated

*Revised on 2026-03-18.*

Version 2 was a complete rewrite of the proposal incorporating all 19 decisions made inline by the human in Version 1. The key structural changes:

- **Decision Register added.** A consolidated table of all 19 decisions was placed at the top of the proposal as the authoritative reference.
- **Research section condensed.** The full industry survey was replaced with a summary of the five key findings that shaped the design. The full research was preserved in the Version 1 history.
- **Rejected alternatives removed.** Each feature section was rewritten to present only the accepted design, not the full alternatives analysis.
- **Accept skill added** (Decision 9/10). A new sixth skill for finalizing proposals.
- **Revise skill updated** to handle inline punch lists (Decision 8) and to remove stale implementation plans (Decision 12).
- **Plan skill updated** to run accept first if the proposal hasn't been accepted yet (Decision 11).
- **Features reorganized** from 8 to 7. Original Features 7 and 8 were folded into the settled Features 6 and 7.

### Punch List (Version 2 → Version 3)

The human reviewed Version 2 and made one inline decision:

**Design History Convention** — The `docs/project/history` directory should contain an index file and one directory per proposal. Each proposal's history directory should contain a summary file with the narrative of the evolving proposal, and a separate file for each version (v1, v2, ... accepted). The content remains the same but is split across multiple files to prevent any single file from becoming too large.

Full text: [v2.md](v2.md)

---

## Version 3 — History Directory Structure

*Revised on 2026-03-18.*

Version 3 incorporated the single decision from the Version 2 punch list: restructuring the design history from a monolithic file per proposal into a directory per proposal containing separate files. This was Decision #20 in the register.

The changes were systematic — every reference to "design history file" throughout the proposal was updated to reflect the directory-based structure:

- **Decision Register:** Added Decision #20 (design history structure).
- **Artifact table:** Updated the Design History location from `docs/project/history/` to `docs/project/history/<slug>/`.
- **Design History Convention section:** Rewritten from a brief description of a single verbose file to a detailed table showing the directory layout (summary.md, v1.md, v2.md, ..., accepted.md).
- **Terminology:** Updated the Design History definition to describe the directory structure.
- **Skill procedures:** All six references to "the design history file" across the `/revise`, `/accept`, and batch protocol sections were updated to reference "the design history directory" or "the summary file in the design history directory."
- **Authority order:** "History file" updated to "History" in the implementation plan template and copilot instructions.
- **History Updates checklist:** Updated to reference the directory.

The change was pure refactoring of the history storage mechanism — no design decisions, feature additions, or behavioral changes beyond the directory structure itself.

Full text: [v3.md](v3.md)

---

## Accepted

*Accepted on 2026-03-18.*

The proposal was accepted after three iterations of design refinement. All 20 decisions in the decision register were resolved through the iterative process — 11 in the original Version 1 review, 8 organizational decisions during the Version 2 rewrite, and 1 in the Version 3 review. No decisions remained open at acceptance time.

The accept step was mechanical: the proposal body already presented all designs as settled fact with no remaining alternatives or open decision prompts. The only changes were setting the status to "accepted," updating the version line, and finalizing the decision register framing.

Full text: The accepted proposal is at [proposal-process-improvements.md](../../proposals/proposal-process-improvements.md). The pre-acceptance Version 3 text is preserved at [v3.md](v3.md).

---

## Implementation

*Implemented on 2026-03-18.*

The implementation plan was generated from the accepted proposal and executed following the authority order defined by the proposal itself — the first use of the new methodology.

Implementation proceeded in five phases. Phase 1 added 11 glossary terms (authority order, decision register, design history, design phase, design proposal, feature coherence audit, implementation phase, implementation plan, planning phase, prompt, punch list) and set the roadmap to "in progress." Phase 2 updated CONTRIBUTING.md with the full three-document lifecycle, authority order, and design history directory conventions, and added a proposal process section to the AI guide orientation.

Phase 3 was the largest: creating `.github/copilot-instructions.md` as the always-loaded agent context file, updating the existing `/propose` skill to include a Decision Register in its output template, and creating five new skills (`/revise`, `/accept`, `/plan`, `/implement`, `/audit`). AGENTS.md was updated with playbook entries for all six skills and a key concept referencing the three-document lifecycle.

Phase 4 ran a feature coherence audit using the `/audit` skill's own procedure — the first such audit in the project. The audit found one real inconsistency: the `/plan` skill's output template listed 13 authority order items instead of the authoritative 12 (splitting "User guide / AI guide" and "Agent documentation" into separate lines). This was corrected.

Phase 5 moved the roadmap entry to "completed," updated this history narrative, and verified all index files.

Implementation plan: [docs/project/plans/proposal-process-improvements.md](../../plans/proposal-process-improvements.md)

---

## Referenced by

- [docs/project/proposals/proposal-process-improvements.md](../../proposals/proposal-process-improvements.md)
- [docs/project/history/INDEX.md](../INDEX.md)
