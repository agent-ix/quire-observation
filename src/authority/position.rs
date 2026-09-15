// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! `quire.observation.position-ledger/v1` owner artifact.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::Identity;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.position-ledger/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-position-ledger-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "aac2fd7fc1b129e24afec397900647f3fe8977341b9907662a3852ac5430211a";

/// One explicit zero-based observation position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    position: u64,
    observation_identity: Identity,
}

impl Position {
    /// Constructs an exact position entry.
    #[must_use]
    pub const fn new(position: u64, observation_identity: Identity) -> Self {
        Self {
            position,
            observation_identity,
        }
    }
}

/// Exact ledger, clock, and total-order selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    ledger_identity: Identity,
    clock_identity: Identity,
    clock_revision: Identity,
    positions: Vec<Position>,
}

impl Selection {
    /// Constructs a complete position-ledger selection.
    #[must_use]
    pub const fn new(
        ledger_identity: Identity,
        clock_identity: Identity,
        clock_revision: Identity,
        positions: Vec<Position>,
    ) -> Self {
        Self {
            ledger_identity,
            clock_identity,
            clock_revision,
            positions,
        }
    }
}

/// Read-only payload produced only inside a validated position-ledger view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionLedgerView {
    ledger_identity: String,
    clock_identity: String,
    clock_revision: String,
    positions: Vec<PositionWire>,
}

impl PositionLedgerView {
    /// Returns the selected ledger identity.
    #[must_use]
    pub fn ledger_identity(&self) -> &str {
        &self.ledger_identity
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

    /// Iterates canonical position text and observation identity in order.
    #[must_use]
    pub fn positions(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.positions
            .iter()
            .map(|entry| (entry.position.as_str(), entry.observation_identity.as_str()))
    }
}

/// Constructor-private validated position-ledger view.
pub type View = Validated<PositionLedgerView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PositionWire {
    position: String,
    observation_identity: String,
}

/// Derives canonical bounded position-ledger owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    if !selection.ledger_identity.valid()
        || !selection.clock_identity.valid()
        || !selection.clock_revision.valid()
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "ledger and clock identities must be explicit",
            Usage::default(),
        ));
    }
    if selection.positions.len() > effective.max_positions {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "position population exceeds effective limit",
            Usage {
                positions: selection.positions.len(),
                ..Usage::default()
            },
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
            "position ledger clock is cross-wired from qualified history",
            Usage::default(),
        ));
    }
    let qualified_ids: BTreeSet<_> = qualified.records().iter().map(|r| &r.identity).collect();
    let mut seen = BTreeSet::new();
    let mut seen_positions = BTreeSet::new();
    for (expected, entry) in (0_u64..).zip(&selection.positions) {
        if !seen_positions.insert(entry.position) {
            return Err(Error::new(
                ErrorCode::AmbiguousOrder,
                "two observations share one declared position",
                Usage::default(),
            ));
        }
        if entry.position != expected
            || !entry.observation_identity.valid()
            || !qualified_ids.contains(&entry.observation_identity)
            || !seen.insert(&entry.observation_identity)
        {
            return Err(Error::new(
                ErrorCode::ExpectedMismatch,
                "positions must be distinct, zero-based, contiguous and qualified",
                Usage::default(),
            ));
        }
    }
    if selection.positions.len() != qualified.records().len() {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "position ledger must cover the complete qualified history",
            Usage::default(),
        ));
    }
    let payload = PositionLedgerView {
        ledger_identity: selection.ledger_identity.as_str().to_owned(),
        clock_identity: selection.clock_identity.as_str().to_owned(),
        clock_revision: selection.clock_revision.as_str().to_owned(),
        positions: selection
            .positions
            .iter()
            .map(|entry| PositionWire {
                position: entry.position.to_string(),
                observation_identity: entry.observation_identity.as_str().to_owned(),
            })
            .collect(),
    };
    let usage = Usage {
        depth: 5,
        string_bytes: selection
            .positions
            .iter()
            .map(|entry| entry.observation_identity.as_str().len())
            .max()
            .unwrap_or(0)
            .max(selection.ledger_identity.as_str().len())
            .max(selection.clock_identity.as_str().len())
            .max(selection.clock_revision.as_str().len()),
        positions: selection.positions.len(),
        visited_fields: 21usize.saturating_add(selection.positions.len().saturating_mul(2)),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical position bytes against independent selections.
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
