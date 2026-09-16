// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Shared bounded envelope, identity, history, and strict-reader machinery.

use std::fmt;
use std::io::{self, Write};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{AdmittedRecord, ClockRange, Digest, Identity, QualifiedObservation, ScopeKind};

/// Owner-enforced ceilings that callers may lower but cannot raise.
pub const OWNER_MAX: Limits = Limits {
    max_input_bytes: 8 * 1024 * 1024,
    max_output_bytes: 8 * 1024 * 1024,
    max_depth: 64,
    max_string_bytes: 1024 * 1024,
    max_population_entries: 10_000,
    max_positions: 10_000,
    max_capture_bindings: 10_000,
    max_required_sources: 10_000,
    max_bundle_components: 10_000,
    max_replacements: 10_000,
    max_conflicts: 10_000,
    max_lineage_children: 10_000,
    max_visited_fields: 1_000_000,
};

/// Caller-controlled resource ceilings. Values are always clamped to owner maxima.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum accepted input document bytes.
    pub max_input_bytes: usize,
    /// Maximum emitted canonical document bytes.
    pub max_output_bytes: usize,
    /// Maximum JSON nesting depth.
    pub max_depth: usize,
    /// Maximum encoded bytes in one JSON string.
    pub max_string_bytes: usize,
    /// Maximum population-like entries.
    pub max_population_entries: usize,
    /// Maximum declared position entries.
    pub max_positions: usize,
    /// Maximum capture bindings.
    pub max_capture_bindings: usize,
    /// Maximum required observation sources.
    pub max_required_sources: usize,
    /// Maximum components retained in one authority bundle.
    pub max_bundle_components: usize,
    /// Maximum explicit replacement edges in one bundle revision.
    pub max_replacements: usize,
    /// Maximum explicit conflict facts in one bundle revision.
    pub max_conflicts: usize,
    /// Maximum direct children retained in one supplied lineage view.
    pub max_lineage_children: usize,
    /// Maximum object members and array elements visited while scanning.
    pub max_visited_fields: usize,
}

impl Limits {
    /// Returns the immutable owner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamps every caller ceiling to the corresponding owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_input_bytes: min(self.max_input_bytes, OWNER_MAX.max_input_bytes),
            max_output_bytes: min(self.max_output_bytes, OWNER_MAX.max_output_bytes),
            max_depth: min(self.max_depth, OWNER_MAX.max_depth),
            max_string_bytes: min(self.max_string_bytes, OWNER_MAX.max_string_bytes),
            max_population_entries: min(
                self.max_population_entries,
                OWNER_MAX.max_population_entries,
            ),
            max_positions: min(self.max_positions, OWNER_MAX.max_positions),
            max_capture_bindings: min(self.max_capture_bindings, OWNER_MAX.max_capture_bindings),
            max_required_sources: min(self.max_required_sources, OWNER_MAX.max_required_sources),
            max_bundle_components: min(self.max_bundle_components, OWNER_MAX.max_bundle_components),
            max_replacements: min(self.max_replacements, OWNER_MAX.max_replacements),
            max_conflicts: min(self.max_conflicts, OWNER_MAX.max_conflicts),
            max_lineage_children: min(self.max_lineage_children, OWNER_MAX.max_lineage_children),
            max_visited_fields: min(self.max_visited_fields, OWNER_MAX.max_visited_fields),
        }
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

impl Default for Limits {
    fn default() -> Self {
        OWNER_MAX
    }
}

/// Deterministic semantic work recorded in an artifact.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    /// Canonical wire bytes scanned or emitted.
    pub wire_bytes: usize,
    /// Deepest JSON value visited.
    pub depth: usize,
    /// Largest encoded JSON string visited.
    pub string_bytes: usize,
    /// Largest population-like array visited.
    pub population_entries: usize,
    /// Position entries visited.
    pub positions: usize,
    /// Capture bindings visited.
    pub capture_bindings: usize,
    /// Required-source entries visited.
    pub required_sources: usize,
    /// Bundle components visited or retained.
    pub bundle_components: usize,
    /// Replacement edges visited or retained.
    pub replacements: usize,
    /// Explicit conflict facts visited or retained.
    pub conflicts: usize,
    /// Direct lineage children visited or retained.
    pub lineage_children: usize,
    /// Object members, array elements, or explicit planner units visited.
    pub visited_fields: usize,
}

/// Stable machine-readable owner-boundary error codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// A required caller selection is absent or malformed.
    InvalidSelection,
    /// A derived identity does not match its canonical preimage.
    IdentityMismatch,
    /// A document revision is incompatible with its lineage position.
    RevisionMismatch,
    /// A correction names an invalid predecessor.
    PredecessorMismatch,
    /// A bounded operation exceeded an effective resource ceiling.
    ResourceIncomplete,
    /// Input bytes are not UTF-8.
    InvalidUtf8,
    /// Input bytes are not a valid closed JSON value.
    InvalidJson,
    /// Valid JSON bytes are not the canonical encoding.
    NonCanonical,
    /// A document names a contract other than the selected reader contract.
    ContractMismatch,
    /// A document or selection differs from independently supplied authority.
    ExpectedMismatch,
    /// A required observation-authority premise is absent.
    MissingPremise,
    /// Activation capture authority is incomplete or cross-wired.
    CaptureMismatch,
    /// Evaluator contribution support is absent or cross-wired.
    SupportMismatch,
    /// A strict-read owner identity, subject, clock, or revision is cross-wired.
    AuthorityMismatch,
    /// Multiple observations claim one declared order position.
    AmbiguousOrder,
    /// A Boolean possibility set is empty or inconsistent with its reason.
    InvalidPossibilitySet,
    /// An event-time interval has an invalid identity or reversed endpoints.
    InvalidInterval,
    /// Two intervals name different clock, revision, or unit domains.
    IntervalDomainMismatch,
    /// A supplied bundle predecessor is not the selected current head.
    StaleHead,
    /// A supplied lineage already contains a competing successor.
    KnownSibling,
    /// A declared bundle replacement is incomplete or cross-wired.
    InvalidReplacement,
    /// One authority/scope/revision key names unequal canonical bytes.
    IdentityContradiction,
    /// A dependency edge names an absent or contradictory graph node.
    InvalidDependency,
    /// A dependency graph is qualified by a different authority revision.
    ForeignRevision,
    /// An explicit dependency graph contains a directed cycle.
    DependencyCycle,
}

