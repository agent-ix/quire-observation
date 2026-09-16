// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Canonical revisioned observation-authority v1 and empty-population v2 bundles.

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use super::common::{
    build_document, digest_hex, preflight_for_contract, read_exact, read_exact_preflighted,
    to_bounded_json, Validated,
};
use super::{
    AuthoritySelection, Context, Document, Error, ErrorCode, Limits, Result, SubjectSelection,
    Usage,
};
use crate::{Digest, Identity};

/// Immutable bundle-contract label.
pub const CONTRACT: &str = "quire.observation-authority/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-authority-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "9614d8077bd1e667507b2107eab08e0f7934337115ce3e10c8c55b5d5538cdab";
/// Immutable structurally empty bundle-contract label.
pub const V2_CONTRACT: &str = "quire.observation-authority/v2";
/// Pinned JSON Schema bytes for [`V2_CONTRACT`].
pub const V2_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-authority-v2.schema.json");
/// Lowercase SHA-256 digest of [`V2_SCHEMA_BYTES`].
pub const V2_SCHEMA_SHA256: &str =
    "f92335c5e5e1e3434f1c2484557ca1f512c22b496eb97319c94fcac3491f116a";
/// Immutable lineage-view contract label.
pub const LINEAGE_CONTRACT: &str = "quire.observation-authority-lineage/v1";
/// Pinned JSON Schema bytes for [`LINEAGE_CONTRACT`].
pub const LINEAGE_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-authority-lineage-v1.schema.json");
/// Lowercase SHA-256 digest of [`LINEAGE_SCHEMA_BYTES`].
pub const LINEAGE_SCHEMA_SHA256: &str =
    "d58c70516e53bde40ab2d276df143fffb05740348b98d09a71e89857a82a22ef";

/// Closed semantic role vocabulary for a complete I07 authority bundle.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentRole {
    /// Canonical observation record.
    Observation,
    /// Exact population.
    Population,
    /// Declared order positions.
    Position,
    /// Selected clock authority.
    Clock,
    /// Partial-value fact.
    Partial,
    /// Capture fact.
    Capture,
    /// Progress authority.
    Progress,
    /// Activation authority.
    Activation,
    /// Scope closure proof.
    Closure,
    /// Completeness proof.
    Completeness,
    /// Result availability fact.
    Availability,
}

impl ComponentRole {
    const ALL: [Self; 11] = [
        Self::Observation,
        Self::Population,
        Self::Position,
        Self::Clock,
        Self::Partial,
        Self::Capture,
        Self::Progress,
        Self::Activation,
        Self::Closure,
        Self::Completeness,
        Self::Availability,
    ];

    const EMPTY_V2: [Self; 7] = [
        Self::Population,
        Self::Position,
        Self::Clock,
        Self::Progress,
        Self::Closure,
        Self::Completeness,
        Self::Availability,
    ];

    const fn contract(self) -> &'static str {
        match self {
            Self::Observation => super::observation::CONTRACT,
            Self::Population => super::population::CONTRACT,
            Self::Position => super::position::CONTRACT,
            Self::Clock => super::clock::CONTRACT,
            Self::Partial => super::partial::CONTRACT,
            Self::Capture => super::capture::CONTRACT,
            Self::Progress => super::progress::CONTRACT,
            Self::Activation => super::activation::CONTRACT,
            Self::Closure => super::closure::CONTRACT,
            Self::Completeness => super::completeness::CONTRACT,
            Self::Availability => super::availability::CONTRACT,
        }
    }
}

#[derive(Clone, Copy)]
enum Profile {
    CompleteV1,
    EmptyV2,
}

/// Role-qualified immutable component, constructible only from an owner document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Component {
    wire: ComponentWire,
    authority: AuthoritySelection,
    subject: SubjectSelection,
    fact_subject: FactSubjectWire,
    observation_identity: Option<String>,
    payload: EmbeddedPayloadWire,
    canonical_document: String,
    document_bytes: usize,
}

impl Component {
    /// Qualifies an owner document for one exact bundle role.
    pub fn from_document(role: ComponentRole, document: &Document) -> Result<Self> {
        if document.contract() != role.contract() {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "component contract does not match its semantic role",
                Usage::default(),
            ));
        }
        let digest: [u8; 32] = Sha256::digest(document.bytes()).into();
        let document_value: serde_json::Value =
            serde_json::from_slice(document.bytes()).map_err(|_| {
                Error::new(
                    ErrorCode::InvalidJson,
                    "owner component document is not valid canonical JSON",
                    document.usage(),
                )
            })?;
        let (semantic_key, observation_identity) = component_subject(role, &document_value)?;
        let payload = typed_payload(role, &document_value)?;
        let canonical_document = std::str::from_utf8(document.bytes())
            .map_err(|_| {
                Error::new(
                    ErrorCode::InvalidUtf8,
                    "owner component document is not UTF-8",
                    document.usage(),
                )
            })?
            .to_owned();
        Ok(Self {
            wire: ComponentWire {
                role,
                contract: document.contract().to_owned(),
                identity: document.identity().as_str().to_owned(),
                digest: digest_hex(&Digest::new(digest)),
                revision: document.revision(),
            },
            authority: document.authority().clone(),
            subject: document.subject().clone(),
            fact_subject: FactSubjectWire {
                scope_identity: document.subject().scope_identity.as_str().to_owned(),
                population_identity: document.subject().population_identity.as_str().to_owned(),
                semantic_key,
            },
            observation_identity,
            payload,
            canonical_document,
            document_bytes: document.bytes().len(),
        })
    }

    /// Returns the component's semantic role.
    #[must_use]
    pub const fn role(&self) -> ComponentRole {
        self.wire.role
    }

    /// Returns the exact owner-document identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentWire {
    role: ComponentRole,
    contract: String,
    identity: String,
    digest: String,
    revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactSubjectWire {
    scope_identity: String,
    population_identity: String,
    semantic_key: serde_json::Value,
}

/// One explicit same-role component supersession.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replacement {
    wire: ReplacementWire,
}

/// Closed conflict vocabulary retained as explicit bundle input.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictKind {
    /// More than one candidate claims the same authority-qualified fact subject.
    Duplicate,
    /// Selected evidence makes mutually incompatible claims.
    Contradiction,
    /// The selected authority cannot resolve a required correlation.
    UnresolvedCorrelation,
}

/// One explicit conflict fact anchored to a selected authority component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conflict {
    wire: ConflictWire,
    authority: AuthoritySelection,
    subject: SubjectSelection,
}

