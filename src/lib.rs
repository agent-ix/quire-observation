//! Qualified, transport-independent observation admission for OB01.
//!
//! This crate verifies the semantic selections that make an observed record
//! usable by later temporal or protocol consumers. It does not replay temporal
//! rules, settle deadlines, or interpret a telemetry transport.

use std::collections::BTreeSet;

pub const NATIVE_LINKED_PACKAGE_FORMAT: &str = "native-linked-package/1";
pub const PRODUCER_INTERFACE_VERSION: &str = "1.2.0";

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identity(String);

impl Identity {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn valid(&self) -> bool {
        !self.0.trim().is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Digest([u8; 32]);

impl Digest {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageSelection {
    pub format: String,
    pub identity: Identity,
    pub revision: Identity,
    pub digest: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerSelection {
    pub interface_version: String,
    pub document_identity: Identity,
    pub document_digest: Digest,
    pub model_identity: Identity,
    pub configuration_identity: Identity,
    pub configuration_digest: Digest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubjectKind {
    Order,
    Shipment,
    PaymentAttempt,
    Refund,
    Delivery,
    Effect,
    Receipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subject {
    pub kind: SubjectKind,
    pub identity: Identity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationshipKind {
    ShipmentForOrder,
    PaymentAttemptForOrder,
    RefundForOrder,
    RefundCompensatesEffect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relationship {
    pub identity: Identity,
    pub kind: RelationshipKind,
    pub from: Subject,
    pub to: Subject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredRelationship {
    pub kind: RelationshipKind,
    pub from: Subject,
    pub to: Subject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Visibility {
    Internal,
    External,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Anchor {
    EventPosition(u64),
    FixedSample {
        index: u64,
        epoch_nanos: i128,
        period_nanos: u64,
    },
    TimestampNanos(i128),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueState {
    Present {
        value_type: Identity,
        canonical_value: String,
    },
    Missing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationBinding {
    pub identity: Identity,
    pub source_identity: Identity,
    pub schema_identity: Identity,
    pub signal_identity: Identity,
    pub unit: Identity,
    pub subject_kind: SubjectKind,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedRecord {
    pub identity: Identity,
    pub binding_identity: Identity,
    pub source_identity: Identity,
    pub schema_identity: Identity,
    pub subject: Subject,
    pub signal_identity: Identity,
    pub unit: Identity,
    pub value: ValueState,
    pub visibility: Visibility,
    pub anchor: Anchor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    pub object_identity: Identity,
    pub record_identity: Identity,
    pub anchor: Anchor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    Snapshot { snapshot_identity: Identity },
    Window { window_identity: Identity },
}

/// The exact clock family and half-open coverage selected for a scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockRange {
    EventPosition {
        start: u64,
        end_exclusive: u64,
    },
    FixedSample {
        start: u64,
        end_exclusive: u64,
        epoch_nanos: i128,
        period_nanos: u64,
    },
    Timestamp {
        start_nanos: i128,
        end_nanos: i128,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeSelection {
    pub population_identity: Identity,
    pub membership_digest: Digest,
    pub closure_identity: Option<Identity>,
    pub closure_digest: Option<Digest>,
    pub kind: ScopeKind,
    pub range: ClockRange,
    pub members: Vec<Member>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub max_records: usize,
    pub max_members: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionRequest {
    pub package: PackageSelection,
    pub producer: ProducerSelection,
    pub binding: ObservationBinding,
    pub expected_subject: Subject,
    pub relationships: Vec<Relationship>,
    pub required_relationships: Vec<RequiredRelationship>,
    pub scope: ScopeSelection,
    pub records: Vec<AdmittedRecord>,
    pub limits: ResourceLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncompleteReason {
    MissingValuation { record: Identity },
    MissingRelationship { from: Identity, to: Identity },
    MissingClosure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefusalCause {
    InvalidSelection(&'static str),
    ProducerVersion,
    BindingMismatch {
        record: Identity,
        field: &'static str,
    },
    SubjectMismatch {
        record: Identity,
    },
    RelationshipConflict {
        from: Identity,
        to: Identity,
    },
    AmbiguousRelationship {
        from: Identity,
        to: Identity,
    },
    ClockMismatch {
        record: Identity,
    },
    InvalidWindow,
    ResourceLimit {
        limit: &'static str,
    },
    DuplicateRecord {
        record: Identity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionOutcome {
    Available { records: Vec<AdmittedRecord> },
    Incomplete { reasons: Vec<IncompleteReason> },
    Refused { cause: RefusalCause },
}

/// Validates an explicit observation handoff for later consumers.
#[must_use]
pub fn admit(request: AdmissionRequest) -> AdmissionOutcome {
    if request.package.format != NATIVE_LINKED_PACKAGE_FORMAT {
        return refused(RefusalCause::InvalidSelection("package format"));
    }
    if !request.package.identity.valid() || !request.package.revision.valid() {
        return refused(RefusalCause::InvalidSelection("package identity"));
    }
    if request.producer.interface_version != PRODUCER_INTERFACE_VERSION {
        return refused(RefusalCause::ProducerVersion);
    }
    if !request.producer.document_identity.valid()
        || !request.producer.model_identity.valid()
        || !request.producer.configuration_identity.valid()
        || !request.binding.identity.valid()
        || !request.expected_subject.identity.valid()
        || !request.scope.population_identity.valid()
    {
        return refused(RefusalCause::InvalidSelection("required identity"));
    }
    if request.records.len() > request.limits.max_records {
        return refused(RefusalCause::ResourceLimit { limit: "records" });
    }
    if request.scope.members.len() > request.limits.max_members {
        return refused(RefusalCause::ResourceLimit { limit: "members" });
    }
    if !valid_range(request.scope.range) {
        return refused(RefusalCause::InvalidWindow);
    }
    if request.scope.closure_identity.is_none() != request.scope.closure_digest.is_none() {
        return refused(RefusalCause::InvalidSelection("closure selection"));
    }
    if request.scope.closure_identity.is_none() {
        return incomplete(IncompleteReason::MissingClosure);
    }

    let mut ids = BTreeSet::new();
    let mut incomplete_reasons = Vec::new();
    for record in &request.records {
        if !ids.insert(record.identity.clone()) {
            return refused(RefusalCause::DuplicateRecord {
                record: record.identity.clone(),
            });
        }
        if record.binding_identity != request.binding.identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: "binding",
            });
        }
        if record.source_identity != request.binding.source_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: "source",
            });
        }
        if record.schema_identity != request.binding.schema_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: "schema",
            });
        }
        if record.signal_identity != request.binding.signal_identity {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: "signal",
            });
        }
        if record.unit != request.binding.unit {
            return refused(RefusalCause::BindingMismatch {
                record: record.identity.clone(),
                field: "unit",
            });
        }
        if record.subject.kind != request.binding.subject_kind
            || record.subject != request.expected_subject
        {
            return refused(RefusalCause::SubjectMismatch {
                record: record.identity.clone(),
            });
        }
        if !anchor_compatible(request.scope.range, record.anchor) {
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

    for wanted in &request.required_relationships {
        let matching: Vec<_> = request
            .relationships
            .iter()
            .filter(|actual| {
                actual.kind == wanted.kind && actual.from == wanted.from && actual.to == wanted.to
            })
            .collect();
        if matching.len() > 1 {
            return refused(RefusalCause::AmbiguousRelationship {
                from: wanted.from.identity.clone(),
                to: wanted.to.identity.clone(),
            });
        }
        if matching.is_empty() {
            if request
                .relationships
                .iter()
                .any(|actual| actual.kind == wanted.kind && actual.from == wanted.from)
            {
                return refused(RefusalCause::RelationshipConflict {
                    from: wanted.from.identity.clone(),
                    to: wanted.to.identity.clone(),
                });
            }
            incomplete_reasons.push(IncompleteReason::MissingRelationship {
                from: wanted.from.identity.clone(),
                to: wanted.to.identity.clone(),
            });
        }
    }
    if incomplete_reasons.is_empty() {
        AdmissionOutcome::Available {
            records: request.records,
        }
    } else {
        AdmissionOutcome::Incomplete {
            reasons: incomplete_reasons,
        }
    }
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
