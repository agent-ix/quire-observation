// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Bounded affected-region planning over validated authority revisions.
//!
//! The planner treats the supplied graph as data. It follows only explicit
//! edges, uses bundle replacements as its only invalidation seeds, and uses
//! scope/window order solely to make the recomputation schedule deterministic.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;
use sha2::{Digest as _, Sha256};

use super::bundle::{ComponentRole, EmbeddedPayloadRef, View};
use super::clock::{ClockView, RangeRef};
use super::common::{to_bounded_json, Error, ErrorCode, Result, Usage as AuthorityUsage};
use crate::{ClockRange, Identity};

/// Immutable owner maxima for repair planning.
pub const OWNER_MAX: Limits = Limits {
    max_nodes: 20_000,
    max_edges: 100_000,
    max_work: 1_000_000,
    max_retained_results: 10_000,
    max_retained_bytes: 8 * 1024 * 1024,
    max_output_bytes: 8 * 1024 * 1024,
};

/// Caller-lowerable resource ceilings for one repair plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum fact and result nodes retained by the selected graph.
    pub max_nodes: usize,
    /// Maximum explicit dependency edges visited.
    pub max_edges: usize,
    /// Maximum deterministic planner work units.
    pub max_work: usize,
    /// Maximum prior result records retained while planning.
    pub max_retained_results: usize,
    /// Maximum total bytes across selected fact/result state and its identity keys.
    pub max_retained_bytes: usize,
    /// Maximum canonical repair-plan bytes emitted.
    pub max_output_bytes: usize,
}

impl Limits {
    /// Returns the immutable planner maxima.
    #[must_use]
    pub const fn owner_max() -> Self {
        OWNER_MAX
    }

    /// Clamps every caller ceiling to the corresponding owner maximum.
    #[must_use]
    pub const fn effective(self) -> Self {
        Self {
            max_nodes: min(self.max_nodes, OWNER_MAX.max_nodes),
            max_edges: min(self.max_edges, OWNER_MAX.max_edges),
            max_work: min(self.max_work, OWNER_MAX.max_work),
            max_retained_results: min(self.max_retained_results, OWNER_MAX.max_retained_results),
            max_retained_bytes: min(self.max_retained_bytes, OWNER_MAX.max_retained_bytes),
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

/// Exact successful planner resource use.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Usage {
    /// Fact and result nodes admitted to the graph.
    pub nodes: usize,
    /// Explicit dependency edges admitted to the graph.
    pub edges: usize,
    /// Deterministic validation and traversal work units.
    pub work: usize,
    /// Prior results retained as bounded planner state.
    pub retained_results: usize,
    /// Total exact bytes across selected fact/result state and its identity keys.
    pub retained_bytes: usize,
    /// Canonical emitted plan bytes.
    pub output_bytes: usize,
}

/// Exact scope and half-open clock window in which a fact or result is observable.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Region {
    scope_identity: Identity,
    clock_identity: Identity,
    clock_revision: Identity,
    window: Window,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Window {
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

impl Region {
    /// Validates and constructs an exact observable region.
    pub fn new(
        scope_identity: Identity,
        clock_identity: Identity,
        clock_revision: Identity,
        range: ClockRange,
    ) -> Result<Self> {
        if !scope_identity.valid() || !clock_identity.valid() || !clock_revision.valid() {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "repair region identities must be non-empty",
                AuthorityUsage::default(),
            ));
        }
        let window = match range {
            ClockRange::EventPosition {
                start,
                end_exclusive,
            } if start < end_exclusive => Window::EventPosition {
                start,
                end_exclusive,
            },
            ClockRange::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            } if start < end_exclusive && period_nanos > 0 => Window::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            },
            ClockRange::Timestamp {
                start_nanos,
                end_nanos,
            } if start_nanos < end_nanos => Window::Timestamp {
                start_nanos,
                end_nanos,
            },
            _ => {
                return Err(Error::new(
                    ErrorCode::InvalidInterval,
                    "repair windows must be non-empty half-open intervals",
                    AuthorityUsage::default(),
                ));
            }
        };
        Ok(Self {
            scope_identity,
            clock_identity,
            clock_revision,
            window,
        })
    }

    /// Returns the exact observable scope identity.
    #[must_use]
    pub const fn scope_identity(&self) -> &Identity {
        &self.scope_identity
    }

    /// Returns the selected clock identity.
    #[must_use]
    pub const fn clock_identity(&self) -> &Identity {
        &self.clock_identity
    }

    /// Returns the selected immutable clock revision.
    #[must_use]
    pub const fn clock_revision(&self) -> &Identity {
        &self.clock_revision
    }

    /// Returns the exact half-open clock range.
    #[must_use]
    pub const fn range(&self) -> ClockRange {
        match self.window {
            Window::EventPosition {
                start,
                end_exclusive,
            } => ClockRange::EventPosition {
                start,
                end_exclusive,
            },
            Window::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            } => ClockRange::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            },
            Window::Timestamp {
                start_nanos,
                end_nanos,
            } => ClockRange::Timestamp {
                start_nanos,
                end_nanos,
            },
        }
    }

    /// Returns whether two exact regions overlap in one scope and clock domain.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        if self.scope_identity != other.scope_identity
            || self.clock_identity != other.clock_identity
            || self.clock_revision != other.clock_revision
        {
            return false;
        }
        match (&self.window, &other.window) {
            (
                Window::EventPosition {
                    start: left_start,
                    end_exclusive: left_end,
                },
                Window::EventPosition {
                    start: right_start,
                    end_exclusive: right_end,
                },
            ) => left_start < right_end && right_start < left_end,
            (
                Window::FixedSample {
                    start: left_start,
                    end_exclusive: left_end,
                    epoch_nanos: left_epoch,
                    period_nanos: left_period,
                },
                Window::FixedSample {
                    start: right_start,
                    end_exclusive: right_end,
                    epoch_nanos: right_epoch,
                    period_nanos: right_period,
                },
            ) => {
                left_epoch == right_epoch
                    && left_period == right_period
                    && left_start < right_end
                    && right_start < left_end
            }
            (
                Window::Timestamp {
                    start_nanos: left_start,
                    end_nanos: left_end,
                },
                Window::Timestamp {
                    start_nanos: right_start,
                    end_nanos: right_end,
                },
            ) => left_start < right_end && right_start < left_end,
            _ => false,
        }
    }

    fn has_same_clock_domain(&self, other: &Self) -> bool {
        match (&self.window, &other.window) {
            (Window::EventPosition { .. }, Window::EventPosition { .. })
            | (Window::Timestamp { .. }, Window::Timestamp { .. }) => true,
            (
                Window::FixedSample {
                    epoch_nanos: left_epoch,
                    period_nanos: left_period,
                    ..
                },
                Window::FixedSample {
                    epoch_nanos: right_epoch,
                    period_nanos: right_period,
                    ..
                },
            ) => left_epoch == right_epoch && left_period == right_period,
            _ => false,
        }
    }
}

