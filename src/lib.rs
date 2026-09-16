// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Qualified, transport-independent observation admission for OB01.
//!
//! This crate verifies the semantic selections that make an observed record
//! usable by later temporal or protocol consumers. Its owner-contract layer
//! derives bounded canonical observation authority; it does not interpret a
//! telemetry transport or claim production-monitor execution.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};

pub mod authority;
mod runtime_reference;

pub use agent_ix_baseline_producer::{AdmittedBundleKey, AdmittedStaticBundle};
pub use runtime_reference::{
    EndpointSide, QualifiedSubject, ReferenceComponent, ReferenceRefusal, Relationship,
    RelationshipIdentity, RequiredRelationship, SubjectIdentity, SubjectKind,
};

/// Required linked-package descriptor format.
pub const NATIVE_LINKED_PACKAGE_FORMAT: &str = "native-linked-package/1";
/// Required Producer interface version.
pub const PRODUCER_INTERFACE_VERSION: &str = agent_ix_baseline_producer::INTERFACE_VERSION;

/// Opaque exact identity; admission validates non-empty values at their boundary.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identity(String);

impl Identity {
    /// Wraps an identity without interpreting or normalizing it.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the exact identity text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn valid(&self) -> bool {
        !self.0.trim().is_empty()
    }
}

/// Exact 32-byte SHA-256 digest in the domain declared by its containing field.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Digest([u8; 32]);

impl Digest {
    /// Wraps exact digest bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Selected compiled native-language package.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageSelection {
    /// Exact descriptor format.
    pub format: String,
    /// Immutable package identity.
    pub identity: Identity,
    /// Exact package revision.
    pub revision: Identity,
    /// Digest of the selected package bytes.
    pub digest: Digest,
}

/// Whether the observation originated inside or outside the assessed system.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    /// Internally recorded observation.
    Internal,
    /// Externally observed fact.
    External,
}

/// Exact position of an observation under one selected clock family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Anchor {
    /// Zero-based event position.
    EventPosition(u64),
    /// Fixed-sample position and interpretation.
    FixedSample {
        /// Zero-based sample index.
        index: u64,
        /// Epoch of sample zero in nanoseconds.
        epoch_nanos: i128,
        /// Positive sample period in nanoseconds.
        period_nanos: u64,
    },
    /// Timestamped-event instant in nanoseconds.
    TimestampNanos(i128),
}

/// Explicit valuation presence; missing is never Boolean false.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueState {
    /// A typed canonical value is present.
    Present {
        /// Exact value-type identity.
        value_type: Identity,
        /// Canonical value text under the selected type.
        canonical_value: String,
    },
    /// The required valuation is absent.
    Missing,
}

/// Exact semantic observation binding selected for admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationBinding {
    /// Binding identity.
    pub identity: Identity,
    /// Required observation source.
    pub source_identity: Identity,
    /// Required source schema.
    pub schema_identity: Identity,
    /// Required signal identity.
    pub signal_identity: Identity,
    /// Required trigger identity.
    pub trigger_identity: Identity,
    /// Required unit identity.
    pub unit: Identity,
    /// Required subject kind.
    pub subject_kind: SubjectKind,
    /// Whether an absent valuation makes admission incomplete.
    pub required: bool,
}

/// Candidate observation record carrying every identity used by FR-288.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedRecord {
    /// Caller-supplied identity, verified against the FR-288 preimage.
    pub identity: Identity,
    /// Selected binding identity.
    pub binding_identity: Identity,
    /// Selected source identity.
    pub source_identity: Identity,
    /// Selected source schema identity.
    pub schema_identity: Identity,
    /// Exact typed subject.
    pub subject: QualifiedSubject,
    /// Selected signal identity.
    pub signal_identity: Identity,
    /// Selected trigger identity.
    pub trigger_identity: Identity,
    /// Selected unit identity.
    pub unit: Identity,
    /// Explicit valuation presence and value.
    pub value: ValueState,
    /// Internal/external provenance.
    pub visibility: Visibility,
    /// Position under the selected clock family.
    pub anchor: Anchor,
    /// Event time remains independent from the carrier anchor and ingestion time.
    pub event_time_nanos: i128,
    /// Ingestion instant retained independently from event time.
    pub ingestion_time_nanos: i128,
    /// Optional explicit causal relationship; timestamps never supply it.
    pub causal_relationship_identity: Option<RelationshipIdentity>,
    /// Selected clock identity.
    pub clock_identity: Identity,
    /// Selected clock revision.
    pub clock_revision: Identity,
    /// Clock uncertainty in nanoseconds.
    pub clock_uncertainty_nanos: u64,
}

