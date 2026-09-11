---
id: SR-012
title: "failure-domain review of the amended observation result model"
type: SpecReview
analysis: failure-domain
scope: "spec/functional/FR-001-qualify-observation-admission.md; spec/functional/FR-002-replay-and-incremental-assessment.md; spec/functional/FR-003-preserve-observation-result-handoff.md; spec/non-functional/NFR-001-bound-retained-observation-state.md; spec/non-functional/NFR-002-reproduce-observation-outcomes.md; spec/test-cases/TC-002-replay-incremental-agreement.md; spec/test-cases/TC-003-result-handoff.md; spec/spec.md"
review_set: subset
---
# SR-012: failure-domain review of the amended observation result model

## Summary

This analysis applied the failure-domain checklist — extension-point failure
behaviour, entity identity, evaluation purity, topological robustness — to the
spec as amended at `d5dd3f9`, reading `src/lib.rs`, `src/replay.rs` and
`src/handoff.rs` only to ground what the authored text does and does not decide.
The amendment closed the axis-collapse and ambiguous-order gaps, but it
introduced three vocabulary terms that carry decisions and define none of them:
*late*, *contradicts*, and *unavailable*. Lateness has no declared input and two
incompatible definitions; the contradiction predicate that now gates every
supersession link is named four times and defined nowhere; and the unavailable
outcome is specified to drop its retained-event count while saying nothing about
the late evidence it also drops. Seven non-success outcomes — eight with the new
ambiguous-order result — compete for one disposition slot with no stated
precedence, which NFR-002's reproducibility claim silently depends on.

## Findings

