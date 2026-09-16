---
id: SR-053
title: "Gap analysis — Plan-002 after Task-022 bounded-work verification"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-002-complete-v1-observation-authority/, spec/tests.md, src/authority/{common,partial,activation,bundle,repair,coordination,query}.rs, tests/{partial_interval,activation_authority,authority,bounded_work}.rs after Task-022"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/Plan-002", type: reviews }
  - { target: "ix://agent-ix/quire-observation/Task-022", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TM-001", type: references }
---
# SR-053: Gap analysis — Plan-002 after Task-022 bounded-work verification

## Summary

Plan-002, TM-001, and the complete C00 Rust surface were audited after Task-022. TC-012 now
binds equal-input, presentation/arrival permutation, exact-limit, one-over refusal and
transactional retry evidence across FR-007 through FR-011. TC-013 adds a revision-bound static
inventory of 28 scoped expansion paths, including the two shared JSON open-form loops and two
explicitly structural-only paths that do not falsely claim a caller-lowered oracle.

## Verdict

**FAIL (overall plan)** — Plan-002 is 7/9 done. Task-022 itself passes its gap, semantic and Rust
reviews; NFR-003, TC-012 and TC-013 are complete and fallback trace coverage is 89/89 with zero
status lies. Task-023 pinned-consumer IT-002 execution and Task-024 completion/promotion readiness
remain outstanding, so the plan and draft PR must remain open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Pinned-consumer integration remains `not_started` | Task-023, IT-002 |
| FND-002 | high | Completion and promotion readiness remains `not_started` | Task-024, TC-007..TC-013, IT-002 |
| FND-003 | medium | Five pre-existing test symbols carry `IT-001`, but the active declarations do not mint an `IT-001` target | tests/admission.rs, tests/authority.rs |

## Task-022 closure

- Every Task-022 subtask is checked and its plan/task status is `done`; Plan-002 records NFR-003,
  TC-012 and TC-013 complete.
- Equal-input bytes/identities are repeated for partial, activation and query artifacts. Generated
  component, DAG, evaluator-arrival and query-effect presentations preserve semantic outcomes.
- Partial and activation owner paths exercise exact and one-over input, output, depth, string,
  visited-work and owned semantic collection limits. Bundle, repair, coordination and query paths
  exercise their count, work, retained-state, arithmetic and output limits.
- Bundle lineage construction checks aggregate head/child source bytes before hashing, current
  calls reapply supplied-lineage limits, and replacement validation is a bounded linear merge.
- Repair dependency count/bytes are checked and charged before fallible reserve and cloning.
- Coordinator result inputs are charged for both retained copies. Failed incremental push and
  close attempts roll back only unretained state while preserving cumulative work, possible-order
  and at-least-once evaluator-call usage; stateful retry tests cover both transaction windows.
- TC-013 pins seven production modules, admission ownership and three evidence sources by SHA-256.
  Every exact/one-over row requires scoped TC-012/NFR-003-AC-2, named-limit, exact and refusal
  anchors; the two structural-only rows are separately typed without fabricated evidence.
- Independent Rust review and static-inventory review are clean after all high and medium findings
  were fixed. No reviewed code/test stub, tautology, unsafe block, production panic, recursive
  graph walk, factorial materialization, unchecked cast or unbounded retry remains.

## Coverage

- Reconciliation used the explicitly permitted npm Quire 0.32.0 fallback because native Quoin
  does not yet expose `coverage` (agent-ix/quoin#538).
- Tasks done: 7/9, with dependency order, task states and plan checkboxes consistent.
- Rows backed by tagged tests: 89/89. NFR-003 is 2/2, FR-010 is 6/6, and all 13 declared test-case
  rows are backed. There are zero unbacked rows and zero status lies.
- Five pre-existing `IT-001` annotations remain untracked because no active target is minted for
  them. This does not satisfy or replace the still-outstanding IT-002 consumer execution.
- Reverse-gap inspection maps every Task-022 production change to FR-009, FR-010 or NFR-003 and
  every new evidence symbol to TC-012 or TC-013; no underspecified implementation was found.

## Validation and execution evidence

- Native `quoin` 0.23.1-87-gb3d02d4, `quoin validate --repo . --strict --json`: PASS, no findings.
- Native scoped-validation regression remains recorded in agent-ix/quoin#541; coverage absence is
  tracked in agent-ix/quoin#538.
- Fallback `quire coverage --scope . --json`: PASS, 89/89 backed and zero status lies.
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS.
- `cargo test --locked --all-targets --all-features`: PASS, 111 tests, zero failed or ignored.
- `RUSTDOCFLAGS=-Dwarnings cargo doc --locked --all-features --no-deps`: PASS.
- `cargo deny check`: PASS (`advisories`, `bans`, `licenses`, `sources`).
- `cargo test --locked --test bounded_work`: PASS, 2/2 revision-bound static checks.
