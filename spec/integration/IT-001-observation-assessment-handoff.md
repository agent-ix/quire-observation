---
id: IT-001
title: "Hand qualified observation assessment to a consumer"
type: IT
relationships:
  - target: "ix://agent-ix/quire-observation/FR-001"
    type: "verifies"
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "verifies"
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "verifies"
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "verifies"
---
# IT-001: Hand qualified observation authority to a consumer

## Objective

Verify that a qualified observation can move from admission through owner
derivation and strict reading without losing selected identities or changing an
independent observation state into a result.

## Target Integration

The integration joins admission, batch/incremental history, all nine owner
derivers and their strict readers. The consumer boundary ends at validated Rust
views; no downstream evaluator or adapter is implemented here.

## Preconditions

A fixed selected package, Producer interface 1.2.0 selection, finite scope,
progress assertion, and bounded fixture records are available.

## Inputs

One valid timed-refund fixture plus variants with a missing provider record,
ambiguous declared order, open progress and a late contradiction.

## Test Procedure

1. Admit the valid fixture using explicit selections and limits.
   - IT-001-SC-01: admission returns qualified records with provenance.
2. Derive the same owner artifacts from batch and incremental history.
   - IT-001-SC-02: both paths return the same canonical bytes.
3. Strict-read the artifacts at the consumer boundary.
   - IT-001-SC-03: each reader yields its exact validated owner view.
4. Repeat with the adverse variants.
   - IT-001-SC-04: no adverse variant is promoted to success or truth.

## Expected Results

The valid fixture remains qualified and deterministic. Missing, ambiguous, late,
or unsupported inputs retain typed observation meanings. No artifact or view
contains a temporal or protocol result.
