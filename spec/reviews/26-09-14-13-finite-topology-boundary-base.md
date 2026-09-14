---
id: SR-034
title: "Base review of finite topology and anchored boundaries"
type: SpecReview
analysis: base
scope: "spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md"
review_set: subset
---
# SR-034: Base review of finite topology and anchored boundaries

## Summary

FR-006, TC-006 and Task-015 were reviewed against live observation issue #13,
QObs main `8d5445854657887a2a04ba0cd994d15bc347d61f`, QSpec FR-262/FR-289 and
the accepted QObs owner boundary. The original ticket wording overstated the
remaining implementation: QObs already implements the complete FR-289
scope/clock boundary, while its accepted architecture forbids graph and
temporal evaluation. The reviewed packet closes the actual evidence/policy gap
without reintroducing the superseded replay engine.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-034 | high | The ticket's former “window anchoring is unimplemented” premise contradicted the all-clock `TemporalBoundary` validation and progress/closure scope/clock checks merged in PR #19. | FR-004; `src/authority/{common,progress,closure}.rs` |
| FND-035 | high | Treating self-loops, cycles or disconnected explicit relationships as globally invalid would invent semantics absent from FR-262 and cross QObs's no-evaluator boundary. | FR-005; FR-006; QSpec FR-262/FR-043 |
| FND-036 | medium | The first packet used an unregistered `TestCase` archetype, mismatched TC headings, retained the obsolete matrix header workaround, and omitted Task-015 from the plan's test/track narrative. | TC-006; TM-001; Plan-001 |

## Remediation

- Recast #13 as an exact proof/closure task. FR-006 retains explicit finite
  edges under the already implemented population ceiling and expressly adds no
  traversal, reachability, closure, settlement or truth operation.
- Selected the least-assumptive topology policy: independently valid
  self-loops, cycles and disconnected components are data. A downstream
  evaluator may apply its own reviewed policy; observation admission does not.
- Bound the temporal evidence to the existing FR-289 representation for event
  position, fixed sample and timestamped event clocks, including exact scope,
  clock identity/revision, checked successor, point interval and strict
  watermark rules.
- Corrected TC-006 to the repository `TC` archetype and required sections,
  restored the currently installed TestMatrix `Status` header, linked NFR-001,
  and completed the Plan-001 test/track/dependency narrative.

## Review results

- **Authority:** FCD/FR-262 own edge validity; QObs owns exact admission and
  bounded retention; QSpec FR-043/qcir #69 own graph evaluation; QSpec FR-289
  owns inclusive-to-half-open meaning; progress/closure own its artifact form.
- **Topology:** no standalone graph object, implicit vertex, synthesized edge,
  transitive closure or topology refusal is introduced. Only exact edge values
  are retained and replay-deduplicated.
- **Bounds:** the offered populations are refused before relationship work.
  Because QObs performs no traversal, it needs no invented depth cutoff or
  resource-exhausted reachability result.
- **Temporal mapping:** the selected native lower/upper, half-open carrier,
  watermark, scope, clock family, identity and revision remain independently
  checkable. Discrete overflow and timestamp epsilon invention refuse.
- **Continuity:** progress/closure v1 bytes and identity preimages remain
  immutable. No owner schema, public evaluator API or result vocabulary changes.
- **Testability:** the pinned FCD fixture already contains a valid same-kind
  `Order-successor` declaration, and every positive/adverse topology or
  boundary clause maps to a real admission/derivation path. Negative
  public-surface claims are paired with review rather than mislabeled as
  runtime output.

## Evidence

- Targeted Quire validation: 7/7 specification, matrix and plan documents are
  grammar-clean after remediation with zero grammar findings.
- Implemented baseline inspection: `TemporalBoundary::validate`,
  `TemporalBoundary::matches_range`, progress/closure derivation and admission
  pre-work relationship ceilings on QObs main `8d54458`.
- Shared authority inspection: QSpec FR-262, FR-289 and FR-043 at standard main
  `a343138`.

## Verdict

PASS. No open specification-review finding blocks the bounded TC-006
implementation/evidence cycle.
