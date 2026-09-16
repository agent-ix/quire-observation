// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Bounded queries over one closed authority-qualified population.
//!
//! The executor joins members to observations only through a validated
//! completeness assertion, uses the validated position ledger as its sole
//! exposure order, and never interprets transport receipts as business effects.

use std::fmt;

use serde::{ser::SerializeSeq as _, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

use super::bundle::{self, ComponentRole, ConflictKind, EmbeddedPayloadRef};
use super::common::{to_bounded_json, Error, ErrorCode, Result, Usage as AuthorityUsage};
use super::observation::{RecordView, RelationshipRef, SubjectRef, ValueRef};
use super::{BoundaryRef, OpenClosed};
use crate::Identity;

/// Immutable owner maxima for one population query.
pub const OWNER_MAX: Limits = Limits {
    max_input_bytes: 8 * 1024 * 1024,
    max_members: 100_000,
    max_arithmetic_steps: 100_000,
    max_work: 1_000_000,
    max_state_bytes: 8 * 1024 * 1024,
    max_output_bytes: 8 * 1024 * 1024,
};

/// Caller-lowerable query bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum validated lineage and bundle input bytes.
    pub max_input_bytes: usize,
    /// Maximum required population members.
    pub max_members: usize,
    /// Maximum exact ordered arithmetic steps.
    pub max_arithmetic_steps: usize,
    /// Maximum deterministic query work units.
    pub max_work: usize,
    /// Maximum retained logical state bytes.
    pub max_state_bytes: usize,
    /// Maximum canonical evaluation bytes.
    pub max_output_bytes: usize,
}

impl Limits {
    /// Returns immutable query maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamps every caller bound to its owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_input_bytes: min(self.max_input_bytes, OWNER_MAX.max_input_bytes),
            max_members: min(self.max_members, OWNER_MAX.max_members),
            max_arithmetic_steps: min(self.max_arithmetic_steps, OWNER_MAX.max_arithmetic_steps),
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

/// Exact successful resource accounting.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Validated input bytes inspected.
    pub input_bytes: usize,
    /// Required population members inspected.
    pub members: usize,
    /// Admitted occurrences participating in duplicate handling.
    pub occurrences: usize,
    /// Exact arithmetic steps performed.
    pub arithmetic_steps: usize,
    /// Deterministic work units consumed.
    pub work: usize,
    /// Logical state bytes retained.
    pub state_bytes: usize,
    /// Canonical evaluation bytes emitted.
    pub output_bytes: usize,
}

/// Duplicate semantics applied after every occurrence is authority-validated.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DuplicatePolicy {
    /// Preserve only the first exposed occurrence of each exact effect key.
    EffectIdentityDeduplicating,
    /// Preserve every distinct admitted observation occurrence.
    OccurrencePreserving,
}

/// Endpoint that names the selected grouping root.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RootEndpoint {
    /// The authored relationship source is the root and target is the effect.
    Source,
    /// The authored relationship target is the root and source is the effect.
    Target,
}

/// Closed admitted member-predicate vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Predicate {
    /// Every authority-qualified member matches.
    All,
    /// Match one exact observation signal identity.
    Signal(Identity),
    /// Match one exact authority-qualified effect identity within the selected effect kind.
    EffectIdentity(Vec<u8>),
    /// Match observations carrying a present exact value.
    ValuePresent,
}

/// Boolean reduction selected for a filter query.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilterQuantifier {
    /// True when at least one admitted occurrence matches.
    Any,
    /// True when every admitted occurrence matches; true for an empty population.
    All,
}

/// Closed numeric representation vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NumericRepresentation {
    /// Canonical signed base-ten `i128` text with no plus sign or leading zeroes.
    SignedInteger,
}

/// One admitted query plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryPlan {
    /// Reduce predicate matches to one exact Boolean.
    Filter {
        /// Exact query-plan identity.
        identity: Identity,
        /// Admitted member predicate.
        predicate: Predicate,
        /// Exact Boolean reduction.
        quantifier: FilterQuantifier,
    },
    /// Count predicate-matching admitted occurrences.
    Count {
        /// Exact query-plan identity.
        identity: Identity,
        /// Admitted member predicate.
        predicate: Predicate,
    },
    /// Left-fold exact numeric values in position-ledger order.
    ExactSum {
        /// Exact query-plan identity.
        identity: Identity,
        /// Admitted member predicate.
        predicate: Predicate,
        /// Closed numeric representation.
        representation: NumericRepresentation,
        /// Exact value-type identity.
        value_type: Identity,
        /// Exact unit identity.
        unit: Identity,
        /// Exact additive-identity definition.
        zero_identity: Identity,
        /// Exact additive identity value.
        zero: i128,
        /// Inclusive result-domain minimum.
        minimum: i128,
        /// Inclusive result-domain maximum.
        maximum: i128,
    },
}

impl QueryPlan {
    fn identity(&self) -> &Identity {
        match self {
            Self::Filter { identity, .. }
            | Self::Count { identity, .. }
            | Self::ExactSum { identity, .. } => identity,
        }
    }

    fn predicate(&self) -> &Predicate {
        match self {
            Self::Filter { predicate, .. }
            | Self::Count { predicate, .. }
            | Self::ExactSum { predicate, .. } => predicate,
        }
    }
}

/// Complete exact authority and relationship selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    lineage_identity: Identity,
    bundle_identity: Identity,
    bundle_revision: u64,
    population_component_identity: Identity,
    position_component_identity: Identity,
    progress_component_identity: Identity,
    closure_component_identity: Identity,
    completeness_component_identity: Identity,
    capture_component_identity: Option<Identity>,
    activation_component_identity: Option<Identity>,
    population_identity: Identity,
    scope_identity: Identity,
    membership_rule_identity: Identity,
    relationship_declaration_identity: Identity,
    root_endpoint: RootEndpoint,
    root_subject_kind: Vec<u8>,
    grouping_key: Vec<u8>,
    effect_subject_kind: Vec<u8>,
    effect_signal_identity: Identity,
    duplicate_policy: DuplicatePolicy,
    plan: QueryPlan,
}

