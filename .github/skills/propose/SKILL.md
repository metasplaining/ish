---
name: propose
description: 'Create a structured design proposal from a prompt file. Use when: the user has a file containing questions, feature requests, or instructions (like a prompt file) and wants a detailed design proposal with critical analysis, alternatives, and implementation details. Trigger words: propose, proposal, design, design proposal, feature request, prompt file, design review.'
argument-hint: 'Path to a file containing questions and feature requests'
---

# Propose

Read a prompt file containing questions and feature requests, then produce a structured design proposal with critical analysis and implementation details. Save the result for human review.

## When to Use

- The user has a file with a list of questions, instructions, or feature requests
- The user wants a plan created from that file before implementation begins
- The user invokes `/propose <filepath>`

## Procedure

1. **Read the input file** at the path provided by the user. If no path is given, ask for one.

2. **Convert the input file to a Request for Proposal (RFP).** Before doing anything else:
   - Clean up the formatting of the input file (fix Markdown structure, indentation, code fences).
   - Correct grammar, spelling, and typos.
   - Preserve the original meanings — do not add, remove, or alter the intent of any content.
   - Save the RFP to `docs/project/rfp/<name>.md` where `<name>` is a meaningful slug derived from the primary topic. Use standard YAML frontmatter with `category: rfp`.
   - Update the RFP index at `docs/project/rfp/INDEX.md` (create it if it doesn't exist).

3. **Identify the contents.** Extract from the RFP (not the original prompt):
   - Any questions that need answering
   - Any feature requests or changes being proposed
   - Any constraints or context provided

4. **Answer all questions.** For each question found in the RFP, research the codebase and provide a thorough answer.

5. **Analyze each feature or change.** For every feature request or proposed change, write the following sections:

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

6. **Append a Documentation Updates section.** List the documentation files likely affected by the proposed changes, referencing `depends-on` frontmatter and cross-references. Include a reminder to update `## Referenced by` sections.

7. **Append a History Updates section.** Remind the implementer to:
   - Create a history directory under `docs/project/history/` named `<isodate>-<topic>/`
   - Add a `summary.md` with narrative prose describing the proposal's evolution
   - Add version files (`v1.md`, `v2.md`, etc.) as the proposal iterates
   - Update the [history index](../../../docs/project/history/INDEX.md)

8. **Save the output** to `docs/project/proposals/<name>.md` where `<name>` is derived from the RFP filename or the primary topic. If the proposals directory doesn't exist, create it. The design proposal must reference the RFP, not the original prompt file.

## Output Format

The saved design proposal file should follow this structure:

```markdown
# Proposal: <Topic>

*Generated from [<rfp-filename>](../rfp/<rfp-filename>) on <date>.*

---

## Decision Register

All decisions made during design, consolidated here as the authoritative reference.

| # | Decision | Outcome |
|---|----------|---------|
| 1 | <decision description> | <outcome or "pending"> |

---

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
- [ ] Create `docs/project/history/<date>-<topic>/` directory
- [ ] Add `summary.md` with narrative prose
- [ ] Update `docs/project/history/INDEX.md`
```
