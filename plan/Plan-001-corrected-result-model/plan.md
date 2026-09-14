---
id: Plan-001
title: "quire-observation — complete observation-owner contracts"
type: Plan
status: complete
relationships:
  - target: ix://agent-ix/quire-observation/FR-001
    type: references
  - target: ix://agent-ix/quire-observation/FR-002
    type: references
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/FR-004
    type: references
  - target: ix://agent-ix/quire-observation/NFR-001
    type: references
  - target: ix://agent-ix/quire-observation/NFR-002
    type: references
---
# Implementation Plan: complete observation-owner contracts

The accepted ecosystem ownership correction replaced the former replay/result-handoff
model with a transport-independent observation-owner boundary. This living plan preserves
Tasks 001–007 as explicitly superseded history and governs the replacement implementation
tracked by `quire-observation#15` in Tasks 008–013. All implementation-plan work is now
complete and ready for its independent gap-analysis and repository promotion gates.

## Requirements Summary

### Functional Requirements

- [x] **FR-001**: qualify explicit observation admission and owner-derived identities.
- [x] **FR-004**: publish the closed owner schema, identity, derivation, limit, and reader
  architecture shared by every artifact.
- [x] **FR-002**: derive all owner artifacts identically from batch and checked incremental
  histories without evaluating temporal or protocol truth.
- [x] **FR-003**: expose complete constructor-private validated views or typed refusal.

### Non-Functional Requirements

- [x] **NFR-001**: enforce admission and owner-document resource bounds without partial
  qualified state, document, or view.
- [x] **NFR-002**: reproduce exact bytes, identities, authority, subject, and lineage.

## Dependency Graph

### Core dependency edges

- `FR-001 -> FR-004`: canonical owner contracts consume only qualified admission.
- `FR-004 -> FR-002`: the common envelope and nine contract modules define the artifacts
  that batch and incremental histories derive.
- `FR-004 + FR-002 -> FR-003`: public views and strict readers require both the contract
  definitions and deterministic expected derivation.

### Shared dependencies

- The common owner envelope, content identity, lineage, bounded scanner, canonical encoder,
  error catalog, and validated-view capability are shared by all nine owner modules.
- Exact observation, membership, population, clock, and cutoff identities are shared by
  admission and downstream artifact derivation and therefore have one implementation.

### Cross-cutting constraints

- `NFR-001` applies to admission, identity helpers, every deriver, the incremental-history
  capability, and every strict reader.
- `NFR-002` applies to every canonical encoding and every batch/incremental path.

### The seams

`src/lib.rs` owns semantic admission. `src/authority/common.rs` owns the bounded envelope,
lineage, strict-reader preflight, and error vocabulary. The nine sibling authority modules
own only their contract-specific selections and payloads. `tests/authority.rs` exercises the
complete integration without importing an evaluator, parser, transport, or protocol result.

## Test Plan

### Admission and identity tests

- [x] **TC-001**: validate exact selection retention, typed incomplete/refused outcomes,
  owner-derived identities, strict membership, and exact/one-over population bounds.

### Owner-contract integration tests

- [x] **TC-002**: compare all nine documents across batch and checked incremental history;
  verify sequence refusal, boundary mapping, state independence, and ambiguous order.
- [x] **TC-003**: read every complete payload fact and reject unsupported, omitted,
  duplicated, reordered, noncanonical, or cross-wired representations.
- [x] **TC-004**: pin schema digests, owner maxima and error codes; test every contract,
  identity, correction, state cross-product, field mutation, and exact/one-over limit.
- [x] **IT-001**: join real admission, both history paths, all derivers and all strict readers,
  including incomplete, ambiguous, open, and late variants.

### Verification

- [x] **NFR-001**: assert zero excess retained state and no partial view/document on refusal.
- [x] **NFR-002**: assert complete byte equality and exact validated envelope selections.

## Completion

Tasks 008–013 are complete. No implementation-plan work remains.

### Track A: Critical path

- **A1 = Task-008** owner architecture and requirements — complete; exit: one acyclic
  ownership model governs admission, derivation, and reading.
- **A2 = Task-009** common strict boundary — complete; exit: all owner modules share one
  bounded canonical envelope and typed refusal model.
- **A3 = Task-010** nine owner contracts — complete; exit: every selected owner artifact has
  immutable schema bytes, deterministic derivation, and strict reading.
- **A4 = Task-012** consumer integration — complete; exit: all facts remain independently
  readable and every state combination remains non-Boolean.
- **Gate = Task-013** review and promotion readiness — complete; exit: all
  specification/code/Rust findings resolved, required local gates green, and the exact
  promotion handoff recorded.

### Track B: Admission integration

- **B1 = Task-011** harden admission and owner identity helpers — complete; exit: qualified
  state cannot carry stale owner identities, foreign clocks, or over-bound relationships.

## Parallel Execution Summary

```text
A1 -> A2 -> A3 -> A4 -> Task-013 review/readiness
           \-> B1 --/
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-008 | A | FR-001..FR-004, NFR-001..NFR-002 | TC-001..TC-004, IT-001 | done |
| Task-009 | A | FR-004, NFR-001, NFR-002 | TC-004 | done |
| Task-010 | A | FR-004 | TC-004 | done |
| Task-011 | B | FR-001, NFR-001 | TC-001 | done |
| Task-012 | A | FR-002, FR-003, NFR-002 | TC-002, TC-003, IT-001 | done |
| Task-013 | Gate | FR-001..FR-004, NFR-001..NFR-002 | TC-001..TC-004, IT-001 | done |

Tasks 001–007 are `done` with `resolution: superseded`; their files retain the historical
design record but contain no remaining implementation work.

## Coordination Rules

- Observation owner modules must not acquire parser, evaluator, transport, protocol-result,
  truth, settlement, conformance, or Boolean-coercion responsibilities.
- Every clock-bearing artifact binds both exact clock identity and exact revision.
- Every public strict reader derives its expected document from independent selections and
  returns only a constructor-private validated view.
- Schema byte changes require simultaneous digest constant, README handoff, and traced test
  updates.
- Repository promotion follows an independent gap-analysis PASS; the GitHub issue records
  the merge and tracker lifecycle rather than making plan completion self-referential.
