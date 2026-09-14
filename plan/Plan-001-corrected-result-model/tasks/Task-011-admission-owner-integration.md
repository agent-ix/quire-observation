---
id: Task-011
title: "#15 — harden admission and owner identity integration"
type: Task
status: done
track: B
priority: P0
relationships:
  - { target: ix://agent-ix/quire-observation/Task-009, type: depends_on }
  - { target: ix://agent-ix/quire-observation/FR-001, type: references }
  - { target: ix://agent-ix/quire-observation/NFR-001, type: references }
  - { target: ix://agent-ix/quire-observation/TC-001, type: verifies }
---
# Task-011: #15 — harden admission and owner identity integration

## Scope

Make admission prove its observation, explicit-members, population, source, clock, and
relationship facts before producing a qualified capability.

## Subtasks

- [x] Derive and compare record, membership, and population identities.
- [x] Strict-read membership bytes against the independent member population.
- [x] Reject foreign clocks, sources, relationships, and stale owner identities.
- [x] Bound both supplied and required relationship populations before matching.
- [x] Make request identity assignment atomic on refusal.

## Deliverables

- Typed admission refusals and incomplete causes with no stringly public discriminators.
- Traced exact/one-over and cross-wiring tests.

## Notes

- Closure-definition bytes remain outside this crate; their exact identity/digest is retained.