/// Exact observable region assigned to one authority fact node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactRegion {
    identity: Identity,
    region: Region,
}

impl FactRegion {
    /// Constructs a fact-region declaration.
    #[must_use]
    pub const fn new(identity: Identity, region: Region) -> Self {
        Self { identity, region }
    }

    /// Returns the exact authority fact identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the exact observable region.
    #[must_use]
    pub const fn region(&self) -> &Region {
        &self.region
    }
}

/// Immutable prior result supplied to the planner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriorResult {
    identity: Identity,
    region: Region,
    bytes: Vec<u8>,
}

impl PriorResult {
    /// Constructs one exact prior result record.
    #[must_use]
    pub const fn new(identity: Identity, region: Region, bytes: Vec<u8>) -> Self {
        Self {
            identity,
            region,
            bytes,
        }
    }

    /// Returns the exact result identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the exact result observable region.
    #[must_use]
    pub const fn region(&self) -> &Region {
        &self.region
    }

    /// Returns the immutable prior canonical result bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// One explicit directed dependency edge from a fact or result to a result.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DependencyEdge {
    source_identity: Identity,
    dependent_identity: Identity,
}

impl DependencyEdge {
    /// Constructs an exact directed dependency edge.
    #[must_use]
    pub const fn new(source_identity: Identity, dependent_identity: Identity) -> Self {
        Self {
            source_identity,
            dependent_identity,
        }
    }

    /// Returns the explicit source identity.
    #[must_use]
    pub const fn source_identity(&self) -> &Identity {
        &self.source_identity
    }

    /// Returns the explicit dependent result identity.
    #[must_use]
    pub const fn dependent_identity(&self) -> &Identity {
        &self.dependent_identity
    }
}

/// Complete revision-qualified planner input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    authority_revision: Identity,
    fact_regions: Vec<FactRegion>,
    prior_results: Vec<PriorResult>,
    edges: Vec<DependencyEdge>,
}

impl Selection {
    /// Constructs an explicit finite repair graph for one prior authority revision.
    #[must_use]
    pub const fn new(
        authority_revision: Identity,
        fact_regions: Vec<FactRegion>,
        prior_results: Vec<PriorResult>,
        edges: Vec<DependencyEdge>,
    ) -> Self {
        Self {
            authority_revision,
            fact_regions,
            prior_results,
            edges,
        }
    }

    /// Returns the exact prior bundle identity qualifying every graph node.
    #[must_use]
    pub const fn authority_revision(&self) -> &Identity {
        &self.authority_revision
    }

    /// Returns the selected authority fact regions.
    #[must_use]
    pub fn fact_regions(&self) -> &[FactRegion] {
        &self.fact_regions
    }

    /// Returns the immutable prior results.
    #[must_use]
    pub fn prior_results(&self) -> &[PriorResult] {
        &self.prior_results
    }

    /// Returns the explicit dependency edges.
    #[must_use]
    pub fn edges(&self) -> &[DependencyEdge] {
        &self.edges
    }
}

/// Borrowed prior result retained by a successful plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetainedResultRef<'a>(&'a PriorResult);

impl<'a> RetainedResultRef<'a> {
    /// Returns the byte-identical prior result identity.
    #[must_use]
    pub const fn identity(self) -> &'a Identity {
        &self.0.identity
    }

    /// Returns the exact prior canonical result bytes.
    #[must_use]
    pub fn bytes(self) -> &'a [u8] {
        &self.0.bytes
    }

    /// Returns the exact prior observable region.
    #[must_use]
    pub const fn region(self) -> &'a Region {
        &self.0.region
    }
}

