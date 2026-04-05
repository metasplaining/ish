*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Terminology Canonicalization.*

---

## Feature: Terminology Canonicalization

### GLOSSARY.md Additions

Add to GLOSSARY.md:

| Term | Definition |
|------|-----------|
| **Role** | A conceptual stage in the agent pipeline: Propose, Revise, Accept, Plan, Implement, Bug Fix, Troubleshoot, or Verify. A skill implements a role. |
| **Skill** | A SKILL.md file defining the procedure for an agent fulfilling a specific role. Loaded at prompt time by the agent's execution environment. |
| **Agent** | The AI process executing a skill within a session. |
| **Backchannel** | A mechanism for an agent to send structured feedback upstream when it cannot proceed. Currently implemented as clarification request files. |
| **Clarification request** | A structured file written by a blocked agent to request human input. Stored in `docs/project/clarifications/`. |
| **Phase** | A subdivision of an implementation plan that can be assigned to a single sub-agent. Each phase has its own file in the plan directory, with context files, tasks, and verification instructions. |
