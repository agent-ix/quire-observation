---
id: TC-005
title: "Admit authority-qualified runtime references"
type: TC
relationships:
  - target: "ix://agent-ix/quire-observation/FR-005"
    type: "verifies"
---
# TC-005: Admit authority-qualified runtime references

## Description

Verify the complete FCD-qualified subject and relationship boundary through
strict one-axis mutations, exact replay, bounded correlation, presentation
invariance and non-causal key ordering.

## Test Procedure

1. Admit one Producer interface 1.2 static bundle through the pinned FCD Rust
   boundary and use that capability for every subject and relationship.
2. Independently mutate every authority revision and digest component, subject
   kind and object byte; remove or empty every runtime member.
3. Compare same-object, foreign-authority and wrong-kind subjects and permute a
   subject collection while changing only non-key presentation/transport facts.
4. Resolve one runtime relationship against its exact producer declaration;
   then mutate its declaration, identity, authority, endpoint kind and endpoint
   order independently.
5. Exercise zero, one, exact-replay, conflicting, same-identity incompatible and
   multiple-distinct candidates for one required relationship slot.
6. Repeat at exactly and one over both relationship limits.
7. Compare the retained v1 schema bytes to their pre-#18 digests and validate
   that the authority-qualified record and population documents name v2.

## Expected Results

Only exact admitted producer facts and exact ordered endpoint values resolve.
Missing input is incomplete; conflicting and ambiguous input refuses; malformed,
foreign, wrong-kind, unknown-declaration and identity-contradictory input refuses
at admission. Exact replay is idempotent. Key order is stable across permutation
and establishes no causal or transport relation.
