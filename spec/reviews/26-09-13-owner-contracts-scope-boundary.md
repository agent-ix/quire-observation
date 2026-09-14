---
id: SR-026
title: "Scope-boundary review of the observation-owner contract system"
type: SpecReview
analysis: scope-boundary
scope: "spec/"
review_set: all
---
# SR-026: Scope-boundary review of the observation-owner contract system

## Summary

Responsibilities were allocated across QObs, QSL, shared specification, TL/protocol, and
Contract-IR boundaries. The final QObs scope owns observation authority completely without
absorbing parsing, evaluation, transport, or result ownership.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings after remediation | - |

## Allocation

- **QObs owns:** qualified admission; observation, explicit-members, population, position,
  clock, capture, progress, closure, completeness, and result-availability authority;
  canonical bytes, identities, schemas, limits, correction lineage, and strict readers.
- **QSL owns:** native source parsing, semantic package compilation, and package selections.
- **Shared specification owns:** cross-ecosystem semantic identity and boundary meanings.
- **TL/protocol owners own:** temporal interpretation, protocol conformance, settlement,
  truth, and result canonicalization.
- **Contract-IR owns:** bridges among already validated owner views, not replacement parsers
  or inferred Boolean meaning.
