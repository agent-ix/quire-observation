---
id: SR-049
title: "Gap analysis — Plan-002 after Task-019 affected-region planning"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-002-complete-v1-observation-authority/, spec/tests.md, src/, tests/ after Task-019"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/Plan-002", type: reviews }
  - { target: "ix://agent-ix/quire-observation/Task-019", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TM-001", type: references }
---
# SR-049: Gap analysis — Plan-002 after Task-019 affected-region planning

## Summary

Plan-002, TM-001, and the Rust source/test surface were audited after Task-019. The repair
planner now derives one canonical finite plan from a validated direct FR-009 revision pair,
exact prior/successor fact regions, explicit dependency edges, immutable prior result bytes,
and caller-lowered resource limits. TC-010 planner evidence covers generated closure oracles,
old-only/new-only/both/neither half-open observability, presentation-order invariance,
dependency-safe scheduling, byte retention, cross-wiring, typed adverse cases, and exact versus
one-over limits.

## Verdict

**FAIL (overall plan)** — Plan-002 is 4/9 done. Task-019 itself passes its targeted gap,
semantic, and Rust reviews with FR-010-AC-1, FR-010-AC-2, and the planner portion of
FR-010-AC-6 backed. Tasks 020 through 024 and their evaluator, FR-011, NFR-003, TC-011 through
TC-013, IT-002, and promotion evidence remain outstanding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Evaluator coordination, decisive-prefix behavior and batch/incremental parity remain `not_started` | Task-020, FR-010-AC-3..5, TC-010 |
| FND-002 | high | Closed-population query work remains `not_started` | Task-021, FR-011, TC-011 |
| FND-003 | high | Bounded-work verification remains `not_started` | Task-022, NFR-003, TC-012, TC-013 |
| FND-004 | high | Pinned-consumer integration remains `not_started` | Task-023, IT-002 |
| FND-005 | high | Completion and promotion gate remains `not_started` | Task-024, TC-007..TC-013, IT-002 |
| FND-006 | medium | Five pre-existing test symbols carry `IT-001`, but the active declarations do not mint an `IT-001` target | tests/admission.rs, tests/authority.rs |

## Task-019 closure

- Every Task-019 subtask is checked and its plan/task status is `done`.
- Replacement relations are the only invalidation seeds. Each retains exact prior and successor
  regions, so moved facts invalidate old-only and new-only observers without filling the gap
  between disjoint windows.
- Fact and result regions are bound to the validated bundle subject and typed clock authority.
  Half-open overlap is exact across timestamp, event-position and fixed-sample families.
- Traversal follows only explicit edges, rejects all graph cycles, and accounts count, work,
  retained-state and output bounds before the corresponding retention or operation.
- A successful plan retains exact affected prior records, byte-identical unaffected records,
  changed-fact inventory, and dependency-safe scope/window/identity scheduling. Refusal returns
  no partial plan.
- Independent reviews found and drove fixes for old/new region modeling, authority cross-wiring,
  early preflight, charged seed lookup, clock-family boundaries, resource-oracle independence,
  changed-fact evidence and full input permutation. Both final reviews are clean.

## Coverage

- Reconciliation: Quire 0.32.0 at the Task-019 working tree.
- Tasks done: 4/9, with dependency order and plan/task checkboxes consistent.
- Rows backed by tagged tests: 74/88. Rust evidence is 68/68/80 bound/tagged/candidates.
- FR-010 is honestly partial at 3/6: AC1, AC2 and planner AC6 are backed. AC3 through AC5 and
  evaluator-failure AC6 remain assigned to Task-020; FR-010 and TC-010 therefore remain planned.
- FR-011 and NFR-003 remain intentionally outstanding. No new untracked target was introduced.

## Validation and execution evidence

- Native `quoin` 0.23.1-86-gc721d71, `quoin validate --strict --repo .`: PASS, no findings.
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- `cargo check --all-targets --all-features --locked`: PASS.
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: PASS.
- `cargo test --all-targets --all-features --locked`: PASS, 80 tests, zero failed or ignored.
- Warning-denied `cargo doc --all-features --no-deps --locked`: PASS.
- `cargo deny check`: PASS.
- No production panic, unsafe block, unchecked integer cast, recursion, async/blocking or lock
  discipline issue remains in the Task-019 delta.