/// Exact semantic member-to-record binding in a selected scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    /// Opaque semantic member identity.
    pub object_identity: Identity,
    /// Exact admitted observation identity.
    pub record_identity: Identity,
    /// Scope-compatible observation anchor.
    pub anchor: Anchor,
}

/// Mutually exclusive finite scope selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    /// Immutable selected snapshot.
    Snapshot {
        /// Snapshot identity.
        snapshot_identity: Identity,
    },
    /// Selected event-time window.
    Window {
        /// Window identity.
        window_identity: Identity,
    },
}

/// The exact clock family and half-open coverage selected for a scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockRange {
    /// Half-open event-position range.
    EventPosition {
        /// Inclusive first position.
        start: u64,
        /// Exclusive end position.
        end_exclusive: u64,
    },
    /// Half-open fixed-sample range.
    FixedSample {
        /// Inclusive first sample index.
        start: u64,
        /// Exclusive end sample index.
        end_exclusive: u64,
        /// Epoch of sample zero in nanoseconds.
        epoch_nanos: i128,
        /// Positive sample period in nanoseconds.
        period_nanos: u64,
    },
    /// Half-open timestamped-event range.
    Timestamp {
        /// Inclusive start instant in nanoseconds.
        start_nanos: i128,
        /// Exclusive end instant in nanoseconds.
        end_nanos: i128,
    },
}

/// Complete population, scope, source, clock, and dependency selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeSelection {
    /// Owner-derived FR-263 population identity.
    pub population_identity: Identity,
    /// FR-264 identity of the exact explicit finite required-member set.
    pub membership_rule_identity: Identity,
    /// SHA-256 digest of the canonical membership document.
    pub membership_digest: Digest,
    /// Exact strict-read `quire.observation.explicit-members/v1` bytes.
    pub membership_document: Vec<u8>,
    /// Complete, sorted, distinct semantic member identity set selected by the rule.
    pub required_member_identities: Vec<Identity>,
    /// Complete, sorted, distinct source set selected for the population.
    pub observation_sources: Vec<Identity>,
    /// Complete, sorted, distinct completeness dependencies.
    pub completeness_dependencies: Vec<Identity>,
    /// Complete, sorted, distinct progress dependencies.
    pub progress_dependencies: Vec<Identity>,
    /// Selected clock identity.
    pub clock_identity: Identity,
    /// Selected clock revision.
    pub clock_revision: Identity,
    /// Whether the caller supplied the complete selected membership view.
    pub membership_complete: bool,
    /// Selected closure-definition identity, when supplied.
    pub closure_identity: Option<Identity>,
    /// Digest of the selected closure-definition bytes, when supplied.
    pub closure_digest: Option<Digest>,
    /// Exact snapshot or window selection.
    pub kind: ScopeKind,
    /// Exact half-open clock range.
    pub range: ClockRange,
    /// Complete member-to-record bindings.
    pub members: Vec<Member>,
}

/// Admission retention ceilings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    /// Maximum records retained by one qualified observation.
    pub max_records: usize,
    /// Maximum member bindings retained by one qualified observation.
    pub max_members: usize,
    /// Maximum supplied relationships inspected and retained.
    pub max_relationships: usize,
    /// Maximum required relationship premises inspected and retained.
    pub max_required_relationships: usize,
}

