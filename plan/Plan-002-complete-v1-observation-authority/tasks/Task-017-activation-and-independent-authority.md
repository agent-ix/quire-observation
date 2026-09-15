---
id: Task-017
title: "FR-008 — activation and independent scope authority"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-016
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-008
    type: references
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-008
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-012
    type: verifies
---
# Task-017: FR-008 — activation and independent scope authority

## Scope

Publish immutable activation captures and a record whose activation, progress, closure,
completeness, lateness, verdict contribution and settlement contribution remain independent.

## Subtasks

- [x] Write the TC-008 state-product, silence-boundary and cross-wire failures first.
- [x] Derive activation identity from trigger, interval and canonical complete capture set.
- [x] Preserve original captured values/provenance across later observations.
- [x] Implement exact progress coverage and three-way lateness from interval relations.
- [x] Refuse foreign authority without changing any unaffected axis or emitting fallback.

## Deliverables

- Versioned activation/scope-authority records, readers and trace-tagged tests.
- Explicit incomplete/refused outcomes naming every failed premise.

## Notes

- No state axis may infer another, even when common cases correlate them.
- Unblocks Task-018 and Task-021.
