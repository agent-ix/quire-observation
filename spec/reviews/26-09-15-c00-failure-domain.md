---
id: SR-039
title: "Failure-domain review of C00 revisioned observation authority"
type: SpecReview
analysis: failure-domain
scope: "FR-007..FR-011; NFR-003; IT-002"
review_set: all
---
# SR-039: Failure-domain review of C00 revisioned observation authority

## Summary

The review covered extension failures, identity keys, evaluator purity, graph
termination and adverse topology. Three gaps were found and resolved without
moving temporal or protocol semantics into QObs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-041 | high | Resolved: branch/stale-lineage refusal was impossible to validate from only a predecessor bundle; FR-009 now requires a strict-read bounded current-lineage view and limits its guarantee to that supplied view. | FR-009; TC-009 | missing-requirement |
| FND-042 | medium | Resolved: FR-011 named a duplicate policy without closing its vocabulary or excluding transport receipts from both policies. | FR-011; TC-011 | missing-requirement |
| FND-043 | medium | Resolved: FR-010 did not state what happens when the selected evaluator refuses, fails, is unsupported or exhausts its budget. | FR-010; TC-010 | missing-requirement |

## Domain Results

- Every authority, subject, scope, clock, revision, fact, relationship and result
  uses an explicit identity domain; presentation and ingestion fields are never
  substitutes.
- Dependency traversal terminates through prechecked node/edge/work bounds and a
  finite visited set; cycles remain explicit adverse inputs.
- Evaluator contributions are opaque owner results. Item-local failure preserves
  unaffected bytes and cannot produce a replacement result.
- QObs remains pure with respect to transport and persistence. Atomic concurrent
  publication is explicitly assigned to an external compare-and-swap store.

## Verdict

PASS. No unresolved failure-domain gap remains in the reviewed scope.