/// One explicit bundle replacement retained by the plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangedFact {
    role: ComponentRole,
    prior_identity: Identity,
    successor_identity: Identity,
    prior_region: Region,
    successor_region: Region,
}

impl ChangedFact {
    /// Returns the replaced component role.
    #[must_use]
    pub const fn role(&self) -> ComponentRole {
        self.role
    }

    /// Returns the exact superseded fact identity.
    #[must_use]
    pub const fn prior_identity(&self) -> &Identity {
        &self.prior_identity
    }

    /// Returns the exact successor fact identity.
    #[must_use]
    pub const fn successor_identity(&self) -> &Identity {
        &self.successor_identity
    }

    /// Returns the exact observable region of the superseded fact.
    #[must_use]
    pub const fn prior_region(&self) -> &Region {
        &self.prior_region
    }

    /// Returns the exact observable region of the successor fact.
    #[must_use]
    pub const fn successor_region(&self) -> &Region {
        &self.successor_region
    }
}

/// Complete deterministic repair plan. Construction is fail-closed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    identity: Identity,
    bytes: Vec<u8>,
    closure_identity: Identity,
    changed_facts: Vec<ChangedFact>,
    affected: Vec<PriorResult>,
    unaffected: Vec<PriorResult>,
    dependencies: Vec<DependencyEdge>,
    recomputation_order: Vec<Identity>,
    usage: Usage,
}

impl Plan {
    /// Returns the content-derived plan identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the canonical deterministic plan bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the exact successor-bundle closure authority component.
    #[must_use]
    pub const fn closure_identity(&self) -> &Identity {
        &self.closure_identity
    }

    /// Returns the exact bundle replacement seeds.
    #[must_use]
    pub fn changed_facts(&self) -> impl ExactSizeIterator<Item = &ChangedFact> {
        self.changed_facts.iter()
    }

    /// Returns affected result identities in canonical identity order.
    #[must_use]
    pub fn affected(&self) -> impl ExactSizeIterator<Item = &Identity> {
        self.affected.iter().map(PriorResult::identity)
    }

    /// Returns affected prior results retained for later supersession.
    #[must_use]
    pub fn affected_prior(&self) -> impl ExactSizeIterator<Item = RetainedResultRef<'_>> {
        self.affected.iter().map(RetainedResultRef)
    }

    /// Returns byte-identical unaffected prior results in canonical identity order.
    #[must_use]
    pub fn unaffected(&self) -> impl ExactSizeIterator<Item = RetainedResultRef<'_>> {
        self.unaffected.iter().map(RetainedResultRef)
    }

    /// Returns every explicit direct dependency of an affected result.
    #[must_use]
    pub fn dependencies(&self) -> impl ExactSizeIterator<Item = &DependencyEdge> {
        self.dependencies.iter()
    }

    /// Returns the dependency-safe canonical recomputation schedule.
    #[must_use]
    pub fn recomputation_order(&self) -> impl ExactSizeIterator<Item = &Identity> {
        self.recomputation_order.iter()
    }

    /// Returns exact successful planner resource use.
    #[must_use]
    pub const fn usage(&self) -> Usage {
        self.usage
    }
}

#[derive(Serialize)]
struct PlanWire<'a> {
    contract: &'static str,
    prior_bundle_identity: &'a str,
    successor_bundle_identity: &'a str,
    closure_identity: &'a str,
    changed_facts: Vec<ChangedFactWire<'a>>,
    affected: Vec<ResultWire<'a>>,
    unaffected: Vec<ResultWire<'a>>,
    dependencies: Vec<DependencyWire<'a>>,
    recomputation_order: Vec<&'a str>,
}

#[derive(Serialize)]
struct DependencyWire<'a> {
    source_identity: &'a str,
    dependent_identity: &'a str,
}

#[derive(Serialize)]
struct ChangedFactWire<'a> {
    role: ComponentRole,
    prior_identity: &'a str,
    successor_identity: &'a str,
    prior_region: RegionWire<'a>,
    successor_region: RegionWire<'a>,
}

#[derive(Serialize)]
struct ResultWire<'a> {
    identity: &'a str,
    region: RegionWire<'a>,
    prior_digest: String,
    prior_bytes: u64,
}

#[derive(Serialize)]
struct RegionWire<'a> {
    scope_identity: &'a str,
    clock_identity: &'a str,
    clock_revision: &'a str,
    window: WindowWire,
}

#[derive(Serialize)]
#[serde(tag = "clock_family", rename_all = "kebab-case")]
enum WindowWire {
    EventPosition {
        start: u64,
        end_exclusive: u64,
    },
    FixedSample {
        start: u64,
        end_exclusive: u64,
        epoch_nanos: String,
        period_nanos: u64,
    },
    Timestamp {
        start_nanos: String,
        end_nanos: String,
    },
}

struct Work {
    value: usize,
    limits: Limits,
    retained_bytes: usize,
}

