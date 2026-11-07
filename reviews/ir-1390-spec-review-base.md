---
id: SR-064
title: "spec-review/base review of IR-1390 structural association identity"
type: SpecReview
analysis: base
scope: "agent-ix/quire-observation; IR-1390; src/authority/progress.rs, tests/authority.rs, schemas/observation-progress-assertion-v1.schema.json, spec/functional/FR-004-publish-observation-authority-artifacts.md, spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md, spec/test-cases/TC-006-finite-topology-and-anchored-boundaries.md"
review_set: subset
---

## Summary

Ticket: IR-1390. Reviewed the changed FR-004 and FR-006 criteria and TC-006 procedure for testability, consistency, and coverage. No defect found.

## Verdict

**PASS** — Reviewed the changed FR-004 and FR-006 criteria and TC-006 procedure for testability, consistency, and coverage. No defect found.

## Scope

- FR-004-AC-4 (examined): Exact and one-over byte/depth/string/population/position/capture/source/work bounds admit or resource-refuse before excess retention.
- FR-004-AC-7 (examined): A progress view exposes owner asserted QSL association rows as a canonical value when present. Each row names the selected model, structural model node, typed claim clause node and occurrence, time source kind, tick operation and exact positive period when applicable, structural unit and dimension nodes, and an observation source and clock/revision admitted in the same progress context. Missing rows refuse with `MissingPremise`; foreign source/clock refuse with `ExpectedMismatch`; malformed or duplicate rows refuse with `InvalidSelection`, noncanonical structural node text with `NonCanonical`, and one-over node or value bytes with `ResourceIncomplete`. Strict reading compares the exact selected rows. The observation owner asserts these links; QSL checks its own semantic selections independently against the rows.
- FR-006-AC-9 (examined): Association model, clause, tick-operation, unit and dimension nodes round-trip as real structural NodeKeys with canonical package/path/ordinal text; the claim retains its exact role and ordinal. Empty package/path, malformed segment or out-of-range ordinal refuses with `InvalidSelection`, an alternate spelling with `NonCanonical`, and one-under node-byte bounds with `ResourceIncomplete`; an exact byte bound admits.
- FR-006-AC-8 (context only): The prerelease progress v1 contract includes FR-004's owner-issued association because no external progress v1 consumer exists; its exact FR-289 boundary mapping remains intact, closure v1 retains its boundary mapping, and neither publishes a traversal, replay, settlement or truth API.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
