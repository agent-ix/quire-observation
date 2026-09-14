---
id: TM-001
title: "quire-observation campaign test matrix"
type: TestMatrix
relationships:
  - target: "ix://agent-ix/quire-observation/FR-001"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-004"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-005"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-006"
    type: "covers"
---
# quire-observation campaign test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1..7 | TC-001 | ✅ complete |
| FR-002 | FR-002-AC-1..4 | TC-002 | ✅ complete |
| FR-003 | FR-003-AC-1..3 | TC-003 | ✅ complete |
| FR-004 | FR-004-AC-1..6 | TC-004 | ✅ complete |
| FR-005 | FR-005-AC-1..9 | TC-005 | ✅ complete |
| FR-006 | FR-006-AC-1..8 | TC-006 | ✅ complete |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Qualify explicit observation admission | Unit | P0 | FR-001-AC-1..FR-001-AC-7, NFR-001-AC-1, NFR-001-AC-2, NFR-001-AC-6 | ✅ complete |
| TC-002 | Agree bounded batch and incremental authority derivation | Property | P0 | FR-002-AC-1..FR-002-AC-4, NFR-001-AC-5, NFR-002-AC-1 | ✅ complete |
| TC-003 | Expose complete validated owner views | Property | P0 | FR-003-AC-1..FR-003-AC-3, NFR-002-AC-2 | ✅ complete |
| TC-004 | Publish and read observation authority artifacts | Integration | P0 | FR-004-AC-1..FR-004-AC-6, NFR-001-AC-3, NFR-001-AC-4 | ✅ complete |
| TC-005 | Admit authority-qualified runtime references | Integration | P0 | FR-005-AC-1..FR-005-AC-9 | ✅ complete |
| TC-006 | Prove finite topology retention and exact boundary anchoring | Integration | P0 | FR-006-AC-1..FR-006-AC-8 | ✅ complete |

## Integration Test Matrix

| Test ID | Purpose | Target | Type |
|---|---|---|---|
| IT-001 | Preserve qualified observation authority across local stages | constructor-private validated Rust views | Integration |