impl Work {
    fn tick(&mut self) -> Result<()> {
        self.value = self.value.checked_add(1).ok_or_else(|| self.exhausted())?;
        if self.value > self.limits.max_work {
            return Err(self.exhausted());
        }
        Ok(())
    }

    fn exhausted(&self) -> Error {
        Error::new(
            ErrorCode::ResourceIncomplete,
            "repair planner work limit exceeded",
            self.authority_usage(0),
        )
    }

    fn authority_usage(&self, wire_bytes: usize) -> AuthorityUsage {
        AuthorityUsage {
            wire_bytes,
            visited_fields: self.value,
            ..AuthorityUsage::default()
        }
    }
}

/// Derives one deterministic bounded repair plan from a validated direct revision pair.
pub fn plan(prior: &View, successor: &View, selection: &Selection, limits: Limits) -> Result<Plan> {
    let effective = limits.effective();
    validate_revision_pair(prior, successor, selection)?;

    let nodes = selection
        .fact_regions
        .len()
        .checked_add(selection.prior_results.len())
        .ok_or_else(|| resource_error(0, "repair node count overflow"))?;
    if nodes > effective.max_nodes
        || selection.edges.len() > effective.max_edges
        || selection.prior_results.len() > effective.max_retained_results
    {
        return Err(resource_error(
            0,
            "repair graph exceeds an effective count limit",
        ));
    }
    let mut work = Work {
        value: 0,
        limits: effective,
        retained_bytes: 0,
    };
    for fact in &selection.fact_regions {
        work.tick()?;
        let fact_bytes = region_identity_bytes(&fact.region)
            .and_then(|region_bytes| fact.identity.as_str().len().checked_add(region_bytes))
            .ok_or_else(|| work.exhausted())?;
        work.retained_bytes = work
            .retained_bytes
            .checked_add(fact_bytes)
            .ok_or_else(|| work.exhausted())?;
        ensure_state_bytes(&work)?;
    }
    for result in &selection.prior_results {
        work.tick()?;
        let result_bytes = result
            .bytes
            .len()
            .checked_add(result.identity.as_str().len())
            .and_then(|value| {
                region_identity_bytes(&result.region)
                    .and_then(|region_bytes| value.checked_add(region_bytes))
            })
            .ok_or_else(|| work.exhausted())?;
        work.retained_bytes = work
            .retained_bytes
            .checked_add(result_bytes)
            .ok_or_else(|| work.exhausted())?;
        ensure_state_bytes(&work)?;
    }
    let prior_component_identities: BTreeSet<_> = prior
        .payload()
        .components()
        .map(|component| component.identity())
        .collect();
    let successor_component_identities: BTreeSet<_> = successor
        .payload()
        .components()
        .map(|component| component.identity())
        .collect();
    let mut fact_by_identity = BTreeMap::new();
    for fact in &selection.fact_regions {
        work.tick()?;
        validate_external_identity(&fact.identity)?;
        if (!prior_component_identities.contains(fact.identity.as_str())
            && !successor_component_identities.contains(fact.identity.as_str()))
            || fact_by_identity
                .insert(fact.identity.as_str(), &fact.region)
                .is_some()
        {
            return Err(invalid_dependency(
                &work,
                "fact regions must name distinct prior or successor bundle components",
            ));
        }
    }

    let mut result_by_identity = BTreeMap::new();
    for result in &selection.prior_results {
        work.tick()?;
        validate_external_identity(&result.identity)?;
        if prior_component_identities.contains(result.identity.as_str())
            || successor_component_identities.contains(result.identity.as_str())
            || result_by_identity
                .insert(result.identity.as_str(), result)
                .is_some()
        {
            return Err(invalid_dependency(
                &work,
                "result identities must be distinct from every fact and result",
            ));
        }
    }
    validate_clock_domains(&selection.fact_regions, &selection.prior_results, &mut work)?;
    validate_authority_regions(
        prior,
        successor,
        &prior_component_identities,
        &successor_component_identities,
        &selection.fact_regions,
        &selection.prior_results,
        &mut work,
    )?;

    let mut changed_facts = Vec::new();
    changed_facts
        .try_reserve_exact(successor.payload().replacements().len())
        .map_err(|_| work.exhausted())?;
    for replacement in successor.payload().replacements() {
        work.tick()?;
        if !prior_component_identities.contains(replacement.prior_identity())
            || !successor_component_identities.contains(replacement.successor_identity())
            || !fact_by_identity.contains_key(replacement.prior_identity())
            || !fact_by_identity.contains_key(replacement.successor_identity())
        {
            return Err(invalid_dependency(
                &work,
                "each bundle replacement requires exact prior and successor fact regions",
            ));
        }
        changed_facts.push(ChangedFact {
            role: replacement.role(),
            prior_identity: Identity::new(replacement.prior_identity()),
            successor_identity: Identity::new(replacement.successor_identity()),
            prior_region: (*fact_by_identity
                .get(replacement.prior_identity())
                .ok_or_else(|| invalid_dependency(&work, "prior replacement region is absent"))?)
            .clone(),
            successor_region: (*fact_by_identity
                .get(replacement.successor_identity())
                .ok_or_else(|| {
                    invalid_dependency(&work, "successor replacement region is absent")
                })?)
            .clone(),
        });
    }
    changed_facts.sort_by(|left, right| {
        (&left.prior_identity, &left.successor_identity)
            .cmp(&(&right.prior_identity, &right.successor_identity))
    });

    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut unique_edges = BTreeSet::new();
    for edge in &selection.edges {
        work.tick()?;
        validate_external_identity(&edge.source_identity)?;
        validate_external_identity(&edge.dependent_identity)?;
        let source_known = (prior_component_identities.contains(edge.source_identity.as_str())
            && fact_by_identity.contains_key(edge.source_identity.as_str()))
            || result_by_identity.contains_key(edge.source_identity.as_str());
        if !source_known || !result_by_identity.contains_key(edge.dependent_identity.as_str()) {
            return Err(invalid_dependency(
                &work,
                "dependency edges must connect known facts or results to known results",
            ));
        }
        if !unique_edges.insert((
            edge.source_identity.as_str(),
            edge.dependent_identity.as_str(),
        )) {
            return Err(invalid_dependency(
                &work,
                "duplicate dependency edges are not canonical",
            ));
        }
        outgoing
            .entry(edge.source_identity.as_str())
            .or_default()
            .push(edge.dependent_identity.as_str());
    }
    for dependents in outgoing.values_mut() {
        dependents.sort_unstable();
    }

    validate_acyclic(&result_by_identity, &selection.edges, &mut work)?;
    let affected_identities =
        affected_closure(&changed_facts, &result_by_identity, &outgoing, &mut work)?;
    let schedule = schedule_affected(
        &affected_identities,
        &result_by_identity,
        &selection.edges,
        &mut work,
    )?;

    let mut affected = Vec::new();
    let mut unaffected = Vec::new();
    affected
        .try_reserve_exact(affected_identities.len())
        .map_err(|_| work.exhausted())?;
    unaffected
        .try_reserve_exact(
            selection
                .prior_results
                .len()
                .checked_sub(affected_identities.len())
                .ok_or_else(|| {
                    invalid_dependency(&work, "affected result count exceeds selected results")
                })?,
        )
        .map_err(|_| work.exhausted())?;
    for result in &selection.prior_results {
        work.tick()?;
        if affected_identities.contains(result.identity.as_str()) {
            affected.push(result.clone());
        } else {
            unaffected.push(result.clone());
        }
    }
    affected.sort_by(|left, right| left.identity.cmp(&right.identity));
    unaffected.sort_by(|left, right| left.identity.cmp(&right.identity));

    let mut dependency_count = 0usize;
    let mut dependency_bytes = 0usize;
    for edge in selection
        .edges
        .iter()
        .filter(|edge| affected_identities.contains(edge.dependent_identity.as_str()))
    {
        work.tick()?;
        dependency_count = dependency_count
            .checked_add(1)
            .ok_or_else(|| work.exhausted())?;
        let edge_bytes = edge
            .source_identity
            .as_str()
            .len()
            .checked_add(edge.dependent_identity.as_str().len())
            .ok_or_else(|| work.exhausted())?;
        dependency_bytes = dependency_bytes
            .checked_add(edge_bytes)
            .ok_or_else(|| work.exhausted())?;
    }
    work.retained_bytes = work
        .retained_bytes
        .checked_add(dependency_bytes)
        .ok_or_else(|| work.exhausted())?;
    ensure_state_bytes(&work)?;
    let mut dependencies = Vec::new();
    dependencies
        .try_reserve_exact(dependency_count)
        .map_err(|_| work.exhausted())?;
    dependencies.extend(
        selection
            .edges
            .iter()
            .filter(|edge| affected_identities.contains(edge.dependent_identity.as_str()))
            .cloned(),
    );
    dependencies.sort();
    work.tick()?;
    let closure_identity = successor_closure_identity(successor)?;
    work.retained_bytes = work
        .retained_bytes
        .checked_add(closure_identity.as_str().len())
        .ok_or_else(|| work.exhausted())?;
    ensure_state_bytes(&work)?;
    let retained_bytes = work.retained_bytes;

    let mut usage = Usage {
        nodes,
        edges: selection.edges.len(),
        work: work.value,
        retained_results: selection.prior_results.len(),
        retained_bytes,
        output_bytes: 0,
    };
    let bytes = encode_plan(
        prior,
        successor,
        &changed_facts,
        ResultPartitions {
            affected: &affected,
            unaffected: &unaffected,
        },
        &dependencies,
        &schedule,
        effective.max_output_bytes,
    )?;
    usage.output_bytes = bytes.len();
    let identity = Identity::new(format!("sha256:{:x}", Sha256::digest(&bytes)));
    Ok(Plan {
        identity,
        bytes,
        closure_identity,
        changed_facts,
        affected,
        unaffected,
        dependencies,
        recomputation_order: schedule,
        usage,
    })
}

