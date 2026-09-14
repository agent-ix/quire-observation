---
id: SR-028
title: "Code review — complete observation-owner contract system"
type: SpecReview
analysis: code-review
scope: "Cargo.toml, deny.toml, schemas/, src/, tests/"
review_set: subset
---
# SR-028: Code review — complete observation-owner contract system

## Summary

The complete QObs implementation was reviewed against baseline `7f6285` using the Rust,
language-independent code, and implementation-gap rubrics. All findings discovered during
review were fixed; the final public boundary is typed, bounded, fail-closed, traced, and
free of stubs or warning suppression.

No applicable `AssuranceProfile` exists under `spec/`; the ordinary review rubric applies.

## Verdict

**PASS** — no open code, Rust-idiom, test, integrity, or implementation-gap finding remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings | - |

## Resolved during review

| Former finding | Resolution | Refs |
| --- | --- | --- |
| Clock-bearing position/progress/closure artifacts omitted exact revision | Added revision to typed selections, schemas, canonical preimages, payloads, readers, accessors, and cross-wire tests | `src/authority/{position,progress,closure}.rs` |
| Public validated views did not expose every owned payload fact | Added complete read-only typed accessors for all nine views | `src/authority/*.rs`, `tests/authority.rs` |
| Position derivation could accept a foreign clock | Bound ledger selection and every position to the qualified clock identity/revision | `src/authority/position.rs` |
| Some admission failures were stringly and relationship populations were unbounded | Added closed typed field/resource/artifact enums and exact pre-match ceilings | `src/lib.rs`, `tests/admission.rs` |
| Request identity assignment could partially mutate before failure | Derive on a clone and commit only after all identities succeed | `src/authority/population.rs` |
| Progress constructor had eight positional arguments | Reused the typed clock selection, keeping identity/revision paired and reducing positional API risk | `src/authority/{clock,progress}.rs` |
| Dependency policy was absent and the trace dependency was incompletely pinned | Added narrow license/source policy and exact compatible version plus immutable tag | `Cargo.toml`, `deny.toml` |
| Spec/test trace semantics retained superseded result-model mappings | Reconciled owner-only specs, plan, atomic ACs, matrix ownership, and integration edges | `spec/`, `plan/` |

## Rust review

- Errors use crate-boundary typed envelopes and a closed stable code catalog; no caller
  decision depends on diagnostic prose.
- The crate forbids unsafe code, contains no production `unwrap`, and has only one
  infallible `String` formatting `expect`; the schema parse `expect` is test-only.
- No async, blocking, lock, filesystem, environment, wall-clock, or shared-state behavior
  exists in the library.
- Untrusted bytes are preflighted under effective byte/depth/string/population/work bounds;
  integer wire conversions use `try_from`, allocation growth is checked, and recursive JSON
  descent is capped at depth 64.
- Emitted wire contracts use typed `Serialize` structs/enums; all deserialized records deny
  unknown fields and strict readers reject duplicate/reordered/trailing/noncanonical data.
- Every changed behavior reaches real production code; there are no mocks, skips,
  `should_panic` tests, conditional assertions without failure paths, or test-only behavior
  switches.

## Implementation-gap inventory

- Public declarations inventoried: 170; public functions: 26; unowned behaviors: 0.
- Substantive source modules: 12; source stubs: 0; test stubs: 0.
- Integration tests: 28 with 204 behavioral assertions/rejections.
- Full discovery and formalization: `gap_analysis.md`; archived reusable practices:
  `retros/quire-observation/retro/RETRO-2026-09-13-owner-contracts/accepted_proposals.md`.

## Gates

- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace --all-targets --all-features --locked` — PASS.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `cargo test --workspace --all-targets --all-features --locked` — PASS: 9 unit, 12
  admission, 16 authority, 0 doctest failures.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked` — PASS.
- `cargo build --workspace --all-features --release --locked` — PASS.
- `cargo deny check` — PASS: advisories, bans, licenses, and sources.
- `cargo audit` — PASS: 26 dependency packages scanned, no vulnerability reported.
- All ten schemas parse and match their pinned SHA-256 digests.
- `quire validate` — PASS for 34/34 spec documents and 16/16 plan documents.
- `quire coverage` — PASS: 33/33 rows backed; no unbacked row, status lie, or untracked
  symbol.
