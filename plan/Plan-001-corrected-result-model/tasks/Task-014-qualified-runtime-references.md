---
id: Task-014
title: "Adopt FCD-qualified runtime references"
type: Task
status: done
relationships:
  - target: ix://agent-ix/quire-observation/FR-005
    type: references
  - target: ix://agent-ix/quire-observation/TC-005
    type: verifies
---
# Task-014: Adopt FCD-qualified runtime references

## Objective

Replace the partial producer, subject and relationship shadows with the exact
FCD Producer interface 1.2 authority and the FR-287/FR-262 runtime values
without adding transport, protocol-evaluation or temporal responsibilities.

## Subsystems

1. Pin and consume the FCD `agent-ix-baseline-producer` Rust crate at `4042882`.
2. Implement constructor-private authority-qualified subject values and typed
   semantic comparison.
3. Implement exact producer-declaration-qualified runtime relationship values.
4. Implement bounded replay-aware four-state correlation in observation
   admission and update owner-artifact projections that retain these facts.
   Preserve the immutable record/population v1 schemas and publish their
   authority-qualified replacements as v2.
5. Add TC-005 mutation, replay, ordering, distinction and boundary tests.
6. Run `/rust-review` and `/gap-analysis`, remediate every finding, then merge
   the one PR closing `quire-observation#18` and update all live trackers.

## Boundaries

- No FCD decoder or telemetry adapter is introduced; the input is an already
  admitted static-bundle capability.
- No protocol verdict, TL implementation, temporal evaluator, heuristic
  correlation or ambient lookup is introduced.
- `quire-observation#13` retains finite topology/window follow-on scope.

## Evidence

- Specification: FR-005, TC-005, the #18 self-review packet.
- Implementation: Rust unit/integration tests carrying TC-005 and FR-005 tags.
- Gates: targeted tests during development; one full fmt/Clippy/test and both
  required self-reviews at PR readiness.

## Completion

Completed on 2026-09-14 against observation baseline `9ac80e9` and FCD
revision `404288282402d60de007295ccbafa960532b955e`.

- Replaced the local producer/subject/relationship shadows with the exact
  admitted FCD capability and constructor-private qualified runtime values.
- Implemented exact declaration and endpoint validation, replay
  idempotence, contradiction refusal, and the four correlation outcomes.
- Preserved both v1 owner schemas byte-for-byte and published the qualified
  record/population contracts and identity preimages as v2.
- Passed 42 Rust tests, warning-denied check/Clippy/docs, both v2 schema
  meta-validations, dependency policy, 42/42 Quire matrix coverage, SR-032
  Rust review, and SR-033 gap analysis.
