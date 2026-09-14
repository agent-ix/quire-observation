---
id: SR-032
title: "Rust review — authority-qualified runtime references"
type: SpecReview
analysis: code-review
scope: "Cargo.toml, Cargo.lock, deny.toml, schemas/, src/, tests/"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/FR-005", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TC-005", type: references }
---
# SR-032: Rust review — authority-qualified runtime references

## Summary

The `quire-observation#18` change was reviewed against observation baseline
`9ac80e93f4b68a2c7d5a337f9a448ad10de798fc`, QSpec FR-287/FR-262, and the
FCD Producer interface 1.2 Rust boundary pinned at
`404288282402d60de007295ccbafa960532b955e`. All findings discovered during
implementation and review were fixed. The final boundary is constructor-private,
exact, bounded, fail-closed, and backed by real Rust paths rather than shadows,
mocks, or approximate matching.

## Verdict

**PASS** — no open Rust-idiom, API-boundary, safety, resource, test, or
specification-faithfulness finding remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved finding | - |

## Resolved during review

| Former finding | Resolution | Refs |
| --- | --- | --- |
| The initial projection draft changed immutable v1 schemas | Restored both v1 files byte-for-byte and introduced record/population v2 contracts plus v2 identity-preimage labels | `schemas/`, `src/authority/{observation,population}.rs` |
| Required-slot error variants made the refusal enums unnecessarily large | Boxed the exact required relationship in missing/conflict/ambiguity variants without losing typed evidence | `src/lib.rs` |
| Relationship-identity collision detection was quadratic | Replaced the pairwise scan with an exact `BTreeMap` keyed by authority and runtime relationship identity | `src/lib.rs` |
| The new AGPL/git dependency was not represented by the existing dependency policy | Added narrow crate-license exceptions and an exact allowed FCD source; pinned both `version = "=0.0.0"` and the full immutable revision | `Cargo.toml`, `Cargo.lock`, `deny.toml` |
| The isolated Cargo target appeared as untracked repository state | Added only `/target-codex-backends/` to the repository ignore list | `.gitignore` |

## Rust and boundary review

- `AdmissionRequest` owns FCD's admitted static-bundle capability. Local producer
  selection, closed subject-kind, and relationship-kind shadows were removed.
- Qualified subject and relationship fields are private and constructible only
  from non-empty opaque bytes plus an admitted producer capability. Semantic
  cross-authority and cross-kind comparisons return typed refusals.
- Declaration resolution uses the retained exact bundle; endpoint order is
  checked against the FCD declaration and is never sorted or inferred.
- Offered relationship populations are checked at their exact caller ceiling
  before validation, sorting, deduplication, or correlation. Duplicate complete
  values are idempotent; an identity rebound to incompatible facts refuses.
- Correlation has exactly the specified resolved, missing, conflicting, and
  ambiguous meanings. No trace, display, time, arrival, transport, or ambient
  value participates in a key or match.
- Owner v2 wire types retain the complete bundle key, Producer interface
  version, configuration digest selection, opaque identity bytes, declaration,
  and ordered endpoints. Strict readers expose those exact facts.
- The crate forbids unsafe code. Production code contains no `unwrap`, panic,
  unchecked integer conversion, async/blocking bridge, lock, filesystem,
  environment, wall-clock, global mutable state, stub, or warning suppression.
- The only production `expect` writes to `String`, an infallible operation; the
  other production-looking schema parse is confined to a test-only helper.
- Tests use the real FCD admission path and exact upstream fixture. There are no
  mocks, ignored tests, `should_panic` cases, test-only behavior switches, or
  tautological production replacements.

## Dependency and provenance review

- `agent-ix-baseline-producer = =0.0.0` is selected from the full FCD revision
  `404288282402d60de007295ccbafa960532b955e`; Cargo.lock resolves that same
  revision.
- The copied test fixture records its exact source path, revision, and
  AGPL-3.0-only terms. It is not embedded in the production library.
- `cargo deny 0.19.8 check` reports `advisories ok, bans ok, licenses ok,
  sources ok`.
- `publish = false` remains effective (`cargo metadata` reports an empty
  publish allow-list).

## Gates

- Toolchain: `rustc 1.98.1`, `cargo 1.98.1`.
- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace --all-targets --all-features --locked --target-dir target-codex-backends --quiet` — PASS.
- `cargo clippy --workspace --all-targets --all-features --locked --target-dir target-codex-backends --quiet -- -D warnings` — PASS.
- `cargo test --workspace --all-targets --all-features --locked --target-dir target-codex-backends --quiet` — PASS: 11 unit, 14 admission, and 17 authority tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --target-dir target-codex-backends --quiet` — PASS.
- `check-jsonschema 0.37.2 --check-metaschema` over both v2 schemas — PASS.
- `cargo deny 0.19.8 check` — PASS.
- `git diff --check` — PASS; `resources/native-v1/` and both v1 schema files have no diff.
