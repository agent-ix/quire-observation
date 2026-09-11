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
activation, participation, completeness, provenance, dependencies, retained
event, and late supersession facts, or SHALL report an explicit mapping
loss/refusal.

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
- The consumer capability selection SHALL identify preservation or explicit
  loss for every independently readable result axis: assessment identity,
  disposition, settlement, support, progress identity, decision progress,
  decision closure, surrounding progress, surrounding closure, global closure,
  activation, participation, completeness, provenance, dependencies, retained
  events, and late supersession.
- A single capability SHALL NOT stand for two facts a consumer can represent
  independently; each scope progress and scope closure fact SHALL carry its own
  capability and its own typed loss.
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
