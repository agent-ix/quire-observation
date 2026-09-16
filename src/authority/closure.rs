// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! `quire.observation.closure-assertion/v1` owner artifact.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{
    boundary_wire, build_document, ensure_sorted_unique, read_exact, BoundaryWire, Validated,
};
use super::{
    BoundaryRef, Context, Document, Error, ErrorCode, OpenClosed, Result, TemporalBoundary, Usage,
};
use crate::Identity;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.closure-assertion/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-closure-assertion-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "8638fb9f4dd5a26d44ae9975388fc734670992914422e23c27eaae82149245f4";

/// Exact clock, sources, boundary, and independent closure selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    clock_identity: Identity,
    clock_revision: Identity,
    required_sources: Vec<Identity>,
    boundary: TemporalBoundary,
    state: OpenClosed,
}

impl Selection {
    /// Constructs a complete closure-authority selection.
    #[must_use]
    pub const fn new(
        clock_identity: Identity,
        clock_revision: Identity,
        required_sources: Vec<Identity>,
        boundary: TemporalBoundary,
        state: OpenClosed,
    ) -> Self {
        Self {
            clock_identity,
            clock_revision,
            required_sources,
            boundary,
            state,
        }
    }
}

/// Read-only payload produced only inside a validated closure view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClosureView {
    scope_identity: String,
    clock_identity: String,
    clock_revision: String,
    required_sources: Vec<String>,
    boundary: BoundaryWire,
    state: OpenClosed,
}

impl ClosureView {
    /// Returns independent closure state without deriving progress or truth.
    #[must_use]
    pub const fn state(&self) -> OpenClosed {
        self.state
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

    /// Returns the complete sorted required-source set.
    #[must_use]
    pub fn required_sources(&self) -> &[String] {
        &self.required_sources
    }

    /// Returns the exact FR-289 boundary mapping.
    #[must_use]
    pub fn boundary(&self) -> BoundaryRef<'_> {
        self.boundary.as_ref()
    }
}

/// Constructor-private validated closure view.
pub type View = Validated<ClosureView>;

/// Derives canonical bounded closure-assertion owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    ensure_sorted_unique(&selection.required_sources, effective.max_required_sources)?;
    selection.boundary.validate(selection.state)?;
    if !selection.clock_identity.valid() || !selection.clock_revision.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "closure clock identity must be explicit",
            Usage::default(),
        ));
    }
    let qualified = context.history().qualified();
    if qualified.scope().clock_identity != selection.clock_identity
        || qualified.scope().clock_revision != selection.clock_revision
        || !selection.boundary.matches_range(qualified.scope().range)
        || selection.required_sources != qualified.scope().observation_sources
        || qualified.records().iter().any(|record| {
            record.clock_identity != selection.clock_identity
                || record.clock_revision != selection.clock_revision
        })
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "closure boundary, source or clock is cross-wired",
            Usage::default(),
        ));
    }
    let payload = ClosureView {
        scope_identity: super::common::scope_identity(qualified.scope().kind.clone()),
        clock_identity: selection.clock_identity.as_str().to_owned(),
        clock_revision: selection.clock_revision.as_str().to_owned(),
        required_sources: selection
            .required_sources
            .iter()
            .map(|value| value.as_str().to_owned())
            .collect(),
        boundary: boundary_wire(selection.boundary),
        state: selection.state,
    };
    let usage = Usage {
        depth: 5,
        string_bytes: payload
            .required_sources
            .iter()
            .map(String::len)
            .max()
            .unwrap_or(0)
            .max(payload.scope_identity.len())
            .max(payload.clock_identity.len())
            .max(payload.clock_revision.len()),
        required_sources: payload.required_sources.len(),
        visited_fields: 25usize.saturating_add(payload.required_sources.len()),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical closure bytes against independent selections.
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