impl ErrorCode {
    const ALL: [Self; 25] = [
        Self::InvalidSelection,
        Self::IdentityMismatch,
        Self::RevisionMismatch,
        Self::PredecessorMismatch,
        Self::ResourceIncomplete,
        Self::InvalidUtf8,
        Self::InvalidJson,
        Self::NonCanonical,
        Self::ContractMismatch,
        Self::ExpectedMismatch,
        Self::MissingPremise,
        Self::CaptureMismatch,
        Self::SupportMismatch,
        Self::AuthorityMismatch,
        Self::AmbiguousOrder,
        Self::InvalidPossibilitySet,
        Self::InvalidInterval,
        Self::IntervalDomainMismatch,
        Self::StaleHead,
        Self::KnownSibling,
        Self::InvalidReplacement,
        Self::IdentityContradiction,
        Self::InvalidDependency,
        Self::ForeignRevision,
        Self::DependencyCycle,
    ];

    /// Returns every stable code exactly once.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &Self::ALL
    }

    /// Parses an exact stable code label without aliases or case folding.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }

    /// Returns the stable wire label for this code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidSelection => "QOBS-AUTH-INVALID-SELECTION",
            Self::IdentityMismatch => "QOBS-AUTH-IDENTITY-MISMATCH",
            Self::RevisionMismatch => "QOBS-AUTH-REVISION-MISMATCH",
            Self::PredecessorMismatch => "QOBS-AUTH-PREDECESSOR-MISMATCH",
            Self::ResourceIncomplete => "QOBS-AUTH-RESOURCE-INCOMPLETE",
            Self::InvalidUtf8 => "QOBS-AUTH-INVALID-UTF8",
            Self::InvalidJson => "QOBS-AUTH-INVALID-JSON",
            Self::NonCanonical => "QOBS-AUTH-NONCANONICAL",
            Self::ContractMismatch => "QOBS-AUTH-CONTRACT-MISMATCH",
            Self::ExpectedMismatch => "QOBS-AUTH-EXPECTED-MISMATCH",
            Self::MissingPremise => "QOBS-AUTH-MISSING-PREMISE",
            Self::CaptureMismatch => "QOBS-AUTH-CAPTURE-MISMATCH",
            Self::SupportMismatch => "QOBS-AUTH-SUPPORT-MISMATCH",
            Self::AuthorityMismatch => "QOBS-AUTH-AUTHORITY-MISMATCH",
            Self::AmbiguousOrder => "QOBS-AUTH-AMBIGUOUS-ORDER",
            Self::InvalidPossibilitySet => "QOBS-AUTH-INVALID-POSSIBILITY-SET",
            Self::InvalidInterval => "QOBS-AUTH-INVALID-INTERVAL",
            Self::IntervalDomainMismatch => "QOBS-AUTH-INTERVAL-DOMAIN-MISMATCH",
            Self::StaleHead => "QOBS-AUTH-STALE-HEAD",
            Self::KnownSibling => "QOBS-AUTH-KNOWN-SIBLING",
            Self::InvalidReplacement => "QOBS-AUTH-INVALID-REPLACEMENT",
            Self::IdentityContradiction => "QOBS-AUTH-IDENTITY-CONTRADICTION",
            Self::InvalidDependency => "QOBS-AUTH-INVALID-DEPENDENCY",
            Self::ForeignRevision => "QOBS-AUTH-FOREIGN-REVISION",
            Self::DependencyCycle => "QOBS-AUTH-DEPENDENCY-CYCLE",
        }
    }
}

/// Error envelope returned by every owner derivation and strict reader.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code}: {detail}", code = .code.as_str())]
pub struct Error {
    code: ErrorCode,
    detail: &'static str,
    usage: Usage,
}

impl Error {
    pub(crate) const fn new(code: ErrorCode, detail: &'static str, usage: Usage) -> Self {
        Self {
            code,
            detail,
            usage,
        }
    }

    /// Returns the stable machine-readable cause.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    /// Returns bounded diagnostic detail; callers must branch on [`Self::code`].
    #[must_use]
    pub const fn detail(&self) -> &'static str {
        self.detail
    }

    /// Returns work observed before refusal without exposing a partial view.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

/// Owner-boundary result using the crate's stable error envelope.
pub type Result<T> = std::result::Result<T, Error>;

/// Independent progress or closure state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OpenClosed {
    /// The selected scope is not closed through its boundary.
    Open,
    /// The selected scope is closed through its boundary.
    Closed,
}

/// Exact FR-289 mapping from an inclusive native interval to its selected
/// half-open observation carrier and progress watermark.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "clock_family", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TemporalBoundary {
    /// Inclusive native interval carried by zero-based event positions.
    EventPosition {
        /// Inclusive native lower position.
        lower: u64,
        /// Inclusive native upper position.
        upper_inclusive: u64,
        /// Checked minimal exclusive carrier end.
        carrier_end_exclusive: u64,
        /// Declared progress watermark.
        watermark: u64,
    },
    /// Inclusive native interval carried by fixed sample indexes.
    FixedSample {
        /// Inclusive native lower sample.
        lower: u64,
        /// Inclusive native upper sample.
        upper_inclusive: u64,
        /// Checked minimal exclusive carrier end.
        carrier_end_exclusive: u64,
        /// Declared progress watermark.
        watermark: u64,
    },
    /// Inclusive native interval carried by timestamped events.
    TimestampedEvent {
        /// Inclusive native lower instant in nanoseconds.
        lower_nanos: i128,
        /// Inclusive native upper instant in nanoseconds.
        upper_inclusive_nanos: i128,
        /// Explicit exclusive carrier end greater than the upper instant.
        carrier_end_exclusive_nanos: i128,
        /// Declared progress watermark in nanoseconds.
        watermark_nanos: i128,
    },
}