fn validate_revision_pair(prior: &View, successor: &View, selection: &Selection) -> Result<()> {
    if selection.authority_revision != *prior.identity() {
        return Err(Error::new(
            ErrorCode::ForeignRevision,
            "repair graph does not belong to the supplied prior bundle",
            AuthorityUsage::default(),
        ));
    }
    if prior.authority() != successor.authority()
        || prior.subject() != successor.subject()
        || successor.predecessor() != Some(prior.identity())
        || successor.revision() <= prior.revision()
    {
        return Err(Error::new(
            ErrorCode::ForeignRevision,
            "repair requires a direct same-authority bundle successor",
            AuthorityUsage::default(),
        ));
    }
    if successor.payload().replacements().len() == 0 {
        return Err(Error::new(
            ErrorCode::InvalidReplacement,
            "repair requires at least one declared bundle replacement",
            AuthorityUsage::default(),
        ));
    }
    Ok(())
}

fn validate_clock_domains(
    facts: &[FactRegion],
    results: &[PriorResult],
    work: &mut Work,
) -> Result<()> {
    let mut domains: BTreeMap<(&str, &str), &Region> = BTreeMap::new();
    for region in facts
        .iter()
        .map(FactRegion::region)
        .chain(results.iter().map(PriorResult::region))
    {
        work.tick()?;
        let key = (
            region.clock_identity.as_str(),
            region.clock_revision.as_str(),
        );
        if let Some(existing) = domains.get(&key) {
            if !region.has_same_clock_domain(existing) {
                return Err(Error::new(
                    ErrorCode::IntervalDomainMismatch,
                    "one clock revision has contradictory repair window domains",
                    work.authority_usage(0),
                ));
            }
        } else {
            domains.insert(key, region);
        }
    }
    Ok(())
}

