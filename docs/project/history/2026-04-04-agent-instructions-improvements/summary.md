---
title: "History: Agent Instructions Improvements"
category: history
audience: [all]
status: draft
last-verified: 2026-04-04
depends-on: [docs/project/proposals/agent-instructions-improvements.md]
---

# History: Agent Instructions Improvements

The agent-instructions-improvements RFP began from a practical observation: agents were frequently disoriented at the start of new sessions. The bootstrap documentation — split across `AGENTS.md`, `CLAUDE.md`, and `.github/copilot-instructions.md` — was fragmented, and the project had begun using both GitHub Copilot and Claude Code simultaneously, creating a file-duplication hazard in the skills directories.

The human's prompt identified two distinct problem areas. The first was organizational: consolidate the three instruction files into a single source of truth, define a vendor-agnostic location for skills, and ensure both vendors point to it via symlinks. The second was operational: the agent pipeline was missing three key roles (Bug Fix, Troubleshoot, Verify), and the existing skills needed guard rails to prevent creative improvisation outside the proposal process.

The design proposal recognized that these two areas are largely independent — the infrastructure work (file layout, symlinks) could proceed without waiting for the skills work, and vice versa. This raised the question of whether to split the proposal. The decision register leaves this open for the human to decide.

The methodology discussion in the proposal engaged seriously with the "no creativity outside the proposal process" rule, finding it sound but in need of an escape hatch for trivially obvious fixes. It also added four practices the RFP had not mentioned: regression testing after bug fixes, escalation hierarchy, debug artifact cleanup, and session continuity.

The terminology section settled on "skill" for the SKILL.md file artifact and "role" for the conceptual pipeline stage, reserving "agent" for the AI process itself.

---

## Revision 1 (v2 → split)

The human reviewed the initial proposal and made inline decisions on all 15 decision points. The most consequential decision was to split the proposal into two: Agent Pipeline Skills and Agent Infrastructure. The split was motivated by the clear independence of the two areas — the skills work does not depend on the file layout, and vice versa.

Key decisions in revision 1:
- Canonical skills location: `.agents/skills/` (a new option not in the original proposal, genuinely vendor-agnostic)
- Agent-friendly style and Rust style guidelines go in `CONTRIBUTING.md`, not AGENTS.md
- The maintenance skill auto-applies changes (with confirmation) rather than only proposing
- Proposal splitting threshold: 10+ independent implementation steps
- Backward compatibility check removed from Revise gap detection (prototype stage)
- Plan directories triggered at > 5 steps; context files extracted verbatim
- Implement stops at the first unclear item
- Bug Fix proceeds only after root cause is identified; must fix all artifact instances of the behavior
- Debug instrumentation uses a scratch file (deleted on completion)
- Verify is standalone but also invoked at each implement checkpoint
- Clarification requests go in `docs/project/clarifications/`

---

---

## Acceptance: Agent Pipeline Skills

The Agent Pipeline Skills proposal was accepted on 2026-04-04 with all 12 decisions finalized. The accepted design adds two new behaviors to the Revise skill (gap detection and split evaluation), extends the Plan skill to produce phase directories for proposals with more than 5 steps, and adds guard rails to the Implement skill. Three new skills are created: Bug Fix, Troubleshoot, and Verify. A backchannel communication mechanism (clarification request files in `docs/project/clarifications/`) is established for all blocking situations. Six terms are added to the glossary: Role, Skill, Agent, Backchannel, Clarification request, and Phase.

The final design is clean and internally consistent. No open questions remain.

---

## Implementation: Agent Pipeline Skills

Implemented on 2026-04-04. All 17 tasks completed across six phases:
- Phase 1: Added six glossary terms; updated AI guide orientation to eight-role pipeline.
- Phase 2: Added gap detection (step 6a) and split evaluation (step 6b) to Revise skill.
- Phase 3: Added phase-directory output, context file rules, and scrutiny step to Plan skill.
- Phase 4: Added guard rails, phase-directory handling, and Verify invocation to Implement skill.
- Phase 5: Created Bug Fix, Troubleshoot, and Verify skills; created clarifications index.
- Phase 6: Updated plan status and index files.

