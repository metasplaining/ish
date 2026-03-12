---
name: propose
description: 'Create a structured proposal from a prompt file. Use when: the user has a file containing questions, feature requests, or instructions (like a prompt file) and wants a detailed plan with critical analysis, alternatives, and implementation details. Trigger words: propose, proposal, plan, feature request, prompt file, design review.'
argument-hint: 'Path to a file containing questions and feature requests'
---

# Propose

Read a prompt file containing questions and feature requests, then produce a structured proposal with critical analysis and implementation details. Save the result for human review.

## When to Use

- The user has a file with a list of questions, instructions, or feature requests
- The user wants a plan created from that file before implementation begins
- The user invokes `/propose <filepath>`

## Procedure

1. **Read the input file** at the path provided by the user. If no path is given, ask for one.

2. **Identify the contents.** Extract:
   - Any questions that need answering
   - Any feature requests or changes being proposed
   - Any constraints or context provided

3. **Answer all questions.** For each question found in the file, research the codebase and provide a thorough answer.

4. **Analyze each feature or change.** For every feature request or proposed change, write the following sections:

   ### Issues to Watch Out For
   Identify risks, edge cases, backwards compatibility concerns, and potential pitfalls.

   ### Critical Analysis
   Evaluate the requested feature honestly. Suggest possible alternatives. For each alternative (including the original request), list pros and cons. Do not simply accept the request at face value — challenge assumptions where appropriate.

   ### Proposed Implementation
   Describe how the feature would be implemented: which files change, what new files are needed, the sequence of steps, and any dependencies between features.

   ### Decisions
   Leave a blank section with decision prompts for the human to fill in. Format as:
   ```
   **Decision:** <question about which alternative to choose>
   -->
   ```

5. **Append a Documentation Updates section.** List the documentation files likely affected by the proposed changes, referencing `depends-on` frontmatter and cross-references. Include a reminder to update `## Referenced by` sections.

6. **Append a History Updates section.** Remind the implementer to:
   - Add a history file under `docs/project/history/` named `<isodate>-<topic>.md`
   - Write it as human-oriented narrative prose (per project conventions)
   - Update the [history index](docs/project/history/INDEX.md)

7. **Save the output** to `docs/project/proposals/<name>.md` where `<name>` is derived from the input filename or the primary topic. If the proposals directory doesn't exist, create it.

## Output Format

The saved proposal file should follow this structure:

```markdown
# Proposal: <Topic>

*Generated from `<input-file-path>` on <date>.*

## Questions and Answers

### Q: <question text>
<answer>

## Feature: <feature name>

### Issues to Watch Out For
<risks and pitfalls>

### Critical Analysis
<evaluation with alternatives, pros/cons>

### Proposed Implementation
<implementation details>

### Decisions
**Decision:** <question>
-->

---

## Documentation Updates
<list of affected docs>

## History Updates
- [ ] Add `docs/project/history/<date>-<topic>.md`
- [ ] Update `docs/project/history/INDEX.md`
```
