// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! `quire.observation.progress-assertion/v1` owner artifact.

use quire_exact::{NodeKey, Origin};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(test)]
use super::common;
use super::common::{
    boundary_wire, build_document, ensure_sorted_unique, read_exact, BoundaryWire, Validated,
};
use super::{
    BoundaryRef, Context, Document, Error, ErrorCode, OpenClosed, Result, TemporalBoundary, Usage,
};
use crate::Identity;

use super::observation::CutoffSelection;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.progress-assertion/v1";
/// JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-progress-assertion-v1.schema.json");

/// Exact clock, sources, boundary, trigger, cutoff, and restoration selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    clock: super::clock::Selection,
    required_sources: Vec<Identity>,
    boundary: TemporalBoundary,
    state: OpenClosed,
    captured_trigger_identity: Identity,
    cutoff: CutoffSelection,
    restoration_basis_identity: Identity,
    association: Option<Vec<AssociationRow>>,
}

/// A structural node identity encoded as the QSL node-wire codec's canonical
/// `{"ordinal":n,"package":"...","path":[...]}` text. Shape validation does
/// not establish membership in a QSL checked package.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NodeIdentity {
    key: NodeKey,
    text: String,
}

impl NodeIdentity {
    /// Encodes a real structural node key under owner byte limits.
    pub fn from_key(key: NodeKey, limits: Limits) -> Result<Self> {
        let text = qsl_foundation::node_wire::node_key_to_text(&key).map_err(|_| {
            Error::new(
                ErrorCode::InvalidSelection,
                "node identity cannot be encoded",
                Usage::default(),
            )
        })?;
        Self::check_size(&text, limits)?;
        Ok(Self { key, text })
    }

    /// Reads canonical structural node text, refusing malformed and noncanonical forms.
    pub fn from_text(text: &str, limits: Limits) -> Result<Self> {
        Self::check_size(text, limits)?;
        let key = qsl_foundation::node_wire::node_key_from_text(text).map_err(|_| {
            Error::new(
                ErrorCode::InvalidSelection,
                "node identity has invalid structural components",
                Usage::default(),
            )
        })?;
        let canonical = qsl_foundation::node_wire::node_key_to_text(&key).map_err(|_| {
            Error::new(
                ErrorCode::InvalidSelection,
                "node identity cannot be encoded",
                Usage::default(),
            )
        })?;
        if canonical != text {
            return Err(Error::new(
                ErrorCode::NonCanonical,
                "node identity text is not canonical",
                Usage::default(),
            ));
        }
        Ok(Self {
            key,
            text: canonical,
        })
    }

    fn check_size(text: &str, limits: Limits) -> Result<()> {
        let effective = limits.effective();
        if text.len() > effective.max_string_bytes || text.len() > effective.max_output_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "node identity exceeds owner byte limit",
                Usage::default(),
            ));
        }
        let escaped = serde_json::to_string(text).map_err(|_| {
            Error::new(
                ErrorCode::InvalidSelection,
                "node identity cannot be JSON encoded",
                Usage::default(),
            )
        })?;
        if escaped.len().saturating_sub(2) > effective.max_string_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "node identity exceeds encoded string limit",
                Usage::default(),
            ));
        }
        Ok(())
    }

    /// Returns the exact structural node key.
    #[must_use]
    pub const fn key(&self) -> &NodeKey {
        &self.key
    }

    /// Returns the canonical structural node text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

impl Serialize for NodeIdentity {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for NodeIdentity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::from_text(&text, Limits::owner_max()).map_err(serde::de::Error::custom)
    }
}

/// Selected temporal claim's clause node and exact occurrence key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ClaimOccurrence {
    clause_node: NodeIdentity,
    occurrence: Origin,
}

impl ClaimOccurrence {
    /// Retains the clause node and selected occurrence without interpreting the role.
    #[must_use]
    pub fn new(clause_node: NodeIdentity, occurrence: &Origin) -> Self {
        Self {
            clause_node,
            occurrence: occurrence.clone(),
        }
    }

