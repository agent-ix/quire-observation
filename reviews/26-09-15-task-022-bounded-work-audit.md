---
id: SR-052
title: "Bounded-work audit — Plan-002 C00 expansion paths"
type: SpecReview
analysis: code-review
scope: "src/authority/{common,partial,activation,bundle,repair,coordination,query}.rs; tests implementing TC-012 and TC-013"
review_set: subset
relationships:
  - { target: "ix://agent-ix/quire-observation/Task-022", type: reviews }
  - { target: "ix://agent-ix/quire-observation/NFR-003", type: reviews }
  - { target: "ix://agent-ix/quire-observation/TC-012", type: references }
  - { target: "ix://agent-ix/quire-observation/TC-013", type: references }
---
# SR-052: Bounded-work audit — Plan-002 C00 expansion paths

## Summary

Every FR-007 through FR-011 traversal, retained collection, possible-order representation,
arithmetic fold and serialization path was mapped to a finite caller-lowered limit. The audit is
revision-bound by `tc013_each_c00_expansion_path_names_its_dominating_bound_and_evidence`, whose
28 scoped rows pin the complete SHA-256 of each reviewed production and evidence source and bind
the entry point, expansion anchor, retained state, dominating limit, precheck or charge, checked
arithmetic or conversion, and either matching TC-012 exact/one-over evidence or an explicitly
typed structurally dominated classification.

## Verdict

**PASS** — no unbounded recursive traversal, factorial order materialization, unchecked integer
cast, unbounded retry, or collection growth without a dominating finite limit remains in the C00
surface. TC-012 supplies exact/one-over and reproducibility evidence where an owner limit is
directly lowerable; TC-013 binds those tests and the two structural-only rows to the same source
revision.

## Inventory

| Family | Revision SHA-256 | Expansion/allocation paths | Dominating bound and fail-closed evidence |
| --- | --- | --- | --- |
| shared owner envelope/scanner | `333c09bb75e0e5d09b4d9ca07883a156a58950df2241e555884b3891829e3cd3` | JSON input scan, canonical writer, depth/string/collection counters | `max_input_bytes`, `max_output_bytes`, `max_depth`, `max_string_bytes`, semantic collection maxima and `max_visited_fields`; checked addition and fallible writer reserve; exact/one-over TC-004/TC-012 |
| partial value/interval | `3c6ca38ac6e8d9760f460750d8e58cf9240c713556c10b6798f8ff8c08ac02bc` | fixed-cardinality possibility and three-relation interval result; owner serialization/strict read | shared owner byte/depth/string/work limits; repeated canonical bytes and output refusal in TC-007/TC-012 |
| activation/scope authority | `d647da2616d54361d882cef5f22901e7de105f699575827ad0f3aa36c19b9873` | capture proof copy, capture validation/sort, required/covered source sets, contribution support | `max_capture_bindings`, `max_required_sources`, shared input/output/depth/string/work limits; count checks precede fallible reserves; canonical and exact/one-over TC-008/TC-012 |
| bundle and lineage | `7dc218a87bceded46e65f90b4b272316cd2acb80aa1eab46ca77a5622bc553a2` | component projection, replacements/conflicts, direct-child lineage sort/scan, linear replacement merge, canonical output | `max_bundle_components`, `max_replacements`, `max_conflicts`, `max_lineage_children`, shared input/output limits; TC-009/TC-012 exact/one-over and permutation evidence |
| repair planner | `cd6ee655b4e2503325e8787bd6f1b87b615b0c335f917473e4c749fd302cae53` | graph indexing, acyclic check, changed-fact breadth-first closure, affected topological schedule, retained prior results and dependencies | `max_nodes`, `max_edges`, `max_work`, `max_retained_results`, `max_retained_bytes`, `max_output_bytes`; visited `(node, changed-fact)` pairs terminate closure; all dependency bytes are precharged before fallible reserve/clone; TC-010/TC-012 exact/one-over and generated DAG oracle |
| evaluator coordination | `0992c4385204fac024012b4a68fdba94c10096dbce5c4b0568e830f4ee936e24` | job/input preparation, incremental prefix state, result dataflow, pairwise interval orders, evaluator output and unaffected-result retention | `max_jobs`, `max_inputs`, `max_possible_orders`, `max_work`, `max_state_bytes`, `max_output_bytes`; result inputs are charged for each retained copy, evaluator bytes are preflighted, and failed attempts roll back retained state only while preserving cumulative work/order/call usage; TC-010/TC-012 batch/incremental exact/one-over and arrival permutations |
| closed-population query | `62f421f6fc0b1910d73f574872973a6023a97670f1b970ad5a28e6f40e30d61a` | authority projection scans, member join/insertion order, duplicate fold, participation copy, filter/count/exact-sum fold, canonical result | `max_input_bytes`, `max_members`, `max_arithmetic_steps`, `max_work`, `max_state_bytes`, `max_output_bytes`; every scan/comparison/fold is metered, state is charged before allocation/retention and arithmetic uses checked prefixes; TC-011/TC-012 exact/one-over and independent generated folds |

