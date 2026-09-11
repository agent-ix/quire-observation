---
id: SR-011
title: "Risk and complexity review of the corrected-result-model work bundle"
type: SpecReview
analysis: risk-complexity
scope: "spec/functional/FR-002-replay-and-incremental-assessment.md, spec/functional/FR-003-preserve-observation-result-handoff.md, spec/non-functional/NFR-001-bound-retained-observation-state.md, spec/integration/IT-001-observation-assessment-handoff.md, spec/tests.md; issues #5-#10 under EPIC #1; src/handoff.rs, src/replay.rs, tests/replay.rs, tests/pipeline.rs at commit d5dd3f9"
review_set: subset
---
## Summary

This analysis scored the requirements touched by the six remaining issues of
EPIC #1 (#5-#10) on technical risk and volatility, as a readiness gate before
the Plan-001 bundle is tasked. The bundle is buildable now and nothing in it
should be deferred: the consumer-facing API it changes has no integrated
consumer, so this is the cheapest moment to move it. The material risks are
semantic, not schedule: #6 writes "an unavailable assessment reports no
retained event" into two requirement documents and an NFR metric row, which
makes the new retained-event axis report zero in exactly the truncation case
FR-003 says the axis exists to expose; and #8 keys its contradiction predicate
on the *current* result's settlement basis while the fact it must contradict
belongs to a *prior* result the library never sees and never validates against
the current rule, scope, deadline, or cutoff. The seventeen-axis capability
record is genuinely volatile (thirteen axes to seventeen in a single amendment
cycle), but `ResultAxis::ALL` plus a length-asserting sweep is the right kind
of stabilizer — it prices an added axis at one table row rather than freezing
the axis set — provided the assertion compares the axis *set* and the capability
lookup is an exhaustive `match`, not a bare `len()` equality.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | #6 makes `unavailable()` report `retained_events: 0`, and FR-002 Behavior, NFR-001's metric table, and NFR-001-AC-5 now all state it as a SHALL. But `Disposition::Exhausted` — the truncated case — is produced through `unavailable()`, so the new retained-event axis reports `0` precisely when the history was truncated, while FR-003 Behavior justifies the axis as the fact "a consumer needs to tell a bounded assessment from a truncated one". The distinction survives only because `Disposition` carries it separately, which makes the axis's stated justification false as written. The count also becomes ambiguous between "assessed zero events" and "no assessment occurred". If this is to change later it costs three documents plus the `quire-protocol` capability contract; decide now whether `retained_events` is `Option<usize>` (or a typed `Bounded`/`Truncated`/`NotAssessed` report) or whether FR-003's justification sentence is the part that is wrong. | FR-002 Behavior ("When an outcome is unavailable, the library SHALL report no retained event"), FR-003 Behavior closing paragraph, NFR-001 Measurement row 5, NFR-001-AC-5; issue #6; src/replay.rs:320-333 |
| FND-002 | high | #8's contradiction table is keyed on `ReplayResult.basis` — the basis of the result being computed now — but the fact a late record must contradict belongs to the prior immutable result named by `request.prior_result_identity`, which is an unvalidated bare `Identity`. Nothing ties that identity to the same rule, scope, deadline, or late cutoff, so the predicate reasons about a result it cannot read. The assumption that the current basis equals the prior result's basis holds only because late records are excluded from the current decision, and fails outright if the prior result was produced under a different `DecisionRule` or `ReplayRequest`. Either carry the prior result's `(disposition, basis)` (or its digest) in the request, or state in FR-002 that the caller warrants input identity between the two assessments. | src/replay.rs:58-59 (`prior_result_identity`), :274-276, :299-302; FR-002 Behavior ("a late record that contradicts the settled result"), FR-002 Inputs; issue #8 |
| FND-003 | high | The `EligibleDeadline` row of #8's table treats only `witness_signal` as contradicting. A late *counterexample* arriving inside the scope window would have produced `Violated` rather than `MissedDeadline` over the full history — a different typed disposition, which FR-003-AC-1 requires to stay distinct. Restricting contradiction to the witness arm therefore leaves a settled `MissedDeadline` result unsuperseded when the complete history no longer supports it. Either add `counterexample_signal` to the `EligibleDeadline` row, or record in FR-002 the decision that a late counterexample agrees with non-satisfaction and does not supersede. | issue #8 contradiction table, `EligibleDeadline` row; src/replay.rs:240-255 (deadline settlement), FR-002 Behavior, FR-003-AC-1 |
| FND-004 | medium | #8's `NotSettled` row supersedes on either rule arm, but FR-002 Behavior says a supersession is linked only "for a late record that contradicts the **settled** result", and under `NotSettled` nothing was settled. The intent is defensible (an `Open` prior result that the late record would turn decisive is genuinely invalidated), but the requirement text and the implementation table disagree on whether an unsettled prior result can be superseded. Resolve in the requirement before the table is coded, or #8 ships out of conformance with the requirement it implements. | FR-002 Behavior, FR-002 Outputs; issue #8 contradiction table, `NotSettled` row; src/replay.rs:304-318 |
| FND-005 | medium | #7's stabilizer is asserted as "the table's length equals `ResultAxis::ALL.len()`". Length equality does not establish coverage: with seventeen axes, a table of seventeen rows that duplicates one axis and omits another still passes. Assert that the set of axes exercised by the table equals the set in `ResultAxis::ALL` (distinct, and equal as a `BTreeSet`), and back it with a capability lookup written as an exhaustive `match` over `ResultAxis` so a new variant fails compilation rather than failing one assertion. As specified the test is the right mechanism at the wrong strength. | issue #7; src/handoff.rs:44-59, :113-128, :175-213; tests/replay.rs:237-266 |
| FND-006 | medium | Loss is asserted today as an ordered `Vec` equality (`lost_axes == vec![ResultAxis::ScopeFacts]`). Once the table-driven sweep of #7 covers seventeen axes, every case is coupled to the declaration order of the enum and to the order of the loss loop, so a cosmetic reordering during #5/#6 breaks unrelated cases. Compare loss as a set, or document the declaration order as part of the contract. | tests/replay.rs:259-266, :221-232; src/handoff.rs:175-213; issues #5, #6, #7 |
| FND-007 | medium | Of the six issues, #6 is the one most likely to need rework once `quire-protocol`#12 integrates. That task's acceptance explicitly demands the other changes — "independently changed or cross-wired decision-scope/surrounding-execution progress and closure" (#5), "late contradiction", "immutable supersession outcome" (#8), "every global closure state" — but names no retained-event or retention-count fact at all. #6 therefore adds a capability flag and a loss axis to the consumer contract with no declared downstream demand, and its shape (count vs. bound vs. truncation state, per FND-001) is the part likeliest to be renegotiated. Build it anyway: it closes a real over-report and supplies NFR-001's evidence. Expect the axis's *type*, not its existence, to churn. | agent-ix/quire-protocol#12 Acceptance (EPIC #14, `Task-010`); issue #6; FR-003 Behavior |
| FND-008 | low | The seventeen-axis record is volatile but cheap to change: the crate is `publish = false` at `0.1.0`, the single declared downstream consumer (`quire-protocol`#12) is OPEN and itself blocked on its `Task-006`/#8, and the axis set already grew from thirteen to seventeen in one amendment cycle. Because callers construct capabilities as `ConsumerCapabilities::all()` plus field mutation rather than struct literals, adding a flag stays source-compatible for existing call sites. No issue in this bundle should be deferred to await the integration — deferral would park the work behind a consumer that is itself blocked, and #7's stabilizer is what makes the later churn cheap. | Cargo.toml:8 (`publish = false`); src/handoff.rs:130-148; tests/replay.rs:221-223, :257-258; agent-ix/quire-protocol#12 Dependencies and boundary |
| FND-009 | low | The `bool` capability record cannot express the per-axis partial representation that `MappingState::Conditional` implies: `Conditional` is computed globally from the disposition, not per axis. If `quire-protocol` needs "this axis is representable only in a degraded form", the flags become three-state and all seventeen change together. Named here as the known next churn vector, not as work for this bundle. | src/handoff.rs:33-40, :113-128, :215-228; FR-003 Behavior, FR-003-AC-3 |
| FND-010 | low | FR-003's Description enumerates fifteen fact names while its Behavior enumerates the seventeen axes; the Description omits progress identity and global closure and uses "execution"/"truth" where Behavior uses the axis names. A reader counting axes from the Description gets a different set than `ResultAxis` will hold after #5 and #6. Low risk, but it is the document #7's length assertion will be read against. | FR-003 Description, FR-003 Behavior |
| FND-011 | low | #9's two changes carry the lowest technical risk in the bundle and the highest evidence value: both are test-only, both have a stated failing-first check, and D7's mutation check (drop `request.progress` in `IncrementalAssessment::new`) is the only named mutation probe in the bundle. Sequence #9 early rather than last; it is the cheapest way to make the later semantic changes (#8 especially) observable. | issue #9; tests/pipeline.rs:176-186, tests/replay.rs:66-79, :93; FR-002-AC-1, IT-001-SC-02 |

## Risk Register

| Req | Tech Risk | Volatility | Drivers | Mitigation |
|---|---|---|---|---|
| FR-001 | Low | Low | Untouched by this bundle; admission envelope settled in the #3 amendment. | None needed. |
| FR-002 | High | Medium | #8's contradiction predicate keyed on a prior result the library cannot read (FND-002); `EligibleDeadline` arm possibly under-inclusive (FND-003); `NotSettled` row disagrees with the requirement text (FND-004); new `AmbiguousOrder` disposition splits a path previously folded into `Unsupported`. | Implement the predicate against a differential oracle — contradiction holds iff replaying the non-late history plus the late record as in-window yields a different `(disposition, basis)` than the settled result — and keep #8's table as the test expectation rather than the implementation. Resolve FND-003/FND-004 in FR-002 before coding. |
| FR-003 | Medium | High | Seventeen axes after #5/#6, grown from thirteen in one cycle; `bool`-only capability record (FND-009); no integrated consumer to validate the shape (FND-007, FND-008). | Keep the model moving cheaply rather than freezing it: `ResultAxis::ALL` plus a set-equality sweep and an exhaustive `match` capability lookup (FND-005), loss compared as a set (FND-006). Do not defer; re-review the axis set when `quire-protocol`#12 lands. |
| NFR-001 | Medium | Medium | #6 states the zero-retained-event rule as a metric row plus AC-5, binding FND-001's ambiguity into the measurement. | Settle FND-001 first; assert the bound (`retained_events <= max_events`) alongside the zero-for-unavailable rule so AC-3's evidence is not a single `== 0` comparison. |
| NFR-002 | Low | Low | #7's C2 change adds the missing `dependencies` assertion so the AC-2 tag binds to something. | Land with #7; no separate mitigation. |
| IT-001 | Low | Low | #9 extends a test that today calls only `admit`; no production coupling is introduced, and the spec declines the type-level link. | Land #9 early as the observability floor for #8 (FND-011). |

## Answers to the Gate Questions

**Volatility of the seventeen-axis record, and is `ResultAxis::ALL` the right
stabilizer.** Volatility is high by evidence, not by suspicion: the axis set
grew by four in the single amendment that just merged, the crate is
`publish = false`, and the only declared consumer has not integrated. The
stabilizer is nonetheless correct, because it does not stabilize the *axis
list* — it stabilizes the invariant that every axis has an omission test. The
cost it imposes on a future axis is one table row, which is the price that
should be paid. It hardens nothing that is still moving; what it hardens is the
sweep. Two corrections: assert set equality rather than length (FND-005), and
make the capability lookup an exhaustive `match` so the compiler, not a test,
is the first line of enforcement.

**Risk that #8's contradiction table is the wrong semantics.** High, on two
independent counts — the basis it keys on belongs to a prior result it cannot
read (FND-002) and the `EligibleDeadline` arm excludes a counterexample that
would have changed the disposition (FND-003) — plus a requirement-conformance
mismatch on `NotSettled` (FND-004). What surfaces it early is a differential
property test rather than more table rows: for any bounded history and any
partition into settled and late records, `supersedes.is_some()` must equal
"recomputing with the late record in-window yields a different
`(disposition, basis)`". That oracle fails immediately on FND-003 and is
indifferent to how the table is written, which is exactly the property a
hand-keyed table lacks.

**Most likely to need rework after integration, and what to defer.** #6 (see
FND-007). Defer nothing. `quire-protocol`#12 is blocked on its own
`Task-006`/#8, so deferral trades a cheap change now for a blocked dependency
later, and #5/#8 are named in #12's acceptance as facts it must exercise — they
are pulled, not speculative. #7 should land with or immediately after #5 and #6
so the churn it prices is priced from the first axis change, and #9 should land
early (FND-011).

