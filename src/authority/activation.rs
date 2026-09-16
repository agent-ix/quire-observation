// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Immutable activation identity and independent assessment-authority primitives.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, read_exact_preflighted, sha256_jcs, Validated};
use super::partial::EventTimeInterval;
use super::{
    AuthoritySelection, BoundaryRef, Context, Document, Error, ErrorCode, Limits, OpenClosed,
    Result, SubjectSelection, Usage,
};
use crate::Identity;

/// Closed activation classification independent from verdict and settlement.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivationState {
    /// The selected obligation is not activated.
    Inactive,
    /// The selected obligation is activated.
    Active,
    /// Available authority cannot establish activation or inactivity.
    Unknown,
}

/// Closed execution-state axis for progress and closure authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionState {
    /// The selected execution boundary remains open.
    Open,
    /// The selected execution boundary is closed.
    Closed,
    /// Required progress or closure authority is incomplete.
    Incomplete,
}

/// Closed evidence-state axis independent from execution closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceState {
    /// Every required evidence premise is complete.
    Complete,
    /// At least one required evidence premise is absent.
    Incomplete,
    /// Qualified evidence contains an explicit contradiction.
    Contradicted,
}

/// Three-way event lateness classification under exact closed intervals.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lateness {
    /// Every admissible event instant is at or before every cutoff instant.
    DefinitelyTimely,
    /// Every admissible event instant is after every cutoff instant.
    DefinitelyLate,
    /// Event and cutoff intervals admit both timely and late selections.
    Uncertain,
}

/// One immutable activation-time captured value and its exact provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capture {
    identity: Identity,
    source_observation_identity: Identity,
    value_type: Identity,
    canonical_value: String,
    provenance_identity: Identity,
}

impl Capture {
    /// Constructs an exact capture without ambient source or type defaults.
    #[must_use]
    pub fn new(
        identity: Identity,
        source_observation_identity: Identity,
        value_type: Identity,
        canonical_value: String,
        provenance_identity: Identity,
    ) -> Self {
        Self {
            identity,
            source_observation_identity,
            value_type,
            canonical_value,
            provenance_identity,
        }
    }

    /// Returns the capture binding identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the exact activation-time source observation identity.
    #[must_use]
    pub fn source_observation_identity(&self) -> &str {
        self.source_observation_identity.as_str()
    }

    /// Returns the exact captured value-type identity.
    #[must_use]
    pub fn value_type(&self) -> &str {
        self.value_type.as_str()
    }

    /// Returns the exact activation-time canonical value.
    #[must_use]
    pub fn canonical_value(&self) -> &str {
        &self.canonical_value
    }

    /// Returns the exact capture provenance identity.
    #[must_use]
    pub fn provenance_identity(&self) -> &str {
        self.provenance_identity.as_str()
    }
}

/// Exact reason that silence progress remains incomplete.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SilenceIncompleteReason {
    /// The progress frontier is not definitely beyond the deadline.
    ProgressNotDefinitelyBeyond,
    /// At least one selected required source is not covered.
    MissingRequiredSource,
    /// No validated matching progress assertion was supplied.
    MissingProgressAuthority,
}

/// Exact silence-coverage result; incomplete reasons are retained independently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SilenceCoverage {
    /// Matching progress is definitely beyond the deadline for every required source.
    Covered,
    /// Coverage is not established; every failed premise is retained.
    Incomplete(IncompleteSilence),
}

/// Constructor-private nonempty set of failed silence premises.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncompleteSilence {
    reasons: BTreeSet<SilenceIncompleteReason>,
}

impl SilenceCoverage {
    /// Returns whether an incomplete result retains the selected reason.
    #[must_use]
    pub fn has_reason(&self, reason: SilenceIncompleteReason) -> bool {
        match self {
            Self::Covered => false,
            Self::Incomplete(incomplete) => incomplete.reasons.contains(&reason),
        }
    }

    /// Iterates every failed premise in canonical order.
    pub fn reasons(&self) -> impl Iterator<Item = SilenceIncompleteReason> + '_ {
        let reasons = match self {
            Self::Covered => None,
            Self::Incomplete(incomplete) => Some(&incomplete.reasons),
        };
        reasons.into_iter().flatten().copied()
    }
}

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.activation-scope-authority/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-activation-scope-authority-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "c17ad8b3ec10c48e535bc3a762738b979206c01af20c31775d8a9a9e0a97fab6";

