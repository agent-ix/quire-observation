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
readable execution, decision progress, decision closure, surrounding progress,
surrounding closure, truth, settlement, support, activation, participation,
completeness, provenance, dependency, retained-event, and late-supersession
facts, or SHALL report an explicit mapping loss or refusal.

## Inputs

- A typed assessment disposition, its retained-event count, and its immutable
  selection dependencies, as returned by [FR-002](FR-002-replay-and-incremental-assessment.md).
- A caller-selected typed consumer mapping declaring one capability per result
  axis.

## Outputs

- A consumer handoff preserving all represented result facts.
- An explicit refusal or loss record when a target cannot represent a required
  fact.

## Behavior

- The library SHALL NOT convert pending, incomplete, refused, unsupported, or
  lossy results into a passed Boolean.
- The library SHALL retain selected definition/profile identities and digests
  with each result.
- The library SHALL report, for each of the seventeen independently readable
  result axes, either preservation or one explicit typed loss: assessment
  identity, disposition, settlement, support, progress identity, decision
  progress, decision closure, surrounding progress, surrounding closure, global
  closure, activation, participation, completeness, provenance, dependency,
  retained-event, and late-supersession.
- The library SHALL accept one independently settable capability per result axis.
- The library SHALL NOT let one capability stand for two result axes.
- The library SHALL preserve prior result bytes when a late contradiction occurs
  and link any superseding/invalidation result to them.

The four scope axes are distinct facts, not one: *decision progress* and
*decision closure* are the progress authority and the closure boundary of the
scope the decision was taken in; *surrounding progress* and *surrounding closure*
are the same two facts for the enclosing scope. A consumer can hold any one of
them without the others. The *retained-event* axis is the count of events the
assessment retained, which a consumer needs to tell a bounded assessment from a
truncated one.

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