impl Conflict {
    /// Constructs a conflict from its closed kind and exact carrying component.
    #[must_use]
    pub fn new(kind: ConflictKind, component: &Component) -> Self {
        Self {
            wire: ConflictWire {
                kind,
                component_identity: component.identity().to_owned(),
                observation_identity: component.observation_identity.clone(),
            },
            authority: component.authority.clone(),
            subject: component.subject.clone(),
        }
    }
}

impl Replacement {
    /// Constructs a replacement after checking its role and qualified subject.
    pub fn new(prior: &Component, successor: &Component) -> Result<Self> {
        if prior.role() != successor.role()
            || prior.authority != successor.authority
            || prior.subject != successor.subject
            || prior.fact_subject != successor.fact_subject
            || prior.observation_identity != successor.observation_identity
            || prior.identity() == successor.identity()
        {
            return Err(Error::new(
                ErrorCode::InvalidReplacement,
                "replacement must change one same-role authority-qualified component",
                Usage::default(),
            ));
        }
        Ok(Self {
            wire: ReplacementWire {
                role: prior.role(),
                prior_identity: prior.identity().to_owned(),
                successor_identity: successor.identity().to_owned(),
            },
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplacementWire {
    role: ComponentRole,
    prior_identity: String,
    successor_identity: String,
}

/// Complete caller selection for one bundle revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    components: Vec<Component>,
    replacements: Vec<Replacement>,
    conflicts: Vec<Conflict>,
}

impl Selection {
    /// Constructs an explicit component and replacement selection.
    #[must_use]
    pub const fn new(components: Vec<Component>, replacements: Vec<Replacement>) -> Self {
        Self {
            components,
            replacements,
            conflicts: Vec::new(),
        }
    }

    /// Constructs a selection with explicit closed conflict facts.
    #[must_use]
    pub const fn with_conflicts(
        components: Vec<Component>,
        replacements: Vec<Replacement>,
        conflicts: Vec<Conflict>,
    ) -> Self {
        Self {
            components,
            replacements,
            conflicts,
        }
    }

    /// Returns the selected complete component set.
    #[must_use]
    pub fn components(&self) -> &[Component] {
        &self.components
    }

    /// Returns the explicit replacement edges.
    #[must_use]
    pub fn replacements(&self) -> &[Replacement] {
        &self.replacements
    }

    /// Returns the explicit conflict facts.
    #[must_use]
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }
}

/// Read-only bundle payload exposed only through a validated view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleView {
    components: Vec<ComponentWire>,
    replacements: Vec<ReplacementWire>,
    records: Vec<EmbeddedFactWire>,
    populations: Vec<EmbeddedFactWire>,
    positions: Vec<EmbeddedFactWire>,
    progress: Vec<EmbeddedFactWire>,
    conflicts: Vec<ConflictWire>,
    bounds: BundleBoundsWire,
    usage: BundleUsageWire,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmbeddedFactWire {
    role: ComponentRole,
    fact_subject: FactSubjectWire,
    observation_identity: Option<String>,
    payload: EmbeddedPayloadWire,
    canonical_document: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
enum EmbeddedPayloadWire {
    Observation(Box<super::observation::RecordView>),
    Population(Box<super::population::PopulationView>),
    Position(Box<super::position::PositionLedgerView>),
    Clock(Box<super::clock::ClockView>),
    Partial(Box<super::partial::PartialView>),
    Capture(Box<super::capture::CaptureView>),
    Progress(Box<super::progress::ProgressView>),
    Activation(Box<super::activation::ActivationView>),
    Closure(Box<super::closure::ClosureView>),
    Completeness(Box<super::completeness::CompletenessView>),
    Availability(Box<super::availability::AvailabilityView>),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConflictWire {
    kind: ConflictKind,
    component_identity: String,
    observation_identity: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleBoundsWire {
    max_components: u64,
    max_replacements: u64,
    max_conflicts: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleUsageWire {
    components: u64,
    replacements: u64,
    conflicts: u64,
}

impl BundleView {
    /// Returns the canonical complete component set.
    #[must_use]
    pub fn components(&self) -> impl ExactSizeIterator<Item = ComponentRef<'_>> + '_ {
        self.components.iter().map(ComponentRef)
    }

    /// Returns the canonical declared replacements.
    #[must_use]
    pub fn replacements(&self) -> impl ExactSizeIterator<Item = ReplacementRef<'_>> + '_ {
        self.replacements.iter().map(ReplacementRef)
    }

    /// Returns the typed record/value/availability facts without ambient lookup.
    #[must_use]
    pub fn records(&self) -> impl ExactSizeIterator<Item = EmbeddedFactRef<'_>> + '_ {
        self.records.iter().map(EmbeddedFactRef)
    }

    /// Returns population membership and completeness facts.
    #[must_use]
    pub fn populations(&self) -> impl ExactSizeIterator<Item = EmbeddedFactRef<'_>> + '_ {
        self.populations.iter().map(EmbeddedFactRef)
    }

    /// Returns semantic position and clock-coordinate facts.
    #[must_use]
    pub fn positions(&self) -> impl ExactSizeIterator<Item = EmbeddedFactRef<'_>> + '_ {
        self.positions.iter().map(EmbeddedFactRef)
    }

    /// Returns progress and closure facts.
    #[must_use]
    pub fn progress(&self) -> impl ExactSizeIterator<Item = EmbeddedFactRef<'_>> + '_ {
        self.progress.iter().map(EmbeddedFactRef)
    }

    /// Returns explicit conflict projections.
    #[must_use]
    pub fn conflicts(&self) -> impl ExactSizeIterator<Item = ConflictRef<'_>> + '_ {
        self.conflicts.iter().map(ConflictRef)
    }

    /// Returns the effective component ceiling retained in canonical bytes.
    #[must_use]
    pub const fn max_components(&self) -> u64 {
        self.bounds.max_components
    }

    /// Returns the effective replacement ceiling retained in canonical bytes.
    #[must_use]
    pub const fn max_replacements(&self) -> u64 {
        self.bounds.max_replacements
    }

    /// Returns the effective conflict ceiling retained in canonical bytes.
    #[must_use]
    pub const fn max_conflicts(&self) -> u64 {
        self.bounds.max_conflicts
    }
}

/// Borrowed typed I07 fact embedded in canonical bundle bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddedFactRef<'a>(&'a EmbeddedFactWire);

impl<'a> EmbeddedFactRef<'a> {
    /// Returns the semantic role.
    #[must_use]
    pub const fn role(self) -> ComponentRole {
        self.0.role
    }

    /// Returns the authority-qualified fact subject as closed JSON.
    #[must_use]
    pub const fn fact_subject(self) -> &'a serde_json::Value {
        &self.0.fact_subject.semantic_key
    }

    /// Returns the authority-qualified scope identity.
    #[must_use]
    pub fn scope_identity(self) -> &'a str {
        &self.0.fact_subject.scope_identity
    }

    /// Returns the authority-qualified population identity.
    #[must_use]
    pub fn population_identity(self) -> &'a str {
        &self.0.fact_subject.population_identity
    }

    /// Returns the exact observation identity when the fact names one.
    #[must_use]
    pub fn observation_identity(self) -> Option<&'a str> {
        self.0.observation_identity.as_deref()
    }

    /// Returns the complete typed owner payload embedded in the bundle.
    #[must_use]
    pub fn payload(self) -> EmbeddedPayloadRef<'a> {
        match &self.0.payload {
            EmbeddedPayloadWire::Observation(value) => {
                EmbeddedPayloadRef::Observation(value.as_ref())
            }
            EmbeddedPayloadWire::Population(value) => {
                EmbeddedPayloadRef::Population(value.as_ref())
            }
            EmbeddedPayloadWire::Position(value) => EmbeddedPayloadRef::Position(value.as_ref()),
            EmbeddedPayloadWire::Clock(value) => EmbeddedPayloadRef::Clock(value.as_ref()),
            EmbeddedPayloadWire::Partial(value) => EmbeddedPayloadRef::Partial(value.as_ref()),
            EmbeddedPayloadWire::Capture(value) => EmbeddedPayloadRef::Capture(value.as_ref()),
            EmbeddedPayloadWire::Progress(value) => EmbeddedPayloadRef::Progress(value.as_ref()),
            EmbeddedPayloadWire::Activation(value) => {
                EmbeddedPayloadRef::Activation(value.as_ref())
            }
            EmbeddedPayloadWire::Closure(value) => EmbeddedPayloadRef::Closure(value.as_ref()),
            EmbeddedPayloadWire::Completeness(value) => {
                EmbeddedPayloadRef::Completeness(value.as_ref())
            }
            EmbeddedPayloadWire::Availability(value) => {
                EmbeddedPayloadRef::Availability(value.as_ref())
            }
        }
    }

    /// Returns the exact canonical owner-document bytes retained for strict reading.
    #[must_use]
    pub fn canonical_document_bytes(self) -> &'a [u8] {
        self.0.canonical_document.as_bytes()
    }
}