/// Optional evaluator-owned contribution with an exact nonempty support set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    identity: Identity,
    support: Vec<Identity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProofCommon {
    document_identity: Identity,
    revision: u64,
    authority: AuthoritySelection,
    subject: SubjectSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CaptureProof {
    common: ProofCommon,
    trigger_identity: String,
    bindings: Vec<CaptureProofBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CaptureProofBinding {
    capture_identity: String,
    observation_identity: String,
    value_type: String,
    canonical_value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProgressProof {
    common: ProofCommon,
    binding_identity: Option<String>,
    scope_identity: String,
    clock_identity: String,
    clock_revision: String,
    required_sources: Vec<String>,
    frontier: i128,
    state: OpenClosed,
    captured_trigger_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClosureProof {
    common: ProofCommon,
    scope_identity: String,
    clock_identity: String,
    clock_revision: String,
    required_sources: Vec<String>,
    state: OpenClosed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompletenessProof {
    common: ProofCommon,
    population_identity: String,
    state: super::completeness::State,
}

/// Constructor-private proof that every selected authority came from a strict reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityProofs {
    capture: Option<CaptureProof>,
    progress: Option<ProgressProof>,
    closure: Option<ClosureProof>,
    completeness: Option<CompletenessProof>,
}

impl AuthorityProofs {
    /// Copies exact evidence from constructor-private validated owner views.
    pub fn new(
        capture: &super::capture::View,
        progress: Option<&super::progress::View>,
        closure: Option<&super::closure::View>,
        completeness: Option<&super::completeness::View>,
    ) -> Result<Self> {
        let mut capture_bindings = Vec::new();
        capture_bindings
            .try_reserve(capture.payload().bindings().len())
            .map_err(|_| {
                Error::new(
                    ErrorCode::ResourceIncomplete,
                    "capture proof allocation failed",
                    Usage::default(),
                )
            })?;
        capture_bindings.extend(
            capture
                .payload()
                .bindings()
                .map(|binding| CaptureProofBinding {
                    capture_identity: binding.capture_identity.to_owned(),
                    observation_identity: binding.observation_identity.to_owned(),
                    value_type: binding.value_type.to_owned(),
                    canonical_value: binding.canonical_value.to_owned(),
                }),
        );
        let capture_proof = CaptureProof {
            common: proof_common(capture),
            trigger_identity: capture.payload().trigger_identity().to_owned(),
            bindings: capture_bindings,
        };
        let progress = progress
            .map(|view| {
                if !super::progress::commits_captured_history(view, Limits::owner_max())? {
                    return Err(Error::new(
                        ErrorCode::AuthorityMismatch,
                        "progress authority does not commit captured-history mode",
                        Usage::default(),
                    ));
                }
                Ok(ProgressProof {
                    common: proof_common(view),
                    binding_identity: None,
                    scope_identity: view.payload().scope_identity().to_owned(),
                    clock_identity: view.payload().clock_identity().to_owned(),
                    clock_revision: view.payload().clock_revision().to_owned(),
                    required_sources: copy_strings(view.payload().required_sources())?,
                    frontier: boundary_frontier(view.payload().boundary())?,
                    state: view.payload().state(),
                    captured_trigger_identity: view
                        .payload()
                        .captured_trigger_identity()
                        .to_owned(),
                })
            })
            .transpose()?;
        let closure = closure
            .map(|view| {
                Ok(ClosureProof {
                    common: proof_common(view),
                    scope_identity: view.payload().scope_identity().to_owned(),
                    clock_identity: view.payload().clock_identity().to_owned(),
                    clock_revision: view.payload().clock_revision().to_owned(),
                    required_sources: copy_strings(view.payload().required_sources())?,
                    state: view.payload().state(),
                })
            })
            .transpose()?;
        let completeness = completeness.map(|view| CompletenessProof {
            common: proof_common(view),
            population_identity: view.payload().population_identity().to_owned(),
            state: view.payload().state(),
        });
        Ok(Self {
            capture: Some(capture_proof),
            progress,
            closure,
            completeness,
        })
    }

    /// Copies scope authority for a trigger scope with no admitted trigger.
    pub fn without_capture(
        binding_identity: &Identity,
        progress: Option<&super::progress::View>,
        closure: Option<&super::closure::View>,
        completeness: Option<&super::completeness::View>,
    ) -> Result<Self> {
        let progress = progress
            .map(|view| {
                if !super::progress::commits_empty_binding(
                    view,
                    binding_identity,
                    Limits::owner_max(),
                )? {
                    return Err(Error::new(
                        ErrorCode::AuthorityMismatch,
                        "progress authority does not commit the selected empty binding",
                        Usage::default(),
                    ));
                }
                Ok(ProgressProof {
                    common: proof_common(view),
                    binding_identity: Some(binding_identity.as_str().to_owned()),
                    scope_identity: view.payload().scope_identity().to_owned(),
                    clock_identity: view.payload().clock_identity().to_owned(),
                    clock_revision: view.payload().clock_revision().to_owned(),
                    required_sources: copy_strings(view.payload().required_sources())?,
                    frontier: boundary_frontier(view.payload().boundary())?,
                    state: view.payload().state(),
                    captured_trigger_identity: view
                        .payload()
                        .captured_trigger_identity()
                        .to_owned(),
                })
            })
            .transpose()?;
        let closure = closure
            .map(|view| {
                Ok(ClosureProof {
                    common: proof_common(view),
                    scope_identity: view.payload().scope_identity().to_owned(),
                    clock_identity: view.payload().clock_identity().to_owned(),
                    clock_revision: view.payload().clock_revision().to_owned(),
                    required_sources: copy_strings(view.payload().required_sources())?,
                    state: view.payload().state(),
                })
            })
            .transpose()?;
        let completeness = completeness.map(|view| CompletenessProof {
            common: proof_common(view),
            population_identity: view.payload().population_identity().to_owned(),
            state: view.payload().state(),
        });
        Ok(Self {
            capture: None,
            progress,
            closure,
            completeness,
        })
    }
}

impl Contribution {
    /// Constructs a contribution selection without interpreting its semantics.
    #[must_use]
    pub fn new(identity: Identity, support: Vec<Identity>) -> Self {
        Self { identity, support }
    }
}

/// Exact immutable activation selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationSelection {
    obligation_identity: Identity,
    binding_identity: Identity,
    trigger_identity: Identity,
    trigger_observation_identity: Option<Identity>,
    interval: EventTimeInterval,
    captures: Vec<Capture>,
}

impl ActivationSelection {
    /// Constructs an activation selection with its complete capture set.
    #[must_use]
    pub fn new(
        obligation_identity: Identity,
        binding_identity: Identity,
        trigger_identity: Identity,
        trigger_observation_identity: Identity,
        interval: EventTimeInterval,
        captures: Vec<Capture>,
    ) -> Self {
        Self {
            obligation_identity,
            binding_identity,
            trigger_identity,
            trigger_observation_identity: Some(trigger_observation_identity),
            interval,
            captures,
        }
    }

    /// Selects a trigger scope for which no trigger was admitted.
    #[must_use]
    pub fn without_trigger(
        obligation_identity: Identity,
        binding_identity: Identity,
        trigger_identity: Identity,
        interval: EventTimeInterval,
    ) -> Self {
        Self {
            obligation_identity,
            binding_identity,
            trigger_identity,
            trigger_observation_identity: None,
            interval,
            captures: Vec::new(),
        }
    }
}

/// Exact progress/deadline selection used to classify silence coverage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressSelection {
    deadline: EventTimeInterval,
    progress: EventTimeInterval,
}

impl ProgressSelection {
    /// Constructs progress authority without inferring closure or evidence state.
    #[must_use]
    pub const fn new(deadline: EventTimeInterval, progress: EventTimeInterval) -> Self {
        Self { deadline, progress }
    }
}

/// Complete selection for one activation/scope-authority owner fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    activation: ActivationSelection,
    progress: ProgressSelection,
    event_interval: EventTimeInterval,
    cutoff_interval: EventTimeInterval,
    proofs: AuthorityProofs,
    verdict: Option<Contribution>,
    settlement: Option<Contribution>,
}

impl Selection {
    /// Constructs independent assessment axes and optional evaluator contributions.
    #[must_use]
    pub fn new(
        activation: ActivationSelection,
        progress: ProgressSelection,
        event_interval: EventTimeInterval,
        cutoff_interval: EventTimeInterval,
        proofs: AuthorityProofs,
    ) -> Self {
        Self {
            activation,
            progress,
            event_interval,
            cutoff_interval,
            proofs,
            verdict: None,
            settlement: None,
        }
    }

    /// Attaches evaluator-owned contributions without promoting either axis.
    #[must_use]
    pub fn with_contributions(
        mut self,
        verdict: Option<Contribution>,
        settlement: Option<Contribution>,
    ) -> Self {
        self.verdict = verdict;
        self.settlement = settlement;
        self
    }
}

/// Read-only activation/scope-authority payload.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivationView {
    activation_identity: String,
    obligation_identity: String,
    trigger_identity: String,
    trigger_observation_identity: Option<String>,
    activation_interval: IntervalWire,
    captures: Vec<CaptureWire>,
    capture_authority: Option<ProofWire>,
    activation: ActivationState,
    deadline_interval: IntervalWire,
    progress_interval: IntervalWire,
    required_sources: Vec<String>,
    covered_sources: Vec<String>,
    progress: ExecutionState,
    silence: SilenceWire,
    progress_authority: Option<ProofWire>,
    closure: ExecutionState,
    closure_authority: Option<ProofWire>,
    evidence: EvidenceState,
    completeness_authority: Option<ProofWire>,
    event_interval: IntervalWire,
    cutoff_interval: IntervalWire,
    lateness: Lateness,
    verdict: Option<ContributionWire>,
    settlement: Option<ContributionWire>,
}