| ID      | Severity | Summary | Refs | Escape Cause |
| ------- | -------- | ------- | ---- | ------------ |
| FND-001 | high | *Late* has no definition and no declared input. FR-002 Behavior defines lateness by arrival relative to settlement ("when a record arrives after settlement"), while the assessment actually classifies it against a caller-declared late cutoff and deadline that appear in no Inputs section of any requirement. Nothing therefore states what happens to a record at exactly the deadline instant (it can be late evidence but never in-window evidence), nor to a late-arriving record whose event time is past the deadline (it is evidence of nothing and is reported as neither late nor retained). Confirms the boundary lead. | FR-002 Inputs, Behavior; FR-001 Inputs; src/replay.rs:212-222 | missing-requirement |
| FND-002 | high | The contradiction predicate the amendment now makes load-bearing is undefined. FR-002 requires a supersession link "only for a late record that contradicts the settled result" and FR-003 requires preserving prior result bytes "when a late contradiction occurs", but no requirement says which signal contradicts which settlement basis: whether a witness contradicts an `EligibleDeadline` (missed-deadline) settlement, whether a counterexample contradicts a witness-settled satisfaction, or whether an unsettled `Open` result is contradictable at all — "settled result" excludes it, yet nothing states that a late record against an open prefix links nothing. Nor is the case of a contradiction with no prior result identity addressed. Two conforming implementations can disagree on every supersession link. | FR-002 Outputs, Behavior; FR-003 Behavior; src/replay.rs:264-276 | missing-requirement |
| FND-003 | high | No precedence is stated among the non-success outcomes, and the amendment added a ninth. FR-002 requires open prefix, incomplete history, incomplete progress, ambiguous membership, ambiguous order, unsupported profile and exhausted state to stay distinct, and requires ambiguous order to be distinct from unsupported profile, but never says which outcome is reported when several conditions hold at once (an unsupported profile over an ambiguous order over an exhausted history). NFR-002 demands the same disposition for repeated evaluation of the same input, which is only achievable under a total precedence no requirement declares; "distinct" constrains the value space, not the choice. | FR-002 Behavior, Error Conditions; NFR-002 Statement; src/replay.rs:155-198 | missing-requirement |
| FND-004 | high | The unavailable outcome is specified to discard the late evidence it carries. FR-002 now requires that an unavailable outcome report no retained event, but says nothing about retained late records, so a late contradiction arriving for an untriggered, ambiguous-boundary, unsupported, incomplete or exhausted assessment may vanish with no link, no loss record and no outcome change — the one case in which evidence disappears silently, which FR-002's "inspect the full bounded history … even when an earlier record was decisive" rule exists to prevent. FR-003's late-supersession axis then truthfully reports preservation of an empty fact. Confirms the `unavailable()` lead: not intended by the authored text, and not stated either way. | FR-002 Behavior, Outputs; FR-003 Behavior; NFR-001-AC-5; src/replay.rs:318-334 | missing-requirement |
| FND-005 | medium | The retained-event axis cannot perform the job FR-003 assigns it. FR-003 states the count exists so "a consumer needs to tell a bounded assessment from a truncated one", while NFR-001-AC-5 requires every unavailable assessment to report zero retained events — and the exhausted-state outcome, the truncation case, is an unavailable outcome. The single number that is supposed to reveal truncation is therefore required to read zero exactly when truncation occurred, and is indistinguishable from an empty history. Either exclude exhaustion from AC-5 or give truncation its own reported fact. | FR-003 Behavior and rationale; NFR-001-AC-5, NFR-001-M-5; TC-002 Expected Results | wrong-requirement |
| FND-006 | medium | FR-003 still carries two different axis sets, in a new way. The Behavior bullet enumerates the seventeen axes; the Description — the normative statement — enumerates fifteen facts, two of which (*execution*, *truth*) appear in no axis list, while assessment identity, disposition, progress identity and global closure appear in no Description item. TC-003 defers to "FR-003's declared seventeen-axis set", so the authoritative set exists only in the Behavior bullet and the Description contradicts it. This is not the resolved scope-fact collapse (the four scope facts are now separate in both places); it is a residual naming and cardinality mismatch the resolution recorded as closed. | FR-003 Description, Behavior; TC-003 Description, Test Procedure | wrong-requirement |
| FND-007 | medium | A partially supplied capability record has no defined outcome, and axis independence is asserted without a loss-dependency rule. FR-003 requires one independently settable capability per axis and forbids one capability standing for two, but never says what a mapping that declares fewer than seventeen capabilities yields; Error Conditions offers "SHALL refuse the handoff or emit explicit loss" disjunctively, so an absent capability may become a refusal in one implementation and a silent preserved axis in another. Separately, the axes are not semantically independent: a late-supersession link is unresolvable once assessment identity is lost, and settlement and support are uninterpretable once disposition is lost, yet no requirement states that losing one axis degrades another's preservation claim. Confirms the partial-capability lead. | FR-003 Behavior, Outputs, Error Conditions; src/handoff.rs:110-125 | missing-requirement |
| FND-008 | medium | Three identities, one axis, and one of them is declared by no requirement. The handoff result identity and the replay assessment identity are distinct values, and the single "assessment identity" axis covers the latter only — a consumer preserving that axis can lose the identity by which the result is cited and superseded. NFR-002 constrains "the same result identity" across repeated evaluation, but no requirement names a result identity as any stage's output (FR-002 Outputs lists a disposition, decision support, supersession link and retained-event count), states how it is derived, or says whether a superseding result carries a new one or re-uses its predecessor's. Confirms the identity lead. | FR-003 Description, Behavior; FR-002 Outputs; NFR-002 Statement; src/handoff.rs:95-107; src/replay.rs:103-112 | missing-requirement |
| FND-009 | medium | Bound exhaustion on the incremental path is not a disposition, so the two paths cannot be compared there. FR-002's Description requires the same assessment disposition from batch replay and incremental processing for the same admitted history, and NFR-001-AC-2 and NFR-001-AC-3 place active-key and event bounds on incremental intake — but nothing states that incremental intake surfaces a refused record as the exhausted disposition rather than as an intake-level rejection the caller may ignore and then finish anyway. The agreement obligation and the bound obligation meet at a surface no requirement specifies. | FR-002 Description; NFR-001-AC-2, NFR-001-AC-3; src/replay.rs:128-152 | missing-requirement |
| FND-010 | medium | The opaque member equality key has no stated comparand and no stated failure outcome. FR-001 permits using a member's object identity "only as an opaque equality key against the selected membership" without saying which field of the membership it is compared to — a member carries both an object identity and a record identity — what equality means for a value the library may not normalize (byte equality is the only reading left, but it is not stated), or what the library does when the key matches nothing. FR-001-AC-5 says an unrecognized object identity does not block admission, which only coheres if the object identity is not the member record identity that every admitted record must bind to; the text never says they are different things. | FR-001 Behavior, FR-001-AC-5, Error Conditions; NFR-001 Statement; src/lib.rs:390-429 | missing-requirement |
| FND-011 | low | Topological robustness of the relationship graph is unaddressed. FR-001 accepts a "related-subject graph" and declares conflicting and ambiguous relationships typed refusals, but states no behaviour for a cyclic, self-referential or disconnected relationship set, and NFR-001 bounds records, active keys, events and members while bounding neither the relationship count nor any traversal depth. A cycle is neither conflicting nor ambiguous, so it falls through every typed refusal the requirement enumerates; correlation is currently pairwise only, which the spec neither requires nor forbids. | FR-001 Inputs, Behavior, Error Conditions; NFR-001 Statement; src/lib.rs:437-467 | missing-requirement |
| FND-012 | low | FR-002's assessment window is not anchored to FR-001's admitted scope. Admission validates every record anchor against a half-open selected clock range, while assessment settles against a scope start, deadline and late cutoff that no requirement relates to that range or to its clock family — FR-002 only requires clock family to be "preserved as a separate input". Nothing forbids assessing admitted records over a window in a different clock family or outside the scope they were admitted against, and nothing states which boundary convention the assessment window uses even though admission's is explicitly half-open. | FR-001 Inputs, Behavior; FR-002 Inputs, Behavior; src/lib.rs:158-198; src/replay.rs:216-222 | missing-requirement |