## Expansion-shape findings

- Repair closure is iterative breadth-first traversal over a finite explicit graph. Its visited key
  is the Cartesian pair of an admitted graph node and one bounded changed fact; it cannot recurse
  or discover an ambient node.
- Repair scheduling and acyclic validation are iterative Kahn traversals over the already bounded
  explicit result/edge sets. Checked indegrees and one-visit ready sets prevent retry growth.
- Coordinator interval uncertainty is represented as at most three pairwise relations for each
  unordered admitted input pair. It never enumerates total orders or factorial permutations.
- Query exposure ordering uses a metered insertion pass over at most `max_members`; semantic
  deduplication uses a metered bounded prefix scan. Neither loop creates additional members.
- The shared JSON scanner has exactly two open-form parser loops. Both consume at least one input
  byte per iteration or return, and every member/element charges `max_visited_fields`; TC-013 pins
  these two loops explicitly rather than applying an unsound blanket loop ban.
- All reviewed production modules contain no `unsafe`, production `unwrap`/`expect`, unchecked
  integer cast, or retry `loop`. The two `expect` calls in `common.rs` are cfg(test)-only schema
  and in-memory formatting assertions.

## Resolved production gaps

- Bundle lineage construction now checked-adds the head and every direct-child document byte,
  rejects the aggregate above `max_input_bytes`, and rechecks supplied lineage count/input limits
  at the current call. Replacement comparison is a linear merge over bounded sorted component
  lists, with all four retained vectors fallibly reserved from admitted counts.
- Repair dependency retention now computes dependency count and byte totals with checked
  arithmetic, charges `max_retained_bytes`, and only then reserves and clones the selected edges.
- Coordinator accounting now includes every retained result-input copy, preflights evaluator bytes
  before promotion, charges unaffected result bytes before cloning, and preserves cumulative
  work/possible-order/evaluator-call usage across failed incremental attempts while rolling back
  only state that was not retained.

## Revision-bound evidence

| Evidence source | SHA-256 | TC-012 owner evidence |
| --- | --- | --- |
| `tests/partial_interval.rs` | `7c2b8fe627b42c6631bb4c1ca12b0e5a4e702d5f59d78e097b4e54a9c347b230` | `tc007_partial_fact_is_canonical_strictly_read_and_fail_closed` |
| `tests/activation_authority.rs` | `967a3774066b12ffd70812306aacfd14e820c97423849fb9ddc16b4a7bbf1032` | `tc008_versioned_owner_round_trips_all_independent_authority` |
| `tests/authority.rs` | `f08b65fcbaa029494b1c5ac4706d904ec66d73ed07268c6afac99e09c3486acd` | TC-009 bundle, TC-010 repair/coordinator, and TC-011 query exact/one-over tests |

The qualified-history lookup and constructor-private `AuthorityProofs` copy are marked as
structurally dominated rows rather than falsely claiming their own lowered-limit oracle. The
history lookup is bounded by admission's finite `max_records`; proof-copy cardinality originates
from strict-read owner views. Every other row requires its cited TC-012 function to contain the
row's named limit (where applicable), an exact-bound admission, and a one-over refusal.

## TC-012 evidence map

- FR-007: canonical repeated partial facts and fail-closed bounded output.
- FR-008: byte-identical activation derivation/read across all independent axes; exact/one-over
  input, output, depth, string, work, capture and required-source limits.
- FR-009: generated component permutations/replays plus component/replacement/conflict/lineage,
  strict-read input and canonical-output exact/one-over cases.
- FR-010: generated DAG and arrival permutations; planner and batch/incremental coordinator
  exact/one-over nodes, edges, jobs, inputs, possible orders, work, state and output.
- FR-011: generated independent membership/effect folds and exact/one-over input, members,
  occurrences, arithmetic, work, retained state and output.

## Gates

- `cargo test --locked --test bounded_work`: PASS.
- `cargo test --locked --all-targets --all-features`: PASS (111 tests).
- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS.
- `RUSTDOCFLAGS=-Dwarnings cargo doc --locked --all-features --no-deps`: PASS.
- `cargo deny check`: PASS.
- Native `quoin validate` (`quoin 0.23.1-87-gb3d02d4`): PASS, no findings.
- Fallback `quire coverage --scope . --json`: PASS, 89/89 backed and zero status lies.
