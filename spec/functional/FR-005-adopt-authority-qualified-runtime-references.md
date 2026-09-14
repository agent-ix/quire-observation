---
id: FR-005
title: "Adopt authority-qualified runtime subjects and relationships"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/FR-001"
    type: "depends_on"
  - target: "ix://agent-ix/quire-specification/FR-262"
    type: "depends_on"
  - target: "ix://agent-ix/quire-specification/FR-287"
    type: "depends_on"
---
# FR-005: Adopt authority-qualified runtime subjects and relationships

## Description

When admitting a runtime subject or relationship, the library SHALL retain the
exact FCD Producer interface 1.2 authority, producer-local discriminator and
opaque runtime identities, or SHALL refuse the input with a typed cause.

## Producer authority subsystem

- The request SHALL consume an `AdmittedStaticBundle` from the pinned
  `agent-ix-baseline-producer` Rust boundary at revision `4042882`; it SHALL NOT
  accept a locally restated producer-selection record.
- The exact `AdmittedBundleKey` (`bundle_identity`, namespaced `bundle_revision`,
  and four-member `DigestSelection`) SHALL qualify every runtime subject and
  relationship. The digest algorithm, domain, domain version and value all
  participate in equality and ordering.
- The Producer interface version SHALL be the version carried by the admitted
  bundle and SHALL equal `1.2.0`. An unadmitted bundle or a different version is
  unrepresentable at this boundary rather than approximated.
- The library SHALL retain the admitted bundle capability so a relationship
  declaration is resolved only from that exact selected producer authority.

## Subject-reference subsystem

- A qualified subject SHALL contain exactly the producer authority key, a
  non-empty producer-local subject-kind discriminator, and non-empty
  producer-local object-identity bytes.
- The library SHALL preserve the kind and object bytes byte-for-byte. It SHALL
  NOT trim, parse, normalize, hash, case-fold, synthesize, or derive either value
  from a relationship, transport field, trace, provider, display string, source
  locus, timestamp, arrival position, or ambient configuration.
- Canonical key equality and lexicographic order SHALL be componentwise over
  authority, kind, and object bytes in that sequence. That order SHALL NOT imply
  causal, delivery, retry, timestamp, or arrival order.
- When semantic object comparison crosses an authority or kind boundary, the
  library SHALL return a typed `foreign authority` or `wrong kind` refusal
  rather than Boolean object equality.

## Relationship subsystem

- A runtime relationship SHALL retain its exact producer authority, exact
  producer-supplied runtime relationship identity, exact FCD relationship
  declaration identity, and authored source and target qualified subjects.
- The declaration SHALL exist in the retained admitted bundle. Its ordered FCD
  source and target type identities SHALL equal the subjects' producer-local
  kind discriminators byte-for-byte. The library SHALL refuse unknown,
  reversed, wrong-kind, or foreign-authority endpoints and SHALL NOT sort them.
- Workflow, role-instance, channel, node, message, send, receive, delivery,
  attempt, effect, receipt and configuration subjects SHALL remain distinct
  values whenever their producer-local kinds or identities differ.
- Exact replay of one complete relationship value SHALL be idempotent. Reusing
  one producer authority and relationship identity with any incompatible
  declaration or endpoint facts SHALL refuse as an identity contradiction.

## Correlation subsystem

- A required relationship slot SHALL retain the selected producer authority,
  declaration identity, and ordered endpoint keys; it SHALL NOT contain or
  infer a runtime relationship identity.
- Correlation outcomes SHALL be exactly `resolved`, `missing`, `conflicting`,
  and `ambiguous`. `missing` is incomplete; `conflicting` and `ambiguous` are
  refusals. An invalid producer, declaration, identity, or endpoint is an
  admission refusal before correlation and is not a fifth outcome.
- Zero distinct matching values SHALL be `missing`; one SHALL be `resolved`;
  more than one SHALL be `ambiguous`. Replayed copies of the same complete value
  count once. A value under the required declaration and source with an
  incompatible target SHALL be `conflicting`.
- The library SHALL NOT correlate from trace IDs, provider identities, foreign
  keys, display names, timestamps, record attributes, transport data, or
  ambient configuration.

## Bounds

The existing exact relationship and required-relationship limits SHALL bound
all validation, deduplication, declaration lookup and correlation work. One
over either caller-declared limit SHALL refuse before retaining partial state.

## Owner artifact versioning

Because the pinned `quire.observation.record/v1` and
`quire.observation.population/v1` schemas cannot represent the FR-287 authority
key and are immutable, the library SHALL preserve their exact schema bytes and
publish the qualified replacements as `quire.observation.record/v2` and
`quire.observation.population/v2`. It SHALL NOT change a v1 schema digest in
place. Their identity-preimage version labels SHALL also advance to v2 so the
new member sets cannot alias a v1 preimage. The other seven owner contracts
remain at v1.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | Admission consumes and retains the pinned FCD `AdmittedStaticBundle`; no local producer-selection shadow remains. | Test (TC-005) |
| FR-005-AC-2 | Independently mutating authority identity, revision namespace/value, digest algorithm/domain/version/value, kind, or object bytes changes the qualified-subject key; missing/empty runtime members refuse. | Test (TC-005) |
| FR-005-AC-3 | Semantic comparison across authority or kind refuses, while presentation, trace, timestamp, arrival and transport changes cannot affect a subject key. | Test (TC-005) |
| FR-005-AC-4 | A relationship resolves only against its exact FCD declaration and authored endpoint order; unknown, reversed, wrong-kind and foreign endpoints refuse. | Test (TC-005) |
| FR-005-AC-5 | Exact relationship replay is idempotent; same-identity incompatible rebinding refuses; multiple distinct values for one slot are ambiguous. | Test (TC-005) |
| FR-005-AC-6 | Missing, conflicting and ambiguous are distinct non-success outcomes, and no heuristic input creates a relationship. | Test (TC-005) |
| FR-005-AC-7 | Collection permutation produces one lexicographic key order without creating causal, delivery, retry or timestamp semantics. | Test (TC-005) |
| FR-005-AC-8 | Exact and one-over relationship limits preserve the existing fail-closed resource behavior. | Test (TC-005) |
| FR-005-AC-9 | The two qualified owner schemas publish as v2 while both v1 schema files remain byte-identical to the pre-#18 baseline. | Test (TC-005) |

## Dependencies

- `agent-ix/filament-core-data#95` / PR #99 at `4042882` owns and validates
  Producer interface 1.2 static bundles, revisions, digest selections and
  relationship declarations.
- `agent-ix/quire-specification` FR-287 defines the runtime subject key and
  FR-262 defines relationship identity, endpoint and correlation semantics.
- [FR-001](FR-001-qualify-observation-admission.md) owns the encompassing
  observation-admission outcome and resource limits.

## Status

Implemented and independently reviewed for `agent-ix/quire-observation#18`.
