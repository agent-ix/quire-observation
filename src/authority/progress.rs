// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! `quire.observation.progress-assertion/v1` owner artifact.

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

use super::observation::CutoffSelection;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.progress-assertion/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-progress-assertion-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "91223fb982b68d4fb7c66b3737370429bd4841321f5b561ebc6c1e9691bd7f1c";

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
        }
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
            .max(payload.restoration_basis_identity.len()),
        required_sources: payload.required_sources.len(),
        visited_fields: 29usize.saturating_add(payload.required_sources.len()),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_digest_is_pinned() {
        assert_eq!(common::schema_sha256(SCHEMA_BYTES), SCHEMA_SHA256);
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
    }
}
