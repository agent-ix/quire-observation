---
id: SR-005
title: "Integrity review of the corrected result-model amendment"
type: SpecReview
analysis: integrity
scope: "spec/functional/FR-001-qualify-observation-admission.md, spec/functional/FR-002-replay-and-incremental-assessment.md, spec/functional/FR-003-preserve-observation-result-handoff.md, spec/non-functional/NFR-001-bound-retained-observation-state.md, spec/test-cases/TC-002-replay-incremental-agreement.md, spec/test-cases/TC-003-result-handoff.md, spec/tests.md"
review_set: subset
---
## Summary

This integrity gate examined only the amendments in commit 18e8f67: two new
FR-001 Behavior bullets, FR-002's ambiguous-order Error Conditions text, FR-003's
split scope axis list with its one-capability-per-fact bullet, NFR-001-AC-3,
NFR-001-AC-4 and their two metric rows, the TC-002 and TC-003 rewrites, and the
changed Traces To cells in spec/tests.md. The ambiguous-order outcome does not
collide with the existing unsupported-profile or exhausted-state outcomes, which
stay disjoint in both the Error Conditions prose and the TC-002 fixture list, but
the new text is under-propagated: three requirements now state an obligation in
one section that their own Behavior list, acceptance table, or sibling test case
still contradicts, and two new obligations are written so that no test can fail
them.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-002 Inputs still accept a "caller-selected ordering policy" while the new Error Conditions paragraph asserts "the caller-declared order is the only ordering authority"; a reader cannot tell whether a declared sequence or a selected policy orders records, so the new ambiguous-order outcome has two valid interpretations. | FR-002 Inputs; FR-002 Error Conditions |
| FND-002 | high | FR-003's Description still preserves collapsed "scope progress/closure" facts while its Behavior list requires four independently readable facts with their own capability each; the requirement both requires and does not require independent representability of the axes the amendment exists to separate. | FR-003 Description; FR-003 Behavior |
| FND-003 | medium | FR-002's Behavior bullet enumerating distinct outcomes ("open prefix, incomplete history, ambiguous membership, unsupported profile, and exhausted state") was not extended with the ambiguous-order outcome added to Error Conditions, so the same document lists two different distinct-outcome sets. | FR-002 Behavior; FR-002 Error Conditions |
| FND-004 | medium | No acceptance criterion covers the ambiguous-order outcome: FR-002-AC-3's bare "ambiguous" now stands for both ambiguous membership and ambiguous order, although TC-002 carries them as separate fixtures, so the new outcome is untestable at criterion level and invisible to the Test Matrix. | FR-002-AC-3; TC-002; TM-001 |
| FND-005 | medium | At 18e8f67 FR-001's two new Behavior bullets (caller-opaque member object identity, digests retained without recomputation) had no acceptance criterion and no TC-001 step, so nothing verified either obligation; a concurrent working-tree edit has since added FR-001-AC-5 and the matching TC-001 sentence, which closes this finding for any commit that carries them. | FR-001 Behavior; FR-001-AC-5; TC-001 |
| FND-006 | medium | "The library SHALL treat a member's object identity as caller-opaque" names no externally observable consequence in the requirement itself (no prohibition on comparison, normalization, or derivation) and the term is defined nowhere in the specification; the working tree's FR-001-AC-5 supplies one consequence (an unrecognized identity does not block admission) but the bullet remains broader than the criterion that verifies it. | FR-001 Behavior; FR-001-AC-5 |
| FND-007 | medium | FR-003's new bullet "a single capability SHALL NOT stand for two facts a consumer can represent independently" is aspirational: independent representability is a property of an unspecified consumer, and no enumeration of which fact pairs qualify exists. Only its second clause (each scope progress and scope closure fact carries its own capability and typed loss) is falsifiable. | FR-003 Behavior; TC-003 |
| FND-008 | medium | TC-003 now asserts the omission cases "cover the declared axis set exhaustively", but its own 14-item axis list differs from FR-003's 17-item Behavior list (TC-003 adds execution and truth, omits assessment identity, disposition, progress identity, and global closure), so the exhaustiveness assertion has no single authoritative axis set to close over. | TC-003 Description; TC-003 Test Procedure; FR-003 Behavior |
| FND-009 | medium | NFR-001-AC-3 is not atomic: it joins a bound obligation (no event retained beyond the declared event bound) to an unrelated reporting obligation (an unavailable assessment reports no retained event); the second clause is not a bound, belongs to FR-002's unavailable-history outcome, and has no row in the Measurement and Evaluation table. | NFR-001-AC-3; NFR-001 Measurement and Evaluation; FR-002 Error Conditions |
| FND-010 | medium | The amendment moved spec/tests.md to criterion-level NFR traces but dropped one: the TC-001 row now cites NFR-001-AC-4 only, while NFR-001-AC-1 declares Test (TC-001) as its verification and TC-002's row enumerates NFR-001-AC-2 and NFR-001-AC-3, leaving NFR-001-AC-1 with no matrix trace. | spec/tests.md TC-001 row; NFR-001-AC-1 |
| FND-011 | low | The specification does not state which stage owns a duplicated declared sequence position: FR-001 already refuses a "duplicate record" at admission, so a caller cannot tell whether two records sharing one position yield an FR-001 typed refusal or an FR-002 ambiguous-order outcome. | FR-001 Error Conditions; FR-002 Error Conditions |
| FND-012 | low | TC-002's Expected Results assert that *only* a contradicting late record produces a linked supersession and that an agreeing or irrelevant late record is retained without one; FR-002's Behavior states the contradiction case but never the exclusivity, so the test case asserts more than any requirement obliges. | TC-002 Expected Results; FR-002 Behavior |
| FND-013 | low | The four new axis names (decision progress, decision closure, surrounding progress, surrounding closure) and "retained events" are introduced without definition in spec/spec.md or any requirement, and gap_analysis.md still describes the same axes as "scope facts". | FR-003 Behavior; spec/spec.md; gap_analysis.md |

## Verification

- `git show 18e8f67` supplied the reviewed diff; no document outside the amended
  set was assessed.
- `quire validate --scope /home/peter/dev/quire-observation "spec/**/*.md"
  --summary` was run after authoring this review.
