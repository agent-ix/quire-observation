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
  - target: "ix://agent-ix/quire-observation/FR-007"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-008"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-009"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-010"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/FR-011"
    type: "covers"
  - target: "ix://agent-ix/quire-observation/NFR-003"
    type: "covers"
---
# quire-observation campaign test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1..7 | TC-001 | ✅ complete |
| FR-002 | FR-002-AC-1..4 | TC-002 | ✅ complete |
| FR-003 | FR-003-AC-1..3 | TC-003 | ✅ complete |
| FR-004 | FR-004-AC-1..6 | TC-004 | ✅ complete |
| FR-005 | FR-005-AC-1..9 | TC-005 | ✅ complete |
| FR-006 | FR-006-AC-1..8 | TC-006 | ✅ complete |
| FR-007 | FR-007-AC-1..5 | TC-007 | 🚧 planned |
| FR-008 | FR-008-AC-1..6 | TC-008 | 🚧 planned |
| FR-009 | FR-009-AC-1..5 | TC-009 | 🚧 planned |
| FR-010 | FR-010-AC-1..6 | TC-010 | 🚧 planned |
| FR-011 | FR-011-AC-1..6 | TC-011 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | Boundary tests | TC-001, TC-002, TC-004 | ✅ complete |
| NFR-002 | Determinism properties | TC-002, TC-003 | ✅ complete |
| NFR-003 | Property and static analysis | TC-010, TC-012 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Qualify explicit observation admission | Unit | P0 | FR-001-AC-1..FR-001-AC-7, NFR-001-AC-1, NFR-001-AC-2, NFR-001-AC-6 | ✅ complete |
| TC-002 | Agree bounded batch and incremental authority derivation | Property | P0 | FR-002-AC-1..FR-002-AC-4, NFR-001-AC-5, NFR-002-AC-1 | ✅ complete |
| TC-003 | Expose complete validated owner views | Property | P0 | FR-003-AC-1..FR-003-AC-3, NFR-002-AC-2 | ✅ complete |
| TC-004 | Publish and read observation authority artifacts | Integration | P0 | FR-004-AC-1..FR-004-AC-6, NFR-001-AC-3, NFR-001-AC-4 | ✅ complete |
| TC-005 | Admit authority-qualified runtime references | Integration | P0 | FR-005-AC-1..FR-005-AC-9 | ✅ complete |
| TC-006 | Prove finite topology retention and exact boundary anchoring | Integration | P0 | FR-006-AC-1..FR-006-AC-8 | ✅ complete |
| TC-007 | Preserve partial values and interval order | Property | P0 | FR-007-AC-1..FR-007-AC-5 | 🚧 planned |
| TC-008 | Preserve activation and scope-authority axes | Property | P0 | FR-008-AC-1..FR-008-AC-6 | 🚧 planned |
| TC-009 | Publish immutable revisioned authority bundles | Property | P0 | FR-009-AC-1..FR-009-AC-5 | 🚧 planned |
| TC-010 | Agree batch, incremental and repaired authority | Property | P0 | FR-010-AC-1..FR-010-AC-6, NFR-003 | 🚧 planned |
| TC-011 | Evaluate closed related-workflow populations | Property | P0 | FR-011-AC-1..FR-011-AC-6 | 🚧 planned |
| TC-012 | Bound and reproduce C00 observation work | Property | P0 | NFR-003-AC-1..NFR-003-AC-2, NFR-003 | 🚧 planned |

## Option Permutation Matrix

| Test Case | Dimension A | Dimension B | Dimension C | Expected Behavior |
|---|---|---|---|---|
| TC-007 | value set: true/false/both | reason: known/missing/conflicting | interval: point/disjoint/overlap | admit only coherent combinations and preserve every admissible order |
| TC-008 | activation: inactive/active/unknown | progress: covered/overlap/missing-source | completeness: complete/incomplete/contradicted | preserve axes independently without promotion |
| TC-009 | revision: initial/later | replay: exact/conflicting | lineage: valid/missing/foreign/branch/self | emit one canonical bundle or one typed refusal |
| TC-010 | path: batch/incremental/repair | prefix: decisive/unresolved/closed | order: total/overlap/arrival-permuted | agree on settled outputs and preserve uncertainty |
| TC-011 | population: snapshot/window | state: closed/open/unknown/contradicted | query: filter/count/sum | emit a value only for complete admitted authority |

## Constraint Boundary Tests

| Constraint | Boundary Type | Test Value | Test Case | Expected |
|---|---|---|---|---|
| possibility-set cardinality | minimum | one Boolean | TC-007 | admitted singleton |
| possibility-set cardinality | below minimum | empty set | TC-007 | typed refusal |
| event-time interval | equality boundary | earliest = latest | TC-007 | admitted point interval |
| event-time interval | reversed | earliest > latest | TC-007 | typed refusal |
| every C00 collection limit | maximum | exact effective limit | TC-012 | admitted |
| every C00 collection limit | above maximum | effective limit + 1 | TC-012 | refusal before partial state |
| exact sum domain | prefix boundary | exact minimum/maximum | TC-011 | admitted exact value |
| exact sum domain | outside prefix | one step outside | TC-011 | typed refusal without reorder |

## Edge Cases

| ID | Description | Related Req | Test Case | Risk if Untested |
|---|---|---|---|---|
| EC-001 | Equal endpoint intervals and overlapping clocks | FR-007 | TC-007 | accidental total order or midpoint selection |
| EC-002 | Progress frontier overlaps an inclusive deadline | FR-008 | TC-008 | silence fabricates settlement |
| EC-003 | Revision graph branches, cycles, or reuses a key | FR-009, FR-010 | TC-009, TC-010 | history overwrite or unbounded repair |
| EC-004 | A decisive prefix arrives before unrelated trailing input | FR-010 | TC-010 | end-buffering masquerades as incremental monitoring |
| EC-005 | Window member lies exactly at start or end | FR-011 | TC-011 | boundary omission or double counting |
| EC-006 | Duplicate receipt and duplicate business effect coexist | FR-011 | TC-011 | transport replay changes semantic cardinality |

## Integration Test Matrix

| Test ID | Purpose | Target | Type |
|---|---|---|---|
| IT-001 | Preserve qualified observation authority across local stages | constructor-private validated Rust views | Integration |
| IT-002 | Preserve uncertainty, repair lineage, and aggregate accounting across pinned consumers | QSL/TL, Protocol, and composed-runtime Rust APIs | Integration |
