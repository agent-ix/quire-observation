---
id: TC-004
title: "Publish and read observation authority artifacts"
type: TC
relationships:
  - { target: ix://agent-ix/quire-observation/FR-004, type: verifies }
---
# TC-004: Publish and read observation authority artifacts

## Description

Complete owner-artifact checks for FR-004.

## Test Procedure

Rust integration/property tests derive all nine contracts from the same real
qualified admission through batch and incremental paths, read each against
independent expected selections, recompute schema/content identities, enumerate
every independent state combination and mutate every field, order, scope,
clock, boundary, predecessor and resource limit in FR-004.

## Expected Results

Equal inputs yield equal canonical bytes. Only exact owner bytes and expected
selections produce validated views. One-over and malformed inputs yield one
typed refusal/incomplete report and no partial value. No output contains or
implies protocol/TL truth, settlement, conformance or a Boolean verdict.

## Status

Implemented for `quire-observation#15` in `tests/authority.rs`.
