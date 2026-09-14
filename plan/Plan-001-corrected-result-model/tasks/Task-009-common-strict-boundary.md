---
id: Task-009
title: "#15 — implement the common strict owner boundary"
type: Task
status: done
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-observation/Task-008, type: depends_on }
  - { target: ix://agent-ix/quire-observation/FR-004, type: references }
  - { target: ix://agent-ix/quire-observation/NFR-001, type: references }
  - { target: ix://agent-ix/quire-observation/NFR-002, type: references }
  - { target: ix://agent-ix/quire-observation/TC-004, type: verifies }
---
# Task-009: #15 — implement the common strict owner boundary

## Scope

Implement one bounded canonical envelope, identity, correction-lineage, error, history, and
validated-view layer shared by all owner modules.

## Subtasks

- [x] Implement owner-clamped limits and deterministic usage accounting.
- [x] Preflight UTF-8, JSON structure, depth, strings, populations, and work before reading.
- [x] Implement canonical encoding, envelope identity, direct predecessor validation, and
  constructor-private validated views.
- [x] Publish and test the closed stable error-code catalog.

## Deliverables

- `src/authority/common.rs` and public common types re-exported by `authority`.
- Exact/one-over and malformed-input tests without partial documents or views.

## Notes

- Unsafe code is forbidden and every public item is documented.