## Proposed Additions

Five of the twelve findings are one missing definition each, and they are the
cheapest to close before the plan bundle is tasked.

- **FR-002 (lateness)**: declare the deadline, the late cutoff and the scope
  start as inputs, state the boundary convention for each (admission's range is
  half-open; the assessment window must say so too), and state the outcome for a
  record whose event time falls on or after the deadline — in-window evidence,
  late evidence, or retained-and-not-evidence. Addresses FND-001 and FND-012.
- **FR-002 (contradiction)**: define the predicate as a relation between a late
  record's signal and the settled basis, case by case over the five bases, and
  state explicitly that an unsettled result is not contradictable and links
  nothing. Addresses FND-002.
- **FR-002 (precedence)**: state the evaluation order of the non-success
  outcomes as a total order, and cite it from NFR-002, whose reproducibility
  claim depends on it. Addresses FND-003.
- **FR-002 (unavailable outcome)**: state whether an unavailable outcome retains
  its late records. If it does not, state that the caller is told the evidence
  was dropped; silent loss is the one behaviour the requirement set otherwise
  forbids throughout. Addresses FND-004.
- **FR-003 (axis set and mapping completeness)**: make the seventeen-axis
  enumeration appear once, normatively, and have the Description cite it rather
  than restate it; state the outcome for a mapping that declares fewer than
  seventeen capabilities as one obligation, not a disjunction; and state which
  axis losses invalidate another axis's preservation claim. Addresses FND-006
  and FND-007.
- **StR or FR-001 (member identity)**: state that a member's object identity and
  its record identity are distinct keys, which of them the opaque equality key is
  compared against, that comparison is byte equality, and what outcome an
  unmatched key produces. Addresses FND-010.
- **NFR-001 (bounds)**: bound the relationship set and state the termination
  guarantee for relationship correlation on a cyclic input. Addresses FND-011.

## Verification

Each finding was taken from the authored text at `d5dd3f9` and confirmed against
the three sources a reader has: the requirement's own sections, the test case
declared to verify it, and the implementation the requirement governs. The
implementation was read only as evidence of what the text leaves open — the
unimplemented amendments to FR-002 and FR-003 tracked in the open assessment and
handoff issues are deliberately not reported here, and no finding above is
discharged by implementing them. FND-005, FND-006 and FND-009 are contradictions
internal to the spec and are confirmable from the documents alone.
