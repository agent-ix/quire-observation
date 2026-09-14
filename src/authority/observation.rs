// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

//! `quire.observation.record/v2` owner artifact.

use serde::{Deserialize, Serialize};

use agent_ix_baseline_producer::{AdmittedBundleKey, DigestSelection, Revision};

use super::common::{self, build_document, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::{AdmittedRecord, Anchor, Identity, QualifiedSubject, ValueState, Visibility};

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.record/v2";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/observation-record-v2.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "8737683a5971aabcb39bbc5f0842fa1d7c1db5c8f283b8b2544c6364c682f882";

/// Closed FR-294 timing classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lateness {
    /// Ingestion satisfied the exact selected cutoff.
    OnTime,
    /// Ingestion did not satisfy the exact selected cutoff.
    Late,
}

/// Closed late-cutoff rule vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CutoffRule {
    /// Ingestion time is less than or equal to the cutoff instant.
    IngestionTimeAtOrBefore,
}

/// Exact clock, instant, and rule selection used to classify lateness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutoffSelection {
    clock_identity: Identity,
    clock_revision: Identity,
    instant_nanos: i128,
    rule: CutoffRule,
    rule_revision: Identity,
}

impl CutoffSelection {
    /// Constructs an explicit cutoff selection without ambient defaults.
    #[must_use]
    pub const fn new(
        clock_identity: Identity,
        clock_revision: Identity,
        instant_nanos: i128,
        rule: CutoffRule,
        rule_revision: Identity,
    ) -> Self {
        Self {
            clock_identity,
            clock_revision,
            instant_nanos,
            rule,
            rule_revision,
        }
    }

    /// Returns the selected clock identity.
    #[must_use]
    pub const fn clock_identity(&self) -> &Identity {
        &self.clock_identity
    }

    /// Returns the selected clock revision.
    #[must_use]
    pub const fn clock_revision(&self) -> &Identity {
        &self.clock_revision
    }

    /// Returns the cutoff instant in nanoseconds.
    #[must_use]
    pub const fn instant_nanos(&self) -> i128 {
        self.instant_nanos
    }

    /// Returns the exact cutoff rule.
    #[must_use]
    pub const fn rule(&self) -> CutoffRule {
        self.rule
    }

    /// Returns the selected cutoff-rule revision.
    #[must_use]
    pub const fn rule_revision(&self) -> &Identity {
        &self.rule_revision
    }
}

/// Record identity and cutoff used to derive one observation artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    record_identity: Identity,
    cutoff: CutoffSelection,
}

impl Selection {
    /// Constructs an exact observation artifact selection.
    #[must_use]
    pub const fn new(record_identity: Identity, cutoff: CutoffSelection) -> Self {
        Self {
            record_identity,
            cutoff,
        }
    }
}

/// Borrowed typed valuation exposed by a validated observation view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueRef<'a> {
    /// A typed canonical value is present.
    Present {
        /// Exact value-type identity.
        value_type: &'a str,
        /// Canonical value text.
        canonical_value: &'a str,
    },
    /// The valuation is explicitly missing.
    Missing,
}

/// Borrowed exact anchor exposed by a validated observation view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorRef<'a> {
    /// Event-position anchor.
    EventPosition {
        /// Canonical unsigned position text.
        position: &'a str,
    },
    /// Fixed-sample anchor.
    FixedSample {
        /// Canonical sample-index text.
        index: &'a str,
        /// Canonical signed epoch-nanoseconds text.
        epoch_nanos: &'a str,
        /// Canonical unsigned period-nanoseconds text.
        period_nanos: &'a str,
    },
    /// Timestamped-event anchor.
    Timestamp {
        /// Canonical signed instant-nanoseconds text.
        instant_nanos: &'a str,
    },
}

/// Borrowed Producer-interface authority key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProducerAuthorityRef<'a> {
    /// Exact producer bundle identity.
    pub identity: &'a str,
    /// Exact namespaced producer revision.
    pub revision: &'a Revision,
    /// Exact four-member digest selection.
    pub digest: &'a DigestSelection,
}