impl Selection {
    /// Constructs a complete selection; evaluation validates every axis exactly.
    // Each argument is an independently authority-bound query axis; collapsing
    // them into an unvalidated options bag would weaken the public boundary.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        lineage_identity: Identity,
        bundle_identity: Identity,
        bundle_revision: u64,
        population_component_identity: Identity,
        position_component_identity: Identity,
        progress_component_identity: Identity,
        closure_component_identity: Identity,
        completeness_component_identity: Identity,
        capture_component_identity: Option<Identity>,
        activation_component_identity: Option<Identity>,
        population_identity: Identity,
        scope_identity: Identity,
        membership_rule_identity: Identity,
        relationship_declaration_identity: Identity,
        root_endpoint: RootEndpoint,
        root_subject_kind: Vec<u8>,
        grouping_key: Vec<u8>,
        effect_subject_kind: Vec<u8>,
        effect_signal_identity: Identity,
        duplicate_policy: DuplicatePolicy,
        plan: QueryPlan,
    ) -> Self {
        Self {
            lineage_identity,
            bundle_identity,
            bundle_revision,
            population_component_identity,
            position_component_identity,
            progress_component_identity,
            closure_component_identity,
            completeness_component_identity,
            capture_component_identity,
            activation_component_identity,
            population_identity,
            scope_identity,
            membership_rule_identity,
            relationship_declaration_identity,
            root_endpoint,
            root_subject_kind,
            grouping_key,
            effect_subject_kind,
            effect_signal_identity,
            duplicate_policy,
            plan,
        }
    }
}

/// Machine-matchable reasons why authority is not definitive.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IncompleteReason {
    /// The selected bundle head has a known direct successor.
    StaleRevision,
    /// Required authority is missing.
    MissingAuthority,
    /// Progress is not closed.
    OpenProgress,
    /// Population closure is not closed.
    OpenClosure,
    /// Required member facts are incomplete.
    IncompleteFacts,
    /// Required member facts are contradicted.
    ContradictedFacts,
    /// Explicit bundle authority reports an unresolved correlation.
    UnresolvedCorrelation,
    /// Explicit bundle authority reports ambiguity.
    AmbiguousAuthority,
}

impl IncompleteReason {
    const ALL: [Self; 8] = [
        Self::StaleRevision,
        Self::MissingAuthority,
        Self::OpenProgress,
        Self::OpenClosure,
        Self::IncompleteFacts,
        Self::ContradictedFacts,
        Self::UnresolvedCorrelation,
        Self::AmbiguousAuthority,
    ];
    const COUNT: usize = Self::ALL.len();

    const fn index(self) -> usize {
        match self {
            Self::StaleRevision => 0,
            Self::MissingAuthority => 1,
            Self::OpenProgress => 2,
            Self::OpenClosure => 3,
            Self::IncompleteFacts => 4,
            Self::ContradictedFacts => 5,
            Self::UnresolvedCorrelation => 6,
            Self::AmbiguousAuthority => 7,
        }
    }
}

/// Incomplete outcome with a non-empty canonical reason set and no aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncompleteQuery {
    reasons: Vec<IncompleteReason>,
}

impl IncompleteQuery {
    /// Returns the non-empty sorted reason set.
    #[must_use]
    pub fn reasons(&self) -> &[IncompleteReason] {
        &self.reasons
    }
}

/// Machine-matchable query refusal reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefusalReason {
    /// A selected lineage, bundle, component, population, scope, or membership axis is foreign.
    ForeignAuthority,
    /// The query plan is malformed or names an unsupported selection.
    InvalidPlan,
    /// A member subject kind or grouping root is foreign.
    ForeignSubject,
    /// A required relationship is absent, reversed, or foreign.
    ForeignRelationship,
    /// A transport receipt or other signal was substituted for an effect.
    ReceiptAsEffect,
    /// An authority-declared member lies outside the selected half-open population window.
    OutsideWindow,
    /// The selected numeric representation or canonical value is invalid.
    InvalidRepresentation,
    /// A sum operand uses a foreign unit.
    WrongUnit,
    /// Exact checked arithmetic overflowed.
    ArithmeticOverflow,
    /// An ordered intermediate sum left the selected result domain.
    InvalidPrefix,
    /// An occurrence lacks one exact unambiguous exposure position.
    InvalidExposureOrder,
}

/// Refused outcome with no aggregate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RefusedQuery {
    reason: RefusalReason,
}

impl RefusedQuery {
    /// Returns the closed refusal reason.
    #[must_use]
    pub const fn reason(&self) -> RefusalReason {
        self.reason
    }
}

/// One canonical participating member in exposure order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participation {
    position: u64,
    member_identity: Identity,
    observation_identity: Identity,
    effect_kind: Vec<u8>,
    effect_identity: Vec<u8>,
    relationship_identity: Vec<u8>,
    predicate_matches: bool,
    included: bool,
}

impl Participation {
    /// Returns the zero-based exposure position.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Returns the exact population member identity.
    #[must_use]
    pub const fn member_identity(&self) -> &Identity {
        &self.member_identity
    }

    /// Returns the exact participating observation identity.
    #[must_use]
    pub const fn observation_identity(&self) -> &Identity {
        &self.observation_identity
    }

    /// Returns the exact authority-qualified effect kind bytes.
    #[must_use]
    pub fn effect_kind(&self) -> &[u8] {
        &self.effect_kind
    }

    /// Returns the exact authority-qualified effect identity bytes.
    #[must_use]
    pub fn effect_identity(&self) -> &[u8] {
        &self.effect_identity
    }

    /// Returns the exact authored causal-relationship identity bytes.
    #[must_use]
    pub fn relationship_identity(&self) -> &[u8] {
        &self.relationship_identity
    }

    /// Returns whether the predicate matched this occurrence.
    #[must_use]
    pub const fn predicate_matches(&self) -> bool {
        self.predicate_matches
    }

    /// Returns whether duplicate policy retained this occurrence for aggregation.
    #[must_use]
    pub const fn included(&self) -> bool {
        self.included
    }
}

/// Exact complete aggregate value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AggregateValue {
    /// Exact filter Boolean.
    Boolean(bool),
    /// Exact retained matching-occurrence count.
    Count(u64),
    /// Exact checked sum.
    Sum(i128),
}

/// Complete result; this is the only type that can carry an aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteResult {
    participation: Vec<Participation>,
    value: AggregateValue,
}

impl CompleteResult {
    /// Returns canonical exposure-ordered participation.
    pub fn participation(&self) -> impl ExactSizeIterator<Item = &Participation> {
        self.participation.iter()
    }

    /// Returns the exact aggregate value.
    #[must_use]
    pub const fn value(&self) -> &AggregateValue {
        &self.value
    }
}

/// Closed evaluation outcome; non-complete variants cannot carry an aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Complete definitive aggregate and participation.
    Complete(CompleteResult),
    /// Non-definitive authority with no aggregate.
    Incomplete(IncompleteQuery),
    /// Invalid or foreign input with no aggregate.
    Refused(RefusedQuery),
}

/// Canonical query evaluation artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    identity: Identity,
    bytes: Vec<u8>,
    outcome: Outcome,
    usage: Usage,
}