/// Borrowed closed typed payload of one embedded owner fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmbeddedPayloadRef<'a> {
    /// Observation record payload.
    Observation(&'a super::observation::RecordView),
    /// Population payload.
    Population(&'a super::population::PopulationView),
    /// Position-ledger payload.
    Position(&'a super::position::PositionLedgerView),
    /// Clock payload.
    Clock(&'a super::clock::ClockView),
    /// Partial-fact payload.
    Partial(&'a super::partial::PartialView),
    /// Capture payload.
    Capture(&'a super::capture::CaptureView),
    /// Progress payload.
    Progress(&'a super::progress::ProgressView),
    /// Activation payload.
    Activation(&'a super::activation::ActivationView),
    /// Closure payload.
    Closure(&'a super::closure::ClosureView),
    /// Completeness payload.
    Completeness(&'a super::completeness::CompletenessView),
    /// Availability payload.
    Availability(&'a super::availability::AvailabilityView),
}

/// Borrowed explicit conflict projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConflictRef<'a>(&'a ConflictWire);

impl<'a> ConflictRef<'a> {
    /// Returns the closed conflict kind.
    #[must_use]
    pub const fn kind(self) -> ConflictKind {
        self.0.kind
    }

    /// Returns the exact component carrying the conflict.
    #[must_use]
    pub fn component_identity(self) -> &'a str {
        &self.0.component_identity
    }

    /// Returns the observation identity carrying the conflict, when present.
    #[must_use]
    pub fn observation_identity(self) -> Option<&'a str> {
        self.0.observation_identity.as_deref()
    }
}

/// Borrowed validated component data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentRef<'a>(&'a ComponentWire);

impl<'a> ComponentRef<'a> {
    /// Returns the semantic role.
    #[must_use]
    pub const fn role(self) -> ComponentRole {
        self.0.role
    }

    /// Returns the exact component identity.
    #[must_use]
    pub fn identity(self) -> &'a str {
        &self.0.identity
    }

    /// Returns the exact component contract.
    #[must_use]
    pub fn contract(self) -> &'a str {
        &self.0.contract
    }

    /// Returns the SHA-256 digest of the exact canonical component bytes.
    #[must_use]
    pub fn digest(self) -> &'a str {
        &self.0.digest
    }

    /// Returns the retained owner-document revision.
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.0.revision
    }
}

/// Borrowed validated replacement data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplacementRef<'a>(&'a ReplacementWire);

impl<'a> ReplacementRef<'a> {
    /// Returns the replaced semantic role.
    #[must_use]
    pub const fn role(self) -> ComponentRole {
        self.0.role
    }

    /// Returns the exact superseded component identity.
    #[must_use]
    pub fn prior_identity(self) -> &'a str {
        &self.0.prior_identity
    }

    /// Returns the exact successor component identity.
    #[must_use]
    pub fn successor_identity(self) -> &'a str {
        &self.0.successor_identity
    }
}

/// Constructor-private validated bundle view.
pub type View = Validated<BundleView>;

/// Bounded current-head view and all known direct children of that head.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageView {
    document: LineageDocument,
    head: View,
    direct_children: Vec<BundleStamp>,
}

/// Canonical immutable bytes for one supplied bounded lineage view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineageDocument {
    identity: Identity,
    bytes: Vec<u8>,
    usage: Usage,
}

