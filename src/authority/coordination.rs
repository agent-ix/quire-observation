// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Bounded streaming coordination over an immutable repair plan.
//!
//! This module owns scheduling, exact input handoff, prefix exposure, result
//! dataflow, lineage, and fail-closed outcome preservation. It does not parse
//! evaluator formulas or manufacture temporal/protocol truth.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use sha2::{Digest as _, Sha256};

use super::common::{
    digest_hex, to_bounded_json, Error, ErrorCode, Result, Usage as AuthorityUsage,
};
use super::partial::EventTimeInterval;
use super::repair::{Plan, Region, RetainedResultRef};
use super::{closure, BoundaryRef, OpenClosed};
use crate::{ClockRange, Digest, Identity};

/// Immutable owner maxima for evaluator coordination.
pub const OWNER_MAX: Limits = Limits {
    max_jobs: 10_000,
    max_inputs: 100_000,
    max_possible_orders: 300_000,
    max_work: 1_000_000,
    max_state_bytes: 8 * 1024 * 1024,
    max_output_bytes: 8 * 1024 * 1024,
};

/// Caller-lowerable bounds for one batch or incremental run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum affected jobs coordinated.
    pub max_jobs: usize,
    /// Maximum external plus result-source inputs.
    pub max_inputs: usize,
    /// Maximum admitted pairwise order possibilities across evaluator calls.
    pub max_possible_orders: usize,
    /// Maximum deterministic coordinator work units.
    pub max_work: usize,
    /// Maximum retained coordinator state bytes.
    pub max_state_bytes: usize,
    /// Maximum canonical replacement or run bytes.
    pub max_output_bytes: usize,
}

impl Limits {
    /// Returns the immutable coordinator maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamps every caller ceiling to its owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_jobs: min(self.max_jobs, OWNER_MAX.max_jobs),
            max_inputs: min(self.max_inputs, OWNER_MAX.max_inputs),
            max_possible_orders: min(self.max_possible_orders, OWNER_MAX.max_possible_orders),
            max_work: min(self.max_work, OWNER_MAX.max_work),
            max_state_bytes: min(self.max_state_bytes, OWNER_MAX.max_state_bytes),
            max_output_bytes: min(self.max_output_bytes, OWNER_MAX.max_output_bytes),
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        OWNER_MAX
    }
}

const fn min(left: usize, right: usize) -> usize {
    if left < right {
        left
    } else {
        right
    }
}

/// Exact successful coordinator resource use.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Jobs coordinated.
    pub jobs: usize,
    /// External plus result-source inputs retained.
    pub inputs: usize,
    /// Pairwise order possibilities passed to evaluators.
    pub possible_orders: usize,
    /// Evaluator calls made.
    pub evaluator_calls: usize,
    /// Deterministic coordinator work units.
    pub work: usize,
    /// Retained coordinator state bytes.
    pub state_bytes: usize,
    /// Canonical final run bytes.
    pub output_bytes: usize,
}

/// Immutable compiled package and evaluator-profile selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageProfile {
    package_identity: Identity,
    package_revision: Identity,
    package_digest: Digest,
    profile_identity: Identity,
    profile_revision: Identity,
    profile_digest: Digest,
}

impl PackageProfile {
    /// Constructs one exact package/profile selection.
    #[must_use]
    pub const fn new(
        package_identity: Identity,
        package_revision: Identity,
        package_digest: Digest,
        profile_identity: Identity,
        profile_revision: Identity,
        profile_digest: Digest,
    ) -> Self {
        Self {
            package_identity,
            package_revision,
            package_digest,
            profile_identity,
            profile_revision,
            profile_digest,
        }
    }

    /// Returns the immutable package identity.
    #[must_use]
    pub const fn package_identity(&self) -> &Identity {
        &self.package_identity
    }

    /// Returns the exact package revision.
    #[must_use]
    pub const fn package_revision(&self) -> &Identity {
        &self.package_revision
    }

    /// Returns the package-byte digest.
    #[must_use]
    pub const fn package_digest(&self) -> Digest {
        self.package_digest
    }

    /// Returns the selected evaluator-profile identity.
    #[must_use]
    pub const fn profile_identity(&self) -> &Identity {
        &self.profile_identity
    }

    /// Returns the exact evaluator-profile revision.
    #[must_use]
    pub const fn profile_revision(&self) -> &Identity {
        &self.profile_revision
    }

    /// Returns the evaluator-profile byte digest.
    #[must_use]
    pub const fn profile_digest(&self) -> Digest {
        self.profile_digest
    }
}

/// One exact external evaluator input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    identity: Identity,
    scope_identity: Identity,
    interval: EventTimeInterval,
    canonical_bytes: Vec<u8>,
}

impl Input {
    /// Constructs one immutable external evaluator input.
    #[must_use]
    pub const fn new(
        identity: Identity,
        scope_identity: Identity,
        interval: EventTimeInterval,
        canonical_bytes: Vec<u8>,
    ) -> Self {
        Self {
            identity,
            scope_identity,
            interval,
            canonical_bytes,
        }
    }

    /// Returns the exact input identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the exact authority scope.
    #[must_use]
    pub const fn scope_identity(&self) -> &Identity {
        &self.scope_identity
    }

    /// Returns the exact event-time uncertainty interval.
    #[must_use]
    pub const fn interval(&self) -> &EventTimeInterval {
        &self.interval
    }

    /// Returns the opaque canonical evaluator input bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    /// Returns the exact logical state bytes charged for retaining this input.
    pub fn retained_state_bytes(&self) -> Result<usize> {
        [
            self.identity.as_str().len(),
            self.scope_identity.as_str().len(),
            self.interval.clock_identity().as_str().len(),
            self.interval.clock_revision().as_str().len(),
            self.interval.unit().as_str().len(),
            std::mem::size_of::<i128>() * 2,
            self.canonical_bytes.len(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, value| {
            sum.checked_add(value).ok_or_else(|| {
                Error::new(
                    ErrorCode::ResourceIncomplete,
                    "external input state-byte count overflow",
                    AuthorityUsage::default(),
                )
            })
        })
    }
}

/// Declares one direct prior-result dependency resolved by the coordinator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultInput {
    source_identity: Identity,
    scope_identity: Identity,
    interval: EventTimeInterval,
}

/// Up-front finite bounds for external inputs that have not arrived yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputManifest {
    count: usize,
    state_bytes: usize,
}

impl InputManifest {
    /// Declares the exact future external input count and retained-state bytes.
    #[must_use]
    pub const fn new(count: usize, state_bytes: usize) -> Self {
        Self { count, state_bytes }
    }

    /// Derives an exact manifest for a complete in-memory input set.
    pub fn for_inputs(inputs: &[Input]) -> Result<Self> {
        let state_bytes = inputs.iter().try_fold(0usize, |sum, input| {
            sum.checked_add(input.retained_state_bytes()?)
                .ok_or_else(|| resource_error("external input manifest byte count overflow"))
        })?;
        Ok(Self::new(inputs.len(), state_bytes))
    }

    /// Returns the exact external input count.
    #[must_use]
    pub const fn count(self) -> usize {
        self.count
    }

    /// Returns the exact retained-state bytes for those inputs.
    #[must_use]
    pub const fn state_bytes(self) -> usize {
        self.state_bytes
    }
}

impl ResultInput {
    /// Constructs a result-source input without caller-supplied result bytes.
    #[must_use]
    pub const fn new(
        source_identity: Identity,
        scope_identity: Identity,
        interval: EventTimeInterval,
    ) -> Self {
        Self {
            source_identity,
            scope_identity,
            interval,
        }
    }

    /// Returns the exact source result identity.
    #[must_use]
    pub const fn source_identity(&self) -> &Identity {
        &self.source_identity
    }
}

/// Immutable job metadata; external inputs arrive separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Job {
    result_identity: Identity,
    selected: PackageProfile,
    input_unit: Identity,
    external_inputs: InputManifest,
    result_inputs: Vec<ResultInput>,
    closure: Option<closure::View>,
}

impl Job {
    /// Constructs one exact job specification.
    #[must_use]
    pub const fn new(
        result_identity: Identity,
        selected: PackageProfile,
        input_unit: Identity,
        external_inputs: InputManifest,
        result_inputs: Vec<ResultInput>,
        closure: Option<closure::View>,
    ) -> Self {
        Self {
            result_identity,
            selected,
            input_unit,
            external_inputs,
            result_inputs,
            closure,
        }
    }

    /// Returns the affected prior result identity.
    #[must_use]
    pub const fn result_identity(&self) -> &Identity {
        &self.result_identity
    }

    /// Returns the immutable package/profile selection.
    #[must_use]
    pub const fn selected(&self) -> &PackageProfile {
        &self.selected
    }

    /// Returns the exact selected interval/value unit.
    #[must_use]
    pub const fn input_unit(&self) -> &Identity {
        &self.input_unit
    }

    /// Returns the exact future external-input manifest.
    #[must_use]
    pub const fn external_inputs(&self) -> InputManifest {
        self.external_inputs
    }