impl Evaluation {
    /// Returns the content-derived evaluation identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns canonical evaluation bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the typed complete, incomplete, or refused outcome.
    #[must_use]
    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    /// Returns exact successful resource use.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

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
            return Err(resource_error("query state-byte limit exceeded", self));
        }
        Ok(())
    }

    fn charge_state(&mut self, bytes: usize) -> Result<()> {
        self.ensure_state_capacity(bytes)?;
        self.usage.state_bytes += bytes;
        Ok(())
    }

    fn arithmetic_step(&mut self) -> Result<()> {
        self.tick()?;
        self.usage.arithmetic_steps = self
            .usage
            .arithmetic_steps
            .checked_add(1)
            .ok_or_else(|| self.exhausted())?;
        if self.usage.arithmetic_steps > self.limits.max_arithmetic_steps {
            return Err(resource_error("query arithmetic-step limit exceeded", self));
        }
        Ok(())
    }

    fn exhausted(&self) -> Error {
        resource_error("query work limit exceeded", self)
    }
}

struct Authority<'a> {
    population: &'a super::population::PopulationView,
    position: &'a super::position::PositionLedgerView,
    progress: &'a super::progress::ProgressView,
    closure: &'a super::closure::ClosureView,
    completeness: &'a super::completeness::CompletenessView,
    capture: Option<&'a super::capture::CaptureView>,
    activation: Option<&'a super::activation::ActivationView>,
}

#[derive(Clone)]
struct Joined<'a> {
    position: u64,
    member_identity: Identity,
    record: &'a RecordView,
    effect: SubjectRef<'a>,
    relationship_identity: &'a [u8],
    predicate_matches: bool,
    included: bool,
}

/// Evaluates a query against the exact bounded lineage head.
pub fn evaluate(
    lineage: &bundle::LineageView,
    selection: &Selection,
    limits: Limits,
) -> Result<Evaluation> {
    let effective = limits.effective();
    let input_bytes = lineage
        .document()
        .bytes()
        .len()
        .checked_add(lineage.head().bytes().len())
        .and_then(|bytes| bytes.checked_add(selection_input_bytes(selection)?))
        .ok_or_else(|| resource_error_without_work("query input-byte count overflowed"))?;
    if input_bytes > effective.max_input_bytes {
        return Err(resource_error_without_work(
            "query input-byte limit exceeded",
        ));
    }
    let mut work = Work {
        limits: effective,
        usage: Usage {
            input_bytes,
            ..Usage::default()
        },
    };
    let outcome = decide(lineage, selection, &mut work)?;
    let bytes = encode(lineage, selection, &outcome, work.usage, effective)?;
    work.usage.output_bytes = bytes.len();
    let identity = Identity::new(format!("sha256:{:x}", Sha256::digest(&bytes)));
    Ok(Evaluation {
        identity,
        bytes,
        outcome,
        usage: work.usage,
    })
}

fn selection_input_bytes(selection: &Selection) -> Option<usize> {
    let identity_bytes = [
        &selection.lineage_identity,
        &selection.bundle_identity,
        &selection.population_component_identity,
        &selection.position_component_identity,
        &selection.progress_component_identity,
        &selection.closure_component_identity,
        &selection.completeness_component_identity,
        &selection.population_identity,
        &selection.scope_identity,
        &selection.membership_rule_identity,
        &selection.relationship_declaration_identity,
        &selection.effect_signal_identity,
        selection.plan.identity(),
    ]
    .into_iter()
    .map(|identity| identity.as_str().len());
    let optional_component_bytes = [
        selection.capture_component_identity.as_ref(),
        selection.activation_component_identity.as_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|identity| identity.as_str().len());
    let axis_bytes = [
        selection.root_subject_kind.len(),
        selection.grouping_key.len(),
        selection.effect_subject_kind.len(),
        std::mem::size_of::<u64>(),
        2,
    ];
    identity_bytes
        .chain(optional_component_bytes)
        .chain(axis_bytes)
        .try_fold(0usize, usize::checked_add)
        .and_then(|bytes| bytes.checked_add(plan_input_bytes(&selection.plan)?))
}

fn plan_input_bytes(plan: &QueryPlan) -> Option<usize> {
    let (predicate, plan_specific) = match plan {
        QueryPlan::Filter { predicate, .. } => (predicate, [1, 0, 0, 0, 0, 0]),
        QueryPlan::Count { predicate, .. } => (predicate, [0, 0, 0, 0, 0, 0]),
        QueryPlan::ExactSum {
            predicate,
            value_type,
            unit,
            zero_identity,
            ..
        } => (
            predicate,
            [
                value_type.as_str().len(),
                unit.as_str().len(),
                zero_identity.as_str().len(),
                std::mem::size_of::<i128>() * 3,
                1,
                0,
            ],
        ),
    };
    let predicate_bytes = match predicate {
        Predicate::All | Predicate::ValuePresent => 1,
        Predicate::Signal(identity) => 1usize.checked_add(identity.as_str().len())?,
        Predicate::EffectIdentity(identity) => 1usize.checked_add(identity.len())?,
    };
    plan_specific
        .into_iter()
        .try_fold(predicate_bytes, usize::checked_add)
}

fn decide(
    lineage: &bundle::LineageView,
    selection: &Selection,
    work: &mut Work,
) -> Result<Outcome> {
    work.tick()?;
    if !valid_selection(selection) {
        return Ok(refused(RefusalReason::InvalidPlan));
    }
    if lineage.document().identity() != &selection.lineage_identity
        || lineage.head().identity() != &selection.bundle_identity
        || lineage.head().revision() != selection.bundle_revision
        || lineage.head().subject().population_identity != selection.population_identity
        || lineage.head().subject().scope_identity != selection.scope_identity
    {
        return Ok(refused(RefusalReason::ForeignAuthority));
    }
    if lineage.direct_children().len() != 0 {
        return incomplete([IncompleteReason::StaleRevision], work);
    }
    let head = lineage.head();
    if !components_match(head, selection, work)? {
        return Ok(refused(RefusalReason::ForeignAuthority));
    }
    let mut conflict_reasons = [false; IncompleteReason::COUNT];
    for conflict in head.payload().conflicts() {
        work.tick()?;
        let reason = match conflict.kind() {
            ConflictKind::Duplicate => IncompleteReason::AmbiguousAuthority,
            ConflictKind::Contradiction => IncompleteReason::ContradictedFacts,
            ConflictKind::UnresolvedCorrelation => IncompleteReason::UnresolvedCorrelation,
        };
        conflict_reasons[reason.index()] = true;
    }
    if conflict_reasons.iter().any(|present| *present) {
        return incomplete(reason_set(conflict_reasons), work);
    }
    let Some(authority) = authority(head, work)? else {
        return incomplete([IncompleteReason::MissingAuthority], work);
    };
    if !authority_axes_match(&authority, selection, work)? {
        return Ok(refused(RefusalReason::ForeignAuthority));
    }
    let mut incomplete_reasons = [false; IncompleteReason::COUNT];
    if authority.progress.state() != OpenClosed::Closed {
        incomplete_reasons[IncompleteReason::OpenProgress.index()] = true;
    }
    if authority.closure.state() != OpenClosed::Closed {
        incomplete_reasons[IncompleteReason::OpenClosure.index()] = true;
    }
    match authority.completeness.state() {
        super::completeness::State::Complete => {}
        super::completeness::State::Incomplete => {
            incomplete_reasons[IncompleteReason::IncompleteFacts.index()] = true;
        }
        super::completeness::State::Contradicted => {
            incomplete_reasons[IncompleteReason::ContradictedFacts.index()] = true;
        }
    }
    if incomplete_reasons.iter().any(|present| *present) {
        return incomplete(reason_set(incomplete_reasons), work);
    }
    let member_count = authority.population.required_members().len();
    if member_count > work.limits.max_members {
        return Err(resource_error("query member limit exceeded", work));
    }
    work.usage.members = member_count;
    if member_count != 0 && !business_authority_is_active(&authority) {
        return Ok(refused(RefusalReason::ReceiptAsEffect));
    }
    let mut joined = match join_members(head, &authority, selection, work)? {
        Ok(joined) => joined,
        Err(reason) => return Ok(refused(reason)),
    };
    let complete = match aggregate(selection, &mut joined, work)? {
        Ok(complete) => complete,
        Err(reason) => return Ok(refused(reason)),
    };
    Ok(Outcome::Complete(complete))
}