/// Borrowed canonical boundary text exposed by validated progress/closure views.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryRef<'a> {
    /// Event-position boundary mapping.
    EventPosition {
        /// Canonical inclusive-lower text.
        lower: &'a str,
        /// Canonical inclusive-upper text.
        upper_inclusive: &'a str,
        /// Canonical exclusive-carrier-end text.
        carrier_end_exclusive: &'a str,
        /// Canonical watermark text.
        watermark: &'a str,
    },
    /// Fixed-sample boundary mapping.
    FixedSample {
        /// Canonical inclusive-lower text.
        lower: &'a str,
        /// Canonical inclusive-upper text.
        upper_inclusive: &'a str,
        /// Canonical exclusive-carrier-end text.
        carrier_end_exclusive: &'a str,
        /// Canonical watermark text.
        watermark: &'a str,
    },
    /// Timestamped-event boundary mapping.
    TimestampedEvent {
        /// Canonical inclusive-lower nanoseconds text.
        lower_nanos: &'a str,
        /// Canonical inclusive-upper nanoseconds text.
        upper_inclusive_nanos: &'a str,
        /// Canonical exclusive-carrier-end nanoseconds text.
        carrier_end_exclusive_nanos: &'a str,
        /// Canonical watermark nanoseconds text.
        watermark_nanos: &'a str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "clock_family", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum BoundaryWire {
    EventPosition {
        lower: String,
        upper_inclusive: String,
        carrier_end_exclusive: String,
        watermark: String,
    },
    FixedSample {
        lower: String,
        upper_inclusive: String,
        carrier_end_exclusive: String,
        watermark: String,
    },
    TimestampedEvent {
        lower_nanos: String,
        upper_inclusive_nanos: String,
        carrier_end_exclusive_nanos: String,
        watermark_nanos: String,
    },
}

impl BoundaryWire {
    pub(crate) fn as_ref(&self) -> BoundaryRef<'_> {
        match self {
            Self::EventPosition {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            } => BoundaryRef::EventPosition {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            },
            Self::FixedSample {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            } => BoundaryRef::FixedSample {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            },
            Self::TimestampedEvent {
                lower_nanos,
                upper_inclusive_nanos,
                carrier_end_exclusive_nanos,
                watermark_nanos,
            } => BoundaryRef::TimestampedEvent {
                lower_nanos,
                upper_inclusive_nanos,
                carrier_end_exclusive_nanos,
                watermark_nanos,
            },
        }
    }
}

pub(crate) fn boundary_wire(value: TemporalBoundary) -> BoundaryWire {
    match value {
        TemporalBoundary::EventPosition {
            lower,
            upper_inclusive,
            carrier_end_exclusive,
            watermark,
        } => BoundaryWire::EventPosition {
            lower: lower.to_string(),
            upper_inclusive: upper_inclusive.to_string(),
            carrier_end_exclusive: carrier_end_exclusive.to_string(),
            watermark: watermark.to_string(),
        },
        TemporalBoundary::FixedSample {
            lower,
            upper_inclusive,
            carrier_end_exclusive,
            watermark,
        } => BoundaryWire::FixedSample {
            lower: lower.to_string(),
            upper_inclusive: upper_inclusive.to_string(),
            carrier_end_exclusive: carrier_end_exclusive.to_string(),
            watermark: watermark.to_string(),
        },
        TemporalBoundary::TimestampedEvent {
            lower_nanos,
            upper_inclusive_nanos,
            carrier_end_exclusive_nanos,
            watermark_nanos,
        } => BoundaryWire::TimestampedEvent {
            lower_nanos: lower_nanos.to_string(),
            upper_inclusive_nanos: upper_inclusive_nanos.to_string(),
            carrier_end_exclusive_nanos: carrier_end_exclusive_nanos.to_string(),
            watermark_nanos: watermark_nanos.to_string(),
        },
    }
}

impl TemporalBoundary {
    pub(crate) fn validate(self, state: OpenClosed) -> Result<()> {
        let valid = match self {
            Self::EventPosition {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            }
            | Self::FixedSample {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            } => {
                let successor = upper_inclusive.checked_add(1);
                lower <= upper_inclusive
                    && successor == Some(carrier_end_exclusive)
                    && (state == OpenClosed::Open || watermark > upper_inclusive)
            }
            Self::TimestampedEvent {
                lower_nanos,
                upper_inclusive_nanos,
                carrier_end_exclusive_nanos,
                watermark_nanos,
            } => {
                lower_nanos <= upper_inclusive_nanos
                    && carrier_end_exclusive_nanos > upper_inclusive_nanos
                    && (state == OpenClosed::Open || watermark_nanos > upper_inclusive_nanos)
            }
        };
        if valid {
            Ok(())
        } else {
            Err(Error::new(
                ErrorCode::InvalidSelection,
                "boundary does not satisfy the exact FR-289 mapping",
                Usage::default(),
            ))
        }
    }

    pub(crate) fn matches_range(self, range: ClockRange) -> bool {
        match (self, range) {
            (
                Self::EventPosition {
                    lower,
                    carrier_end_exclusive,
                    ..
                },
                ClockRange::EventPosition {
                    start,
                    end_exclusive,
                },
            ) => lower == start && carrier_end_exclusive == end_exclusive,
            (
                Self::FixedSample {
                    lower,
                    carrier_end_exclusive,
                    ..
                },
                ClockRange::FixedSample {
                    start,
                    end_exclusive,
                    ..
                },
            ) => lower == start && carrier_end_exclusive == end_exclusive,
            (
                Self::TimestampedEvent {
                    lower_nanos,
                    carrier_end_exclusive_nanos,
                    ..
                },
                ClockRange::Timestamp {
                    start_nanos,
                    end_nanos,
                },
            ) => lower_nanos == start_nanos && carrier_end_exclusive_nanos == end_nanos,
            _ => false,
        }
    }
}

