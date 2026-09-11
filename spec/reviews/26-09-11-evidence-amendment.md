---
id: SR-007
title: "Evidence review of the corrected observation result-model amendment"
type: SpecReview
analysis: evidence
scope: "spec/non-functional/NFR-001-bound-retained-observation-state.md (NFR-001-AC-3, NFR-001-AC-4, NFR-001-M-3, NFR-001-M-4), spec/test-cases/TC-002-replay-incremental-agreement.md, spec/test-cases/TC-003-result-handoff.md (amended by commit 18e8f67)"
review_set: subset
---
## Summary

This lens reviewed the four verification obligations commit 18e8f67 added to
NFR-001 — acceptance criteria NFR-001-AC-3 and NFR-001-AC-4 and metric rows
NFR-001-M-3 and NFR-001-M-4, all authored `Test` — and the amended TC-002 and
TC-003 procedures and expected results that must discharge them. `quire coverage
--json` confirms the engine minted exactly these four obligations and no others
from the amendment (22 obligations total: 16 acceptance criteria, 6 metrics).
`quoin advise` reports 0 mismatch, 0 uncatalogued and 0 inconclusive across all
22, so the authored *class* is right everywhere: every recommendation for the new
rows (`bdd-spec-by-example` / `unit-testing` on the criteria, matched on the
`example` property shape; `performance-benchmarking` on the metrics, matched on
`quantified-threshold`) is class `Test`. Neither `Inspection` nor `Analysis` is
advised for any new obligation, and nothing in the statements warrants one by
judgement: every new claim is observable from a returned value in a bounded run.

The defects are therefore not in the declared methods but in what discharges
them. NFR-001-AC-4 and NFR-001-M-4 are properly supported: TC-001 already
describes exercising "exactly/one-over each bound" over "record/member bounds"
and expects over-limit inputs refused. NFR-001-AC-3 and NFR-001-M-3 are not.
TC-002's amended Expected Results asserts the new retained-event outcome, but
its Test Procedure declares no bound, feeds no one-over event input, and reads no
retained-state counter, so the amended text states a conclusion it never sets up.
The setup for AC-3's second clause — making a representation unavailable — exists
only in TC-003's procedure, which AC-3 does not name and which carries no
`verifies` edge to NFR-001.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | NFR-001-AC-3 carries two independent obligations under one id, one statement hash, one declared method and one target test case: a bound ("retains no event beyond its declared event bound") and a reporting outcome ("an unavailable assessment reports no retained event"). The two fail independently and no single measurement discharges both, so a test that proves only the bound will mark the whole criterion backed and the reporting clause will never be reported undischarged by any engine check. Split into two criteria before the #6 tests are written, so each gets its own hash and its own binding. (Same split recommended from the EARS lens for a different reason: SR-006/FND-009.) | NFR-001-AC-3 |
| FND-002 | high | TC-002's amended Test Procedure describes no evidence for either half of NFR-001-AC-3. It sets no event bound, submits no one-over event input, and inspects no retained-state counter; its only bound-adjacent element is the pre-existing "retained-state-exhausted" fixture in the Description, which discharges FR-002-AC-3's "exhausted cases stay explicit", not a bound. The Expected Results sentence "an unavailable assessment reports no retained event" is an assertion with no procedure step producing an unavailable assessment. As amended, TC-002 cannot discharge NFR-001-AC-3 or NFR-001-M-3 — add explicit procedure steps: declare an event bound, replay at exactly-at-limit and one-over in batch mode, read the retained-event counter, and construct an unavailable assessment. | NFR-001-AC-3; NFR-001-M-3; TC-002 |
| FND-003 | medium | The only procedure in the corpus that makes a representation unavailable is TC-003's ("make a target representation unavailable for each required axis"), and the same commit added the retained-event axis to TC-003's declared axis set — so AC-3's second clause is in fact set up by TC-003, while AC-3's Verification cell names TC-002 alone. Either name TC-003 on the clause (preferably as its own criterion, per FND-001) or move the unavailable-assessment setup into TC-002; do not leave the assertion and its setup in different documents. | NFR-001-AC-3; TC-003 |
| FND-004 | medium | The Measurement and Evaluation table no longer covers the Acceptance Criteria table one-for-one. Four metric rows now pair with four criteria, but the pairing only holds for the bound clauses: AC-3's reporting clause has no metric row, no target and no threshold, so the quantified half of the amendment is complete while the qualitative half has no measurement at all. Splitting AC-3 (FND-001) leaves a criterion with no metric row unless one is added for it. | NFR-001; NFR-001-AC-3; NFR-001-M-3 |
| FND-005 | medium | NFR-001-AC-3 narrows the Statement's retained-event bound to batch replay, while the Statement bounds retained events unconditionally and the existing evidence already exercises the incremental path (`tests/replay.rs::tc002_incremental_intake_enforces_event_and_active_key_bounds`, bound to TC-002). NFR-001-M-3 inherits the narrowing ("after one-over batch replay"). The incremental event bound is thus tested but unasserted, the mirror image of AC-2, which asserts the active-key bound for incremental intake only while M-2 measures it "after one-over replay". Scope both criteria to both paths, or say per path which path each governs. | NFR-001-AC-3; NFR-001-AC-2; NFR-001-M-2; NFR-001-M-3 |
| FND-006 | low | `quoin advise` recommends `performance-benchmarking` (class Test, Benchmark) for all four NFR-001 metric rows, matched solely on the `quantified-threshold` characteristic. Judgement, not the catalog: these are correctness counts with a 0-excess threshold, not latency or throughput figures, so a benchmark harness is the wrong instrument and the authored `Test` cell should stand. Recording the non-escalation so a later auditor does not read the advisor's output as an unactioned mismatch. | NFR-001-M-1; NFR-001-M-2; NFR-001-M-3; NFR-001-M-4 |
| FND-007 | low | NFR-001 declares `quality_attribute: reliability`, and the catalog applies `fault-injection` (class Test, Integration) to the `reliability` characteristic, yet the advisor offered it for no NFR-001 obligation — the new criteria matched on the `example` property shape only, so the NFR's own quality attribute contributed no characteristic to the match. Judgement: exhausting a declared bound is a resource-exhaustion condition the library claims to tolerate, and fault-injection is the closer instrument for the one-over cases than a fixed-value unit test. The declared class is unchanged either way, so this is a suite-planning note, not a correction to the Verification cell. | NFR-001; NFR-001-AC-3; NFR-001-AC-4 |
| FND-008 | low | NFR-001's `## Verification` prose was not extended with the acceptance and metric tables: it says the tests "submit exactly-at-limit and one-over inputs and inspect the returned disposition and retained-state counters", which describes the three bound criteria and nothing about AC-3's unavailable-assessment reporting clause. A reader planning the suite from this section will build only the counter tests. | NFR-001 |
