---
id: SR-061
title: "Spec delta integrity review of IR-1385 FR-006 continuity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-observation; spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md; spec/functional/FR-004-publish-observation-authority-artifacts.md"
review_set: subset
---

## Summary

Ticket: IR-1385. The FR-006 continuity paragraph and AC-8 consistently scope the permitted progress v1 association addition to FR-004 while keeping FR-289 boundary mapping and closure v1 boundary behavior. The TC-006 digest instruction is recorded separately in the base review.

## Verdict

**PASS** for continuity, association ownership, boundary mapping and closure semantics.

## Scope

- FR-006 continuity paragraph (examined): The accepted `TemporalBoundary` representation and the progress/closure v1 owner schemas carry the complete FR-289 mapping. FR-004 adds the owner-issued association to progress v1 because this prerelease contract has no external progress v1 consumer. The association SHALL preserve the exact FR-289 boundary mapping and SHALL NOT add a replay/evaluation API, traversal API, or the superseded result model. Exact admission and owner-derivation tests cover the boundary behavior.
- FR-006-AC-8 (examined): The prerelease progress v1 contract includes FR-004's owner-issued association because no external progress v1 consumer exists; its exact FR-289 boundary mapping remains intact, closure v1 retains its boundary mapping, and neither publishes a traversal, replay, settlement or truth API.
- FR-004-AC-7 (context only): A progress view exposes owner asserted QSL association rows as a canonical value when present.
- FR-006-AC-4 through AC-7 (context only): The existing clock-family, carrier, scope, point interval and strict-watermark boundary criteria remain unchanged.
- FR-289 (context only): Inclusive temporal mapping retains the exact shared clock, native interval, half-open carrier and strict progress coverage of the inclusive upper bound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
