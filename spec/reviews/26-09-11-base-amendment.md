---
id: SR-004
title: "Base review of the corrected result-model amendment"
type: SpecReview
analysis: base
scope: "spec/functional/FR-001, FR-002, FR-003; spec/non-functional/NFR-001; spec/test-cases/TC-002, TC-003; spec/tests.md"
review_set: subset
---
## Summary

The base checklist reviewed the amendment recorded in commit `18e8f67`, which splits
FR-003's single scope-facts consumer axis into four independently readable facts, adds the
retained-event axis, mints `NFR-001-AC-3` and `NFR-001-AC-4` for two bounds the NFR
statement already claimed, names an ambiguous-order outcome in FR-002, and states the
previously silent admission boundary in FR-001. ID formats, acceptance-criterion
numbering, typed links and the error-path and constraint-boundary coverage rules hold.
One coverage rule does not: FR-001 gained normative behavior with no acceptance criterion.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | medium | FR-001's two new Behavior bullets — a caller-opaque member object identity, and membership/closure digests retained without recomputation — carry no acceptance criterion, so the boundary is stated but unverifiable. Mint `FR-001-AC-5` and bind it in the admission tests. | FR-001 Behavior; FR-001-AC-1..4; TM-001 | missing-requirement |
| FND-002 | low | The functional coverage row for FR-001 cites `FR-001-AC-1..4`; minting `FR-001-AC-5` per FND-001 requires that range and TC-001's *Traces To* cell to be extended in the same change. | TM-001; spec/tests.md | correct-requirement-no-evidence |

## Checklist results

- **ID format and uniqueness** — pass. `NFR-001-AC-3` and `NFR-001-AC-4` continue the
  existing sequence with no duplicate or gap; `quire coverage` mints both, and the four
  `NFR-001-M-*` metric obligations resolve to `TC-002` and `TC-001` respectively.
- **Acceptance-criterion atomicity** — `NFR-001-AC-3` states two things in one criterion
  (no event retained beyond the declared bound, and an unavailable assessment reports no
  retained event). Both halves are the same bound measured at its two exits, so it is
  recorded here rather than raised as a finding or split.
- **Functional requirement quality** — one failure, FND-001. Inputs, outputs, behavior,
  error conditions and dependencies are otherwise complete for all three amended FRs; the
  new FR-002 error condition carries its own rationale (the caller-declared sequence is the
  only ordering authority).
- **Test coverage, six rules** — coverage holds for every acceptance criterion except the
  missing one in FND-001. Error path: FR-002's new ambiguous-order condition is named in
  TC-002's Description and Expected Results. Constraint boundary: both new bounds are
  one-over cases already described by TC-001 and TC-002. State transition: the
  contradicting-versus-agreeing late record distinction is now stated in TC-002's Expected
  Results, which is what makes the supersession link testable.
- **Cross-referencing** — pass. FR→US links unchanged; TC-001 and TC-002 *Traces To* cells
  carry the new criterion ids; terminology for the four scope facts matches the names the
  handoff type must expose.

## Known and tracked, not findings

- `NFR-001-AC-3` and `NFR-001-AC-4` are unbacked on this commit. Their tests land with the
  retained-events work; coverage reads 18/20 by design until then.
- The functional coverage table's status column cannot be classified by the active trace
  configuration. That is an upstream module defect, `agent-ix/quoin#368`, and no local
  change can fix it without failing structural validation.