    /// Returns the selected structural clause node identity.
    #[must_use]
    pub const fn clause_node(&self) -> &NodeIdentity {
        &self.clause_node
    }

    /// Returns the exact occurrence role.
    #[must_use]
    pub fn role(&self) -> &str {
        self.occurrence.role().as_str()
    }

    /// Returns the exact selected occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> &Origin {
        &self.occurrence
    }

    /// Returns the occurrence ordinal.
    #[must_use]
    pub fn ordinal(&self) -> u64 {
        self.occurrence.ordinal()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClaimOccurrenceWire {
    clause_node: NodeIdentity,
    role: String,
    ordinal: String,
}

impl Serialize for ClaimOccurrence {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        ClaimOccurrenceWire {
            clause_node: self.clause_node.clone(),
            role: self.role().to_owned(),
            ordinal: self.ordinal().to_string(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ClaimOccurrence {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let wire = ClaimOccurrenceWire::deserialize(deserializer)?;
        if wire.role.trim().is_empty() || !valid_natural_decimal(&wire.ordinal) {
            return Err(serde::de::Error::custom("invalid claim occurrence"));
        }
        let ordinal = wire.ordinal.parse().map_err(serde::de::Error::custom)?;
        Ok(Self::new(
            wire.clause_node,
            &Origin::new(quire_exact::Role::new(wire.role), ordinal),
        ))
    }
}

impl Selection {
    /// Constructs a complete progress-authority selection.
    #[must_use]
    pub const fn new(
        clock: super::clock::Selection,
        required_sources: Vec<Identity>,
        boundary: TemporalBoundary,
        state: OpenClosed,
        captured_trigger_identity: Identity,
        cutoff: CutoffSelection,
        restoration_basis_identity: Identity,
    ) -> Self {
        Self {
            clock,
            required_sources,
            boundary,
            state,
            captured_trigger_identity,
            cutoff,
            restoration_basis_identity,
            association: None,
        }
    }

    /// Selects the exact owner asserted QSL to observation association rows.
    #[must_use]
    pub fn with_association(mut self, rows: Vec<AssociationRow>) -> Self {
        self.association = Some(rows);
        self
    }
}

/// Selected QSL time-source shape asserted by this progress owner.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TimeSource {
    /// Dense model time.
    Dense,
    /// Tick model time with an exact positive rational period.
    Tick {
        /// Resolved tick operation identity.
        operation: String,
        /// Resolved tick operation node identity.
        operation_node: NodeIdentity,
        /// Positive decimal numerator in the selected unit.
        period_numerator: String,
        /// Positive decimal denominator in the selected unit.
        period_denominator: String,
    },
}

/// One explicit owner asserted mapping from a checked QSL selection to admitted progress.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssociationRow {
    /// Selected QSL model identity.
    pub model: String,
    /// Selected structural QSL model-node identity.
    pub model_node: NodeIdentity,
    /// Selected clause node and occurrence for the QSL claim.
    pub claim: ClaimOccurrence,
    /// Selected QSL time-source shape.
    pub source: TimeSource,
    /// Selected QSL unit name.
    pub unit: String,
    /// Selected structural QSL unit-node identity.
    pub unit_node: NodeIdentity,
    /// Selected structural QSL time-dimension node identity.
    pub time_dimension: NodeIdentity,
    /// Matching admitted observation source identity.
    pub observation_source: String,
    /// Matching admitted observation clock identity.
    pub observation_clock: String,
    /// Matching admitted observation clock revision.
    pub observation_revision: String,
}

/// Borrowed canonical association value issued only by a validated progress view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AssociationRef<'a> {
    progress_document_identity: &'a str,
    progress_authority_identity: &'a str,
    scope_identity: &'a str,
    population_identity: &'a str,
    clock_identity: &'a str,
    clock_revision: &'a str,
    rows: &'a [AssociationRow],
}

