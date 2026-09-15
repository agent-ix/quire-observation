---
id: Task-022
title: "NFR-003 — bounded-work property and static verification"
type: Task
status: not_started
track: C
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-020
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-021
    type: depends_on
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-012
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-013
    type: verifies
---
# Task-022: NFR-003 — bounded-work property and static verification

## Scope

Close the cross-cutting reproducibility and resource evidence for every C00 collection,
traversal, order representation, arithmetic loop, retained object and output byte.

## Subtasks

- [ ] Generate equal-input and presentation/arrival permutation fixtures across all modules.
- [ ] Exercise every exact and one-over caller-lowered limit with no partial retained output.
- [ ] Inventory each loop/allocation and map it to a dominating checked bound.
- [ ] Reject recursion, factorial order materialization, unbounded retry and unchecked casts.
- [ ] Bind TC-012 and TC-013 tags to the same reviewed source revision.

## Deliverables

- Cross-module property suite and checked-limit static inventory.
- Zero-excess, zero-partial-output evidence for both NFR-003 criteria.

## Notes

- This is a gate, not an optimization task; failures return to the owning feature task.
- Unblocks Task-023.