/// Exact selected definition that owns an assertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritySelection {
    /// Immutable definition identity.
    pub definition_identity: Identity,
    /// Exact selected definition revision.
    pub definition_revision: Identity,
    /// SHA-256 digest of the exact selected definition bytes.
    pub definition_digest: Digest,
}

/// Exact assessment subject retained by every owner artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectSelection {
    /// Exact snapshot or window identity.
    pub scope_identity: Identity,
    /// Exact FR-263 population identity.
    pub population_identity: Identity,
}

/// Complete admitted history. Its constructor is deliberately restricted to
/// batch qualification or a checked incremental intake.
#[derive(Clone, Copy, Debug)]
pub struct History<'a> {
    qualified: &'a QualifiedObservation,
}

impl<'a> History<'a> {
    /// Borrows a complete already-qualified history for batch derivation.
    #[must_use]
    pub const fn batch(qualified: &'a QualifiedObservation) -> Self {
        Self { qualified }
    }

    /// Returns the qualified history backing this capability.
    #[must_use]
    pub const fn qualified(self) -> &'a QualifiedObservation {
        self.qualified
    }
}

/// Checked incremental construction of the same complete history.
#[derive(Debug)]
pub struct IncrementalHistory<'a> {
    qualified: &'a QualifiedObservation,
    next: usize,
}

impl<'a> IncrementalHistory<'a> {
    /// Starts checked incremental intake at the first qualified record.
    #[must_use]
    pub const fn new(qualified: &'a QualifiedObservation) -> Self {
        Self { qualified, next: 0 }
    }

    /// Accepts exactly the next record from the qualified history.
    pub fn push(&mut self, record: &AdmittedRecord) -> Result<()> {
        let Some(expected) = self.qualified.records().get(self.next) else {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "incremental history exceeds qualified record population",
                Usage::default(),
            ));
        };
        if expected != record {
            return Err(Error::new(
                ErrorCode::ExpectedMismatch,
                "incremental record is out of order or cross-wired",
                Usage::default(),
            ));
        }
        self.next += 1;
        Ok(())
    }

    /// Completes intake only after the entire qualified history was supplied.
    pub fn finish(self) -> Result<History<'a>> {
        if self.next != self.qualified.records().len() {
            return Err(Error::new(
                ErrorCode::ExpectedMismatch,
                "incremental history is incomplete",
                Usage::default(),
            ));
        }
        Ok(History {
            qualified: self.qualified,
        })
    }
}

/// Immutable derivation context. Corrections name an exact prior document;
/// callers cannot inject an unrelated predecessor identity.
#[derive(Clone, Copy, Debug)]
pub struct Context<'a> {
    history: History<'a>,
    authority: &'a AuthoritySelection,
    subject: &'a SubjectSelection,
    revision: u64,
    predecessor: Option<&'a Document>,
}

impl<'a> Context<'a> {
    /// Creates an original or correction context from exact owner selections.
    #[must_use]
    pub const fn new(
        history: History<'a>,
        authority: &'a AuthoritySelection,
        subject: &'a SubjectSelection,
        revision: u64,
        predecessor: Option<&'a Document>,
    ) -> Self {
        Self {
            history,
            authority,
            subject,
            revision,
            predecessor,
        }
    }

    /// Returns the complete qualified history.
    #[must_use]
    pub const fn history(self) -> History<'a> {
        self.history
    }

    /// Returns the exact owner definition selection.
    #[must_use]
    pub const fn authority(self) -> &'a AuthoritySelection {
        self.authority
    }

    /// Returns the exact scope and population subject.
    #[must_use]
    pub const fn subject(self) -> &'a SubjectSelection {
        self.subject
    }

    /// Returns the exact owner-document revision being derived.
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }

    pub(crate) const fn predecessor(self) -> Option<&'a Document> {
        self.predecessor
    }
}

/// Canonical immutable owner bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    contract: &'static str,
    identity: Identity,
    revision: u64,
    predecessor: Option<Identity>,
    authority: AuthoritySelection,
    subject: SubjectSelection,
    bytes: Vec<u8>,
    usage: Usage,
}

impl Document {
    /// Returns the immutable owner-contract label.
    #[must_use]
    pub const fn contract(&self) -> &'static str {
        self.contract
    }

    /// Returns the derived envelope identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the positive lineage revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the exact direct predecessor identity for a correction.
    #[must_use]
    pub const fn predecessor(&self) -> Option<&Identity> {
        self.predecessor.as_ref()
    }

    /// Returns the owner definition selection retained by the document.
    #[must_use]
    pub const fn authority(&self) -> &AuthoritySelection {
        &self.authority
    }

    /// Returns the scope and population subject retained by the document.
    #[must_use]
    pub const fn subject(&self) -> &SubjectSelection {
        &self.subject
    }

    /// Returns the canonical immutable bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns exact bounded work recorded by the document.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

/// Strict-reader output. Construction is private to the owner modules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validated<P> {
    contract: &'static str,
    identity: Identity,
    revision: u64,
    predecessor: Option<Identity>,
    authority: AuthoritySelection,
    subject: SubjectSelection,
    payload: P,
    bytes: Vec<u8>,
}

