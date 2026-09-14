---
id: Task-004
title: "#8 — supersede only on a contradicting late record; typed ambiguous order"
type: Task
status: done
resolution: superseded
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-observation/Task-002
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-007
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-002
    type: references
  - target: ix://agent-ix/quire-observation/TC-002
    type: verifies
---
# Task-004: #8 — supersede only on a contradicting late record; typed ambiguous order

> Superseded by the accepted owner-contract architecture in FR-004. Temporal
> contradiction and supersession decisions are downstream; QObs retains only
> immutable direct artifact lineage and typed ambiguous position order.

## Scope

Link a supersession only when a late record actually contradicts the settled result, and
report a duplicated declared order position as its own outcome rather than as an unsupported
profile.

## Subtasks

- **Write the failing tests first.** A settled `Satisfied` result plus a late *agreeing*
      witness, and plus a late record matching neither rule arm, both link
      `supersedes = Some(prior)` today. FR-002 requires neither to.
- **Implement contradiction as a differential.** A late record contradicts when replaying
      with that record in window yields a different `(disposition, basis)`. Do not hand-write
      a basis-to-signal table: the version in the epic plan keyed on the wrong result's basis
      and left a settled `MissedDeadline` unsuperseded by a late counterexample that would
      have produced `Violated`.
- **Use the differential as the test oracle too.** `supersedes.is_some()` must equal
      "recomputing with the late record in window changes the typed outcome", asserted over
      the fixture set rather than case by case.
- **Add `superseding_records`** to `ReplayResult` as the contradicting subset, and derive
      `supersedes` from it, so a consumer can see which record forced the link.
- **Add `Disposition::AmbiguousOrder`** for a duplicated declared sequence, bind
      `FR-002-AC-4`, and **assert its handoff mapping** — it currently reaches the
      classification through a non-exhaustive `matches!` and falls to `Unrepresented` with no
      compile error. Correct by intent, unverified today.
- **Preserve Task-002's invariant.** Both tasks rewrite `unavailable()`; an unavailable
      outcome must still report zero retained events.
- **Keep the existing late-record assertions passing unchanged** — both use a
      contradicting signal and must stay green.

## Deliverables

- `src/replay.rs` with the differential predicate, `superseding_records`, and
  `Disposition::AmbiguousOrder`.
- `tests/replay.rs` binding `FR-002-AC-4`, the two non-contradicting late cases, and the
  differential oracle.

## Notes

- The wrong behavior matters downstream: `quire-protocol#12` would invalidate a still-valid
  immutable result on the strength of a late record that agrees with it.
- `prior_result_identity` is an unvalidated bare identity with nothing tying it to the same
  rule, scope, deadline or cutoff. The differential predicate needs nothing from it, which is
  why it is the right shape.
- `NFR-002`'s reproducibility criterion must keep holding with the new derived field.
