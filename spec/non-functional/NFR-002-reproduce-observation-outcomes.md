---
id: NFR-002
title: "Reproduce qualified observation outcomes"
type: NFR
quality_attribute: reliability
relationships:
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "constrains"
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "constrains"
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "constrains"
---
# NFR-002: Reproduce qualified observation outcomes

## Statement

The library SHALL produce byte-identical owner artifacts and identities for
repeated derivation of the same qualified history, selections and limits.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Repeated owner-artifact byte equality | 100% of fixed fixtures | 100% of fixed fixtures | Test (TC-002) |
| Validated-view authority/subject retention | 100% of strict reads | 100% of strict reads | Test (TC-003) |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-002-AC-1 | Repeated batch and incremental derivation of the same ordered input returns byte-identical documents and identities. | Test (TC-002) |
| NFR-002-AC-2 | Every validated view retains the exact authority, subject, revision and predecessor selections from its admitted document. | Test (TC-003) |

## Verification

The tests derive every owner contract through batch and incremental history and
compare complete bytes. Strict-reader tests inspect retained authority, subject,
revision and correction lineage.
