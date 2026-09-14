---
id: FR-006
title: "Preserve finite topology and anchored temporal boundaries without evaluation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "depends_on"
  - target: "ix://agent-ix/quire-observation/FR-005"
    type: "depends_on"
  - target: "ix://agent-ix/quire-specification/FR-262"
    type: "references"
  - target: "ix://agent-ix/quire-specification/FR-289"
    type: "references"
---
# FR-006: Preserve finite topology and anchored temporal boundaries without evaluation

## Description

When qualified relationship facts or a temporal boundary are admitted, the
library SHALL retain only the explicitly selected finite edges and the exact
admitted scope/clock mapping, or SHALL refuse incompatible or over-bound input
without evaluating reachability, settlement, conformance, or truth.

## Finite relationship-topology boundary

- The retained relationship population SHALL contain only complete FR-005
  values whose source and target were validated against their exact FCD
  declaration. An endpoint SHALL become a graph vertex only because an admitted
  relationship names it; the library SHALL NOT synthesize a vertex or edge.
- A well-typed self-loop, a directed cycle, and a component disconnected from
  the request's expected subject SHALL remain admissible explicit facts. Their
  topology alone SHALL NOT be treated as a contradiction, causal violation,
  reachability result, protocol result, or reason to discard the component.
- Exact edge replay SHALL retain one value, and the relationship population
  SHALL be placed in the existing exact lexicographic order. Neither replay
  removal nor collection order SHALL change topology or create causal order.
- The existing `max_relationships` and `max_required_relationships` ceilings
  SHALL refuse one-over input before validation, sorting, deduplication, or
  correlation. The library SHALL perform no transitive closure, path search,
  recursive expansion, or implicit traversal, so it SHALL NOT invent a path
  depth or graph-expansion result under those ceilings.
- A future reachability or topology-policy operation would be a separately
  specified evaluator. QSpec FR-043 and Contract-IR #69 remain the owners of
  finite-graph evaluation; observation admission SHALL NOT approximate them.

## Anchored temporal-boundary subsystem

- Every progress or closure boundary SHALL retain the exact admitted scope
  identity, clock identity, clock revision, clock family, half-open carrier and
  native inclusive lower/upper interval. A free `scope_start_nanos` or
  `deadline_nanos` scalar SHALL NOT bypass or duplicate those selections.
- For event-position and fixed-sample clocks, native `[a,b]` SHALL map only to
  the admitted carrier `[a,b+1)`, where `b+1` is the checked integer successor.
  Overflow, a reversed interval, a different carrier start/end, or a different
  clock family SHALL refuse.
- For timestamped-event clocks, native `[a,b]` SHALL map only to the admitted
  carrier `[a,e)` with the exact explicit `e` supplied by the scope and `e > b`.
  The library SHALL NOT invent an epsilon or require `e` to be the arithmetic
  successor of `b`.
- A point interval `[a,a]` SHALL be valid only when its selected carrier end is
  the checked successor for a discrete clock or an explicit greater end for a
  timestamped-event clock. The carrier end itself SHALL remain excluded.
- A closed progress or closure assertion SHALL require its watermark to be
  strictly greater than the inclusive upper bound. A watermark equal to that
  bound SHALL refuse and SHALL NOT establish inclusive coverage.
- Carrier records at the inclusive lower or upper bound SHALL remain admitted;
  a record at the half-open carrier end SHALL refuse admission. Records above
  the native upper bound but below a timestamp carrier end remain carrier facts
  and SHALL NOT be classified here as participating in an assessment.

## Existing contract continuity

The accepted `TemporalBoundary` representation and the progress/closure v1
owner schemas already carry the complete FR-289 mapping. This requirement SHALL
NOT change their schema bytes, add a replay/evaluation API, add a traversal API,
or reintroduce the superseded result model. It closes the residual policy and
executable-evidence gap with exact admission and owner-derivation tests.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-006-AC-1 | Independently valid self-loop, two-vertex cycle and disconnected relationship component are retained exactly; topology alone neither refuses nor removes them. | Test (TC-006) |
| FR-006-AC-2 | No graph order, replay, timestamp or arrival mutation creates an edge, reachability result, transitive closure or causal verdict. | Test (TC-006) |
| FR-006-AC-3 | Exact relationship ceilings admit and one-over inputs refuse before topology work; exact replay still counts once only after the offered-population check. | Test (TC-006) |
| FR-006-AC-4 | Timestamp `[a,b]` maps to the exact admitted `[a,e)` with `e > b`; records at `a` and `b` admit, a record at `e` refuses, and a carrier record between `b` and `e` gains no participation verdict. | Test (TC-006) |
| FR-006-AC-5 | Event-position and fixed-sample intervals require checked `b+1`; a reversed interval, overflow, wrong start/end or wrong clock family refuses. | Test (TC-006) |
| FR-006-AC-6 | Exact clock identity/revision and scope identity are retained; independently cross-wiring any one prevents progress/closure derivation. | Test (TC-006) |
| FR-006-AC-7 | A point interval admits with a valid greater carrier end; watermark equality cannot close it, while a strictly greater watermark can. | Test (TC-006) |
| FR-006-AC-8 | Progress/closure v1 schemas remain byte-identical and no public traversal, replay, settlement or truth API is introduced. | Test (TC-006) |

## Dependencies

- [FR-004](FR-004-publish-observation-authority-artifacts.md) owns the existing
  strict progress/closure boundary artifacts and the no-evaluator boundary.
- [FR-005](FR-005-adopt-authority-qualified-runtime-references.md) owns exact
  finite edge values, replay, correlation and pre-work population ceilings.
- QSpec FR-262 owns explicit relationship facts and QSpec FR-289 owns the exact
  inclusive-to-half-open boundary mapping.

## Status

Implemented and independently reviewed for `agent-ix/quire-observation#13`.
