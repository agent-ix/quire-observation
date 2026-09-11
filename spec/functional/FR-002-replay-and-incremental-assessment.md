---
id: FR-002
title: "Assess qualified records consistently in replay and incrementally"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
---
# FR-002: Assess qualified records consistently in replay and incrementally

## Description

When a caller supplies qualified records and declared progress, the
`quire-observation` library SHALL produce the same assessment disposition from
batch replay and incremental processing for the same admitted history.

## Inputs

- Qualified records from [FR-001](FR-001-qualify-observation-admission.md).
- A caller-selected temporal assessment profile, progress/closure assertions,
  ordering policy, and retention limits.

## Outputs

- A deterministic typed disposition and its decision support.
- A late-record supersession/invalidation link where applicable.

## Behavior

- The library SHALL preserve event time, ingestion time, clock family, and
  progress authority as separate inputs.
- When a deadline is silent, the library SHALL settle it only when an eligible,
  matching progress assertion covers that deadline.
- When a record arrives after settlement, the library SHALL retain it as late.
- When a late record contradicts a result, the library SHALL emit a linked
  supersession or invalidation instead of overwriting the earlier result.
- The library SHALL preserve open prefix, incomplete history, ambiguous
  membership, unsupported profile, and exhausted state as distinct outcomes.

## Error Conditions

Mismatched progress scope or clock, unavailable history, ambiguous membership,
unsupported profile, and exhausted retained state SHALL be explicit non-success
outcomes and SHALL NOT settle truth.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | Batch and incremental paths agree for decisive satisfaction, violation, and missed deadline. | Test (TC-002) |
| FR-002-AC-2 | A quiet deadline settles only under declared progress authority. | Test (TC-002) |
| FR-002-AC-3 | Open, incomplete, ambiguous, late, and exhausted cases do not become violation or success. | Test (TC-002) |

## Dependencies

- [FR-001](FR-001-qualify-observation-admission.md) supplies qualified records
  and declared scope inputs.
- The temporal interpretation is caller-selected; this requirement does not
  parse or compile a temporal profile.
