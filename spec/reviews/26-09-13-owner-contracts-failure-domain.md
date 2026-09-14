---
id: SR-021
title: "Failure-domain review of the observation-owner contract system"
type: SpecReview
analysis: failure-domain
scope: "spec/"
review_set: all
---
# SR-021: Failure-domain review of the observation-owner contract system

## Summary

The review covered identity confusion, partial outputs, unknown representations,
topological edge cases, correction lineage, and state-axis coercion across the full owner
boundary. The final specification fails closed without assigning evaluator or protocol
semantics to observation facts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings after remediation | - |

## Failure-domain results

- Required relationships use exact kind plus directed endpoints; no traversal, transitive
  inference, cyclic inference, trace correlation, or timestamp ordering is permitted.
- Every strict reader rejects malformed, unknown, duplicate, reordered, noncanonical,
  cross-wired, over-bound, or trailing input before returning a view.
- Progress, closure, completeness, availability, and lateness remain independent closed
  vocabularies and are exhaustively tested over their 48-state cross-product.
- Corrections require a greater revision and exact direct predecessor while prior bytes
  remain immutable.
