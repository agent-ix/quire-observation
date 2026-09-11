---
id: Task-007
title: "#12 — define contradiction, lateness and outcome precedence"
type: Task
status: not_started
track: S
priority: P0
relationships:
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
---
# Task-007: #12 — define contradiction, lateness and outcome precedence

## Scope

The readiness gate for this plan found that three things the code must decide are not defined
by any requirement, and that the first amendment introduced two contradictions of its own.
Settle them in the spec before any semantic code lands.

## Subtasks

- [ ] **Define contradiction differentially.** A late record contradicts the result when
      replaying with that record in window would yield a different `(disposition, basis)`.
      This replaces the basis-keyed table the epic plan proposed, which keyed on the current
      result's basis while the contradicted fact belongs to the prior result the library
      cannot read, and which left a settled `MissedDeadline` unsuperseded by a late
      counterexample.
- [ ] **Declare the lateness inputs and the boundary convention.** The late cutoff, deadline
      and scope start appear in no Inputs section, and nothing states the outcome for a record
      at exactly the deadline or a late arrival past it. Admission's range is half-open; say
      whether replay's is.
- [ ] **State the precedence of the non-success outcomes.** NFR-002's reproducibility claim
      needs a total order; "distinct" only constrains the value space.
- [ ] **Say what an unavailable outcome does with late records**, not only with the
      retained-event count.
- [ ] **Fix the retained-event justification.** FR-003 says the axis tells bounded from
      truncated while `NFR-001-AC-5` forces zero for every unavailable outcome, and the
      truncation case is unavailable. Truncation is carried by the disposition; say so.
- [ ] **Reconcile FR-003's two axis sets.** The Description still enumerates fifteen names,
      using "execution" and "truth" and omitting four axes the Behavior list names.
- [ ] **Correct FR-001's Dependencies.** It cites `quire-spec-language` FR-015 and FR-027 for
      an export neither states. Name the owning component without citing a requirement that
      does not carry the obligation.
- [ ] **Trim FR-003's Inputs claim** that the immutable dependencies come from FR-002's
      result; they are supplied at the handoff call site.
- [ ] **Resolve the remaining identity, partial-capability, incremental-exhaustion and
      member-comparand gaps** recorded in #12.

## Deliverables

- Amended FR-001, FR-002, FR-003, NFR-001 with a resolved `/spec-review` pass.
- Any new acceptance criterion assigned to the task that binds it.

## Notes

- This is the gate in front of the semantic work: `superseding_records` cannot be implemented
  from a requirement that does not say what contradicts what.
- Source: SR-010 (dependency), SR-011 (risk-complexity), SR-012 (failure-domain).