/// Borrowed exact interval exposed by a validated activation view.
pub type IntervalRef<'a> = super::partial::IntervalRef<'a>;

/// Borrowed immutable activation-time capture exposed by a validated view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CaptureRef<'a> {
    /// Exact capture binding identity.
    pub identity: &'a str,
    /// Exact admitted observation captured at activation.
    pub source_observation_identity: &'a str,
    /// Exact value-type identity.
    pub value_type: &'a str,
    /// Canonical captured value text.
    pub canonical_value: &'a str,
    /// Exact provenance identity retained at activation.
    pub provenance_identity: &'a str,
}

/// Borrowed evaluator-owned contribution exposed by a validated view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContributionRef<'a> {
    /// Exact contribution identity.
    pub identity: &'a str,
    /// Complete sorted support set.
    pub support: &'a [String],
}

/// Borrowed strict-read authority evidence retained by the assessment fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorityRef<'a> {
    /// Exact owner-document identity.
    pub document_identity: &'a str,
    /// Exact positive owner-document revision.
    pub revision: u64,
}

impl ActivationView {
    /// Returns the canonical activation identity.
    #[must_use]
    pub fn activation_identity(&self) -> &str {
        &self.activation_identity
    }

    /// Returns the selected obligation identity.
    #[must_use]
    pub fn obligation_identity(&self) -> &str {
        &self.obligation_identity
    }

    /// Returns the exact semantic trigger identity assessed for presence or absence.
    #[must_use]
    pub fn trigger_identity(&self) -> &str {
        &self.trigger_identity
    }

    /// Returns the selected trigger observation identity.
    #[must_use]
    pub fn trigger_observation_identity(&self) -> Option<&str> {
        self.trigger_observation_identity.as_deref()
    }