/// Borrowed FR-287 subject key exposed by a validated record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubjectRef<'a> {
    /// Exact selected Producer authority.
    pub authority: ProducerAuthorityRef<'a>,
    /// Exact producer-local subject-kind bytes.
    pub kind: &'a [u8],
    /// Exact producer-local object-identity bytes.
    pub identity: &'a [u8],
}

/// Borrowed explicit causal relationship exposed by a validated record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationshipRef<'a> {
    /// Exact FCD relationship declaration identity.
    pub declaration_identity: &'a str,
    /// Exact selected Producer authority.
    pub authority: ProducerAuthorityRef<'a>,
    /// Exact Producer interface version.
    pub interface_version: &'a str,
    /// Exact producer-local relationship identity.
    pub identity: &'a [u8],
    /// Authored source endpoint.
    pub source: SubjectRef<'a>,
    /// Authored target endpoint.
    pub target: SubjectRef<'a>,
}

/// Borrowed clock facts exposed by a validated record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockRef<'a> {
    /// Exact clock identity.
    pub identity: &'a str,
    /// Exact clock revision.
    pub revision: &'a str,
    /// Canonical uncertainty-nanoseconds text.
    pub uncertainty_nanos: &'a str,
}

/// Read-only payload produced only inside a validated observation view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordView {
    observation_id: String,
    binding_identity: String,
    source_identity: String,
    schema_identity: String,
    subject: SubjectWire,
    signal_identity: String,
    trigger_identity: String,
    unit: String,
    value: ValueWire,
    visibility: VisibilityWire,
    anchor: AnchorWire,
    event_time_nanos: String,
    ingestion_time_nanos: String,
    causal_relationship: Option<RelationshipWire>,
    clock: ClockWire,
    lateness: Lateness,
    cutoff_identity: String,
}

impl RecordView {
    /// Returns the FR-288 observation identity.
    #[must_use]
    pub fn observation_identity(&self) -> &str {
        &self.observation_id
    }

    /// Returns the selected binding identity.
    #[must_use]
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    /// Returns the selected source identity.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    /// Returns the selected schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        &self.schema_identity
    }

    /// Returns the complete authority-qualified subject key.
    #[must_use]
    pub fn subject(&self) -> SubjectRef<'_> {
        subject_ref(&self.subject)
    }

    /// Returns the selected signal identity.
    #[must_use]
    pub fn signal_identity(&self) -> &str {
        &self.signal_identity
    }

    /// Returns the selected trigger identity.
    #[must_use]
    pub fn trigger_identity(&self) -> &str {
        &self.trigger_identity
    }

    /// Returns the selected unit identity.
    #[must_use]
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// Returns the exact valuation presence and content.
    #[must_use]
    pub fn value(&self) -> ValueRef<'_> {
        match &self.value {
            ValueWire::Present {
                value_type,
                canonical_value,
            } => ValueRef::Present {
                value_type,
                canonical_value,
            },
            ValueWire::Missing => ValueRef::Missing,
        }
    }

    /// Returns whether the source observation was internal or external.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        match self.visibility {
            VisibilityWire::Internal => Visibility::Internal,
            VisibilityWire::External => Visibility::External,
        }
    }

    /// Returns the exact clock-family anchor.
    #[must_use]
    pub fn anchor(&self) -> AnchorRef<'_> {
        match &self.anchor {
            AnchorWire::EventPosition { position } => AnchorRef::EventPosition { position },
            AnchorWire::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            } => AnchorRef::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            },
            AnchorWire::Timestamp { instant_nanos } => AnchorRef::Timestamp { instant_nanos },
        }
    }

    /// Returns canonical event-time nanoseconds text.
    #[must_use]
    pub fn event_time_nanos(&self) -> &str {
        &self.event_time_nanos
    }

    /// Returns canonical ingestion-time nanoseconds text.
    #[must_use]
    pub fn ingestion_time_nanos(&self) -> &str {
        &self.ingestion_time_nanos
    }

    /// Returns the explicit causal relationship, if one was selected.
    #[must_use]
    pub fn causal_relationship(&self) -> Option<RelationshipRef<'_>> {
        self.causal_relationship
            .as_ref()
            .map(|relationship| RelationshipRef {
                declaration_identity: &relationship.declaration_identity,
                authority: authority_ref(&relationship.authority),
                interface_version: &relationship.interface_version,
                identity: &relationship.identity,
                source: subject_ref(&relationship.source),
                target: subject_ref(&relationship.target),
            })
    }

    /// Returns the exact selected clock facts.
    #[must_use]
    pub fn clock(&self) -> ClockRef<'_> {
        ClockRef {
            identity: &self.clock.identity,
            revision: &self.clock.revision,
            uncertainty_nanos: &self.clock.uncertainty_nanos,
        }
    }

    /// Returns the owner-derived FR-294 lateness classification.
    #[must_use]
    pub const fn lateness(&self) -> Lateness {
        self.lateness
    }

    /// Returns the owner-derived cutoff identity.
    #[must_use]
    pub fn cutoff_identity(&self) -> &str {
        &self.cutoff_identity
    }
}