impl<P> Validated<P> {
    /// Returns the validated immutable owner-contract label.
    #[must_use]
    pub const fn contract(&self) -> &'static str {
        self.contract
    }

    /// Returns the validated document identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the validated positive revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the validated direct predecessor identity, when corrected.
    #[must_use]
    pub const fn predecessor(&self) -> Option<&Identity> {
        self.predecessor.as_ref()
    }

    /// Returns the validated owner definition selection.
    #[must_use]
    pub const fn authority(&self) -> &AuthoritySelection {
        &self.authority
    }

    /// Returns the validated scope and population subject.
    #[must_use]
    pub const fn subject(&self) -> &SubjectSelection {
        &self.subject
    }

    /// Returns the validated contract-specific payload.
    #[must_use]
    pub const fn payload(&self) -> &P {
        &self.payload
    }

    /// Returns the exact validated canonical bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityWire {
    definition_identity: String,
    definition_revision: String,
    definition_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectWire {
    scope_identity: String,
    population_identity: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectiveLimitsWire {
    max_input_bytes: u64,
    max_output_bytes: u64,
    max_depth: u64,
    max_string_bytes: u64,
    max_population_entries: u64,
    max_positions: u64,
    max_capture_bindings: u64,
    max_required_sources: u64,
    max_visited_fields: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UsageWire {
    wire_bytes: u64,
    depth: u64,
    string_bytes: u64,
    population_entries: u64,
    positions: u64,
    capture_bindings: u64,
    required_sources: u64,
    visited_fields: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitsWire {
    effective: EffectiveLimitsWire,
    usage: UsageWire,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope<P> {
    contract: String,
    identity: String,
    revision: u64,
    authority: AuthorityWire,
    subject: SubjectWire,
    payload: P,
    predecessor: Option<String>,
    limits: LimitsWire,
}

#[derive(Serialize)]
struct EnvelopeWithoutIdentity<'a, P> {
    contract: &'static str,
    revision: u64,
    authority: &'a AuthorityWire,
    subject: &'a SubjectWire,
    payload: &'a P,
    predecessor: &'a Option<String>,
    limits: LimitsWire,
}

pub(crate) fn build_document<P>(
    contract: &'static str,
    context: Context<'_>,
    payload: &P,
    declared_usage: Usage,
    limits: Limits,
) -> Result<Document>
where
    P: Clone + Serialize,
{
    let effective = limits.effective();
    validate_context(contract, context)?;
    let semantic_floor = Usage {
        population_entries: declared_usage.population_entries,
        positions: declared_usage.positions,
        capture_bindings: declared_usage.capture_bindings,
        required_sources: declared_usage.required_sources,
        bundle_components: declared_usage.bundle_components,
        replacements: declared_usage.replacements,
        conflicts: declared_usage.conflicts,
        lineage_children: declared_usage.lineage_children,
        ..Usage::default()
    };
    let mut usage = semantic_floor;
    validate_usage(usage, effective)?;
    let authority = authority_wire(context.authority);
    let subject = subject_wire(context.subject);
    let predecessor = context
        .predecessor
        .map(|document| document.identity().as_str().to_owned());
    let effective_wire = effective_wire(effective)?;
    for _ in 0..8 {
        let limits_wire = LimitsWire {
            effective: effective_wire,
            usage: usage_wire(usage)?,
        };
        let without_identity = EnvelopeWithoutIdentity {
            contract,
            revision: context.revision,
            authority: &authority,
            subject: &subject,
            payload,
            predecessor: &predecessor,
            limits: limits_wire,
        };
        let identity = derive_envelope_identity(&without_identity, effective.max_output_bytes)?;
        let envelope = Envelope {
            contract: contract.to_owned(),
            identity: identity.as_str().to_owned(),
            revision: context.revision,
            authority: authority.clone(),
            subject: subject.clone(),
            payload: payload.clone(),
            predecessor: predecessor.clone(),
            limits: limits_wire,
        };
        let bytes = to_bounded_json(&envelope, effective.max_output_bytes)?;
        let scan_limits = Limits {
            max_input_bytes: effective.max_output_bytes,
            ..effective
        };
        let observed = preflight_for_contract(&bytes, scan_limits, contract)?;
        let next_usage = Usage {
            wire_bytes: observed.wire_bytes,
            depth: observed.depth,
            string_bytes: observed.string_bytes,
            population_entries: observed
                .population_entries
                .max(semantic_floor.population_entries),
            positions: observed.positions.max(semantic_floor.positions),
            capture_bindings: observed
                .capture_bindings
                .max(semantic_floor.capture_bindings),
            required_sources: observed
                .required_sources
                .max(semantic_floor.required_sources),
            bundle_components: semantic_floor.bundle_components,
            replacements: semantic_floor.replacements,
            conflicts: semantic_floor.conflicts,
            lineage_children: semantic_floor.lineage_children,
            visited_fields: observed.visited_fields,
        };
        if next_usage == usage {
            return Ok(Document {
                contract,
                identity,
                revision: context.revision,
                predecessor: predecessor.map(Identity::new),
                authority: context.authority.clone(),
                subject: context.subject.clone(),
                bytes,
                usage,
            });
        }
        usage = next_usage;
        validate_usage(usage, effective)?;
    }
    Err(Error::new(
        ErrorCode::ResourceIncomplete,
        "wire-size identity did not converge",
        usage,
    ))
}

pub(crate) fn read_exact<P>(
    contract: &'static str,
    bytes: &[u8],
    expected: &Document,
    limits: Limits,
) -> Result<Validated<P>>
where
    P: Clone + Eq + Serialize + DeserializeOwned,
{
    let effective = limits.effective();
    let observed = preflight_for_contract(bytes, effective, contract)?;
    let contract_probe: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        Error::new(
            ErrorCode::InvalidJson,
            "document is not valid JSON",
            observed,
        )
    })?;
    if contract_probe
        .get("contract")
        .and_then(serde_json::Value::as_str)
        != Some(contract)
    {
        return Err(Error::new(
            ErrorCode::ContractMismatch,
            "contract label does not select this owner reader",
            observed,
        ));
    }
    drop(contract_probe);
    let envelope: Envelope<P> = serde_json::from_slice(bytes).map_err(|_| {
        Error::new(
            ErrorCode::InvalidJson,
            "document is not a closed owner envelope",
            observed,
        )
    })?;
    debug_assert_eq!(envelope.contract, contract);
    let canonical = to_bounded_json(&envelope, effective.max_output_bytes)?;
    if canonical != bytes {
        return Err(Error::new(
            ErrorCode::NonCanonical,
            "document bytes are not the canonical closed encoding",
            observed,
        ));
    }
    let without_identity = EnvelopeWithoutIdentity {
        contract,
        revision: envelope.revision,
        authority: &envelope.authority,
        subject: &envelope.subject,
        payload: &envelope.payload,
        predecessor: &envelope.predecessor,
        limits: envelope.limits,
    };
    let recomputed = derive_envelope_identity(&without_identity, effective.max_output_bytes)?;
    if recomputed.as_str() != envelope.identity {
        return Err(Error::new(
            ErrorCode::IdentityMismatch,
            "document identity does not match canonical bytes",
            observed,
        ));
    }
    if expected.contract != contract || expected.bytes != bytes {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "document does not match independently derived authority selections",
            observed,
        ));
    }
    Ok(Validated {
        contract,
        identity: Identity::new(envelope.identity),
        revision: envelope.revision,
        predecessor: envelope.predecessor.map(Identity::new),
        authority: expected.authority.clone(),
        subject: expected.subject.clone(),
        payload: envelope.payload,
        bytes: canonical,
    })
}

