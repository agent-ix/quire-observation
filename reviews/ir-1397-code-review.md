---
id: SR-067
title: "code-review and rust-review of IR-1397 retained association value"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-observation; IR-1397; PR #50 head 1921c42d8ddebb378c6d7642872e2af707fa82a9; src/authority/common.rs, src/authority/progress.rs, tests/authority.rs, spec/functional/FR-004-publish-observation-authority-artifacts.md"
review_set: subset
---

## Summary

Ticket: IR-1397. Code review with the rust-review lane folded in, over `git diff origin/main...HEAD` of PR #50. Strict progress reading now encodes the association value once through the existing bounded `to_bounded_json` sink, retains it as `Option<Box<[u8]>>` (`#[serde(skip)]`), and lends it through `Validated<ProgressView>::association_canonical_bytes() -> Result<&[u8]>`. Association struct fields were reordered to lexical wire-name order so the streaming serializer emits RFC 8785 member order.

## Verdict

**CONDITIONAL** — the production change is correct. The streaming encoding really is RFC 8785 for every value shape in this closed domain, not only for the tested shapes. Every leaf is a JSON string: `NodeIdentity` serializes as a str, the ordinal and the period are `String`, and there are no maps and no numbers. The internally tagged `kind` is emitted first, and it sorts before every Tick field name. serde_json escaping matches RFC 8785: lowercase `\u00xx`, short escapes for `\b \t \n \f \r`, and no escaping of `/` or non-ASCII. Neither `preserve_order` nor `arbitrary_precision` is enabled in the dependency graph, so the `to_value` oracle really sorts. The two-shape oracle test covers both `TimeSource` variants, which is every shape the domain has, and the node texts in its fixture contain characters that need escaping. The retained slice length equals the exact copy charge. Two medium test-intent gaps remain against AC1 and AC3: the public one-over assertion is vacuous for the association bound, and `ptr::eq` is not a no-allocation assertion. Focused checks passed: `progress::tests` (4) and `tc004_progress_association_is_canonical_and_bound_to_admitted_context`.

Wire contract: the reorder changes the bytes and identities of progress documents that carry an association, because the document canonical form is declaration order and `read_exact` compares the bytes exactly. FR-006-AC-8 records that no external progress v1 consumer exists. In /home/peter/dev, quire-mltl and quire-protocol do not use associations. QSL #2489 (IR-1383) derives its documents at runtime and has no hard-coded association goldens or identities. It still calls the removed `association_canonical_bytes(limits)` at qsl-analyze/src/monitor_root/progress.rs:502 and tests/it/monitor_authority.rs:104,386, so it must adapt to the new signature. That is expected downstream work, not a defect in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The public one-over assertion does not exercise the association bound. `read_exact` already encodes the whole envelope under the same `max_output_bytes` (common.rs:1091), and the document carries the same rows plus more fields than the value's roughly 20 extra key bytes. So `read(..., tight)` with `tight = value_len - 1` is refused by the document ceiling first. Mutating `encode_association` to `to_bounded_json(value, usize::MAX)` leaves this integration test green. The refusal inside `read` (progress.rs:898-901) cannot be reached through the public API. Relabel the assertion as a document-bound check, or delete it. The unit boundary test is the only real evidence. | tests/authority.rs:934-942; src/authority/progress.rs:549-552,898-901 |
| FND-002 | medium | AC3 asks for a no-allocation assertion, and `ptr::eq` across two getter calls is not one. A lazy implementation, for example a `OnceCell` filled on the first getter call, allocates on that call and still passes, and so does moving the encoding out of admission. allocation-counter 0.8.1 is already a dev-dependency in quire-exact (thread-local counters; the crate owns the global allocator, so `#![forbid(unsafe_code)]` here is unaffected). It can assert zero allocations across `association_canonical_bytes()`. | tests/authority.rs:843-847; src/authority/progress.rs:536-545 |
| FND-003 | low | `ProgressView` derives `PartialEq`/`Eq` over the new `association_value` field. A strictly read view (`Some`) and a `ProgressView` of the same document that was derived or decoded from a bundle (`None`, from `#[serde(skip)]`) compare unequal. No comparator exists today. A later equality check between a read view and a bundle-embedded payload would refuse identical documents. | src/authority/progress.rs:443-445; src/authority/bundle.rs:1722 |

## Examined scope

```yaml
{
  "scope": [
    {"id": "IR-1397-AC1", "path": "src/authority/progress.rs", "role": "examined", "excerpt": "admission computes the canonical association value once, refusing with a typed resource error before retaining anything over the owner byte ceiling (exact admits, one over refuses)"},
    {"id": "IR-1397-AC2", "path": "src/authority/progress.rs", "role": "examined", "excerpt": "borrowed getter returns retained canonical bytes with no allocation, length = exact copy charge"},
    {"id": "IR-1397-AC3", "path": "tests/authority.rs", "role": "examined", "excerpt": "value-asserting tests, exact-ceiling boundary test, no-allocation assertion, spec lines in same PR; no full-document mirror, no second encoder"},
    {"id": "FR-004-AC-7", "path": "spec/functional/FR-004-publish-observation-authority-artifacts.md", "role": "examined", "excerpt": "a value of exactly the owner output byte ceiling is admitted and one byte over is refused before any over-ceiling bytes are retained."},
    {"id": "FR-006-AC-8", "path": "spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md", "role": "context_only", "excerpt": "The prerelease progress v1 contract includes FR-004's owner-issued association because no external progress v1 consumer exists"}
  ]
}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit bccf80ac labels the public `tight` read at tests/authority.rs:939-940 as a document-bound refusal: "The document always carries the value, so this one-under ceiling is refused by the document bound; the value-encoder bound has its own unit test." The unit test `encoding_admits_the_exact_ceiling_and_refuses_one_byte_over` remains the evidence for the value boundary. |
| FND-002 | fixed | Commit bccf80ac adds the allocation-counter 0.8.1 dev-dependency (Cargo.toml and Cargo.lock) and wraps the getter in `allocation_counter::measure`, asserting `count_total == 0` (tests/authority.rs:844-851). Mutation check: with one `Vec::with_capacity(1)` added to the getter, the test fails at tests/authority.rs:848, so the assertion catches an allocating getter. |
| FND-003 | fixed | Commit bccf80ac replaces the derived equality with a manual `PartialEq`/`Eq` that covers all 11 wire fields and excludes `association_value` (progress.rs:448-466). The new test `equality_ignores_the_retained_association_value` checks that a strict view equals its decoded copy and that changing `state` makes them unequal. |
