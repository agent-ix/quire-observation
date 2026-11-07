---
id: SR-058
title: "gap-analysis review of IR-1385 progress association"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-observation; IR-1385; spec/functional/FR-004-publish-observation-authority-artifacts.md, tests/authority.rs, src/authority/progress.rs"
review_set: subset
---

## Summary

Ticket: IR-1385. AC-7 is tagged by a real public-API test. The test exercises tick association and several typed refusals; the dense variant and lowered canonical output bound have no value assertions.

## Verdict

**CONDITIONAL** — AC-7 is tagged by a real public-API test. The test exercises tick association and several typed refusals; the dense variant and lowered canonical output bound have no value assertions.

## Scope

- FR-004-AC-7 (examined): A progress view exposes owner asserted QSL association rows as a canonical value when present. Each row names the checked model, model node, claim, time source kind, tick operation and exact positive period when applicable, unit and dimension nodes, and an observation source and clock/revision admitted in the same progress context. Missing rows refuse with `MissingPremise`; foreign source/clock refuse with `ExpectedMismatch`; malformed or duplicate rows refuse with `InvalidSelection`. Strict reading compares the exact selected rows. The observation owner asserts these links; QSL checks its own semantic selections independently against the rows.
- FR-004 (context only): When qualified immutable observation state is exported, the library SHALL emit canonical bounded owner documents and expose only constructor-private validated views through strict public readers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-7 exercises tick association only; add dense row round-trip and lowerable association canonical-byte limit assertions. | tests/authority.rs:791-909 |

## Coverage

Plan completion: not assessed. The computed matrix tags FR-004-AC-7 to `tc004_progress_association_is_canonical_and_bound_to_admitted_context`; the finding concerns its assertions.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The public API test now strict-reads a dense association, asserts its exact canonical value, and checks one-under and exact output-byte bounds. |

## Post-review code delta (b591ec3)

`FR-006-AC-8` now binds to the existing progress association value test and to the timestamp and discrete boundary tests. Those tests assert the association and retained boundary behavior. Removing the schema digest assertion does not remove behavioral evidence for the criterion. The earlier `FR-004-AC-7` finding remains fixed; no new gap finding arises from this code delta.
