---
id: NFR-003
title: "Bound and reproduce revisioned observation work"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/quire-observation/FR-007
    type: constrains
  - target: ix://agent-ix/quire-observation/FR-008
    type: constrains
  - target: ix://agent-ix/quire-observation/FR-009
    type: constrains
  - target: ix://agent-ix/quire-observation/FR-010
    type: constrains
  - target: ix://agent-ix/quire-observation/FR-011
    type: constrains
---
# NFR-003: Bound and reproduce revisioned observation work

## Statement

The library SHALL keep revisioned observation work reproducible and fail-closed
under finite caller-lowered limits for every retained revision, dependency edge,
possible order, population member, arithmetic step, and output byte.

## Scope

- Applies to FR-007 through FR-011 batch, incremental, repair, strict-read, and
  population-query paths.
- Excludes downstream evaluator execution cost, network transport, and external
  effect latency.

## Rationale

Uncertainty and revision repair can otherwise create unbounded order expansion or
silently different results. Fail-closed bounds and canonical outputs make the
authority usable in qualification and deterministic replay.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Equal-input canonical byte equality | 100% of generated fixtures | 100% | property-based-testing (TC-012) |
| Retained items after one-over bound | 0 excess items | 0 | property-based-testing (TC-012) |
| Partial artifact/view after refusal | 0 artifacts or views | 0 | unit-testing (TC-012) |
| Unaffected result identity/byte changes after repair | 0 changes | 0 | property-based-testing (TC-010) |
| Semantic outcomes changed by input permutation or arrival order | 0 changes | 0 | metamorphic-testing (TC-012) |

## Verification

Generated fixtures permute presentation and arrival order, repeat exact inputs,
exercise exact and one-over limits for every new collection, and compare complete
canonical bytes plus stable error codes. Repair fixtures additionally compare all
unaffected identities and bytes before and after a replacement. TC-013 statically
maps every expansion and allocation path to its dominating checked limit.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-003-AC-1 | The implementation uses no unbounded expansion of overlapping interval orders or dependency closure. | static-quality (TC-013) |
| NFR-003-AC-2 | Every one-over resource case fails before retaining a partial new bundle, repair plan, aggregate, or validated view. | property-based-testing (TC-012) |

## Dependencies

- [NFR-001](./NFR-001-bound-retained-observation-state.md) defines the existing
  owner maxima and fail-closed resource behavior.
- [NFR-002](./NFR-002-reproduce-observation-outcomes.md) defines current owner-
  artifact reproducibility; this requirement extends it to C00 behavior.
