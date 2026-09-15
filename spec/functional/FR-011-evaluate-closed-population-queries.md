---
id: FR-011
title: "Evaluate exact queries over closed related-workflow populations"
type: FR
relationships:
  - target: ix://agent-ix/quire-observation/US-001
    type: implements
  - target: ix://agent-ix/quire-observation/FR-008
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-009
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-041
    type: references
  - target: ix://agent-ix/quire-specification/FR-153
    type: references
  - target: ix://agent-ix/quire-specification/FR-180
    type: references
---
# FR-011: Evaluate exact queries over closed related-workflow populations

## Description

When a consumer requests a finite query over reservation, refund, or related-
workflow observations, the library SHALL evaluate every and only member of the
authority-qualified closed snapshot or event-time population selected by the
request.

## Inputs

- A validated FR-009 bundle revision and exact population/snapshot-or-window,
  membership-rule, completeness, progress, and authority selections.
- An exact subject kind, root relationship, grouping key, deterministic exposure
  order, and one admitted `filter`, `count`, or exact `sum` query plan.
- Exactly one duplicate policy: `effect-identity-deduplicating` or
  `occurrence-preserving`. Transport receipts are never population members under
  either policy.
- For `sum`, one exact numeric representation/unit, result domain, zero identity,
  and bounded ordered value projection.
- Caller-lowered member, arithmetic, byte, and work limits.

## Outputs

- A canonical ordered member/accounting record and exact Boolean, count, or sum
  value when the population and every required fact are complete.
- A typed incomplete or refused result with no partial definitive aggregate.

## Behavior

- The library SHALL admit a member only through its exact membership rule,
  authority-qualified subject, required relationship, and selected snapshot or
  half-open event-time window.
- The library SHALL include a member at a selected window start.
- The library SHALL exclude a member at a selected window end.
- The library SHALL apply the declared duplicate policy to semantic business-
  effect identities.
- The library SHALL NOT count repeated receipts as new effects.
- The library SHALL preserve duplicate occurrences when the selected query plan
  declares occurrence-preserving semantics.
- The library SHALL evaluate `filter`, `count`, and `sum` in the declared
  deterministic exposure order.
- The library SHALL retain each participating fact identity.
- The library SHALL perform exact checked sum arithmetic in the selected
  representation and unit.
- The library SHALL refuse an invalid intermediate prefix even when a reordered
  or final mathematical sum would fit.
- When membership, relationship, observation, progress, or closure is open,
  unknown, stale, ambiguous, contradicted, or incomplete, the library SHALL emit
  no partial definitive aggregate.
- The library SHALL refuse a foreign authority, subject kind, grouping key,
  relationship, unit, revision, duplicate policy, or one-over resource input.
- The library SHALL NOT discover members from ambient stores, trace identifiers,
  provider identities, timestamps, or display order.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-011-AC-1 | A closed selected population includes every and only admitted member once according to its explicit duplicate policy and in deterministic exposure order. | property-based-testing (TC-011) |
| FR-011-AC-2 | A member at the half-open window start participates and one at the end does not; adjacent windows never double-count a boundary member. | property-based-testing (TC-011) |
| FR-011-AC-3 | Two related partial refunds sum exactly to the captured amount only when both effects, their relationships, and the complete population are authority-qualified. | unit-testing (TC-011) |
| FR-011-AC-4 | Replayed receipts do not create effects, while distinct admitted effects and occurrence-preserving duplicate values remain distinguishable. | unit-testing (TC-011) |
| FR-011-AC-5 | Open, unknown, stale, ambiguous, contradicted, incomplete, foreign, or over-bound authority produces no partial definitive aggregate. | property-based-testing (TC-011) |
| FR-011-AC-6 | Exact sum rejects wrong representation/unit, overflow, and an invalid intermediate prefix without reordering operands. | property-based-testing (TC-011) |

## Error Conditions

Open, unknown, stale, ambiguous, contradicted, or incomplete authority returns a
typed incomplete outcome. A foreign identity, undeclared duplicate policy,
receipt-as-effect substitution, representation/unit mismatch, arithmetic
overflow or invalid prefix, malformed query, or one-over resource input returns
a typed refusal. Neither outcome contains a definitive aggregate.

## Dependencies

- [FR-008](./FR-008-bind-activation-progress-and-completeness.md) supplies
  independent population, progress, closure, and completeness authority.
- [FR-009](./FR-009-publish-revisioned-authority-bundles.md) supplies the exact
  revisioned I07 input.
- QSpec FR-041 owns common bounded query semantics; FR-153 owns closed-environment
  selection; FR-180 owns the reference-workflow corpus.