    /// Returns direct result-source inputs resolved by schedule dataflow.
    #[must_use]
    pub fn result_inputs(&self) -> &[ResultInput] {
        &self.result_inputs
    }

    /// Returns the validated closure proof, when selected.
    #[must_use]
    pub const fn closure(&self) -> Option<&closure::View> {
        self.closure.as_ref()
    }
}

/// Complete affected-job metadata selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    jobs: Vec<Job>,
}

impl Selection {
    /// Constructs an explicit finite job selection.
    #[must_use]
    pub const fn new(jobs: Vec<Job>) -> Self {
        Self { jobs }
    }

    /// Returns selected evaluator jobs.
    #[must_use]
    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }
}

/// Complete external input set for one batch job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobInputs {
    result_identity: Identity,
    inputs: Vec<Input>,
}

impl JobInputs {
    /// Constructs the full external input set for one job.
    #[must_use]
    pub const fn new(result_identity: Identity, inputs: Vec<Input>) -> Self {
        Self {
            result_identity,
            inputs,
        }
    }

    /// Returns the selected result identity.
    #[must_use]
    pub const fn result_identity(&self) -> &Identity {
        &self.result_identity
    }

    /// Returns complete external inputs in presentation order.
    #[must_use]
    pub fn inputs(&self) -> &[Input] {
        &self.inputs
    }
}

/// Complete batch input selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchInputs {
    jobs: Vec<JobInputs>,
}

impl BatchInputs {
    /// Constructs one full batch input selection.
    #[must_use]
    pub const fn new(jobs: Vec<JobInputs>) -> Self {
        Self { jobs }
    }

    /// Returns every complete job input set.
    #[must_use]
    pub fn jobs(&self) -> &[JobInputs] {
        &self.jobs
    }
}

/// One admitted relation between instants from two uncertain intervals.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrderRelation {
    /// A selected instant from the left input may precede the right.
    Before,
    /// The inputs may select the same instant.
    Equal,
    /// A selected instant from the left input may follow the right.
    After,
}

/// One exact admissible pairwise order passed to the evaluator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntervalOrder {
    left_index: usize,
    right_index: usize,
    relation: OrderRelation,
}

impl IntervalOrder {
    /// Returns the left index in the canonical request input slice.
    #[must_use]
    pub const fn left_index(&self) -> usize {
        self.left_index
    }

    /// Returns the right index in the canonical request input slice.
    #[must_use]
    pub const fn right_index(&self) -> usize {
        self.right_index
    }

    /// Returns the admitted semantic relation.
    #[must_use]
    pub const fn relation(&self) -> OrderRelation {
        self.relation
    }
}

/// Whether decisive support is a witness, counterexample, or closure proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SupportKind {
    /// Positive decisive evidence.
    Witness,
    /// Negative decisive evidence.
    Counterexample,
    /// Matching validated closure authority.
    Closure,
}

/// Exact decision certificate returned by an evaluator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionSupport {
    kind: SupportKind,
    identity: Identity,
    evidence_identities: Vec<Identity>,
}

impl DecisionSupport {
    /// Constructs support plus the exact observed inputs used by the decision.
    #[must_use]
    pub const fn new(
        kind: SupportKind,
        identity: Identity,
        evidence_identities: Vec<Identity>,
    ) -> Self {
        Self {
            kind,
            identity,
            evidence_identities,
        }
    }

    /// Returns the closed support kind.
    #[must_use]
    pub const fn kind(&self) -> SupportKind {
        self.kind
    }

    /// Returns the decisive support identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the exact certified input identities.
    #[must_use]
    pub fn evidence_identities(&self) -> &[Identity] {
        &self.evidence_identities
    }
}

/// Closed settled result disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    /// The selected evaluator settled positively.
    Satisfied,
    /// The selected evaluator settled negatively.
    Violated,
}

/// Opaque selected-evaluator response for one observed prefix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluatorOutcome {
    /// More input or matching closure may settle the item.
    Pending(Identity),
    /// Required evidence is absent.
    Incomplete(Identity),
    /// Available evidence does not determine one result.
    Indeterminate(Identity),
    /// Selected package/profile lacks the capability.
    Unsupported(Identity),
    /// Selected evaluator refused the item.
    Refused(Identity),
    /// Selected evaluator failed the item.
    Failed(Identity),
    /// Selected evaluator exhausted its budget.
    Exhausted(Identity),
    /// A validated certificate settled the item.
    Decisive {
        /// Exact settled disposition.
        disposition: Disposition,
        /// Exact decision certificate.
        support: DecisionSupport,
        /// Opaque canonical result bytes.
        canonical_result: Vec<u8>,
    },
}

/// Borrowed resolved evaluator input. Result bytes come from plan dataflow.
#[derive(Clone, Copy, Debug)]
pub struct EvaluatorInput<'a> {
    identity: &'a Identity,
    scope_identity: &'a Identity,
    interval: &'a EventTimeInterval,
    canonical_bytes: &'a [u8],
}

impl<'a> EvaluatorInput<'a> {
    /// Returns the exact input identity.
    #[must_use]
    pub const fn identity(self) -> &'a Identity {
        self.identity
    }

    /// Returns the exact authority scope.
    #[must_use]
    pub const fn scope_identity(self) -> &'a Identity {
        self.scope_identity
    }

    /// Returns the exact event-time interval.
    #[must_use]
    pub const fn interval(self) -> &'a EventTimeInterval {
        self.interval
    }

    /// Returns exact external or coordinator-resolved result bytes.
    #[must_use]
    pub const fn canonical_bytes(self) -> &'a [u8] {
        self.canonical_bytes
    }
}

/// Borrowed exact request passed through the evaluator trait seam.
#[derive(Clone, Copy, Debug)]
pub struct Request<'a> {
    result_identity: &'a Identity,
    result_region: &'a Region,
    prior_result_bytes: &'a [u8],
    selected: &'a PackageProfile,
    inputs: &'a [EvaluatorInput<'a>],
    orders: &'a [IntervalOrder],
    closure: Option<&'a closure::View>,
    end_of_input: bool,
}

impl<'a> Request<'a> {
    /// Returns the affected prior result identity.
    #[must_use]
    pub const fn result_identity(self) -> &'a Identity {
        self.result_identity
    }

    /// Returns the plan-bound result authority region.
    #[must_use]
    pub const fn result_region(self) -> &'a Region {
        self.result_region
    }

    /// Returns byte-identical prior result bytes.
    #[must_use]
    pub const fn prior_result_bytes(self) -> &'a [u8] {
        self.prior_result_bytes
    }

    /// Returns the immutable package/profile selection.
    #[must_use]
    pub const fn selected(self) -> &'a PackageProfile {
        self.selected
    }

    /// Returns currently observed inputs in canonical identity order.
    #[must_use]
    pub const fn inputs(self) -> &'a [EvaluatorInput<'a>] {
        self.inputs
    }

    /// Returns every admissible pairwise order for the current prefix.
    #[must_use]
    pub const fn orders(self) -> &'a [IntervalOrder] {
        self.orders
    }

    /// Returns the exact validated closure proof, when selected.
    #[must_use]
    pub const fn closure(self) -> Option<&'a closure::View> {
        self.closure
    }

    /// Returns whether the caller closed this job's external input stream.
    #[must_use]
    pub const fn end_of_input(self) -> bool {
        self.end_of_input
    }
}

/// Mockable opaque evaluator seam.
pub trait Evaluator {
    /// Evaluates one exact observed prefix without ingestion metadata.
    ///
    /// Invocation is at-least-once: the coordinator can reject the returned
    /// contribution during validation or resource accounting, and retrying the
    /// same logical input invokes the evaluator again. Implementations must make
    /// repeated identical requests safe; usage counts every invocation.
    fn evaluate(&mut self, request: Request<'_>) -> EvaluatorOutcome;
}

/// Canonical replacement result and explicit supersession link.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplacementResult {
    identity: Identity,
    prior_identity: Identity,
    disposition: Disposition,
    support: DecisionSupport,
    evidence_digest: Digest,
    evaluator_digest: Digest,
    evaluator_bytes: Vec<u8>,
    bytes: Vec<u8>,
}

impl ReplacementResult {
    /// Returns the content-derived replacement identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the explicitly superseded prior result identity.
    #[must_use]
    pub const fn prior_identity(&self) -> &Identity {
        &self.prior_identity
    }

    /// Returns the settled disposition.
    #[must_use]
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// Returns the exact decision certificate.
    #[must_use]
    pub const fn support(&self) -> &DecisionSupport {
        &self.support
    }

    /// Returns the digest of only the certified observed evidence.
    #[must_use]
    pub const fn evidence_digest(&self) -> Digest {
        self.evidence_digest
    }

    /// Returns the digest of the exact opaque evaluator result bytes.
    #[must_use]
    pub const fn evaluator_digest(&self) -> Digest {
        self.evaluator_digest
    }