---

## Bug Fix: Clarification Request Format in New Skills

**What was wrong**: The three new skills (bug-fix, troubleshoot, verify) wrote clarification request files to `docs/project/clarifications/` but did not use the structured format specified in the Backchannel Communication spec. The skills described content ad-hoc ("Include: X, Y, Z") and did not instruct agents to update `docs/project/clarifications/INDEX.md`. Additionally, bug-fix had `blocked` in its Output Status but no steps for writing clarification files in its blocking cases (steps 7–8).

**What was fixed**: All three skills now include a `Clarification Request Format` section showing the exact template (title, metadata, Context, Blocked On, Questions, Recommended Resolution). Steps that write clarification files (troubleshoot steps 12–13, verify step 6, bug-fix steps 7–8) now reference the format and include the INDEX.md update instruction.

**Artifacts updated**: `.claude/skills/troubleshoot/SKILL.md`, `.claude/skills/verify/SKILL.md`, `.claude/skills/bug-fix/SKILL.md`

---

---

## Revision: Agent Infrastructure — Prerequisite Resolved

The punch list for this revision was a single fact: the agent-pipeline-skills proposal has been implemented. This resolved the only external dependency blocking the agent-infrastructure proposal.

Changes made:
- **Preamble**: Removed the "implement after agent-pipeline-skills" blocking note; replaced with a confirmation that the prerequisite was completed on 2026-04-04.
- **Vendor-Agnostic File Layout**: Added a "Current skill inventory" section documenting the pre-migration split — six skills in `.github/skills/` and three new skills in `.claude/skills/`. Step 2 of the implementation now explicitly collects from both source directories. Step 5 now lists all nine skills to verify.
- **AGENTS.md Rewrite — Issues**: Replaced the "Implement Proposal B first" placeholder with concrete pre- and post-migration paths for all nine skills.
- **Gap detection (new open questions injected)**:
  - `/update-agents` step 2 reads `docs/INDEX.md` — no error handling specified. Injected open question: what should the skill do if `docs/INDEX.md` is missing?
  - No acceptance test specified for the `/update-agents` skill. Injected open question: what test verifies it correctly detects a skill absent from the Task Playbooks table?
- **Split evaluation**: 4 independent implementation groups (file layout chain, AGENTS.md chain, update-agents skill, CONTRIBUTING.md). Less than 10 → no split.
- **Frontmatter**: Removed `docs/project/proposals/agent-pipeline-skills.md` from `depends-on`; the dependency is satisfied. `depends-on` now lists the files this proposal will modify.

---

## Revision: Agent Infrastructure — Open Questions Resolved

The punch list for this revision was two inline decisions on open questions in the AGENTS.md Maintenance Skill feature.

**Decision 6**: `/update-agents` step 2 reads `docs/INDEX.md` — if the file is missing, report the gap to the human and abort. The option to skip silently was rejected; the skill should never proceed with incomplete information.

**Decision 7**: No acceptance test is required for the `/update-agents` skill's missing-skill detection capability.

Changes made:
- **Decision register**: Added decisions 6 and 7.
- **AGENTS.md Maintenance Skill — Proposed Implementation**: Step 2 now specifies the abort behavior explicitly: "If `docs/INDEX.md` is missing, report the gap to the human and abort."
- **Open questions removed**: Both resolved open questions removed from the body; their outcomes are captured in the decision register.

No open questions remain. The proposal is ready for acceptance.

---

## Acceptance: Agent Infrastructure

The Agent Infrastructure proposal was accepted on 2026-04-05 with all 7 decisions finalized.

The accepted design consolidates the project's agent instruction files and skills directories into a vendor-agnostic layout. `AGENTS.md` becomes the single canonical instruction file; `CLAUDE.md` and `.github/copilot-instructions.md` become symlinks. All nine existing skills move from their split vendor directories (`.github/skills/` and `.claude/skills/`) to a new canonical location at `.agents/skills/`; both vendor directories become symlinks. A tenth skill, `/update-agents`, is introduced to maintain `AGENTS.md` for staleness and consistency. Agent-friendly style guidelines and Rust style guidelines are added to `CONTRIBUTING.md`. The new `AGENTS.md` merges content from the current `AGENTS.md` and `CLAUDE.md`, adds a "Never Touch" section, a prototype-stage rule, an expanded tech stack section, and the full ten-skill playbook table.