fn validate_authority_regions(
    prior: &View,
    successor: &View,
    prior_components: &BTreeSet<&str>,
    successor_components: &BTreeSet<&str>,
    facts: &[FactRegion],
    results: &[PriorResult],
    work: &mut Work,
) -> Result<()> {
    let prior_clock = bundle_clock(prior)?;
    let successor_clock = bundle_clock(successor)?;
    for fact in facts {
        work.tick()?;
        let matches = if prior_components.contains(fact.identity.as_str()) {
            region_matches_authority(&fact.region, prior, prior_clock)
        } else if successor_components.contains(fact.identity.as_str()) {
            region_matches_authority(&fact.region, successor, successor_clock)
        } else {
            false
        };
        if !matches {
            return Err(Error::new(
                ErrorCode::AuthorityMismatch,
                "fact region is cross-wired from its bundle scope or clock authority",
                work.authority_usage(0),
            ));
        }
    }
    for result in results {
        work.tick()?;
        if !region_matches_authority(&result.region, prior, prior_clock) {
            return Err(Error::new(
                ErrorCode::AuthorityMismatch,
                "prior result region is cross-wired from prior bundle authority",
                work.authority_usage(0),
            ));
        }
    }
    Ok(())
}

fn bundle_clock(view: &View) -> Result<&ClockView> {
    let mut clocks = view
        .payload()
        .positions()
        .filter_map(|fact| match fact.payload() {
            EmbeddedPayloadRef::Clock(clock) => Some(clock),
            _ => None,
        });
    let Some(clock) = clocks.next() else {
        return Err(Error::new(
            ErrorCode::MissingPremise,
            "authority bundle has no exact clock binding",
            AuthorityUsage::default(),
        ));
    };
    if clocks.next().is_some() {
        return Err(Error::new(
            ErrorCode::AuthorityMismatch,
            "authority bundle has multiple clock bindings",
            AuthorityUsage::default(),
        ));
    }
    Ok(clock)
}

fn region_matches_authority(region: &Region, bundle: &View, clock: &ClockView) -> bool {
    if region.scope_identity.as_str() != bundle.subject().scope_identity.as_str()
        || region.clock_identity.as_str() != clock.clock_identity()
        || region.clock_revision.as_str() != clock.clock_revision()
    {
        return false;
    }
    match (&region.window, clock.selection()) {
        (
            Window::EventPosition {
                start,
                end_exclusive,
            },
            RangeRef::EventPosition {
                start: authority_start,
                end_exclusive: authority_end,
            },
        ) => contains_u64(authority_start, authority_end, *start, *end_exclusive),
        (
            Window::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            },
            RangeRef::FixedSample {
                start: authority_start,
                end_exclusive: authority_end,
                epoch_nanos: authority_epoch,
                period_nanos: authority_period,
            },
        ) => {
            authority_epoch.parse::<i128>() == Ok(*epoch_nanos)
                && authority_period.parse::<u64>() == Ok(*period_nanos)
                && contains_u64(authority_start, authority_end, *start, *end_exclusive)
        }
        (
            Window::Timestamp {
                start_nanos,
                end_nanos,
            },
            RangeRef::TimestampedEvent {
                start_nanos: authority_start,
                end_exclusive_nanos: authority_end,
            },
        ) => contains_i128(authority_start, authority_end, *start_nanos, *end_nanos),
        _ => false,
    }
}

fn contains_u64(authority_start: &str, authority_end: &str, start: u64, end: u64) -> bool {
    authority_start
        .parse::<u64>()
        .is_ok_and(|value| value <= start)
        && authority_end.parse::<u64>().is_ok_and(|value| end <= value)
}

