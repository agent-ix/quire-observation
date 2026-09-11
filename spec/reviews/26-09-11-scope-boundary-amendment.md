---
id: SR-008
title: "Scope-boundary review of the corrected-result-model amendment"
type: SpecReview
analysis: scope-boundary
scope: "spec/functional/FR-001-qualify-observation-admission.md, spec/functional/FR-002-replay-and-incremental-assessment.md, spec/functional/FR-003-preserve-observation-result-handoff.md, spec/spec.md (In Scope / Out of Scope, References); commit 18e8f67"
review_set: subset
---
## Summary

This analysis examined the boundary claims added by commit 18e8f67 — FR-001's
caller-opaque member identity and retained-not-recomputed digests, FR-002's
ambiguous-order outcome grounded on caller-declared ordering authority, and
FR-003's seventeen-axis consumer capability selection — against the master
boundary in `spec/spec.md`. All three claims sit inside the declared boundary
and none of them imports native grammar, parsing, or protocol conformance work.
Two allocation defects remain: FR-001 declines canonicalization without naming
the component that owns it, and FR-003 conditions an in-scope obligation on a
consumer-internal property this library cannot observe.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-001 declines membership/closure canonicalization ("a canonicalization this library does not own") without allocating it: no owning component, no named contract, and no entry in the Dependencies section, so the responsibility is unallocated rather than delegated. The owner should be named as `quire-spec-language`, which owns semantic package compilation and exports the `native-linked-package/1` artifact with its canonical digest; `quire-protocol`'s RFC 8785 canonicalization governs protocol result identity, not population membership or closure, and is the wrong owner. | FR-001 Behavior, FR-001 Dependencies; spec.md:37-39, spec.md References; agent-ix/quire-spec-language FR-015, FR-027; agent-ix/quire-protocol spec.md:48, FR-006 |
| FND-002 | low | The retained-not-verified digest boundary is stated only inside FR-001. spec.md In Scope still reads "Digest-bound package and producer selections" with no statement that the selected digests are trusted unverified, and Out of Scope does not list digest canonicalization, so the master boundary and the requirement disagree on where the boundary runs. | spec.md:28-45; FR-001 Behavior |
| FND-003 | medium | FR-003's "A single capability SHALL NOT stand for two facts a consumer can represent independently" makes an in-scope obligation conditional on the consumer's internal representational ability, which this library cannot observe, test, or verify — downstream consumers are explicitly out of scope. Restate it as a constraint on the capability record this library accepts: one independently settable capability and one typed loss per declared axis. | FR-003 Behavior; FR-003 Dependencies; spec.md:44 |
| FND-004 | medium | The seventeen-axis list itself stays inside the boundary — every axis is a fact the library's own result already carries — but the new `retained events` axis is not allocated at the input boundary: FR-003 Inputs still names only the typed disposition, its immutable selection dependencies, and the consumer mapping, so the axis enters the handoff from FR-002's replay result with no declared input. | FR-003 Description, Inputs, Behavior; FR-002 Outputs |
| FND-005 | low | `retained events` is NFR-001 resource accounting (infrastructure class) promoted to a core consumer-preservation axis. FR-003 states the obligation but not why the count is semantically load-bearing at the handoff; TC-002 carries the intent ("an unavailable assessment reports no retained event") while FR-003 does not. | FR-003 Behavior; NFR-001 Statement, NFR-001-AC-3; TC-002 Expected Results |
| FND-006 | medium | The ambiguous-order outcome is correctly allocated to this library — the caller declares the sequence, so a duplicated position is an admitted-input defect, not a profile gap — but it is authored only as rationale under Error Conditions. FR-002 Behavior states no ordering-authority obligation, Outputs names no such outcome, and no acceptance criterion covers it distinctly (FR-002-AC-3's "ambiguous" is ambiguous membership), while TC-002 now claims ambiguous-order coverage. The library's current outcome for a duplicated position is the `Unsupported` disposition the amendment forbids. | FR-002 Behavior, Outputs, Error Conditions, FR-002-AC-3; TC-002; src/replay.rs:201-206 |

## System Boundary After the Amendment

The amendment narrows rather than widens the boundary. It adds no native
grammar, parser, temporal engine, or protocol conformance responsibility, so it
is consistent with spec.md:28-45 on all three documents.

- FR-001 moves member object identity and membership/closure digest derivation
  outside the boundary and keeps only retention of the caller's selection.
- FR-002 keeps record ordering outside the boundary by making the caller the
  sole ordering authority, and keeps the detection of a contradictory
  declaration inside it.
- FR-003 keeps consumer implementation outside the boundary and adds only the
  shape of the capability declaration this library accepts.

## External Dependencies

| Dependency | Type | Assumed or Guaranteed | Contract |
|---|---|---|---|
| `native-linked-package/1` selection and its membership/closure canonicalization | caller-supplied artifact | Assumed | Unnamed — FND-001 |
| Producer interface 1.2.0 identity, digest, model, configuration | caller-supplied selection | Assumed | Producer interface 1.2.0, selected by the shared Quire specification |
| Caller-declared record sequence and progress authority | caller-supplied ordering | Assumed | FR-002 Inputs; contradiction detected, order not derived |
| Temporal, protocol, and verification consumers | downstream adapter | Assumed | FR-003 typed consumer mapping; capability record only |

## Responsibility Allocation

| Requirement | Owning Component | Class |
|---|---|---|
| FR-001 (admission, digest retention) | quire-observation admission | core |
| FR-002 (replay, incremental, ambiguous order) | quire-observation replay | core |
| FR-003 (result handoff, capability axes) | quire-observation handoff | core |
| NFR-001 (retained state bounds) | quire-observation admission and replay | infrastructure |
| NFR-002 (reproducible outcomes) | quire-observation replay and handoff | cross-cutting |
| Membership/closure canonicalization | quire-spec-language (proposed, FND-001) | core, external |
