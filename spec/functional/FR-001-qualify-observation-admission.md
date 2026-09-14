---
id: FR-001
title: "Qualify observation admission without ambient inference"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
  - target: "ix://agent-ix/quire-observation/FR-005"
    type: "refined-by"
---
# FR-001: Qualify observation admission without ambient inference

## Description

When admitting an observation request, the library SHALL retain the selected
package, producer, binding, source/schema, subject, scope, and resource-limit
identities, or SHALL return a typed refusal or incomplete outcome.

## Inputs

- A `native-linked-package/1` identity, revision, and digest.
- An FCD-admitted Producer interface 1.2.0 static-bundle capability.
- One binding, bounded related-subject relationship set, strict-read
  `quire.observation.explicit-members/v1` selection, finite scope, record set,
  and explicit record, member, relationship, and required-relationship limits.

## Outputs

- An `Available` qualified-observation envelope retaining the selected package,
  producer, binding, subject, relationship set, scope, records, and limits.
- `Incomplete` with one or more missing-premise reasons.
- `Refused` with the affected identity and typed validation cause.

## Behavior

- The library SHALL accept only `native-linked-package/1` and Producer interface
  1.2.0 selections with explicit identities and digests.
- The library SHALL refuse a source, schema, signal, unit, subject, producer, or
  clock-family substitution.
- The library SHALL correlate related instances only through supplied typed
  relationships and SHALL NOT use trace IDs, provider identities, or record
  attributes as a substitute.
- The library SHALL match a required relationship only by its exact type and
  directed endpoints and SHALL NOT traverse or infer transitive, cyclic, or
  timestamp-based relationships.
- The library SHALL bind every admitted record to exactly one supplied member
  record identity and compatible anchor in the selected scope.
- The library SHALL return missing required values, membership, closure, and
  relationships as explicit incomplete reasons rather than Boolean false.
- The library SHALL use a member's object identity only as an opaque equality
  key against the selected membership.
- The library SHALL NOT parse, normalize, or derive meaning from a member's
  object identity.
- The library SHALL derive and compare the FR-264 explicit-members identity from
  its sorted, distinct, bounded required-member set before admission.
- The library SHALL retain the selected producer-definition and
  closure-definition raw-artifact digests without interpreting their bytes.
- The library SHALL bind the qualified observation to the FR-263 population
  identity derived by the observation owner from the strict-read membership,
  selection, source, configuration, closure, completeness and progress inputs.

## Error Conditions

Selection mismatch, unsupported producer version, binding mismatch, subject
mismatch, conflicting/ambiguous relationship, clock mismatch, invalid scope,
duplicate record, and record/member/relationship resource-limit exhaustion are
typed refusal conditions.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | A valid record retains its exact selections and visibility. | Test (TC-001) |
| FR-001-AC-2 | Wrong entity, signal, unit, trigger, schema, or producer selection is refused. | Test (TC-001) |
| FR-001-AC-3 | Missing required valuation, relationship, membership, or closure remains incomplete. | Test (TC-001) |
| FR-001-AC-4 | Conflicting or ambiguous relationships and one-over resource inputs are refused. | Test (TC-001) |
| FR-001-AC-5 | A member object identity is compared only as an exact opaque key and is never parsed, normalized, or interpreted. | Test (TC-001) |
| FR-001-AC-6 | Explicit-members, observation, and population identities are independently derived and stale caller-authored identities are refused. | Test (TC-001) |
| FR-001-AC-7 | Selected producer and closure definition digests are retained in their declared domains without interpreting definition bytes. | Test (TC-001) |

## Dependencies

- [US-001](../usecase/US-001-qualify-observations.md) defines the consumer need.
- **Upstream**: `agent-ix/quire-spec-language` owns semantic package compilation
  and exports the `native-linked-package/1` selection and producer-definition
  digest. This library treats member object identities as opaque, but owns the
  FR-263/FR-264 population and explicit-members canonical identities and verifies
  them through its strict readers. Closure authority is also owned here;
  `quire-protocol` owns only protocol-result canonicalization.
- Producer interface 1.2.0 is consumed from the pinned FCD Rust crate as defined
  by [FR-005](FR-005-adopt-authority-qualified-runtime-references.md). The
  selected `native-linked-package/1` artifact remains caller-provided.
