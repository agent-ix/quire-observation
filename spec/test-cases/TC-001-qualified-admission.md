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
closure, clock family, owner-derived observation/membership/population identity,
observation-source population, and exact record/member bounds. Closure bytes are
not interpreted here, but their explicitly selected identity and raw digest are
retained without substitution.

## Test Procedure

Construct a valid selected package, Producer interface 1.2.0 selection,
relationship set, finite scope, and record. Run admission for the valid input,
then repeat after each independent mutation and at exactly/one-over each record,
member, relationship, and required-relationship bound.

## Expected Results

The valid input is available with retained provenance. Selection substitutions,
ambiguous/conflicting relationships, and over-limit inputs are refused. Missing
values, closure, membership, or relationship inputs are incomplete and never
represented as Boolean false.
