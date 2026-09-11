---
id: Plan-001
title: "quire-observation — corrected result model and its evidence"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-observation/FR-001
    type: references
  - target: ix://agent-ix/quire-observation/FR-002
    type: references
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/NFR-001
    type: references
  - target: ix://agent-ix/quire-observation/NFR-002
    type: references
---
# Implementation Plan: corrected result model and its evidence

PR #11 amended FR-001, FR-002, FR-003 and NFR-001 after a five-lens review. The code
predates that amendment: `src/handoff.rs` exposes one `preserves_scope_facts` capability for
four independently readable facts and no retained-event axis, and `src/replay.rs` links a
supersession for any late record and reports a duplicated order position as an unsupported
profile. This plan closes the gap between the reviewed spec and the code, and binds the five
acceptance criteria the amendment deliberately left unbacked.

Each task mirrors one GitHub issue under EPIC #1 and cites it. The issues are the tracker;
this bundle is the governed target the audit runs against.

## Requirements Summary

### Functional Requirements

- [ ] **FR-001**: qualified observation admission. Amended with the admission boundary —
      a member's object identity is an opaque equality key, and the selected membership and
      closure digests are retained without recomputation. `FR-001-AC-5` is unbacked.
- [ ] **FR-002**: replay and incremental assessment. Amended with three obligations the code
      violates: link a supersession *only* for a contradicting late record, report a
      duplicated declared order position as an ambiguous order distinct from an unsupported
      profile, and report no retained event for an unavailable outcome. `FR-002-AC-4` is
      unbacked.
- [ ] **FR-003**: result handoff. Amended to seventeen independently readable axes with one
      independently settable capability each, and no capability standing for two axes. The
      code implements thirteen, four of them collapsed into one and one axis absent.

### Non-Functional Requirements

- [ ] **NFR-001**: bounded retained state. Five criteria; `NFR-001-AC-3` (event bound on
      both paths), `NFR-001-AC-4` (member bound) and `NFR-001-AC-5` (unavailable assessment
      reports no retained event) are unbacked.
- [x] **NFR-002**: reproducible outcomes. Both criteria backed; this plan must not regress
      them — `superseding_records` is a derived field and must stay deterministic.

## Dependency Graph

### Core dependency edges

- `FR-003 (axis set) -> NFR-002 (exhaustive omission evidence)`
  Reason: an omission test that closes over the axis set can only be written once the set
  is final. Splitting the scope axes and adding the retained-event axis both change it, so
  the sweep comes after both.
- `FR-002 (ReplayResult shape) -> FR-003 (handoff inputs)`
  Reason: FR-003's Inputs now name the retained-event count as coming from FR-002's result.
  Adding `superseding_records` to that result changes the type the handoff reads, so the two
  changes touch one struct and must not land concurrently.
- `FR-001 + FR-002 + FR-003 -> IT-001`
  Reason: IT-001 asserts an incomplete admission cannot reach a passed handoff, which needs
  the admission boundary, the replay outcome and the capability loss model all in place.
- `everything -> the audit`
  Reason: the gap-analysis verdict is only meaningful against the finished tree.

### Shared dependencies

- `src/replay.rs` is written by two tasks — the retained-event count in `unavailable()` and
  the supersession predicate. Single-writer discipline: they land in sequence, not in
  parallel.
- `ResultAxis` / `ConsumerCapabilities` in `src/handoff.rs` are written by three tasks. The
  axis set must be final before the exhaustive sweep closes over it.

### Cross-cutting constraints

- `NFR-001` applies to `admit` (record and member bounds), `IncrementalAssessment::push`
  (event and active-key bounds), `replay` (event and active-key bounds) and `unavailable`
  (the reported count).
- `NFR-002` applies to every change to `ReplayResult` and `ConsumerHandoff`: equal ordered
  input must keep returning an equal typed result.

### The seams

`src/handoff.rs:45-59` is the axis enum, `:113-128` the capability record, `:175-213` the
loss loop that pairs them — all three change together or the pairing silently drifts, which
is the defect this plan exists to fix. `src/replay.rs:209-247` is the single scan that
classifies late records and settles a decision; the supersession predicate belongs there,
keyed on the basis the scan produces, not on a separate pass. `src/lib.rs:402-436` is the
member-to-record binding, already correct — the admission work only adds the tests that
prove its stated boundary.

