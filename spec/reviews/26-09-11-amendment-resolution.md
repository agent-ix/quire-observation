---
id: SR-009
title: "Resolution of the amendment review findings"
type: SpecReview
analysis: base
scope: "SR-004..SR-008 findings; spec/spec.md; spec/functional/FR-001, FR-002, FR-003; spec/non-functional/NFR-001; spec/test-cases/TC-001, TC-002, TC-003; spec/tests.md"
review_set: subset
---
## Summary

The base checklist and four analyses — integrity (SR-005), EARS conformance (SR-006),
evidence (SR-007) and scope boundary (SR-008) — raised 40 findings against the amendment in
commit `18e8f67`. This document records the resolution of each group. Four lenses converged
on the same five defects, which is why they are resolved as one change rather than five: the
amendment split FR-003's scope axis in the Behavior list but left the Description, Inputs,
test case, and capability rule describing the collapsed model it replaced.

Three findings are recorded as knowing non-escalations and one is deferred to the issue that
owns the document it concerns. Everything else is resolved in the spec.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-001 | high | Resolved: the amendment separated the four scope facts in FR-003's Behavior list only. The Description, the Inputs, the capability rule and TC-003's axis list all still described the collapsed model, so the document both required and did not require the separation. All four now name the same seventeen axes, with the four scope facts defined where they are introduced. | SR-005/FND-002, SR-005/FND-008, SR-006/FND-001, SR-008/FND-004; FR-003; TC-003 | wrong-requirement |
| FND-002 | high | Resolved: the ambiguous-order outcome was authored only as rationale under FR-002's Error Conditions. It now has a Behavior obligation in If/then form with the library as subject, an entry in the distinct-outcomes bullet, an Outputs entry, and its own criterion `FR-002-AC-4`. FR-002's Inputs no longer claim a "caller-selected ordering policy" that contradicted the declared-order authority. | SR-005/FND-001, SR-005/FND-003, SR-005/FND-004, SR-005/FND-011, SR-006/FND-005, SR-006/FND-006, SR-008/FND-006; FR-002 | wrong-requirement |
| FND-003 | high | Resolved: `NFR-001-AC-3` joined a bound obligation to an unavailable-assessment reporting obligation under one id, so a test proving only the bound would have marked both backed. AC-3 is now the event bound on both paths, `NFR-001-AC-5` is the reporting obligation, each has its own metric row, and TC-002's procedure gained the steps that produce the evidence — an event bound, exactly-at-limit and one-over histories on both paths, and an unassessable outcome whose retained-event count is read. | SR-004/atomicity note, SR-005/FND-009, SR-006/FND-009, SR-007/FND-001, SR-007/FND-002, SR-007/FND-003, SR-007/FND-004, SR-007/FND-005, SR-007/FND-008; NFR-001; TC-002 | correct-requirement-no-evidence |
| FND-004 | medium | Resolved: FR-003's "a single capability SHALL NOT stand for two facts a consumer can represent independently" predicated an in-scope obligation on an out-of-scope consumer's internal representation, which this library cannot observe. It is restated as two statements about what the library accepts and reports: one independently settable capability per axis, and no capability standing for two axes. | SR-005/FND-007, SR-006/FND-002, SR-006/FND-003, SR-006/FND-004, SR-008/FND-003; FR-003 | wrong-requirement |
| FND-005 | medium | Resolved: FR-001's new bullets named no observable consequence and packed two obligations into one statement. "Caller-opaque" is replaced by what the library may and may not do with a member's object identity (opaque equality key; no parsing, normalizing or deriving), the digest bullet is split into retention and non-recomputation, and the embedded "because …" rationale moved out of both statements into non-normative prose. `FR-001-AC-5` verifies the boundary. | SR-005/FND-005, SR-005/FND-006, SR-006/FND-007, SR-006/FND-008, SR-006/FND-010, SR-004/FND-001; FR-001 | missing-requirement |
| FND-006 | medium | Resolved: declining membership and closure canonicalization left the responsibility unallocated rather than delegated. FR-001's Dependencies now names `agent-ix/quire-spec-language` as the owner — it exports the `native-linked-package/1` selection with its canonical digests under its FR-015 and FR-027 — and records that `quire-protocol`'s canonicalization governs protocol result identity, not population membership. `spec.md` Out of Scope states the same boundary so the master document and the requirement agree. | SR-008/FND-001, SR-008/FND-002; FR-001; spec.md | missing-requirement |
| FND-007 | medium | Resolved: the retained-event axis entered the handoff with no declared input, and the matrix lost `NFR-001-AC-1`'s trace when the NFR cells moved to criterion level. FR-003's Inputs now names the retained-event count as coming from FR-002's result, FR-002's Outputs names it, and TC-001's *Traces To* cell carries `NFR-001-AC-1` again. | SR-005/FND-010, SR-008/FND-004, SR-008/FND-005; FR-003; FR-002; spec/tests.md | correct-requirement-no-evidence |
| FND-008 | low | Resolved: TC-002 asserted that *only* a contradicting late record links a supersession, which no requirement obliged. FR-002's Behavior now states the exclusivity, so the test case asserts no more than the requirement. This is the obligation the current code violates; the fix is tracked in the supersession issue. | SR-005/FND-012; FR-002; TC-002 | missing-requirement |
| FND-009 | low | Resolved: one axis and one bound were each named three ways ("retained event" / "retained events" / "retained-event"; "population-member" / "members" / "member"). The amendment now uses `retained-event` for the axis, "retained event" for the quantity, and "member" for the bound throughout, and the four scope axes are defined where FR-003 introduces them. | SR-005/FND-013, SR-006/FND-011; FR-003; NFR-001 | wrong-requirement |

## Knowing non-escalations

- `quoin advise` recommends `performance-benchmarking` for all five NFR-001 metric rows on
  the `quantified-threshold` characteristic alone. These are correctness counts with a
  zero-excess threshold, not latency or throughput figures, so the authored `Test` method
  stands (SR-007/FND-006).
- The catalog's `fault-injection` method fits `reliability`, NFR-001's declared quality
  attribute, but was offered for no obligation. Exhausting a declared bound is closer to
  fault injection than to a fixed-value unit test; this is a suite-planning note, not a
  change to any Verification cell (SR-007/FND-007).
- `NFR-001-AC-4` and `NFR-001-M-4` are already supported by TC-001's existing one-over bound
  procedure and drew no finding (SR-007).

## Deferred

- `gap_analysis.md` still describes the four scope facts as one "scope facts" axis
  (SR-005/FND-013, second clause). That document is rewritten by the issue that corrects the
  coverage record, so it is not touched here.

## Coverage after resolution

`quire validate` reports 20/20 documents grammar-clean. `quire coverage` reports 18 of 23
trace targets backed, with five acceptance criteria deliberately unbacked until their tests
land: `FR-001-AC-5`, `NFR-001-AC-3`, `NFR-001-AC-4` and `NFR-001-AC-5` with the
retained-state work, and `FR-002-AC-4` with the ambiguous-order work. `unbacked_rows` and
`status_lies` are both empty; the status column on the functional coverage table remains
unclassifiable for the upstream reason recorded in `agent-ix/quoin#368`.
