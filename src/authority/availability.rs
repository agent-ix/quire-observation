// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

//! `quire.observation.result-availability/v1` owner artifact.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, ensure_sorted_unique, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::Identity;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.result-availability/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-result-availability-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "2e6c00d6dc94a00b0859b346dc12a14c7142735a565e35d15b85895d311e8416";

/// Closed result-availability assertion vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    /// Every exact required result identity is available.
    Available,
    /// The producer and contract are available, but required results are not all observed.
    NotYetObserved,
    /// The selected result producer is unavailable.
    ProducerUnavailable,
    /// The selected result contract is unavailable.
    ContractUnavailable,
}

/// Availability of an exact producer or contract dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyState {
    /// The exact dependency is available.
    Available,
    /// The exact dependency is unavailable.
    Unavailable,
}

/// Exact required/available result population and dependency selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    required_results: Vec<Identity>,
    available_results: Vec<Identity>,
    producer: DependencyState,
    contract: DependencyState,
}

impl Selection {
    /// Constructs an explicit result-availability selection.
    #[must_use]
    pub const fn new(
        required_results: Vec<Identity>,
        available_results: Vec<Identity>,
        producer: DependencyState,
        contract: DependencyState,
    ) -> Self {
        Self {
            required_results,
            available_results,
            producer,
            contract,
        }
    }
}

/// Read-only payload produced only inside a validated availability view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvailabilityView {
    required_results: Vec<String>,
    available_results: Vec<String>,
    state: State,
}

impl AvailabilityView {
    /// Returns the owner-derived availability state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Returns the exact sorted required-result population.
    #[must_use]
    pub fn required_results(&self) -> &[String] {
        &self.required_results
    }

    /// Returns the exact sorted available-result subset.
    #[must_use]
    pub fn available_results(&self) -> &[String] {
        &self.available_results
    }
}

/// Constructor-private validated result-availability view.
pub type View = Validated<AvailabilityView>;

/// Derives canonical bounded result-availability owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    ensure_sorted_unique(
        &selection.required_results,
        effective.max_population_entries,
    )?;
    ensure_sorted_unique(
        &selection.available_results,
        effective.max_population_entries,
    )?;
    if selection
        .available_results
        .iter()
        .any(|identity| selection.required_results.binary_search(identity).is_err())
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "available results must be a subset of the exact required population",
            Usage::default(),
        ));
    }
    let state = if selection.contract == DependencyState::Unavailable {
        State::ContractUnavailable
    } else if selection.producer == DependencyState::Unavailable {
        State::ProducerUnavailable
    } else if selection.available_results == selection.required_results {
        State::Available
    } else {
        State::NotYetObserved
    };
    let payload = AvailabilityView {
        required_results: strings(&selection.required_results),
        available_results: strings(&selection.available_results),
        state,
    };
    let usage = Usage {
        depth: 4,
        string_bytes: payload
            .required_results
            .iter()
            .chain(&payload.available_results)
            .map(String::len)
            .max()
            .unwrap_or(0),
        population_entries: payload
            .required_results
            .len()
            .max(payload.available_results.len()),
        visited_fields: 18usize
            .saturating_add(payload.required_results.len())
            .saturating_add(payload.available_results.len()),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical availability bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

fn strings(values: &[Identity]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.as_str().to_owned())
        .collect()
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