## Test Plan

### Unit Tests

- [ ] **`tc003_surrounding_closure_loss_is_independent`** (FR-003-AC-3, TC-003): a consumer
      preserving the three other scope facts but not surrounding closure reports
      `lost_axes == [SurroundingClosure]` and a mapping that is not `Preserved`.
- [ ] **`tc003_retained_events_loss_is_explicit`** (FR-003-AC-2, TC-003): a consumer that
      cannot hold the retained-event count reports that axis lost.
- [ ] **`tc003_every_axis_has_an_omission_case`** (FR-003-AC-2, FR-003-AC-3, TC-003): one
      case per capability flag, each asserting exactly one lost axis, plus an assertion that
      the case count equals `ResultAxis::ALL.len()`.
- [ ] **`tc001_opaque_member_identity_and_unverified_digests_admit`** (FR-001-AC-5, TC-001):
      an unrecognized member object identity and a membership digest that matches nothing
      still yield `Available`.
- [ ] **`tc001_member_bound_refuses_one_over`** (NFR-001-AC-4, TC-001): one member over the
      declared bound is refused with `ResourceLimit { limit: "members" }`.
- [ ] **`tc002_event_bound_holds_on_both_paths`** (NFR-001-AC-3, TC-002): one event over the
      declared bound is `Exhausted` in batch and `RetentionExhausted` incrementally.
- [ ] **`tc002_unavailable_reports_no_retained_event`** (NFR-001-AC-5, TC-002): `Exhausted`
      and `Untriggered` outcomes both report `retained_events == 0`.
- [ ] **`tc002_agreeing_late_record_does_not_supersede`** (FR-002-AC-3, TC-002): a settled
      result plus a late record carrying the same decisive signal retains the record and
      links no supersession.
- [ ] **`tc002_irrelevant_late_record_does_not_supersede`** (FR-002-AC-3, TC-002): the same
      with a signal matching neither rule arm.
- [ ] **`tc002_duplicate_order_position_is_ambiguous_order`** (FR-002-AC-4, TC-002): two
      records sharing one declared sequence report `AmbiguousOrder`, not `Unsupported`.
- [ ] **`tc002_missed_deadline_agrees_across_paths`** (FR-002-AC-1, TC-002): a progress
      assertion covering past the deadline returns `MissedDeadline` from both paths, equal.

### Integration Tests

- [ ] **`it001_incomplete_admission_cannot_reach_passed_handoff`** (IT-001): the existing
      test is extended to actually drive replay and handoff from the incomplete admission,
      asserting `IncompleteHistory` and then a `Completeness` axis loss with a mapping of
      `Unrepresented`.

### Verification (NFRs)

- [ ] **`verify_retained_state_bounds`** (NFR-001): exactly-at-limit and one-over inputs on
      both paths, reading the retained-state counters, plus the unavailable-outcome count.
- [ ] **`verify_reproducible_outcomes`** (NFR-002): repeated replay of identical ordered
      input returns an equal typed result, including the new `superseding_records` field.

## Remaining Work

Mostly one serial track. This is a three-file crate and the semantic tasks all write
`src/handoff.rs`, `src/replay.rs` or `tests/replay.rs`, so optimistic parallelism would buy
merge conflicts rather than time. Two tasks are genuinely independent: the test-only
evidence work, which goes first because it guards everything after it, and the second spec
amendment, which gates every semantic change.

### Track S: Specification gate (first, blocks Track A)

- **S1 = Task-007** define contradiction, lateness and outcome precedence — Medium; exit: no
  code task is left implementing a decision no requirement states.

### Track B: Evidence first (independent, start now)

- **B1 = Task-005** make IT-001 and the batch/incremental agreement real — Medium; exit:
  three acceptance-criterion tags that cannot currently fail begin to fail when the behavior
  regresses, and the missed-deadline agreement case guards every later two-path edit.

### Track A: Critical Path (serial, after S1)

- **A1 = Task-001** four independent scope-fact axes — Easy; exit: a consumer that cannot
  hold surrounding closure stops reporting preservation.