    /// Returns exact opaque evaluator result bytes.
    #[must_use]
    pub fn evaluator_bytes(&self) -> &[u8] {
        &self.evaluator_bytes
    }

    /// Returns canonical replacement-record bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Final item-local outcome. Only `Replaced` carries a promoted result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// A settled replacement result.
    Replaced(ReplacementResult),
    /// More selected input or closure may settle the item.
    Pending(Identity),
    /// Required evidence is absent, including an unavailable dependency.
    Incomplete(Identity),
    /// Evidence remains non-singleton.
    Indeterminate(Identity),
    /// Selected evaluator capability is absent.
    Unsupported(Identity),
    /// Selected evaluator refused the item.
    Refused(Identity),
    /// Selected evaluator failed the item.
    Failed(Identity),
    /// Selected evaluator exhausted its budget.
    Exhausted(Identity),
}

/// One result identity paired with its complete-input digest and final outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeRecord {
    result_identity: Identity,
    selected: PackageProfile,
    input_unit: Identity,
    selection_digest: Digest,
    outcome: Outcome,
}

impl OutcomeRecord {
    /// Returns the exact affected prior result identity.
    #[must_use]
    pub const fn result_identity(&self) -> &Identity {
        &self.result_identity
    }

    /// Returns the immutable package/profile selection.
    #[must_use]
    pub const fn selected(&self) -> &PackageProfile {
        &self.selected
    }

    /// Returns the exact selected interval/value unit.
    #[must_use]
    pub const fn input_unit(&self) -> &Identity {
        &self.input_unit
    }

    /// Returns the digest of the complete final input selection.
    #[must_use]
    pub const fn selection_digest(&self) -> Digest {
        self.selection_digest
    }

    /// Returns the item-local final outcome.
    #[must_use]
    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }
}

/// Non-promoted decisive prefix certificate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettledPrefix {
    disposition: Disposition,
    support: DecisionSupport,
    evidence_digest: Digest,
    evaluator_digest: Digest,
    evaluator_bytes: Vec<u8>,
}

impl SettledPrefix {
    /// Returns the settled evaluator disposition.
    #[must_use]
    pub const fn disposition(&self) -> Disposition {
        self.disposition
    }

    /// Returns the exact decision certificate.
    #[must_use]
    pub const fn support(&self) -> &DecisionSupport {
        &self.support
    }

    /// Returns the digest of certified observed evidence.
    #[must_use]
    pub const fn evidence_digest(&self) -> Digest {
        self.evidence_digest
    }

    /// Returns the digest of opaque evaluator result bytes.
    #[must_use]
    pub const fn evaluator_digest(&self) -> Digest {
        self.evaluator_digest
    }

    /// Returns exact opaque evaluator result bytes.
    #[must_use]
    pub fn evaluator_bytes(&self) -> &[u8] {
        &self.evaluator_bytes
    }
}

/// Immediately observable prefix state. `Settled` is not a promoted replacement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefixOutcome {
    /// A decisive certificate whose replacement remains internal until `finish`.
    Settled(SettledPrefix),
    /// More selected input or closure may settle the item.
    Pending(Identity),
    /// Required evidence is absent.
    Incomplete(Identity),
    /// Evidence remains non-singleton.
    Indeterminate(Identity),
    /// Selected evaluator capability is absent.
    Unsupported(Identity),
    /// Selected evaluator refused the item.
    Refused(Identity),
    /// Selected evaluator failed the item.
    Failed(Identity),
    /// Selected evaluator exhausted its budget.
    Exhausted(Identity),
}

/// One immediately observable incremental prefix result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefixEvent {
    result_identity: Identity,
    observed_inputs: usize,
    end_of_input: bool,
    outcome: PrefixOutcome,
}

impl PrefixEvent {
    /// Returns the affected result identity.
    #[must_use]
    pub const fn result_identity(&self) -> &Identity {
        &self.result_identity
    }

    /// Returns the number of observed external and result-source inputs.
    #[must_use]
    pub const fn observed_inputs(&self) -> usize {
        self.observed_inputs
    }

    /// Returns whether the job input stream was closed.
    #[must_use]
    pub const fn end_of_input(&self) -> bool {
        self.end_of_input
    }

    /// Returns the item-local prefix outcome.
    #[must_use]
    pub const fn outcome(&self) -> &PrefixOutcome {
        &self.outcome
    }
}

/// Byte-identical unaffected result retained by a completed run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetainedResult {
    identity: Identity,
    bytes: Vec<u8>,
}

impl RetainedResult {
    /// Returns the exact retained identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns byte-identical prior bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Complete semantic run. Prefix diagnostics are excluded from final bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Run {
    identity: Identity,
    bytes: Vec<u8>,
    outcomes: Vec<OutcomeRecord>,
    unaffected: Vec<RetainedResult>,
    prefixes: Vec<PrefixEvent>,
    usage: Usage,
}

impl Run {
    /// Returns the content-derived final run identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns canonical final semantic bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns final outcomes in repair schedule order.
    #[must_use]
    pub fn outcomes(&self) -> impl ExactSizeIterator<Item = &OutcomeRecord> {
        self.outcomes.iter()
    }

    /// Finds one final item-local outcome.
    #[must_use]
    pub fn outcome(&self, identity: &Identity) -> Option<&Outcome> {
        self.outcomes
            .iter()
            .find(|record| record.result_identity == *identity)
            .map(OutcomeRecord::outcome)
    }

    /// Returns byte-identical unaffected prior results.
    #[must_use]
    pub fn unaffected(&self) -> impl ExactSizeIterator<Item = &RetainedResult> {
        self.unaffected.iter()
    }

    /// Returns incremental prefix diagnostics; batch runs expose none.
    #[must_use]
    pub fn prefixes(&self) -> impl ExactSizeIterator<Item = &PrefixEvent> {
        self.prefixes.iter()
    }

    /// Returns exact successful coordinator resource use.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

struct PreparedResultInput<'a> {
    template: ResultInput,
    prior: RetainedResultRef<'a>,
    affected: bool,
}

struct PreparedJob<'a> {
    job: Job,
    prior: RetainedResultRef<'a>,
    result_inputs: Vec<PreparedResultInput<'a>>,
}

#[derive(Clone, Copy)]
struct Work {
    limits: Limits,
    usage: Usage,
}

impl Work {
    fn tick(&mut self) -> Result<()> {
        self.usage.work = self
            .usage
            .work
            .checked_add(1)
            .ok_or_else(|| self.exhausted())?;
        if self.usage.work > self.limits.max_work {
            return Err(self.exhausted());
        }
        Ok(())
    }

    fn ensure_state_capacity(&self, bytes: usize) -> Result<()> {
        let next = self
            .usage
            .state_bytes
            .checked_add(bytes)
            .ok_or_else(|| self.exhausted())?;
        if next > self.limits.max_state_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "coordinator state-byte limit exceeded",
                self.authority_usage(),
            ));
        }
        Ok(())
    }

    fn charge_state(&mut self, bytes: usize) -> Result<()> {
        self.ensure_state_capacity(bytes)?;
        self.usage.state_bytes += bytes;
        Ok(())
    }

    fn add_input(&mut self) -> Result<()> {
        self.reserve_inputs(1)
    }

    fn reserve_inputs(&mut self, count: usize) -> Result<()> {
        self.usage.inputs = self
            .usage
            .inputs
            .checked_add(count)
            .ok_or_else(|| self.exhausted())?;
        if self.usage.inputs > self.limits.max_inputs {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "coordinator input count exceeded",
                self.authority_usage(),
            ));
        }
        Ok(())
    }

    fn call(&mut self) -> Result<()> {
        self.tick()?;
        self.usage.evaluator_calls = self
            .usage
            .evaluator_calls
            .checked_add(1)
            .ok_or_else(|| self.exhausted())?;
        Ok(())
    }

    fn exhausted(&self) -> Error {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "coordinator work limit exceeded",
            self.authority_usage(),
        )
    }

    fn authority_usage(&self) -> AuthorityUsage {
        AuthorityUsage {
            visited_fields: self.usage.work,
            ..AuthorityUsage::default()
        }
    }
}

/// Stateful coordinator that exposes each prefix before later input is supplied.
pub struct IncrementalRun<'a, E: Evaluator> {
    plan: &'a Plan,
    evaluator: &'a mut E,
    prepared: Vec<PreparedJob<'a>>,
    current: usize,
    current_inputs: Vec<Input>,
    current_input_state: usize,
    terminal: Option<Outcome>,
    outcomes: Vec<OutcomeRecord>,
    outcomes_by_identity: BTreeMap<String, Outcome>,
    prefixes: Vec<PrefixEvent>,
    work: Work,
}