/// Complete caller-supplied observation admission request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRequest {
    /// Selected linked package.
    pub package: PackageSelection,
    /// Selected producer interface and configuration.
    pub producer: AdmittedStaticBundle,
    /// Selected observation binding.
    pub binding: ObservationBinding,
    /// Exact expected record subject.
    pub expected_subject: QualifiedSubject,
    /// Complete supplied relationship graph.
    pub relationships: Vec<Relationship>,
    /// Relationship premises required by the binding.
    pub required_relationships: Vec<RequiredRelationship>,
    /// Complete finite scope selection.
    pub scope: ScopeSelection,
    /// Candidate record population.
    pub records: Vec<AdmittedRecord>,
    /// Caller-declared retention ceilings.
    pub limits: ResourceLimits,
}

/// A fully qualified observation set with every caller-selected premise retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualifiedObservation {
    package: PackageSelection,
    producer: AdmittedStaticBundle,
    binding: ObservationBinding,
    expected_subject: QualifiedSubject,
    relationships: Vec<Relationship>,
    required_relationships: Vec<RequiredRelationship>,
    scope: ScopeSelection,
    records: Vec<AdmittedRecord>,
    limits: ResourceLimits,
}

impl QualifiedObservation {
    /// Returns the exact selected package.
    #[must_use]
    pub const fn package(&self) -> &PackageSelection {
        &self.package
    }

    /// Returns the exact selected producer interface and configuration.
    #[must_use]
    pub const fn producer(&self) -> &AdmittedStaticBundle {
        &self.producer
    }

    /// Returns the exact selected observation binding.
    #[must_use]
    pub const fn binding(&self) -> &ObservationBinding {
        &self.binding
    }

    /// Returns the exact expected subject.
    #[must_use]
    pub const fn expected_subject(&self) -> &QualifiedSubject {
        &self.expected_subject
    }

    /// Returns the complete admitted relationship graph.
    #[must_use]
    pub fn relationships(&self) -> &[Relationship] {
        &self.relationships
    }

    /// Returns every required relationship premise.
    #[must_use]
    pub fn required_relationships(&self) -> &[RequiredRelationship] {
        &self.required_relationships
    }

    /// Returns the complete selected population and scope.
    #[must_use]
    pub const fn scope(&self) -> &ScopeSelection {
        &self.scope
    }

    /// Returns the complete admitted record population.
    #[must_use]
    pub fn records(&self) -> &[AdmittedRecord] {
        &self.records
    }

    /// Returns the admission retention ceilings.
    #[must_use]
    pub const fn limits(&self) -> ResourceLimits {
        self.limits
    }
}

/// Missing-premise causes that remain incomplete rather than becoming false.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteReason {
    /// A required record valuation is absent.
    MissingValuation {
        /// Identity of the record lacking a valuation.
        record: Identity,
    },
    /// Required population membership is absent or incomplete.
    MissingMembership,
    /// A required directed relationship is absent.
    MissingRelationship {
        /// Complete exact required relationship slot.
        required: Box<RequiredRelationship>,
    },
    /// Required closure selection is absent.
    MissingClosure,
}

