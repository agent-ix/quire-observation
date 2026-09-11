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
---
# IT-001: Hand qualified observation assessment to a consumer

## Objective

Verify that a qualified observation can move from admission through replay to a
consumer handoff without losing its selected identities or changing an
incomplete/non-success disposition into Boolean success.

## Target Integration

The integration joins this library's admission, replay, and result-handoff
interfaces; the downstream temporal/protocol consumer is represented by a typed
in-process test sink, not a production adapter.

## Preconditions

A fixed selected package, Producer interface 1.2.0 selection, finite scope,
progress assertion, and bounded fixture records are available.

## Inputs

One valid timed-refund fixture plus variants with a missing provider record, an
ambiguous refund relationship, a quiet deadline, and a late contradiction.

## Test Procedure

1. Admit the valid fixture using explicit selections and limits.
   - IT-001-SC-01: admission returns qualified records with provenance.
2. Evaluate the same history by replay and incrementally.
   - IT-001-SC-02: both paths return the same full disposition.
3. Emit the result to the typed consumer sink.
   - IT-001-SC-03: the sink receives every required result axis and dependency.
4. Repeat with the adverse variants.
   - IT-001-SC-04: no adverse variant is represented as passed.

## Expected Results

The valid fixture remains qualified and deterministic. Missing, ambiguous, late,
or unsupported inputs retain their typed non-success meanings through handoff.
