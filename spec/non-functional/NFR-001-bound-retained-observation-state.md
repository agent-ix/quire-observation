---
id: NFR-001
title: "Bound retained observation state"
type: NFR
quality_attribute: reliability
relationships:
  - target: "ix://agent-ix/quire-observation/FR-001"
    type: "constrains"
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "constrains"
---
# NFR-001: Bound retained observation state

## Statement

The library SHALL enforce caller-supplied record, active-key, retained-event,
and population-member bounds before retaining an input beyond its bound.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Retained records after one-over admission | 0 excess records | 0 excess records | Test (TC-001) |
| Retained active keys after one-over intake | 0 excess keys | 0 excess keys | Test (TC-002) |
| Retained events after one-over intake | 0 excess events | 0 excess events | Test (TC-002) |
| Retained members after one-over admission | 0 excess members | 0 excess members | Test (TC-001) |
| Reported retained events for an unavailable assessment | 0 events | 0 events | Test (TC-002) |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-001-AC-1 | Admission retains no record beyond its declared record bound. | Test (TC-001) |
| NFR-001-AC-2 | Incremental intake retains no active key beyond its declared active-key bound. | Test (TC-002) |
| NFR-001-AC-3 | Batch replay and incremental intake each retain no event beyond the declared event bound. | Test (TC-002) |
| NFR-001-AC-4 | Admission retains no member beyond its declared member bound. | Test (TC-001) |
| NFR-001-AC-5 | An unavailable assessment reports no retained event. | Test (TC-002) |

## Verification

The executable tests submit exactly-at-limit and one-over inputs on both the
batch and incremental paths and inspect the returned disposition and
retained-state counters. A further test drives an outcome the library cannot
assess and inspects the reported retained-event count, which must be zero.
