---
id: Task-003
title: "#7 — exhaustive, self-enforcing per-axis omission sweep"
type: Task
status: done
resolution: superseded
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-004
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/NFR-002
    type: references
  - target: ix://agent-ix/quire-observation/TC-003
    type: verifies
---
# Task-003: #7 — exhaustive, self-enforcing per-axis omission sweep

> Superseded by the accepted owner-contract architecture in FR-004. The current
> strict-reader field mutation sweep replaces the removed capability-axis sweep.

## Scope

Replace the single-axis test that claims to cover every axis with one that does, and make it
impossible to add a result fact without an omission case.

## Subtasks

- **Add `ResultAxis::ALL`.** The authoritative axis list in code, matching FR-003.
- **Rewrite the sweep as a table.** One case per capability flag: flip exactly that flag,
      assert the lost-axis set is exactly that axis and the mapping is not `Preserved`.
- **Assert set equality, not length.** The set of axes exercised must equal
      `ResultAxis::ALL` as a set. A length assertion passes a table of seventeen rows that
      duplicates one axis and omits another.
- **Back it with an exhaustive `match`.** Write the capability lookup as a `match` over
      `ResultAxis` so a new variant fails to compile rather than failing at runtime.
- **Compare loss as a set, not an ordered `Vec`.** Ordered equality couples every case to
      the enum's declaration order, so a cosmetic reordering breaks unrelated cases.
- **Cover the supersession fact.** `superseding_records` arrives with Task-004 as a new
      independently readable result fact; the sweep must account for it rather than leave the
      D2 class open one field to the left.
- **Rename the test** to say what it does, and bind the stray `NFR-002-AC-2` tag by
      asserting the dependencies it claims to cover.

## Deliverables

- `src/handoff.rs` exposing `ResultAxis::ALL` and an exhaustive capability lookup.
- `tests/replay.rs` with one omission case per axis and the set-equality assertion.

## Notes

- Lands after Task-004 deliberately: a sweep over `ResultAxis` cannot see a new
  `ReplayResult` field, so closing over the axis set before the supersession field exists
  would harden the wrong set — the same mistake one field over.
- This is the durable fix for the class: both high defects in this epic were a result fact
  with no omission test.
