---
id: FR-003
title: "Preserve qualified result facts at consumer handoff"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
---
# FR-003: Preserve qualified result facts at consumer handoff

## Description

When handing observation authority to a consumer, the library SHALL expose only
FR-004 validated owner views for position, clock, capture, admitted observation,
population, progress, closure, completeness and result availability, or SHALL
return a typed refusal without partial authority.

## Inputs

- Qualified admission and derived authority artifacts from FR-001 and FR-002.
- Exact consumer-selected contract/schema revisions and caller-lowered limits.

## Outputs

- Constructor-private validated observation-authority views preserving every
  independently owned fact.
- An explicit typed refusal when a selected contract cannot represent the exact
  artifact; no lossy substitute is emitted.

## Behavior

- The library SHALL preserve each owner artifact under its own contract,
  identity, revision, digest, scope and correction lineage.
- The library SHALL NOT convert availability, completeness, progress or closure
  into truth, settlement, adequacy, conformance or Boolean values.
- Decision-scope and surrounding-execution assertions SHALL remain separate
  artifacts; progress and closure SHALL remain independent `open`/`closed`
  facts, and completeness SHALL remain a separate `complete`/`incomplete`/
  `contradicted` fact.
- A late correction SHALL preserve predecessor bytes and name the exact direct
  predecessor; consumers choose whether a result must be recomputed.

## Error Conditions

Missing, duplicate, scope-cross-wired, unknown-version, noncanonical or
unsupported fields SHALL refuse the handoff; none may be discarded silently.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | Every owner artifact remains independently readable and no handoff exposes a truth, settlement or Boolean field. | Test (TC-003) |
| FR-003-AC-2 | A handoff preserves all required owner selections and rejects omission, duplication or scope cross-wiring. | Test (TC-003) |
| FR-003-AC-3 | Unsupported consumer representation returns a typed refusal and cannot promote loss to availability, completeness or success. | Test (TC-003) |

## Dependencies

- [FR-002](FR-002-replay-and-incremental-assessment.md) supplies the typed
  assessment disposition.
- Downstream temporal, protocol, and verification consumers select their own
  mappings; this library does not implement those consumers or their results.