**Risk in #6's change to `unavailable()`'s `retained_events`.** The breakage
risk of the observable change is low: `publish = false`, no integrated
consumer, and exactly one in-repo site reads a `retained_events` value
(`tests/replay.rs:333`, inside a struct literal). The semantic risk is high and
is FND-001 — not that the output changes, but that `0` is now specified as the
answer for the truncated case and is ambiguous against a genuine zero-event
assessment. Make the change now, while it costs one line; decide the
`Option`/typed-report question before `quire-protocol` consumes the axis.

## Top Hazards

1. **FND-001 — `retained_events: 0` for `Exhausted`.** The bundle's own
   justification for the new axis is the bounded-vs-truncated distinction, and
   the rule as written nulls the axis in the truncated case. Now written as a
   SHALL in FR-002, NFR-001's metric table, and NFR-001-AC-5; the cost of
   reversing rises the moment `quire-protocol` reads the flag.
2. **FND-002 — the contradiction predicate reasons about an unreadable prior
   result.** `prior_result_identity` is an unvalidated `Identity` with no tie to
   the rule or window that produced it.
3. **FND-003 — `EligibleDeadline` contradiction arm may be under-inclusive**, so
   a `MissedDeadline` result stays unsuperseded when the full history would
   read `Violated`.
4. **FND-005 — length-only exhaustiveness.** The durable fix #7 claims is
   weaker than stated; a duplicated row hides a missing axis.
5. **FND-007 — #6 has no downstream demand.** Build it, but expect the axis
   type to be renegotiated at integration.

## Failure-Domain Gaps

Cross-reference: SR-005 (integrity), SR-008 (scope-boundary) and SR-009
(amendment resolution) are the current failure-domain-adjacent deliverables;
this repository has no document with `analysis: failure-domain`, so no
dedicated failure-domain register exists to cross-check. Open domain gaps
carried by this bundle, from the analysis above: identity (FND-002, prior
result identity unvalidated against the current request), extension (FND-005,
exhaustiveness not compiler-enforced), and representation (FND-001, a count
overloaded with an absence). SR-008's FND-004 — the retained-event axis having
no declared entry in FR-003 Inputs — remains open and compounds FND-001.
