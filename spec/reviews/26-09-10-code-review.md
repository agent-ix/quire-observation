---
id: SR-002
title: "Code review of quire-observation Rust core and campaign evidence"
type: SpecReview
analysis: code-review
scope: "src/, tests/, spec/"
review_set: all
---
## Summary

This review examined the complete Rust library, its integration tests, and the
repository-local OB01–03 campaign evidence. Formatting, strict Clippy, and all
18 tests pass, but the implementation and matrix have five high-severity
defects; this review therefore fails.

## Verdict

**FAIL** — high-severity defects can omit late contradictions, claim preserved
facts that were never modeled at the consumer boundary, and report unbacked
work as complete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Replay returns immediately on the first decisive record. A timely witness at sequence 1 followed by a late counterexample at sequence 2 returns `Satisfied` with no late record or supersession link, violating the requirement to retain a post-settlement record. | src/replay.rs:222; FR-002 |
| FND-002 | high | Successful admission returns only records. Package, producer, scope, membership, closure, and resource-limit selections are discarded from `Available`, so callers cannot receive the retained exact selections FR-001 requires. | src/lib.rs:260; src/lib.rs:400; FR-001 |
| FND-003 | high | Admission treats `membership_complete` as authoritative but never validates members, their identities, their anchors, or their record linkage. A complete-looking unrelated member set is accepted, so the selected population cannot constrain an admitted record. | src/lib.rs:189; src/lib.rs:287; src/lib.rs:306; FR-001 |
| FND-004 | high | Consumer capabilities model only five axes, while the handoff claims preservation for a result that also carries progress/closure scope facts, global closure, provenance, support, truth, and settlement. A consumer that cannot represent one of those omitted axes is still labeled `Preserved`. | src/handoff.rs:95; src/handoff.rs:125; FR-003 |
| FND-005 | high | The Test Matrix reports TC-001 and IT-001 complete although their evidence is unbound; all ten functional acceptance criteria are unbacked. Admission tests self-name TC-140 through TC-142, which have no matrix rows. | spec/tests.md:19; spec/tests.md:27; tests/admission.rs:90 |

## Gate Results

- `cargo fmt --check` — passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `cargo test --locked` — passed: 18 integration tests, 0 failures.
- `quire validate --scope . "spec/**/*.md" --summary` — passed: 13/13 grammar-clean; catalog duplicate-archetype warnings remain external to this repository.
- `quire coverage --scope . --json` — failed evidence completeness: 2/14 backed rows; 10/10 functional acceptance criteria unbacked.

## Gap-analysis Boundary

The installed Quoin plan-based gap-analysis workflow cannot issue its normal
validated verdict because this repository has no `plan/<Plan-id>-<slug>/plan.md`
bundle. The accompanying `gap_analysis.md` records the implementation and
traceability gaps found by the shared implementation-gap-analysis workflow; it
does not substitute an invented plan for the missing governed plan.
