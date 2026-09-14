---
id: SR-030
title: "Base review of authority-qualified runtime references"
type: SpecReview
analysis: base
scope: "spec/functional/FR-005-adopt-authority-qualified-runtime-references.md"
review_set: subset
---
# SR-030: Base review of authority-qualified runtime references

## Summary

FR-005, TC-005 and Task-014 were reviewed against live observation issue #18,
QSpec FR-287/FR-262 at `b85df3e`, the merged observation baseline at `9ac80e9`,
and the accepted FCD Producer interface 1.2 Rust boundary at `4042882`. The
packet is coherent and ready for implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-028 | medium | Initial frontmatter used three relationship verbs outside the installed vocabulary. | FR-005, Task-014 |
| FND-030 | high | The first implementation draft changed two schema files while retaining their immutable v1 contract labels. | FR-004, FR-005 |

## Remediation

Replaced the invalid verbs with `depends_on`, `references`, and `verifies`;
targeted Quire validation is clean.

Preserved both v1 schema files byte-for-byte and assigned the qualified record
and population projections new v2 contract labels, schema files and digests.

## Review results

- **Authority:** FCD owns static bundle, revision, digest and relationship
  declaration validity. Observation owns runtime subject identity, runtime
  relationship identity, bounded correlation and admission outcome.
- **Boundary:** the request consumes FCD's constructor-private admitted bundle;
  it neither decodes FCD wire data nor creates a second producer-selection
  shadow. This direct semantic dependency is not an FCD transport adapter.
- **Identity:** every FR-287 member and every FR-262 relationship member has an
  exact comparison rule. Digest domain/version remain first-class, endpoints
  remain ordered, and key order is expressly non-causal.
- **Failure:** malformed runtime members, foreign/wrong-kind endpoints, unknown
  declarations and same-identity incompatible rebinding refuse. Missing,
  conflicting and ambiguous correlation remain unequal outcomes; exact replay
  is idempotent.
- **Bounds:** the existing two relationship limits cover declaration lookup,
  deduplication and matching before any qualified state is returned.
- **Scope:** transport decoding, heuristics, protocol evaluation, TL work,
  temporal interpretation and #13 topology/window anchoring remain excluded.
- **Plan continuity:** Task-014 changes only the partial admission seam and does
  not reopen Tasks 008–013 or their accepted owner architecture.

## Evidence

- `quire validate --scope . --summary` over FR-005, TC-005, Plan-001 and
  Task-014: 4/4 grammar-clean with zero grammar findings.
- `quire validate --scope . --okf spec --summary`: the new artifacts add no
  grammar findings; remaining bundle warnings predate #18 or are expected
  cross-repository references.

## Verdict

PASS. No open specification-review finding blocks Rust implementation.
