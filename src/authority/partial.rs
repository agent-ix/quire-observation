// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! `quire.observation.partial-fact/v1` owner artifact and exact interval algebra.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::Identity;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.partial-fact/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-partial-fact-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "9dc068376a7db5a7509810910c36e2efff6b9c3701d83c474701ec572f13f23c";

/// Closed explanation for a Boolean possibility set.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValueReason {
    /// Independently qualified evidence establishes one Boolean value.
    Known,
    /// Required evidence is absent, so both Boolean values remain possible.
    Missing,
    /// Qualified evidence conflicts, so both Boolean values remain possible.
    Conflicting,
}

/// A coherent, nonempty subset of the Boolean domain and its exact reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PossibilitySet {
    can_be_false: bool,
    can_be_true: bool,
    reason: ValueReason,
}

impl PossibilitySet {
    /// Constructs a nonempty Boolean possibility set coherent with `reason`.
    pub fn new(can_be_false: bool, can_be_true: bool, reason: ValueReason) -> Result<Self> {
        let cardinality = u8::from(can_be_false) + u8::from(can_be_true);
        let coherent = match reason {
            ValueReason::Known => cardinality == 1,
            ValueReason::Missing | ValueReason::Conflicting => cardinality == 2,
        };
        if !coherent {
            return Err(Error::new(
                ErrorCode::InvalidPossibilitySet,
                "Boolean possibility set is empty or inconsistent with its reason",
                Usage::default(),
            ));
        }
        Ok(Self {
            can_be_false,
            can_be_true,
            reason,
        })
    }

    /// Returns whether false remains possible.
    #[must_use]
    pub const fn can_be_false(self) -> bool {
        self.can_be_false
    }

    /// Returns whether true remains possible.
    #[must_use]
    pub const fn can_be_true(self) -> bool {
        self.can_be_true
    }

    /// Returns the exact evidence reason.
    #[must_use]
    pub const fn reason(self) -> ValueReason {
        self.reason
    }
}

/// Exact closed event-time interval under one clock domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventTimeInterval {
    clock_identity: Identity,
    clock_revision: Identity,
    unit: Identity,
    earliest: i128,
    latest: i128,
}

impl EventTimeInterval {
    /// Constructs a closed interval with explicit, nonempty domain identities.
    pub fn new(
        clock_identity: Identity,
        clock_revision: Identity,
        unit: Identity,
        earliest: i128,
        latest: i128,
        limits: Limits,
    ) -> Result<Self> {
        if !clock_identity.valid() || !clock_revision.valid() || !unit.valid() || earliest > latest
        {
            return Err(Error::new(
                ErrorCode::InvalidInterval,
                "event-time interval identities must be explicit and endpoints ordered",
                Usage::default(),
            ));
        }
        let interval = Self {
            clock_identity,
            clock_revision,
            unit,
            earliest,
            latest,
        };
        interval.validate_limits(limits)?;
        Ok(interval)
    }

    /// Returns the exact clock identity.
    #[must_use]
    pub const fn clock_identity(&self) -> &Identity {
        &self.clock_identity
    }

    /// Returns the exact clock revision.
    #[must_use]
    pub const fn clock_revision(&self) -> &Identity {
        &self.clock_revision
    }

    /// Returns the exact clock unit.
    #[must_use]
    pub const fn unit(&self) -> &Identity {
        &self.unit
    }

    /// Returns the inclusive earliest endpoint.
    #[must_use]
    pub const fn earliest(&self) -> i128 {
        self.earliest
    }

    /// Returns the inclusive latest endpoint.
    #[must_use]
    pub const fn latest(&self) -> i128 {
        self.latest
    }

    /// Returns every and only order relation admitted by two closed intervals.
    pub fn relation_to(&self, other: &Self, limits: Limits) -> Result<OrderPossibilities> {
        self.validate_limits(limits)?;
        other.validate_limits(limits)?;
        if self.clock_identity != other.clock_identity
            || self.clock_revision != other.clock_revision
            || self.unit != other.unit
        {
            return Err(Error::new(
                ErrorCode::IntervalDomainMismatch,
                "event-time intervals use different clock, revision, or unit domains",
                Usage::default(),
            ));
        }
        Ok(OrderPossibilities {
            before: self.earliest < other.latest,
            equal: self.earliest <= other.latest && other.earliest <= self.latest,
            after: self.latest > other.earliest,
        })
    }

    fn validate_limits(&self, limits: Limits) -> Result<()> {
        let string_bytes = self
            .clock_identity
            .as_str()
            .len()
            .max(self.clock_revision.as_str().len())
            .max(self.unit.as_str().len())
            .max(self.earliest.to_string().len())
            .max(self.latest.to_string().len());
        let usage = Usage {
            string_bytes,
            ..Usage::default()
        };
        if string_bytes > limits.effective().max_string_bytes {
            return Err(Error::new(
                ErrorCode::ResourceIncomplete,
                "event-time interval exceeds the effective string bound",
                usage,
            ));
        }
        Ok(())
    }
}

/// Exact nonempty subset of before/equal/after admitted by two intervals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrderPossibilities {
    before: bool,
    equal: bool,
    after: bool,
}

impl OrderPossibilities {
    /// Returns whether a selected instant from the left interval may be before one from the right.
    #[must_use]
    pub const fn before(self) -> bool {
        self.before
    }