    /// Returns the exact activation interval.
    #[must_use]
    pub fn activation_interval(&self) -> IntervalRef<'_> {
        self.activation_interval.as_ref()
    }

    /// Returns the exact activation classification.
    #[must_use]
    pub const fn activation(&self) -> ActivationState {
        self.activation
    }

    /// Returns the exact progress state.
    #[must_use]
    pub const fn progress(&self) -> ExecutionState {
        self.progress
    }

    /// Returns matching progress authority, or `None` for explicit incompleteness.
    #[must_use]
    pub fn progress_authority(&self) -> Option<AuthorityRef<'_>> {
        self.progress_authority.as_ref().map(ProofWire::as_ref)
    }

    /// Returns the exact closure state.
    #[must_use]
    pub const fn closure(&self) -> ExecutionState {
        self.closure
    }

    /// Returns matching closure authority, or `None` for explicit incompleteness.
    #[must_use]
    pub fn closure_authority(&self) -> Option<AuthorityRef<'_>> {
        self.closure_authority.as_ref().map(ProofWire::as_ref)
    }

    /// Returns the exact evidence state.
    #[must_use]
    pub const fn evidence(&self) -> EvidenceState {
        self.evidence
    }

    /// Returns matching completeness authority, or `None` for missing evidence.
    #[must_use]
    pub fn completeness_authority(&self) -> Option<AuthorityRef<'_>> {
        self.completeness_authority.as_ref().map(ProofWire::as_ref)
    }

    /// Returns the owner-derived interval lateness.
    #[must_use]
    pub const fn lateness(&self) -> Lateness {
        self.lateness
    }

    /// Iterates retained activation captures in canonical order.
    #[must_use]
    pub fn captures(&self) -> impl ExactSizeIterator<Item = CaptureRef<'_>> {
        self.captures.iter().map(CaptureWire::as_ref)
    }

    /// Returns the strict-read complete capture-environment authority.
    #[must_use]
    pub fn capture_authority(&self) -> Option<AuthorityRef<'_>> {
        self.capture_authority.as_ref().map(ProofWire::as_ref)
    }

    /// Returns the exact deadline interval.
    #[must_use]
    pub fn deadline_interval(&self) -> IntervalRef<'_> {
        self.deadline_interval.as_ref()
    }

    /// Returns the exact progress interval.
    #[must_use]
    pub fn progress_interval(&self) -> IntervalRef<'_> {
        self.progress_interval.as_ref()
    }

    /// Returns the complete sorted required-source set.
    #[must_use]
    pub fn required_sources(&self) -> &[String] {
        &self.required_sources
    }

    /// Returns the complete sorted covered-source set.
    #[must_use]
    pub fn covered_sources(&self) -> &[String] {
        &self.covered_sources
    }

    /// Returns whether matching progress established silence coverage.
    #[must_use]
    pub const fn silence_covered(&self) -> bool {
        matches!(self.silence, SilenceWire::Covered)
    }

    /// Iterates every failed silence premise in canonical order.
    pub fn silence_incomplete_reasons(&self) -> impl Iterator<Item = SilenceIncompleteReason> + '_ {
        let reasons = match &self.silence {
            SilenceWire::Covered => None,
            SilenceWire::Incomplete { reasons } => Some(reasons.as_slice()),
        };
        reasons.into_iter().flatten().map(|reason| match reason {
            SilenceReasonWire::ProgressNotDefinitelyBeyond => {
                SilenceIncompleteReason::ProgressNotDefinitelyBeyond
            }
            SilenceReasonWire::MissingRequiredSource => {
                SilenceIncompleteReason::MissingRequiredSource
            }
            SilenceReasonWire::MissingProgressAuthority => {
                SilenceIncompleteReason::MissingProgressAuthority
            }
        })
    }

    /// Returns the exact event interval used for lateness.
    #[must_use]
    pub fn event_interval(&self) -> IntervalRef<'_> {
        self.event_interval.as_ref()
    }

    /// Returns the exact cutoff interval used for lateness.
    #[must_use]
    pub fn cutoff_interval(&self) -> IntervalRef<'_> {
        self.cutoff_interval.as_ref()
    }

    /// Returns the evaluator-owned verdict contribution, when supplied.
    #[must_use]
    pub fn verdict(&self) -> Option<ContributionRef<'_>> {
        self.verdict.as_ref().map(ContributionWire::as_ref)
    }

    /// Returns the evaluator-owned settlement contribution, when supplied.
    #[must_use]
    pub fn settlement(&self) -> Option<ContributionRef<'_>> {
        self.settlement.as_ref().map(ContributionWire::as_ref)
    }
}

/// Constructor-private validated activation/scope-authority view.
pub type View = Validated<ActivationView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntervalWire {
    clock_identity: String,
    clock_revision: String,
    unit: String,
    earliest: String,
    latest: String,
}

