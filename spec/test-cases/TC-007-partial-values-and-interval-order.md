---
id: TC-007
title: "Preserve partial values and interval order"
type: TC
relationships:
  - target: ix://agent-ix/quire-observation/FR-007
    type: verifies
---
# TC-007: Preserve partial values and interval order

## Description

Property-test the complete partial-Boolean domain and exact interval-order
relation without allowing arrival order or midpoint time to become authority.

## Test Procedure

1. Generate every nonempty subset of `{false,true}` with every value reason and
   verify that only the FR-007 coherent pairings admit.
2. Generate signed integer point, disjoint, touching, equal, nested, and partially
   overlapping closed intervals under one exact clock/revision/unit.
3. Compare each interval pair in both directions and verify the returned possible
   order set against endpoint arithmetic.
4. Permute ingestion position and collection order while holding semantic inputs
   fixed and compare canonical bytes and relations.
5. Mutate clock, revision, unit, endpoint order, and each effective resource
   limit independently.

## Expected Results

Known, missing, and conflicting values retain exact distinct meaning. Disjoint
intervals establish only endpoint-required order; overlaps retain uncertainty.
Every invalid or cross-wired input refuses without a partial authority fact, and
non-semantic permutations change no relation or canonical bytes.
