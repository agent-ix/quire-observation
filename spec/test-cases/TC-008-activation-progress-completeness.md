---
id: TC-008
title: "Preserve activation and scope-authority axes"
type: TC
relationships:
  - target: ix://agent-ix/quire-observation/FR-008
    type: verifies
---
# TC-008: Preserve activation and scope-authority axes

## Description

Exercise activation capture, silence progress, completeness, lateness, verdict,
and settlement as independently selected authority facts.

## Test Procedure

1. Generate active inputs with an admitted trigger plus strict complete capture
   authority, inactive inputs with an empty optional binding plus closed/complete
   authority, activation-unknown inputs with an optional empty binding plus every
   open or incomplete scope-authority combination, and a required-empty refusal.
   Retain equal captured values
   under distinct admitted trigger identities and later value mutations.
2. Cross trigger presence with open/closed/incomplete progress and
   complete/incomplete/contradicted evidence; derive activation from those
   authority facts rather than accepting a caller-selected state.
3. Compare deadline and progress intervals that are disjoint, touching, equal,
   and overlapping, with complete and missing required-source sets.
4. Compare event and late-cutoff intervals for definitely timely, definitely
   late, and uncertain-lateness cases while permuting arrival order.
5. Independently omit or cross-wire each population, window, clock, source,
   capture, support, and authority-revision selection.
6. Claim trigger absence against history that contains that exact trigger, bind
   absent selection to a foreign trigger progress proof, substitute same-trigger
   progress and activation bytes across distinct binding identities, attempt to
   compose binding-committing empty-history progress with admitted-trigger capture
   authority, and mutate emitted schema instances across
   trigger/capture/proof/classification invariants.

## Expected Results

Capture identities and bytes preserve their activation-time values and sources.
Only definitely covering progress from every required source proves silence
coverage. Every state axis remains independent, and every foreign or cross-wired
selection or required empty binding refuses without a fallback result.