impl IntervalWire {
    fn as_ref(&self) -> IntervalRef<'_> {
        IntervalRef {
            clock_identity: &self.clock_identity,
            clock_revision: &self.clock_revision,
            unit: &self.unit,
            earliest: &self.earliest,
            latest: &self.latest,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureWire {
    identity: String,
    source_observation_identity: String,
    value_type: String,
    canonical_value: String,
    provenance_identity: String,
}

impl CaptureWire {
    fn as_ref(&self) -> CaptureRef<'_> {
        CaptureRef {
            identity: &self.identity,
            source_observation_identity: &self.source_observation_identity,
            value_type: &self.value_type,
            canonical_value: &self.canonical_value,
            provenance_identity: &self.provenance_identity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum SilenceWire {
    Covered,
    Incomplete { reasons: Vec<SilenceReasonWire> },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SilenceReasonWire {
    ProgressNotDefinitelyBeyond,
    MissingRequiredSource,
    MissingProgressAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContributionWire {
    identity: String,
    support: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofWire {
    document_identity: String,
    revision: u64,
}

impl ProofWire {
    fn as_ref(&self) -> AuthorityRef<'_> {
        AuthorityRef {
            document_identity: &self.document_identity,
            revision: self.revision,
        }
    }
}

impl ContributionWire {
    fn as_ref(&self) -> ContributionRef<'_> {
        ContributionRef {
            identity: &self.identity,
            support: &self.support,
        }
    }
}

fn proof_common<P>(view: &Validated<P>) -> ProofCommon {
    ProofCommon {
        document_identity: view.identity().clone(),
        revision: view.revision(),
        authority: view.authority().clone(),
        subject: view.subject().clone(),
    }
}

fn boundary_frontier(boundary: BoundaryRef<'_>) -> Result<i128> {
    let value = match boundary {
        BoundaryRef::EventPosition { watermark, .. }
        | BoundaryRef::FixedSample { watermark, .. } => watermark,
        BoundaryRef::TimestampedEvent {
            watermark_nanos, ..
        } => watermark_nanos,
    };
    value.parse().map_err(|_| {
        Error::new(
            ErrorCode::InvalidSelection,
            "validated progress boundary is outside the exact integer domain",
            Usage::default(),
        )
    })
}

fn proof_wire(common: &ProofCommon) -> ProofWire {
    ProofWire {
        document_identity: common.document_identity.as_str().to_owned(),
        revision: common.revision,
    }
}

fn copy_strings(values: &[String]) -> Result<Vec<String>> {
    let mut copied = Vec::new();
    copied.try_reserve(values.len()).map_err(|_| {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "authority proof allocation failed",
            Usage::default(),
        )
    })?;
    copied.extend(values.iter().cloned());
    Ok(copied)
}

/// Derives canonical bounded activation/scope-authority bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let qualified = context.history().qualified();
    let effective = limits.effective();
    if selection.activation.binding_identity != qualified.binding().identity
        || selection.activation.trigger_identity != qualified.binding().trigger_identity
    {
        return Err(Error::new(
            ErrorCode::AuthorityMismatch,
            "activation binding or trigger is cross-wired from qualified history",
            Usage::default(),
        ));
    }
    if qualified.records().len() > effective.max_population_entries {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "qualified record index exceeds the effective population bound",
            Usage {
                population_entries: qualified.records().len(),
                ..Usage::default()
            },
        ));
    }
    let record_index = qualified
        .records()
        .iter()
        .map(|record| (&record.identity, record))
        .collect::<BTreeMap<_, _>>();
    for interval in [
        &selection.activation.interval,
        &selection.progress.deadline,
        &selection.progress.progress,
        &selection.event_interval,
        &selection.cutoff_interval,
    ] {
        validate_scope_interval(interval, qualified, &selection.activation.interval, limits)?;
    }
    let activation_id = activation_scope_identity(
        &selection.activation.obligation_identity,
        Some(&selection.activation.binding_identity),
        &selection.activation.trigger_identity,
        selection.activation.trigger_observation_identity.as_ref(),
        &selection.activation.interval,
        &selection.activation.captures,
        limits,
    )?;
    let (trigger, capture_authority) = match (
        selection.activation.trigger_observation_identity.as_ref(),
        selection.proofs.capture.as_ref(),
    ) {
        (Some(trigger_identity), Some(capture_proof)) => {
            let trigger = record_index.get(trigger_identity).copied().ok_or_else(|| {
                Error::new(
                    ErrorCode::MissingPremise,
                    "activation trigger is absent from qualified history",
                    Usage::default(),
                )
            })?;
            if trigger.trigger_identity != selection.activation.trigger_identity
                || trigger.clock_identity != *selection.activation.interval.clock_identity()
                || trigger.clock_revision != *selection.activation.interval.clock_revision()
            {
                return Err(Error::new(
                    ErrorCode::AuthorityMismatch,
                    "activation interval is cross-wired from its trigger observation clock",
                    Usage::default(),
                ));
            }
            validate_proof_common(&capture_proof.common, context)?;
            if capture_proof.trigger_identity != trigger.trigger_identity.as_str()
                || capture_proof.bindings.len() != selection.activation.captures.len()
                || selection
                    .activation
                    .captures
                    .iter()
                    .zip(&capture_proof.bindings)
                    .any(|(capture, proof)| {
                        capture.identity.as_str() != proof.capture_identity
                            || capture.source_observation_identity.as_str()
                                != proof.observation_identity
                            || capture.value_type.as_str() != proof.value_type
                            || capture.canonical_value != proof.canonical_value
                    })
            {
                return Err(Error::new(
                    ErrorCode::CaptureMismatch,
                    "activation captures do not exactly match complete capture authority",
                    Usage::default(),
                ));
            }
            for capture in &selection.activation.captures {
                let record = record_index
                    .get(&capture.source_observation_identity)
                    .copied()
                    .ok_or_else(|| {
                        Error::new(
                            ErrorCode::MissingPremise,
                            "capture source is absent from qualified history",
                            Usage::default(),
                        )
                    })?;
                let crate::ValueState::Present {
                    value_type,
                    canonical_value,
                } = &record.value
                else {
                    return Err(Error::new(
                        ErrorCode::MissingPremise,
                        "capture source has no qualified value",
                        Usage::default(),
                    ));
                };
                if value_type != &capture.value_type
                    || canonical_value != &capture.canonical_value
                    || record.source_identity != capture.provenance_identity
                {
                    return Err(Error::new(
                        ErrorCode::CaptureMismatch,
                        "capture value or provenance is cross-wired from qualified history",
                        Usage::default(),
                    ));
                }
            }
            (Some(trigger), Some(proof_wire(&capture_proof.common)))
        }
        (None, None) if selection.activation.captures.is_empty() => {
            if qualified.binding().required {
                return Err(Error::new(
                    ErrorCode::MissingPremise,
                    "required activation trigger is absent from qualified history",
                    Usage::default(),
                ));
            }
            if qualified
                .records()
                .iter()
                .any(|record| record.trigger_identity == selection.activation.trigger_identity)
            {
                return Err(Error::new(
                    ErrorCode::MissingPremise,
                    "trigger absence is contradicted by qualified history",
                    Usage::default(),
                ));
            }
            (None, None)
        }
        _ => {
            return Err(Error::new(
                ErrorCode::CaptureMismatch,
                "trigger, captures, and strict capture authority must be present together",
                Usage::default(),
            ))
        }
    };
    let required = sorted_sources(&qualified.scope().observation_sources, limits)?;
    let (progress, progress_authority) = match &selection.proofs.progress {
        Some(proof) => {
            validate_proof_common(&proof.common, context)?;
            let expected_binding = trigger
                .is_none()
                .then(|| selection.activation.binding_identity.as_str());
            if proof.scope_identity != context.subject().scope_identity.as_str()
                || proof.binding_identity.as_deref() != expected_binding
                || proof.clock_identity != selection.progress.progress.clock_identity().as_str()
                || proof.clock_revision != selection.progress.progress.clock_revision().as_str()
                || proof.required_sources != required
                || proof.captured_trigger_identity != selection.activation.trigger_identity.as_str()
                || selection.progress.progress.earliest() != proof.frontier
                || selection.progress.progress.latest() != proof.frontier
            {
                return Err(Error::new(
                    ErrorCode::AuthorityMismatch,
                    "progress interval or authority is cross-wired",
                    Usage::default(),
                ));
            }
            (
                match proof.state {
                    OpenClosed::Open => ExecutionState::Open,
                    OpenClosed::Closed => ExecutionState::Closed,
                },
                Some(proof_wire(&proof.common)),
            )
        }
        None => (ExecutionState::Incomplete, None),
    };
    let covered_identities = if progress_authority.is_some() {
        qualified.scope().observation_sources.as_slice()
    } else {
        &[]
    };
    let covered = sorted_sources(covered_identities, limits)?;
    let mut silence = silence_coverage(
        &selection.progress.deadline,
        &selection.progress.progress,
        &qualified.scope().observation_sources,
        covered_identities,
        limits,
    )?;
    if progress_authority.is_none() {
        silence = silence_with_reason(silence, SilenceIncompleteReason::MissingProgressAuthority);
    }
    let (closure, closure_authority) = match &selection.proofs.closure {
        Some(proof) => {
            validate_proof_common(&proof.common, context)?;
            if proof.scope_identity != context.subject().scope_identity.as_str()
                || proof.clock_identity != selection.activation.interval.clock_identity().as_str()
                || proof.clock_revision != selection.activation.interval.clock_revision().as_str()
                || proof.required_sources != required
            {
                return Err(Error::new(
                    ErrorCode::AuthorityMismatch,
                    "closure authority is cross-wired",
                    Usage::default(),
                ));
            }
            (
                match proof.state {
                    OpenClosed::Open => ExecutionState::Open,
                    OpenClosed::Closed => ExecutionState::Closed,
                },
                Some(proof_wire(&proof.common)),
            )
        }
        None => (ExecutionState::Incomplete, None),
    };
    let (evidence, completeness_authority) = match &selection.proofs.completeness {
        Some(proof) => {
            validate_proof_common(&proof.common, context)?;
            if proof.population_identity != context.subject().population_identity.as_str() {
                return Err(Error::new(
                    ErrorCode::AuthorityMismatch,
                    "completeness authority is cross-wired",
                    Usage::default(),
                ));
            }
            (
                match proof.state {
                    super::completeness::State::Complete => EvidenceState::Complete,
                    super::completeness::State::Incomplete => EvidenceState::Incomplete,
                    super::completeness::State::Contradicted => EvidenceState::Contradicted,
                },
                Some(proof_wire(&proof.common)),
            )
        }
        None => (EvidenceState::Incomplete, None),
    };
    let activation = if trigger.is_some() {
        ActivationState::Active
    } else if closure == ExecutionState::Closed && evidence == EvidenceState::Complete {
        ActivationState::Inactive
    } else {
        ActivationState::Unknown
    };
    let lateness = classify_lateness(
        &selection.event_interval,
        &selection.cutoff_interval,
        limits,
    )?;
    let verdict = contribution_wire(selection.verdict.as_ref(), &record_index, limits)?;
    let settlement = contribution_wire(selection.settlement.as_ref(), &record_index, limits)?;
    let mut captures = Vec::new();
    captures
        .try_reserve(selection.activation.captures.len())
        .map_err(|_| {
            Error::new(
                ErrorCode::ResourceIncomplete,
                "activation capture allocation failed",
                Usage::default(),
            )
        })?;
    captures.extend(selection.activation.captures.iter().map(capture_wire));
    captures.sort_unstable_by(|left, right| left.identity.cmp(&right.identity));
    let payload = ActivationView {
        activation_identity: activation_id.as_str().to_owned(),
        obligation_identity: selection.activation.obligation_identity.as_str().to_owned(),
        trigger_identity: selection.activation.trigger_identity.as_str().to_owned(),
        trigger_observation_identity: selection
            .activation
            .trigger_observation_identity
            .as_ref()
            .map(|identity| identity.as_str().to_owned()),
        activation_interval: interval_wire(&selection.activation.interval),
        captures,
        capture_authority,
        activation,
        deadline_interval: interval_wire(&selection.progress.deadline),
        progress_interval: interval_wire(&selection.progress.progress),
        required_sources: required,
        covered_sources: covered,
        progress,
        silence: silence_wire(silence),
        progress_authority,
        closure,
        closure_authority,
        evidence,
        completeness_authority,
        event_interval: interval_wire(&selection.event_interval),
        cutoff_interval: interval_wire(&selection.cutoff_interval),
        lateness,
        verdict,
        settlement,
    };
    build_document(
        CONTRACT,
        context,
        &payload,
        Usage {
            capture_bindings: payload.captures.len(),
            required_sources: payload
                .required_sources
                .len()
                .max(payload.covered_sources.len()),
            ..Usage::default()
        },
        limits,
    )
}

/// Strict-reads activation/scope-authority bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let observed = super::common::preflight_for_contract(bytes, limits.effective(), CONTRACT)?;
    let expected = derive(context, selection, limits)?;
    read_exact_preflighted(CONTRACT, bytes, &expected, limits, observed)
}