impl<'a, E: Evaluator> IncrementalRun<'a, E> {
    /// Starts a run without receiving any future external input.
    pub fn start(
        plan: &'a Plan,
        selection: &Selection,
        evaluator: &'a mut E,
        limits: Limits,
    ) -> Result<Self> {
        let effective = limits.effective();
        if selection.jobs.len() > effective.max_jobs {
            return Err(resource_error("coordinator job count exceeded"));
        }
        let mut work = Work {
            limits: effective,
            usage: Usage {
                jobs: selection.jobs.len(),
                ..Usage::default()
            },
        };
        let prepared = prepare(plan, selection, &mut work)?;
        Ok(Self {
            plan,
            evaluator,
            prepared,
            current: 0,
            current_inputs: Vec::new(),
            current_input_state: 0,
            terminal: None,
            outcomes: Vec::new(),
            outcomes_by_identity: BTreeMap::new(),
            prefixes: Vec::new(),
            work,
        })
    }

    /// Supplies one external input and returns its immediately observable prefix.
    pub fn push(&mut self, result_identity: &Identity, input: Input) -> Result<PrefixEvent> {
        let current = self.current_index(result_identity)?;
        {
            let prepared = &self.prepared[current];
            validate_external_input(
                &input,
                &prepared.job.input_unit,
                prepared.prior.region(),
                &self.work,
            )?;
            if prepared
                .result_inputs
                .iter()
                .any(|source| source.template.source_identity == input.identity)
                || self
                    .current_inputs
                    .iter()
                    .any(|existing| existing.identity == input.identity)
            {
                return Err(selection_error(
                    &self.work,
                    "job input identities must be distinct",
                ));
            }
        }
        let input_bytes = input_state_bytes(&input, &self.work)?;
        let next_state = self
            .current_input_state
            .checked_add(input_bytes)
            .ok_or_else(|| self.work.exhausted())?;
        if self.current_inputs.len() >= self.prepared[current].job.external_inputs.count
            || next_state > self.prepared[current].job.external_inputs.state_bytes
        {
            return Err(selection_error(
                &self.work,
                "external input exceeds its declared finite manifest",
            ));
        }
        self.current_inputs
            .try_reserve(1)
            .map_err(|_| self.work.exhausted())?;
        self.prefixes
            .try_reserve(1)
            .map_err(|_| self.work.exhausted())?;
        let prior_state_bytes = self.work.usage.state_bytes;
        let prior_terminal = self.terminal.clone();
        let prior_input_state = self.current_input_state;
        self.current_inputs.push(input);
        self.current_input_state = next_state;

        let result = (|| {
            let outcome = if let Some(terminal) = &self.terminal {
                terminal.clone()
            } else {
                let prepared = &self.prepared[current];
                let outcome = evaluate_job(
                    self.plan,
                    prepared,
                    &self.current_inputs,
                    &self.outcomes_by_identity,
                    false,
                    self.evaluator,
                    &mut self.work,
                )?;
                if terminal_outcome(&outcome) {
                    self.terminal = Some(outcome.clone());
                }
                outcome
            };
            self.record_prefix(false, outcome)
        })();
        match result {
            Ok(event) => Ok(event),
            Err(error) => {
                self.current_inputs.pop();
                self.current_input_state = prior_input_state;
                self.terminal = prior_terminal;
                self.work.usage.state_bytes = prior_state_bytes;
                Err(error)
            }
        }
    }

    /// Closes the current job and returns its final observable prefix.
    pub fn close(&mut self, result_identity: &Identity) -> Result<PrefixEvent> {
        let current = self.current_index(result_identity)?;
        let manifest = self.prepared[current].job.external_inputs;
        if self.current_inputs.len() != manifest.count
            || self.current_input_state != manifest.state_bytes
        {
            return Err(selection_error(
                &self.work,
                "closed job does not match its external input manifest",
            ));
        }
        let prior_state_bytes = self.work.usage.state_bytes;
        let staged = (|| {
            let outcome = if let Some(terminal) = &self.terminal {
                terminal.clone()
            } else {
                let prepared = &self.prepared[current];
                evaluate_job(
                    self.plan,
                    prepared,
                    &self.current_inputs,
                    &self.outcomes_by_identity,
                    true,
                    self.evaluator,
                    &mut self.work,
                )?
            };
            let (selection_digest, record_identity, selected) = {
                let prepared = &self.prepared[current];
                let digest = derive_selection_digest(
                    prepared,
                    &self.current_inputs,
                    &self.outcomes_by_identity,
                    &mut self.work,
                )?;
                charge_outcome_record(prepared, &outcome, &mut self.work)?;
                (
                    digest,
                    prepared.job.result_identity.clone(),
                    prepared.job.selected.clone(),
                )
            };
            let event = self.record_prefix(true, outcome.clone())?;
            Ok((outcome, selection_digest, record_identity, selected, event))
        })();
        let (outcome, selection_digest, record_identity, selected, event) = match staged {
            Ok(staged) => staged,
            Err(error) => {
                self.work.usage.state_bytes = prior_state_bytes;
                return Err(error);
            }
        };
        self.outcomes.push(OutcomeRecord {
            result_identity: record_identity.clone(),
            selected,
            input_unit: self.prepared[current].job.input_unit.clone(),
            selection_digest,
            outcome: outcome.clone(),
        });
        self.outcomes_by_identity
            .insert(record_identity.as_str().to_owned(), outcome);
        self.current += 1;
        self.current_inputs.clear();
        self.current_input_state = 0;
        self.terminal = None;
        Ok(event)
    }

    /// Finishes after every scheduled job has been explicitly closed.
    pub fn finish(self) -> Result<Run> {
        if self.current != self.prepared.len() {
            return Err(selection_error(
                &self.work,
                "incremental run cannot finish before every job is closed",
            ));
        }
        finish_run(self.plan, self.outcomes, self.prefixes, self.work)
    }

    fn current_index(&self, result_identity: &Identity) -> Result<usize> {
        let prepared = self.prepared.get(self.current).ok_or_else(|| {
            selection_error(&self.work, "incremental input follows the final job")
        })?;
        if prepared.job.result_identity != *result_identity {
            return Err(selection_error(
                &self.work,
                "incremental jobs must follow the repair schedule",
            ));
        }
        Ok(self.current)
    }

    fn record_prefix(&mut self, end_of_input: bool, outcome: Outcome) -> Result<PrefixEvent> {
        let prepared = self.prepared.get(self.current).ok_or_else(|| {
            selection_error(&self.work, "incremental prefix follows the final job")
        })?;
        let observed_inputs = self
            .current_inputs
            .len()
            .checked_add(prepared.result_inputs.len())
            .ok_or_else(|| self.work.exhausted())?;
        let event = PrefixEvent {
            result_identity: prepared.job.result_identity.clone(),
            observed_inputs,
            end_of_input,
            outcome: prefix_outcome(&outcome),
        };
        self.work
            .charge_state(prefix_state_bytes(&event, &self.work)?)?;
        self.prefixes
            .try_reserve(1)
            .map_err(|_| self.work.exhausted())?;
        self.prefixes.push(event.clone());
        Ok(event)
    }
}

/// Runs every affected job once with its complete external input set.
pub fn run_batch<E: Evaluator>(
    plan: &Plan,
    selection: &Selection,
    batch: &BatchInputs,
    evaluator: &mut E,
    limits: Limits,
) -> Result<Run> {
    let effective = limits.effective();
    if selection.jobs.len() > effective.max_jobs {
        return Err(resource_error("coordinator job count exceeded"));
    }
    let mut work = Work {
        limits: effective,
        usage: Usage {
            jobs: selection.jobs.len(),
            ..Usage::default()
        },
    };
    let prepared = prepare(plan, selection, &mut work)?;
    let external = validate_batch_inputs(batch, &prepared, &mut work)?;
    let mut outcomes = Vec::new();
    outcomes
        .try_reserve_exact(prepared.len())
        .map_err(|_| work.exhausted())?;
    let mut outcomes_by_identity = BTreeMap::new();
    for job in &prepared {
        work.tick()?;
        let inputs = external
            .get(job.job.result_identity.as_str())
            .copied()
            .ok_or_else(|| selection_error(&work, "batch job has no exact input set"))?;
        let outcome = evaluate_job(
            plan,
            job,
            inputs,
            &outcomes_by_identity,
            true,
            evaluator,
            &mut work,
        )?;
        let selection_digest =
            derive_selection_digest(job, inputs, &outcomes_by_identity, &mut work)?;
        charge_outcome_record(job, &outcome, &mut work)?;
        outcomes.push(OutcomeRecord {
            result_identity: job.job.result_identity.clone(),
            selected: job.job.selected.clone(),
            input_unit: job.job.input_unit.clone(),
            selection_digest,
            outcome: outcome.clone(),
        });
        outcomes_by_identity.insert(job.job.result_identity.as_str().to_owned(), outcome);
    }
    finish_run(plan, outcomes, Vec::new(), work)
}

