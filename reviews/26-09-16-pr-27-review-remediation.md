---
id: SR-054
title: "PR-27 review remediation — activation ownership and bounded authority"
type: SpecReview
analysis: code-review
scope: "PR #27 owner-stage delta after independent semantic, Rust, and resource-limit review"
review_set: all
relationships:
  - { target: "ix://agent-ix/quire-observation/Plan-002", type: reviews }
  - { target: "ix://agent-ix/quire-observation/Task-017", type: reviews }
  - { target: "ix://agent-ix/quire-observation/Task-022", type: reviews }
  - { target: "ix://agent-ix/quire-observation/FR-008", type: reviews }
  - { target: "ix://agent-ix/quire-observation/NFR-003", type: reviews }
---
# SR-054: PR-27 review remediation — activation ownership and bounded authority

## Summary

Three independent full-PR reviews of draft PR #27 found one semantic ownership gap, one exact
multi-role selection gap, one escaped-key preflight bypass, late strict-read preflight, and four
under-accounted work/state paths. The owner-stage delta now derives activation state from strict
authority inputs, resolves every selected multi-role fact by its exact component identity, decodes
fixed JSON keys allocation-free before collection preflight, and accounts for dependency scans,
support lookup, activation record lookup, and incremental prefix allocation before retention.

## Verdict

**PASS (owner-stage remediation); FAIL (overall Plan-002)** — all local review blockers are
remediated and the local owner artifact is ready for a new exact-head review. Task-023 pinned real
consumer integration and Task-024 final promotion remain `not_started`; neither this review nor a
staged owner merge completes issue #23 or the C00 plan.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Removed caller-selected activation state. An admitted trigger plus matching strict capture authority derives `active`; a trigger-absent closed/complete scope derives `inactive`; every other trigger-absent scope derives `unknown`. | FR-008, TC-008 Cartesian oracle, `activation::derive` |
| FND-002 | medium | Reconciled FR-008 success/error semantics: semantic unknown and missing-source coverage are valid explicit incomplete facts, not refusal paths. | FR-008 Error Conditions, TC-008 |
| FND-003 | medium | Reconciled NFR-003 matrix status to complete. | TM-001, TC-010, TC-012, TC-013 |
| FND-004 | high | Exact role-plus-component resolution now carries the selected embedded payload into query evaluation; either of two activation facts is independently queryable and a foreign selected component refuses. | `query::selected_fact`, `tc011_exact_multi_activation_selection_never_uses_an_unselected_payload` |
| FND-005 | high | Fixed-key matching decodes JSON escapes without allocation, so escaped spellings cannot bypass any population, position, capture, source, bundle, replacement, conflict, or lineage-child ceiling. | `common::json_key_matches`, eight-family escaped-key unit test |
| FND-006 | high | Public partial, activation, and bundle readers byte-preflight before semantic derivation. | TC-007, TC-008, TC-009 byte-first one-over cases |
| FND-007 | high | Coordinator dependency preparation charges every edge scan; support validation prechecks its evidence index and uses a charged binary lookup over sorted inputs. | TC-010 exact/one-over work/state evidence |
| FND-008 | high | Activation indexes the bounded qualified history once and uses indexed trigger, capture-source, verdict, and settlement support lookup. | TC-008 exact/one-over `max_population_entries` evidence |
| FND-009 | medium | Incremental prefix capacity is computed from a borrowed evaluator outcome and checked before cloning or terminal-state promotion. | TC-010 rollback and exact/one-over state evidence |

## Revision-bound resource evidence

TC-013 now inventories 39 scoped expansion/allocation paths instead of 28. The added rows bind
escaped-key classification, preflighted strict deserialization, all three byte-first readers, activation record indexing, charged
dependency preparation, support-index allocation, binary support lookup, prefix preallocation,
and exact multi-role query selection. Current reviewed SHA-256 values are:

| Source | SHA-256 |
| --- | --- |
| `src/authority/common.rs` | `379bcf5cd4bf014cb0b4633bc76b188e4aed308f2532db3fd5028d99a2d870bd` |
| `src/authority/partial.rs` | `33de9f39f3e422bf2d364ceba7e302101b230d4fe6a3ce8d4f6ce4cf8a0e8237` |
| `src/authority/activation.rs` | `2d9be2340233051555eeb8f7cb63a3ceb0de47687c1290b1adf2f8a4a3345034` |
| `src/authority/bundle.rs` | `25081ee72b4ac4aefc2d26972afb095d1b84f6dcb9c6900a2a331d727f44404a` |
| `src/authority/coordination.rs` | `309268d9b80a148d6bceff78f7c2a81a63934d4797d05f54224aff70a4594d54` |
| `src/authority/query.rs` | `26eaaf49db4113d5c011bf474a496e189351e9274de01be911ac8f1d82ec612f` |
| `tests/partial_interval.rs` | `711adfb64cfb0827dd9d5a62ad65081fc1321eb5d6089d660b8bd6e33e1a1d59` |
| `tests/activation_authority.rs` | `a2aa964d30448d069cf3333dfcbce022eab2cf6753f9d70a9a8e7d6e9d4bdd4c` |
| `tests/authority.rs` | `ddd7109d13cbbb996cfc2bedd90977ef095329af8473c1b4e98ad5a1161296b5` |

The activation/scope-authority schema digest is
`0a6c26385d9f64166b72e939315bfe4accfdd125f9354e22ddf127023eaf1823`.

## Remaining gates

- Task-023 must run pinned QSL, TL, Protocol, and composed-runtime consumers against the accepted
  owner revision and record IT-002.
- Task-024 must rerun the unchanged-head local and consumer gate, reconcile the plan, and decide
  final C00 promotion. The current plan remains 7/9 complete.

## Validation evidence

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS.
- `cargo test --locked --all-targets --all-features -- --test-threads=1`: PASS, 115 tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps`: PASS.
- `cargo deny check --disable-fetch`: PASS (`advisories`, `bans`, `licenses`, `sources`) using a
  writable temporary copy of the existing advisory database because the shared cache is read-only.
- Native `quoin` 0.23.1-87-gb3d02d4, `quoin validate --strict --repo . --json`: PASS, no findings.
- `quire` 0.32.0, `quire coverage --scope . --strict --json`: PASS, 91/91 backed and zero status
  lies. Five pre-existing `IT-001` tags remain untracked and do not substitute for IT-002.
- `cargo test --locked --test bounded_work`: PASS, 2/2 revision-bound checks.
- `git diff --check`: PASS.