fn contains_i128(authority_start: &str, authority_end: &str, start: i128, end: i128) -> bool {
    authority_start
        .parse::<i128>()
        .is_ok_and(|value| value <= start)
        && authority_end
            .parse::<i128>()
            .is_ok_and(|value| end <= value)
}

fn validate_acyclic(
    results: &BTreeMap<&str, &PriorResult>,
    edges: &[DependencyEdge],
    work: &mut Work,
) -> Result<()> {
    let mut indegree: BTreeMap<&str, usize> =
        results.keys().map(|identity| (*identity, 0)).collect();
    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in edges {
        work.tick()?;
        if results.contains_key(edge.source_identity.as_str()) {
            let entry = indegree
                .get_mut(edge.dependent_identity.as_str())
                .ok_or_else(|| invalid_dependency(work, "dependent result is absent"))?;
            *entry = entry.checked_add(1).ok_or_else(|| work.exhausted())?;
            outgoing
                .entry(edge.source_identity.as_str())
                .or_default()
                .push(edge.dependent_identity.as_str());
        }
    }
    let mut ready: BTreeSet<&str> = indegree
        .iter()
        .filter_map(|(identity, count)| (*count == 0).then_some(*identity))
        .collect();
    let mut visited = 0usize;
    while let Some(identity) = ready.pop_first() {
        work.tick()?;
        visited = visited.checked_add(1).ok_or_else(|| work.exhausted())?;
        if let Some(dependents) = outgoing.get(identity) {
            for dependent in dependents {
                work.tick()?;
                let entry = indegree
                    .get_mut(dependent)
                    .ok_or_else(|| invalid_dependency(work, "dependent result is absent"))?;
                *entry = entry
                    .checked_sub(1)
                    .ok_or_else(|| invalid_dependency(work, "dependency indegree underflow"))?;
                if *entry == 0 {
                    ready.insert(dependent);
                }
            }
        }
    }
    if visited != results.len() {
        return Err(Error::new(
            ErrorCode::DependencyCycle,
            "explicit dependency graph contains a directed cycle",
            work.authority_usage(0),
        ));
    }
    Ok(())
}

fn affected_closure<'a>(
    changed: &'a [ChangedFact],
    results: &BTreeMap<&'a str, &'a PriorResult>,
    outgoing: &BTreeMap<&'a str, Vec<&'a str>>,
    work: &mut Work,
) -> Result<BTreeSet<&'a str>> {
    let mut queue = VecDeque::new();
    let mut visited = BTreeSet::new();
    for fact in changed {
        let identity = fact.prior_identity.as_str();
        queue.push_back((identity, fact));
    }
    let mut affected = BTreeSet::new();
    while let Some((node, changed_fact)) = queue.pop_front() {
        work.tick()?;
        if !visited.insert((node, changed_fact.prior_identity.as_str())) {
            continue;
        }
        if let Some(dependents) = outgoing.get(node) {
            for dependent in dependents {
                work.tick()?;
                let result = results
                    .get(dependent)
                    .ok_or_else(|| invalid_dependency(work, "dependent result is absent"))?;
                if result.region.overlaps(&changed_fact.prior_region)
                    || result.region.overlaps(&changed_fact.successor_region)
                {
                    affected.insert(*dependent);
                }
                queue.push_back((dependent, changed_fact));
            }
        }
    }
    Ok(affected)
}

fn schedule_affected(
    affected: &BTreeSet<&str>,
    results: &BTreeMap<&str, &PriorResult>,
    edges: &[DependencyEdge],
    work: &mut Work,
) -> Result<Vec<Identity>> {
    let mut indegree: BTreeMap<&str, usize> =
        affected.iter().map(|identity| (*identity, 0)).collect();
    let mut outgoing: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for edge in edges {
        work.tick()?;
        if affected.contains(edge.source_identity.as_str())
            && affected.contains(edge.dependent_identity.as_str())
        {
            let entry = indegree
                .get_mut(edge.dependent_identity.as_str())
                .ok_or_else(|| invalid_dependency(work, "affected dependent is absent"))?;
            *entry = entry.checked_add(1).ok_or_else(|| work.exhausted())?;
            outgoing
                .entry(edge.source_identity.as_str())
                .or_default()
                .push(edge.dependent_identity.as_str());
        }
    }
    let mut ready = BTreeSet::new();
    for (identity, count) in &indegree {
        if *count == 0 {
            let result = results
                .get(identity)
                .ok_or_else(|| invalid_dependency(work, "affected result is absent"))?;
            ready.insert((result.region.clone(), *identity));
        }
    }
    let mut schedule = Vec::new();
    schedule
        .try_reserve_exact(affected.len())
        .map_err(|_| work.exhausted())?;
    while let Some((_, identity)) = ready.pop_first() {
        work.tick()?;
        schedule.push(Identity::new(identity));
        if let Some(dependents) = outgoing.get(identity) {
            for dependent in dependents {
                work.tick()?;
                let entry = indegree
                    .get_mut(dependent)
                    .ok_or_else(|| invalid_dependency(work, "affected dependent is absent"))?;
                *entry = entry
                    .checked_sub(1)
                    .ok_or_else(|| invalid_dependency(work, "affected indegree underflow"))?;
                if *entry == 0 {
                    let result = results
                        .get(dependent)
                        .ok_or_else(|| invalid_dependency(work, "affected result is absent"))?;
                    ready.insert((result.region.clone(), *dependent));
                }
            }
        }
    }
    if schedule.len() != affected.len() {
        return Err(Error::new(
            ErrorCode::DependencyCycle,
            "affected dependency graph contains a directed cycle",
            work.authority_usage(0),
        ));
    }
    Ok(schedule)
}