fn prepare<'a>(
    plan: &'a Plan,
    selection: &Selection,
    work: &mut Work,
) -> Result<Vec<PreparedJob<'a>>> {
    if selection.jobs.len() != plan.affected().len() {
        return Err(selection_error(
            work,
            "coordination requires exactly one job per affected result",
        ));
    }
    let affected: BTreeMap<_, _> = plan
        .affected_prior()
        .map(|prior| (prior.identity().as_str(), prior))
        .collect();
    let unaffected: BTreeMap<_, _> = plan
        .unaffected()
        .map(|prior| (prior.identity().as_str(), prior))
        .collect();
    let mut jobs = BTreeMap::new();
    for job in &selection.jobs {
        work.tick()?;
        validate_job_identity(job, work)?;
        if jobs.insert(job.result_identity.as_str(), job).is_some() {
            return Err(selection_error(work, "job identities must be distinct"));
        }
    }

    let mut prepared = Vec::new();
    prepared
        .try_reserve_exact(selection.jobs.len())
        .map_err(|_| work.exhausted())?;
    for identity in plan.recomputation_order() {
        work.tick()?;
        let job = jobs
            .get(identity.as_str())
            .copied()
            .ok_or_else(|| selection_error(work, "affected result has no exact job"))?;
        let prior = affected
            .get(identity.as_str())
            .copied()
            .ok_or_else(|| selection_error(work, "affected result has no retained prior"))?;
        validate_closure(
            job.closure.as_ref(),
            plan.closure_identity(),
            prior.region(),
            work,
        )?;
        work.reserve_inputs(job.external_inputs.count)?;
        work.charge_state(job.external_inputs.state_bytes)?;

        let expected_sources = plan
            .dependencies()
            .filter(|edge| edge.dependent_identity() == identity)
            .filter_map(|edge| {
                affected
                    .get(edge.source_identity().as_str())
                    .copied()
                    .map(|source| (edge.source_identity().as_str(), (source, true)))
                    .or_else(|| {
                        unaffected
                            .get(edge.source_identity().as_str())
                            .copied()
                            .map(|source| (edge.source_identity().as_str(), (source, false)))
                    })
            })
            .collect::<BTreeMap<_, _>>();
        if expected_sources.len() != job.result_inputs.len() {
            return Err(selection_error(
                work,
                "job must declare every and only direct result-source dependency",
            ));
        }
        let mut seen = BTreeSet::new();
        let mut result_inputs = Vec::new();
        result_inputs
            .try_reserve_exact(job.result_inputs.len())
            .map_err(|_| work.exhausted())?;
        for template in &job.result_inputs {
            work.tick()?;
            if !seen.insert(template.source_identity.as_str()) {
                return Err(selection_error(
                    work,
                    "result-source input identities must be distinct",
                ));
            }
            let (source, is_affected) = expected_sources
                .get(template.source_identity.as_str())
                .copied()
                .ok_or_else(|| {
                    selection_error(work, "result-source input is not a direct dependency")
                })?;
            validate_input_authority(
                &template.source_identity,
                &template.scope_identity,
                &template.interval,
                &job.input_unit,
                prior.region(),
                work,
            )?;
            work.add_input()?;
            work.charge_state(result_input_state_bytes(template, work)?)?;
            result_inputs.push(PreparedResultInput {
                template: template.clone(),
                prior: source,
                affected: is_affected,
            });
        }
        prepared.push(PreparedJob {
            job: job.clone(),
            prior,
            result_inputs,
        });
    }
    Ok(prepared)
}

fn validate_job_identity(job: &Job, work: &mut Work) -> Result<()> {
    let selected = &job.selected;
    if [
        &job.result_identity,
        &selected.package_identity,
        &selected.package_revision,
        &selected.profile_identity,
        &selected.profile_revision,
        &job.input_unit,
    ]
    .iter()
    .any(|identity| !identity.valid())
    {
        return Err(selection_error(
            work,
            "coordinator job identities must be explicit",
        ));
    }
    work.charge_state(job_state_bytes(job, work)?)
}

fn validate_closure(
    closure: Option<&closure::View>,
    expected_identity: &Identity,
    region: &Region,
    work: &Work,
) -> Result<()> {
    let Some(closure) = closure else {
        return Ok(());
    };
    let payload = closure.payload();
    if closure.identity() != expected_identity
        || payload.state() != OpenClosed::Closed
        || payload.scope_identity() != region.scope_identity().as_str()
        || payload.clock_identity() != region.clock_identity().as_str()
        || payload.clock_revision() != region.clock_revision().as_str()
        || !boundary_covers(payload.boundary(), region.range())
    {
        return Err(selection_error(
            work,
            "closure proof must be closed and cover the exact result authority region",
        ));
    }
    Ok(())
}

fn boundary_covers(boundary: BoundaryRef<'_>, range: ClockRange) -> bool {
    match (boundary, range) {
        (
            BoundaryRef::EventPosition {
                lower,
                carrier_end_exclusive,
                ..
            },
            ClockRange::EventPosition {
                start,
                end_exclusive,
            },
        )
        | (
            BoundaryRef::FixedSample {
                lower,
                carrier_end_exclusive,
                ..
            },
            ClockRange::FixedSample {
                start,
                end_exclusive,
                ..
            },
        ) => {
            lower
                .parse::<u64>()
                .ok()
                .is_some_and(|value| value <= start)
                && carrier_end_exclusive
                    .parse::<u64>()
                    .ok()
                    .is_some_and(|value| value >= end_exclusive)
        }
        (
            BoundaryRef::TimestampedEvent {
                lower_nanos,
                carrier_end_exclusive_nanos,
                ..
            },
            ClockRange::Timestamp {
                start_nanos,
                end_nanos,
            },
        ) => {
            lower_nanos
                .parse::<i128>()
                .ok()
                .is_some_and(|value| value <= start_nanos)
                && carrier_end_exclusive_nanos
                    .parse::<i128>()
                    .ok()
                    .is_some_and(|value| value >= end_nanos)
        }
        _ => false,
    }
}

fn validate_batch_inputs<'a>(
    batch: &'a BatchInputs,
    prepared: &[PreparedJob<'_>],
    work: &mut Work,
) -> Result<BTreeMap<&'a str, &'a [Input]>> {
    if batch.jobs.len() != prepared.len() {
        return Err(selection_error(
            work,
            "batch requires exactly one external input set per job",
        ));
    }
    let by_identity: BTreeMap<_, _> = prepared
        .iter()
        .map(|job| (job.job.result_identity.as_str(), job))
        .collect();
    let mut selected = BTreeMap::new();
    for job_inputs in &batch.jobs {
        work.tick()?;
        let prepared = by_identity
            .get(job_inputs.result_identity.as_str())
            .copied()
            .ok_or_else(|| selection_error(work, "batch input names an unknown job"))?;
        let actual_state = job_inputs.inputs.iter().try_fold(0usize, |sum, input| {
            sum.checked_add(input_state_bytes(input, work)?)
                .ok_or_else(|| work.exhausted())
        })?;
        if job_inputs.inputs.len() != prepared.job.external_inputs.count
            || actual_state != prepared.job.external_inputs.state_bytes
        {
            return Err(selection_error(
                work,
                "batch input does not match its declared finite manifest",
            ));
        }
        if selected
            .insert(
                job_inputs.result_identity.as_str(),
                job_inputs.inputs.as_slice(),
            )
            .is_some()
        {
            return Err(selection_error(work, "batch input jobs must be distinct"));
        }
        let result_sources: BTreeSet<_> = prepared
            .result_inputs
            .iter()
            .map(|source| source.template.source_identity.as_str())
            .collect();
        let mut identities = BTreeSet::new();
        for input in &job_inputs.inputs {
            work.tick()?;
            validate_external_input(
                input,
                &prepared.job.input_unit,
                prepared.prior.region(),
                work,
            )?;
            if result_sources.contains(input.identity.as_str())
                || !identities.insert(input.identity.as_str())
            {
                return Err(selection_error(
                    work,
                    "external and result-source input identities must be distinct",
                ));
            }
        }
    }
    Ok(selected)
}

fn validate_external_input(
    input: &Input,
    expected_unit: &Identity,
    region: &Region,
    work: &Work,
) -> Result<()> {
    validate_input_authority(
        &input.identity,
        &input.scope_identity,
        &input.interval,
        expected_unit,
        region,
        work,
    )
}

fn validate_input_authority(
    identity: &Identity,
    scope_identity: &Identity,
    interval: &EventTimeInterval,
    expected_unit: &Identity,
    region: &Region,
    work: &Work,
) -> Result<()> {
    if !identity.valid()
        || !scope_identity.valid()
        || scope_identity != region.scope_identity()
        || interval.clock_identity() != region.clock_identity()
        || interval.clock_revision() != region.clock_revision()
        || interval.unit() != expected_unit
    {
        return Err(selection_error(
            work,
            "evaluator input must match the exact result scope and clock authority",
        ));
    }
    Ok(())
}

