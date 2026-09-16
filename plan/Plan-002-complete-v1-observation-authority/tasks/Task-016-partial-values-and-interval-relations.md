---
id: Task-016
title: "FR-007 — partial values and interval relations"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/FR-007
    type: references
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-007
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-012
    type: verifies
---
# Task-016: FR-007 — partial values and interval relations

## Scope

Add coherent partial-Boolean facts and exact closed interval relations as bounded,
canonical QObs authority primitives.

## Subtasks

- [x] Write failing TC-007 cases and generated interval/value properties first.
- [x] Define closed value-reason and interval-order types with no Boolean fallback.
- [x] Validate clock, revision, unit, endpoint and effective-limit identity before comparison.
- [x] Canonically encode/decode the fact through a constructor-private validated view.
- [x] Prove ingestion order and midpoint mutations cannot affect the relation.

## Deliverables

- New authority module(s), exports and TC-007 trace-tagged tests.
- Stable typed refusals for incoherent value sets and cross-wired intervals.

## Notes

- Reuse the Plan-001 common owner envelope; version rather than mutate existing schemas.
- Unblocks Task-017 and Task-018.
