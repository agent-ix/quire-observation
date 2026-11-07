---
id: SR-068
title: "gap-analysis of IR-1397 retained association value"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-observation; IR-1397; PR #50 head 1921c42d8ddebb378c6d7642872e2af707fa82a9; spec/functional/FR-004-publish-observation-authority-artifacts.md, spec/tests.md, spec/test-cases/TC-004-observation-authority-artifacts.md, src/authority/progress.rs, tests/authority.rs"
review_set: subset
---

## Summary

Ticket: IR-1397. Scoped gap analysis of the PR diff against the ticket ACs and FR-004-AC-7. `quoin matrix --repo .` (quoin 0.29.4) shows FR-004-AC-7 as tagged, with no run evidence bound, which is the same as every other FR-004 criterion. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — the code has no stub, no second encoder and no full-document mirror. The retained value is produced by the existing `to_bounded_json`. The new AC-7 clause promises an exact-ceiling admission through strict reading, and strict reading cannot produce that state. The hand-written trace table also omits AC-7.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-004-AC-7 now says "a value of exactly the owner output byte ceiling is admitted and one byte over is refused". Strict reading applies the same `max_output_bytes` to the whole document first, and the document holds the same rows plus envelope, limits and boundary fields. So a value at the ceiling can never be admitted, and its one-over refusal never comes from the value bound. The AC states behavior that no public test can show; only the private `encode_association` test shows it. Reword the clause to bound the value's encoding and say the document ceiling dominates, or give the value its own ceiling. | spec/functional/FR-004-publish-observation-authority-artifacts.md:148; src/authority/progress.rs:1070-1089 |
| FND-002 | low | spec/tests.md still maps FR-004 and TC-004 to FR-004-AC-1..FR-004-AC-6, although this PR edits AC-7 and adds two tests tagged FR-004-AC-7. The TC-004 procedure also has no association-value or ceiling step. The gap predates this PR, but this PR touches the AC. | spec/tests.md:40,64; spec/test-cases/TC-004-observation-authority-artifacts.md |

## Examined scope

```yaml
{
  "scope": [
    {"id": "FR-004-AC-7", "path": "spec/functional/FR-004-publish-observation-authority-artifacts.md", "role": "examined", "excerpt": "one-over node or value bytes with `ResourceIncomplete`; a value of exactly the owner output byte ceiling is admitted and one byte over is refused before any over-ceiling bytes are retained."},
    {"id": "FR-004", "path": "spec/functional/FR-004-publish-observation-authority-artifacts.md", "role": "examined", "excerpt": "Strict reading encodes the value once into a bounded sink that refuses with `ResourceIncomplete` as soon as its length passes the owner output byte ceiling, and the view retains those bytes. A borrowed getter returns the retained bytes without allocating; its length is the exact charge for a copy."},
    {"id": "TC-004", "path": "spec/test-cases/TC-004-observation-authority-artifacts.md", "role": "examined", "excerpt": "Complete owner-artifact checks for FR-004."},
    {"id": "FR-004-AC-4", "path": "spec/functional/FR-004-publish-observation-authority-artifacts.md", "role": "context_only", "excerpt": "Exact and one-over byte/depth/string/population/position/capture/source/work bounds admit or resource-refuse before excess retention."}
  ]
}
```
