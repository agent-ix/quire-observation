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
incomplete, ambiguous, quiet-deadline, late, and retained-state-exhausted
fixtures in both batch and incremental modes.

## Test Procedure

For each fixture, feed the same qualified records and progress assertions to a
batch replay and a sequence of incremental updates. Repeat deterministic
fixtures twice and test a late contradictory record after a settled result.

## Expected Results

Both modes return equal full dispositions for the same history. Silence settles
only under matching progress authority. Open, incomplete, ambiguous, late, and
exhausted cases stay explicit, and late contradiction produces a linked result.