fn validate_proof_common(common: &ProofCommon, context: Context<'_>) -> Result<()> {
    if common.authority != *context.authority()
        || common.subject != *context.subject()
        || !common.document_identity.valid()
        || common.revision != context.revision()
    {
        return Err(Error::new(
            ErrorCode::AuthorityMismatch,
            "strict-read authority identity or revision is cross-wired",
            Usage::default(),
        ));
    }
    Ok(())
}

fn silence_with_reason(
    coverage: SilenceCoverage,
    reason: SilenceIncompleteReason,
) -> SilenceCoverage {
    let mut reasons = match coverage {
        SilenceCoverage::Covered => BTreeSet::new(),
        SilenceCoverage::Incomplete(incomplete) => incomplete.reasons,
    };
    reasons.insert(reason);
    SilenceCoverage::Incomplete(IncompleteSilence { reasons })
}

fn validate_scope_interval(
    interval: &EventTimeInterval,
    qualified: &crate::QualifiedObservation,
    domain: &EventTimeInterval,
    limits: Limits,
) -> Result<()> {
    interval.validate_limits(limits)?;
    if interval.clock_identity() != &qualified.scope().clock_identity
        || interval.clock_revision() != &qualified.scope().clock_revision
        || interval.unit() != domain.unit()
    {
        return Err(Error::new(
            ErrorCode::AuthorityMismatch,
            "assessment interval is cross-wired from the qualified scope domain",
            Usage::default(),
        ));
    }
    Ok(())
}

