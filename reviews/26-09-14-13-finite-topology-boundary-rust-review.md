---
id: SR-036
title: "Rust review — finite topology and anchored-boundary evidence"
type: SpecReview
analysis: code-review
scope: "tests/support/mod.rs, tests/topology_boundary.rs"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/FR-006", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TC-006", type: references }
---
# SR-036: Rust review — finite topology and anchored-boundary evidence

## Summary

The #13 Rust diff was reviewed against QObs main `8d54458`, FR-006/TC-006 and
the accepted owner boundary. The diff adds only real integration evidence and
one test-support constant for an existing pinned FCD declaration. No production
Rust, public API, owner schema or identity preimage changes because the tests
confirmed the required behavior was already implemented.

## Verdict

**PASS** — no open Rust-idiom, test-quality, boundary, safety, resource or
specification-faithfulness finding remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved finding | - |

## Resolved during review

| Former finding | Resolution | Refs |
| --- | --- | --- |
| The initial overflow case also failed the closed-watermark rule, so it did not isolate checked successor overflow | Made the overflow boundary open; successor overflow is now the sole invalidity | `tests/topology_boundary.rs` |
| The initial foreign-revision case cross-wired the progress clock and cutoff as well as the qualified history | Parameterized the cutoff revision so the selected foreign revision remains internally consistent and only mismatches qualified authority | `tests/topology_boundary.rs` |
| The topology assertion proved the three supplied edges individually but made absence of a synthetic fourth edge indirect | Compare the complete retained `(identity, source, target)` set to one exact expected set | `tests/topology_boundary.rs` |
| The first acceptance table used an unregistered compound verification-method spelling | Restored the catalogued `Test (TC-006)` method and retained public-surface review evidence here | FR-006 |

## Rust and test review

- All topology inputs pass through the real pinned FCD
  `StaticProducerBundle::admit_json` capability and the production
  `Relationship::new`/`admit` paths. `Order-successor` is an existing exact
  declaration in the pinned fixture; no mock producer or test-only bypass exists.
- The exact retained edge set contains one self-loop and both directions of a
  separate two-vertex component. Input replay and permutation plus record time
  mutations cannot add, remove or rewrite an edge.
- The offered-population test deliberately supplies two replay copies against a
  ceiling of one and observes resource refusal before deduplication.
- Boundary tests call real admission and progress/closure derivation for all
  three clock families. Each adverse case isolates family, start, end, scope,
  identity, revision, watermark, reversal or overflow as intended.
- Test helpers derive real observation/membership/population identities before
  admission; no caller-authored stale identity is accepted for convenience.
- The diff adds no production unwrap/panic, unsafe code, unchecked conversion,
  async/blocking/lock/filesystem/environment behavior, allocation loop, warning
  suppression, skip, `should_panic`, mock, stub or test-only product switch.
- The public production surface is unchanged. It still contains no graph
  traversal, reachability, transitive closure, replay evaluator, settlement,
  result or truth API.

## Gates

- `cargo fmt --all -- --check` — PASS.
- `cargo check --workspace --all-targets --all-features --locked --target-dir target-codex-backends --quiet` — PASS.
- `cargo clippy --workspace --all-targets --all-features --locked --target-dir target-codex-backends --quiet -- -D warnings` — PASS.
- `cargo test --workspace --all-targets --all-features --locked --target-dir target-codex-backends --quiet` — PASS: 11 unit, 14 admission, 17 authority and 4 TC-006 tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked --target-dir target-codex-backends --quiet` — PASS.
- `cargo deny 0.19.8 check` — PASS: advisories, bans, licenses and sources.
- `git diff --check` — PASS; `src/`, `schemas/` and `resources/native-v1/` have no diff.