fn valid_selection(selection: &Selection) -> bool {
    let identities = [
        &selection.lineage_identity,
        &selection.bundle_identity,
        &selection.population_component_identity,
        &selection.position_component_identity,
        &selection.progress_component_identity,
        &selection.closure_component_identity,
        &selection.completeness_component_identity,
        &selection.population_identity,
        &selection.scope_identity,
        &selection.membership_rule_identity,
        &selection.relationship_declaration_identity,
        &selection.effect_signal_identity,
        selection.plan.identity(),
    ];
    if selection.bundle_revision == 0
        || identities.iter().any(|identity| !identity.valid())
        || [
            selection.capture_component_identity.as_ref(),
            selection.activation_component_identity.as_ref(),
        ]
        .into_iter()
        .flatten()
        .any(|identity| !identity.valid())
        || selection.capture_component_identity.is_some()
            != selection.activation_component_identity.is_some()
        || selection.root_subject_kind.is_empty()
        || selection.grouping_key.is_empty()
        || selection.effect_subject_kind.is_empty()
    {
        return false;
    }
    match selection.plan.predicate() {
        Predicate::Signal(identity) if !identity.valid() => return false,
        Predicate::EffectIdentity(identity) if identity.is_empty() => return false,
        Predicate::All
        | Predicate::Signal(_)
        | Predicate::EffectIdentity(_)
        | Predicate::ValuePresent => {}
    }
    match &selection.plan {
        QueryPlan::ExactSum {
            value_type,
            unit,
            zero_identity,
            zero,
            minimum,
            maximum,
            ..
        } => {
            value_type.valid()
                && unit.valid()
                && zero_identity.valid()
                && *zero == 0
                && minimum <= maximum
                && minimum <= zero
                && zero <= maximum
        }
        QueryPlan::Filter { .. } | QueryPlan::Count { .. } => true,
    }
}

fn components_match(head: &bundle::View, selection: &Selection, work: &mut Work) -> Result<bool> {
    let expected = [
        (
            ComponentRole::Population,
            &selection.population_component_identity,
        ),
        (
            ComponentRole::Position,
            &selection.position_component_identity,
        ),
        (
            ComponentRole::Progress,
            &selection.progress_component_identity,
        ),
        (
            ComponentRole::Closure,
            &selection.closure_component_identity,
        ),
        (
            ComponentRole::Completeness,
            &selection.completeness_component_identity,
        ),
    ];
    for (role, identity) in expected {
        if !component_identity_matches(head, role, identity, work)? {
            return Ok(false);
        }
    }
    let business_authority = match (
        selection.capture_component_identity.as_ref(),
        selection.activation_component_identity.as_ref(),
    ) {
        (None, None) => head.contract() == bundle::V2_CONTRACT,
        (Some(capture), Some(activation)) if head.contract() == bundle::CONTRACT => {
            for (role, identity) in [
                (ComponentRole::Capture, capture),
                (ComponentRole::Activation, activation),
            ] {
                if !component_identity_matches(head, role, identity, work)? {
                    return Ok(false);
                }
            }
            true
        }
        (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => false,
    };
    Ok(business_authority)
}

fn component_identity_matches(
    head: &bundle::View,
    role: ComponentRole,
    identity: &Identity,
    work: &mut Work,
) -> Result<bool> {
    for component in head.payload().components() {
        work.tick()?;
        if component.role() == role {
            return Ok(component.identity() == identity.as_str());
        }
    }
    Ok(false)
}

fn authority<'a>(head: &'a bundle::View, work: &mut Work) -> Result<Option<Authority<'a>>> {
    let mut population = None;
    let mut completeness = None;
    for fact in head.payload().populations() {
        work.tick()?;
        match fact.payload() {
            EmbeddedPayloadRef::Population(value) => population = Some(value),
            EmbeddedPayloadRef::Completeness(value) => completeness = Some(value),
            _ => {}
        }
    }
    let mut position = None;
    for fact in head.payload().positions() {
        work.tick()?;
        if let EmbeddedPayloadRef::Position(value) = fact.payload() {
            position = Some(value);
        }
    }
    let mut progress = None;
    let mut closure = None;
    for fact in head.payload().progress() {
        work.tick()?;
        match fact.payload() {
            EmbeddedPayloadRef::Progress(value) => progress = Some(value),
            EmbeddedPayloadRef::Closure(value) => closure = Some(value),
            _ => {}
        }
    }
    let mut capture = None;
    let mut activation = None;
    for fact in head.payload().records() {
        work.tick()?;
        match fact.payload() {
            EmbeddedPayloadRef::Capture(value) => capture = Some(value),
            EmbeddedPayloadRef::Activation(value) => activation = Some(value),
            _ => {}
        }
    }
    let (Some(population), Some(position), Some(progress), Some(closure), Some(completeness)) =
        (population, position, progress, closure, completeness)
    else {
        return Ok(None);
    };
    Ok(Some(Authority {
        population,
        position,
        progress,
        closure,
        completeness,
        capture,
        activation,
    }))
}

