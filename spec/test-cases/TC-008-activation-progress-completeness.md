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

1. Generate inactive, active, and activation-unknown inputs with equal capture
   values under distinct trigger identities and with later value mutations.
2. Cross every activation state with open/closed/incomplete progress and
   complete/incomplete/contradicted evidence.
3. Compare deadline and progress intervals that are disjoint, touching, equal,
   and overlapping, with complete and missing required-source sets.
4. Compare event and late-cutoff intervals for definitely timely, definitely
   late, and uncertain-lateness cases while permuting arrival order.
5. Independently omit or cross-wire each population, window, clock, source,
   capture, support, and authority-revision selection.

## Expected Results

Capture identities and bytes preserve their activation-time values and sources.
Only definitely covering progress from every required source proves silence
coverage. Every state axis remains independent, and every foreign or cross-wired
selection refuses without a fallback result.
