# Spec Writing & Review Improvement Report

## Overview

This report records gaps found while reviewing `quire-observation` against its
OB01–03 specification and its executable evidence. It is a review artifact,
not an accepted design change: the result-model and scope contracts need owner
decisions before code or requirements are changed.

## Remediation Status

All four design gaps and the traceability gap recorded below were remediated in
the repository and verified by SR-003: replay now performs a complete bounded
scan for late evidence; admission retains and binds its premise envelope;
handoff enumerates every consumer loss axis; and the matrix is 18/18 backed.
The formal Quoin plan-based gap-analysis verdict remains unavailable until a
governed plan bundle exists. The requested review-to-plan capability is tracked
in agent-ix/quoin#365.

## Coverage Summary

| Category | Total | Covered | Gaps | Coverage |
| --- | ---: | ---: | ---: | ---: |
| Stakeholder Requirements (StR) | 0 | 0 | 0 | n/a |
| Functional Requirements (FR) | 3 | 0 | 3 | 0% |
| Non-Functional Requirements (NFR) | 2 | 2 | 0 | 100% |
| Constraints (C) | 0 | 0 | 0 | n/a |
| Acceptance Criteria (AC) | 10 | 0 | 10 | 0% |

The counts use `quire coverage --json`: it found 2 backed rows of 14 total
matrix rows. The two backed rows are TC-002 and TC-003; this does not establish
their individual acceptance-criterion coverage.

### Gap Inventory

**StR Gaps**: None authored; ownership is currently only by the OB01–03 local
campaign documents.

**FR Gaps**: FR-001 does not specify the exact `Available` envelope needed to
retain selections; FR-002 does not state whether replay must inspect all records
after a decisive record; FR-003 does not enumerate which consumer capabilities
are required for every result axis.

**NFR Gaps**: The NFR metric rows exist, but the documents lack the
`Acceptance Criteria` section expected by the active trace configuration.

**Constraint Gaps**: No explicit constraint artifacts exist.

**AC Gaps**: FR-001-AC-1 through FR-003-AC-3 are all unbound. TC-001 and
IT-001 report complete coverage without recognized evidence.

## Analysis of Gaps

### GAP-001: Post-settlement late-record completeness

**The Gap:** `replay` stops on the first decisive observation. A later record
whose ingestion time is beyond the late cutoff is never collected, so a late
contradiction neither appears in `late_records` nor links to the prior result.

**Root Cause:** FR-002 requires late-record retention but does not state the
full-scan invariant or provide a test where a decisive timely record precedes a
late contradictory record.

**Skill Improvement:**

- **Technique**: Post-decision input sweep.
  - *Description*: For every evaluator that can settle early, test records both
    before and after the decisive input, including late conflicting inputs.
- **Checklist Item**: Does every input that can affect audit, late-data, or
  supersession metadata get examined after a decision is known?

### GAP-002: Admission retention and membership binding

**The Gap:** `Available` exposes records only, and admission does not bind a
member selection to an admitted record. The caller can supply unrelated members
and still obtain an available outcome without the selected scope envelope.

**Root Cause:** FR-001 says selections are retained, but the output shape and
member-to-record invariant are not specified as testable fields.

**Skill Improvement:**

- **Technique**: Input-to-output retention map.
  - *Description*: Enumerate every selected input and state whether it is
    returned, internally bound, or intentionally excluded with a typed reason.
- **Checklist Item**: Can a caller inspect the accepted outcome and prove that
  each population member and every immutable selection constrained the result?

### GAP-003: Consumer preservation is under-modeled

**The Gap:** `ConsumerCapabilities` can represent loss for five fields only,
yet `AssessmentHandoff` has more independently readable axes. The code can
report `Preserved` when the downstream target cannot represent scope facts,
global closure, provenance, support, settlement, or truth.

**Root Cause:** FR-003 names all axes but does not define a capability matrix or
a refusal/loss rule for each axis.

**Skill Improvement:**

- **Technique**: Axis-by-axis loss matrix.
  - *Description*: Derive one consumer capability and one omission test per
    independently readable result fact.
- **Checklist Item**: For every result field, what exact consumer capability
  proves it is preserved, and what typed loss occurs when it is absent?

### GAP-004: Traceability reports completion without bound evidence

**The Gap:** Matrix status claims TC-001 and IT-001 are complete, while the
active trace binder recognizes no evidence for them; all functional acceptance
criteria are unbacked.

**Root Cause:** Test names use local TC-140/141/142 identifiers rather than
the minted TC-001 trace identifier, and the coverage table uses `Coverage
Status` while the active trace configuration expects `Status`.

**Skill Improvement:**

- **Technique**: Binding-before-complete gate.
  - *Description*: Run `quire coverage --json` and reject a complete status for
    any unbacked row before finalizing a matrix.
- **Checklist Item**: Does every complete matrix row resolve to at least one
  recognized test symbol and each cited acceptance criterion?

## Required Next Decisions

1. Define the immutable admission result envelope and member-to-record binding.
2. Define whether replay must scan the entire bounded history after decisive
   evidence, then specify supersession semantics for a late contradiction.
3. Define a complete result-axis capability/loss matrix for handoff.
4. Choose a governed plan bundle before requesting Quoin's plan-based
   `gap-analysis` verdict.