impl LineageDocument {
    /// Returns the content-derived lineage identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the exact canonical lineage bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns exact bounded work recorded while producing the lineage.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LineageAuthorityWire {
    definition_identity: String,
    definition_revision: String,
    definition_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LineageSubjectWire {
    scope_identity: String,
    population_identity: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleStamp {
    revision: u64,
    identity: String,
    digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LineageEnvelope {
    contract: String,
    identity: String,
    authority: LineageAuthorityWire,
    subject: LineageSubjectWire,
    head: BundleStamp,
    direct_children: Vec<BundleStamp>,
    max_direct_children: u64,
}

#[derive(Serialize)]
struct LineageEnvelopeWithoutIdentity<'a> {
    contract: &'static str,
    authority: &'a LineageAuthorityWire,
    subject: &'a LineageSubjectWire,
    head: &'a BundleStamp,
    direct_children: &'a [BundleStamp],
    max_direct_children: u64,
}

impl LineageView {
    /// Builds a bounded view from already strict-read bundle views.
    pub fn from_views(head: &View, direct_children: &[View], limits: Limits) -> Result<Self> {
        let effective = limits.effective();
        if direct_children.len() > effective.max_lineage_children {
            return Err(resource_error(0, 0, 0, direct_children.len()));
        }
        let source_bytes = direct_children
            .iter()
            .try_fold(head.bytes().len(), |total, child| {
                total.checked_add(child.bytes().len())
            });
        let Some(source_bytes) = source_bytes else {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "lineage source byte count overflowed",
                Usage {
                    wire_bytes: usize::MAX,
                    lineage_children: direct_children.len(),
                    ..Usage::default()
                },
            ));
        };
        if source_bytes > effective.max_input_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "lineage source bytes exceed the effective input bound",
                Usage {
                    wire_bytes: source_bytes,
                    lineage_children: direct_children.len(),
                    ..Usage::default()
                },
            ));
        }
        for child in direct_children {
            if child.contract() != head.contract()
                || child.authority() != head.authority()
                || child.subject() != head.subject()
                || child.predecessor() != Some(head.identity())
                || child.revision() <= head.revision()
            {
                return Err(Error::new(
                    ErrorCode::InvalidSelection,
                    "lineage child is not a direct same-contract same-authority successor of the head",
                    Usage::default(),
                ));
            }
        }
        let mut children = Vec::new();
        children
            .try_reserve_exact(direct_children.len())
            .map_err(|_| resource_error(0, 0, 0, direct_children.len()))?;
        children.extend(direct_children.iter().map(bundle_stamp));
        children.sort_by(|left, right| left.identity.cmp(&right.identity));
        if children
            .windows(2)
            .any(|pair| pair[0].identity == pair[1].identity)
        {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "lineage children must be distinct",
                Usage::default(),
            ));
        }
        let document = build_lineage_document(head, &children, limits)?;
        Ok(Self {
            document,
            head: head.clone(),
            direct_children: children,
        })
    }

    /// Returns the canonical lineage document that was strict-derived or read.
    #[must_use]
    pub const fn document(&self) -> &LineageDocument {
        &self.document
    }

    /// Returns the strict-read current head.
    #[must_use]
    pub const fn head(&self) -> &View {
        &self.head
    }

    /// Returns every known direct child in canonical identity order.
    #[must_use]
    pub fn direct_children(&self) -> impl ExactSizeIterator<Item = ChildRef<'_>> + '_ {
        self.direct_children.iter().map(ChildRef)
    }
}

/// Borrowed bounded direct-child stamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChildRef<'a>(&'a BundleStamp);

impl<'a> ChildRef<'a> {
    /// Returns the child's exact bundle identity.
    #[must_use]
    pub fn identity(self) -> &'a str {
        &self.0.identity
    }

    /// Returns the child's bundle revision.
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.0.revision
    }

    /// Returns the SHA-256 digest of the child's canonical bytes.
    #[must_use]
    pub fn digest(self) -> &'a str {
        &self.0.digest
    }
}

/// Strict-reads canonical lineage bytes against independently supplied views.
pub fn read_lineage(
    bytes: &[u8],
    head: &View,
    direct_children: &[View],
    limits: Limits,
) -> Result<LineageView> {
    let effective = limits.effective();
    let observed = preflight_for_contract(
        bytes,
        Limits {
            max_population_entries: effective
                .max_population_entries
                .min(effective.max_lineage_children),
            ..effective
        },
        LINEAGE_CONTRACT,
    )?;
    let contract_probe: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        Error::new(
            ErrorCode::InvalidJson,
            "lineage document is not valid JSON",
            observed,
        )
    })?;
    if contract_probe
        .get("contract")
        .and_then(serde_json::Value::as_str)
        != Some(LINEAGE_CONTRACT)
    {
        return Err(Error::new(
            ErrorCode::ContractMismatch,
            "contract label does not select the lineage reader",
            observed,
        ));
    }
    drop(contract_probe);
    let envelope: LineageEnvelope = serde_json::from_slice(bytes).map_err(|_| {
        Error::new(
            ErrorCode::InvalidJson,
            "lineage document is not a closed JSON value",
            observed,
        )
    })?;
    debug_assert_eq!(envelope.contract, LINEAGE_CONTRACT);
    let canonical = to_bounded_json(&envelope, effective.max_output_bytes)?;
    if canonical != bytes {
        return Err(Error::new(
            ErrorCode::NonCanonical,
            "lineage bytes are not the canonical closed encoding",
            observed,
        ));
    }
    let without_identity = LineageEnvelopeWithoutIdentity {
        contract: LINEAGE_CONTRACT,
        authority: &envelope.authority,
        subject: &envelope.subject,
        head: &envelope.head,
        direct_children: &envelope.direct_children,
        max_direct_children: envelope.max_direct_children,
    };
    if lineage_identity(&without_identity, limits)?.as_str() != envelope.identity {
        return Err(Error::new(
            ErrorCode::IdentityMismatch,
            "lineage identity does not match its canonical preimage",
            observed,
        ));
    }
    let expected = LineageView::from_views(head, direct_children, limits)?;
    if expected.document.bytes() != bytes {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "lineage bytes differ from the independently supplied views",
            observed,
        ));
    }
    Ok(expected)
}

/// Whether publication creates a successor or recognizes an exact replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    /// No supplied direct child already represented these bytes.
    Published,
    /// The one supplied direct child was byte-identical.
    Replayed,
}

/// Complete successful publication result; no partial result exists on refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Publication {
    document: Document,
    view: View,
    lineage: LineageView,
    disposition: Disposition,
}

impl Publication {
    /// Returns the canonical immutable document.
    #[must_use]
    pub const fn document(&self) -> &Document {
        &self.document
    }

    /// Returns the constructor-private strict-read view.
    #[must_use]
    pub const fn view(&self) -> &View {
        &self.view
    }

    /// Returns the canonical successor lineage with this bundle as head.
    #[must_use]
    pub const fn lineage(&self) -> &LineageView {
        &self.lineage
    }

    /// Returns whether this result is a new publication or exact replay.
    #[must_use]
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }
}