fn authority_axes_match(
    authority: &Authority<'_>,
    selection: &Selection,
    work: &mut Work,
) -> Result<bool> {
    let business_authority_matches = match (
        authority.capture,
        authority.activation,
        selection.capture_component_identity.as_ref(),
        selection.activation_component_identity.as_ref(),
    ) {
        (None, None, None, None) => true,
        (Some(_), Some(activation), Some(capture_identity), Some(_)) => {
            activation.capture_authority().document_identity == capture_identity.as_str()
                && activation.progress_authority().is_some_and(|proof| {
                    proof.document_identity == selection.progress_component_identity.as_str()
                })
                && activation.closure_authority().is_some_and(|proof| {
                    proof.document_identity == selection.closure_component_identity.as_str()
                })
                && activation.completeness_authority().is_some_and(|proof| {
                    proof.document_identity == selection.completeness_component_identity.as_str()
                })
        }
        _ => false,
    };
    let required_sources_match = equal_required_sources(
        authority.progress.required_sources(),
        authority.closure.required_sources(),
        work,
    )?;
    Ok(business_authority_matches
        && authority.population.population_identity() == selection.population_identity.as_str()
        && authority.population.membership_rule_identity()
            == selection.membership_rule_identity.as_str()
        && authority.completeness.population_identity() == selection.population_identity.as_str()
        && authority.progress.scope_identity() == selection.scope_identity.as_str()
        && authority.closure.scope_identity() == selection.scope_identity.as_str()
        && authority.position.clock_identity() == authority.progress.clock_identity()
        && authority.position.clock_revision() == authority.progress.clock_revision()
        && authority.progress.clock_identity() == authority.closure.clock_identity()
        && authority.progress.clock_revision() == authority.closure.clock_revision()
        && required_sources_match
        && boundary_carrier_equal(authority.progress.boundary(), authority.closure.boundary())
        && boundary_matches_population(
            authority.population.selection(),
            authority.progress.boundary(),
        ))
}

fn business_authority_is_active(authority: &Authority<'_>) -> bool {
    match (authority.capture, authority.activation) {
        (None, None) => true,
        (Some(_), Some(activation)) => {
            activation.activation() == super::activation::ActivationState::Active
                && activation.progress() == super::activation::ExecutionState::Closed
                && activation.closure() == super::activation::ExecutionState::Closed
                && activation.evidence() == super::activation::EvidenceState::Complete
        }
        (None, Some(_)) | (Some(_), None) => false,
    }
}

fn equal_required_sources(left: &[String], right: &[String], work: &mut Work) -> Result<bool> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        work.tick()?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

fn boundary_matches_population(
    population: super::population::ScopeRef<'_>,
    boundary: BoundaryRef<'_>,
) -> bool {
    match (population, boundary) {
        (super::population::ScopeRef::Snapshot { .. }, _) => true,
        (
            super::population::ScopeRef::EventTimeWindow {
                start_nanos,
                end_exclusive_nanos,
                ..
            },
            BoundaryRef::TimestampedEvent {
                lower_nanos,
                carrier_end_exclusive_nanos,
                ..
            },
        ) => start_nanos == lower_nanos && end_exclusive_nanos == carrier_end_exclusive_nanos,
        (
            super::population::ScopeRef::EventTimeWindow { .. },
            BoundaryRef::EventPosition { .. } | BoundaryRef::FixedSample { .. },
        ) => false,
    }
}

fn boundary_carrier_equal(left: BoundaryRef<'_>, right: BoundaryRef<'_>) -> bool {
    match (left, right) {
        (
            BoundaryRef::EventPosition {
                lower: left_lower,
                upper_inclusive: left_upper,
                carrier_end_exclusive: left_end,
                watermark: _,
            },
            BoundaryRef::EventPosition {
                lower: right_lower,
                upper_inclusive: right_upper,
                carrier_end_exclusive: right_end,
                watermark: _,
            },
        )
        | (
            BoundaryRef::FixedSample {
                lower: left_lower,
                upper_inclusive: left_upper,
                carrier_end_exclusive: left_end,
                watermark: _,
            },
            BoundaryRef::FixedSample {
                lower: right_lower,
                upper_inclusive: right_upper,
                carrier_end_exclusive: right_end,
                watermark: _,
            },
        ) => left_lower == right_lower && left_upper == right_upper && left_end == right_end,
        (
            BoundaryRef::TimestampedEvent {
                lower_nanos: left_lower,
                upper_inclusive_nanos: left_upper,
                carrier_end_exclusive_nanos: left_end,
                watermark_nanos: _,
            },
            BoundaryRef::TimestampedEvent {
                lower_nanos: right_lower,
                upper_inclusive_nanos: right_upper,
                carrier_end_exclusive_nanos: right_end,
                watermark_nanos: _,
            },
        ) => left_lower == right_lower && left_upper == right_upper && left_end == right_end,
        (
            BoundaryRef::EventPosition { .. },
            BoundaryRef::FixedSample { .. } | BoundaryRef::TimestampedEvent { .. },
        )
        | (
            BoundaryRef::FixedSample { .. },
            BoundaryRef::EventPosition { .. } | BoundaryRef::TimestampedEvent { .. },
        )
        | (
            BoundaryRef::TimestampedEvent { .. },
            BoundaryRef::EventPosition { .. } | BoundaryRef::FixedSample { .. },
        ) => false,
    }
}

fn join_members<'a>(
    head: &'a bundle::View,
    authority: &Authority<'a>,
    selection: &Selection,
    work: &mut Work,
) -> Result<std::result::Result<Vec<Joined<'a>>, RefusalReason>> {
    let required = authority.population.required_members();
    if authority.completeness.fact_count() != required.len() {
        return Ok(Err(RefusalReason::ForeignAuthority));
    }
    for (fact, required_identity) in authority.completeness.facts().zip(required) {
        work.tick()?;
        if fact.member_identity != required_identity {
            return Ok(Err(RefusalReason::ForeignAuthority));
        }
    }
    let structural_bytes = required
        .len()
        .checked_mul(std::mem::size_of::<Joined<'_>>())
        .ok_or_else(|| work.exhausted())?;
    work.ensure_state_capacity(structural_bytes)?;
    let mut joined: Vec<Joined<'a>> = Vec::new();
    joined
        .try_reserve_exact(required.len())
        .map_err(|_| resource_error("query joined-state allocation failed", work))?;
    for fact in authority.completeness.facts() {
        work.tick()?;
        let Some(observation_identity) = fact.observation_identity else {
            return Ok(Err(RefusalReason::ForeignAuthority));
        };
        let Some(record) = find_record(head, observation_identity, work)? else {
            return Ok(Err(RefusalReason::ForeignAuthority));
        };
        let Some(position) = find_position(authority.position, observation_identity, work)? else {
            return Ok(Err(RefusalReason::InvalidExposureOrder));
        };
        if !inside_population(authority.population.selection(), record) {
            return Ok(Err(RefusalReason::OutsideWindow));
        }
        if record.signal_identity() != selection.effect_signal_identity.as_str() {
            return Ok(Err(RefusalReason::ReceiptAsEffect));
        }
        if !business_effect_is_bound(authority, record, work)? {
            return Ok(Err(RefusalReason::ReceiptAsEffect));
        }
        let Some(relationship) = record.causal_relationship() else {
            return Ok(Err(RefusalReason::ForeignRelationship));
        };
        let Some((root, effect)) = relationship_axes(relationship, selection) else {
            return Ok(Err(RefusalReason::ForeignRelationship));
        };
        if !subject_matches(record.subject(), root)
            || root.kind != selection.root_subject_kind.as_slice()
            || root.identity != selection.grouping_key.as_slice()
            || effect.kind != selection.effect_subject_kind.as_slice()
        {
            return Ok(Err(RefusalReason::ForeignSubject));
        }
        let predicate_matches = predicate_matches(selection.plan.predicate(), record, effect);
        work.charge_state(joined_state_bytes(
            fact.member_identity,
            record,
            effect,
            relationship.identity,
            work,
        )?)?;
        let member_identity = Identity::new(fact.member_identity);
        let joined_member = Joined {
            position,
            member_identity,
            record,
            effect,
            relationship_identity: relationship.identity,
            predicate_matches,
            included: true,
        };
        let mut insert_at = joined.len();
        while insert_at != 0 {
            work.tick()?;
            match joined[insert_at - 1].position.cmp(&position) {
                std::cmp::Ordering::Less => break,
                std::cmp::Ordering::Equal => {
                    return Ok(Err(RefusalReason::InvalidExposureOrder));
                }
                std::cmp::Ordering::Greater => insert_at -= 1,
            }
        }
        joined.insert(insert_at, joined_member);
    }
    work.usage.occurrences = joined.len();
    Ok(Ok(joined))
}