/// Typed causes that refuse observation admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefusalCause {
    /// A required selection is absent or malformed.
    InvalidSelection(SelectionField),
    /// The Producer interface version is unsupported.
    ProducerVersion,
    /// A record does not match the selected binding.
    BindingMismatch {
        /// Affected record identity.
        record: Identity,
        /// Exact mismatched binding component.
        field: BindingField,
    },
    /// A record does not match the selected subject.
    SubjectMismatch {
        /// Affected record identity.
        record: Identity,
    },
    /// A runtime subject or relationship failed exact reference validation.
    InvalidReference(ReferenceRefusal),
    /// One producer relationship identity was rebound to incompatible facts.
    RelationshipIdentityContradiction {
        /// Exact contradicted producer-local relationship identity.
        identity: RelationshipIdentity,
    },
    /// A supplied relationship conflicts with the required target.
    RelationshipConflict {
        /// Complete exact required relationship slot.
        required: Box<RequiredRelationship>,
    },
    /// More than one supplied relationship satisfies one premise.
    AmbiguousRelationship {
        /// Complete exact required relationship slot.
        required: Box<RequiredRelationship>,
    },
    /// A record uses a foreign clock or incompatible anchor.
    ClockMismatch {
        /// Affected record identity.
        record: Identity,
    },
    /// The selected half-open clock range is empty or malformed.
    InvalidWindow,
    /// A declared admission retention ceiling was exceeded.
    ResourceLimit {
        /// Exact exceeded population.
        limit: ResourceKind,
    },
    /// Two records supplied the same observation identity.
    DuplicateRecord {
        /// Duplicated record identity.
        record: Identity,
    },
    /// Two members supplied the same record identity.
    DuplicateMember {
        /// Duplicated record identity.
        record: Identity,
    },
    /// A member does not bind an exact compatible record.
    MemberRecordMismatch {
        /// Affected member identity.
        member: Identity,
        /// Selected record identity.
        record: Identity,
    },
    /// A caller-authored identity differs from the owner derivation.
    DerivedIdentityMismatch {
        /// Exact owner artifact that failed comparison.
        artifact: OwnerArtifact,
    },
}

/// Exact input selection that failed admission validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionField {
    /// Linked-package format.
    PackageFormat,
    /// Linked-package identity or revision.
    PackageIdentity,
    /// Observation binding identity.
    BindingIdentity,
    /// Binding source identity.
    BindingSourceIdentity,
    /// Binding schema identity.
    BindingSchemaIdentity,
    /// Binding signal identity.
    BindingSignalIdentity,
    /// Binding trigger identity.
    BindingTriggerIdentity,
    /// Binding unit identity.
    BindingUnit,
    /// Population identity.
    PopulationIdentity,
    /// Closure identity/digest pair.
    Closure,
    /// Snapshot/window, clock, or closure scope selection.
    Scope,
    /// Record or record-subject identity.
    RecordIdentity,
    /// Member or member-record identity.
    MemberIdentity,
}

/// Binding component whose exact selection did not match a record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingField {
    /// Binding identity.
    Binding,
    /// Source identity.
    Source,
    /// Schema identity.
    Schema,
    /// Signal identity.
    Signal,
    /// Trigger identity.
    Trigger,
    /// Unit identity.
    Unit,
}

/// Admission population whose declared retention ceiling was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    /// Record population.
    Records,
    /// Member-binding population.
    Members,
    /// Supplied relationship graph.
    Relationships,
    /// Required relationship-premise population.
    RequiredRelationships,
}

/// Owner-derived identity that did not match its canonical preimage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerArtifact {
    /// FR-264 identity/digest.
    Membership,
    /// Strict explicit-members document.
    MembershipDocument,
    /// FR-263 population identity.
    Population,
    /// Required-members/member-binding equality.
    MembershipPopulation,
    /// Exact selected source set.
    ObservationSources,
    /// FR-288 observation identity.
    Observation,
    /// Explicit causal relationship reference.
    CausalRelationship,
}

/// Result of observation admission; no variant is a Boolean truth value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionOutcome {
    /// Every required premise was admitted and retained.
    Available {
        /// Constructor-private qualified observation capability.
        observation: Box<QualifiedObservation>,
    },
    /// One or more required premises are absent.
    Incomplete {
        /// Exact missing-premise causes.
        reasons: Vec<IncompleteReason>,
    },
    /// An invalid or incompatible selection refused admission.
    Refused {
        /// Exact typed refusal cause.
        cause: RefusalCause,
    },
}

