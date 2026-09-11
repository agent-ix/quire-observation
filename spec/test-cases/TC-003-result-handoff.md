---
id: TC-003
title: "Preserve result facts through consumer handoff"
type: TC
relationships:
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "verifies"
---
# TC-003: Preserve result facts through consumer handoff

## Description

Verify that every assessment disposition and each of the seventeen result axes
FR-003 declares — assessment identity, disposition, settlement, support, progress
identity, decision progress, decision closure, surrounding progress, surrounding
closure, global closure, activation, participation, completeness, provenance,
dependency, retained-event, and late-supersession — survive a typed consumer
handoff.

## Test Procedure

Map healthy, violating, untriggered, incomplete, unsupported, and late
superseding results to a typed sink. Independently omit, duplicate, cross-wire,
or make a target representation unavailable for each required axis, one axis at a
time, and assert the omission cases cover FR-003's declared seventeen-axis set
exhaustively.

## Expected Results

Valid mappings retain all facts. Invalid mappings refuse or report explicit loss;
no consumer result becomes passed merely because information is absent or lossy.
