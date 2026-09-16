---
id: SR-051
title: "Gap analysis — Plan-002 after Task-021 closed-population queries"
type: SpecReview
analysis: gap-analysis
scope: "plan/Plan-002-complete-v1-observation-authority/, spec/tests.md, schemas/observation-authority-v2.schema.json, src/authority/{bundle,query}.rs, tests/authority.rs after Task-021"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/Plan-002", type: reviews }
  - { target: "ix://agent-ix/quire-observation/Task-021", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TM-001", type: references }
---
# SR-051: Gap analysis — Plan-002 after Task-021 closed-population queries

## Summary

Plan-002, TM-001, and the Task-021 Rust/schema surface were audited after the
closed-population query implementation. The query evaluator consumes one exact validated bundle
revision; binds population, membership, component, relationship, subject, effect and signal axes;
evaluates canonical filter/count/exact-sum plans in position-ledger order; applies the selected
duplicate policy; and returns typed incomplete or refused outcomes without a partial aggregate.
The accompanying immutable authority v2 contract admits exactly the seven structurally empty
owner roles and preserves the existing authority v1 contract unchanged.

## Verdict

**FAIL (overall plan)** — Plan-002 is 6/9 done. Task-021 itself passes its targeted gap,
semantic, and Rust reviews; FR-009 and FR-011 are complete at 6/6 acceptance criteria each, and
TC-009 and TC-011 are backed. Tasks 022 through 024 and their remaining NFR-003 static audit,
pinned-consumer IT-002 evidence, and promotion gate remain outstanding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Bounded-work static verification remains `not_started`; NFR-003-AC-1 and TC-013 are the only unbacked matrix rows | Task-022, NFR-003, TC-013 |
| FND-002 | high | Pinned-consumer integration remains `not_started` | Task-023, IT-002 |
| FND-003 | high | Completion and promotion readiness remains `not_started` | Task-024, TC-007..TC-013, IT-002 |
| FND-004 | medium | Five pre-existing test symbols carry `IT-001`, but the active declarations do not mint an `IT-001` target | tests/admission.rs, tests/authority.rs |

## Task-021 closure

- Every Task-021 subtask is checked and its plan/task status is `done`.
- Authority v2 is a standalone draft-2020-12 schema with exactly one population, position,
  clock, progress, closure, completeness and availability component. Its seven embedded owner
  payloads are closed full shapes, and schema plus typed producer tests reject role omission,
  duplication, field deletion, nonempty state and unavailable empty state. V1 schema bytes,
  digest, API and the pinned canonical fixture remain unchanged.
- Query selection binds the exact lineage/head/revision; five required and two optional
  component identities; population, membership and scope; directed relationship, root, grouping,
  effect kind and signal; duplicate policy; and typed plan. Foreign axes fail closed.
- Explicit population members are joined to completeness, observation, position, relationship,
  capture and activation evidence. Ambient nonmembers fail admission, missing relationships
  refuse, and observation-anchored duplicate/contradiction/unresolved-correlation conflicts are
  typed incomplete outcomes.
- Snapshot and half-open window membership, adjacent-window boundaries, empty complete
  populations, deterministic filter/count/sum, effect replay, receipt replay under both policies,
  exact units/representations, checked prefixes, overflow and inclusive result bounds all have
  traced unit or property evidence.
- Work, retained state, output and input scans are metered before allocation or retention. The
  final pinned two-member oracle is 47,968 input bytes, 2 members, 2 occurrences, 2 arithmetic
  steps, 101 work units, 1,160 retained-state bytes and 2,885 output bytes, with every lowered
  exact limit admitted and one-under limit refused without an aggregate.
- Independent semantic and Rust reviews drove removal of label-derived receipt logic, shallow v2
  owner shapes, unmetered scans, pre-charge allocation, a tautological authority check, literal
  captured-total evidence, and missing ambient/relationship/coherence adversarial fixtures.

## Coverage

- Reconciliation used the explicitly permitted npm Quire 0.32.0 fallback because native Quoin
  does not yet expose `coverage` (agent-ix/quoin#538).
- Tasks done: 6/9, with dependency order and plan/task checkboxes consistent.
- Rows backed by tagged tests: 87/89. Rust evidence is 95 bound/tagged symbols out of 107
  candidates. FR-009 and FR-011 are 6/6 each; TC-009 and TC-011 are backed.
- The only unbacked rows are NFR-003-AC-1 and TC-013, both intentionally assigned to Task-022.
  There are zero status lies and the same five pre-existing untracked `IT-001` symbols.

## Validation and execution evidence

- Native `quoin` 0.23.1-87-gb3d02d4, `quoin validate --repo . --strict`: PASS, no findings.
- Native `quoin validate --scope ...` regression is recorded with a minimal reproducer and full
  environment details in agent-ix/quoin#541; scoped validation passed through the permitted Quire
  fallback with only pre-existing module warnings.
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS.
- `cargo test --locked --all-targets --all-features`: PASS, 107 tests, zero failed or ignored.
- `cargo rustdoc --locked --all-features -- -D warnings`: PASS.
- `cargo deny check`: PASS (`advisories`, `bans`, `licenses`, `sources`).
- Standalone authority v2 schema compilation plus emitted, mutated and field-deleted instance
  validation: PASS.
- No production panic, unsafe block, unchecked integer cast, recursion, async/blocking or lock
  discipline issue remains in the Task-021 delta.
