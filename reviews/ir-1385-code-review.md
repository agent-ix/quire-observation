---
id: SR-057
title: "code-review review of IR-1385 progress association"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-observation; IR-1385; src/authority/progress.rs, schemas/observation-progress-assertion-v1.schema.json, tests/authority.rs"
review_set: subset
---

## Summary

Ticket: IR-1385. The Rust change uses qualified progress source and clock context, validates selected rows before derivation, and strict-reads against independently supplied rows. No code defect found.

## Verdict

**PASS** — The Rust change uses qualified progress source and clock context, validates selected rows before derivation, and strict-reads against independently supplied rows. No code defect found.

## Scope

- FR-004-AC-7 (examined): A progress view exposes owner asserted QSL association rows as a canonical value when present. Each row names the checked model, model node, claim, time source kind, tick operation and exact positive period when applicable, unit and dimension nodes, and an observation source and clock/revision admitted in the same progress context. Missing rows refuse with `MissingPremise`; foreign source/clock refuse with `ExpectedMismatch`; malformed or duplicate rows refuse with `InvalidSelection`. Strict reading compares the exact selected rows. The observation owner asserts these links; QSL checks its own semantic selections independently against the rows.
- FR-004 (context only): When qualified immutable observation state is exported, the library SHALL emit canonical bounded owner documents and expose only constructor-private validated views through strict public readers.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Post-review delta

Frozen head `1b7332e` removes the source-byte SHA-256 field, digest catalog, import, and hash assertion from `tests/bounded_work.rs`. The test still includes the live source text, keeps all 44 `ROWS`, and checks their production anchors, dominating bounds, trace tags, and exact/one-over evidence. The removed assertion detected any source text edit, including edits unrelated to bounded work; it did not assert a C00 bound. This delta introduces no finding and preserves the reviewed bounded-work assertions.

## Post-review code delta (b591ec3)

The deleted `tc006_progress_and_closure_v1_schemas_remain_immutable` asserted two fixed schema SHA-256 strings. It checked source bytes against a manually maintained catalog and no progress or closure boundary behavior. The three remaining TC-006 tests still assert exact timestamp, event-position and fixed-sample boundary behavior and typed mismatches. Adding `FR-006-AC-8` traces to the two boundary tests and the association value test matches the corresponding behavioral portions of that criterion. No new code or test finding.
