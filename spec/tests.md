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
---
# quire-observation campaign test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1..4 | TC-001 | ✅ complete |
| FR-002 | FR-002-AC-1..3 | TC-002 | ✅ complete |
| FR-003 | FR-003-AC-1..3 | TC-003 | ✅ complete |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Qualify explicit observation admission | Unit | P0 | FR-001-AC-1..FR-001-AC-4 | ✅ complete |
| TC-002 | Agree bounded replay and incremental assessment | Property | P0 | FR-002-AC-1..FR-002-AC-3, NFR-001, NFR-002 | ✅ complete |
| TC-003 | Preserve result facts through consumer handoff | Property | P0 | FR-003-AC-1..FR-003-AC-3, NFR-002 | ✅ complete |
| IT-001 | Observation assessment handoff | Integration | P1 | FR-001-AC-1, FR-002-AC-1..FR-002-AC-3, FR-003-AC-1..FR-003-AC-3 | ✅ local typed-sink coverage |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Preserve qualified assessment facts across local stages | typed in-process consumer sink | service | IT-001 |
