---
type: master-requirements
name: quire-observation
org: agent-ix
component_type: rust-library
implementation_language: rust
tags:
  - quire
  - observation
  - OB01
  - OB02
  - OB03
  - C00
depends_on: []
standards_alignment:
  - iso-iec-ieee-29148
relationships: []
security_critical: false
---
# Master Requirements Specification

## Purpose

This specification defines the private Rust observation boundary required for
the complete Quire V1 runtime. It qualifies incoming records, represents exact
partial and clock-uncertain facts, derives revisioned observation authority, and
evaluates only the finite population queries owned by the observation boundary.
Temporal and protocol meaning remain with their domain evaluators.

## Scope

### In Scope

- Digest-bound package and producer selections, typed observation admission,
  declared relationship correlation, and finite scope completeness.
- Deterministic batch and incremental owner-artifact derivation under explicit
  progress, closure, lateness, and clock-family inputs.
- Exact partial-value, clock-interval, activation-capture, and independent
  progress/completeness semantics.
- Explicit revision lineage, bounded dependency-closure invalidation, and
  repair plans that preserve unaffected authority and settled result identities.
- Count, filter, and exact-sum queries over authority-qualified closed snapshots
  or event-time windows for reservation, refund, and related-workflow subjects.
- Immutable strict-read views for temporal, protocol, and verification consumers.

### Out of Scope

- Native source parsing and semantic package compilation, which belong to
  `quire-spec-language`.
- FCD decoding, OTLP/other transport adapters, production monitoring, and
  business-action execution.
- Parsing or interpreting producer and closure definition bytes. Their
  raw-artifact digests are retained in the declared domain. This library does
  own and rederive canonical identities for its observation, explicit-members,
  population, position, clock, capture, progress, closure, completeness and
  availability authority documents.
- Temporal formula interpretation, protocol conformance interpretation,
  composed execution, external effects, and verification-result storage.
- Ambient population discovery, inferred workflow relationships, midpoint time,
  arrival-order semantics, or partial definitive aggregates.

## System Overview

### System Description

`quire-observation` is a transport-independent Rust library. It accepts
explicitly selected semantic records and scopes, rejects unsafe substitutions,
and emits qualified inputs or explicit non-success outcomes for downstream
assessment.

### Intended Users

The native temporal evaluator, protocol conformance implementation, and result
consumer adapters use this library. They rely on it to preserve provenance and
to distinguish absent evidence from a negative value.

## Requirements Architecture

The user story [US-001](usecase/US-001-qualify-observations.md) drives admission
(OB01), deterministic batch/incremental authority derivation (OB02), validated
consumer access (OB03), and the canonical owner-artifact boundary in
[FR-004](functional/FR-004-publish-observation-authority-artifacts.md). FR-007
through FR-009 extend that boundary with exact uncertainty, independent scope
facts, and the I07 revisioned bundle. FR-010 owns repair-impact and incremental
coordination without interpreting domain formulas. FR-011 owns finite closed-
population query evaluation. The non-functional requirements constrain retained
state, reproducibility, and bounded repair. Integration coverage connects the
stages to pinned consumers.

## Ownership correction for the temporal ecosystem

This library owns observation identity, membership/population, position, clock,
capture, progress, closure, completeness and result-availability facts. It does
not own temporal or protocol truth, settlement, claim, conformance, or Boolean
result vocabularies. FR-002 owns observation-authority derivation only; FR-003
exposes those artifacts through strict owner readers and never emits a canonical
protocol result. `quire-protocol` and `tl-mltl` own their results, and
`quire-contract-ir` joins only validated owner views.

The complete-V1 extension adopts QSpec AD-002 and AD-008. I07 is the authority
boundary implemented here. I06 temporal and I09 protocol evaluators consume I07
and retain their own result semantics. I10 composition belongs to
`quire-integration`; this crate supplies versioned inputs and repair lineage to
that facade without becoming the facade.

## References

- `agent-ix/quire-research#44`, `#45`, and `#46` under OBSERVATION #37.
- Accepted `quire-specification` FR-090 through FR-095, FR-110 through FR-116,
  FR-153, FR-160, FR-180, AD-002, AD-008, I06, I07, I10, TC-198, TC-199,
  TC-209, and IT-072 at reviewed revision `8d0fbad`.
- Producer interface 1.2.0 selected by the shared Quire specification.