/// Constructor-private validated observation view.
pub type View = Validated<RecordView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum ValueWire {
    Present {
        value_type: String,
        canonical_value: String,
    },
    Missing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum VisibilityWire {
    Internal,
    External,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum AnchorWire {
    EventPosition {
        position: String,
    },
    FixedSample {
        index: String,
        epoch_nanos: String,
        period_nanos: String,
    },
    Timestamp {
        instant_nanos: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationshipWire {
    declaration_identity: String,
    authority: ProducerAuthorityWire,
    interface_version: String,
    identity: Vec<u8>,
    source: SubjectWire,
    target: SubjectWire,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProducerAuthorityWire {
    bundle_identity: String,
    bundle_revision: Revision,
    digest: DigestSelection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectWire {
    authority: ProducerAuthorityWire,
    kind: Vec<u8>,
    identity: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockWire {
    identity: String,
    revision: String,
    uncertainty_nanos: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObservationIdentityPreimage<'a> {
    identity_version: &'static str,
    observation: ObservationWithoutIdentity<'a>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CutoffIdentityPreimage<'a> {
    identity_version: &'static str,
    clock_identity: &'a str,
    clock_revision: &'a str,
    cutoff_instant_nanos: String,
    cutoff_rule: CutoffRule,
    cutoff_rule_revision: &'a str,
}

#[derive(Serialize)]
struct ObservationWithoutIdentity<'a> {
    binding_identity: &'a str,
    source_identity: &'a str,
    schema_identity: &'a str,
    subject: SubjectWire,
    signal_identity: &'a str,
    trigger_identity: &'a str,
    unit: &'a str,
    value: ValueWire,
    visibility: VisibilityWire,
    anchor: AnchorWire,
    event_time_nanos: String,
    ingestion_time_nanos: String,
    causal_relationship_identity: Option<&'a [u8]>,
    clock_identity: &'a str,
    clock_revision: &'a str,
    clock_uncertainty_nanos: String,
}

/// Derives the exact FR-288 record identity while ignoring the authored identity field.
pub fn record_identity(record: &AdmittedRecord, limits: Limits) -> Result<Identity> {
    validate_record(record)?;
    let preimage = ObservationIdentityPreimage {
        identity_version: "quire.observation.identity/v2",
        observation: without_identity(record),
    };
    common::sha256_jcs(&preimage, limits).map(|value| value.0)
}

/// Derives the exact FR-268 cutoff identity from its selected clock, instant and rule revision.
pub fn cutoff_identity(selection: &CutoffSelection, limits: Limits) -> Result<Identity> {
    if !selection.clock_identity.valid()
        || !selection.clock_revision.valid()
        || !selection.rule_revision.valid()
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "cutoff clock and rule revisions must be explicit",
            Usage::default(),
        ));
    }
    common::sha256_jcs(
        &CutoffIdentityPreimage {
            identity_version: "quire.observation.late-cutoff-identity/v1-draft.1",
            clock_identity: selection.clock_identity.as_str(),
            clock_revision: selection.clock_revision.as_str(),
            cutoff_instant_nanos: selection.instant_nanos.to_string(),
            cutoff_rule: selection.rule,
            cutoff_rule_revision: selection.rule_revision.as_str(),
        },
        limits,
    )
    .map(|value| value.0)
}

/// Derives canonical bounded owner bytes from qualified observation state.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let qualified = context.history().qualified();
    let record = qualified
        .records()
        .iter()
        .find(|record| record.identity == selection.record_identity)
        .ok_or_else(|| {
            Error::new(
                ErrorCode::ExpectedMismatch,
                "selected record is absent from qualified history",
                Usage::default(),
            )
        })?;
    let derived = record_identity(record, limits)?;
    if derived != record.identity {
        return Err(Error::new(
            ErrorCode::IdentityMismatch,
            "admitted record identity does not match FR-288 preimage",
            Usage::default(),
        ));
    }
    if record.clock_identity != selection.cutoff.clock_identity
        || record.clock_revision != selection.cutoff.clock_revision
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "late-data cutoff is cross-wired from the record clock",
            Usage::default(),
        ));
    }
    let cutoff_identity = cutoff_identity(&selection.cutoff, limits)?;
    let lateness = match selection.cutoff.rule {
        CutoffRule::IngestionTimeAtOrBefore => {
            if record.ingestion_time_nanos <= selection.cutoff.instant_nanos {
                Lateness::OnTime
            } else {
                Lateness::Late
            }
        }
    };
    let relationship = match &record.causal_relationship_identity {
        None => None,
        Some(identity) => {
            let relation = qualified
                .relationships()
                .iter()
                .find(|relation| relation.identity() == identity)
                .ok_or_else(|| {
                    Error::new(
                        ErrorCode::ExpectedMismatch,
                        "declared causal relationship is absent from qualified history",
                        Usage::default(),
                    )
                })?;
            Some(relationship_wire(relation))
        }
    };
    let payload = RecordView {
        observation_id: record.identity.as_str().to_owned(),
        binding_identity: record.binding_identity.as_str().to_owned(),
        source_identity: record.source_identity.as_str().to_owned(),
        schema_identity: record.schema_identity.as_str().to_owned(),
        subject: subject_wire(&record.subject),
        signal_identity: record.signal_identity.as_str().to_owned(),
        trigger_identity: record.trigger_identity.as_str().to_owned(),
        unit: record.unit.as_str().to_owned(),
        value: value_wire(&record.value),
        visibility: visibility_wire(record.visibility),
        anchor: anchor_wire(record.anchor),
        event_time_nanos: record.event_time_nanos.to_string(),
        ingestion_time_nanos: record.ingestion_time_nanos.to_string(),
        causal_relationship: relationship,
        clock: ClockWire {
            identity: record.clock_identity.as_str().to_owned(),
            revision: record.clock_revision.as_str().to_owned(),
            uncertainty_nanos: record.clock_uncertainty_nanos.to_string(),
        },
        lateness,
        cutoff_identity: cutoff_identity.as_str().to_owned(),
    };
    build_document(CONTRACT, context, &payload, usage(&payload), limits)
}