enum Resolution<'a> {
    Ready(Vec<EvaluatorInput<'a>>),
    Unavailable(&'a Identity),
}

fn resolve_inputs<'a>(
    prepared: &'a PreparedJob<'_>,
    external: &'a [Input],
    outcomes: &'a BTreeMap<String, Outcome>,
    work: &mut Work,
) -> Result<Resolution<'a>> {
    let count = external
        .len()
        .checked_add(prepared.result_inputs.len())
        .ok_or_else(|| work.exhausted())?;
    let mut resolved = Vec::new();
    resolved
        .try_reserve_exact(count)
        .map_err(|_| work.exhausted())?;
    for input in external {
        work.tick()?;
        resolved.push(EvaluatorInput {
            identity: &input.identity,
            scope_identity: &input.scope_identity,
            interval: &input.interval,
            canonical_bytes: &input.canonical_bytes,
        });
    }
    for source in &prepared.result_inputs {
        work.tick()?;
        let bytes = if source.affected {
            match outcomes.get(source.template.source_identity.as_str()) {
                Some(Outcome::Replaced(replacement)) => replacement.evaluator_bytes(),
                Some(_) => return Ok(Resolution::Unavailable(&source.template.source_identity)),
                None => {
                    return Err(selection_error(
                        work,
                        "dependency-safe schedule has no predecessor outcome",
                    ));
                }
            }
        } else {
            source.prior.bytes()
        };
        resolved.push(EvaluatorInput {
            identity: &source.template.source_identity,
            scope_identity: &source.template.scope_identity,
            interval: &source.template.interval,
            canonical_bytes: bytes,
        });
    }
    resolved.sort_by(|left, right| left.identity.cmp(right.identity));
    Ok(Resolution::Ready(resolved))
}

fn evaluate_job<E: Evaluator>(
    plan: &Plan,
    prepared: &PreparedJob<'_>,
    external: &[Input],
    outcomes: &BTreeMap<String, Outcome>,
    end_of_input: bool,
    evaluator: &mut E,
    work: &mut Work,
) -> Result<Outcome> {
    let inputs = match resolve_inputs(prepared, external, outcomes, work)? {
        Resolution::Ready(inputs) => inputs,
        Resolution::Unavailable(identity) => {
            return Ok(Outcome::Incomplete(Identity::new(format!(
                "dependency-unavailable:{}",
                identity.as_str()
            ))));
        }
    };
    let orders = derive_orders(&inputs, work)?;
    work.call()?;
    let response = evaluator.evaluate(Request {
        result_identity: &prepared.job.result_identity,
        result_region: prepared.prior.region(),
        prior_result_bytes: prepared.prior.bytes(),
        selected: &prepared.job.selected,
        inputs: &inputs,
        orders: &orders,
        closure: prepared.job.closure.as_ref(),
        end_of_input,
    });
    convert_outcome(plan, prepared, &inputs, response, work)
}

