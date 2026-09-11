---
id: SR-006
title: "EARS conformance review of the corrected observation result-model amendment"
type: SpecReview
analysis: ears-conformance
scope: "spec/functional/FR-001-qualify-observation-admission.md, spec/functional/FR-002-replay-and-incremental-assessment.md, spec/functional/FR-003-preserve-observation-result-handoff.md, spec/non-functional/NFR-001-bound-retained-observation-state.md (statements added or changed by commit 18e8f67)"
review_set: subset
---
## Summary

This lens reviewed the nine requirement statements added or changed by commit
18e8f67 — two FR-001 Behavior bullets, the FR-002 Error Conditions amendment and
its new ambiguous-order paragraph, FR-003's amended Description sentence and its
two amended/new Behavior bullets, and NFR-001-AC-3 and NFR-001-AC-4. The
deterministic grammar check is satisfied (15/15 documents grammar-clean, zero
`[ears:*]` findings), so every finding below is semantic. One statement only
(NFR-001-AC-4) is conformant as written. The dominant defect is a requirement
written about a data structure — a capability, a capability selection, a declared
sequence — instead of about the library that must act, which leaves the
obligation without a responsible subject and, in FR-003, leaves the collapsed
scope axis the amendment exists to remove still standing in the statement of
record.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | FR-003's amended Description still collapses the four scope facts into one "scope progress/closure" item while the amended Behavior bullet names decision progress, decision closure, surrounding progress, and surrounding closure separately; the Description is the normative statement, so the exact collapse the amendment forbids survives in it — extend the Description's axis list to the four facts. | FR-003 |
| FND-002 | medium | FR-003's new bullet "A single capability SHALL NOT stand for two facts a consumer can represent independently" is a design constraint on the mapping type, not a requirement on an actor: no subject performs a verifiable action. Restate under the emit event with the library as subject, e.g. "When emitting an assessment result, the library SHALL map each independently readable result axis to exactly one consumer capability and SHALL report that capability's typed loss." | FR-003 |
| FND-003 | medium | The same FR-003 bullet is two statements joined by a semicolon (a universal prohibition plus a positive obligation), and the two differ in scope: the prohibition covers every independently representable pair of facts while the positive obligation covers only scope progress and scope closure, leaving the other thirteen declared axes with no one-capability-each obligation. Split, and state the positive obligation over the whole axis set. | FR-003 |
| FND-004 | medium | FR-003's amended axis bullet makes "The consumer capability selection" the subject of SHALL; a selection is an input value, not an actor that can identify anything — the library identifies. Restate with the library as subject. | FR-003 |
| FND-005 | medium | FR-002's new paragraph makes "A duplicated caller-declared sequence" the subject of SHALL report; the sequence reports nothing. The statement is also an unwanted condition written as a ubiquitous rule. Restate as unwanted-behaviour with a real subject: "If two qualified records declare the same caller-declared position, then the library SHALL report an ambiguous-order outcome distinct from the unsupported-profile outcome." | FR-002 |
| FND-006 | medium | FR-002's new ambiguous-order outcome has no acceptance criterion of its own, and FR-002-AC-3's bare "ambiguous" — written when the only ambiguous outcome was ambiguous membership — now reads as covering both outcomes, so one criterion stands for two distinct typed results. Name the order case explicitly in FR-002-AC-3 or add a criterion. | FR-002; FR-002-AC-3 |
| FND-007 | medium | FR-001's "SHALL treat a member's object identity as caller-opaque" names no concrete response: "treat as opaque" is a property of the design, and nothing observable distinguishes compliance from violation. Restate as the actions permitted and forbidden, e.g. "The library SHALL use a member's object identity only as an opaque equality key and SHALL NOT parse, normalize, or derive meaning from it." | FR-001 |
| FND-008 | medium | FR-001's digest bullet packs two obligations into one statement — retain the selected membership and closure digests, and do not recompute them. The prohibition is the testable half and needs its own statement; split into a ubiquitous retention requirement and a ubiquitous prohibition on recomputation. | FR-001 |
| FND-009 | medium | NFR-001-AC-3 is two criteria in one row, and the second ("an unavailable assessment reports no retained event") is not a bound at all: it is an outcome obligation on FR-002's unavailable-history result, outside NFR-001's Statement, and it has no row in the Measurement and Evaluation table, which covers only retained events after one-over batch replay. Split the bound criterion from the outcome obligation and site the latter on FR-002. | NFR-001-AC-3; NFR-001; FR-002 |
| FND-010 | low | FR-001's digest bullet and FR-002's new paragraph each carry a "because …" rationale clause inside the requirement statement ("because recomputing either requires a canonicalization this library does not own"; "because the caller-declared order is the only ordering authority"). Rationale is not part of an EARS statement — move it to the Dependencies or Description prose. | FR-001; FR-002 |
| FND-011 | low | One axis and one bound are each named three ways across the amended statements: "retained event" (FR-003 Description), "retained events" (FR-003 Behavior), "retained-event" (FR-003 Description of the handoff axis set); and "population-member" (NFR-001 Statement), "members" (metric row), "member" (NFR-001-AC-4). Settle one name per axis so a reader can match statement to criterion. | FR-003; NFR-001; NFR-001-AC-4 |
