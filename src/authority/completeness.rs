// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

//! `quire.observation.completeness-assertion/v1` owner artifact.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::Identity;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.completeness-assertion/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-completeness-assertion-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "b7c3c48d5f907cfb5bd7b4450e009b6c95f7d558095ece8b81d2ae1d5821bcf0";

/// Per-member input completeness status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FactStatus {
    /// The exact required observation is available.
    Available,
    /// The required observation is absent or incomplete.
    Incomplete,
    /// An admitted correction contradicts the retained fact.
    Contradicted,
}

/// Closed FR-287 aggregate completeness vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    /// Every exact required member is available.
    Complete,
    /// At least one required member is incomplete and none is contradicted.
    Incomplete,
    /// At least one required fact is contradicted.
    Contradicted,
}

/// One exact required-member completeness input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fact {
    member_identity: Identity,
    observation_identity: Option<Identity>,
    status: FactStatus,
}

impl Fact {
    /// Constructs one explicit member status.
    #[must_use]
    pub const fn new(
        member_identity: Identity,
        observation_identity: Option<Identity>,
        status: FactStatus,
    ) -> Self {
        Self {
            member_identity,
            observation_identity,
            status,
        }
    }
}

/// Exact boundary and complete per-member fact selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    boundary_identity: Identity,
    facts: Vec<Fact>,
}

impl Selection {
    /// Constructs a completeness selection; derivation validates exact coverage.
    #[must_use]
    pub const fn new(boundary_identity: Identity, facts: Vec<Fact>) -> Self {
        Self {
            boundary_identity,
            facts,
        }
    }
}

/// Borrowed per-member fact exposed by a validated completeness view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactRef<'a> {
    /// Exact required-member identity.
    pub member_identity: &'a str,
    /// Exact admitted observation identity, when present.
    pub observation_identity: Option<&'a str>,
    /// Owner-derived per-member status.
    pub status: FactStatus,
}

/// Read-only payload produced only inside a validated completeness view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletenessView {
    population_identity: String,
    boundary_identity: String,
    facts: Vec<FactWire>,
    state: State,
}

impl CompletenessView {
    /// Returns the exact FR-263 population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }

    /// Returns the exact selected boundary identity.
    #[must_use]
    pub fn boundary_identity(&self) -> &str {
        &self.boundary_identity
    }

    /// Iterates the complete per-member fact population in canonical order.
    pub fn facts(&self) -> impl ExactSizeIterator<Item = FactRef<'_>> {
        self.facts.iter().map(|fact| FactRef {
            member_identity: &fact.member_identity,
            observation_identity: fact.observation_identity.as_deref(),
            status: fact.status,
        })
    }

    /// Returns the owner-derived aggregate state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Returns the exact number of required-member facts.
    #[must_use]
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

/// Constructor-private validated completeness view.
pub type View = Validated<CompletenessView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactWire {
    member_identity: String,
    observation_identity: Option<String>,
    status: FactStatus,
}

/// Derives canonical bounded completeness-assertion owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    if !selection.boundary_identity.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "completeness boundary identity must be explicit",
            Usage::default(),
        ));
    }
    if selection.facts.len() > effective.max_population_entries {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "completeness population exceeds effective limit",
            Usage {
                population_entries: selection.facts.len(),
                ..Usage::default()
            },
        ));
    }
    let qualified = context.history().qualified();
    if selection.facts.len() != qualified.scope().required_member_identities.len()
        || selection
            .facts
            .iter()
            .map(|fact| &fact.member_identity)
            .ne(qualified.scope().required_member_identities.iter())
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "completeness facts must exactly cover the required population",
            Usage::default(),
        ));
    }
    let records = qualified.records();
    for fact in &selection.facts {
        if !fact.member_identity.valid()
            || fact
                .observation_identity
                .as_ref()
                .is_some_and(|identity| !identity.valid())
        {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "completeness fact contains an absent identity",
                Usage::default(),
            ));
        }
        let selected_record = qualified
            .scope()
            .members
            .iter()
            .find(|member| member.object_identity == fact.member_identity)
            .map(|member| &member.record_identity);
        let observed = fact.observation_identity.as_ref().is_some_and(|identity| {
            selected_record == Some(identity)
                && records.iter().any(|record| record.identity == *identity)
        });
        match fact.status {
            FactStatus::Available if !observed => {
                return Err(Error::new(
                    ErrorCode::ExpectedMismatch,
                    "available completeness fact lacks a qualified observation",
                    Usage::default(),
                ));
            }
            FactStatus::Incomplete if fact.observation_identity.is_some() => {
                return Err(Error::new(
                    ErrorCode::ExpectedMismatch,
                    "incomplete completeness fact cannot name an available observation",
                    Usage::default(),
                ));
            }
            FactStatus::Contradicted if !observed => {
                return Err(Error::new(
                    ErrorCode::ExpectedMismatch,
                    "contradiction must retain its admitted correction fact",
                    Usage::default(),
                ));
            }
            _ => {}
        }
    }
    let state = if selection
        .facts
        .iter()
        .any(|fact| fact.status == FactStatus::Contradicted)
    {
        State::Contradicted
    } else if selection
        .facts
        .iter()
        .any(|fact| fact.status == FactStatus::Incomplete)
    {
        State::Incomplete
    } else {
        State::Complete
    };
    let payload = CompletenessView {
        population_identity: qualified.scope().population_identity.as_str().to_owned(),
        boundary_identity: selection.boundary_identity.as_str().to_owned(),
        facts: selection
            .facts
            .iter()
            .map(|fact| FactWire {
                member_identity: fact.member_identity.as_str().to_owned(),
                observation_identity: fact
                    .observation_identity
                    .as_ref()
                    .map(|value| value.as_str().to_owned()),
                status: fact.status,
            })
            .collect(),
        state,
    };
    let usage = Usage {
        depth: 5,
        string_bytes: payload
            .facts
            .iter()
            .flat_map(|fact| {
                [
                    fact.member_identity.len(),
                    fact.observation_identity.as_ref().map_or(0, String::len),
                ]
            })
            .max()
            .unwrap_or(0)
            .max(payload.population_identity.len())
            .max(payload.boundary_identity.len()),
        population_entries: payload.facts.len(),
        visited_fields: 20usize.saturating_add(payload.facts.len().saturating_mul(3)),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical completeness bytes against independent selections.
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