impl<'a> AssociationRef<'a> {
    /// Returns the exact strict-read progress document identity.
    #[must_use]
    pub const fn progress_document_identity(&self) -> &'a str {
        self.progress_document_identity
    }

    /// Returns the exact progress authority identity.
    #[must_use]
    pub const fn progress_authority_identity(&self) -> &'a str {
        self.progress_authority_identity
    }

    /// Returns the selected snapshot or window identity.
    #[must_use]
    pub const fn scope_identity(&self) -> &'a str {
        self.scope_identity
    }

    /// Returns the selected population identity.
    #[must_use]
    pub const fn population_identity(&self) -> &'a str {
        self.population_identity
    }

    /// Returns the admitted clock identity.
    #[must_use]
    pub const fn clock_identity(&self) -> &'a str {
        self.clock_identity
    }

    /// Returns the admitted clock revision.
    #[must_use]
    pub const fn clock_revision(&self) -> &'a str {
        self.clock_revision
    }

    /// Returns the exact asserted mapping rows.
    #[must_use]
    pub const fn rows(&self) -> &'a [AssociationRow] {
        self.rows
    }
}

/// Read-only payload produced only inside a validated progress view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProgressView {
    authority_identity: String,
    scope_identity: String,
    clock_identity: String,
    clock_revision: String,
    required_sources: Vec<String>,
    boundary: BoundaryWire,
    state: OpenClosed,
    captured_trigger_identity: String,
    cutoff_identity: String,
    restoration_basis_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    association: Option<Vec<AssociationRow>>,
}

impl ProgressView {
    /// Returns the FR-268 derived progress-authority identity.
    #[must_use]
    pub fn authority_identity(&self) -> &str {
        &self.authority_identity
    }

    /// Returns independent progress state without deriving closure or truth.
    #[must_use]
    pub const fn state(&self) -> OpenClosed {
        self.state
    }

    /// Returns the complete sorted required-source set.
    #[must_use]
    pub fn required_sources(&self) -> &[String] {
        &self.required_sources
    }

    /// Returns the exact subject scope identity.
    #[must_use]
    pub fn scope_identity(&self) -> &str {
        &self.scope_identity
    }

    /// Returns the selected clock identity.
    #[must_use]
    pub fn clock_identity(&self) -> &str {
        &self.clock_identity
    }

    /// Returns the selected clock revision.
    #[must_use]
    pub fn clock_revision(&self) -> &str {
        &self.clock_revision
    }

    /// Returns the exact FR-289 boundary mapping.
    #[must_use]
    pub fn boundary(&self) -> BoundaryRef<'_> {
        self.boundary.as_ref()
    }

    /// Returns the captured trigger identity.
    #[must_use]
    pub fn captured_trigger_identity(&self) -> &str {
        &self.captured_trigger_identity
    }

    /// Returns the independently derived cutoff identity.
    #[must_use]
    pub fn cutoff_identity(&self) -> &str {
        &self.cutoff_identity
    }

    /// Returns the exact restoration-basis identity.
    #[must_use]
    pub fn restoration_basis_identity(&self) -> &str {
        &self.restoration_basis_identity
    }
}

impl Validated<ProgressView> {
    /// Returns the owner asserted association value, refusing absent rows.
    pub fn association(&self) -> Result<AssociationRef<'_>> {
        match self.payload().association.as_deref() {
            Some(rows) if !rows.is_empty() => Ok(AssociationRef {
                progress_document_identity: self.identity().as_str(),
                progress_authority_identity: self.payload().authority_identity(),
                scope_identity: self.subject().scope_identity.as_str(),
                population_identity: self.subject().population_identity.as_str(),
                clock_identity: self.payload().clock_identity(),
                clock_revision: self.payload().clock_revision(),
                rows,
            }),
            _ => Err(Error::new(
                ErrorCode::MissingPremise,
                "progress has no QSL association",
                Usage::default(),
            )),
        }
    }

    /// Returns RFC 8785 bytes of the association value only, bounded by owner limits.
    /// The value includes progress document, authority, subject, and clock identities,
    /// plus rows sorted by `(model, model_node, claim, observation_source)`.
    pub fn association_canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>> {
        let association = self.association()?;
        // This closed value domain contains only strings, arrays, and objects.
        // serde_json::Value sorts the ASCII field names before encoding.
        let value = serde_json::to_value(association).map_err(|_| {
            Error::new(
                ErrorCode::InvalidSelection,
                "association is not JSON",
                Usage::default(),
            )
        })?;
        let bytes = serde_json::to_vec(&value).map_err(|_| {
            Error::new(
                ErrorCode::InvalidSelection,
                "association is not JSON",
                Usage::default(),
            )
        })?;
        if bytes.len() > limits.effective().max_output_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "association exceeds canonical bounds",
                Usage::default(),
            ));
        }
        Ok(bytes)
    }
}

