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
A subsequent exact-head review found that caller omission still stood in for trigger absence,
historical bundle reads could construct lineage before byte preflight, and two coordination
allocations were not dominated by their full retained-state checks. Those second-order findings
are now included in this remediation rather than treated as review-only follow-up.

## Verdict

**PASS (owner-stage remediation); FAIL (overall Plan-002)** — all local review blockers are
remediated and the local owner artifact is ready for a new exact-head review. Task-023 pinned real
consumer integration and Task-024 final promotion remain `not_started`; neither this review nor a
staged owner merge completes issue #23 or the C00 plan.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Removed caller-selected activation state and caller-asserted absence. Every selection carries the exact qualified binding and semantic trigger; activation and progress reject foreign binding triggers. Absence is exercised by an empty optional binding and rejects matching admitted history. Required empty bindings return `MissingPremise`. Empty-history progress and no-trigger activation identities commit the exact binding, so same-trigger/different-binding bytes cannot replay. | FR-008, TC-008 Cartesian and negative-control oracles, `activation::derive`, `progress::derive` |
| FND-002 | medium | Reconciled FR-008 success/error semantics: semantic unknown and missing-source coverage are valid explicit incomplete facts, not refusal paths. | FR-008 Error Conditions, TC-008 |
| FND-003 | medium | Reconciled NFR-003 matrix status to complete. | TM-001, TC-010, TC-012, TC-013 |
| FND-004 | high | Exact role-plus-component resolution now carries the selected embedded payload into query evaluation; either of two activation facts is independently queryable and a foreign selected component refuses. | `query::selected_fact`, `tc011_exact_multi_activation_selection_never_uses_an_unselected_payload` |
| FND-005 | high | Fixed-key matching decodes JSON escapes without allocation, so escaped spellings cannot bypass any population, position, capture, source, bundle, replacement, conflict, or lineage-child ceiling. | `common::json_key_matches`, eight-family escaped-key unit test |
| FND-006 | high | Public partial, activation, and bundle readers byte-preflight before semantic derivation. | TC-007, TC-008, TC-009 byte-first one-over cases |
| FND-007 | high | Coordinator dependency preparation charges every edge scan; support validation prechecks its evidence index and uses a charged binary lookup over sorted inputs. | TC-010 exact/one-over work/state evidence |
| FND-008 | high | Activation indexes the bounded qualified history once and uses indexed trigger, capture-source, verdict, and settlement support lookup. | TC-008 exact/one-over `max_population_entries` evidence |
| FND-009 | medium | Incremental prefix capacity is computed from a borrowed evaluator outcome and checked before cloning or terminal-state promotion. | TC-010 rollback and exact/one-over state evidence |
| FND-010 | medium | Historical revision readers now byte-preflight before predecessor lineage construction and reuse the observed usage in the strict reader. | TC-009 historical one-over negative control, `bundle::read_revision_for` |
| FND-011 | high | Terminal outcome clones are dominated by full outcome-state capacity checks; decisive replacement retained size is counted before canonical wire/result allocation. | TC-010 rollback evidence, TC-013 ordering assertions |
| FND-012 | medium | Multi-activation selection evidence now pairs two active payloads that differ in progress authority, so payload substitution changes a complete query into a foreign-authority refusal. | TC-011 exact multi-role oracle |
| FND-013 | medium | TC-013 now revision-pins progress, inventories the bounded trigger-absence scan, and asserts byte-preflight-before-lineage plus capacity-guard-before-replacement ordering. The historical negative control pairs oversized bytes with a missing predecessor so the old order fails observably. | TC-009, TC-013 |
| FND-014 | high | Empty required bindings no longer mint inactivity and refuse as a missing premise, preserving the V1 schema's closed/complete classification invariant. Empty-history progress and absent-activation identity preimages commit the exact binding without changing the immutable V1 schema shape or the established active-history V1 bundle fixture. Direct foreign-trigger and same-trigger/different-binding proof-substitution oracles exercise each check independently. | FR-008, TC-008 |
| FND-015 | high | Capture-present proof composition now accepts only the established captured-history V1 progress identity mode. An empty-history V2 progress view cannot shed its binding commitment by entering `AuthorityProofs::new`; the exact same-trigger foreign-binding negative control refuses at composition. | FR-008, TC-008, `AuthorityProofs::new` |

## Revision-bound resource evidence

TC-013 now inventories 44 scoped expansion/allocation paths instead of 28. The added rows bind
escaped-key classification, preflighted strict deserialization, all three byte-first readers, activation record indexing, charged
dependency preparation, support-index allocation, binary support lookup, prefix preallocation,
progress proof-identity reconstruction, and exact multi-role query selection. Current reviewed SHA-256 values are:

| Source | SHA-256 |
| --- | --- |
| `src/authority/common.rs` | `379bcf5cd4bf014cb0b4633bc76b188e4aed308f2532db3fd5028d99a2d870bd` |
| `src/authority/partial.rs` | `33de9f39f3e422bf2d364ceba7e302101b230d4fe6a3ce8d4f6ce4cf8a0e8237` |
| `src/authority/activation.rs` | `776e82b5a4cd21480079452e4d62e814626f8047d217642bceb9390f8ce5f2a5` |
| `src/authority/progress.rs` | `b0376f42974c5cdc2b21f5c92accf785f0feb6be65077139fc8891cbe4609594` |
| `src/authority/bundle.rs` | `752c833a4103edca25f9e3ed029dce9fcaf1a98e5b701a12ab9c8e71067f0d2e` |
| `src/authority/coordination.rs` | `5be19d0e20d935f75534fbaa19b17614b118e3952969768366d2a7be1207eec5` |
| `src/authority/query.rs` | `26eaaf49db4113d5c011bf474a496e189351e9274de01be911ac8f1d82ec612f` |
| `tests/partial_interval.rs` | `711adfb64cfb0827dd9d5a62ad65081fc1321eb5d6089d660b8bd6e33e1a1d59` |
| `tests/activation_authority.rs` | `f07bd4e2f90aea998af8a3d33a3d3e315795f6fe0dc0294a7083ae1d2fa17d76` |
| `tests/authority.rs` | `5ceab5286f5e138d81d9bc617bb62ba779d833fc9fe033ab7cdd4f2223315edc` |

The activation/scope-authority schema digest is
`c17ad8b3ec10c48e535bc3a762738b979206c01af20c31775d8a9a9e0a97fab6`.

## Remaining gates

- Task-023 must run pinned QSL, TL, Protocol, and composed-runtime consumers against the accepted
  owner revision and record IT-002.
- Task-024 must rerun the unchanged-head local and consumer gate, reconcile the plan, and decide
  final C00 promotion. The current plan remains 7/9 complete.

## Validation evidence

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS.
- `cargo test --locked --all-targets --all-features -- --test-threads=1`: PASS, 117 tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps`: PASS.
- `cargo deny check --disable-fetch`: PASS (`advisories`, `bans`, `licenses`, `sources`) using a
  writable temporary copy of the existing advisory database because the shared cache is read-only.
- Native `quoin` 0.23.1-87-gb3d02d4, `quoin validate --strict --repo . --json`: PASS, no findings.
- `quire` 0.32.0, `quire coverage --scope . --strict --json`: PASS, 91/91 backed and zero status
  lies. Five pre-existing `IT-001` tags remain untracked and do not substitute for IT-002.
- `cargo test --locked --test bounded_work`: PASS, 2/2 revision-bound checks.
- `git diff --check`: PASS.