    /// Returns whether the intervals admit equal selected instants.
    #[must_use]
    pub const fn equal(self) -> bool {
        self.equal
    }

    /// Returns whether a selected instant from the left interval may be after one from the right.
    #[must_use]
    pub const fn after(self) -> bool {
        self.after
    }

    /// Returns true only when every admitted selection places left before right.
    #[must_use]
    pub const fn definitely_before(self) -> bool {
        self.before && !self.equal && !self.after
    }
}

/// Exact inputs used to derive one partial-observation owner fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    observation_identity: Identity,
    possibilities: PossibilitySet,
    interval: EventTimeInterval,
    ingestion_position: i128,
}

impl Selection {
    /// Constructs an explicit partial-observation selection.
    #[must_use]
    pub const fn new(
        observation_identity: Identity,
        possibilities: PossibilitySet,
        interval: EventTimeInterval,
        ingestion_position: i128,
    ) -> Self {
        Self {
            observation_identity,
            possibilities,
            interval,
            ingestion_position,
        }
    }

    /// Compares this fact's interval with another without using ingestion position.
    pub fn relation_to(&self, other: &Self, limits: Limits) -> Result<OrderPossibilities> {
        self.interval.relation_to(&other.interval, limits)
    }
}

/// Borrowed exact interval exposed by a validated partial fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntervalRef<'a> {
    /// Exact clock identity.
    pub clock_identity: &'a str,
    /// Exact clock revision.
    pub clock_revision: &'a str,
    /// Exact unit identity.
    pub unit: &'a str,
    /// Canonical inclusive earliest endpoint.
    pub earliest: &'a str,
    /// Canonical inclusive latest endpoint.
    pub latest: &'a str,
}

/// Read-only payload produced only inside a validated partial-fact view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialView {
    observation_identity: String,
    possibilities: PossibilityWire,
    interval: IntervalWire,
    ingestion_position: String,
}

impl PartialView {
    /// Returns the exact admitted observation identity.
    #[must_use]
    pub fn observation_identity(&self) -> &str {
        &self.observation_identity
    }

    /// Returns whether false remains possible.
    #[must_use]
    pub const fn can_be_false(&self) -> bool {
        self.possibilities.can_be_false
    }

    /// Returns whether true remains possible.
    #[must_use]
    pub const fn can_be_true(&self) -> bool {
        self.possibilities.can_be_true
    }

    /// Returns the exact partial-value reason.
    #[must_use]
    pub const fn reason(&self) -> ValueReason {
        self.possibilities.reason
    }

    /// Returns the exact closed event-time interval.
    #[must_use]
    pub fn interval(&self) -> IntervalRef<'_> {
        IntervalRef {
            clock_identity: &self.interval.clock_identity,
            clock_revision: &self.interval.clock_revision,
            unit: &self.interval.unit,
            earliest: &self.interval.earliest,
            latest: &self.interval.latest,
        }
    }

    /// Returns the retained ingestion position, which is not an order input.
    #[must_use]
    pub fn ingestion_position(&self) -> &str {
        &self.ingestion_position
    }
}

/// Constructor-private validated partial-observation view.
pub type View = Validated<PartialView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PossibilityWire {
    can_be_false: bool,
    can_be_true: bool,
    reason: ValueReason,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IntervalWire {
    clock_identity: String,
    clock_revision: String,
    unit: String,
    earliest: String,
    latest: String,
}

/// Derives canonical bounded partial-observation owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    if !selection.observation_identity.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "observation identity must be explicit",
            Usage::default(),
        ));
    }
    selection.interval.validate_limits(limits)?;
    let record = context
        .history()
        .qualified()
        .records()
        .iter()
        .find(|record| record.identity == selection.observation_identity)
        .ok_or_else(|| {
            Error::new(
                ErrorCode::ExpectedMismatch,
                "partial fact observation is absent from qualified history",
                Usage::default(),
            )
        })?;
    if record.clock_identity != selection.interval.clock_identity
        || record.clock_revision != selection.interval.clock_revision
        || record.unit != selection.interval.unit
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "partial fact interval is cross-wired from the admitted observation",
            Usage::default(),
        ));
    }

    let payload = PartialView {
        observation_identity: selection.observation_identity.as_str().to_owned(),
        possibilities: PossibilityWire {
            can_be_false: selection.possibilities.can_be_false,
            can_be_true: selection.possibilities.can_be_true,
            reason: selection.possibilities.reason,
        },
        interval: IntervalWire {
            clock_identity: selection.interval.clock_identity.as_str().to_owned(),
            clock_revision: selection.interval.clock_revision.as_str().to_owned(),
            unit: selection.interval.unit.as_str().to_owned(),
            earliest: selection.interval.earliest.to_string(),
            latest: selection.interval.latest.to_string(),
        },
        ingestion_position: selection.ingestion_position.to_string(),
    };
    let usage = Usage {
        depth: 4,
        string_bytes: payload
            .observation_identity
            .len()
            .max(payload.interval.clock_identity.len())
            .max(payload.interval.clock_revision.len())
            .max(payload.interval.unit.len())
            .max(payload.interval.earliest.len())
            .max(payload.interval.latest.len())
            .max(payload.ingestion_position.len()),
        visited_fields: 24,
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical partial-observation bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
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
