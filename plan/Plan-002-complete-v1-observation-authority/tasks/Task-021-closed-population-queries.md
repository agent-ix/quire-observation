---
id: Task-021
title: "FR-011 — closed-population filter, count and sum"
type: Task
status: done
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-017
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-018
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-011
    type: references
  - target: ix://agent-ix/quire-observation/FR-009
    type: references
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-011
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-009
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-012
    type: verifies
---
# Task-021: FR-011 — closed-population filter, count and sum

## Scope

Evaluate bounded admitted queries over one exact closed authority-qualified population with
explicit effect-deduplicating or occurrence-preserving semantics.

## Subtasks

- [x] Write TC-011 membership, boundary, duplicate, receipt and arithmetic tests first.
- [x] Add and strict-read the immutable FR-009 v2 seven-role empty-population profile
  without changing v1 schema bytes, digest, APIs, or canonical outputs.
- [x] Select members only through declared authority, relation and snapshot/window rules.
- [x] Implement canonical filter/count and checked ordered sum without operand reordering.
- [x] Exclude transport receipts from effect identity under both duplicate policies.
- [x] Return no definitive aggregate for open/incomplete/foreign/over-bound authority.

## Deliverables

- Query-plan/result types, executor and trace-tagged property tests.
- Canonical participating-fact accounting and stable non-success outcomes.

## Notes

- This track may proceed independently of repair coordination after Task-018.
- Unblocks Task-023.
