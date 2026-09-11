---
id: US-001
title: "Qualify observations for a declared assessment"
type: US
relationships: []
---
# US-001: Qualify observations for a declared assessment

## Story

**As a** native Quire assessment consumer
**I want** observations to arrive with their selected identities, scope, and
availability status
**So that** I can assess an obligation without treating missing or unrelated
telemetry as evidence.

## Context

Orders, shipments, payment attempts, and refunds may share provider or trace
infrastructure but remain different business subjects. A finite assessment also
needs explicit population, window, clock, progress, and closure premises.

## Acceptance Examples (Illustrative)

### US-001-EX-1: Qualified provider effect

- **Given** a selected producer, package, binding, and finite scope
- **When** the consumer submits a matching provider effect
- **Then** it receives a qualified assessment input with provenance.

### US-001-EX-2: Missing provider evidence

- **Given** a required provider effect is unavailable for a selected scope
- **When** the consumer submits the assessment request
- **Then** it receives an explicit incomplete outcome rather than a false value.

## Dependencies (Contextual)

The story depends on the selected Producer interface 1.2.0 and a linked native
package. It feeds downstream temporal, protocol, and result consumers.

## Priority and Risk (Informative)

Priority is P0 because the native-language launch cannot honestly assess an
external workflow without qualified observations. The principal risk is a false
success caused by inferred identity, completeness, or time.
