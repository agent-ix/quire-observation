---
id: SR-062
title: "code-review review of IR-1390 structural association identity"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-observation; IR-1390; src/authority/progress.rs, tests/authority.rs, schemas/observation-progress-assertion-v1.schema.json, spec/functional/FR-004-publish-observation-authority-artifacts.md, spec/functional/FR-006-preserve-finite-topology-and-anchored-boundaries.md, spec/test-cases/TC-006-finite-topology-and-anchored-boundaries.md"
review_set: subset
---

## Summary

Ticket: IR-1390. Reviewed the Rust codec boundary, typed claim, schema, and authority test on frozen PR #49. The encode constructor retains an over-limit string before refusal.

## Verdict

**CONDITIONAL** — Reviewed the Rust codec boundary, typed claim, schema, and authority test on frozen PR #49. The encode constructor retains an over-limit string before refusal.

## Scope

- FR-004-AC-4 (examined): Exact and one-over byte/depth/string/population/position/capture/source/work bounds admit or resource-refuse before excess retention.
- FR-004-AC-7 (examined): A progress view exposes owner asserted QSL association rows as a canonical value when present. Each row names the selected model, structural model node, typed claim clause node and occurrence, time source kind, tick operation and exact positive period when applicable, structural unit and dimension nodes, and an observation source and clock/revision admitted in the same progress context. Missing rows refuse with `MissingPremise`; foreign source/clock refuse with `ExpectedMismatch`; malformed or duplicate rows refuse with `InvalidSelection`, noncanonical structural node text with `NonCanonical`, and one-over node or value bytes with `ResourceIncomplete`. Strict reading compares the exact selected rows. The observation owner asserts these links; QSL checks its own semantic selections independently against the rows.
- FR-006-AC-9 (examined): Association model, clause, tick-operation, unit and dimension nodes round-trip as real structural NodeKeys with canonical package/path/ordinal text; the claim retains its exact role and ordinal. Empty package/path, malformed segment or out-of-range ordinal refuses with `InvalidSelection`, an alternate spelling with `NonCanonical`, and one-under node-byte bounds with `ResourceIncomplete`; an exact byte bound admits.
- FR-006-AC-8 (context only): The prerelease progress v1 contract includes FR-004's owner-issued association because no external progress v1 consumer exists; its exact FR-289 boundary mapping remains intact, closure v1 retains its boundary mapping, and neither publishes a traversal, replay, settlement or truth API.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | NodeIdentity::from_key encodes the entire caller-supplied NodeKey before checking max_string_bytes or max_output_bytes. A structurally valid key with a very large package or path retains an over-limit canonical String before returning ResourceIncomplete, contrary to FR-004-AC-4 pre-retention bounds. Bound the encode operation itself. | src/authority/progress.rs:52-60 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The new pre-retention guard `check_key_size` has no test of its own exact boundary. `key_at_the_exact_limit_is_admitted_and_one_under_is_refused` uses a key whose path text contains `"`, and sets `max_string_bytes` to the JSON-escaped length, which is longer than the raw canonical text. The refusal at one under therefore comes from the existing `check_size`, not from the new guard, and the test also passes on the pre-fix code. Mutating the guard's budget to `min(..) - 3`, so it wrongly refuses valid keys at their exact bound, leaves every test in the crate green. Add a key with no escapes, set `max_string_bytes` to its canonical length (it must be admitted) and to that length minus one (it must be refused with ResourceIncomplete). | tests/node_identity_bound.rs:36-52; src/authority/progress.rs:98-159 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit 1c4c459 calls `check_key_size` before `node_key_to_text`. The guard writes the exact canonical layout into a counting writer capped at min(max_string_bytes, max_output_bytes) and refuses with ResourceIncomplete once the cap is passed, so an over-limit key never builds its canonical String. A doubling key of about 2^48 bytes is refused under the new code; under the old code (with `ulimit -v 4000000`) the same test goes red, InvalidSelection versus ResourceIncomplete. |
| FND-002 | fixed | Commit e679fc3 adds the unit test `key_guard_admits_exact_canonical_length_and_refuses_one_under` (src/authority/progress.rs:968). It calls `check_key_size` directly on a key with no escapes and checks both `max_string_bytes` and `max_output_bytes`: the exact canonical length is admitted and that length minus one is refused with ResourceIncomplete. It also adds the integration test `key_without_escapes_is_admitted_at_the_exact_output_bound` (tests/node_identity_bound.rs:56). Budget mutations of -1 and +1 at progress.rs:117 both fail the unit test, and -1 also fails the integration test. The coder is right that the exact `max_string_bytes` limit cannot be reached through `from_key`: canonical text always contains `"`, so the escaped-length check in `check_size` refuses first. Testing the guard directly is therefore the right oracle. |