fn validate_context(contract: &'static str, context: Context<'_>) -> Result<()> {
    let qualified = context.history.qualified;
    if !context.authority.definition_identity.valid()
        || !context.authority.definition_revision.valid()
        || !context.subject.scope_identity.valid()
        || !context.subject.population_identity.valid()
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "authority and subject identities must be explicit",
            Usage::default(),
        ));
    }
    if context.subject.population_identity != qualified.scope().population_identity
        || context.subject.scope_identity.as_str() != scope_identity(qualified.scope().kind.clone())
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "subject is cross-wired from qualified scope or population",
            Usage::default(),
        ));
    }
    match context.predecessor {
        None if context.revision == 1 => {}
        Some(prior)
            if context.revision > prior.revision
                && prior.contract == contract
                && prior.authority == *context.authority
                && prior.subject == *context.subject
                && prior.identity.valid() => {}
        None => {
            return Err(Error::new(
                ErrorCode::RevisionMismatch,
                "an original document has revision one",
                Usage::default(),
            ));
        }
        Some(_) => {
            return Err(Error::new(
                ErrorCode::PredecessorMismatch,
                "a correction requires a lower-revision predecessor of the same contract",
                Usage::default(),
            ));
        }
    }
    Ok(())
}

fn validate_usage(usage: Usage, limits: Limits) -> Result<()> {
    let exceeded = usage.wire_bytes > limits.max_output_bytes
        || usage.depth > limits.max_depth
        || usage.string_bytes > limits.max_string_bytes
        || usage.population_entries > limits.max_population_entries
        || usage.positions > limits.max_positions
        || usage.capture_bindings > limits.max_capture_bindings
        || usage.required_sources > limits.max_required_sources
        || usage.bundle_components > limits.max_bundle_components
        || usage.replacements > limits.max_replacements
        || usage.conflicts > limits.max_conflicts
        || usage.lineage_children > limits.max_lineage_children
        || usage.visited_fields > limits.max_visited_fields;
    if exceeded {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "owner derivation exceeded an effective resource limit",
            usage,
        ));
    }
    Ok(())
}

fn derive_envelope_identity<P: Serialize>(
    without_identity: &EnvelopeWithoutIdentity<'_, P>,
    max_output_bytes: usize,
) -> Result<Identity> {
    let bytes = to_bounded_json(without_identity, max_output_bytes)?;
    let mut hasher = Sha256::new();
    hasher.update(without_identity.contract.as_bytes());
    hasher.update([0]);
    hasher.update(bytes);
    Ok(Identity::new(format!("sha256:{:x}", hasher.finalize())))
}

pub(crate) fn sha256_jcs<T: Serialize>(value: &T, limits: Limits) -> Result<(Identity, Digest)> {
    let effective = limits.effective();
    let direct = to_bounded_json(value, effective.max_output_bytes)?;
    preflight(
        &direct,
        Limits {
            max_input_bytes: effective.max_output_bytes,
            ..effective
        },
    )?;
    // The owner preimages contain only integers encoded as decimal strings,
    // strings, arrays, nulls, and objects. Converting through `Value` sorts all
    // object member names lexicographically (serde_json's non-preserve_order
    // map), which is the RFC 8785 ordering for this closed value domain.
    let canonical_value = serde_json::to_value(value).map_err(|_| {
        Error::new(
            ErrorCode::InvalidSelection,
            "identity preimage is outside the closed JCS value domain",
            Usage::default(),
        )
    })?;
    let bytes = to_bounded_json(&canonical_value, effective.max_output_bytes)?;
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    Ok((
        Identity::new(format!("sha256-jcs:{}", hex(&digest))),
        Digest::new(digest),
    ))
}

#[cfg(test)]
pub(crate) fn schema_sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

#[cfg(test)]
pub(crate) fn assert_closed_schema(bytes: &[u8], contract: &str) {
    let schema: serde_json::Value = serde_json::from_slice(bytes).expect("schema is JSON");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["contract"]["const"], contract);
    assert_eq!(
        schema["required"].as_array().map(Vec::len),
        Some(8),
        "owner envelope has exactly eight required fields"
    );
}

pub(crate) fn digest_hex(digest: &Digest) -> String {
    hex(digest.as_bytes())
}

fn hex(bytes: &[u8]) -> String {
    use fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

pub(crate) fn scope_identity(kind: ScopeKind) -> String {
    match kind {
        ScopeKind::Snapshot { snapshot_identity } => snapshot_identity.as_str().to_owned(),
        ScopeKind::Window { window_identity } => window_identity.as_str().to_owned(),
    }
}

pub(crate) fn ensure_sorted_unique(values: &[Identity], limit: usize) -> Result<()> {
    if values.len() > limit {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "identity population exceeds its effective limit",
            Usage {
                population_entries: values.len(),
                ..Usage::default()
            },
        ));
    }
    if values.iter().any(|value| !value.valid()) || values.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "identity population must be explicit, sorted and distinct",
            Usage::default(),
        ));
    }
    Ok(())
}

fn authority_wire(value: &AuthoritySelection) -> AuthorityWire {
    AuthorityWire {
        definition_identity: value.definition_identity.as_str().to_owned(),
        definition_revision: value.definition_revision.as_str().to_owned(),
        definition_digest: digest_hex(&value.definition_digest),
    }
}

