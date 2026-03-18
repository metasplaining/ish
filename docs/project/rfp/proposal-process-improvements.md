---
title: "RFP: Proposal Process Improvements"
category: rfp
audience: [all]
status: stable
last-verified: 2026-03-18
depends-on: [AGENTS.md, CONTRIBUTING.md, GLOSSARY.md, docs/ai-guide/INDEX.md]
---

# RFP: Proposal Process Improvements

*Cleaned-up version of the `proposals2` prompt file.*

---

## Background

The agent has been underperforming on recent proposals. The proposal system needs improvements to context management and state management.

### Observed Problems

1. **Contradictory information in follow-on proposals.** When creating a follow-on proposal, the agent sometimes left contradictory information in the proposal because it did not thoroughly scan for elements of the previous proposal that contradicted the decisions made.

2. **Poor implementation readability.** When implementing a proposal, the agent became confused because the final proposal chain was organized for historical analysis rather than organized for the agent to understand the final state of the proposal.

3. **Specification drift during implementation.** While in a test/fix cycle as part of implementation, the agent lost track of the specified behavior and intentionally injected behavior that contradicted the proposal.

4. **Incomplete task execution.** The agent lost the thread of the changes it was implementing and left some aspects of the task — such as documentation updates and history updates — undone.

---

## Proposed Improvements

### 1. Consolidated Authoritative Proposal

There should be one consolidated authoritative proposal from which implementation can proceed.

### 2. Agent-Friendly Format

The final proposal should be agent-friendly and formatted for efficient context management.

### 3. Comprehensive Final Proposal

The final proposal should be comprehensive, including requirements, design, descriptions of what changes are needed to each aspect of the project (documentation, roadmap, user guide, AI guide, proposal history, etc.) that needs to be changed, and an ordered TODO list of tasks that need to be accomplished.

### 4. Iterative Development

Development of the final proposal should still be iterative and conversational, beginning with an RFP and proceeding through enough iterations until the final proposal is acceptable.

### 5. Full History Capture

Full history should still be captured, including a narrative of what was suggested by the human, what was suggested by the agent, and what was decided by the human, and in what order.

### 6. Authority-Ordered TODO List

The TODO list should be organized in authority order. For example, during implementation of a recent feature, the agent referred to the project roadmap, which indicated that a feature which was already complete had not yet been implemented. So it proceeded as if that feature was both unimplemented and out of scope. In future, the roadmap should be updated to set the feature status to "in progress" when work starts and "completed" when work finishes. That way, if the implementing agent refers to the roadmap, it will not draw the wrong conclusion.

Similarly, an authoritative order of artifacts needs to be established (e.g., docs > assurance tests > code) and the most authoritative artifacts should be changed first. If implementation requires historical information (for example, if some files are being moved and the implementation needs to know the original location of the files), that information should be moved to a temporary location (the final proposal, perhaps).

### 7. Updated Agent Documentation

The agent documentation needs to be updated so that the agent will consistently follow the new process. This may require a copilot instructions file and almost certainly requires new skills.

### 8. Batch-Oriented Conversation

The requirements and design process needs to remain batch-oriented. The "conversation" between the human and the agent is one in which the human submits a lengthy and detailed request document, then the agent creates a lengthy and detailed response, then the human responds with a punch list of things that need to be changed, the agent creates an updated response that addresses all of the items in the punch list, and so on.

---

## Terminology Improvements

The terminology needs to be improved to make proper distinctions between:

- The artifacts and activities of the **design process** (RFP, interim proposal)
- The artifacts and activities of the **implementation planning process** (final proposal)

---

## Feature Coherence Audit Skill

A feature coherence audit skill is needed. This is a skill that an agent uses to verify consistency of a single feature throughout the codebase. It requires:

1. Defining all the places that contain information about a feature (roadmap, spec, acceptance test, architecture, developer guide, AI guide, code, unit test, history, etc.)
2. Cross-checking all of these information sources against each other
3. Verifying that they are consistent

---

## Research Request

Research best practices and standard terminology for using agents for requirements and design work. Propose alternative approaches with pros and cons for each, and a recommendation.

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
