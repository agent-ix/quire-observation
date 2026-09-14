---
id: Task-012
title: "#15 — complete history and strict-reader integration"
type: Task
status: done
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-observation/Task-010, type: depends_on }
  - { target: ix://agent-ix/quire-observation/Task-011, type: depends_on }
  - { target: ix://agent-ix/quire-observation/FR-002, type: references }
  - { target: ix://agent-ix/quire-observation/FR-003, type: references }
  - { target: ix://agent-ix/quire-observation/NFR-002, type: references }
  - { target: ix://agent-ix/quire-observation/TC-002, type: verifies }
  - { target: ix://agent-ix/quire-observation/TC-003, type: verifies }
  - { target: ix://agent-ix/quire-observation/IT-001, type: verifies }
---
# Task-012: #15 — complete history and strict-reader integration

## Scope

Prove deterministic equality across batch/checked incremental histories and preserve every
owner fact through the strict public consumer boundary.

## Subtasks

- [x] Refuse early finish, excess records, and out-of-order incremental substitution.
- [x] Compare complete bytes for all nine contracts across both history paths.
- [x] Mutate every required field and cross-wire every owner-reader boundary.
- [x] Exercise all 48 progress/closure/completeness/availability state combinations.
- [x] Inspect every public payload accessor and retain exact envelope selections.

## Deliverables

- `tests/authority.rs` with TC-002, TC-003, TC-004, NFR, and IT-001 traces.
- Complete 31/31 Quire matrix coverage.

## Notes

- Tests execute real library paths and use no mocks.
