---
id: SR-043
title: "Risk and complexity review of C00 observation authority"
type: SpecReview
analysis: risk-complexity
scope: "US-001; FR-001, FR-007..FR-011; NFR-003"
review_set: all
---
# SR-043: Risk and complexity review of C00 observation authority

## Summary

Revision lineage, bounded cyclic dependency repair and exact population
aggregation are the principal C00 hazards. All high-risk items have explicit
property, boundary, static and integration mitigations and low policy volatility.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-049 | high | Mitigated: immutable lineage plus caller-supplied head state can race without external atomic compare-and-swap; FR-009 now limits the library guarantee and names the external requirement. | FR-009; TC-009 |
| FND-050 | high | Mitigated: cyclic affected-region repair and interval-order uncertainty can expand combinatorially; NFR-003, TC-012 and TC-013 enforce finite checked work. | FR-010; NFR-003; TC-012; TC-013 |
| FND-051 | high | Mitigated: duplicate/receipt confusion or arithmetic reorder can change financial aggregates; FR-011 closes duplicate policies, excludes receipts and rejects invalid prefixes. | FR-011; TC-011 |

## Risk Register

| Req | Tech Risk | Volatility | Drivers | Mitigation |
|---|---|---|---|---|
| FR-001 | Medium | Low | Multi-domain identity admission and strict bounds | Retain established TC-001 mutation/boundary evidence |
| FR-007 | Medium | Low | Exact interval partial order and multi-valued facts | Generated interval/value properties in TC-007 |
| FR-008 | High | Low | Independent activation/progress/closure/completeness axes | Cartesian state properties and cross-wire cases in TC-008 |
| FR-009 | High | Low | Immutable revision identity, replay and concurrent publication boundary | Lineage-view properties, external CAS boundary and TC-009 |
| FR-010 | High | Low | Cyclic graph traversal, incremental settlement and cross-owner callbacks | Reference closure oracle, parity properties, failure injection and static bounds |
| FR-011 | High | Low | Exact financial-style aggregation, duplicate identity and population closure | Closed policy vocabulary, checked prefix arithmetic and TC-011 |
| NFR-003 | Medium | Low | Global determinism and resource containment | TC-012 generated bounds plus TC-013 static inventory |

## Top Hazards

1. FR-009 concurrent successor publication outside the pure library boundary.
2. FR-010 affected-region soundness under cycles and uncertain event order.
3. FR-011 effect identity, occurrence policy and exact-prefix arithmetic.
4. FR-008 false settlement from overlapping progress/deadline intervals.

## Failure-Domain Gaps

See SR-039. All three identified gaps are resolved in the reviewed revision.

## Verdict

PASS. High-risk work is ordered early and paired with discriminating evidence.
