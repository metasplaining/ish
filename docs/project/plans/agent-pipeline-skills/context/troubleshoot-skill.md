*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Troubleshoot Skill (New).*

---

## Feature: Troubleshoot Skill (New)

### Purpose

Determine the root cause of a problem without fixing it. The troubleshoot agent diagnoses; the bug-fix agent repairs. Troubleshoot must not change the specification, even if the specification appears incorrect.

### Specification Evaluation

Before concluding that the bug is an implementation error, evaluate the specification:
- Is the specification complete? (Does it address the failing case?)
- Is the specification unambiguous? (Could a careful reader reach a different interpretation?)
- Is the specification achievable? (Given the architecture, can it be implemented as written?)

If the answer to any question is "no": the problem requires a specification fix, not a code fix. Write a clarification request and stop. Do not attempt to fix the specification.

### Skill Procedure

New skill: `.claude/skills/troubleshoot/SKILL.md`

1. Read the problem report.
2. Read the implementation plan and accepted proposal for the affected feature.
3. Read the relevant spec and architecture docs.
4. Create a scratch file at `docs/project/clarifications/debug-scratch.md` (deleted on completion).
5. Form a hypothesis. Record it in the scratch file.
6. Gather evidence:
   - Add debug logging if needed. Document each change in the scratch file.
   - Run targeted tests: `cd proto && cargo test <test_name>` or equivalent.
   - Do not change behavior beyond adding debug instrumentation.
7. Confirm or refute the hypothesis. Update the scratch file.
8. Repeat steps 5–7 until root cause is identified.
9. Remove all debug instrumentation from code.
10. Delete the scratch file.
11. Evaluate the specification (see above).
12. If specification problem: write a clarification request at `docs/project/clarifications/<date>-<topic>.md`. Include: what was found, why it requires specification change, suggested resolution. Stop.
13. If implementation error: write a root-cause document at `docs/project/clarifications/<date>-<topic>.md`. Include: what is wrong, where in the code, what the correct behavior should be (per spec). Hand off to Bug Fix.
14. Report findings, with path to the clarification document.

### Issues to Watch Out For

- **Scope creep**: If additional bugs are found during investigation, document them separately. Do not expand scope.
- **"Achievable" judgment**: The troubleshoot agent must not conclude that a spec is "unachievable" simply because the current architecture makes it difficult. Consult the architecture docs before concluding unachievability.
- **Premature handoff**: Do not hand off to Bug Fix until root cause is confirmed, not just hypothesized.
