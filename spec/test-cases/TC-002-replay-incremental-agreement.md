---
id: TC-002
title: "Agree bounded replay and incremental assessment"
type: TC
relationships:
  - target: "ix://agent-ix/quire-observation/FR-002"
    type: "verifies"
---
# TC-002: Agree bounded replay and incremental assessment

## Description

Verify the selected timed-refund profile for satisfied, violated, open,
incomplete, ambiguous-membership, ambiguous-order, quiet-deadline,
missed-deadline, late, and retained-state-exhausted fixtures in both batch and
incremental modes.

## Test Procedure

For each fixture, feed the same qualified records and progress assertions to a
batch replay and a sequence of incremental updates. Repeat deterministic
fixtures twice, test a late contradictory record after a settled result, and
test a late record that agrees with or is irrelevant to the settled result. Then
declare an event bound and submit exactly-at-limit and one-over histories on both
the batch and incremental paths, reading the retained-event count each time, and
drive an outcome the library cannot assess and read its retained-event count.
Finally submit two records sharing one declared order position.

## Expected Results

Both modes return equal full dispositions for the same history, including the
missed-deadline fixture. Silence settles only under matching progress authority.
Open, incomplete, ambiguous-membership, ambiguous-order, late, and exhausted
cases stay explicit, and two records sharing one order position are ambiguous in
order rather than unsupported in profile. Neither path retains an event beyond
the declared bound, and an unavailable assessment reports a retained-event count
of zero. Only a late record that contradicts the settled result produces a linked
supersession; an agreeing or irrelevant late record is retained without one.
