---
id: Task-006
title: "#10 — correct the coverage record and run the authoritative audit"
type: Task
status: done
resolution: superseded
track: D
priority: P1
relationships:
  - target: ix://agent-ix/quire-observation/Task-003
    type: depends_on
  - target: ix://agent-ix/quire-observation/Task-005
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-001
    type: references
  - target: ix://agent-ix/quire-observation/FR-002
    type: references
  - target: ix://agent-ix/quire-observation/FR-003
    type: references
  - target: ix://agent-ix/quire-observation/NFR-001
    type: references
  - target: ix://agent-ix/quire-observation/NFR-002
    type: references
  - target: ix://agent-ix/quire-observation/TC-001
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-002
    type: verifies
  - target: ix://agent-ix/quire-observation/TC-003
    type: verifies
  - target: ix://agent-ix/quire-observation/IT-001
    type: verifies
---
# Task-006: #10 — correct the coverage record and run the authoritative audit

> Superseded in shape. The current owner-contract review gate and Task-013 own
> the authoritative coverage, review, and promotion record.

## Scope

Make `gap_analysis.md` state what is true about its own coverage, then get the first
tool-authoritative verdict this repository has had.

## Subtasks

- **Correct the Remediation Status.** 18 was the trace-target count, not matrix rows;
      the matrix authors 8 rows. Say which number counts what.
- **Remove or date the stale Coverage Summary** that still reports FR 0%, AC 0% and
      "2 backed rows of 14 total matrix rows" three paragraphs below a claim of full
      coverage.
- **Replace the blocker sentence.** `quoin#365` closed on 2026-09-11; the installed
      plugin still wants a plan bundle, and `Plan-001` is now that bundle.
- **Rename the remaining "scope facts" language** to the four axes the spec now names.
- **Run the audit.** `quoin:gap-analysis --plan Plan-001`, recording its verdict as a
      validated SpecReview under `spec/reviews/`.

## Deliverables

- A `gap_analysis.md` whose numbers agree with each other and with `quire coverage`.
- A SpecReview carrying the gap-analysis verdict.

## Notes

- This task is the epic's exit gate: `quire coverage` backed == total with `unbacked_rows`
  empty, and a PASS verdict from the audit rather than from self-assessment.
- The functional coverage table's status column stays unclassifiable for the upstream reason
  in `agent-ix/quoin#368`; `unbacked_rows` gates independently of it and must be empty.
