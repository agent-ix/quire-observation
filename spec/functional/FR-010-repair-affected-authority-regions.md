---
id: FR-010
title: "Repair affected authority regions with replay and incremental parity"
type: FR
relationships:
  - target: ix://agent-ix/quire-observation/US-001
    type: implements
  - target: ix://agent-ix/quire-observation/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-114
    type: references
  - target: ix://agent-ix/quire-specification/FR-160
    type: references
  - target: ix://agent-ix/quire-specification/FR-324
    type: references
---
# FR-010: Repair affected authority regions with replay and incremental parity

## Description

When an observation-authority revision replaces an admitted fact, the library SHALL
derive the bounded transitive closure of explicitly declared dependents.
When that dependency closure is valid, the library SHALL coordinate exact replay
or incremental recomputation for only that region.

## Inputs

- A validated prior and replacement FR-009 bundle pair.
- A finite explicit dependency graph from authority fact identities to temporal,
  protocol, aggregate, and composed-result identities.
- Exact evaluation windows, immutable package/profile selections, prior result
  bytes, evaluator contributions, and work/state limits.

## Outputs

- A deterministic repair plan containing changed facts, affected result
  identities, retained unaffected identities, and canonical recomputation order.
- Retained prior and replacement result records with their evidence digests and
  explicit supersession links after the selected evaluator returns.
- A typed incomplete, unsupported, refused, or exhausted outcome without a
  falsely repaired result.

## Behavior

- The library SHALL seed invalidation only from replacements declared by the
  validated authority revision.
- The library SHALL traverse only explicit dependency edges.
- The library SHALL bound every visited node and edge before retaining a repair
  plan.
- The library SHALL include a result in the affected region only when it is in
  that dependency closure and its declared scope/window can observe a replaced
  fact.
- The library SHALL retain byte-identical identities and result bytes for every
  unaffected result.
- The library SHALL order recomputation canonically by scope, window, and result
  identity for scheduling only; this order SHALL NOT establish event causality.
- The library SHALL pass exact event-time intervals and every admissible interval
  order to the selected evaluator without using ingestion order.
- The incremental coordinator SHALL expose a settled prefix as soon as a selected
  evaluator contribution carries a decisive witness or counterexample, or a
  matching authority proves the required boundary closed.
- The incremental coordinator SHALL NOT require end-of-input before exposing a
  genuinely settled prefix.
- For identical authority revisions and evaluator contributions, the batch replay
  and incremental paths SHALL emit identical settled dispositions, decision
  support, identities, and supersession lineage.
- The library SHALL preserve incomplete, pending, indeterminate, unsupported,
  refused, failed, and exhausted outcomes without promoting one to a settled
  Boolean result.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-010-AC-1 | A replacement invalidates every and only explicit dependent results whose scopes/windows can observe it. | Property Test (TC-010) |
| FR-010-AC-2 | Unaffected results retain byte-identical identities and bytes across repair. | Property Test (TC-010) |
| FR-010-AC-3 | Batch replay and incremental repair agree exactly on settled result axes, decision-support identities, and old/new lineage for identical inputs. | Property Test (TC-010) |
| FR-010-AC-4 | A decisive prefix is emitted before end-of-input, while an unresolved prefix remains non-settled until a decisive witness, decisive counterexample, or matching closure exists. | Test (TC-010) |
| FR-010-AC-5 | Out-of-order arrival and overlapping event-time intervals never become a total semantic order; every admissible order reaches the evaluator. | Property Test (TC-010) |
| FR-010-AC-6 | Cycles, unknown dependency identities, foreign revisions, and one-over work/state bounds return their typed outcomes without a partial repair plan or promoted result. | Test (TC-010) |

## Dependencies

- [FR-009](./FR-009-publish-revisioned-authority-bundles.md) supplies validated
  revision lineage and replacements.
- QSpec FR-160 and I06 own temporal evaluation and result meaning; this
  requirement coordinates exact inputs, impact, and lineage without interpreting
  a formula or protocol.
