---
id: TC-011
title: "Evaluate closed related-workflow populations"
type: TC
relationships:
  - target: ix://agent-ix/quire-observation/FR-011
    type: verifies
---
# TC-011: Evaluate closed related-workflow populations

## Description

Property-test filter, count, and exact sum over authority-qualified reservation,
refund, and related-workflow populations.

## Test Procedure

1. Generate closed explicit-member snapshots and half-open windows with members
   before, at, and after each boundary and compare selected membership with a
   reference set computation.
2. Generate exact typed relationships for concurrent workflows, a foreign member,
   a missing relationship, and ambiguous/conflicting relationship variants.
3. Compare repeated receipts, exact effect replay, distinct effects, and duplicate
   occurrence values under `effect-identity-deduplicating` and
   `occurrence-preserving` policies.
4. Evaluate filter, count, and exact sum in deterministic exposure order,
   retaining every participating fact identity.
5. Exercise empty complete populations, two partial refunds matching a captured
   amount, wrong units/representations, arithmetic overflow, and an invalid
   intermediate prefix with an otherwise in-range final mathematical sum.
6. Repeat with open, unknown, stale, contradicted, incomplete, foreign,
   receipt-as-effect, undeclared-policy, and exact/one-over bounded authority.

## Expected Results

Only complete admitted population members participate. Window boundaries do not
double-count; receipt replay does not create an effect; exact occurrence meaning
is preserved. Incomplete or invalid authority emits no definitive aggregate, and
exact arithmetic never converts, saturates, reorders, or hides an invalid prefix.
