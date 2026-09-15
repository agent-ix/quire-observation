---
id: Task-020
title: "FR-010 — evaluator coordination and replay parity"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-019
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-010
    type: references
  - target: ix://agent-ix/quire-observation/TC-010
    type: verifies
---
# Task-020: FR-010 — evaluator coordination and replay parity

## Scope

Coordinate opaque selected evaluator contributions over a repair plan so decisive prefixes,
failure states and batch/incremental outputs preserve exact external semantics.

## Subtasks

- [ ] Add failing decisive-prefix, unresolved-prefix and path-parity TC-010 tests.
- [ ] Define a typed evaluator trait carrying exact intervals and admissible orders.
- [ ] Emit genuinely settled prefixes before end-of-input without quiet-time inference.
- [ ] Retain item-local unsupported/refused/failed/exhausted outcomes without replacement.
- [ ] Compare complete disposition, support, identity and supersession bytes across paths.

## Deliverables

- Evaluator coordination seam and batch/incremental repair runners.
- Trace-tagged parity and failure-injection tests.

## Notes

- The trait must be mockable for unit tests but integration evidence uses pinned real consumers.
- Unblocks Task-023.
