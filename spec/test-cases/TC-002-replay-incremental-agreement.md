---
id: TC-002
title: "Agree bounded batch and incremental authority derivation"
type: TC
relationships:
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "verifies"
---
# TC-002: Agree bounded batch and incremental authority derivation

## Description

Verify that batch and incremental intake of the same qualified history derive
byte-identical observation-owner artifacts without evaluating temporal truth or
protocol settlement.

## Test Procedure

Construct one FR-001 qualified history through the batch constructor and through
checked incremental intake. Derive all nine FR-004 artifacts from both. Exercise
open/closed progress and closure, incomplete/contradicted completeness, late
classification, ambiguous declared positions, cross-wired clock selections,
correction lineage, and exact/one-over resource ceilings.

## Expected Results

Both modes return byte-identical owner artifacts for equal inputs. Open, closed,
incomplete, contradicted, ambiguous-order, late, cross-wired and exhausted cases
remain typed owner facts or errors. No path emits truth, settlement, conformance
or a Boolean result. A selected correction creates a direct immutable artifact
lineage; prior bytes are never rewritten.