fn interval_wire(interval: &EventTimeInterval) -> IntervalWire {
    IntervalWire {
        clock_identity: interval.clock_identity().as_str().to_owned(),
        clock_revision: interval.clock_revision().as_str().to_owned(),
        unit: interval.unit().as_str().to_owned(),
        earliest: interval.earliest().to_string(),
        latest: interval.latest().to_string(),
    }
}

fn capture_wire(capture: &Capture) -> CaptureWire {
    CaptureWire {
        identity: capture.identity.as_str().to_owned(),
        source_observation_identity: capture.source_observation_identity.as_str().to_owned(),
        value_type: capture.value_type.as_str().to_owned(),
        canonical_value: capture.canonical_value.clone(),
        provenance_identity: capture.provenance_identity.as_str().to_owned(),
    }
}

fn silence_wire(coverage: SilenceCoverage) -> SilenceWire {
    match coverage {
        SilenceCoverage::Covered => SilenceWire::Covered,
        SilenceCoverage::Incomplete(incomplete) => SilenceWire::Incomplete {
            reasons: incomplete
                .reasons
                .into_iter()
                .map(|reason| match reason {
                    SilenceIncompleteReason::ProgressNotDefinitelyBeyond => {
                        SilenceReasonWire::ProgressNotDefinitelyBeyond
                    }
                    SilenceIncompleteReason::MissingRequiredSource => {
                        SilenceReasonWire::MissingRequiredSource
                    }
                    SilenceIncompleteReason::MissingProgressAuthority => {
                        SilenceReasonWire::MissingProgressAuthority
                    }
                })
                .collect(),
        },
    }
}

fn sorted_sources(sources: &[Identity], limits: Limits) -> Result<Vec<String>> {
    let effective = limits.effective();
    if sources.len() > effective.max_required_sources {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "source set exceeds the effective required-source bound",
            Usage {
                required_sources: sources.len(),
                ..Usage::default()
            },
        ));
    }
    let set = exact_source_set(sources, effective)?;
    Ok(set.into_iter().map(str::to_owned).collect())
}

fn contribution_wire(
    contribution: Option<&Contribution>,
    record_index: &BTreeMap<&Identity, &crate::AdmittedRecord>,
    limits: Limits,
) -> Result<Option<ContributionWire>> {
    let Some(contribution) = contribution else {
        return Ok(None);
    };
    if !contribution.identity.valid() || contribution.support.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "contribution identity and nonempty support are required",
            Usage::default(),
        ));
    }
    validate_string(contribution.identity.as_str(), limits.effective())?;
    let support = sorted_sources(&contribution.support, limits)?;
    if contribution
        .support
        .iter()
        .any(|identity| !record_index.contains_key(identity))
    {
        return Err(Error::new(
            ErrorCode::SupportMismatch,
            "contribution support is absent from qualified history",
            Usage::default(),
        ));
    }
    Ok(Some(ContributionWire {
        identity: contribution.identity.as_str().to_owned(),
        support,
    }))
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct ActivationPreimage<'a> {
    identity_version: &'static str,
    obligation_identity: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    binding_identity: Option<&'a str>,
    trigger_identity: &'a str,
    trigger_observation_identity: Option<&'a str>,
    interval: IntervalPreimage<'a>,
    captures: Vec<CapturePreimage<'a>>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct IntervalPreimage<'a> {
    clock_identity: &'a str,
    clock_revision: &'a str,
    unit: &'a str,
    earliest: String,
    latest: String,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct CapturePreimage<'a> {
    identity: &'a str,
    source_observation_identity: &'a str,
    value_type: &'a str,
    canonical_value: &'a str,
    provenance_identity: &'a str,
}

/// Derives the exact activation identity from semantic trigger, admitted observation,
/// interval, and complete capture set.
pub fn activation_identity(
    obligation_identity: &Identity,
    trigger_identity: &Identity,
    trigger_observation_identity: &Identity,
    interval: &EventTimeInterval,
    captures: &[Capture],
    limits: Limits,
) -> Result<Identity> {
    activation_scope_identity(
        obligation_identity,
        None,
        trigger_identity,
        Some(trigger_observation_identity),
        interval,
        captures,
        limits,
    )
}