/// Validates an explicit observation handoff for later consumers.
#[must_use]
pub fn admit(mut request: AdmissionRequest) -> AdmissionOutcome {
    if request.package.format != NATIVE_LINKED_PACKAGE_FORMAT {
        return refused(RefusalCause::InvalidSelection(
            SelectionField::PackageFormat,
        ));
    }
    if !request.package.identity.valid() || !request.package.revision.valid() {
        return refused(RefusalCause::InvalidSelection(
            SelectionField::PackageIdentity,
        ));
    }
    if request.producer.interface_version() != PRODUCER_INTERFACE_VERSION {
        return refused(RefusalCause::ProducerVersion);
    }
    if let Some(field) = invalid_required_selection(&request) {
        return refused(RefusalCause::InvalidSelection(field));
    }
    if request.records.len() > request.limits.max_records {
        return refused(RefusalCause::ResourceLimit {
            limit: ResourceKind::Records,
        });
    }
    if request.scope.members.len() > request.limits.max_members {
        return refused(RefusalCause::ResourceLimit {
            limit: ResourceKind::Members,
        });
    }
    if request.relationships.len() > request.limits.max_relationships {
        return refused(RefusalCause::ResourceLimit {
            limit: ResourceKind::Relationships,
        });
    }
    if request.required_relationships.len() > request.limits.max_required_relationships {
        return refused(RefusalCause::ResourceLimit {
            limit: ResourceKind::RequiredRelationships,
        });
    }
    if !valid_range(request.scope.range) {
        return refused(RefusalCause::InvalidWindow);
    }
    if request.scope.closure_identity.is_none() != request.scope.closure_digest.is_none() {
        return refused(RefusalCause::InvalidSelection(SelectionField::Closure));
    }
    if request.scope.closure_identity.is_none() {
        return incomplete(IncompleteReason::MissingClosure);
    }
    if !request.scope.membership_complete {
        return incomplete(IncompleteReason::MissingMembership);
    }
    if !valid_scope(&request.scope) {
        return refused(RefusalCause::InvalidSelection(SelectionField::Scope));
    }

    let mut ids = BTreeSet::new();
    let mut incomplete_reasons = Vec::new();
    if let Some(cause) = invalid_relationship_selection(&request) {
        return refused(cause);
    }
    request.relationships.sort();
    request.relationships.dedup();
    request.required_relationships.sort();
    request.required_relationships.dedup();
    for record in &request.records {
        if !record.identity.valid() {
            return refused(RefusalCause::InvalidSelection(
                SelectionField::RecordIdentity,
            ));
        }
        if !ids.insert(record.identity.clone()) {
            return refused(RefusalCause::DuplicateRecord {
                record: record.identity.clone(),
            });
        }
        if record.binding_identity != request.binding.identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: BindingField::Binding,
            });
        }
        if record.source_identity != request.binding.source_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: BindingField::Source,
            });
        }
        if record.schema_identity != request.binding.schema_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: BindingField::Schema,
            });
        }
        if record.signal_identity != request.binding.signal_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: BindingField::Signal,
            });
        }
        if record.trigger_identity != request.binding.trigger_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: BindingField::Trigger,
            });
        }
        if record.unit != request.binding.unit {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: BindingField::Unit,
            });
        }
        if record.subject.kind() != &request.binding.subject_kind
            || record.subject != request.expected_subject
        {
            return refused(RefusalCause::SubjectMismatch {
                record: record.identity.clone(),
            });
        }
        if record.clock_identity != request.scope.clock_identity
            || record.clock_revision != request.scope.clock_revision
            || !anchor_compatible(request.scope.range, record.anchor)
        {
            return refused(RefusalCause::ClockMismatch {
                record: record.identity.clone(),
            });
        }
        if request.binding.required && matches!(record.value, ValueState::Missing) {
            incomplete_reasons.push(IncompleteReason::MissingValuation {
                record: record.identity.clone(),
            });
        }
    }

    let mut member_records = BTreeSet::new();
    for member in &request.scope.members {
        if !member.object_identity.valid() || !member.record_identity.valid() {
            return refused(RefusalCause::InvalidSelection(
                SelectionField::MemberIdentity,
            ));
        }
        if !member_records.insert(member.record_identity.clone()) {
            return refused(RefusalCause::DuplicateMember {
                record: member.record_identity.clone(),
            });
        }
        let Some(record) = request
            .records
            .iter()
            .find(|record| record.identity == member.record_identity)
        else {
            return refused(RefusalCause::MemberRecordMismatch {
                member: member.object_identity.clone(),
                record: member.record_identity.clone(),
            });
        };
        if record.anchor != member.anchor || !anchor_compatible(request.scope.range, member.anchor)
        {
            return refused(RefusalCause::MemberRecordMismatch {
                member: member.object_identity.clone(),
                record: member.record_identity.clone(),
            });
        }
    }
    if request
        .records
        .iter()
        .any(|record| !member_records.contains(&record.identity))
    {
        return incomplete(IncompleteReason::MissingMembership);
    }

    for wanted in &request.required_relationships {
        let matching: BTreeSet<_> = request
            .relationships
            .iter()
            .filter(|actual| wanted.matches(actual))
            .collect();
        if matching.len() > 1 {
            return refused(RefusalCause::AmbiguousRelationship {
                required: Box::new(wanted.clone()),
            });
        }
        if matching.is_empty() {
            if request
                .relationships
                .iter()
                .any(|actual| wanted.conflicts_with(actual))
            {
                return refused(RefusalCause::RelationshipConflict {
                    required: Box::new(wanted.clone()),
                });
            }
            incomplete_reasons.push(IncompleteReason::MissingRelationship {
                required: Box::new(wanted.clone()),
            });
        }
    }
    if incomplete_reasons.is_empty() {
        if let Err(artifact) = validate_owner_identities(&request) {
            return refused(RefusalCause::DerivedIdentityMismatch { artifact });
        }
        AdmissionOutcome::Available {
            observation: Box::new(QualifiedObservation {
                package: request.package,
                producer: request.producer,
                binding: request.binding,
                expected_subject: request.expected_subject,
                relationships: request.relationships,
                required_relationships: request.required_relationships,
                scope: request.scope,
                records: request.records,
                limits: request.limits,
            }),
        }
    } else {
        AdmissionOutcome::Incomplete {
            reasons: incomplete_reasons,
        }
    }
}

