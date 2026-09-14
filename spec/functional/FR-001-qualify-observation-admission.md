---
id: FR-001
title: "Qualify observation admission without ambient inference"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
---
# FR-001: Qualify observation admission without ambient inference

## Description

When admitting an observation request, the library SHALL retain the selected
package, producer, binding, source/schema, subject, scope, and resource-limit
identities, or SHALL return a typed refusal or incomplete outcome.

## Inputs

- A `native-linked-package/1` identity, revision, and digest.
- A Producer interface 1.2.0 identity, digest, model, and configuration.
- One binding, related-subject graph, strict-read
  `quire.observation.explicit-members/v1` selection, finite scope, record set,
  and explicit resource limits.

## Outputs

- An `Available` qualified-observation envelope retaining the selected package,
  producer, binding, subject, relationship graph, scope, records, and limits.
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
duplicate record, and resource-limit exhaustion are typed refusal conditions.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | A valid record retains its exact selections and visibility. | Test (TC-001) |
| FR-001-AC-2 | Wrong entity, signal, unit, trigger, schema, or producer selection is refused. | Test (TC-001) |
| FR-001-AC-3 | Missing required valuation, relationship, membership, or closure remains incomplete. | Test (TC-001) |
| FR-001-AC-4 | Conflicting or ambiguous relationships and one-over resource inputs are refused. | Test (TC-001) |
| FR-001-AC-5 | A member object identity remains opaque, while a membership or population identity is independently derived and compared and a selected producer/closure definition digest is retained in its declared domain. | Test (TC-001) |

## Dependencies

- [US-001](../usecase/US-001-qualify-observations.md) defines the consumer need.
- **Upstream**: `agent-ix/quire-spec-language` owns semantic package compilation
  and exports the `native-linked-package/1` selection and producer-definition
  digest. This library treats member object identities as opaque, but owns the
  FR-263/FR-264 population and explicit-members canonical identities and verifies
  them through its strict readers. Closure authority is also owned here;
  `quire-protocol` owns only protocol-result canonicalization.
- Producer interface 1.2.0 and the selected `native-linked-package/1` artifact
  are caller-provided dependencies, not crate dependencies.
