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
| Retained active keys after one-over replay | 0 excess keys | 0 excess keys | Test (TC-002) |

## Verification

The executable tests submit exactly-at-limit and one-over inputs and inspect the
returned disposition and retained-state counters.
