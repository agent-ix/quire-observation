---
id: FR-003
title: "Expose validated observation authority to consumers"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "depends_on"
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "depends_on"
---
# FR-003: Expose validated observation authority to consumers

## Description

When a consumer reads observation authority, the library SHALL expose only
FR-004 constructor-private validated views for position, clock, capture,
admitted observation, population, progress, closure, completeness and result
availability, or SHALL return a typed refusal without partial authority.

## Inputs

- Qualified admission and derived authority artifacts from FR-001 and FR-002.
- The exact owner-contract reader, independent expected selections, and
  caller-lowered limits.

## Outputs

- Constructor-private validated observation-authority views with public
  read-only access to every independently owned payload fact.
- An explicit typed refusal when a selected contract cannot represent the exact
  artifact; no lossy substitute is emitted.

## Behavior

- The library SHALL preserve each owner artifact under its own contract,
  identity, revision, scope and correction lineage, including every selected
  raw-definition digest carried by that contract.
- The library SHALL NOT convert availability, completeness, progress or closure
  into truth, settlement, adequacy, conformance or Boolean values.
- Progress and closure SHALL remain separate `open`/`closed` facts, and
  completeness SHALL remain a separate `complete`/`incomplete`/`contradicted`
  fact.
- A correction SHALL preserve predecessor bytes and name the exact direct
  predecessor; consumers choose whether any downstream result must be
  recomputed.

## Error Conditions

Missing, duplicate, scope-cross-wired, unknown-version, noncanonical or
unsupported fields SHALL refuse the handoff; none may be discarded silently.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | Every owner artifact remains independently readable through complete read-only payload accessors and no view exposes a truth, settlement or Boolean field. | Test (TC-003) |
| FR-003-AC-2 | A strict read preserves all required owner selections and rejects omission, duplication or scope cross-wiring. | Test (TC-003) |
| FR-003-AC-3 | An unsupported document or owner-contract representation returns a typed refusal and cannot promote loss to availability, completeness or success. | Test (TC-003) |

## Dependencies

- [FR-002](FR-002-replay-and-incremental-assessment.md) supplies deterministic
  batch and incremental owner-artifact derivation.
- Downstream temporal, protocol, and verification consumers select their own
  mappings; this library does not implement those consumers or their results.
