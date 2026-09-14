---
id: FR-002
title: "Derive observation authority consistently"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
  - target: "ix://agent-ix/quire-observation/FR-001"
    type: "depends_on"
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "depends_on"
---
# FR-002: Derive observation authority consistently

## Description

When a caller supplies the same qualified history and complete owner selections,
the `quire-observation` library SHALL derive byte-identical observation-authority
artifacts through batch and checked incremental intake.

## Inputs

- Qualified records from [FR-001](FR-001-qualify-observation-admission.md).
- Explicit owner-definition, subject, position, clock, capture, progress,
  closure, completeness, availability, revision, predecessor, and
  resource-limit selections.

## Outputs

- Deterministic canonical observation, population, position, clock, capture,
  progress, closure, completeness and availability documents.
- An immutable direct-predecessor link for every selected correction revision.
- An ambiguous-order outcome when the declared record order is not a total order.
- A typed refusal without a partial artifact when any selection is invalid,
  cross-wired, incomplete, noncanonical, or over an effective bound.

## Behavior

- The library SHALL preserve event time, ingestion time, clock identity,
  clock revision, uncertainty, and progress authority as separate facts.
- When accepting incremental history, the library SHALL accept exactly the next
  record from the already-qualified history and SHALL finish only after the full
  history was supplied.
- Owner derivation SHALL preserve independent progress, closure, completeness,
  availability, and lateness vocabularies without evaluating temporal or
  protocol truth.
- A correction SHALL have a greater positive revision, a new derived identity,
  and the exact direct predecessor identity; prior bytes remain immutable.
- If two records share one declared order position, the library SHALL report an
  ambiguous-order error rather than inferring order from timestamps.
- Missing, open, late, contradicted, unavailable, and exhausted conditions SHALL
  remain typed owner facts or refusals and SHALL NOT become Boolean results.

## Error Conditions

When history, authority, scope, population, clock, boundary, order, correction
lineage, or limits are invalid, the library SHALL return the applicable typed
error without a partial document or view.

The declared position ledger is the only ordering authority here. A duplicated
record *identity* is refused at admission under FR-001; a duplicated order
*position* is this requirement's ambiguous-order error.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | Batch and incremental paths derive byte-identical observation authority artifacts for the same admitted history and selections. | Test (TC-002) |
| FR-002-AC-2 | Checked incremental intake accepts only the exact next qualified record and refuses early finish, excess input, or out-of-order substitution without partial history. | Test (TC-002) |
| FR-002-AC-3 | Progress, closure, completeness, availability, lateness, and resource exhaustion remain independent typed facts or errors and never become Boolean or protocol/temporal results. | Test (TC-002) |
| FR-002-AC-4 | Two records sharing one declared order position report an ambiguous-order error without timestamp inference. | Test (TC-002) |

## Dependencies

- [FR-001](FR-001-qualify-observation-admission.md) supplies qualified records
  and declared scope inputs.
- FR-004 defines the exact owner contracts. This requirement does not parse,
  compile, or evaluate temporal or protocol definitions.