fn derive_orders(inputs: &[EvaluatorInput<'_>], work: &mut Work) -> Result<Vec<IntervalOrder>> {
    let mut orders = Vec::new();
    for right in 0..inputs.len() {
        for left in 0..right {
            work.tick()?;
            let possibilities = inputs[left]
                .interval
                .relation_to(inputs[right].interval, super::Limits::owner_max())?;
            for (admitted, relation) in [
                (possibilities.before(), OrderRelation::Before),
                (possibilities.equal(), OrderRelation::Equal),
                (possibilities.after(), OrderRelation::After),
            ] {
                if admitted {
                    let next = work
                        .usage
                        .possible_orders
                        .checked_add(1)
                        .ok_or_else(|| work.exhausted())?;
                    if next > work.limits.max_possible_orders {
                        return Err(Error::new(
                            ErrorCode::ResourceIncomplete,
                            "possible-order limit exceeded",
                            work.authority_usage(),
                        ));
                    }
                    work.tick()?;
                    orders.try_reserve(1).map_err(|_| work.exhausted())?;
                    orders.push(IntervalOrder {
                        left_index: left,
                        right_index: right,
                        relation,
                    });
                    work.usage.possible_orders = next;
                }
            }
        }
    }
    Ok(orders)
}

fn convert_outcome(
    plan: &Plan,
    prepared: &PreparedJob<'_>,
    inputs: &[EvaluatorInput<'_>],
    response: EvaluatorOutcome,
    work: &mut Work,
) -> Result<Outcome> {
    Ok(match response {
        EvaluatorOutcome::Pending(reason) => Outcome::Pending(validate_reason(reason, work)?),
        EvaluatorOutcome::Incomplete(reason) => Outcome::Incomplete(validate_reason(reason, work)?),
        EvaluatorOutcome::Indeterminate(reason) => {
            Outcome::Indeterminate(validate_reason(reason, work)?)
        }
        EvaluatorOutcome::Unsupported(reason) => {
            Outcome::Unsupported(validate_reason(reason, work)?)
        }
        EvaluatorOutcome::Refused(reason) => Outcome::Refused(validate_reason(reason, work)?),
        EvaluatorOutcome::Failed(reason) => Outcome::Failed(validate_reason(reason, work)?),
        EvaluatorOutcome::Exhausted(reason) => Outcome::Exhausted(validate_reason(reason, work)?),
        EvaluatorOutcome::Decisive {
            disposition,
            support,
            canonical_result,
        } => {
            let evidence_digest =
                validate_support_and_digest(prepared, inputs, disposition, &support, work)?;
            work.ensure_state_capacity(canonical_result.len())?;
            let replacement = build_replacement(
                plan,
                prepared,
                disposition,
                support,
                evidence_digest,
                canonical_result,
                work.limits.max_output_bytes,
            )?;
            work.charge_state(replacement_state_bytes(&replacement, work)?)?;
            Outcome::Replaced(replacement)
        }
    })
}

fn validate_reason(reason: Identity, work: &mut Work) -> Result<Identity> {
    if !reason.valid() {
        return Err(selection_error(
            work,
            "evaluator outcome reason must be explicit",
        ));
    }
    work.charge_state(reason.as_str().len())?;
    Ok(reason)
}

fn validate_support_and_digest(
    prepared: &PreparedJob<'_>,
    inputs: &[EvaluatorInput<'_>],
    disposition: Disposition,
    support: &DecisionSupport,
    work: &mut Work,
) -> Result<Digest> {
    if !support.identity.valid()
        || (support.kind != SupportKind::Closure && support.evidence_identities.is_empty())
    {
        return Err(support_error(
            work,
            "decisive support and certified evidence must be explicit",
        ));
    }
    let mut evidence = BTreeMap::new();
    for identity in &support.evidence_identities {
        work.tick()?;
        if !identity.valid() || evidence.contains_key(identity.as_str()) {
            return Err(support_error(
                work,
                "certified evidence identities must be explicit and distinct",
            ));
        }
        let input = inputs
            .iter()
            .find(|input| input.identity == identity)
            .copied()
            .ok_or_else(|| support_error(work, "decision certificate names an unobserved input"))?;
        evidence.insert(identity.as_str(), input);
    }
    let valid = match support.kind {
        SupportKind::Witness => {
            disposition == Disposition::Satisfied
                && evidence.contains_key(support.identity.as_str())
        }
        SupportKind::Counterexample => {
            disposition == Disposition::Violated && evidence.contains_key(support.identity.as_str())
        }
        SupportKind::Closure => prepared
            .job
            .closure
            .as_ref()
            .is_some_and(|closure| closure.identity() == &support.identity),
    };
    if !valid {
        return Err(support_error(
            work,
            "decision support does not match disposition, evidence, or closure",
        ));
    }

    let mut hasher = Sha256::new();
    hash_field(&mut hasher, b"quire.observation.decision-evidence/v1", work)?;
    for input in evidence.values() {
        hash_input(&mut hasher, *input, work)?;
    }
    if support.kind == SupportKind::Closure {
        let closure =
            prepared.job.closure.as_ref().ok_or_else(|| {
                support_error(work, "closure support has no validated closure proof")
            })?;
        hash_field(&mut hasher, closure.identity().as_str().as_bytes(), work)?;
        hash_field(&mut hasher, closure.bytes(), work)?;
    }
    Ok(Digest::new(hasher.finalize().into()))
}

fn derive_selection_digest(
    prepared: &PreparedJob<'_>,
    external: &[Input],
    outcomes: &BTreeMap<String, Outcome>,
    work: &mut Work,
) -> Result<Digest> {
    let mut hasher = Sha256::new();
    hash_field(
        &mut hasher,
        b"quire.observation.complete-selection/v1",
        work,
    )?;
    let mut external_by_identity: BTreeMap<_, _> = external
        .iter()
        .map(|input| (input.identity.as_str(), input))
        .collect();
    for source in &prepared.result_inputs {
        work.tick()?;
        if external_by_identity
            .remove(source.template.source_identity.as_str())
            .is_some()
        {
            return Err(selection_error(
                work,
                "external input shadows a result-source input",
            ));
        }
    }
    enum Selected<'a> {
        External(&'a Input),
        Result(&'a PreparedResultInput<'a>),
    }
    let mut selected = BTreeMap::new();
    for input in external_by_identity.values() {
        selected.insert(input.identity.as_str(), Selected::External(input));
    }
    for source in &prepared.result_inputs {
        selected.insert(
            source.template.source_identity.as_str(),
            Selected::Result(source),
        );
    }
    for entry in selected.values() {
        match entry {
            Selected::External(input) => hash_owned_input(&mut hasher, input, work)?,
            Selected::Result(source) => {
                hash_field(
                    &mut hasher,
                    source.template.source_identity.as_str().as_bytes(),
                    work,
                )?;
                hash_field(
                    &mut hasher,
                    source.template.scope_identity.as_str().as_bytes(),
                    work,
                )?;
                hash_interval(&mut hasher, &source.template.interval, work)?;
                if source.affected {
                    let outcome = outcomes
                        .get(source.template.source_identity.as_str())
                        .ok_or_else(|| selection_error(work, "result-source outcome is absent"))?;
                    match outcome {
                        Outcome::Replaced(replacement) => {
                            hash_field(&mut hasher, replacement.evaluator_bytes(), work)?;
                        }
                        other => hash_field(&mut hasher, outcome_tag(other), work)?,
                    }
                } else {
                    hash_field(&mut hasher, source.prior.bytes(), work)?;
                }
            }
        }
    }
    Ok(Digest::new(hasher.finalize().into()))
}

fn outcome_tag(outcome: &Outcome) -> &'static [u8] {
    match outcome {
        Outcome::Replaced(_) => b"replaced",
        Outcome::Pending(_) => b"pending",
        Outcome::Incomplete(_) => b"incomplete",
        Outcome::Indeterminate(_) => b"indeterminate",
        Outcome::Unsupported(_) => b"unsupported",
        Outcome::Refused(_) => b"refused",
        Outcome::Failed(_) => b"failed",
        Outcome::Exhausted(_) => b"exhausted",
    }
}

fn hash_owned_input(hasher: &mut Sha256, input: &Input, work: &mut Work) -> Result<()> {
    hash_field(hasher, input.identity.as_str().as_bytes(), work)?;
    hash_field(hasher, input.scope_identity.as_str().as_bytes(), work)?;
    hash_interval(hasher, &input.interval, work)?;
    hash_field(hasher, &input.canonical_bytes, work)
}

fn hash_input(hasher: &mut Sha256, input: EvaluatorInput<'_>, work: &mut Work) -> Result<()> {
    hash_field(hasher, input.identity.as_str().as_bytes(), work)?;
    hash_field(hasher, input.scope_identity.as_str().as_bytes(), work)?;
    hash_interval(hasher, input.interval, work)?;
    hash_field(hasher, input.canonical_bytes, work)
}

fn hash_interval(hasher: &mut Sha256, interval: &EventTimeInterval, work: &mut Work) -> Result<()> {
    hash_field(hasher, interval.clock_identity().as_str().as_bytes(), work)?;
    hash_field(hasher, interval.clock_revision().as_str().as_bytes(), work)?;
    hash_field(hasher, interval.unit().as_str().as_bytes(), work)?;
    hash_field(hasher, &interval.earliest().to_be_bytes(), work)?;
    hash_field(hasher, &interval.latest().to_be_bytes(), work)
}

fn hash_field(hasher: &mut Sha256, bytes: &[u8], work: &mut Work) -> Result<()> {
    work.tick()?;
    let length = u64::try_from(bytes.len()).map_err(|_| work.exhausted())?;
    hasher.update(length.to_be_bytes());
    hasher.update(bytes);
    Ok(())
}

fn build_replacement(
    plan: &Plan,
    prepared: &PreparedJob<'_>,
    disposition: Disposition,
    support: DecisionSupport,
    evidence_digest: Digest,
    evaluator_bytes: Vec<u8>,
    output_limit: usize,
) -> Result<ReplacementResult> {
    let evaluator_digest = Digest::new(Sha256::digest(&evaluator_bytes).into());
    let wire = ReplacementWire {
        contract: "quire.observation.repair-result/v1",
        plan_identity: plan.identity().as_str(),
        prior_identity: prepared.prior.identity().as_str(),
        prior_digest: format!("sha256:{:x}", Sha256::digest(prepared.prior.bytes())),
        package_identity: prepared.job.selected.package_identity.as_str(),
        package_revision: prepared.job.selected.package_revision.as_str(),
        package_digest: digest_hex(&prepared.job.selected.package_digest),
        profile_identity: prepared.job.selected.profile_identity.as_str(),
        profile_revision: prepared.job.selected.profile_revision.as_str(),
        profile_digest: digest_hex(&prepared.job.selected.profile_digest),
        input_unit: prepared.job.input_unit.as_str(),
        disposition,
        support_kind: support.kind,
        support_identity: support.identity.as_str(),
        evidence_digest: digest_hex(&evidence_digest),
        evaluator_digest: digest_hex(&evaluator_digest),
    };
    let bytes = to_bounded_json(&wire, output_limit)?;
    Ok(ReplacementResult {
        identity: content_identity(&bytes),
        prior_identity: prepared.prior.identity().clone(),
        disposition,
        support,
        evidence_digest,
        evaluator_digest,
        evaluator_bytes,
        bytes,
    })
}

fn finish_run(
    plan: &Plan,
    outcomes: Vec<OutcomeRecord>,
    prefixes: Vec<PrefixEvent>,
    mut work: Work,
) -> Result<Run> {
    let mut unaffected = Vec::new();
    unaffected
        .try_reserve_exact(plan.unaffected().len())
        .map_err(|_| work.exhausted())?;
    for retained in plan.unaffected() {
        work.tick()?;
        work.charge_state(
            retained
                .identity()
                .as_str()
                .len()
                .checked_add(retained.bytes().len())
                .ok_or_else(|| work.exhausted())?,
        )?;
        unaffected.push(RetainedResult {
            identity: retained.identity().clone(),
            bytes: retained.bytes().to_vec(),
        });
    }
    let bytes = encode_run(plan, &outcomes, &unaffected, work.limits.max_output_bytes)?;
    work.usage.output_bytes = bytes.len();
    Ok(Run {
        identity: content_identity(&bytes),
        bytes,
        outcomes,
        unaffected,
        prefixes,
        usage: work.usage,
    })
}

#[derive(Serialize)]
struct ReplacementWire<'a> {
    contract: &'static str,
    plan_identity: &'a str,
    prior_identity: &'a str,
    prior_digest: String,
    package_identity: &'a str,
    package_revision: &'a str,
    package_digest: String,
    profile_identity: &'a str,
    profile_revision: &'a str,
    profile_digest: String,
    input_unit: &'a str,
    disposition: Disposition,
    support_kind: SupportKind,
    support_identity: &'a str,
    evidence_digest: String,
    evaluator_digest: String,
}

#[derive(Serialize)]
struct RunWire<'a> {
    contract: &'static str,
    plan_identity: &'a str,
    outcomes: Vec<OutcomeWire<'a>>,
    unaffected: Vec<RetainedWire<'a>>,
}

#[derive(Serialize)]
struct OutcomeWire<'a> {
    result_identity: &'a str,
    selected: PackageProfileWire<'a>,
    input_unit: &'a str,
    selection_digest: String,
    #[serde(flatten)]
    outcome: OutcomeStateWire<'a>,
}

#[derive(Serialize)]
struct PackageProfileWire<'a> {
    package_identity: &'a str,
    package_revision: &'a str,
    package_digest: String,
    profile_identity: &'a str,
    profile_revision: &'a str,
    profile_digest: String,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
enum OutcomeStateWire<'a> {
    Replaced {
        replacement_identity: &'a str,
        replacement_digest: String,
    },
    Pending {
        reason: &'a str,
    },
    Incomplete {
        reason: &'a str,
    },
    Indeterminate {
        reason: &'a str,
    },
    Unsupported {
        reason: &'a str,
    },
    Refused {
        reason: &'a str,
    },
    Failed {
        reason: &'a str,
    },
    Exhausted {
        reason: &'a str,
    },
}

#[derive(Serialize)]
struct RetainedWire<'a> {
    identity: &'a str,
    digest: String,
    bytes: u64,
}

fn encode_run(
    plan: &Plan,
    outcomes: &[OutcomeRecord],
    unaffected: &[RetainedResult],
    output_limit: usize,
) -> Result<Vec<u8>> {
    let outcome_wires = outcomes.iter().map(outcome_wire).collect::<Vec<_>>();
    let unaffected_wires = unaffected
        .iter()
        .map(|retained| {
            Ok(RetainedWire {
                identity: retained.identity.as_str(),
                digest: format!("sha256:{:x}", Sha256::digest(&retained.bytes)),
                bytes: u64::try_from(retained.bytes.len()).map_err(|_| {
                    Error::new(
                        ErrorCode::ResourceIncomplete,
                        "retained result byte count is not wire-representable",
                        AuthorityUsage::default(),
                    )
                })?,
            })
        })
        .collect::<Result<_>>()?;
    to_bounded_json(
        &RunWire {
            contract: "quire.observation.repair-run/v1",
            plan_identity: plan.identity().as_str(),
            outcomes: outcome_wires,
            unaffected: unaffected_wires,
        },
        output_limit,
    )
}

