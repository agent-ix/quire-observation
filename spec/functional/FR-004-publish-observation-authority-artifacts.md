---
id: FR-004
title: "Publish strict observation authority artifacts"
type: FR
relationships:
  - { target: ix://agent-ix/quire-observation/FR-001, type: depends_on }
  - { target: ix://agent-ix/quire-observation/FR-002, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-260, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-263, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-267, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-268, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-287, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-294, type: depends_on }
  - { target: ix://agent-ix/tl-syntax/IF-002, type: implements }
  - { target: ix://agent-ix/tl-syntax/VO-004, type: implements }
---
# FR-004: Publish strict observation authority artifacts

## Description

When qualified immutable observation state is exported, the library SHALL emit
canonical bounded owner documents and expose only constructor-private validated
views through strict public readers.

## Subsystem and contract set

The implementation SHALL be organized as
`authority::{observation,population,position,clock,capture,progress,closure,completeness,availability}`.
Each module SHALL publish `CONTRACT`, `SCHEMA_BYTES`, `SCHEMA_SHA256`, `Limits`,
an owner derivation function and `read(bytes, expected, limits)`. The closed v1
contracts are:

| Module | Contract | Semantic payload |
| --- | --- | --- |
| observation | `quire.observation.record/v1` | FR-260/FR-288 admitted observation and correction preimage |
| population | `quire.observation.population/v1` | FR-263 membership, snapshot/window and population identities |
| position | `quire.observation.position-ledger/v1` | ordered distinct zero-based positions and observation identities |
| clock | `quire.observation.clock-binding/v1` | event-position, fixed-sample or timestamped-event selection and exact parameters |
| capture | `quire.observation.capture-environment/v1` | immutable trigger/anchor and complete sorted typed value bindings |
| progress | `quire.observation.progress-assertion/v1` | FR-268 authority, scope, clock, source set, boundary and `open`/`closed` state |
| closure | `quire.observation.closure-assertion/v1` | scope, closure authority, boundary, source set and `open`/`closed` state |
| completeness | `quire.observation.completeness-assertion/v1` | exact population/facts and FR-287 state |
| availability | `quire.observation.result-availability/v1` | required result identities and availability state |

Availability admits exactly `available`, `not-yet-observed`,
`producer-unavailable`, and `contract-unavailable`. Progress and closure admit
only `open` and `closed`. Completeness admits only `complete`, `incomplete`, and
`contradicted`. These vocabularies are independent and never cross-coerced.

## Common envelope and identity

Every document contains exactly `contract`, `identity`, `revision`, `authority`,
`subject`, `payload`, `predecessor`, and `limits` in canonical order. Authority
identifies the producer definition/revision/raw-artifact digest. Subject retains
the exact scope/population identities. Payload is the contract-specific closed
record above. Predecessor is absent for an original and is the exact direct
predecessor identity for a correction. Limits records effective limits and use.

Identity is lowercase SHA-256 over the contract label, a zero byte, and the
canonical document bytes with `identity` omitted. Contract-specific identities
whose preimages are fixed by `quire-specification` SHALL use those exact
preimages instead. A correction has a greater positive revision, new identity
and immutable predecessor; it never rewrites prior bytes.

## Admission and limits

Derivation SHALL consume only FR-001 qualified state and FR-002 deterministic
authority derivations. Each reader SHALL revalidate its entire document against
independently supplied expected authority, subject, clock, scope, population,
boundary and predecessor selections. Readers reject unknown/duplicate/missing/
out-of-order fields, trailing data, invalid UTF-8, noncanonical bytes, unknown
contracts or enum labels, unsorted/duplicate populations, identity/digest/
revision mismatch and every cross-wired scope or clock.

Limits independently bound input/output bytes, JSON depth, string bytes,
population entries, position count, capture bindings, required sources and
visited fields. Work is charged before allocation or traversal. Exact-limit
inputs are admitted; one-over inputs return resource-incomplete with no partial
document/view. Caller limits may lower but not raise owner maxima.

Position order is authoritative only under its selected ledger/clock and never
inferred from timestamps. Clock uncertainty is a value under a selected clock,
not a replacement clock. Inclusive temporal `[a,b]` coverage uses the exact
FR-289 mapping. Late classification uses only FR-294 `on-time`/`late` and never
mutates prior results. Completeness derives from the complete required fact
population with contradiction precedence; availability does not imply truth.

No public constructor, trust/qualified flag, parser, evaluator, transport
adapter, callback, protocol-result vocabulary or Boolean coercion is admitted.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | Each of the nine exact contracts publishes immutable schema bytes/digest, deterministic canonical bytes/identity and a public strict reader returning only its constructor-private view. | Test (TC-004) |
| FR-004-AC-2 | Batch and incremental derivation produce byte-identical documents for equal qualified history and selections; every semantic mutation changes only the applicable identity. | Test (TC-004) |
| FR-004-AC-3 | Every required field is independently omitted, duplicated, reordered and cross-wired; unknown fields/labels/contracts, trailing/noncanonical bytes and same identity on unequal bytes refuse without partial authority. | Test (TC-004) |
| FR-004-AC-4 | Exact and one-over byte/depth/string/population/position/capture/source/work bounds admit or resource-refuse before excess retention. | Test (TC-004) |
| FR-004-AC-5 | All combinations of independent progress, closure, completeness and availability values remain representable; no combination yields a truth, settlement or Boolean value. | Test (TC-004) |
| FR-004-AC-6 | Corrections preserve predecessor bytes/direct lineage; late data, restoration and timestamp ordering cannot rewrite authority or manufacture origin/causality. | Test (TC-004) |

## Dependencies

FR-001 supplies qualified admission and FR-002 supplies deterministic authority
derivation. `quire-specification` FR-260 through FR-268 and FR-287 through
FR-294 own the shared identity, boundary and vocabulary meaning. Downstream
result and bridge crates consume only the public validated views.

## Status

Proposed complete owner boundary for `quire-observation#15` and
`tl-syntax#52`.