/// Validates lineage and derives one canonical complete bundle.
pub fn publish(
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<Publication> {
    publish_for(
        CONTRACT,
        Profile::CompleteV1,
        context,
        selection,
        lineage,
        limits,
    )
}

/// Validates authority and derives one canonical structurally empty v2 bundle.
pub fn publish_v2(
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<Publication> {
    publish_for(
        V2_CONTRACT,
        Profile::EmptyV2,
        context,
        selection,
        lineage,
        limits,
    )
}

fn publish_for(
    contract: &'static str,
    profile: Profile,
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<Publication> {
    let payload = payload(contract, profile, context, selection, lineage, limits)?;
    let usage = Usage {
        depth: 4,
        string_bytes: payload
            .components
            .iter()
            .flat_map(|component| [&component.contract, &component.identity, &component.digest])
            .map(String::len)
            .max()
            .unwrap_or(0),
        population_entries: 0,
        bundle_components: payload.components.len(),
        replacements: payload.replacements.len(),
        conflicts: payload.conflicts.len(),
        visited_fields: 18usize
            .saturating_add(payload.components.len().saturating_mul(5))
            .saturating_add(payload.replacements.len().saturating_mul(3))
            .saturating_add(payload.conflicts.len().saturating_mul(3)),
        ..Usage::default()
    };
    let document = build_document(contract, context, &payload, usage, limits)?;
    let view = read_exact(contract, document.bytes(), &document, limits)?;
    let disposition = disposition(&document, lineage)?;
    let successor_lineage = match (disposition, lineage) {
        (Disposition::Replayed, Some(supplied)) if supplied.head.bytes() == document.bytes() => {
            supplied.clone()
        }
        _ => LineageView::from_views(&view, &[], limits)?,
    };
    Ok(Publication {
        document,
        view,
        lineage: successor_lineage,
        disposition,
    })
}

/// Strict-reads canonical bundle bytes against every independent selection.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<View> {
    read_for(
        CONTRACT,
        Profile::CompleteV1,
        bytes,
        context,
        selection,
        lineage,
        limits,
    )
}

/// Strict-reads canonical structurally empty v2 bundle bytes.
pub fn read_v2(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<View> {
    read_for(
        V2_CONTRACT,
        Profile::EmptyV2,
        bytes,
        context,
        selection,
        lineage,
        limits,
    )
}

// The shared reader keeps contract/profile dispatch explicit while preserving
// the same independently supplied strict-read selections as the public APIs.
// The strict-reader boundary keeps every independently validated owner axis explicit.
#[allow(clippy::too_many_arguments)]
fn read_for(
    contract: &'static str,
    profile: Profile,
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<View> {
    let observed = preflight_for_contract(bytes, limits.effective(), contract)?;
    read_for_preflighted(
        contract, profile, bytes, context, selection, lineage, limits, observed,
    )
}

// Preflighted reads mirror the public strict-reader axes without a lossy options bag.
#[allow(clippy::too_many_arguments)]
fn read_for_preflighted(
    contract: &'static str,
    profile: Profile,
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
    observed: Usage,
) -> Result<View> {
    let expected = publish_for(contract, profile, context, selection, lineage, limits)?;
    read_exact_preflighted(contract, bytes, expected.document(), limits, observed)
}

/// Strict-reads one initial or historical revision without consulting current publication state.
///
/// A later revision requires its exact strict-read predecessor view. Branch and
/// current-head decisions remain exclusively in [`publish`].
pub fn read_revision(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    predecessor: Option<&View>,
    limits: Limits,
) -> Result<View> {
    read_revision_for(
        CONTRACT,
        Profile::CompleteV1,
        bytes,
        context,
        selection,
        predecessor,
        limits,
    )
}

/// Strict-reads one structurally empty v2 initial or historical revision.
pub fn read_revision_v2(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    predecessor: Option<&View>,
    limits: Limits,
) -> Result<View> {
    read_revision_for(
        V2_CONTRACT,
        Profile::EmptyV2,
        bytes,
        context,
        selection,
        predecessor,
        limits,
    )
}

// Historical reads add an independently validated predecessor to the same
// explicit contract/profile boundary used by current-head reads.
// Historical reads add the predecessor axis to the same explicit strict-reader boundary.
#[allow(clippy::too_many_arguments)]
fn read_revision_for(
    contract: &'static str,
    profile: Profile,
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    predecessor: Option<&View>,
    limits: Limits,
) -> Result<View> {
    let observed = preflight_for_contract(bytes, limits.effective(), contract)?;
    let lineage = match (context.predecessor(), predecessor) {
        (None, None) => None,
        (Some(document), Some(view)) if document.identity() == view.identity() => {
            Some(LineageView::from_views(view, &[], limits)?)
        }
        _ => {
            return Err(Error::new(
                ErrorCode::PredecessorMismatch,
                "historical read requires the exact strict-read predecessor view",
                Usage::default(),
            ));
        }
    };
    read_for_preflighted(
        contract,
        profile,
        bytes,
        context,
        selection,
        lineage.as_ref(),
        limits,
        observed,
    )
}

fn payload(
    contract: &'static str,
    profile: Profile,
    context: Context<'_>,
    selection: &Selection,
    lineage: Option<&LineageView>,
    limits: Limits,
) -> Result<BundleView> {
    let effective = limits.effective();
    if let Some(lineage) = lineage {
        if lineage.direct_children.len() > effective.max_lineage_children {
            return Err(resource_error(0, 0, 0, lineage.direct_children.len()));
        }
        if lineage.document.bytes().len() > effective.max_input_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "supplied lineage bytes exceed the effective input bound",
                Usage {
                    wire_bytes: lineage.document.bytes().len(),
                    lineage_children: lineage.direct_children.len(),
                    ..Usage::default()
                },
            ));
        }
    }
    if selection.components.len() > effective.max_bundle_components
        || selection.replacements.len() > effective.max_replacements
        || selection.conflicts.len() > effective.max_conflicts
    {
        return Err(resource_error(
            selection.components.len(),
            selection.replacements.len(),
            selection.conflicts.len(),
            0,
        ));
    }
    let embedded_bytes = selection
        .components
        .iter()
        .try_fold(0usize, |total, component| {
            total.checked_add(component.document_bytes).ok_or_else(|| {
                Error::new(
                    ErrorCode::ResourceIncomplete,
                    "embedded component byte count overflowed",
                    Usage {
                        wire_bytes: usize::MAX,
                        ..Usage::default()
                    },
                )
            })
        })?;
    if embedded_bytes > effective.max_output_bytes {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "embedded component bytes exceed the effective output limit",
            Usage {
                wire_bytes: embedded_bytes,
                bundle_components: selection.components.len(),
                ..Usage::default()
            },
        ));
    }
    let mut components: Vec<_> = selection.components.iter().collect();
    components.sort_by(|left, right| {
        left.role()
            .cmp(&right.role())
            .then_with(|| left.identity().cmp(right.identity()))
    });
    let complete_roles = match profile {
        Profile::CompleteV1 => ComponentRole::ALL
            .iter()
            .all(|role| components.iter().any(|component| component.role() == *role)),
        Profile::EmptyV2 => {
            components.len() == ComponentRole::EMPTY_V2.len()
                && ComponentRole::EMPTY_V2
                    .iter()
                    .all(|role| components.iter().any(|component| component.role() == *role))
                && empty_profile_is_coherent(&components)
        }
    };
    let singleton_roles = [
        ComponentRole::Population,
        ComponentRole::Position,
        ComponentRole::Clock,
        ComponentRole::Progress,
        ComponentRole::Closure,
        ComponentRole::Completeness,
    ];
    let singleton_roles_are_exact = singleton_roles.iter().all(|role| {
        components
            .iter()
            .filter(|component| component.role() == *role)
            .count()
            == 1
    });
    let duplicate_identity = components
        .windows(2)
        .any(|pair| pair[0].role() == pair[1].role() && pair[0].identity() == pair[1].identity());
    let mut fact_subjects = components
        .iter()
        .map(|component| {
            Ok((
                component.role(),
                component.fact_subject.scope_identity.as_str(),
                component.fact_subject.population_identity.as_str(),
                to_bounded_json(
                    &component.fact_subject.semantic_key,
                    effective.max_output_bytes,
                )?,
                component.observation_identity.as_deref(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    fact_subjects.sort_unstable();
    let duplicate_fact_subject = fact_subjects.windows(2).any(|pair| pair[0] == pair[1]);
    if !complete_roles || !singleton_roles_are_exact || duplicate_identity || duplicate_fact_subject
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "bundle must contain every role in its declared singleton or distinct-subject canonical set",
            Usage::default(),
        ));
    }
    if components
        .iter()
        .any(|component| component.wire.revision > context.revision())
    {
        return Err(Error::new(
            ErrorCode::RevisionMismatch,
            "component revision cannot exceed its containing bundle revision",
            Usage::default(),
        ));
    }
    if components.iter().any(|component| {
        component.authority != *context.authority() || component.subject != *context.subject()
    }) {
        return Err(Error::new(
            ErrorCode::AuthorityMismatch,
            "bundle component authority or subject is cross-wired",
            Usage::default(),
        ));
    }
    let mut replacements = selection.replacements.clone();
    replacements.sort_by(|left, right| left.wire.cmp(&right.wire));
    if replacements.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::new(
            ErrorCode::InvalidReplacement,
            "bundle replacements must be distinct",
            Usage::default(),
        ));
    }
    let component_identities = components
        .iter()
        .map(|component| component.identity())
        .collect::<std::collections::BTreeSet<_>>();
    let mut conflicts = selection.conflicts.clone();
    if conflicts.iter().any(|conflict| {
        conflict.authority != *context.authority() || conflict.subject != *context.subject()
    }) {
        return Err(Error::new(
            ErrorCode::AuthorityMismatch,
            "bundle conflict authority or subject is cross-wired",
            Usage::default(),
        ));
    }
    if conflicts
        .iter()
        .any(|conflict| !component_identities.contains(conflict.wire.component_identity.as_str()))
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "bundle conflict does not name a selected component",
            Usage::default(),
        ));
    }
    conflicts.sort_by(|left, right| left.wire.cmp(&right.wire));
    if conflicts
        .windows(2)
        .any(|pair| pair[0].wire == pair[1].wire)
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "bundle conflicts must be distinct",
            Usage::default(),
        ));
    }
    validate_lineage(contract, context, &components, &replacements, lineage)?;
    let bounds = BundleBoundsWire {
        max_components: wire_count(effective.max_bundle_components)?,
        max_replacements: wire_count(effective.max_replacements)?,
        max_conflicts: wire_count(effective.max_conflicts)?,
    };
    let usage = BundleUsageWire {
        components: wire_count(components.len())?,
        replacements: wire_count(replacements.len())?,
        conflicts: wire_count(conflicts.len())?,
    };
    let records = embedded_components(
        &components,
        &[
            ComponentRole::Observation,
            ComponentRole::Partial,
            ComponentRole::Capture,
            ComponentRole::Activation,
            ComponentRole::Availability,
        ],
    );
    let populations = embedded_components(
        &components,
        &[ComponentRole::Population, ComponentRole::Completeness],
    );
    let positions = embedded_components(
        &components,
        &[ComponentRole::Position, ComponentRole::Clock],
    );
    let progress = embedded_components(
        &components,
        &[ComponentRole::Progress, ComponentRole::Closure],
    );
    Ok(BundleView {
        components: components
            .into_iter()
            .map(|component| component.wire.clone())
            .collect(),
        replacements: replacements
            .into_iter()
            .map(|replacement| replacement.wire)
            .collect(),
        records,
        populations,
        positions,
        progress,
        conflicts: conflicts
            .into_iter()
            .map(|conflict| conflict.wire)
            .collect(),
        bounds,
        usage,
    })
}