fn subject_wire(value: &SubjectSelection) -> SubjectWire {
    SubjectWire {
        scope_identity: value.scope_identity.as_str().to_owned(),
        population_identity: value.population_identity.as_str().to_owned(),
    }
}

fn effective_wire(value: Limits) -> Result<EffectiveLimitsWire> {
    Ok(EffectiveLimitsWire {
        max_input_bytes: wire_usize(value.max_input_bytes)?,
        max_output_bytes: wire_usize(value.max_output_bytes)?,
        max_depth: wire_usize(value.max_depth)?,
        max_string_bytes: wire_usize(value.max_string_bytes)?,
        max_population_entries: wire_usize(value.max_population_entries)?,
        max_positions: wire_usize(value.max_positions)?,
        max_capture_bindings: wire_usize(value.max_capture_bindings)?,
        max_required_sources: wire_usize(value.max_required_sources)?,
        max_visited_fields: wire_usize(value.max_visited_fields)?,
    })
}

fn usage_wire(value: Usage) -> Result<UsageWire> {
    Ok(UsageWire {
        wire_bytes: wire_usize(value.wire_bytes)?,
        depth: wire_usize(value.depth)?,
        string_bytes: wire_usize(value.string_bytes)?,
        population_entries: wire_usize(value.population_entries)?,
        positions: wire_usize(value.positions)?,
        capture_bindings: wire_usize(value.capture_bindings)?,
        required_sources: wire_usize(value.required_sources)?,
        visited_fields: wire_usize(value.visited_fields)?,
    })
}

fn wire_usize(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "resource counter is not representable on the wire",
            Usage::default(),
        )
    })
}

pub(crate) fn to_bounded_json<T: Serialize>(value: &T, limit: usize) -> Result<Vec<u8>> {
    let mut writer = BoundedWriter::new(limit);
    serde_json::to_writer(&mut writer, value).map_err(|_| {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "canonical output exceeded its byte limit",
            Usage {
                wire_bytes: writer.bytes.len(),
                ..Usage::default()
            },
        )
    })?;
    Ok(writer.bytes)
}

struct BoundedWriter {
    bytes: Vec<u8>,
    limit: usize,
}

impl BoundedWriter {
    const fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
}

impl Write for BoundedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let Some(next) = self.bytes.len().checked_add(buffer.len()) else {
            return Err(io::Error::other("canonical output size overflow"));
        };
        if next > self.limit {
            return Err(io::Error::other("canonical output limit exceeded"));
        }
        self.bytes
            .try_reserve(buffer.len())
            .map_err(|_| io::Error::other("canonical output allocation failed"))?;
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ScanState {
    usage: Usage,
}

pub(crate) fn preflight(bytes: &[u8], limits: Limits) -> Result<Usage> {
    preflight_with_profile(bytes, limits, ScanProfile::Generic)
}

pub(crate) fn preflight_for_contract(
    bytes: &[u8],
    limits: Limits,
    contract: &str,
) -> Result<Usage> {
    let profile = match contract {
        "quire.observation-authority/v1" | "quire.observation-authority/v2" => ScanProfile::Bundle,
        "quire.observation-authority-lineage/v1" => ScanProfile::Lineage,
        _ => ScanProfile::Generic,
    };
    preflight_with_profile(bytes, limits, profile)
}

fn preflight_with_profile(bytes: &[u8], limits: Limits, profile: ScanProfile) -> Result<Usage> {
    if bytes.len() > limits.max_input_bytes {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "input exceeds effective byte limit",
            Usage {
                wire_bytes: bytes.len(),
                ..Usage::default()
            },
        ));
    }
    std::str::from_utf8(bytes).map_err(|_| {
        Error::new(
            ErrorCode::InvalidUtf8,
            "input is not valid UTF-8",
            Usage::default(),
        )
    })?;
    let mut scanner = Scanner {
        bytes,
        position: 0,
        limits,
        profile,
        state: ScanState::default(),
    };
    scanner.value(1, ArrayKind::Other)?;
    scanner.whitespace();
    if scanner.position != bytes.len() {
        return Err(Error::new(
            ErrorCode::InvalidJson,
            "trailing data follows the owner document",
            scanner.state.usage,
        ));
    }
    scanner.state.usage.wire_bytes = bytes.len();
    Ok(scanner.state.usage)
}

#[derive(Clone, Copy)]
enum ArrayKind {
    Population,
    Positions,
    Captures,
    Sources,
    BundleComponents,
    Replacements,
    Conflicts,
    LineageChildren,
    Other,
}

#[derive(Clone, Copy)]
enum ScanProfile {
    Generic,
    Bundle,
    Lineage,
}

struct Scanner<'a> {
    bytes: &'a [u8],
    position: usize,
    limits: Limits,
    profile: ScanProfile,
    state: ScanState,
}