No open questions remain.

---

## Bug Fix: Implement Skill Halts at Verify Invocation

**What was wrong**: The implement skill's procedure for directory plans said "at the end of each phase, invoke `/verify <plan-name>/phase-N.md`." When the agent followed this instruction it used the `Skill` tool, which works by injecting the target skill's content as a new user message. This created a new conversation turn where verify became the active context. When verify finished, that response was the terminal output for the turn — the implement execution context was gone, with no return path. The result was that implement halted after completing each phase, requiring the user to manually re-invoke it.

**Root cause**: The instruction "invoke `/verify`" was ambiguous. The Skill-tool interpretation (spawning a new conversation turn) terminates implement; the inline interpretation (running the verify checks within the same turn) allows continuation. The spec did not specify which mechanism to use, and the agent chose the wrong one.

**What was fixed**: The implement skill's phase-checkpoint instruction was rewritten to say "run the verify procedure inline (do not use the Skill tool)" and explains exactly why — invoking it creates a new conversation turn and terminates implement. The verify skill's "Integration with Implement" section was updated to reflect that implement now runs verification inline, and that the `/verify` skill is for user-invoked standalone checks only.

**Artifacts updated**: `.agents/skills/implement/SKILL.md` (line 49), `.agents/skills/verify/SKILL.md` (Integration with Implement section)

---

## Implementation: Agent Infrastructure

Implemented on 2026-04-05. All tasks completed across six phases.

Phase 1 updated the roadmap to mark Agent Infrastructure as in progress. Phase 2 performed the file layout migration: all nine existing skills were moved from their split vendor directories (six in `.github/skills/`, three in `.claude/skills/`) into a new canonical directory at `.agents/skills/`. Both vendor directories were replaced with symlinks pointing to `../.agents/skills`, making them transparent aliases from the vendors' perspective.

Phase 3 rewrote `AGENTS.md`. The new file merges the content from the previous `AGENTS.md` (~108 lines) with the content of `CLAUDE.md` (~46 lines, which contained only the proposal process sections), adds four new sections (Never Touch, Project Stage Rule, Tech Stack, and a merged Proposal Process), and updates the Task Playbooks table to reflect all ten skills at their new `.agents/skills/` paths. The existing `CLAUDE.md` file was replaced with a symlink to `AGENTS.md`, and `.github/copilot-instructions.md` was similarly replaced with a symlink to `../AGENTS.md`. Both vendors now read from a single source of truth.

Phase 4 created the new `/update-agents` skill at `.agents/skills/update-agents/SKILL.md`. The skill's procedure checks AGENTS.md for staleness by cross-referencing the skills directory, the crate map, and `docs/INDEX.md`. Its critical step 2 specifies that if `docs/INDEX.md` is missing, the skill must report the gap and abort rather than proceeding with incomplete information.

Phase 5 extended `CONTRIBUTING.md` with two new sections: Agent-Friendly Style Guidelines (seven bullets covering imperative mood, actionable instructions, and copy-pasteable commands for AGENTS.md and SKILL.md authors) and Rust Style Guidelines (six bullets covering idiomatic Rust, error propagation, and the prototype-stage rule against backward-compatibility shims).

Phase 6 updated the roadmap, this history document, and index files.

---

## Versions

- [v1.md](v1.md) — Initial combined proposal, 2026-04-04
- [v2.md](v2.md) — With inline decisions, before split, 2026-04-04
- [v3.md](v3.md) — Pre-accept snapshot of agent-pipeline-skills, 2026-04-04
- [v4.md](v4.md) — Pre-revision snapshot of agent-infrastructure, 2026-04-04
- [v5.md](v5.md) — Pre-revision snapshot before open-question resolution, 2026-04-05
- [v6.md](v6.md) — Pre-accept snapshot of agent-infrastructure, 2026-04-05