fn activation_scope_identity(
    obligation_identity: &Identity,
    binding_identity: Option<&Identity>,
    trigger_identity: &Identity,
    trigger_observation_identity: Option<&Identity>,
    interval: &EventTimeInterval,
    captures: &[Capture],
    limits: Limits,
) -> Result<Identity> {
    interval.validate_limits(limits)?;
    let effective = limits.effective();
    if !obligation_identity.valid()
        || binding_identity.is_some_and(|identity| !identity.valid())
        || !trigger_identity.valid()
        || trigger_observation_identity.is_some_and(|identity| !identity.valid())
        || trigger_observation_identity.is_some() != !captures.is_empty()
        || (trigger_observation_identity.is_none() && binding_identity.is_none())
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "activation requires a semantic trigger and either an observation with captures or an absent-trigger scope",
            Usage::default(),
        ));
    }
    if captures.len() > effective.max_capture_bindings {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "activation capture set exceeds the effective binding bound",
            Usage {
                capture_bindings: captures.len(),
                ..Usage::default()
            },
        ));
    }
    validate_string(obligation_identity.as_str(), effective)?;
    if let Some(binding_identity) = binding_identity {
        validate_string(binding_identity.as_str(), effective)?;
    }
    validate_string(trigger_identity.as_str(), effective)?;
    if let Some(trigger_observation_identity) = trigger_observation_identity {
        validate_string(trigger_observation_identity.as_str(), effective)?;
    }
    for capture in captures {
        if !capture.identity.valid()
            || !capture.source_observation_identity.valid()
            || !capture.value_type.valid()
            || !capture.provenance_identity.valid()
        {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "capture identities must be explicit",
                Usage::default(),
            ));
        }
        for value in [
            capture.identity.as_str(),
            capture.source_observation_identity.as_str(),
            capture.value_type.as_str(),
            capture.canonical_value.as_str(),
            capture.provenance_identity.as_str(),
        ] {
            validate_string(value, effective)?;
        }
    }

    if captures
        .windows(2)
        .any(|pair| pair[0].identity >= pair[1].identity)
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "activation captures must be sorted and distinct by identity",
            Usage::default(),
        ));
    }
    let mut sorted = Vec::new();
    sorted.try_reserve(captures.len()).map_err(|_| {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "activation identity allocation failed",
            Usage::default(),
        )
    })?;
    sorted.extend(captures);
    let retained_binding_identity = if trigger_observation_identity.is_none() {
        Some(
            binding_identity
                .ok_or_else(|| {
                    Error::new(
                        ErrorCode::InvalidSelection,
                        "absent-trigger activation requires an exact binding identity",
                        Usage::default(),
                    )
                })?
                .as_str(),
        )
    } else {
        None
    };
    let preimage = ActivationPreimage {
        identity_version: if trigger_observation_identity.is_none() {
            "quire.observation.activation/v3"
        } else {
            "quire.observation.activation/v2"
        },
        obligation_identity: obligation_identity.as_str(),
        binding_identity: retained_binding_identity,
        trigger_identity: trigger_identity.as_str(),
        trigger_observation_identity: trigger_observation_identity.map(Identity::as_str),
        interval: IntervalPreimage {
            clock_identity: interval.clock_identity().as_str(),
            clock_revision: interval.clock_revision().as_str(),
            unit: interval.unit().as_str(),
            earliest: interval.earliest().to_string(),
            latest: interval.latest().to_string(),
        },
        captures: sorted
            .into_iter()
            .map(|capture| CapturePreimage {
                identity: capture.identity.as_str(),
                source_observation_identity: capture.source_observation_identity.as_str(),
                value_type: capture.value_type.as_str(),
                canonical_value: &capture.canonical_value,
                provenance_identity: capture.provenance_identity.as_str(),
            })
            .collect(),
    };
    sha256_jcs(&preimage, limits).map(|(identity, _)| identity)
}

/// Classifies whether matching progress proves silence through a deadline.
pub fn silence_coverage(
    deadline: &EventTimeInterval,
    progress: &EventTimeInterval,
    required_sources: &[Identity],
    covered_sources: &[Identity],
    limits: Limits,
) -> Result<SilenceCoverage> {
    deadline.relation_to(progress, limits)?;
    let effective = limits.effective();
    if required_sources.len() > effective.max_required_sources
        || covered_sources.len() > effective.max_required_sources
    {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "silence source set exceeds the effective required-source bound",
            Usage {
                required_sources: required_sources.len().max(covered_sources.len()),
                ..Usage::default()
            },
        ));
    }
    let required = exact_source_set(required_sources, effective)?;
    let covered = exact_source_set(covered_sources, effective)?;
    let mut reasons = BTreeSet::new();
    if progress.earliest() <= deadline.latest() {
        reasons.insert(SilenceIncompleteReason::ProgressNotDefinitelyBeyond);
    }
    if !required.is_subset(&covered) {
        reasons.insert(SilenceIncompleteReason::MissingRequiredSource);
    }
    if reasons.is_empty() {
        Ok(SilenceCoverage::Covered)
    } else {
        Ok(SilenceCoverage::Incomplete(IncompleteSilence { reasons }))
    }
}

/// Classifies event lateness from exact interval endpoints only.
pub fn classify_lateness(
    event: &EventTimeInterval,
    cutoff: &EventTimeInterval,
    limits: Limits,
) -> Result<Lateness> {
    event.relation_to(cutoff, limits)?;
    if event.latest() <= cutoff.earliest() {
        Ok(Lateness::DefinitelyTimely)
    } else if event.earliest() > cutoff.latest() {
        Ok(Lateness::DefinitelyLate)
    } else {
        Ok(Lateness::Uncertain)
    }
}

fn exact_source_set(sources: &[Identity], limits: Limits) -> Result<BTreeSet<&str>> {
    let mut exact = BTreeSet::new();
    for source in sources {
        if !source.valid() {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "source identities must be explicit",
                Usage::default(),
            ));
        }
        validate_string(source.as_str(), limits)?;
        if !exact.insert(source.as_str()) {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "source identities must be unique",
                Usage::default(),
            ));
        }
    }
    Ok(exact)
}

fn validate_string(value: &str, limits: Limits) -> Result<()> {
    if value.len() > limits.max_string_bytes {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "activation input exceeds the effective string bound",
            Usage {
                string_bytes: value.len(),
                ..Usage::default()
            },
        ));
    }
    Ok(())
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
