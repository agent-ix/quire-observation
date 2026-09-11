---
id: Task-001
title: "#5 — four independent scope-fact axes"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-007
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/TC-003
    type: verifies
---
# Task-001: #5 — four independent scope-fact axes

## Scope

Make each of FR-003's four scope facts its own result axis with its own capability, so a
consumer that can represent one without the others says so instead of reporting preservation.

## Subtasks

- [ ] **Write the failing test first.** A consumer preserving decision progress, decision
      closure and surrounding progress but not surrounding closure must report
      `lost_axes == [SurroundingClosure]` and a mapping other than `Preserved`. Today it
      reports `Preserved`, which is the defect.
- [ ] **Split the axis.** `ResultAxis::ScopeFacts` becomes `DecisionProgress`,
      `DecisionClosure`, `SurroundingProgress`, `SurroundingClosure`.
- [ ] **Split the capability.** `preserves_scope_facts` becomes four flags; update `all()`
      and the loss table so each flag pairs with exactly one axis.
- [ ] **Update the existing single-axis assertion** that currently expects
      `[ResultAxis::ScopeFacts]`.

## Deliverables

- `src/handoff.rs` with sixteen axes and sixteen capabilities (the seventeenth arrives with
  Task-002).
- `tests/replay.rs` proving the independence of all four scope facts.

## Notes

- `validate()` constrains only `scope_identity` to pair up within each scope; authority and
  boundary identities vary freely, which is why the four are independent facts rather than
  one. FR-003 now states this and defines each of the four.
- Unblocks: the retained-event axis, which extends the same enum and record.
- Gated by Task-007: FR-003's Description and Behavior still enumerate different axis sets,
  and this task must split against a single authoritative set.
