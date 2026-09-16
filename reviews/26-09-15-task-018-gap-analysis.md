---
id: SR-048
title: "Gap analysis — Plan-002 after Task-018 revisioned bundles"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-002-complete-v1-observation-authority/, spec/tests.md, src/, schemas/, tests/ after Task-018"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/Plan-002", type: reviews }
  - { target: "ix://agent-ix/quire-observation/Task-018", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TM-001", type: references }
---
# SR-048: Gap analysis — Plan-002 after Task-018 revisioned bundles

## Summary

Plan-002, TM-001, and the Rust source/test surface were audited after Task-018. The
revisioned authority owner now publishes canonical immutable bundles and bounded lineage views
from exact typed owner facts, explicit replacements and conflicts, authority-qualified fact
subjects, and caller-lowered resource limits. TC-009 exercises deterministic derivation,
strict reading, every component role, lineage/replay/contradiction behavior, schema conformance,
wire mutations, cross-wiring, and exact/one-over bounds.

## Verdict

**FAIL (overall plan)** — Plan-002 is 3/9 done. Task-018 itself passes its targeted gap,
semantic, and Rust reviews with FR-009 at 5/5 and TC-009 backed, but Tasks 019 through 024 and
their declared FR-010, FR-011, NFR-003, TC-010 through TC-013 and IT-002 evidence remain
outstanding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Bounded affected-region planning remains `not_started` | Task-019, FR-010, TC-010 |
| FND-002 | high | Evaluator coordination and parity remain `not_started` | Task-020, FR-010, TC-010 |
| FND-003 | high | Closed-population query work remains `not_started` | Task-021, FR-011, TC-011 |
| FND-004 | high | Bounded-work verification remains `not_started` | Task-022, NFR-003, TC-012, TC-013 |
| FND-005 | high | Pinned-consumer integration remains `not_started` | Task-023, IT-002 |
| FND-006 | high | Completion and promotion gate remains `not_started` | Task-024, TC-007..TC-013, IT-002 |
| FND-007 | medium | Five pre-existing test symbols carry `IT-001`, but the active declarations do not mint an `IT-001` target | tests/admission.rs, tests/authority.rs |

## Task-018 closure

- Every `Task-018` subtask is checked and its status is `done`; the Plan-002 FR-009, TC-009,
  task-table and TM-001 statuses agree.
- FR-009 acceptance coverage is 5/5. Seven TC-009 tests carry the exact acceptance-criterion
  tags they exercise; the schema-conformance test adds an eighth TC-009 evidence symbol without
  claiming unfinished NFR-003 evidence.
- The behavior inventory covers all eleven owner roles, exact canonical owner bytes and typed
  payloads, distinct authority-qualified fact subjects, explicit three-kind conflicts, exact
  replacement deltas, initial/successor/replay/contradiction/branch lineage, strict bundle and
  lineage reading, and bounded preflight. Every group is owned by FR-009/Task-018.
- Independent Rust-review findings were fixed before closure: same-subject duplicates, missing
  collection-specific preflight bounds, large enum storage, a weak typed-wire schema, and
  name-only position-array classification.
- Independent semantic review findings were fixed before closure: untyped embedded facts,
  incomplete conflict vocabulary, and missing predecessor/cross-authority/cross-scope cases.

## Coverage

- Reconciliation: `quire coverage` 0.32.0 / engine `a874fb641cb70da83c8c8b23f9fea0a44255b88a` at the Task-018 working tree.
- Tasks done: 3/9, with dependency order and plan/task checkboxes consistent.
- Rows backed by tagged tests: 70/88. The report contains 20 unbacked-row records, zero status
  lies, five pre-existing untracked `IT-001` symbols, and the expected future-plan gaps.
- Target breakdown: FR-007 5/5, FR-008 6/6, FR-009 5/5, and TC-007 through TC-009 are backed.
  FR-010 0/6, FR-011 0/6, and NFR-003 0/2 remain intentionally outstanding.

## Validation and execution evidence

- Native `quoin` 0.23.1-86-gc721d71, `quoin validate --strict --repo .`: PASS, no findings.
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- `cargo check --all-targets --all-features --locked`: PASS.
- `cargo clippy --all-targets --all-features --locked -- -D warnings`: PASS.
- `cargo test --all-targets --all-features --locked`: PASS, 73 tests, zero failed or ignored.
- Warning-denied `cargo doc --all-features --no-deps --locked`: PASS.
- `cargo deny check`: PASS (`advisories`, `bans`, `licenses`, `sources`).
- Standalone bundle JSON Schema compilation plus emitted/mutated instance validation: PASS.
- No production panic, unsafe block, unchecked integer cast, recursion, async/blocking or lock
  discipline issue remains in the Task-018 delta.