fn validate_association(
    rows: &[AssociationRow],
    selection: &Selection,
    scope: &crate::ScopeSelection,
    limits: Limits,
) -> Result<()> {
    if rows.is_empty() {
        return Err(Error::new(
            ErrorCode::MissingPremise,
            "association rows are absent",
            Usage::default(),
        ));
    }
    if rows.len() > limits.max_required_sources || rows.len() > limits.max_population_entries {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "association exceeds owner row limit",
            Usage::default(),
        ));
    }
    let mut previous = None;
    let mut association_bytes = 0usize;
    for row in rows {
        association_bytes = association_bytes.saturating_add(row.claim.ordinal().to_string().len());
        for value in [
            row.model.as_str(),
            row.model_node.as_str(),
            row.claim.clause_node().as_str(),
            row.claim.role(),
            row.unit.as_str(),
            row.unit_node.as_str(),
            row.time_dimension.as_str(),
            row.observation_source.as_str(),
            row.observation_clock.as_str(),
            row.observation_revision.as_str(),
        ] {
            association_bytes = association_bytes.saturating_add(value.len());
            if association_bytes > limits.max_output_bytes {
                return Err(Error::new(
                    ErrorCode::ResourceIncomplete,
                    "association exceeds owner byte limit",
                    Usage::default(),
                ));
            }
        }
        let key = (
            &row.model,
            &row.model_node,
            &row.claim,
            &row.observation_source,
        );
        if previous.is_some_and(|prior| prior >= key) {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "association rows must be sorted and unique",
                Usage::default(),
            ));
        }
        previous = Some(key);
        if [row.model.as_str(), row.claim.role(), row.unit.as_str()]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "association QSL identities must be explicit",
                Usage::default(),
            ));
        }
        for node in [
            &row.model_node,
            row.claim.clause_node(),
            &row.unit_node,
            &row.time_dimension,
        ] {
            NodeIdentity::check_size(node.as_str(), limits)?;
        }
        if row.observation_clock != selection.clock.clock_identity().as_str()
            || row.observation_revision != selection.clock.clock_revision().as_str()
            || !scope
                .observation_sources
                .iter()
                .any(|source| source.as_str() == row.observation_source)
        {
            return Err(Error::new(
                ErrorCode::ExpectedMismatch,
                "association source or clock is foreign to admitted progress",
                Usage::default(),
            ));
        }
        if let TimeSource::Tick {
            operation,
            operation_node,
            period_numerator,
            period_denominator,
        } = &row.source
        {
            for value in [
                operation.as_str(),
                operation_node.as_str(),
                period_numerator.as_str(),
                period_denominator.as_str(),
            ] {
                association_bytes = association_bytes.saturating_add(value.len());
                if association_bytes > limits.max_output_bytes {
                    return Err(Error::new(
                        ErrorCode::ResourceIncomplete,
                        "association exceeds owner byte limit",
                        Usage::default(),
                    ));
                }
            }
            NodeIdentity::check_size(operation_node.as_str(), limits)?;
            if operation.trim().is_empty()
                || !valid_positive_decimal(period_numerator)
                || !valid_positive_decimal(period_denominator)
            {
                return Err(Error::new(
                    ErrorCode::InvalidSelection,
                    "tick association must name an operation and positive canonical period",
                    Usage::default(),
                ));
            }
        }
    }
    Ok(())
}