fn business_effect_is_bound(
    authority: &Authority<'_>,
    record: &RecordView,
    work: &mut Work,
) -> Result<bool> {
    let (Some(capture), Some(activation)) = (authority.capture, authority.activation) else {
        return Ok(false);
    };
    let mut binding = None;
    for candidate in capture.bindings() {
        work.tick()?;
        if candidate.observation_identity == record.observation_identity()
            && binding.replace(candidate).is_some()
        {
            return Ok(false);
        }
    }
    let Some(binding) = binding else {
        return Ok(false);
    };
    let ValueRef::Present {
        value_type,
        canonical_value,
    } = record.value()
    else {
        return Ok(false);
    };
    if binding.binding_identity != record.binding_identity()
        || binding.value_type != value_type
        || binding.canonical_value != canonical_value
        || binding.unit != record.unit()
    {
        return Ok(false);
    }
    let mut activated = false;
    for candidate in activation.captures() {
        work.tick()?;
        if candidate.source_observation_identity == record.observation_identity() {
            if activated
                || candidate.identity != binding.capture_identity
                || candidate.value_type != value_type
                || candidate.canonical_value != canonical_value
                || candidate.provenance_identity != record.source_identity()
            {
                return Ok(false);
            }
            activated = true;
        }
    }
    Ok(activated)
}

fn find_record<'a>(
    head: &'a bundle::View,
    observation_identity: &str,
    work: &mut Work,
) -> Result<Option<&'a RecordView>> {
    let mut found = None;
    for fact in head.payload().records() {
        work.tick()?;
        let EmbeddedPayloadRef::Observation(record) = fact.payload() else {
            continue;
        };
        if record.observation_identity() == observation_identity {
            if found.is_some() {
                return Ok(None);
            }
            found = Some(record);
        }
    }
    Ok(found)
}

fn find_position(
    position: &super::position::PositionLedgerView,
    observation_identity: &str,
    work: &mut Work,
) -> Result<Option<u64>> {
    let mut found = None;
    for (candidate, observation) in position.positions() {
        work.tick()?;
        if observation == observation_identity {
            let Ok(candidate) = candidate.parse::<u64>() else {
                return Ok(None);
            };
            if found.replace(candidate).is_some() {
                return Ok(None);
            }
        }
    }
    Ok(found)
}

fn relationship_axes<'a>(
    relationship: RelationshipRef<'a>,
    selection: &Selection,
) -> Option<(SubjectRef<'a>, SubjectRef<'a>)> {
    if relationship.declaration_identity != selection.relationship_declaration_identity.as_str() {
        return None;
    }
    Some(match selection.root_endpoint {
        RootEndpoint::Source => (relationship.source, relationship.target),
        RootEndpoint::Target => (relationship.target, relationship.source),
    })
}

fn subject_matches(left: SubjectRef<'_>, right: SubjectRef<'_>) -> bool {
    left == right
}

fn inside_population(scope: super::population::ScopeRef<'_>, record: &RecordView) -> bool {
    match scope {
        super::population::ScopeRef::Snapshot { .. } => true,
        super::population::ScopeRef::EventTimeWindow {
            start_nanos,
            end_exclusive_nanos,
            ..
        } => {
            let Ok(instant) = record.event_time_nanos().parse::<i128>() else {
                return false;
            };
            start_nanos
                .parse::<i128>()
                .ok()
                .zip(end_exclusive_nanos.parse::<i128>().ok())
                .is_some_and(|(start, end)| start <= instant && instant < end)
        }
    }
}

fn predicate_matches(predicate: &Predicate, record: &RecordView, effect: SubjectRef<'_>) -> bool {
    match predicate {
        Predicate::All => true,
        Predicate::Signal(identity) => record.signal_identity() == identity.as_str(),
        Predicate::EffectIdentity(identity) => effect.identity == identity,
        Predicate::ValuePresent => matches!(record.value(), ValueRef::Present { .. }),
    }
}

