*Extracted verbatim from [docs/project/proposals/agent-pipeline-skills.md](../../../proposals/agent-pipeline-skills.md) §Feature: Revise Skill — Gap Detection and Proposal Splitting.*

---

## Feature: Revise Skill — Gap Detection and Proposal Splitting

### What Changes

The Revise skill gains two new behaviors after incorporating the punch list:

1. **Gap detection**: After rewriting, scan each feature for specification completeness. Add open questions where gaps are found.
2. **Proposal splitting**: If the revised proposal contains ≥ 10 implementation steps that can be extracted and implemented independently, propose a split.

### Gap Detection Checklist

For each feature section, check:
- [ ] Are the error cases specified?
- [ ] Is concurrency behavior addressed (if the feature touches async code)?
- [ ] Is there a testability statement (what acceptance test would verify this)?

Note: backward-compatibility analysis is explicitly excluded. This project is in the prototype stage; backward compatibility is not a concern.

For each unchecked item, inject:
```
**Open Question:** [specific description of the gap]
-->
```

### Proposal Splitting

After gap detection, count the features in the proposal. Apply the split heuristic:

- **Trigger**: ≥ 10 implementation steps that can be implemented before and independently of the remaining features.
- **Do not split** if the features are tightly coupled (shared data structures, shared configuration, interleaved implementation steps).

Split procedure:
1. Identify the independent group and the dependent group.
2. Order so the independent group is implemented first.
3. Present the split as a decision:
   ```
   **Decision:** Split into Proposal A ([list features]) and Proposal B ([list features])?
   -->
   ```
4. If the human accepts the split: create new proposal files, update the history directory, and mark the parent proposal as split in its frontmatter.

### Issues to Watch Out For

- **History linking**: When splitting, both new proposals must reference the parent history directory. Use a `split-from` field in their frontmatter.
- **Gap detection false positives**: Not every feature needs explicit error case documentation. Features that are pure refactors or documentation-only changes can skip the error case check.
- **Circular dependencies**: When evaluating independence for splitting, check for shared types, shared constants, or shared test fixtures that would force a specific implementation order.

### Updated Revise Skill Procedure

Add the following steps to the Revise skill, between step 6 (scan for contradictions) and step 7 (save):

**Step 6a — Gap Detection:**
1. For each feature section, apply the gap detection checklist above.
2. For each uncovered item, add an Open Question prompt in the relevant section.
3. If any open questions were added, note this in the summary.

**Step 6b — Split Evaluation:**
1. Count extractable, independent implementation steps.
2. If ≥ 10: draft a split proposal and present it as a decision point.
3. If < 10: continue with the single proposal.
