---
title: Agent Instructions Improvements
category: rfp
audience: [ai-dev]
status: draft
last-verified: 2026-04-04
depends-on: [AGENTS.md, .github/copilot-instructions.md, docs/project/rfp/proposal-process-improvements.md]
---

# RFP: Agent Instructions Improvements

*Cleaned up from `agent_instructions` prompt file on 2026-04-04.*

---

## Background

The agent is underperforming. Agents frequently start new sessions confused. The bootstrap documentation is not working well and needs to be improved. Additionally, with the move to both GitHub Copilot and Claude Code, the agent instructions and skills need to be vendor-agnostic.

---

## 1. AGENTS.md Consolidation and Vendor-Agnostic Organization

`AGENTS.md` should be the single bootstrap file that every agent reads when it first starts up.

### 1.1 Content Consolidation

The contents of the existing `AGENTS.md` and `.github/copilot-instructions.md` should be combined as a starting point for a new, unified `AGENTS.md`.

### 1.2 Vendor-Agnostic Layout

Since the project now uses both GitHub Copilot and Claude Code, all agent instructions and skills should be moved to vendor-agnostic locations. The vendor-specific files should become symbolic links to the agnostic files:

- `.github/copilot-instructions.md` → symlink to `AGENTS.md`
- `CLAUDE.md` → symlink to `AGENTS.md`
- Skills should move to a standard, vendor-agnostic location. Vendor-specific skill directories should reference the agnostic location. Verify that the correct referencing mechanism works for both Claude Code and Copilot.

### 1.3 AGENTS.md Rules

The new `AGENTS.md` must follow these rules:

1. Write agent instructions using agent-friendly style guidelines.
2. Target approximately 500 lines. Pack in as much useful context as possible without overdoing it.
3. Provide an efficient index for all other agent instructions, skills, and documentation in the project.
4. Put executable commands (e.g., `cd proto && bash ish-tests/run_all.sh  # Run acceptance tests`) in an early, visible section.
5. Explicitly list what the agent should never touch (secrets, production configs, specific vendor folders, etc.).
6. Be hyper-specific about the tech stack rather than general.
7. Describe style and workflow.
8. For every line in `AGENTS.md`, ask: "Would the agent make a mistake without this?" If the agent already does it correctly, delete the line to save tokens.
9. When a mistake is made, update agent instructions so it does not happen again.
10. Information needed only for specific tasks should be in a separate file.

### 1.4 Agent Query Support

`AGENTS.md` must enable each agent or sub-agent to answer the following questions:

1. What rules apply to my specific task?
2. What product requirements apply to the part of the system I am working on?
3. What is the architecture of the part of the system I am working on?

---

## 2. AGENTS.md Maintenance Skill

We should regularly maintain `AGENTS.md`. Create a skill for doing this, so that we can regularly update it as the product evolves. Each time it is updated, re-evaluate the most valuable content based on the rules in §1.3.

---

## 3. Rust Style Guideline

The Rust style guideline for this project is to conform to idiomatic Rust as much as possible.

---

## 4. Skill Improvements

### 4.1 Revise

When revising a proposal, the agent should:

- Evaluate what the proposal is silent or unclear about. Parts of the proposal that are not fully specified do not get implemented well. Inject these concerns as decision points or open questions, to assist the human in knowing what needs elaboration.
- Evaluate whether the proposal is growing too large. If it is too large, divide it into multiple proposals. The new proposals should be organized to be as independent of one another as possible, with a clear implementation order. Divide proposals with an eye toward implementing the best-understood, least-dependent part first, while the other parts are still being worked out.

### 4.2 Plan

When planning an implementation, the agent should:

- Organize the plan to be implemented by sub-agents. Each phase of the plan should be a separate file (each plan will need its own directory).
- Extract relevant technical specifications into context files. Context files should contain everything the implementing agent needs to know, with references back to the sources of truth in the docs and proposal.
- Each phase file should contain: references to the relevant context files, requirements for the phase, a detailed ordered list of tasks, and verification instructions.
- Scrutinize each phase of the plan to ensure it contains all the technical detail the implementing agent will need. When there are gaps in the information, create a list of additional information needed and send the proposal back for further revisions rather than filling in the blanks.

### 4.3 Implement

When implementing a plan:

- Follow the plan.
- If the plan is unclear, contradictory, or asks for the impossible, request clarifications rather than continuing to implement.

### 4.4 Bug Fix (New Skill)

After a plan has been implemented, the human will sometimes detect something that was specified or implemented incorrectly. When fixing a bug:

- Take extra care that the whole set of necessary steps takes place: all parts of the documentation must be updated, history must be updated, and all tests must be re-run after the fix.
- The bug-fixing agent may fix tests — but must never change their meaning.
  - If a test is failing because some unrelated part of the system changed, the bug-fixing agent may fix the test.
  - If a test is failing because it expects the wrong behavior, the bug-fixing agent must report the issue rather than fixing it. When a test expects the wrong behavior, a human must be consulted to determine whether the test or the code is in error.

### 4.5 Troubleshoot (New Skill)

When implementation or bug fix does not go as expected:

- Be aware that many issues requiring troubleshooting are the result of specification problems.
- First determine the root cause without significantly changing the behavior of the system. Debug logs and temporary behavior changes are allowed, but must be documented so they can be backed out once the root cause is identified.
- Once the root cause is identified, determine whether the problem should be fixed or sent back for further specification:
  - Examine the documentation and the proposal being implemented. Determine whether the specification is complete, unambiguous, and achievable.
  - If not, request clarification of the specification rather than attempting to fix it.
  - If the specification is complete, unambiguous, and achievable, document the root cause and hand the problem to the bug-fixing agent.
- The troubleshooting agent's job is to determine what went wrong. The bug-fixing agent's job is to make sure all parts of the fix occur.
- The troubleshooting agent must not change the specification.

### 4.6 Verify (New Skill)

When a phase of a plan or a bug fix has been implemented:

- Check for consistency: code should be consistent with the tests; tests should be consistent with the documentation; documentation should be consistent with the proposal.
- The verification agent detects inconsistencies and determines their scope. It does not fix them.
- Once an inconsistency is identified, hand the problem to the troubleshooting agent.

---

## 5. Agent Pipeline and No-Creativity Rule

As a whole, the system consists of a set of skills/agents, each with a different role: Propose, Revise, Plan, Implement, Bug Fix, Troubleshoot, Verify. Almost all tasks should be performed by one of these agents in one of these roles. Each agent should have guard rails to prevent creative improvisation. Only the Propose and Revise agents are creative, in cooperation with a human. All other agents should be doing something completely predictable.

**Project-wide rule:** Creativity is not allowed outside the proposal process. When creativity is required, send the problem back to the proposal process.

When they run into a problem that requires creativity, the non-creative agents should send the problem back to the proposal process.

---

## 6. Backchannel Communication

The changed skills require new communication channels for agents to send feedback upstream. Propose artifacts and mechanisms for these backchannels.

---

## 7. Questions

1. Are additional skills, agents, or roles needed beyond the seven described above?
2. What terminology should be used? "Skills," "agents," "roles," or something else?
3. Should the brief sketches in §4 be elaborated into full, useful sets of instructions? Research best practices for agents in each of these roles.
4. Consider the methodology as a whole: are any of these practices a bad idea? Are there other practices that should be included?

---

## Referenced by

- [docs/project/rfp/INDEX.md](INDEX.md)