fn valid_positive_decimal(value: &str) -> bool {
    !value.is_empty()
        && value != "0"
        && !value.starts_with('0')
        && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_natural_decimal(value: &str) -> bool {
    !value.is_empty()
        && (value == "0" || !value.starts_with('0'))
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}

fn max_association_string(rows: &[AssociationRow]) -> usize {
    rows.iter()
        .flat_map(|row| {
            let source = match &row.source {
                TimeSource::Dense => ["", "", "", ""],
                TimeSource::Tick {
                    operation,
                    operation_node,
                    period_numerator,
                    period_denominator,
                } => [
                    operation.as_str(),
                    operation_node.as_str(),
                    period_numerator.as_str(),
                    period_denominator.as_str(),
                ],
            };
            [
                row.model.as_str(),
                row.model_node.as_str(),
                row.claim.clause_node().as_str(),
                row.claim.role(),
                row.unit.as_str(),
                row.unit_node.as_str(),
                row.time_dimension.as_str(),
                row.observation_source.as_str(),
                row.observation_clock.as_str(),
                row.observation_revision.as_str(),
                source[0],
                source[1],
                source[2],
                source[3],
            ]
        })
        .map(|text| {
            serde_json::to_string(text)
                .map(|encoded| encoded.len().saturating_sub(2))
                .unwrap_or(usize::MAX)
        })
        .max()
        .unwrap_or(0)
        .max(
            rows.iter()
                .map(|row| row.claim.ordinal().to_string().len())
                .max()
                .unwrap_or(0),
        )
}

/// Constructor-private validated progress view.
pub type View = Validated<ProgressView>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorityIdentityPreimage<'a> {
    identity_version: &'static str,
    definition_identity: &'a str,
    definition_revision: &'a str,
    definition_digest: String,
    scope_identity: &'a str,
    population_identity: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    binding_identity: Option<&'a str>,
    clock_identity: &'a str,
    clock_revision: &'a str,
    required_sources: Vec<&'a str>,
    boundary: BoundaryWire,
    restoration_basis_identity: &'a str,
}

