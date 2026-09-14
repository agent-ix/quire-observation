# Implementation Gap Analysis — Observation Owner Contracts

## Overview

This report maps the implemented QObs admission and observation-authority boundary back to
the unified FR-001 through FR-004 and NFR-001/NFR-002 specification. It supersedes the
earlier replay/result-handoff report because that architecture was explicitly replaced by
the accepted ecosystem owner model.

## Final status

No open implementation gap remains. Review discoveries were formalized in the
specification, implemented, and covered by executable Rust traces:

- exact opaque membership keys and independently derived observation/membership/population
  identities;
- exact clock identity plus revision on every clock-bearing owner artifact;
- bounded relationship and required-relationship populations;
- closed schemas, canonical identities, immutable correction lineage, strict readers, and
  constructor-private validated views;
- independent progress, closure, completeness, availability, and lateness vocabularies;
- byte/depth/string/population/position/capture/source/work ceilings with no partial output.

## Coverage summary

| Category | Inventory | Unowned or hollow | Evidence |
| --- | ---: | ---: | --- |
| Public Rust declarations | 170 | 0 | FR-001..FR-004, NFR-001..NFR-002 |
| Public Rust functions | 26 | 0 | TC-001..TC-004, IT-001 |
| Substantive source modules | 12 | 0 | all exceed stub threshold and implement owner logic |
| Integration test functions | 28 | 0 | 204 behavioral assertions/rejections |
| Test Matrix rows | 33 | 0 unbacked | `quire coverage` 33/33 |

The intentionally thin `src/authority/mod.rs` is the documented subsystem facade. It is
not a masquerading implementation: the eleven substantive sibling/common modules contain
the actual behavior.

## Discovery and remediation

### GAP-001: Caller-authored owner identities could become trusted state

**Discovery:** admission and helper APIs needed a single fail-closed rule for canonical
membership, population, and record identities.

**Remediation:** derivation is performed from validated selections; stale identities are
refused; request identity assignment derives into a clone before committing atomically.

**Prevention technique:** For every caller-supplied identity, identify the owning preimage
and test stale, cross-wired, and failed-helper cases.

### GAP-002: Clock-bearing artifacts could bind only a family name

**Discovery:** identity without exact revision permits a different clock definition to be
substituted under the same label.

**Remediation:** position, progress, and closure payloads, schemas, preimages, expected
selections, readers, and tests all bind both clock identity and revision. The progress API
now accepts the shared typed clock selection.

**Prevention technique:** Treat `(identity, revision)` as one typed selection at every
construction and reader boundary.

### GAP-003: Strict JSON parsing needed pre-deserialization resource accounting

**Discovery:** `serde_json` strictness alone does not guarantee byte, depth, string, array,
or visited-field ceilings before allocation and traversal.

**Remediation:** one bounded scanner accounts for all raw input before semantic
deserialization, and every deriver uses bounded output encoding plus semantic population
limits.

**Prevention technique:** Require a preflight stage that charges work before the general
decoder and returns no partial typed value.

### GAP-004: Independent states were vulnerable to implicit Boolean/result coercion

**Discovery:** progress, closure, completeness, and availability can be accidentally
collapsed by plausible combinations.

**Remediation:** each has a closed enum under its own contract; the 48-state product is
derived and strict-read without emitting truth, settlement, conformance, or Boolean data.

**Prevention technique:** Enumerate the complete Cartesian product of independently owned
state axes and assert the exact round-trip state on every axis.

### GAP-005: Dependency policy was implicit

**Discovery:** the crate had no `deny.toml`; `cargo deny` therefore could not express its
actual source and license policy.

**Remediation:** a narrow policy allows only the observed permissive dependency licenses,
the crate's own AGPL license as a crate-specific exception, and the exact test-only
`ix-trace-rs` Git source. The Git dependency is version- and tag-pinned.

**Prevention technique:** Treat dependency policy as a first-class repository artifact and
run it with the same strictness as compiler/test gates.

## Skill-evolution assessment

The useful techniques above are already present in the active Rust review, failure-domain,
scope-boundary, evidence, and gap-analysis skills. No global skill patch is proposed; the
value here is the concrete owner-contract application and its executable evidence.
