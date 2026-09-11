---
id: FR-003
title: "Preserve qualified result facts at consumer handoff"
type: FR
relationships:
  - target: "ix://agent-ix/quire-observation/US-001"
    type: "implements"
---
# FR-003: Preserve qualified result facts at consumer handoff

## Description

When emitting an assessment result, the library SHALL preserve independently
readable execution, scope progress/closure, truth, settlement, support,
activation, participation, completeness, provenance, dependencies, and late
supersession facts, or SHALL report an explicit mapping loss/refusal.

## Inputs

- A typed assessment disposition and its immutable selection dependencies.
- A caller-selected typed consumer mapping.

## Outputs

- A consumer handoff preserving all represented result facts.
- An explicit refusal or loss record when a target cannot represent a required
  fact.

## Behavior

- The library SHALL NOT convert pending, incomplete, refused, unsupported, or
  lossy results into a passed Boolean.
- The library SHALL retain selected definition/profile identities and digests
  with each result.
- The library SHALL preserve prior result bytes when a late contradiction occurs
  and link any superseding/invalidation result to them.

## Error Conditions

Missing, duplicate, scope-cross-wired, or unsupported target fields SHALL
refuse the handoff or emit explicit loss; none may be discarded silently.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | Healthy, violating, untriggered, incomplete, and unsupported outcomes remain distinct. | Test (TC-003) |
| FR-003-AC-2 | A handoff preserves all required result axes and rejects omission, duplication, or scope cross-wiring. | Test (TC-003) |
| FR-003-AC-3 | A consumer mapping records explicit loss and cannot promote it to preservation. | Test (TC-003) |

## Dependencies

- [FR-002](FR-002-replay-and-incremental-assessment.md) supplies the typed
  assessment disposition.
- Downstream temporal, protocol, and verification consumers select their own
  mappings; this library does not implement those consumers.
