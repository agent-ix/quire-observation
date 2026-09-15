---
id: TC-009
title: "Publish immutable revisioned authority bundles"
type: TC
relationships:
  - target: ix://agent-ix/quire-observation/FR-009
    type: verifies
---
# TC-009: Publish immutable revisioned authority bundles

## Description

Verify canonical I07 bundle identity, strict reading, replay, and linear explicit
revision lineage across the complete component set.

## Test Procedure

1. Generate valid initial bundles, permute presentation order, and compare bytes
   and identities across repeated derivation and strict reading.
2. Strict-read the bounded lineage view, replace one fact at a time with the same
   semantic role and subject, derive the next revision and successor lineage,
   and retain the prior bytes for comparison.
3. Exercise missing predecessor, stale/non-head predecessor, non-increasing
   revision, already-recorded sibling, self-link, cross-authority, cross-scope,
   and cross-role replacement variants.
4. Replay identical bundles and then reuse the same authority/scope/revision key
   with one semantic byte changed.
5. Omit, duplicate, cross-wire, and exceed the bound for each component family.

## Expected Results

Equal complete inputs produce one canonical identity. Valid replacements retain
old/new facts and direct lineage without rewriting prior bytes. Every invalid
supplied lineage, identity contradiction, component mutation, or one-over input
refuses before a bundle, successor lineage, or validated view exists.
