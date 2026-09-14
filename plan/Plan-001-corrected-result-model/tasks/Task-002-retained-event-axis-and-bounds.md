---
id: Task-002
title: "#6 — retained-event axis, honest retention counts, four bound criteria"
type: Task
status: done
resolution: superseded
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-001
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/NFR-001
    type: references
  - target: ix://agent-ix/quire-observation/FR-001
    type: references
  - target: ix://agent-ix/quire-observation/TC-001
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-002
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-003
    type: verifies
---
# Task-002: #6 — retained-event axis, honest retention counts, four bound criteria

> Superseded by the accepted owner-contract architecture in FR-004. Observation
> authority no longer owns or exposes replay retained-event results.

## Scope

Give the retained-event count a capability and a typed loss, stop over-reporting retention on
an unavailable outcome, and bind the four acceptance criteria the amendment left unbacked in
admission and replay.

## Subtasks

- **Write the failing tests first.** A consumer that cannot hold the retained-event count
      reports `lost_axes == []` today. `replay` on a one-over history reports `max_events`
      retained today, where FR-002 now requires zero for an unavailable outcome.
- **Add the axis.** `ResultAxis::RetainedEvents` and `preserves_retained_events`, wired
      into `all()` and the loss table — seventeen axes, matching FR-003.
- **Correct `unavailable()`.** Report `retained_events: 0` and document why: nothing was
      retained because nothing was assessed.
- **Bind `NFR-001-AC-3`.** One event over the declared bound on both paths — `Exhausted`
      from batch replay, `RetentionExhausted` from incremental intake.
- **Bind `NFR-001-AC-4`.** One member over the declared bound is refused. `src/lib.rs`
      enforces this already and nothing tests it.
- **Bind `NFR-001-AC-5`.** `Exhausted` and `Untriggered` both report zero retained.
- **Bind `FR-001-AC-5`.** An unrecognized member object identity and a membership digest
      matching nothing still yield `Available` — the stated boundary, proven.

## Deliverables

- `src/handoff.rs` at seventeen axes; `src/replay.rs` with an honest unavailable count.
- `tests/admission.rs` and `tests/replay.rs` binding `FR-001-AC-5`, `NFR-001-AC-3`,
  `NFR-001-AC-4`, `NFR-001-AC-5`.

## Notes

- Changing `unavailable()`'s reported count is an observable output change. It is required by
  FR-002 and no caller outside this repository consumes it yet.
- Serialized after Task-001 rather than parallel: both write `ResultAxis` and
  `ConsumerCapabilities` in `src/handoff.rs`.
- Unblocks: the exhaustive omission sweep, which closes over the final axis set.
