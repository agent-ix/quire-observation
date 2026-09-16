---
id: TC-013
title: "Audit bounded iteration and allocation"
type: TC
relationships:
  - target: ix://agent-ix/quire-observation/NFR-003
    type: verifies
---
# TC-013: Audit bounded iteration and allocation

## Description

Statically prove that every C00 traversal, order representation, retained
collection, arithmetic loop, and output allocation is dominated by a finite
caller-lowered limit.

## Test Procedure

1. Enumerate the production entry points implementing FR-007 through FR-011.
2. Trace each interval-order, revision-lineage, dependency-closure, population,
   proof-identity reconstruction, arithmetic, and serialization loop to its
   checked effective limit.
3. Verify each retained allocation reserves only after the corresponding bound
   check and uses checked integer conversions and arithmetic.
4. Reject recursive dependency traversal, factorial order materialization,
   unbounded retry, or collection growth without a dominating limit.
5. Bind the audit inventory to the same revision exercised by TC-012's exact and
   one-over property cases.

## Expected Results

Every expansion path has a named finite bound and fail-closed one-over behavior.
No path recursively or factorially materializes possible interval orders, and no
untrusted integer reaches allocation or arithmetic without a checked conversion.