impl Scanner<'_> {
    fn value(&mut self, depth: usize, kind: ArrayKind) -> Result<()> {
        self.whitespace();
        self.charge_depth(depth)?;
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth, kind),
            Some(b'"') => self.string().map(|_| ()),
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => self.invalid("invalid JSON value"),
        }
    }

    fn object(&mut self, depth: usize) -> Result<()> {
        self.position += 1;
        self.whitespace();
        if self.take(b'}') {
            return Ok(());
        }
        loop {
            let kind = {
                let profile = self.profile;
                let key = self.string()?;
                array_kind(key, depth, profile)
            };
            self.charge_visit()?;
            self.whitespace();
            if !self.take(b':') {
                return self.invalid("object key is missing a colon");
            }
            self.value(depth + 1, kind)?;
            self.whitespace();
            if self.take(b'}') {
                return Ok(());
            }
            if !self.take(b',') {
                return self.invalid("object member is not comma separated");
            }
            self.whitespace();
        }
    }

    fn array(&mut self, depth: usize, kind: ArrayKind) -> Result<()> {
        self.position += 1;
        self.whitespace();
        let mut count = 0usize;
        if self.take(b']') {
            self.charge_array(kind, count)?;
            return Ok(());
        }
        loop {
            count = count.checked_add(1).ok_or_else(|| {
                Error::new(
                    ErrorCode::ResourceIncomplete,
                    "array count overflow",
                    self.state.usage,
                )
            })?;
            self.charge_visit()?;
            self.charge_array(kind, count)?;
            self.value(depth + 1, ArrayKind::Other)?;
            self.whitespace();
            if self.take(b']') {
                return Ok(());
            }
            if !self.take(b',') {
                return self.invalid("array member is not comma separated");
            }
            self.whitespace();
        }
    }

    fn string(&mut self) -> Result<&[u8]> {
        if !self.take(b'"') {
            return self.invalid("expected JSON string");
        }
        let start = self.position;
        while let Some(byte) = self.peek() {
            match byte {
                b'"' => {
                    let end = self.position;
                    self.position += 1;
                    let length = end - start;
                    self.state.usage.string_bytes = self.state.usage.string_bytes.max(length);
                    if length > self.limits.max_string_bytes {
                        return Err(Error::new(
                            ErrorCode::ResourceIncomplete,
                            "string exceeds effective byte limit",
                            self.state.usage,
                        ));
                    }
                    return Ok(&self.bytes[start..end]);
                }
                b'\\' => {
                    self.position += 1;
                    let Some(escape) = self.peek() else {
                        return self.invalid("unterminated JSON escape");
                    };
                    self.position += 1;
                    if escape == b'u' {
                        for _ in 0..4 {
                            if !matches!(self.peek(), Some(b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F'))
                            {
                                return self.invalid("invalid Unicode escape");
                            }
                            self.position += 1;
                        }
                    } else if !matches!(
                        escape,
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't'
                    ) {
                        return self.invalid("invalid JSON escape");
                    }
                }
                0x00..=0x1f => return self.invalid("control byte in JSON string"),
                _ => self.position += 1,
            }
        }
        self.invalid("unterminated JSON string")
    }

    fn number(&mut self) -> Result<()> {
        let start = self.position;
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.position += 1;
        }
        if self.position == start {
            self.invalid("invalid JSON number")
        } else {
            Ok(())
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<()> {
        if self.bytes.get(self.position..self.position + literal.len()) == Some(literal) {
            self.position += literal.len();
            Ok(())
        } else {
            self.invalid("invalid JSON literal")
        }
    }

    fn charge_depth(&mut self, depth: usize) -> Result<()> {
        self.state.usage.depth = self.state.usage.depth.max(depth);
        if depth > self.limits.max_depth {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "JSON depth exceeds effective limit",
                self.state.usage,
            ));
        }
        Ok(())
    }

    fn charge_visit(&mut self) -> Result<()> {
        self.state.usage.visited_fields = self
            .state
            .usage
            .visited_fields
            .checked_add(1)
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::ResourceIncomplete,
                    "visited-field count overflow",
                    self.state.usage,
                )
            })?;
        if self.state.usage.visited_fields > self.limits.max_visited_fields {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "visited fields exceed effective limit",
                self.state.usage,
            ));
        }
        Ok(())
    }

    fn charge_array(&mut self, kind: ArrayKind, count: usize) -> Result<()> {
        let (slot, limit, detail) = match kind {
            ArrayKind::Population => (
                &mut self.state.usage.population_entries,
                self.limits.max_population_entries,
                "population exceeds effective limit",
            ),
            ArrayKind::Positions => (
                &mut self.state.usage.positions,
                self.limits.max_positions,
                "positions exceed effective limit",
            ),
            ArrayKind::Captures => (
                &mut self.state.usage.capture_bindings,
                self.limits.max_capture_bindings,
                "capture bindings exceed effective limit",
            ),
            ArrayKind::Sources => (
                &mut self.state.usage.required_sources,
                self.limits.max_required_sources,
                "required sources exceed effective limit",
            ),
            ArrayKind::BundleComponents => (
                &mut self.state.usage.bundle_components,
                self.limits.max_bundle_components,
                "bundle components exceed effective limit",
            ),
            ArrayKind::Replacements => (
                &mut self.state.usage.replacements,
                self.limits.max_replacements,
                "bundle replacements exceed effective limit",
            ),
            ArrayKind::Conflicts => (
                &mut self.state.usage.conflicts,
                self.limits.max_conflicts,
                "bundle conflicts exceed effective limit",
            ),
            ArrayKind::LineageChildren => (
                &mut self.state.usage.lineage_children,
                self.limits.max_lineage_children,
                "lineage children exceed effective limit",
            ),
            ArrayKind::Other => return Ok(()),
        };
        *slot = (*slot).max(count);
        if count > limit {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                detail,
                self.state.usage,
            ));
        }
        Ok(())
    }

    fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn take(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn invalid<T>(&self, detail: &'static str) -> Result<T> {
        Err(Error::new(ErrorCode::InvalidJson, detail, self.state.usage))
    }
}

fn array_kind(key: &[u8], object_depth: usize, profile: ScanProfile) -> ArrayKind {
    match key {
        b"members" | b"required_members" | b"required_results" | b"available_results"
        | b"facts" => ArrayKind::Population,
        b"positions" if matches!(profile, ScanProfile::Bundle) && object_depth == 2 => {
            ArrayKind::BundleComponents
        }
        b"positions" => ArrayKind::Positions,
        b"bindings" => ArrayKind::Captures,
        b"required_sources" | b"observation_sources" | b"sources" => ArrayKind::Sources,
        b"components" | b"records" | b"populations" | b"progress"
            if matches!(profile, ScanProfile::Bundle) && object_depth == 2 =>
        {
            ArrayKind::BundleComponents
        }
        b"replacements" if matches!(profile, ScanProfile::Bundle) && object_depth == 2 => {
            ArrayKind::Replacements
        }
        b"conflicts" if matches!(profile, ScanProfile::Bundle) && object_depth == 2 => {
            ArrayKind::Conflicts
        }
        b"direct_children" if matches!(profile, ScanProfile::Lineage) && object_depth == 1 => {
            ArrayKind::LineageChildren
        }
        _ => ArrayKind::Other,
    }
}