fn validate_owner_identities(request: &AdmissionRequest) -> std::result::Result<(), OwnerArtifact> {
    let limits = authority::Limits::owner_max();
    let (membership_identity, membership_digest) = authority::population::membership_identity(
        &request.scope.required_member_identities,
        limits,
    )
    .map_err(|_| OwnerArtifact::Membership)?;
    if membership_identity != request.scope.membership_rule_identity
        || membership_digest != request.scope.membership_digest
    {
        return Err(OwnerArtifact::Membership);
    }
    authority::population::read_membership(
        &request.scope.membership_document,
        &request.scope.required_member_identities,
        limits,
    )
    .map_err(|_| OwnerArtifact::MembershipDocument)?;
    let population_identity = authority::population::population_identity(request, limits)
        .map_err(|_| OwnerArtifact::Population)?;
    if population_identity != request.scope.population_identity {
        return Err(OwnerArtifact::Population);
    }
    if request.scope.members.len() != request.scope.required_member_identities.len()
        || request
            .scope
            .members
            .iter()
            .map(|member| &member.object_identity)
            .ne(request.scope.required_member_identities.iter())
    {
        return Err(OwnerArtifact::MembershipPopulation);
    }
    if request.scope.observation_sources.len() != 1
        || request.scope.observation_sources[0] != request.binding.source_identity
    {
        return Err(OwnerArtifact::ObservationSources);
    }
    for record in &request.records {
        let identity = authority::observation::record_identity(record, limits)
            .map_err(|_| OwnerArtifact::Observation)?;
        if identity != record.identity {
            return Err(OwnerArtifact::Observation);
        }
        if record
            .causal_relationship_identity
            .as_ref()
            .is_some_and(|identity| {
                !request
                    .relationships
                    .iter()
                    .any(|relationship| relationship.identity() == identity)
            })
        {
            return Err(OwnerArtifact::CausalRelationship);
        }
    }
    Ok(())
}