fn outcome_wire(record: &OutcomeRecord) -> OutcomeWire<'_> {
    let selected = &record.selected;
    let outcome = match &record.outcome {
        Outcome::Replaced(replacement) => OutcomeStateWire::Replaced {
            replacement_identity: replacement.identity.as_str(),
            replacement_digest: format!("sha256:{:x}", Sha256::digest(&replacement.bytes)),
        },
        Outcome::Pending(reason) => OutcomeStateWire::Pending {
            reason: reason.as_str(),
        },
        Outcome::Incomplete(reason) => OutcomeStateWire::Incomplete {
            reason: reason.as_str(),
        },
        Outcome::Indeterminate(reason) => OutcomeStateWire::Indeterminate {
            reason: reason.as_str(),
        },
        Outcome::Unsupported(reason) => OutcomeStateWire::Unsupported {
            reason: reason.as_str(),
        },
        Outcome::Refused(reason) => OutcomeStateWire::Refused {
            reason: reason.as_str(),
        },
        Outcome::Failed(reason) => OutcomeStateWire::Failed {
            reason: reason.as_str(),
        },
        Outcome::Exhausted(reason) => OutcomeStateWire::Exhausted {
            reason: reason.as_str(),
        },
    };
    OutcomeWire {
        result_identity: record.result_identity.as_str(),
        selected: PackageProfileWire {
            package_identity: selected.package_identity.as_str(),
            package_revision: selected.package_revision.as_str(),
            package_digest: digest_hex(&selected.package_digest),
            profile_identity: selected.profile_identity.as_str(),
            profile_revision: selected.profile_revision.as_str(),
            profile_digest: digest_hex(&selected.profile_digest),
        },
        input_unit: record.input_unit.as_str(),
        selection_digest: digest_hex(&record.selection_digest),
        outcome,
    }
}

fn terminal_outcome(outcome: &Outcome) -> bool {
    matches!(
        outcome,
        Outcome::Replaced(_)
            | Outcome::Unsupported(_)
            | Outcome::Refused(_)
            | Outcome::Failed(_)
            | Outcome::Exhausted(_)
    )
}

fn prefix_outcome(outcome: &Outcome) -> PrefixOutcome {
    match outcome {
        Outcome::Replaced(replacement) => PrefixOutcome::Settled(SettledPrefix {
            disposition: replacement.disposition,
            support: replacement.support.clone(),
            evidence_digest: replacement.evidence_digest,
            evaluator_digest: replacement.evaluator_digest,
            evaluator_bytes: replacement.evaluator_bytes.clone(),
        }),
        Outcome::Pending(reason) => PrefixOutcome::Pending(reason.clone()),
        Outcome::Incomplete(reason) => PrefixOutcome::Incomplete(reason.clone()),
        Outcome::Indeterminate(reason) => PrefixOutcome::Indeterminate(reason.clone()),
        Outcome::Unsupported(reason) => PrefixOutcome::Unsupported(reason.clone()),
        Outcome::Refused(reason) => PrefixOutcome::Refused(reason.clone()),
        Outcome::Failed(reason) => PrefixOutcome::Failed(reason.clone()),
        Outcome::Exhausted(reason) => PrefixOutcome::Exhausted(reason.clone()),
    }
}

fn charge_outcome_record(
    prepared: &PreparedJob<'_>,
    outcome: &Outcome,
    work: &mut Work,
) -> Result<()> {
    work.charge_state(checked_state_sum(
        [
            prepared.job.result_identity.as_str().len(),
            package_profile_state_bytes(&prepared.job.selected, work)?,
            prepared.job.input_unit.as_str().len(),
            std::mem::size_of::<Digest>(),
            outcome_state_bytes(outcome, work)?,
        ],
        work,
    )?)
}

fn job_state_bytes(job: &Job, work: &Work) -> Result<usize> {
    let result_input_bytes = job.result_inputs.iter().try_fold(0usize, |total, input| {
        total
            .checked_add(result_input_state_bytes(input, work)?)
            .ok_or_else(|| work.exhausted())
    })?;
    checked_state_sum(
        [
            job.result_identity.as_str().len(),
            package_profile_state_bytes(&job.selected, work)?,
            job.input_unit.as_str().len(),
            job.closure
                .as_ref()
                .map_or(0, |closure| closure.bytes().len()),
            std::mem::size_of::<InputManifest>(),
            result_input_bytes,
        ],
        work,
    )
}

fn package_profile_state_bytes(selected: &PackageProfile, work: &Work) -> Result<usize> {
    checked_state_sum(
        [
            selected.package_identity.as_str().len(),
            selected.package_revision.as_str().len(),
            std::mem::size_of::<Digest>(),
            selected.profile_identity.as_str().len(),
            selected.profile_revision.as_str().len(),
            std::mem::size_of::<Digest>(),
        ],
        work,
    )
}

fn input_state_bytes(input: &Input, work: &Work) -> Result<usize> {
    input.retained_state_bytes().map_err(|_| work.exhausted())
}

fn result_input_state_bytes(input: &ResultInput, work: &Work) -> Result<usize> {
    checked_state_sum(
        [
            input.source_identity.as_str().len(),
            input.scope_identity.as_str().len(),
            interval_state_bytes(&input.interval, work)?,
        ],
        work,
    )
}

fn interval_state_bytes(interval: &EventTimeInterval, work: &Work) -> Result<usize> {
    checked_state_sum(
        [
            interval.clock_identity().as_str().len(),
            interval.clock_revision().as_str().len(),
            interval.unit().as_str().len(),
            std::mem::size_of::<i128>() * 2,
        ],
        work,
    )
}

fn prefix_state_bytes(event: &PrefixEvent, work: &Work) -> Result<usize> {
    checked_state_sum(
        [
            event.result_identity.as_str().len(),
            std::mem::size_of::<usize>(),
            std::mem::size_of::<bool>(),
            prefix_outcome_state_bytes(&event.outcome, work)?,
        ],
        work,
    )
}

fn prefix_outcome_state_bytes(outcome: &PrefixOutcome, work: &Work) -> Result<usize> {
    match outcome {
        PrefixOutcome::Settled(settled) => {
            let evidence_identity_bytes =
                settled
                    .support
                    .evidence_identities
                    .iter()
                    .try_fold(0usize, |sum, identity| {
                        sum.checked_add(identity.as_str().len())
                            .ok_or_else(|| work.exhausted())
                    })?;
            checked_state_sum(
                [
                    settled.support.identity.as_str().len(),
                    evidence_identity_bytes,
                    std::mem::size_of::<Digest>() * 2,
                    settled.evaluator_bytes.len(),
                ],
                work,
            )
        }
        PrefixOutcome::Pending(reason)
        | PrefixOutcome::Incomplete(reason)
        | PrefixOutcome::Indeterminate(reason)
        | PrefixOutcome::Unsupported(reason)
        | PrefixOutcome::Refused(reason)
        | PrefixOutcome::Failed(reason)
        | PrefixOutcome::Exhausted(reason) => Ok(reason.as_str().len()),
    }
}

fn outcome_state_bytes(outcome: &Outcome, work: &Work) -> Result<usize> {
    match outcome {
        Outcome::Replaced(replacement) => replacement_state_bytes(replacement, work),
        Outcome::Pending(reason)
        | Outcome::Incomplete(reason)
        | Outcome::Indeterminate(reason)
        | Outcome::Unsupported(reason)
        | Outcome::Refused(reason)
        | Outcome::Failed(reason)
        | Outcome::Exhausted(reason) => Ok(reason.as_str().len()),
    }
}

fn replacement_state_bytes(replacement: &ReplacementResult, work: &Work) -> Result<usize> {
    let evidence_identity_bytes =
        replacement
            .support
            .evidence_identities
            .iter()
            .try_fold(0usize, |sum, identity| {
                sum.checked_add(identity.as_str().len())
                    .ok_or_else(|| work.exhausted())
            })?;
    checked_state_sum(
        [
            replacement.identity.as_str().len(),
            replacement.prior_identity.as_str().len(),
            replacement.support.identity.as_str().len(),
            evidence_identity_bytes,
            std::mem::size_of::<Digest>() * 2,
            replacement.bytes.len(),
            replacement.evaluator_bytes.len(),
        ],
        work,
    )
}

fn checked_state_sum(values: impl IntoIterator<Item = usize>, work: &Work) -> Result<usize> {
    values.into_iter().try_fold(0usize, |sum, value| {
        sum.checked_add(value).ok_or_else(|| work.exhausted())
    })
}

fn content_identity(bytes: &[u8]) -> Identity {
    Identity::new(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn selection_error(work: &Work, detail: &'static str) -> Error {
    Error::new(ErrorCode::InvalidSelection, detail, work.authority_usage())
}

fn support_error(work: &Work, detail: &'static str) -> Error {
    Error::new(ErrorCode::SupportMismatch, detail, work.authority_usage())
}

fn resource_error(detail: &'static str) -> Error {
    Error::new(
        ErrorCode::ResourceIncomplete,
        detail,
        AuthorityUsage::default(),
    )
}
