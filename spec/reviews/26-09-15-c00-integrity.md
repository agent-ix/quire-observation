---
id: SR-040
title: "Integrity review of the C00 observation specification"
type: SpecReview
analysis: integrity
scope: "spec/spec.md; US-001; FR-001, FR-007..FR-011; NFR-003; TM-001"
review_set: all
---
# SR-040: Integrity review of the C00 observation specification

## Summary

Completeness, consistency, atomicity and testability were checked across the C00
trace chain. The specification is internally consistent after separating packed
modal clauses and making lineage and bounded-work evidence explicit.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-044 | medium | Resolved: several descriptions and behavior bullets packed two `SHALL` clauses, obscuring one-to-one review and test mapping. | FR-001; FR-008; FR-010; FR-011 | wrong-requirement |
| FND-045 | high | Resolved: FR-009 claimed stale/branch rejection without declaring the lineage state needed to make that decision observable. | FR-009 | missing-requirement |
| FND-046 | medium | Resolved: NFR-003's unbounded-expansion criterion used generic Analysis without a separately typed static test and inventory. | NFR-003-AC-1; TC-013 | correct-requirement-no-evidence |

## Traceability

| User need | Functional realization | Constraint | Verification |
|---|---|---|---|
| US-001 partial/time authority | FR-007, FR-008 | NFR-003 | TC-007, TC-008, TC-012, TC-013 |
| US-001 immutable revisions | FR-009 | NFR-003 | TC-009, TC-012, TC-013 |
| US-001 affected-region repair | FR-010 | NFR-003 | TC-010, TC-012, TC-013 |
| US-001 closed population queries | FR-011 | NFR-003 | TC-011, TC-012, TC-013 |
| Cross-owner handoff | IT-002 | NFR-003 | IT-002-SC-01..IT-002-SC-05 |

No external CLI, registry lookup, paginated API or authenticated remote API is
part of a production requirement. The only callback-like seam is the explicitly
selected typed evaluator contribution covered by FR-010's item-local failure rule.

## Verdict

PASS. Every reviewed obligation has one observable interpretation and a mapped
verification route.
