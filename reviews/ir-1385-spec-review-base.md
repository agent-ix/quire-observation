---
id: SR-059
title: "spec-review/base review of IR-1385 progress association"
type: SpecReview
analysis: base
scope: "agent-ix/quire-observation; IR-1385; spec/functional/FR-004-publish-observation-authority-artifacts.md"
review_set: subset
---

## Summary

Ticket: IR-1385. FR-004 association text and AC-7 define the row shape, owner/QSL responsibility split, typed refusal outcomes, and canonical value. The changed requirement is concrete and internally consistent.

## Verdict

**PASS** — FR-004 association text and AC-7 define the row shape, owner/QSL responsibility split, typed refusal outcomes, and canonical value. The changed requirement is concrete and internally consistent.

## Scope

- FR-004-AC-7 (examined): A progress view exposes owner asserted QSL association rows as a canonical value when present. Each row names the checked model, model node, claim, time source kind, tick operation and exact positive period when applicable, unit and dimension nodes, and an observation source and clock/revision admitted in the same progress context. Missing rows refuse with `MissingPremise`; foreign source/clock refuse with `ExpectedMismatch`; malformed or duplicate rows refuse with `InvalidSelection`. Strict reading compares the exact selected rows. The observation owner asserts these links; QSL checks its own semantic selections independently against the rows.
- FR-004 (context only): When qualified immutable observation state is exported, the library SHALL emit canonical bounded owner documents and expose only constructor-private validated views through strict public readers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
