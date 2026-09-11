---
id: Task-005
title: "#9 — make IT-001 and the batch/incremental agreement real"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/FR-002
    type: references
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/IT-001
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-002
    type: verifies
---
# Task-005: #9 — make IT-001 and the batch/incremental agreement real

## Scope

Two tags that currently bind to nothing: an integration test that never leaves admission, and
an agreement criterion whose missed-deadline clause no test exercises. Lands **first**: it is
test-only, depends on nothing, and its missed-deadline case is the only regression guard over
the two-path edits that follow.

## Subtasks

- [ ] **Extend the integration test.** From the incomplete admission, drive a replay with no
      available history, assert `IncompleteHistory`, then hand off with
      `Completeness::Incomplete` to a consumer that cannot preserve completeness and assert
      the `Completeness` axis is lost with a mapping of `Unrepresented` — never `Preserved`.
      Today the test calls only `admit`, so its `FR-002-AC-3`, `FR-003-AC-1` and
      `FR-003-AC-3` tags cannot fail for the reasons those criteria state.
- [ ] **Add the missed-deadline agreement case.** A progress assertion covering past the
      deadline must return `MissedDeadline` from both paths, equal.
- [ ] **Confirm the failing-first direction.** Dropping `request.progress` in
      `IncrementalAssessment::new` must turn the suite red; today it stays green.

## Deliverables

- `tests/pipeline.rs` driving all three stages from one incomplete admission.
- `tests/replay.rs` with the third agreement case.

## Notes

- Deliberately *not* adding a compile-time coupling from an incomplete admission to the
  handoff type: FR-002 takes qualified records as caller-supplied inputs, so that coupling
  would be an unrequested feature. IT-001's own procedure already requires the adverse
  variants to be carried through to the sink.
- It flips one flag on `ConsumerCapabilities::all()`, which later tasks update themselves, so
  it holds in any order — which is why it goes first rather than last.
