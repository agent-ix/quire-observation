---
id: Task-008
title: "#15 — establish the observation-owner contract architecture"
type: Task
status: done
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-observation/FR-001, type: references }
  - { target: ix://agent-ix/quire-observation/FR-002, type: references }
  - { target: ix://agent-ix/quire-observation/FR-003, type: references }
  - { target: ix://agent-ix/quire-observation/FR-004, type: references }
  - { target: ix://agent-ix/quire-observation/NFR-001, type: references }
  - { target: ix://agent-ix/quire-observation/NFR-002, type: references }
  - { target: ix://agent-ix/quire-observation/TC-004, type: verifies }
---
# Task-008: #15 — establish the observation-owner contract architecture

## Scope

Replace the obsolete replay/result-handoff model with the accepted QObs owner boundary and
allocate all nine contracts to cohesive authority subsystems.

## Subtasks

- [x] Remove temporal/protocol result ownership from the crate.
- [x] Define the shared envelope and nine closed subsystem contracts in FR-004.
- [x] Make FR-001 -> FR-004 -> FR-002 -> FR-003 an acyclic dependency chain.
- [x] Preserve old plan work as explicitly superseded history.

## Deliverables

- Unified reviewed requirements and module boundary.
- No parser, evaluator, transport adapter, result vocabulary, or Boolean coercion.

## Notes

- This task is the accepted replacement for superseded Tasks 001–007.