fn empty_profile_is_coherent(components: &[&Component]) -> bool {
    let population_is_empty = components.iter().any(|component| {
        matches!(
            &component.payload,
            EmbeddedPayloadWire::Population(value) if value.required_members().is_empty()
        )
    });
    let positions_are_empty = components.iter().any(|component| {
        matches!(
            &component.payload,
            EmbeddedPayloadWire::Position(value) if value.positions().len() == 0
        )
    });
    let completeness_is_empty = components.iter().any(|component| {
        matches!(
            &component.payload,
            EmbeddedPayloadWire::Completeness(value)
                if value.fact_count() == 0
        )
    });
    let availability_is_empty = components.iter().any(|component| {
        matches!(
            &component.payload,
            EmbeddedPayloadWire::Availability(value)
                if value.required_results().is_empty()
                    && value.available_results().is_empty()
                    && value.state() == super::availability::State::Available
        )
    });
    population_is_empty && positions_are_empty && completeness_is_empty && availability_is_empty
}

fn validate_lineage(
    contract: &'static str,
    context: Context<'_>,
    components: &[&Component],
    replacements: &[Replacement],
    lineage: Option<&LineageView>,
) -> Result<()> {
    if context
        .predecessor()
        .is_some_and(|predecessor| predecessor.contract() != contract)
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "bundle predecessor uses a different contract version",
            Usage::default(),
        ));
    }
    if let Some(lineage) = lineage {
        if lineage.head.contract() != contract
            || lineage.head.authority() != context.authority()
            || lineage.head.subject() != context.subject()
        {
            return Err(Error::new(
                ErrorCode::AuthorityMismatch,
                "lineage head authority or subject is cross-wired",
                Usage::default(),
            ));
        }
        let candidate_is_head_key = lineage.head.revision() == context.revision()
            && lineage.head.authority() == context.authority()
            && lineage.head.subject() == context.subject();
        if candidate_is_head_key {
            let predecessor_matches = match context.predecessor() {
                Some(predecessor) => lineage.head.predecessor() == Some(predecessor.identity()),
                None => context.revision() == 1 && lineage.head.predecessor().is_none(),
            };
            return if predecessor_matches {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorCode::PredecessorMismatch,
                    "replayed bundle predecessor differs from the supplied head",
                    Usage::default(),
                ))
            };
        }
    }
    match (context.predecessor(), lineage) {
        (None, None) if context.revision() == 1 && replacements.is_empty() => Ok(()),
        (None, _) if context.revision() == 1 => Err(Error::new(
            ErrorCode::InvalidReplacement,
            "initial bundle must omit lineage and replacements",
            Usage::default(),
        )),
        (None, None) => Err(Error::new(
            ErrorCode::RevisionMismatch,
            "a bundle without a predecessor must use revision one",
            Usage::default(),
        )),
        (Some(predecessor), Some(lineage)) => {
            if predecessor.identity() != lineage.head.identity() {
                return Err(Error::new(
                    ErrorCode::StaleHead,
                    "bundle predecessor is not the supplied current head",
                    Usage::default(),
                ));
            }
            if replacements.is_empty() {
                return Err(Error::new(
                    ErrorCode::InvalidReplacement,
                    "later bundle requires a nonempty replacement set",
                    Usage::default(),
                ));
            }
            validate_replacements(lineage.head.payload(), components, replacements)
        }
        (Some(_), None) => Err(Error::new(
            ErrorCode::StaleHead,
            "later bundle requires a strict-read current lineage view",
            Usage::default(),
        )),
        (None, Some(_)) => Err(Error::new(
            ErrorCode::StaleHead,
            "initial bundle cannot select an existing lineage head",
            Usage::default(),
        )),
    }
}

