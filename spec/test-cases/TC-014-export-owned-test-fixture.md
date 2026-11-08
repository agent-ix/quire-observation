---
id: TC-014
title: "Export the owned test fixture"
type: TC
relationships:
  - { target: ix://agent-ix/quire-observation/FR-012, type: verifies }
---
# TC-014: Export the owned test fixture

## Description

Check the `test-support` fixture export for FR-012.

## Test Procedure

Build with `--features test-support`; compare `test_support::FCD_STATIC_BUNDLE_1_2`
with the fixture file and admit it. Inspect `Cargo.toml` for an empty default
feature set.

## Expected Results

The bytes are identical and admitted; no default feature enables the export.

## Status

Implemented in `tests/test_support.rs`.