fn encode_plan(
    prior: &View,
    successor: &View,
    changed: &[ChangedFact],
    results: ResultPartitions<'_>,
    dependencies: &[DependencyEdge],
    schedule: &[Identity],
    output_limit: usize,
) -> Result<Vec<u8>> {
    let wire = PlanWire {
        contract: "quire.observation.repair-plan/v1",
        prior_bundle_identity: prior.identity().as_str(),
        successor_bundle_identity: successor.identity().as_str(),
        closure_identity: successor
            .payload()
            .components()
            .find(|component| component.role() == ComponentRole::Closure)
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::MissingPremise,
                    "successor bundle has no closure authority component",
                    AuthorityUsage::default(),
                )
            })?
            .identity(),
        changed_facts: changed
            .iter()
            .map(|fact| ChangedFactWire {
                role: fact.role,
                prior_identity: fact.prior_identity.as_str(),
                successor_identity: fact.successor_identity.as_str(),
                prior_region: region_wire(&fact.prior_region),
                successor_region: region_wire(&fact.successor_region),
            })
            .collect(),
        affected: results
            .affected
            .iter()
            .map(result_wire)
            .collect::<Result<_>>()?,
        unaffected: results
            .unaffected
            .iter()
            .map(result_wire)
            .collect::<Result<_>>()?,
        dependencies: dependencies
            .iter()
            .map(|edge| DependencyWire {
                source_identity: edge.source_identity.as_str(),
                dependent_identity: edge.dependent_identity.as_str(),
            })
            .collect(),
        recomputation_order: schedule.iter().map(Identity::as_str).collect(),
    };
    to_bounded_json(&wire, output_limit)
}

fn successor_closure_identity(successor: &View) -> Result<Identity> {
    successor
        .payload()
        .components()
        .find(|component| component.role() == ComponentRole::Closure)
        .map(|component| Identity::new(component.identity()))
        .ok_or_else(|| {
            Error::new(
                ErrorCode::MissingPremise,
                "successor bundle has no closure authority component",
                AuthorityUsage::default(),
            )
        })
}

#[derive(Clone, Copy)]
struct ResultPartitions<'a> {
    affected: &'a [PriorResult],
    unaffected: &'a [PriorResult],
}

fn result_wire(result: &PriorResult) -> Result<ResultWire<'_>> {
    Ok(ResultWire {
        identity: result.identity.as_str(),
        region: region_wire(&result.region),
        prior_digest: format!("sha256:{:x}", Sha256::digest(&result.bytes)),
        prior_bytes: u64::try_from(result.bytes.len()).map_err(|_| {
            Error::new(
                ErrorCode::ResourceIncomplete,
                "retained result byte count is not wire-representable",
                AuthorityUsage::default(),
            )
        })?,
    })
}

fn region_wire(region: &Region) -> RegionWire<'_> {
    let window = match region.window {
        Window::EventPosition {
            start,
            end_exclusive,
        } => WindowWire::EventPosition {
            start,
            end_exclusive,
        },
        Window::FixedSample {
            start,
            end_exclusive,
            epoch_nanos,
            period_nanos,
        } => WindowWire::FixedSample {
            start,
            end_exclusive,
            epoch_nanos: epoch_nanos.to_string(),
            period_nanos,
        },
        Window::Timestamp {
            start_nanos,
            end_nanos,
        } => WindowWire::Timestamp {
            start_nanos: start_nanos.to_string(),
            end_nanos: end_nanos.to_string(),
        },
    };
    RegionWire {
        scope_identity: region.scope_identity.as_str(),
        clock_identity: region.clock_identity.as_str(),
        clock_revision: region.clock_revision.as_str(),
        window,
    }
}

fn validate_external_identity(identity: &Identity) -> Result<()> {
    if !identity.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "repair graph identities must be non-empty",
            AuthorityUsage::default(),
        ));
    }
    Ok(())
}

fn region_identity_bytes(region: &Region) -> Option<usize> {
    region
        .scope_identity
        .as_str()
        .len()
        .checked_add(region.clock_identity.as_str().len())?
        .checked_add(region.clock_revision.as_str().len())
}

fn ensure_state_bytes(work: &Work) -> Result<()> {
    if work.retained_bytes > work.limits.max_retained_bytes {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "repair state exceeds the retained-byte limit",
            work.authority_usage(0),
        ));
    }
    Ok(())
}

fn invalid_dependency(work: &Work, detail: &'static str) -> Error {
    Error::new(
        ErrorCode::InvalidDependency,
        detail,
        work.authority_usage(0),
    )
}

fn resource_error(work: usize, detail: &'static str) -> Error {
    Error::new(
        ErrorCode::ResourceIncomplete,
        detail,
        AuthorityUsage {
            visited_fields: work,
            ..AuthorityUsage::default()
        },
    )
}
