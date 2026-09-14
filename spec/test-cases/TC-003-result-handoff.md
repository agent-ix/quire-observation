---
id: TC-003
title: "Expose complete validated owner views"
type: TC
relationships:
  - target: "ix://agent-ix/quire-observation/FR-003"
    type: "verifies"
---
# TC-003: Expose complete validated owner views

## Description

Verify that the public handoff consists only of constructor-private FR-004
validated views and preserves each observation-owned axis without introducing a
protocol-result vocabulary.

## Test Procedure

Strict-read the record, population, position, clock, capture, progress, closure,
completeness and availability documents against independent selections.
Independently omit, duplicate, reorder and cross-wire their required envelope and
payload fields, and offer each document to the wrong owner reader.

## Expected Results

Valid bytes yield only the corresponding constructor-private view with read-only
access to every payload fact. Every invalid or unsupported representation
returns one typed refusal and no partial view.
Availability, completeness, progress and closure never become truth, settlement,
adequacy, conformance or Boolean values.
