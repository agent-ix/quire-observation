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
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "constrains"
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "constrains"
---
# NFR-001: Bound retained observation state

## Statement

The library SHALL enforce caller-supplied record, member, relationship,
required-relationship, byte, depth, string, population, position, capture,
source, and visited-work bounds before retaining an input beyond its bound.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Retained records after one-over admission | 0 excess records | 0 excess records | Test (TC-001) |
| Retained members after one-over admission | 0 excess members | 0 excess members | Test (TC-001) |
| Retained relationships after one-over admission | 0 excess relationships | 0 excess relationships | Test (TC-001) |
| Retained bytes/depth/strings after one-over strict read | 0 excess work | 0 excess work | Test (TC-004) |
| Retained semantic population after one-over derivation | 0 excess entries | 0 excess entries | Test (TC-004) |
| Partial view returned after resource refusal | 0 views | 0 views | Test (TC-004) |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-001-AC-1 | Admission retains no record beyond its declared record bound. | Test (TC-001) |
| NFR-001-AC-2 | Admission retains no member beyond its declared member bound. | Test (TC-001) |
| NFR-001-AC-3 | Exact byte, depth, string and visited-work limits admit; one-over inputs resource-refuse before a view exists. | Test (TC-004) |
| NFR-001-AC-4 | Exact population, position, capture and source limits admit; one-over selections resource-refuse before a document exists. | Test (TC-004) |
| NFR-001-AC-5 | Incremental history cannot retain an out-of-order, excess or incomplete record sequence. | Test (TC-002) |
| NFR-001-AC-6 | Exact relationship and required-relationship limits admit; one-over inputs refuse before relationship matching. | Test (TC-001) |

## Verification

The executable tests submit exactly-at-limit and one-over inputs to admission,
including both relationship populations, incremental history, every semantic
artifact population and the shared strict reader. Resource errors carry usage
only and never a partial document or view.
