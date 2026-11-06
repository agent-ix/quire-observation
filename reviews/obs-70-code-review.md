---
id: SR-055
title: "OBS-70 code-review of canonical main adoption"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-observation; Cargo.toml, Cargo.lock, src/producer.rs; OBS-70; PR #47"
review_set: subset
---

## Summary

Refs OBS-70. Refs IR-1005. Reviewed PR #47 dependency selection and required API adaptations.

## Verdict

PASS for the reviewed owner scope: no source findings. This artifact is not GitHub native approval; downstream AC3 remains pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The manifest selects the canonical main branch with the required serde_json feature and a compatible exact package version. The normal lock records one canonical package plus its upstream derive crate. The producer still removes its own digest member, hashes canonical JSON under the same 1,048,576-byte ceiling, and maps encoder failures to the existing producer Refusal type. Canonical main implements Value encoding through its iterative writer; sha256 propagates errors before returning a digest. Removing the obsolete depth argument follows the upstream byte-only API and removes an invariant expect without changing the owner byte bound. No copied encoder, compatibility layer, source override, integration edit, or gate narrowing appears in PR #47.

Existing owner evidence: /tmp/obs70-fmt.log, /tmp/obs70-clippy.log, /tmp/obs70-test.log (117 passing tests), /tmp/obs70-doc.log and /tmp/obs70-owner-results.log report all four required checks passing. No reviewer build, retest, Kani, census, or final gate was run, per the authorized review scope. The lead owns the final scoped gate after artifact custody. No applicable AssuranceProfile was found. AGENTS.md, CLAUDE.md, CONTRIBUTING.md, CLA.md and AGPL license were read; contribution identity and CLA signing remain the lead's responsibility.

## Examined scope

```yaml
{
  "scope": [
    {
      "id": "OBS-70-AC1",
      "path": "Cargo.toml",
      "role": "examined",
      "excerpt": "QObs Cargo.toml selects quire-canonical branch = main (no tag, no rev pin); normal Cargo.lock update; QObs owner checks pass."
    },
    {
      "id": "OBS-70-AC2",
      "path": "src/producer.rs",
      "role": "examined",
      "excerpt": "any API adaptation needed for canonical main is made in QObs only; no copy, no compat layer, no guard change in integration."
    },
    {
      "id": "OBS-70-AC3",
      "path": "Cargo.lock",
      "role": "context_only",
      "excerpt": "integration dependency_cohort test passes with QObs adopted."
    }
  ]
}
```
