---
id: Task-019
title: "FR-010 — bounded affected-region planning"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-018
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-010
    type: references
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-010
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-012
    type: verifies
---
# Task-019: FR-010 — bounded affected-region planning

## Scope

Derive a canonical finite repair plan from validated replacements and explicit dependency
edges while retaining every unaffected result byte-for-byte.

## Subtasks

- [x] Write TC-010 closure-oracle, cycle, unknown-identity and one-over tests first.
- [x] Seed invalidation only from FR-009 replacement relations.
- [x] Traverse explicit edges with checked node/edge/work accounting and cycle safety.
- [x] Filter dependents by exact observable scope/window and order only for scheduling.
- [x] Retain unaffected identities/bytes and refuse without a partial plan.

## Deliverables

- Pure repair-planner module and trace-tagged property tests.
- Canonical affected/unaffected inventories and typed exhausted/refused outcomes.

## Notes

- Scheduling order is not event causality.
- Unblocks Task-020.
