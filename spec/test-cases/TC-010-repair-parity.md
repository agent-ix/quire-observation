---
id: TC-010
title: "Agree batch, incremental, and repaired authority"
type: TC
relationships:
  - target: ix://agent-ix/quire-observation/FR-010
    type: verifies
  - target: ix://agent-ix/quire-observation/NFR-003
    type: verifies
---
# TC-010: Agree batch, incremental, and repaired authority

## Description

Property-test exact dependency-closure invalidation, early settled prefixes, and
batch/incremental/repair parity under revisioned and uncertain event time.

## Test Procedure

1. Generate finite dependency graphs including disconnected components, shared
   dependents, diamonds, and cycles; replace one authority fact and compare the
   repair plan with a reference bounded closure computation.
2. Attach result windows that include, exclude, and overlap the old/new fact
   intervals and verify exact affected-region membership.
3. Feed decisive-witness, decisive-counterexample, unresolved, and closure-
   settled contributions one record at a time; record the first emitted settled
   prefix before supplying unrelated trailing input.
4. Run full batch replay from the same immutable inputs and compare every settled
   axis, support identity, result byte, and supersession link.
5. Permute arrival order for overlapping intervals and verify that no path chooses
   a semantic total order not established by the interval endpoints.
6. Exercise unknown nodes, foreign revisions, graph cycles, and exact/one-over
   node, edge, retained-result, possible-order, and output limits.

## Expected Results

The repair plan contains every and only observable explicit dependents.
Unaffected bytes remain identical. A decisive prefix settles before end-of-input;
an unresolved prefix does not. Batch, incremental, and repaired results agree
exactly, and adverse/bounded cases remain typed without partial repair state.
