---
id: IT-002
title: "Qualify revisioned observation repair and aggregate handoff"
type: IT
relationships:
  - target: ix://agent-ix/quire-observation/FR-007
    type: verifies
  - target: ix://agent-ix/quire-observation/FR-008
    type: verifies
  - target: ix://agent-ix/quire-observation/FR-009
    type: verifies
  - target: ix://agent-ix/quire-observation/FR-010
    type: verifies
  - target: ix://agent-ix/quire-observation/FR-011
    type: verifies
  - target: ix://agent-ix/quire-specification/IT-072
    type: references
---
# IT-002: Qualify revisioned observation repair and aggregate handoff

## Objective

Verify that real pinned QObs owner views drive the selected Rust temporal and
protocol consumers through initial assessment, revisioned repair, and closed-
population aggregation without losing uncertainty or lineage.

## Target Integration

The integration links the `quire-observation` Rust crate to exact pinned QSL/TL,
Protocol, and composed-runtime Rust consumer revisions through I06, I07, I09, and
I10. It invokes the real local Rust APIs and canonical wire readers. No semantic
consumer, file I/O, or clock behavior is mocked.

## Preconditions

- Exact dependency revisions and schema digests are recorded in one ecosystem
  lock and are locally buildable without network access.
- Version-locked timed-refund and related partial-refund fixtures are loaded from
  their real package and authority readers.
- Every consumer supports the selected contract versions or reports a typed
  unsupported result.

## Inputs

Partial and conflicting valuations; disjoint and overlapping event-time
intervals; silence progress; complete/open populations; two related refunds; a
foreign refund; an exact replay; a late replacement revision; and exact/one-over
limits.

## Test Procedure

1. Read the initial QObs bundle through the real I07 reader and run each selected
   consumer.
   - IT-002-SC-01: every consumer retains the same possibility, clock, population,
     progress, completeness, and authority identities.
2. Incrementally submit the decisive timed-refund prefix before end-of-input.
   - IT-002-SC-02: the consumer exposes the same settled result that full replay
     later produces, including exact support.
3. Submit a valid out-of-order authority revision and execute its repair plan.
   - IT-002-SC-03: only affected results are replaced; prior/replacement bytes and
     evidence digests remain explicitly linked.
4. Execute the closed related-refund query and its open, foreign, ambiguous, and
   one-over variants.
   - IT-002-SC-04: only the complete qualified population emits the exact
     aggregate; every adverse variant remains non-definitive or refused.
5. Compare incremental, repaired, and full replay outputs under the same lock.
   - IT-002-SC-05: settled axes, decision support, aggregate accounting, and
     lineage are byte-identical across paths.

## Expected Results

The pinned consumers agree on exact uncertainty, authority, repaired lineage,
and complete aggregate accounting. No path uses ingestion order, interval
midpoints, quiet time, missing-as-false, an ambient population, or a partial
definitive aggregate. Unsupported dependencies remain explicit and do not erase
independent completed siblings.

## Metadata

- Priority: P0
- Target Integration: QSpec IT-072 observation repair parity plus the observation
  cells of IT-071
- Automation: Automated local Rust integration test

## Dependencies

**Upstream**: merged compatible QSL/TL/Protocol consumer revisions and FR-007
through FR-011. **Downstream**: QObs #26 qualification and the composed
`quire-integration` C14 campaign.

## Traceability

This test implements the QObs-owned portion of QSpec IT-072 and supplies the I07
evidence required by the complete composed integration lock.
