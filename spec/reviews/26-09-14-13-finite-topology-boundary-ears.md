---
id: SR-035
title: "EARS review of finite topology and anchored boundaries"
type: SpecReview
analysis: ears-conformance
scope: "spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md"
review_set: subset
---
# SR-035: EARS review of finite topology and anchored boundaries

## Summary

FR-006's event-driven admission statement and ubiquitous topology, limit and
boundary constraints were reviewed for explicit actor, condition, response and
modal force. The requirement is EARS-compatible and preserves refusal rather
than approximation at every unsupported boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-037 | low | No unresolved EARS finding. | FR-006 |

## Conformance results

- The primary statement uses `When <event>, the library SHALL <response>, or
  SHALL <refusal>` and names the acting system.
- Ubiquitous topology constraints state what the library SHALL retain and SHALL
  NOT infer; they do not disguise reachability or causality as ordering.
- Discrete, timestamped, point, watermark and cross-wire clauses name the
  triggering boundary condition and exact admission/refusal response.
- Resource clauses name the populations and require refusal before work; the
  no-traversal clause does not manufacture a path-depth limit.
- Contract-continuity clauses explicitly prohibit schema mutation and the
  superseded evaluator/result APIs.
- Acceptance criteria are observable boundary examples or a clearly labeled
  test-plus-review public-surface check.

## Evidence

Targeted `quire validate --scope . --summary` reports all seven changed
specification/plan documents grammar-clean with zero grammar findings.

## Verdict

PASS. FR-006 may enter its bounded Rust evidence cycle.
