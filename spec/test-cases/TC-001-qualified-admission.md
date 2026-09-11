---
id: TC-001
title: "Qualify explicit observation admission"
type: TC
relationships:
  - target: "ix://agent-ix/quire-observation/FR-001"
    type: "verifies"
---
# TC-001: Qualify explicit observation admission

## Description

Verify valid qualified admission and independently mutate selected entity,
source, schema, signal, unit, producer version, relationship, required value,
closure, clock family, and exact record/member bounds. Also verify that an
unrecognized member object identity and an unrecomputed membership or closure
digest leave an otherwise qualified admission available.

## Test Procedure

Construct a valid selected package, Producer interface 1.2.0 selection,
relationship graph, finite scope, and record. Run admission for the valid input,
then repeat after each independent mutation and at exactly/one-over each bound.

## Expected Results

The valid input is available with retained provenance. Selection substitutions,
ambiguous/conflicting relationships, and over-limit inputs are refused. Missing
values, closure, membership, or relationship inputs are incomplete and never
represented as Boolean false.
