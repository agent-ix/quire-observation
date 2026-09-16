---
id: FR-009
title: "Publish revisioned observation authority bundles"
type: FR
relationships:
  - target: ix://agent-ix/quire-observation/US-001
    type: implements
  - target: ix://agent-ix/quire-observation/FR-007
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-008
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-160
    type: references
  - target: ix://agent-ix/quire-specification/FR-325
    type: references
---
# FR-009: Publish revisioned observation authority bundles

## Description

When the selected observation authority is exported, the library SHALL publish
one canonical immutable bundle keyed by its authority, scope, positive revision,
and exact `quire.observation-authority/v1` or
`quire.observation-authority/v2` contract with explicit
predecessor/supersession relations. The library SHALL select v1 for a nonempty
population and v2 only for a structurally empty population.

## Inputs

- Exact selected authority and scope identities.
- The exact bundle contract version; v1 requires a nonempty population, while
  v2 requires an empty required-member population.
- Canonical records, populations, positions, clock/progress, activation,
  completeness, lateness, conflict, and availability facts from FR-004 and
  [FR-007](./FR-007-preserve-partial-values-and-clock-intervals.md) through
  [FR-008](./FR-008-bind-activation-progress-and-completeness.md).
- A positive revision and, after the initial revision, the exact direct
  predecessor bundle identity plus a nonempty set of declared replacement
  relations.
- For a later revision, one strict-read bounded lineage view identifying the
  selected authority/scope's current head and every already-issued direct child
  of that head.

## Outputs

- Canonical bundle bytes, a content-derived bundle identity, and a constructor-
  private validated read view.
- A canonical successor lineage view derived from the accepted prior view and
  new bundle.
- A typed refusal without a partial bundle or view for invalid contract,
  authority, scope, revision, lineage, component, encoding, or bounds.

## Behavior

- The library SHALL include every selected I07 component exactly once or in its
  declared canonical set.
- For v1, the library SHALL preserve the existing complete eleven-role profile
  and its immutable schema bytes.
- For v2, the library SHALL require an empty required-member population and
  exactly the `population`, `position`, `clock`, `progress`, `closure`,
  `completeness`, and `availability` roles.
- For v2, the library SHALL require zero positions, zero completeness facts with
  complete state, and zero required and available results with available state.
- For v2, the library SHALL forbid the record-dependent `observation`, `partial`,
  `capture`, and `activation` roles rather than manufacturing a sentinel record.
- The library SHALL derive the bundle identity from the contract version,
  authority, scope, revision, predecessor, replacements, and exact component
  identities and digests.
- The library SHALL require revision one to omit a predecessor and replacements.
- The library SHALL require every later revision to increase the predecessor's
  revision and to name that exact predecessor bundle.
- The library SHALL require a later revision's predecessor to equal the current
  head in the supplied strict-read lineage view.
- The library SHALL refuse a successor when the supplied lineage view already
  records a different direct child for the same predecessor.
- The library SHALL require each replacement to name one prior fact and one new
  fact of the same semantic role and authority-qualified subject.
- The library SHALL preserve prior bundle bytes.
- The library SHALL NOT infer a replacement from ingestion order, equal labels,
  or a changed current default.
- The library SHALL distinguish exact replay from same-key/different-bytes
  contradiction.
- The strict reader SHALL validate canonical bytes and every independently
  supplied expected selection before exposing a view.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | Repeated derivation from the same complete inputs produces byte-identical bundle bytes and identity. | property-based-testing (TC-009) |
| FR-009-AC-2 | A valid later revision names its exact predecessor and replacements while prior bytes remain unchanged. | unit-testing (TC-009) |
| FR-009-AC-3 | Missing, stale, cross-authority, cross-role, self-referential, or already-branched supplied lineage refuses without a bundle. | unit-testing (TC-009) |
| FR-009-AC-4 | Exact replay is idempotent; reusing one authority/scope/revision key for unequal bytes is an identity contradiction. | property-based-testing (TC-009) |
| FR-009-AC-5 | Omitting, duplicating, cross-wiring, or exceeding a component bound refuses before a validated view exists. | unit-testing (TC-009) |
| FR-009-AC-6 | A complete empty population publishes and strict-reads only as the exact seven-role v2 profile; v1 bytes and schema digest remain unchanged, while a v1 empty profile, a nonempty v2 profile, a sentinel record, an added/omitted role, or contract substitution refuses before a validated view exists. | unit-testing (TC-009) |

## Error Conditions

An invalid initial revision, non-increasing revision, missing or stale current
head, known sibling successor, self-link, cross-authority/scope/role replacement,
same-key unequal replay, malformed component, or one-over bound returns a typed
refusal without bundle or successor-lineage output. Concurrent publication needs
an external compare-and-swap store; this pure library validates only the exact
lineage view supplied to the call. A v1/v2 lineage transition is refused; an
empty and nonempty population have different exact population subjects and do
not form one revision lineage.

## Dependencies

- [FR-004](./FR-004-publish-observation-authority-artifacts.md) supplies the
  existing owner documents.
- [FR-007](./FR-007-preserve-partial-values-and-clock-intervals.md) and
  [FR-008](./FR-008-bind-activation-progress-and-completeness.md) supply the
  complete-V1 facts.
- QSpec FR-325 defines the I07 bundle contract; QSpec FR-160 defines revisioned
  uncertainty semantics.
