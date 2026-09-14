---
id: Task-015
title: "Close finite-topology and anchored-boundary evidence"
type: Task
status: done
relationships:
  - target: ix://agent-ix/quire-observation/FR-006
    type: references
  - target: ix://agent-ix/quire-observation/TC-006
    type: verifies
---
# Task-015: Close finite-topology and anchored-boundary evidence

## Objective

Close `quire-observation#13` against implemented reality: preserve arbitrary
well-typed finite relationship topology without adding a graph evaluator, and
prove the already implemented FR-289 boundary mapping across every admitted
clock family and adverse boundary case.

## Subsystems

1. Expose the real admitted FCD fixture's existing `Order-successor` same-kind
   relationship declaration in test support for loops and cycles.
2. Add TC-006 admission tests for self-loop, cycle, disconnected component,
   replay and exact/one-over offered populations.
3. Add TC-006 authority tests for all clock families, point/deadline/carrier
   boundaries, checked overflow and independent scope/clock cross-wiring.
4. Prove immutable progress/closure schemas and the absence of an evaluator or
   traversal API; change production Rust only if a reviewed acceptance case
   exposes a real implementation gap.
5. Run `/rust-review` and `/gap-analysis`, fix every finding, merge the one PR
   closing #13, and immediately reconcile #1, Plan-001, QS #1 and CAP-011.

## Boundaries

- Do not introduce graph reachability, closure, path-depth, temporal replay,
  settlement, conformance, result or truth semantics.
- Do not edit FCD, Contract-IR, protocol or Agent B's TL-* repositories.
- Do not change any existing owner schema or identity preimage.

## Evidence

- Specification: FR-006, TC-006, scoped base and EARS self-reviews.
- Implementation: production-reaching Rust tests tagged TC-006/FR-006.
- Gates: targeted tests during development; one full local Rust gate plus
  `/rust-review` and `/gap-analysis` at PR readiness.

## Completion

Completed on 2026-09-14 against QObs main `8d54458`.

- The pinned FCD fixture's existing `Order-successor` declaration proves exact
  self-loop, directed-cycle and disconnected-component retention.
- Event-position, fixed-sample and timestamped-event cases prove point,
  inclusive-upper, excluded-end, carrier-slack, overflow, watermark and
  independent scope/clock refusal behavior.
- No production Rust, schema, identity preimage or public evaluator API changed;
  the executable evidence confirmed the implemented owner boundary.
- The final local gate passed 46 Rust tests, dependency policy, 52/52 matrix
  coverage, SR-036 Rust review and SR-037 gap analysis.
