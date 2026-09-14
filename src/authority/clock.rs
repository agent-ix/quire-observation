// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

//! `quire.observation.clock-binding/v1` owner artifact.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::{ClockRange, Identity};

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.clock-binding/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-clock-binding-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "c9a7b154a2c2d775ba71ba001842e6f0595e123c07b23a2bb37215c1f05de38d";

/// Exact clock identity and revision selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    clock_identity: Identity,
    clock_revision: Identity,
}

impl Selection {
    /// Constructs an explicit clock selection.
    #[must_use]
    pub const fn new(clock_identity: Identity, clock_revision: Identity) -> Self {
        Self {
            clock_identity,
            clock_revision,
        }
    }

    /// Returns the selected clock identity.
    #[must_use]
    pub const fn clock_identity(&self) -> &Identity {
        &self.clock_identity
    }

    /// Returns the selected clock revision.
    #[must_use]
    pub const fn clock_revision(&self) -> &Identity {
        &self.clock_revision
    }
}

/// Borrowed clock-range selection exposed by a validated clock view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeRef<'a> {
    /// Half-open event-position range.
    EventPosition {
        /// Canonical inclusive-start text.
        start: &'a str,
        /// Canonical exclusive-end text.
        end_exclusive: &'a str,
    },
    /// Half-open fixed-sample range.
    FixedSample {
        /// Canonical inclusive-start text.
        start: &'a str,
        /// Canonical exclusive-end text.
        end_exclusive: &'a str,
        /// Canonical epoch-nanoseconds text.
        epoch_nanos: &'a str,
        /// Canonical period-nanoseconds text.
        period_nanos: &'a str,
    },
    /// Half-open timestamped-event range.
    TimestampedEvent {
        /// Canonical inclusive-start nanoseconds text.
        start_nanos: &'a str,
        /// Canonical exclusive-end nanoseconds text.
        end_exclusive_nanos: &'a str,
    },
}

/// Read-only payload produced only inside a validated clock view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClockView {
    clock_identity: String,
    clock_revision: String,
    selection: ClockRangeWire,
}

impl ClockView {
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

    /// Returns the exact typed half-open clock range.
    #[must_use]
    pub fn selection(&self) -> RangeRef<'_> {
        match &self.selection {
            ClockRangeWire::EventPosition {
                start,
                end_exclusive,
            } => RangeRef::EventPosition {
                start,
                end_exclusive,
            },
            ClockRangeWire::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            } => RangeRef::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            },
            ClockRangeWire::TimestampedEvent {
                start_nanos,
                end_exclusive_nanos,
            } => RangeRef::TimestampedEvent {
                start_nanos,
                end_exclusive_nanos,
            },
        }
    }
}

/// Constructor-private validated clock view.
pub type View = Validated<ClockView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "family", rename_all = "kebab-case", deny_unknown_fields)]
enum ClockRangeWire {
    EventPosition {
        start: String,
        end_exclusive: String,
    },
    FixedSample {
        start: String,
        end_exclusive: String,
        epoch_nanos: String,
        period_nanos: String,
    },
    TimestampedEvent {
        start_nanos: String,
        end_exclusive_nanos: String,
    },
}

/// Derives canonical bounded clock-binding owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    if !selection.clock_identity.valid() || !selection.clock_revision.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "clock identity and revision must be explicit",
            Usage::default(),
        ));
    }
    let qualified = context.history().qualified();
    if qualified.scope().clock_identity != selection.clock_identity
        || qualified.scope().clock_revision != selection.clock_revision
        || qualified.records().iter().any(|record| {
            record.clock_identity != selection.clock_identity
                || record.clock_revision != selection.clock_revision
        })
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "record clock is cross-wired from the selected clock binding",
            Usage::default(),
        ));
    }
    let payload = ClockView {
        clock_identity: selection.clock_identity.as_str().to_owned(),
        clock_revision: selection.clock_revision.as_str().to_owned(),
        selection: range_wire(qualified.scope().range),
    };
    let usage = Usage {
        depth: 4,
        string_bytes: payload
            .clock_identity
            .len()
            .max(payload.clock_revision.len()),
        visited_fields: 22,
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical clock bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

fn range_wire(value: ClockRange) -> ClockRangeWire {
    match value {
        ClockRange::EventPosition {
            start,
            end_exclusive,
        } => ClockRangeWire::EventPosition {
            start: start.to_string(),
            end_exclusive: end_exclusive.to_string(),
        },
        ClockRange::FixedSample {
            start,
            end_exclusive,
            epoch_nanos,
            period_nanos,
        } => ClockRangeWire::FixedSample {
            start: start.to_string(),
            end_exclusive: end_exclusive.to_string(),
            epoch_nanos: epoch_nanos.to_string(),
            period_nanos: period_nanos.to_string(),
        },
        ClockRange::Timestamp {
            start_nanos,
            end_nanos,
        } => ClockRangeWire::TimestampedEvent {
            start_nanos: start_nanos.to_string(),
            end_exclusive_nanos: end_nanos.to_string(),
        },
    }
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
