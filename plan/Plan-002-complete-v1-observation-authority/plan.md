---
id: Plan-002
title: "quire-observation — complete V1 observation authority"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-observation/FR-007
    type: references
  - target: ix://agent-ix/quire-observation/FR-008
    type: references
  - target: ix://agent-ix/quire-observation/FR-009
    type: references
  - target: ix://agent-ix/quire-observation/FR-010
    type: references
  - target: ix://agent-ix/quire-observation/FR-011
    type: references
  - target: ix://agent-ix/quire-observation/NFR-003
    type: references
---
# Implementation Plan: complete V1 observation authority

This plan implements the reviewed C00 extension after the completed owner-contract
Plan-001. It preserves the transport-independent boundary: QObs owns uncertainty,
activation, revision lineage, repair coordination and closed-population queries, while
selected temporal and protocol evaluators retain their own semantics.

## Requirements Summary

### Functional Requirements

- [x] **FR-007**: preserve coherent partial values and exact interval-order uncertainty.
- [x] **FR-008**: bind immutable activation capture and independent scope-authority axes.
- [x] **FR-009**: publish canonical revisioned I07 bundles and strict lineage views.
- [x] **FR-010**: derive bounded affected regions and coordinate replay/incremental parity.
- [ ] **FR-011**: evaluate exact filter/count/sum queries over closed qualified populations.

### Non-Functional Requirements

- [ ] **NFR-003**: reproduce every C00 result under finite caller-lowered bounds.

## Dependency Graph

### Core dependency edges

- `FR-007 -> FR-008`: activation, progress and lateness use the exact partial-value and
  interval relation vocabulary.
- `FR-007 + FR-008 -> FR-009`: the I07 bundle must carry both uncertainty facts and
  independent activation/scope authority.
- `FR-009 -> FR-010`: repair starts only from validated predecessor/replacement lineage.
- `FR-008 + FR-009 -> FR-011`: definitive queries need independent closure/completeness
  authority and one exact bundle revision.
- `FR-010 + FR-011 -> IT-002`: the cross-owner handoff demonstrates both repaired results
  and complete aggregate accounting.

### Shared dependencies

- Canonical identity, bounded encoding, strict-reader preflight and typed refusal helpers in
  `src/authority/common.rs` remain the only shared owner-document primitives.
- FR-010 and FR-011 share bounded explicit graph/population traversal but not evaluator or
  aggregation semantics; the shared part is limited to checked work accounting.

### Cross-cutting constraints

- `NFR-003` applies before retention, traversal, arithmetic, serialization and validation in
  every FR-007 through FR-011 path.
- Existing FR-001 and FR-004 validated views remain prerequisites and are not reimplemented.

### The seams

New owner modules attach below `src/authority/mod.rs` and reuse `authority/common.rs` for
canonical envelopes and checked limits. Repair consumes evaluator contributions through a
typed trait seam without interpreting them. Cross-project evidence lives in a dedicated
integration test and uses pinned strict readers rather than copied producer semantics.

## Test Plan

### Authority unit and property tests

- [x] **TC-007**: coherent value sets, exact interval relations, cross-wire refusals and
  presentation-order metamorphisms for FR-007-AC-1 through FR-007-AC-5.
- [x] **TC-008**: activation identity, immutable captures, independent state products,
  silence coverage and lateness for FR-008-AC-1 through FR-008-AC-6.
- [x] **TC-009**: canonical revision bytes, predecessor/replacement lineage, replay,
  stale/branched views and strict-reader mutations for FR-009-AC-1 through FR-009-AC-5.
- [x] **TC-010**: exact affected closure, unaffected-byte retention, decisive prefixes,
  evaluator failure and batch/incremental parity for FR-010-AC-1 through FR-010-AC-6.
- [ ] **TC-011**: closed membership, window boundaries, duplicate policy, receipt exclusion
  and checked-prefix arithmetic for FR-011-AC-1 through FR-011-AC-6.

### Integration tests

- [ ] **IT-002**: pass real I07 views through pinned QSL/TL/Protocol/composed-runtime
  consumers and compare initial, incremental, repaired and aggregate results.

### Verification

- [ ] **TC-012**: generated equal-input, permutation and exact/one-over resource properties
  for NFR-003-AC-2.
- [ ] **TC-013**: static inventory proving every expansion/allocation path has a dominating
  checked limit for NFR-003-AC-1.

## Remaining Work

### Track A: Critical path (serial)

- **A1 = Task-016** partial values and interval relations — Medium; exit: no input order or
  midpoint can collapse an admissible interval order.
- **A2 = Task-017** activation and independent authority — Hard; exit: every axis remains
  separately typed and cross-wired inputs fail closed.
- **A3 = Task-018** revisioned I07 bundles — Hard; exit: exact replay is idempotent while
  stale or already-branched lineage cannot publish.
- **A4 = Task-019** bounded affected-region planning — Hard; exit: only explicit observable
  dependents enter a finite canonical repair plan.
- **A5 = Task-020** evaluator coordination and parity — Hard; exit: batch and incremental
  paths agree without QObs interpreting evaluator semantics.

### Track B: Post-bundle parallel feature

- **B1 = Task-021** closed-population queries — Hard; exit: definitive values exist only for
  complete qualified populations under the selected duplicate and arithmetic policy.

### Track C: Assurance and integration gates

- **C1 = Task-022** bounded-work verification — Medium; exit: every exact/one-over property
  and static expansion inventory passes.
- **C2 = Task-023** pinned consumer integration — Hard; exit: real consumers preserve
  uncertainty, repaired lineage and aggregate accounting byte-for-byte.
- **Gate = Task-024** completion and promotion readiness — Medium; measures trace coverage
  and all local gates; pass: 90/90 matrix rows backed with zero status lies and no open
  high/medium review finding.

## Parallel Execution Summary

```text
Task-016 -> Task-017 -> Task-018 -> Task-019 -> Task-020 --\
                              \-> Task-021 --------------+-> Task-022 -> Task-023 -> Task-024
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-016 | A | FR-007, NFR-003 | TC-007, TC-012 | done |
| Task-017 | A | FR-008, NFR-003 | TC-008, TC-012 | done |
| Task-018 | A | FR-009, NFR-003 | TC-009, TC-012 | done |
| Task-019 | A | FR-010, NFR-003 | TC-010, TC-012 | done |
| Task-020 | A | FR-010 | TC-010 | done |
| Task-021 | B | FR-011, NFR-003 | TC-011, TC-012 | not_started |
| Task-022 | C | NFR-003 | TC-012, TC-013 | not_started |
| Task-023 | C | FR-007..FR-011 | IT-002 | not_started |
| Task-024 | Gate | FR-007..FR-011, NFR-003 | TC-007..TC-013, IT-002 | not_started |

## Coordination Rules

- Preserve Plan-001 owner schemas and public readers unless a versioned successor is added;
  never mutate existing schema bytes in place.
- Keep evaluator traits incapable of parsing formulas or manufacturing temporal/protocol
  results; only exact inputs and opaque typed contributions cross the seam.
- Keep revision publication pure. Atomic compare-and-swap remains an external store duty.
- Use a single writer for `authority/common.rs` and `authority/mod.rs`; feature modules and
  their tests may proceed independently after Task-018 stabilizes shared types.
- Do not mark matrix rows complete until executable trace tags bind the exact criteria.