fn validate_replacements(
    predecessor: &BundleView,
    components: &[&Component],
    replacements: &[Replacement],
) -> Result<()> {
    let mut removed = Vec::new();
    let mut added = Vec::new();
    let mut declared_removed = Vec::new();
    let mut declared_added = Vec::new();
    removed
        .try_reserve_exact(predecessor.components.len())
        .map_err(|_| resource_error(components.len(), replacements.len(), 0, 0))?;
    added
        .try_reserve_exact(components.len())
        .map_err(|_| resource_error(components.len(), replacements.len(), 0, 0))?;
    declared_removed
        .try_reserve_exact(replacements.len())
        .map_err(|_| resource_error(components.len(), replacements.len(), 0, 0))?;
    declared_added
        .try_reserve_exact(replacements.len())
        .map_err(|_| resource_error(components.len(), replacements.len(), 0, 0))?;
    let mut prior_index = 0;
    let mut successor_index = 0;
    while prior_index < predecessor.components.len() && successor_index < components.len() {
        let prior = &predecessor.components[prior_index];
        let successor = components[successor_index];
        match prior
            .role
            .cmp(&successor.role())
            .then_with(|| prior.identity.as_str().cmp(successor.identity()))
        {
            std::cmp::Ordering::Less => {
                removed.push((prior.role, prior.identity.clone()));
                prior_index += 1;
            }
            std::cmp::Ordering::Greater => {
                added.push((successor.role(), successor.identity().to_owned()));
                successor_index += 1;
            }
            std::cmp::Ordering::Equal => {
                prior_index += 1;
                successor_index += 1;
            }
        }
    }
    removed.extend(
        predecessor.components[prior_index..]
            .iter()
            .map(|prior| (prior.role, prior.identity.clone())),
    );
    added.extend(
        components[successor_index..]
            .iter()
            .map(|successor| (successor.role(), successor.identity().to_owned())),
    );
    declared_removed.extend(replacements.iter().map(|replacement| {
        (
            replacement.wire.role,
            replacement.wire.prior_identity.clone(),
        )
    }));
    declared_added.extend(replacements.iter().map(|replacement| {
        (
            replacement.wire.role,
            replacement.wire.successor_identity.clone(),
        )
    }));
    removed.sort_unstable();
    added.sort_unstable();
    declared_removed.sort_unstable();
    declared_added.sort_unstable();
    let one_to_one = declared_removed.windows(2).all(|pair| pair[0] != pair[1])
        && declared_added.windows(2).all(|pair| pair[0] != pair[1]);
    if removed.is_empty() || !one_to_one || removed != declared_removed || added != declared_added {
        return Err(Error::new(
            ErrorCode::InvalidReplacement,
            "declared replacements must exactly explain every changed component",
            Usage::default(),
        ));
    }
    Ok(())
}

fn component_subject(
    role: ComponentRole,
    document: &serde_json::Value,
) -> Result<(serde_json::Value, Option<String>)> {
    let semantic_path = match role {
        ComponentRole::Observation => "/payload/subject",
        ComponentRole::Population => "/payload/population_identity",
        ComponentRole::Position => "/payload/ledger_identity",
        ComponentRole::Clock => "/payload/clock_identity",
        ComponentRole::Partial => "/payload/observation_identity",
        ComponentRole::Capture => "/payload/trigger_identity",
        ComponentRole::Progress | ComponentRole::Closure => "/payload/scope_identity",
        ComponentRole::Activation => "/payload/obligation_identity",
        ComponentRole::Completeness => "/payload/population_identity",
        ComponentRole::Availability => "/payload/required_results",
    };
    let semantic_key = document.pointer(semantic_path).cloned().ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidSelection,
            "component document omits its role-specific fact subject",
            Usage::default(),
        )
    })?;
    let observation_path = match role {
        ComponentRole::Observation => Some(("/payload/observation_id", true)),
        ComponentRole::Partial => Some(("/payload/observation_identity", true)),
        ComponentRole::Activation => Some(("/payload/trigger_observation_identity", false)),
        _ => None,
    };
    let observation_identity = observation_path
        .and_then(|(path, _)| document.pointer(path))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    if observation_path.is_some_and(|(_, required)| required) && observation_identity.is_none() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "record-like component omits its observation identity",
            Usage::default(),
        ));
    }
    Ok((semantic_key, observation_identity))
}

