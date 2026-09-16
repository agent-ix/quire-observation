---
id: Task-018
title: "FR-009 — revisioned I07 bundles and lineage views"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-016
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-017
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-009
    type: references
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-009
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-012
    type: verifies
---
# Task-018: FR-009 — revisioned I07 bundles and lineage views

## Scope

Create canonical `quire.observation-authority/v1` bundles plus bounded strict-read lineage
views that distinguish initial publication, exact replay, valid succession and conflict.

## Subtasks

- [x] Write TC-009 canonicalization, mutation, replay and lineage failure tests first.
- [x] Define bundle/component/replacement identity domains and canonical ordering.
- [x] Validate initial and later revisions against the supplied current-head/child view.
- [x] Preserve predecessor bytes and produce one canonical successor lineage view.
- [x] Implement a strict reader that validates every expected independent selection.

## Deliverables

- Immutable bundle schema/deriver/reader and lineage-view types.
- Stable stale-head, sibling-branch, replacement and identity-contradiction refusals.

## Notes

- The API validates a supplied view; it must not imply atomic publication or ambient lookup.
- Unblocks Tasks 019 and 021.
