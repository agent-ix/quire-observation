---
id: FR-008
title: "Bind activation, progress, closure, and completeness independently"
type: FR
relationships:
  - target: ix://agent-ix/quire-observation/US-001
    type: implements
  - target: ix://agent-ix/quire-observation/FR-007
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-093
    type: references
  - target: ix://agent-ix/quire-specification/FR-112
    type: references
  - target: ix://agent-ix/quire-specification/FR-113
    type: references
  - target: ix://agent-ix/quire-specification/FR-115
    type: references
---
# FR-008: Bind activation, progress, closure, and completeness independently

## Description

When constructing assessment authority for an obligation, the library SHALL bind
an immutable activation capture and SHALL preserve activation, progress, closure,
completeness, lateness, verdict contribution, and settlement contribution as
independent typed facts.

## Inputs

- An exact obligation, trigger observation, activation interval, and complete
  sorted set of typed capture bindings with source observation identities.
- An exact population/snapshot-or-window selection, required sources, clock,
  progress assertion, closure assertion, completeness assertion, and late cutoff.
- Optional evaluator-owned verdict and settlement contributions, each carrying
  its own identity and support set.

## Outputs

- Canonical immutable activation and scope-authority facts.
- An independent assessment-fact record that does not flatten any axis.
- A typed incomplete or refused outcome naming every missing or inconsistent
  authority premise.

## Behavior

- The library SHALL derive an activation identity from the obligation, trigger,
  activation interval, and complete ordered capture set.
- The library SHALL preserve captured values and provenance from the activation
  revision even when a later observation carries another value.
- The library SHALL classify an obligation as inactive, active, or activation-
  unknown independently of any verdict or settlement contribution.
- The library SHALL preserve open, closed, and incomplete execution inputs
  independently from complete, incomplete, and contradicted evidence.
- When silence advances a deadline, the library SHALL recognize coverage only
  from a matching progress authority whose earliest possible frontier is strictly
  beyond the deadline's latest possible endpoint and whose source set covers
  every required source.
- When a progress interval overlaps the deadline or omits a required source, the
  library SHALL retain an incomplete progress fact rather than claim coverage.
- The library SHALL classify lateness against the exact declared cutoff interval
  and SHALL retain an uncertain-lateness outcome when event or cutoff intervals
  do not establish one relation.
- The library SHALL NOT derive activation, verdict, settlement, closure, or
  completeness from another axis.
- The library SHALL refuse a foreign population, window, clock, source set,
  capture, support identity, or authority revision without emitting a flattened
  fallback record.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-008-AC-1 | Equal captured values under distinct triggers create distinct activation identities and retain their original observation provenance. | Test (TC-008) |
| FR-008-AC-2 | Inactive, active, and activation-unknown remain independent of verdict and settlement; no omitted axis is inferred from another. | Property Test (TC-008) |
| FR-008-AC-3 | Silence covers a deadline only when the matching progress interval is definitely beyond it and covers every required source. | Test (TC-008) |
| FR-008-AC-4 | Overlapping progress/deadline intervals, missing sources, open closure, and incomplete or contradicted evidence remain distinct non-conclusive facts. | Property Test (TC-008) |
| FR-008-AC-5 | Definitely timely, definitely late, and uncertain-lateness outcomes follow interval endpoints and never ingestion order. | Property Test (TC-008) |
| FR-008-AC-6 | Any foreign or cross-wired scope authority refuses without changing an unaffected axis or emitting a fallback result. | Test (TC-008) |

## Dependencies

- [FR-007](./FR-007-preserve-partial-values-and-clock-intervals.md) supplies the
  exact value-set and interval relations.
- QSpec FR-093 and FR-113 own activation/time meaning; FR-112 owns population
  completeness; FR-115 requires the result axes to remain independent.