fn invalid_required_selection(request: &AdmissionRequest) -> Option<SelectionField> {
    [
        (
            request.binding.identity.valid(),
            SelectionField::BindingIdentity,
        ),
        (
            request.binding.source_identity.valid(),
            SelectionField::BindingSourceIdentity,
        ),
        (
            request.binding.schema_identity.valid(),
            SelectionField::BindingSchemaIdentity,
        ),
        (
            request.binding.signal_identity.valid(),
            SelectionField::BindingSignalIdentity,
        ),
        (
            request.binding.trigger_identity.valid(),
            SelectionField::BindingTriggerIdentity,
        ),
        (request.binding.unit.valid(), SelectionField::BindingUnit),
        (
            request.scope.population_identity.valid(),
            SelectionField::PopulationIdentity,
        ),
    ]
    .into_iter()
    .find_map(|(valid, field)| (!valid).then_some(field))
}

fn invalid_relationship_selection(request: &AdmissionRequest) -> Option<RefusalCause> {
    if !request.expected_subject.belongs_to(&request.producer)
        || request
            .records
            .iter()
            .any(|record| !record.subject.belongs_to(&request.producer))
        || request
            .relationships
            .iter()
            .any(|relationship| !relationship.belongs_to(&request.producer))
        || request
            .required_relationships
            .iter()
            .any(|relationship| !relationship.belongs_to(&request.producer))
    {
        return Some(RefusalCause::InvalidReference(
            ReferenceRefusal::ForeignAuthority,
        ));
    }
    let mut identities = BTreeMap::new();
    for relationship in &request.relationships {
        let slot = (relationship.authority(), relationship.identity());
        if let Some(previous) = identities.insert(slot, relationship) {
            if previous != relationship {
                return Some(RefusalCause::RelationshipIdentityContradiction {
                    identity: relationship.identity().clone(),
                });
            }
        }
    }
    None
}

fn valid_scope(scope: &ScopeSelection) -> bool {
    (match &scope.kind {
        ScopeKind::Snapshot { snapshot_identity } => snapshot_identity.valid(),
        ScopeKind::Window { window_identity } => window_identity.valid(),
    }) && scope.closure_identity.as_ref().is_some_and(Identity::valid)
        && scope.clock_identity.valid()
        && scope.clock_revision.valid()
}

fn valid_range(range: ClockRange) -> bool {
    match range {
        ClockRange::EventPosition {
            start,
            end_exclusive,
        } => start < end_exclusive,
        ClockRange::FixedSample {
            start,
            end_exclusive,
            period_nanos,
            ..
        } => start < end_exclusive && period_nanos > 0,
        ClockRange::Timestamp {
            start_nanos,
            end_nanos,
        } => start_nanos < end_nanos,
    }
}

fn anchor_compatible(range: ClockRange, anchor: Anchor) -> bool {
    match (range, anchor) {
        (
            ClockRange::EventPosition {
                start,
                end_exclusive,
            },
            Anchor::EventPosition(value),
        ) => value >= start && value < end_exclusive,
        (
            ClockRange::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            },
            Anchor::FixedSample {
                index,
                epoch_nanos: actual_epoch,
                period_nanos: actual_period,
            },
        ) => {
            index >= start
                && index < end_exclusive
                && actual_epoch == epoch_nanos
                && actual_period == period_nanos
        }
        (
            ClockRange::Timestamp {
                start_nanos,
                end_nanos,
            },
            Anchor::TimestampNanos(value),
        ) => value >= start_nanos && value < end_nanos,
        _ => false,
    }
}

fn incomplete(reason: IncompleteReason) -> AdmissionOutcome {
    AdmissionOutcome::Incomplete {
        reasons: vec![reason],
    }
}

fn refused(cause: RefusalCause) -> AdmissionOutcome {
    AdmissionOutcome::Refused { cause }
}
