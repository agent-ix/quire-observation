---
id: Task-023
title: "IT-002 — pinned consumer repair and aggregate integration"
type: Task
status: not_started
track: C
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-020
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-021
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-022
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-007
    type: references
  - target: ix://agent-ix/quire-observation/FR-008
    type: references
  - target: ix://agent-ix/quire-observation/FR-009
    type: references
  - target: ix://agent-ix/quire-observation/FR-010
    type: references
  - target: ix://agent-ix/quire-observation/FR-011
    type: references
  - target: ix://agent-ix/quire-observation/IT-002
    type: verifies
---
# Task-023: IT-002 — pinned consumer repair and aggregate integration

## Scope

Execute the real local Rust handoff from QObs I07 through compatible pinned temporal,
protocol and composed-runtime consumers for initial, incremental, repair and query paths.

## Subtasks

- [ ] Record exact consumer revisions, contracts, schema digests and fixture rights.
- [ ] Load real timed-refund and related-refund fixtures through strict readers.
- [ ] Exercise IT-002-SC-01 through IT-002-SC-05 without semantic or file-I/O mocks.
- [ ] Compare complete bytes, support, aggregate accounting and lineage across all paths.
- [ ] Retain unsupported consumers as explicit evidence rather than substituting them.

## Deliverables

- Trace-tagged real integration harness and immutable evidence artifacts.
- Reproducible pinned dependency/fixture record.

## Notes

- Requires compatible external consumer revisions; do not copy their semantics into QObs.
- Unblocks Task-024.
