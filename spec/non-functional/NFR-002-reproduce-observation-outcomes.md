---
id: NFR-002
title: "Reproduce qualified observation outcomes"
type: NFR
quality_attribute: reliability
relationships:
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "constrains"
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "constrains"
---
# NFR-002: Reproduce qualified observation outcomes

## Statement

The library SHALL produce the same result identity and disposition for repeated
evaluation of the same ordered records, selections, progress assertions, and
limits.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Repeated replay result equality | 100% of fixed fixtures | 100% of fixed fixtures | Test (TC-002) |
| Result dependency retention | 100% of required dependencies | 100% of required dependencies | Test (TC-003) |

## Verification

The tests execute each deterministic fixture twice and compare the full typed
result, including dependencies and provenance.