fn typed_payload(role: ComponentRole, document: &serde_json::Value) -> Result<EmbeddedPayloadWire> {
    let payload = document.get("payload").cloned().ok_or_else(|| {
        Error::new(
            ErrorCode::InvalidSelection,
            "component document omits its typed payload",
            Usage::default(),
        )
    })?;
    macro_rules! decode {
        ($variant:ident, $type:path) => {
            serde_json::from_value::<$type>(payload)
                .map(Box::new)
                .map(EmbeddedPayloadWire::$variant)
                .map_err(|_| {
                    Error::new(
                        ErrorCode::InvalidSelection,
                        "component payload does not match its semantic role",
                        Usage::default(),
                    )
                })
        };
    }
    match role {
        ComponentRole::Observation => decode!(Observation, super::observation::RecordView),
        ComponentRole::Population => decode!(Population, super::population::PopulationView),
        ComponentRole::Position => decode!(Position, super::position::PositionLedgerView),
        ComponentRole::Clock => decode!(Clock, super::clock::ClockView),
        ComponentRole::Partial => decode!(Partial, super::partial::PartialView),
        ComponentRole::Capture => decode!(Capture, super::capture::CaptureView),
        ComponentRole::Progress => decode!(Progress, super::progress::ProgressView),
        ComponentRole::Activation => decode!(Activation, super::activation::ActivationView),
        ComponentRole::Closure => decode!(Closure, super::closure::ClosureView),
        ComponentRole::Completeness => decode!(Completeness, super::completeness::CompletenessView),
        ComponentRole::Availability => decode!(Availability, super::availability::AvailabilityView),
    }
}

fn embedded_components(
    components: &[&Component],
    roles: &[ComponentRole],
) -> Vec<EmbeddedFactWire> {
    components
        .iter()
        .filter(|component| roles.contains(&component.role()))
        .map(|component| EmbeddedFactWire {
            role: component.role(),
            fact_subject: component.fact_subject.clone(),
            observation_identity: component.observation_identity.clone(),
            payload: component.payload.clone(),
            canonical_document: component.canonical_document.clone(),
        })
        .collect()
}

fn disposition(document: &Document, lineage: Option<&LineageView>) -> Result<Disposition> {
    let Some(lineage) = lineage else {
        return Ok(Disposition::Published);
    };
    if lineage.head.revision() == document.revision() {
        if lineage.head.bytes() == document.bytes() {
            return Ok(Disposition::Replayed);
        }
        return Err(Error::new(
            ErrorCode::IdentityContradiction,
            "authority, scope, and revision key names unequal bundle bytes",
            Usage::default(),
        ));
    }
    if lineage.direct_children.is_empty() {
        return Ok(Disposition::Published);
    }
    let digest: [u8; 32] = Sha256::digest(document.bytes()).into();
    let document_digest = digest_hex(&Digest::new(digest));
    let mut exact_replay = false;
    for child in &lineage.direct_children {
        if child.revision == document.revision() {
            if child.identity == document.identity().as_str() && child.digest == document_digest {
                exact_replay = true;
                continue;
            }
            return Err(Error::new(
                ErrorCode::IdentityContradiction,
                "authority, scope, and revision key names unequal bundle bytes",
                Usage::default(),
            ));
        }
    }
    if exact_replay && lineage.direct_children.len() == 1 {
        return Ok(Disposition::Replayed);
    }
    Err(Error::new(
        ErrorCode::KnownSibling,
        "supplied lineage already records a competing direct child",
        Usage::default(),
    ))
}

fn build_lineage_document(
    head: &View,
    direct_children: &[BundleStamp],
    limits: Limits,
) -> Result<LineageDocument> {
    let effective = limits.effective();
    let authority = LineageAuthorityWire {
        definition_identity: head.authority().definition_identity.as_str().to_owned(),
        definition_revision: head.authority().definition_revision.as_str().to_owned(),
        definition_digest: digest_hex(&head.authority().definition_digest),
    };
    let subject = LineageSubjectWire {
        scope_identity: head.subject().scope_identity.as_str().to_owned(),
        population_identity: head.subject().population_identity.as_str().to_owned(),
    };
    let head_stamp = bundle_stamp(head);
    let children = direct_children.to_vec();
    let max_direct_children = u64::try_from(effective.max_lineage_children).map_err(|_| {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "lineage child limit cannot be represented on the wire",
            Usage::default(),
        )
    })?;
    let without_identity = LineageEnvelopeWithoutIdentity {
        contract: LINEAGE_CONTRACT,
        authority: &authority,
        subject: &subject,
        head: &head_stamp,
        direct_children: &children,
        max_direct_children,
    };
    let identity = lineage_identity(&without_identity, limits)?;
    let envelope = LineageEnvelope {
        contract: LINEAGE_CONTRACT.to_owned(),
        identity: identity.as_str().to_owned(),
        authority,
        subject,
        head: head_stamp,
        direct_children: children,
        max_direct_children,
    };
    let bytes = to_bounded_json(&envelope, effective.max_output_bytes)?;
    let mut usage = preflight_for_contract(
        &bytes,
        Limits {
            max_input_bytes: effective.max_output_bytes,
            ..effective
        },
        LINEAGE_CONTRACT,
    )?;
    usage.lineage_children = direct_children.len();
    Ok(LineageDocument {
        identity,
        bytes,
        usage,
    })
}

fn bundle_stamp(view: &View) -> BundleStamp {
    let digest: [u8; 32] = Sha256::digest(view.bytes()).into();
    BundleStamp {
        revision: view.revision(),
        identity: view.identity().as_str().to_owned(),
        digest: digest_hex(&Digest::new(digest)),
    }
}

fn lineage_identity(
    value: &LineageEnvelopeWithoutIdentity<'_>,
    limits: Limits,
) -> Result<Identity> {
    let bytes = to_bounded_json(value, limits.effective().max_output_bytes)?;
    let mut hasher = Sha256::new();
    hasher.update(LINEAGE_CONTRACT.as_bytes());
    hasher.update([0]);
    hasher.update(bytes);
    Ok(Identity::new(format!("sha256:{:x}", hasher.finalize())))
}

fn resource_error(
    components: usize,
    replacements: usize,
    conflicts: usize,
    children: usize,
) -> Error {
    Error::new(
        ErrorCode::ResourceIncomplete,
        "bundle collection exceeds its effective population limit",
        Usage {
            bundle_components: components,
            replacements,
            conflicts,
            lineage_children: children,
            ..Usage::default()
        },
    )
}

fn wire_count(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "bundle count cannot be represented on the wire",
            Usage::default(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::super::common;
    use super::*;

    #[test]
    fn schema_digest_is_pinned() {
        assert_eq!(common::schema_sha256(SCHEMA_BYTES), SCHEMA_SHA256);
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
        assert_eq!(common::schema_sha256(V2_SCHEMA_BYTES), V2_SCHEMA_SHA256);
        common::assert_closed_schema(V2_SCHEMA_BYTES, V2_CONTRACT);
        assert_eq!(
            common::schema_sha256(LINEAGE_SCHEMA_BYTES),
            LINEAGE_SCHEMA_SHA256
        );
    }
}
