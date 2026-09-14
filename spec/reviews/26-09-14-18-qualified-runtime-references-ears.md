---
id: SR-031
title: "EARS review of authority-qualified runtime references"
type: SpecReview
analysis: ears-conformance
scope: "spec/functional/FR-005-adopt-authority-qualified-runtime-references.md"
review_set: subset
---
# SR-031: EARS review of authority-qualified runtime references

## Summary

FR-005's event-driven admission clause and its ubiquitous subsystem constraints
were reviewed for explicit actor, condition, response and modal force. The
requirement is EARS-compatible and Quire reports no requirement-grammar finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-029 | low | No unresolved EARS finding. | FR-005 |

## Conformance results

- The primary event-driven statement uses `When <event>, the library SHALL
  <response>, or SHALL <refusal>` and names the acting system.
- Ubiquitous constraints use `SHALL` or `SHALL NOT`, keep one authority or
  comparison rule per sentence, and name the refusing subsystem where absence
  cannot be represented by construction.
- Correlation clauses define the complete state partition and map each trigger
  cardinality to exactly one response.
- Bound clauses name the threshold condition and require refusal before partial
  retention.
- Acceptance criteria are observable mutations or boundary examples and do not
  weaken the normative clauses into implementation suggestions.

## Evidence

`quire validate --scope . --summary` reports FR-005 grammar-clean with no EARS
grammar finding.

## Verdict

PASS. The reviewed requirement may enter its TDD implementation cycle.