- **A2 = Task-002** retained-event axis, honest retention counts, four bound criteria —
  Medium; exit: every result fact a consumer can drop is reportable as lost, and an
  unavailable assessment stops claiming retention it does not have.
- **Gate = axis set final** — measures whether the capability record matches FR-003's axis
  list and whether the four previously unbacked bound criteria are bound; pass:
  `quire coverage` shows `FR-001-AC-5`, `NFR-001-AC-3`, `NFR-001-AC-4` and `NFR-001-AC-5`
  backed, with one independently settable flag per axis. If it fails, do not start A3.
- **A3 = Task-004** supersede only on a contradicting late record, typed ambiguous order —
  Hard; exit: an agreeing or irrelevant late record is retained without inviting a downstream
  consumer to invalidate a still-valid result.
- **A4 = Task-003** exhaustive, self-enforcing omission sweep — Easy; exit: adding a result
  fact without an omission case fails the build, `superseding_records` included.

### Track D: Documentation and audit (last)

- **D1 = Task-006** correct the coverage record and run the authoritative audit — Easy; exit:
  the repository's own report of its coverage agrees with the tool's, and the verdict comes
  from the tool rather than from self-assessment.

## Parallel Execution Summary

```text
S1 ─────> A1 ──> A2 ──> [Gate: axis set final] ──> A3 ──> A4 ──┐
                                                               ├──> D1
B1 ────────────────────────────────────────────────────────────┘
   (test-only; guards every two-path edit after it)
```

B1 runs alongside S1 and Track A. Nothing else is concurrent.

## Task File Mapping

| Task     | Track | Owns (references)                        | Verified by (verifies)         | Status      |
| -------- | ----- | ---------------------------------------- | ------------------------------ | ----------- |
| Task-007 | S     | FR-001, FR-002, FR-003, NFR-001, NFR-002 | TC-001, TC-002, TC-003         | not_started |
| Task-005 | B     | FR-002, FR-003                           | IT-001, TC-002                 | not_started |
| Task-001 | A     | FR-003                                   | TC-003                         | not_started |
| Task-002 | A     | FR-003, NFR-001, FR-001                  | TC-001, TC-002, TC-003         | not_started |
| Task-004 | A     | FR-002                                   | TC-002                         | not_started |
| Task-003 | A     | FR-003, NFR-002                          | TC-003                         | not_started |
| Task-006 | D     | FR-001, FR-002, FR-003, NFR-001, NFR-002 | TC-001, TC-002, TC-003, IT-001 | not_started |

## Coordination Rules

- **Specification before semantics.** Task-007 lands before Task-001, Task-004 and Task-003.
  Three things those tasks must decide — what contradicts what, what counts as late, and which
  non-success outcome wins — are stated by no requirement today.
- **Single writer per file.** `src/handoff.rs` is written by A1, A2 and A4; `src/replay.rs` by
  A2 and A3; `tests/replay.rs` by B1 and all of Track A. One task lands before the next
  starts, and B1 lands before any of them touches `tests/replay.rs`.
- **Freeze the axis set at the gate.** After A2, `ResultAxis` does not change except for the
  supersession fact A3 introduces. A later axis is a new plan, and A4's set assertion will
  force it to arrive with its omission test.
- **A4 comes after A3, not before.** A sweep over `ResultAxis` cannot see a new `ReplayResult`
  field, so closing over the axis set before `superseding_records` exists would harden a set
  that is one field short — the same defect one field over.
- **Each task lands as its own PR** against `main`, green on `cargo fmt --check`,
  `cargo clippy -- -D warnings` and `cargo test --locked`, with `quire coverage` showing no
  new unbacked criterion and `unbacked_rows` empty.
- **No hosted CI dispatch.** `AGENTS.md` makes hosted workflows `workflow_dispatch` only;
  verification is local.
- **The unbacked criteria are the plan's ledger.** `FR-001-AC-5`, `NFR-001-AC-3`,
  `NFR-001-AC-4` and `NFR-001-AC-5` clear at A2; `FR-002-AC-4` clears at A3; anything Task-007
  mints clears with the task it is assigned to. Nothing may be left unbacked when D1 runs.
