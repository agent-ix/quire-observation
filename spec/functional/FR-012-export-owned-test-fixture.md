---
id: FR-012
title: "Export the owned static-bundle fixture for downstream tests"
type: FR
relationships:
  - target: ix://agent-ix/quire-observation/US-001
    type: implements
  - target: ix://agent-ix/quire-observation/FR-001
    type: depends_on
---
# FR-012: Export the owned static-bundle fixture for downstream tests

## Description

Where the `test-support` cargo feature is enabled, the library SHALL export the
bytes of its own Producer interface 1.2 static-bundle test fixture so that
downstream test code reads the owner's fixture instead of keeping a copy.

## Behavior

- The library SHALL export the bytes as `test_support::FCD_STATIC_BUNDLE_1_2`,
  embedded from this repository's fixture file.
- The library SHALL NOT compile the export unless `test-support` is enabled, and
  `test-support` SHALL NOT be a default feature.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-012-AC-1 | With `test-support` enabled, the exported bytes equal the owned fixture file and are admitted by `StaticProducerBundle::admit_json`. | unit-testing (TC-014) |
| FR-012-AC-2 | `test-support` is absent from the default feature set, so the default public API has no `test_support` module. | inspection (TC-014) |

## Dependencies

- [FR-001](./FR-001-qualify-observation-admission.md) owns the admission of the exported bundle.
