---
id: SR-042
title: "Evidence-method review of the C00 observation specification"
type: SpecReview
analysis: evidence
scope: "FR-007..FR-011; NFR-003; TC-007..TC-013; TM-001"
review_set: all
---
# SR-042: Evidence-method review of the C00 observation specification

## Summary

Native Quoin advice over the repository's 88 obligations reports 18 method
recommendation mismatches, zero uncatalogued methods and zero inconclusive
obligations after method-name normalization. The mismatches were reviewed by
judgment against the observable property; they do not identify missing evidence
and are retained for the reasons below.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-048 | medium | Resolved: the no-unbounded-expansion obligation had only generic Analysis; it now names Static Analysis TC-013, whose inventory is paired with TC-012 exact/one-over properties. | NFR-003-AC-1; TC-012; TC-013 | correct-requirement-no-evidence |

## Method Decisions

- TC-007 through TC-012 remain `Property` where the oracle quantifies over
  values, interval relations, permutations, graphs, lineages or populations.
- TC-013 is `Static` because failure is the existence of an expansion/allocation
  path without a dominating checked bound.
- IT-002 remains `Integration` because it can fail only across real strict
  readers and pinned QSL/TL/Protocol/composed-runtime consumers.
- The advisor's example/unit recommendations do not replace property tests for
  invariants over values, interval relations, permutations, graphs, lineages or
  populations. Fixed examples remain regression cases inside those suites.
- The advisor classifies each quantified NFR-003 metric as performance
  benchmarking. These metrics constrain exact retained-work counts and semantic
  equality, not elapsed time or throughput, so property, metamorphic and unit
  evidence are the direct oracles.
- Example-based adverse cases supplement rather than replace generated
  invariants.

## Deterministic Evidence

Native `quoin 0.23.1-260-gf9aa659 advise --mismatch-only` and
`--inconclusive-only` report `18 mismatch`, `0 uncatalogued` and `0
inconclusive` across 88 obligations. Each mismatch falls into the stronger
generated/static method or exact-count metric decisions above; no
default-to-Test substitution was made.

## Verdict

PASS. Every C00 obligation has a confirmed method and planned producing suite.