/// Strict-reads canonical bytes against independently supplied selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

fn validate_record(record: &AdmittedRecord) -> Result<()> {
    let identities_valid = record.binding_identity.valid()
        && record.source_identity.valid()
        && record.schema_identity.valid()
        && record.signal_identity.valid()
        && record.trigger_identity.valid()
        && record.unit.valid()
        && record.clock_identity.valid()
        && record.clock_revision.valid()
        && record
            .causal_relationship_identity
            .as_ref()
            .is_none_or(|identity| !identity.as_bytes().is_empty());
    if !identities_valid {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "record identity preimage contains an absent identity",
            Usage::default(),
        ));
    }
    Ok(())
}

fn without_identity(record: &AdmittedRecord) -> ObservationWithoutIdentity<'_> {
    ObservationWithoutIdentity {
        binding_identity: record.binding_identity.as_str(),
        source_identity: record.source_identity.as_str(),
        schema_identity: record.schema_identity.as_str(),
        subject: subject_wire(&record.subject),
        signal_identity: record.signal_identity.as_str(),
        trigger_identity: record.trigger_identity.as_str(),
        unit: record.unit.as_str(),
        value: value_wire(&record.value),
        visibility: visibility_wire(record.visibility),
        anchor: anchor_wire(record.anchor),
        event_time_nanos: record.event_time_nanos.to_string(),
        ingestion_time_nanos: record.ingestion_time_nanos.to_string(),
        causal_relationship_identity: record
            .causal_relationship_identity
            .as_ref()
            .map(crate::RelationshipIdentity::as_bytes),
        clock_identity: record.clock_identity.as_str(),
        clock_revision: record.clock_revision.as_str(),
        clock_uncertainty_nanos: record.clock_uncertainty_nanos.to_string(),
    }
}

