---
id: FR-007
title: "Preserve partial values and exact clock intervals"
type: FR
relationships:
  - target: ix://agent-ix/quire-observation/US-001
    type: implements
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-160
    type: references
  - target: ix://agent-ix/quire-specification/FR-325
    type: references
---
# FR-007: Preserve partial values and exact clock intervals

## Description

When an admitted observation is partial or clock-uncertain, the library SHALL
retain its nonempty Boolean possibility set, typed evidence reason, and exact
closed event-time interval under one selected clock and unit.

## Inputs

- One admitted observation identity and authority-qualified subject.
- A nonempty subset of `{true, false}` and exactly one value reason.
- A selected clock identity/revision, exact unit identity, signed integer
  earliest/latest endpoints, and separate ingestion position.
- Caller-lowered resource limits.

## Outputs

- A canonical partial-observation fact suitable for inclusion in the I07
  authority bundle.
- A typed refusal without a partial fact when the value set, reason, interval,
  clock, unit, or bounds are invalid.

## Behavior

- The library SHALL represent known false and known true as singleton sets.
- The library SHALL represent missing and conflicting evidence as `{false,true}`
  with distinct `missing` and `conflicting` reasons.
- The library SHALL refuse an empty possibility set, a singleton paired with a
  missing/conflicting reason, or a two-valued set paired with a known reason.
- The library SHALL accept an event-time interval only when its endpoints share
  the exact selected clock, revision, and unit and `earliest <= latest`.
- The library SHALL classify interval `a` as definitely before interval `b` only
  when `a.latest < b.earliest`.
- The library SHALL retain every admissible before/equal/after order when two
  intervals overlap.
- The library SHALL NOT select an interval midpoint, ingestion position,
  collection order, or timestamp spelling as semantic order.
- The library SHALL refuse arithmetic or comparison across different clocks,
  revisions, or units rather than infer a conversion.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | Known true, known false, missing, and conflicting inputs retain their exact possibility sets and distinct reasons. | unit-testing (TC-007) |
| FR-007-AC-2 | Empty or reason-inconsistent possibility sets refuse without an authority fact. | unit-testing (TC-007) |
| FR-007-AC-3 | Disjoint intervals establish only the order required by their endpoints; overlapping intervals retain every admissible order. | property-based-testing (TC-007) |
| FR-007-AC-4 | Reversing an interval or cross-wiring its clock, revision, or unit refuses before comparison. | unit-testing (TC-007) |
| FR-007-AC-5 | Mutating ingestion order or an interval midpoint cannot change the semantic order relation. | metamorphic-testing (TC-007) |

## Error Conditions

An empty or reason-inconsistent possibility set, reversed endpoint range,
cross-clock/revision/unit comparison, unsupported representation, arithmetic
overflow, or exceeded resource bound returns a typed refusal without a partial
authority fact.

## Dependencies

- [FR-004](./FR-004-publish-observation-authority-artifacts.md) owns the existing
  canonical authority-document and strict-reader boundary.
- QSpec FR-160 owns the multi-valued and interval-order semantics adopted here;
  QSpec FR-325 owns the I07 bundle shape.
