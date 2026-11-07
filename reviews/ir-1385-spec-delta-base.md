---
id: SR-060
title: "Spec delta base review of IR-1385 FR-006 continuity"
type: SpecReview
analysis: base
scope: "agent-ix/quire-observation; spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md; spec/test-cases/TC-006-finite-topology-and-anchored-boundaries.md"
review_set: subset
---

## Summary

Ticket: IR-1385. The FR-006 continuity amendment correctly limits progress v1's new association to FR-004 and retains the exact FR-289 mapping and closure behavior. TC-006 still instructs a schema-digest pin for progress and closure, which conflicts with this progress schema change and is a prohibited file-tracking check.

## Verdict

**FAIL** — remove TC-006 procedure step 5's schema-digest pin instruction; retain structural boundary and public-surface evidence.

## Scope

- FR-006 continuity paragraph (examined): The accepted `TemporalBoundary` representation and the progress/closure v1 owner schemas carry the complete FR-289 mapping. FR-004 adds the owner-issued association to progress v1 because this prerelease contract has no external progress v1 consumer. The association SHALL preserve the exact FR-289 boundary mapping and SHALL NOT add a replay/evaluation API, traversal API, or the superseded result model. Exact admission and owner-derivation tests cover the boundary behavior.
- FR-006-AC-8 (examined): The prerelease progress v1 contract includes FR-004's owner-issued association because no external progress v1 consumer exists; its exact FR-289 boundary mapping remains intact, closure v1 retains its boundary mapping, and neither publishes a traversal, replay, settlement or truth API.
- TC-006 procedure step 5 (examined): Pin the progress/closure v1 schema digests and review the public surface for absence of traversal, replay, settlement and truth APIs.
- FR-004-AC-7 (context only): A progress view exposes owner asserted QSL association rows as a canonical value when present.
- FR-289 (context only): The adapter selects and retains one explicit boundary mapping for native inclusive intervals over half-open observation carriers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-006 step 5 requires progress/closure schema-digest pins although FR-006-AC-8 now admits a progress v1 schema change; remove the pin instruction and retain structural boundary/public-surface checks. | TC-006; FR-006-AC-8 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | TC-006 step 5 now compares progress and closure boundary views with independently selected scope and clock, and still checks the public surface. The schema-digest pin instruction is gone. |