/// Derives canonical bounded progress-assertion owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    ensure_sorted_unique(&selection.required_sources, effective.max_required_sources)?;
    selection.boundary.validate(selection.state)?;
    if !selection.clock.clock_identity().valid()
        || !selection.clock.clock_revision().valid()
        || !selection.captured_trigger_identity.valid()
        || !selection.restoration_basis_identity.valid()
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "progress selection identities must be explicit",
            Usage::default(),
        ));
    }
    let qualified = context.history().qualified();
    if let Some(rows) = &selection.association {
        validate_association(rows, selection, qualified.scope(), effective)?;
    }
    if &qualified.scope().clock_identity != selection.clock.clock_identity()
        || &qualified.scope().clock_revision != selection.clock.clock_revision()
        || selection.cutoff.clock_identity() != selection.clock.clock_identity()
        || selection.cutoff.clock_revision() != selection.clock.clock_revision()
        || !selection.boundary.matches_range(qualified.scope().range)
        || selection.required_sources != qualified.scope().observation_sources
        || selection.captured_trigger_identity != qualified.binding().trigger_identity
        || qualified.records().iter().any(|record| {
            &record.clock_identity != selection.clock.clock_identity()
                || &record.clock_revision != selection.clock.clock_revision()
        })
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "progress boundary, source, clock, or binding trigger is cross-wired",
            Usage::default(),
        ));
    }
    let cutoff_identity = super::observation::cutoff_identity(&selection.cutoff, limits)?;
    let authority_selection = context.authority();
    let subject = context.subject();
    let authority_identity = super::common::sha256_jcs(
        &AuthorityIdentityPreimage {
            identity_version: if qualified.records().is_empty() {
                "quire.observation.progress-authority-identity/v2-draft.1"
            } else {
                "quire.observation.progress-authority-identity/v1-draft.1"
            },
            definition_identity: authority_selection.definition_identity.as_str(),
            definition_revision: authority_selection.definition_revision.as_str(),
            definition_digest: super::common::digest_hex(&authority_selection.definition_digest),
            scope_identity: subject.scope_identity.as_str(),
            population_identity: subject.population_identity.as_str(),
            binding_identity: qualified
                .records()
                .is_empty()
                .then(|| qualified.binding().identity.as_str()),
            clock_identity: selection.clock.clock_identity().as_str(),
            clock_revision: selection.clock.clock_revision().as_str(),
            required_sources: selection
                .required_sources
                .iter()
                .map(Identity::as_str)
                .collect(),
            boundary: boundary_wire(selection.boundary),
            restoration_basis_identity: selection.restoration_basis_identity.as_str(),
        },
        limits,
    )?
    .0;
    let payload = ProgressView {
        authority_identity: authority_identity.as_str().to_owned(),
        scope_identity: super::common::scope_identity(qualified.scope().kind.clone()),
        clock_identity: selection.clock.clock_identity().as_str().to_owned(),
        clock_revision: selection.clock.clock_revision().as_str().to_owned(),
        required_sources: selection
            .required_sources
            .iter()
            .map(|value| value.as_str().to_owned())
            .collect(),
        boundary: boundary_wire(selection.boundary),
        state: selection.state,
        captured_trigger_identity: selection.captured_trigger_identity.as_str().to_owned(),
        cutoff_identity: cutoff_identity.as_str().to_owned(),
        restoration_basis_identity: selection.restoration_basis_identity.as_str().to_owned(),
        association: selection.association.clone(),
    };
    let usage = Usage {
        depth: 5,
        string_bytes: payload
            .required_sources
            .iter()
            .map(String::len)
            .max()
            .unwrap_or(0)
            .max(payload.authority_identity.len())
            .max(payload.scope_identity.len())
            .max(payload.clock_identity.len())
            .max(payload.clock_revision.len())
            .max(payload.captured_trigger_identity.len())
            .max(payload.cutoff_identity.len())
            .max(payload.restoration_basis_identity.len())
            .max(
                payload
                    .association
                    .as_ref()
                    .map_or(0, |rows| max_association_string(rows)),
            ),
        population_entries: payload.association.as_ref().map_or(0, Vec::len),
        required_sources: payload.required_sources.len(),
        visited_fields: 29usize
            .saturating_add(payload.required_sources.len())
            .saturating_add(
                payload
                    .association
                    .as_ref()
                    .map_or(0, |rows| rows.len().saturating_mul(18)),
            ),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical progress bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

pub(crate) fn commits_empty_binding(
    view: &View,
    binding_identity: &Identity,
    limits: Limits,
) -> Result<bool> {
    commits_binding_mode(
        view,
        "quire.observation.progress-authority-identity/v2-draft.1",
        Some(binding_identity),
        limits,
    )
}

pub(crate) fn commits_captured_history(view: &View, limits: Limits) -> Result<bool> {
    commits_binding_mode(
        view,
        "quire.observation.progress-authority-identity/v1-draft.1",
        None,
        limits,
    )
}

fn commits_binding_mode(
    view: &View,
    identity_version: &'static str,
    binding_identity: Option<&Identity>,
    limits: Limits,
) -> Result<bool> {
    if binding_identity.is_some_and(|identity| !identity.valid()) {
        return Ok(false);
    }
    let authority = view.authority();
    let subject = view.subject();
    let payload = view.payload();
    let (expected, _) = super::common::sha256_jcs(
        &AuthorityIdentityPreimage {
            identity_version,
            definition_identity: authority.definition_identity.as_str(),
            definition_revision: authority.definition_revision.as_str(),
            definition_digest: super::common::digest_hex(&authority.definition_digest),
            scope_identity: subject.scope_identity.as_str(),
            population_identity: subject.population_identity.as_str(),
            binding_identity: binding_identity.map(Identity::as_str),
            clock_identity: &payload.clock_identity,
            clock_revision: &payload.clock_revision,
            required_sources: payload
                .required_sources
                .iter()
                .map(String::as_str)
                .collect(),
            boundary: payload.boundary.clone(),
            restoration_basis_identity: &payload.restoration_basis_identity,
        },
        limits,
    )?;
    Ok(expected.as_str() == payload.authority_identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_is_closed() {
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
    }
}