fn value_wire(value: &ValueState) -> ValueWire {
    match value {
        ValueState::Present {
            value_type,
            canonical_value,
        } => ValueWire::Present {
            value_type: value_type.as_str().to_owned(),
            canonical_value: canonical_value.clone(),
        },
        ValueState::Missing => ValueWire::Missing,
    }
}

const fn visibility_wire(value: Visibility) -> VisibilityWire {
    match value {
        Visibility::Internal => VisibilityWire::Internal,
        Visibility::External => VisibilityWire::External,
    }
}

fn anchor_wire(value: Anchor) -> AnchorWire {
    match value {
        Anchor::EventPosition(position) => AnchorWire::EventPosition {
            position: position.to_string(),
        },
        Anchor::FixedSample {
            index,
            epoch_nanos,
            period_nanos,
        } => AnchorWire::FixedSample {
            index: index.to_string(),
            epoch_nanos: epoch_nanos.to_string(),
            period_nanos: period_nanos.to_string(),
        },
        Anchor::TimestampNanos(instant_nanos) => AnchorWire::Timestamp {
            instant_nanos: instant_nanos.to_string(),
        },
    }
}

fn relationship_wire(value: &crate::Relationship) -> RelationshipWire {
    RelationshipWire {
        declaration_identity: value.declaration_identity().to_owned(),
        authority: authority_wire(value.authority()),
        interface_version: value.interface_version().to_owned(),
        identity: value.identity().as_bytes().to_vec(),
        source: subject_wire(value.source()),
        target: subject_wire(value.target()),
    }
}

fn authority_wire(value: &AdmittedBundleKey) -> ProducerAuthorityWire {
    ProducerAuthorityWire {
        bundle_identity: value.bundle_identity.clone(),
        bundle_revision: value.bundle_revision.clone(),
        digest: value.digest.clone(),
    }
}

fn subject_wire(value: &QualifiedSubject) -> SubjectWire {
    SubjectWire {
        authority: authority_wire(value.authority()),
        kind: value.kind().as_bytes().to_vec(),
        identity: value.identity().as_bytes().to_vec(),
    }
}

fn authority_ref(value: &ProducerAuthorityWire) -> ProducerAuthorityRef<'_> {
    ProducerAuthorityRef {
        identity: &value.bundle_identity,
        revision: &value.bundle_revision,
        digest: &value.digest,
    }
}

fn subject_ref(value: &SubjectWire) -> SubjectRef<'_> {
    SubjectRef {
        authority: authority_ref(&value.authority),
        kind: &value.kind,
        identity: &value.identity,
    }
}

fn usage(payload: &RecordView) -> Usage {
    let mut string_bytes = payload.observation_id.len();
    for value in [
        &payload.binding_identity,
        &payload.source_identity,
        &payload.schema_identity,
        &payload.signal_identity,
        &payload.trigger_identity,
        &payload.unit,
        &payload.cutoff_identity,
    ] {
        string_bytes = string_bytes.max(value.len());
    }
    string_bytes = string_bytes
        .max(payload.subject.authority.bundle_identity.len())
        .max(payload.subject.authority.bundle_revision.namespace.len())
        .max(payload.subject.authority.bundle_revision.value.len())
        .max(payload.subject.authority.digest.algorithm.len())
        .max(payload.subject.authority.digest.domain.len())
        .max(payload.subject.authority.digest.version.len())
        .max(payload.subject.authority.digest.value.len())
        .max(payload.subject.kind.len())
        .max(payload.subject.identity.len());
    Usage {
        depth: 5,
        string_bytes,
        visited_fields: 34,
        ..Usage::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_digest_is_pinned() {
        assert_eq!(common::schema_sha256(SCHEMA_BYTES), SCHEMA_SHA256);
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
    }
}