fn aggregate(
    selection: &Selection,
    joined: &mut [Joined<'_>],
    work: &mut Work,
) -> Result<std::result::Result<CompleteResult, RefusalReason>> {
    if selection.duplicate_policy == DuplicatePolicy::EffectIdentityDeduplicating {
        for index in 0..joined.len() {
            let (prior, current) = joined.split_at_mut(index);
            let current = &mut current[0];
            for earlier in prior.iter().filter(|member| member.included) {
                work.tick()?;
                if earlier.effect.kind == current.effect.kind
                    && earlier.effect.identity == current.effect.identity
                {
                    current.included = false;
                    break;
                }
            }
        }
    }
    let structural_bytes = joined
        .len()
        .checked_mul(std::mem::size_of::<Participation>())
        .ok_or_else(|| work.exhausted())?;
    work.ensure_state_capacity(structural_bytes)?;
    let mut participation = Vec::new();
    participation
        .try_reserve_exact(joined.len())
        .map_err(|_| resource_error("query participation allocation failed", work))?;
    for member in joined.iter() {
        work.tick()?;
        work.charge_state(participation_state_bytes(
            &member.member_identity,
            member.record,
            member.effect,
            member.relationship_identity,
            work,
        )?)?;
        participation.push(Participation {
            position: member.position,
            member_identity: member.member_identity.clone(),
            observation_identity: Identity::new(member.record.observation_identity()),
            effect_kind: member.effect.kind.to_vec(),
            effect_identity: member.effect.identity.to_vec(),
            relationship_identity: member.relationship_identity.to_vec(),
            predicate_matches: member.predicate_matches,
            included: member.included,
        });
    }
    let value = match &selection.plan {
        QueryPlan::Filter { quantifier, .. } => {
            let mut result = *quantifier == FilterQuantifier::All;
            for member in joined.iter().filter(|member| member.included) {
                work.tick()?;
                match quantifier {
                    FilterQuantifier::Any => result |= member.predicate_matches,
                    FilterQuantifier::All => result &= member.predicate_matches,
                }
            }
            AggregateValue::Boolean(result)
        }
        QueryPlan::Count { .. } => {
            let mut count = 0_u64;
            for member in joined {
                work.tick()?;
                if member.included && member.predicate_matches {
                    let Some(next) = count.checked_add(1) else {
                        return Ok(Err(RefusalReason::InvalidPlan));
                    };
                    count = next;
                }
            }
            AggregateValue::Count(count)
        }
        QueryPlan::ExactSum {
            representation,
            value_type,
            unit,
            zero,
            minimum,
            maximum,
            ..
        } => {
            let mut sum = *zero;
            for member in joined {
                work.tick()?;
                if !member.included || !member.predicate_matches {
                    continue;
                }
                work.arithmetic_step()?;
                if member.record.unit() != unit.as_str() {
                    return Ok(Err(RefusalReason::WrongUnit));
                }
                let ValueRef::Present {
                    value_type: observed_type,
                    canonical_value,
                } = member.record.value()
                else {
                    return Ok(Err(RefusalReason::InvalidRepresentation));
                };
                if observed_type != value_type.as_str() {
                    return Ok(Err(RefusalReason::InvalidRepresentation));
                }
                let Some(operand) = parse_number(*representation, canonical_value) else {
                    return Ok(Err(RefusalReason::InvalidRepresentation));
                };
                let Some(next) = sum.checked_add(operand) else {
                    return Ok(Err(RefusalReason::ArithmeticOverflow));
                };
                sum = next;
                if sum < *minimum || sum > *maximum {
                    return Ok(Err(RefusalReason::InvalidPrefix));
                }
            }
            AggregateValue::Sum(sum)
        }
    };
    Ok(Ok(CompleteResult {
        participation,
        value,
    }))
}

fn parse_number(representation: NumericRepresentation, value: &str) -> Option<i128> {
    match representation {
        NumericRepresentation::SignedInteger => {
            let parsed = value.parse::<i128>().ok()?;
            (parsed.to_string() == value).then_some(parsed)
        }
    }
}

fn joined_state_bytes(
    member: &str,
    record: &RecordView,
    effect: SubjectRef<'_>,
    relationship_identity: &[u8],
    work: &Work,
) -> Result<usize> {
    checked_sum(
        [
            std::mem::size_of::<Joined<'_>>(),
            member.len(),
            record.observation_identity().len(),
            effect.kind.len(),
            effect.identity.len(),
            relationship_identity.len(),
        ],
        work,
    )
}

fn participation_state_bytes(
    member: &Identity,
    record: &RecordView,
    effect: SubjectRef<'_>,
    relationship_identity: &[u8],
    work: &Work,
) -> Result<usize> {
    checked_sum(
        [
            std::mem::size_of::<Participation>(),
            member.as_str().len(),
            record.observation_identity().len(),
            effect.kind.len(),
            effect.identity.len(),
            relationship_identity.len(),
        ],
        work,
    )
}

fn reason_set(present: [bool; IncompleteReason::COUNT]) -> impl Iterator<Item = IncompleteReason> {
    IncompleteReason::ALL
        .into_iter()
        .zip(present)
        .filter_map(|(reason, present)| present.then_some(reason))
}

fn incomplete(
    reasons: impl IntoIterator<Item = IncompleteReason>,
    work: &mut Work,
) -> Result<Outcome> {
    let mut present = [false; IncompleteReason::COUNT];
    for reason in reasons {
        present[reason.index()] = true;
    }
    let count = present.iter().filter(|present| **present).count();
    let bytes = count
        .checked_mul(std::mem::size_of::<IncompleteReason>())
        .ok_or_else(|| work.exhausted())?;
    work.charge_state(bytes)?;
    let mut reasons = Vec::new();
    reasons
        .try_reserve_exact(count)
        .map_err(|_| resource_error("query incomplete-reason allocation failed", work))?;
    reasons.extend(reason_set(present));
    debug_assert!(!reasons.is_empty());
    Ok(Outcome::Incomplete(IncompleteQuery { reasons }))
}

const fn refused(reason: RefusalReason) -> Outcome {
    Outcome::Refused(RefusedQuery { reason })
}

#[derive(Serialize)]
struct EvaluationWire<'a> {
    contract: &'static str,
    lineage_identity: &'a str,
    bundle_identity: &'a str,
    bundle_revision: u64,
    population_component_identity: &'a str,
    position_component_identity: &'a str,
    progress_component_identity: &'a str,
    closure_component_identity: &'a str,
    completeness_component_identity: &'a str,
    capture_component_identity: Option<&'a str>,
    activation_component_identity: Option<&'a str>,
    population_identity: &'a str,
    scope_identity: &'a str,
    membership_rule_identity: &'a str,
    relationship_declaration_identity: &'a str,
    root_endpoint: RootEndpoint,
    root_subject_kind: Hex<'a>,
    grouping_key: Hex<'a>,
    effect_subject_kind: Hex<'a>,
    effect_signal_identity: &'a str,
    duplicate_policy: DuplicatePolicy,
    plan: PlanWire<'a>,
    outcome: OutcomeWire<'a>,
    usage: UsageWire,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum PlanWire<'a> {
    Filter {
        identity: &'a str,
        predicate: PredicateWire<'a>,
        quantifier: FilterQuantifier,
    },
    Count {
        identity: &'a str,
        predicate: PredicateWire<'a>,
    },
    ExactSum {
        identity: &'a str,
        predicate: PredicateWire<'a>,
        representation: NumericRepresentation,
        value_type: &'a str,
        unit: &'a str,
        zero_identity: &'a str,
        zero: String,
        minimum: String,
        maximum: String,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "identity", rename_all = "kebab-case")]
