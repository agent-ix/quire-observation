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
  declared record order, and retention limits.

## Outputs

- Deterministic observation progress, closure, completeness and availability
  assertions with their exact supporting observation identities.
- An immutable correction link where a late record changes an observation
  authority artifact.
- An ambiguous-order outcome when the declared record order is not a total order.
- A retained-event count for the assessed history.

## Behavior

- The library SHALL preserve event time, ingestion time, clock family, and
  progress authority as separate inputs.
- When a deadline is silent, the library SHALL report coverage only when an
  eligible matching progress assertion covers that deadline; the result owner
  decides whether that coverage settles anything.
- When a record arrives after settlement, the library SHALL inspect the full
  bounded history and retain it as late even when an earlier record was
  decisive.
- When a late record contradicts an observation premise, the library SHALL emit
  a linked correction or invalidation of the affected observation authority
  artifact instead of overwriting prior bytes.
- The library SHALL retain agreeing and irrelevant late records without
  manufacturing a result correction.
- If two records share one declared order position, the library SHALL report an
  ambiguous-order outcome distinct from an unsupported profile.
- When an outcome is unavailable, the library SHALL report no retained event.
- The library SHALL preserve open prefix, incomplete history, ambiguous
  membership, ambiguous order, unsupported profile, and exhausted state as
  distinct outcomes.

## Error Conditions

When progress scope or clock is mismatched, or history, membership, declared
order, profile or retained state is invalid, the library SHALL return the
applicable explicit non-success outcome without settling truth.

The declared record order is the only ordering authority here, so two records
sharing one position is an ambiguous order rather than an unsupported profile.
A duplicated record *identity* is refused at admission under FR-001; a duplicated
order *position* is this requirement's ambiguous-order outcome.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | Batch and incremental paths derive byte-identical observation authority artifacts for the same admitted history and selections. | Test (TC-002) |
| FR-002-AC-2 | A quiet deadline reports coverage only under declared progress authority and emits no truth or settlement value. | Test (TC-002) |
| FR-002-AC-3 | Open, incomplete, ambiguous-membership, late, and exhausted cases do not become Boolean or protocol/temporal results. | Test (TC-002) |
| FR-002-AC-4 | Two records sharing one declared order position report an ambiguous-order outcome distinct from an unsupported profile. | Test (TC-002) |

## Dependencies

- [FR-001](FR-001-qualify-observation-admission.md) supplies qualified records
  and declared scope inputs.
- The temporal interpretation is caller-selected; this requirement does not
  parse or compile a temporal profile.
