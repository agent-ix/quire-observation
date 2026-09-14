---
id: Task-010
title: "#15 — implement all observation-owner contracts"
type: Task
status: done
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-observation/Task-009, type: depends_on }
  - { target: ix://agent-ix/quire-observation/FR-004, type: references }
  - { target: ix://agent-ix/quire-observation/TC-004, type: verifies }
---
# Task-010: #15 — implement all observation-owner contracts

## Scope

Implement the record, population, position, clock, capture, progress, closure,
completeness, and result-availability owner modules as one coherent ecosystem.

## Subtasks

- [x] Publish immutable schema bytes and pinned schema digests for every contract.
- [x] Implement deterministic derivation and strict reading against independent selections.
- [x] Bind exact scope, population, clock identity/revision, boundary, and lineage facts.
- [x] Expose complete read-only payload accessors without public validated constructors.

## Deliverables

- Nine `src/authority/` modules plus the explicit-members input contract.
- Ten pinned JSON schemas and README handoff digests.

## Notes

- Each module owns only its payload and delegates shared wire behavior to Task-009.