enum PredicateWire<'a> {
    All,
    Signal(&'a str),
    EffectIdentity(Hex<'a>),
    ValuePresent,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
enum OutcomeWire<'a> {
    Complete {
        participation: ParticipationList<'a>,
        value: AggregateWire,
    },
    Incomplete {
        reasons: &'a [IncompleteReason],
    },
    Refused {
        reason: RefusalReason,
    },
}

#[derive(Serialize)]
struct ParticipationWire<'a> {
    position: u64,
    member_identity: &'a str,
    observation_identity: &'a str,
    effect_kind: Hex<'a>,
    effect_identity: Hex<'a>,
    relationship_identity: Hex<'a>,
    predicate_matches: bool,
    included: bool,
}

#[derive(Clone, Copy)]
struct ParticipationList<'a>(&'a [Participation]);

impl Serialize for ParticipationList<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for row in self.0 {
            sequence.serialize_element(&ParticipationWire {
                position: row.position,
                member_identity: row.member_identity.as_str(),
                observation_identity: row.observation_identity.as_str(),
                effect_kind: Hex(&row.effect_kind),
                effect_identity: Hex(&row.effect_identity),
                relationship_identity: Hex(&row.relationship_identity),
                predicate_matches: row.predicate_matches,
                included: row.included,
            })?;
        }
        sequence.end()
    }
}

#[derive(Clone, Copy)]
struct Hex<'a>(&'a [u8]);

impl Serialize for Hex<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl fmt::Display for Hex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
enum AggregateWire {
    Boolean(bool),
    Count(u64),
    Sum(String),
}

#[derive(Clone, Copy, Serialize)]
struct UsageWire {
    input_bytes: u64,
    members: u64,
    occurrences: u64,
    arithmetic_steps: u64,
    work: u64,
    state_bytes: u64,
}

fn encode(
    lineage: &bundle::LineageView,
    selection: &Selection,
    outcome: &Outcome,
    usage: Usage,
    limits: Limits,
) -> Result<Vec<u8>> {
    let wire = EvaluationWire {
        contract: "quire.observation.closed-population-query/v1",
        lineage_identity: lineage.document().identity().as_str(),
        bundle_identity: lineage.head().identity().as_str(),
        bundle_revision: lineage.head().revision(),
        population_component_identity: selection.population_component_identity.as_str(),
        position_component_identity: selection.position_component_identity.as_str(),
        progress_component_identity: selection.progress_component_identity.as_str(),
        closure_component_identity: selection.closure_component_identity.as_str(),
        completeness_component_identity: selection.completeness_component_identity.as_str(),
        capture_component_identity: selection
            .capture_component_identity
            .as_ref()
            .map(Identity::as_str),
        activation_component_identity: selection
            .activation_component_identity
            .as_ref()
            .map(Identity::as_str),
        population_identity: selection.population_identity.as_str(),
        scope_identity: selection.scope_identity.as_str(),
        membership_rule_identity: selection.membership_rule_identity.as_str(),
        relationship_declaration_identity: selection.relationship_declaration_identity.as_str(),
        root_endpoint: selection.root_endpoint,
        root_subject_kind: Hex(&selection.root_subject_kind),
        grouping_key: Hex(&selection.grouping_key),
        effect_subject_kind: Hex(&selection.effect_subject_kind),
        effect_signal_identity: selection.effect_signal_identity.as_str(),
        duplicate_policy: selection.duplicate_policy,
        plan: plan_wire(&selection.plan),
        outcome: outcome_wire(outcome),
        usage: usage_wire(usage)?,
    };
    to_bounded_json(&wire, limits.max_output_bytes)
}

fn plan_wire(plan: &QueryPlan) -> PlanWire<'_> {
    match plan {
        QueryPlan::Filter {
            identity,
            predicate,
            quantifier,
        } => PlanWire::Filter {
            identity: identity.as_str(),
            predicate: predicate_wire(predicate),
            quantifier: *quantifier,
        },
        QueryPlan::Count {
            identity,
            predicate,
        } => PlanWire::Count {
            identity: identity.as_str(),
            predicate: predicate_wire(predicate),
        },
        QueryPlan::ExactSum {
            identity,
            predicate,
            representation,
            value_type,
            unit,
            zero_identity,
            zero,
            minimum,
            maximum,
        } => PlanWire::ExactSum {
            identity: identity.as_str(),
            predicate: predicate_wire(predicate),
            representation: *representation,
            value_type: value_type.as_str(),
            unit: unit.as_str(),
            zero_identity: zero_identity.as_str(),
            zero: zero.to_string(),
            minimum: minimum.to_string(),
            maximum: maximum.to_string(),
        },
    }
}

fn predicate_wire(predicate: &Predicate) -> PredicateWire<'_> {
    match predicate {
        Predicate::All => PredicateWire::All,
        Predicate::Signal(identity) => PredicateWire::Signal(identity.as_str()),
        Predicate::EffectIdentity(identity) => PredicateWire::EffectIdentity(Hex(identity)),
        Predicate::ValuePresent => PredicateWire::ValuePresent,
    }
}

fn outcome_wire(outcome: &Outcome) -> OutcomeWire<'_> {
    match outcome {
        Outcome::Complete(result) => OutcomeWire::Complete {
            participation: ParticipationList(&result.participation),
            value: match &result.value {
                AggregateValue::Boolean(value) => AggregateWire::Boolean(*value),
                AggregateValue::Count(value) => AggregateWire::Count(*value),
                AggregateValue::Sum(value) => AggregateWire::Sum(value.to_string()),
            },
        },
        Outcome::Incomplete(value) => OutcomeWire::Incomplete {
            reasons: &value.reasons,
        },
        Outcome::Refused(value) => OutcomeWire::Refused {
            reason: value.reason,
        },
    }
}

fn usage_wire(value: Usage) -> Result<UsageWire> {
    Ok(UsageWire {
        input_bytes: wire(value.input_bytes)?,
        members: wire(value.members)?,
        occurrences: wire(value.occurrences)?,
        arithmetic_steps: wire(value.arithmetic_steps)?,
        work: wire(value.work)?,
        state_bytes: wire(value.state_bytes)?,
    })
}

fn wire(value: usize) -> Result<u64> {
    u64::try_from(value)
        .map_err(|_| resource_error_without_work("query count is not wire-representable"))
}

fn checked_sum(values: impl IntoIterator<Item = usize>, work: &Work) -> Result<usize> {
    values.into_iter().try_fold(0usize, |sum, value| {
        sum.checked_add(value).ok_or_else(|| work.exhausted())
    })
}

fn resource_error(detail: &'static str, work: &Work) -> Error {
    Error::new(
        ErrorCode::ResourceIncomplete,
        detail,
        AuthorityUsage {
            visited_fields: work.usage.work,
            ..AuthorityUsage::default()
        },
    )
}

fn resource_error_without_work(detail: &'static str) -> Error {
    Error::new(
        ErrorCode::ResourceIncomplete,
        detail,
        AuthorityUsage::default(),
    )
}
