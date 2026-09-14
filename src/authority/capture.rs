// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

//! `quire.observation.capture-environment/v1` owner artifact.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{build_document, read_exact, Validated};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::{Anchor, Identity, ValueState};

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "quire.observation.capture-environment/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-capture-environment-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "6d9260d3c4de65b5c3304debdb9baa7816aaea55949839924182d1db4c24c516";

/// One capture name bound to one exact observation and semantic binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    capture_identity: Identity,
    binding_identity: Identity,
    observation_identity: Identity,
}

impl Binding {
    /// Constructs one explicit capture binding.
    #[must_use]
    pub const fn new(
        capture_identity: Identity,
        binding_identity: Identity,
        observation_identity: Identity,
    ) -> Self {
        Self {
            capture_identity,
            binding_identity,
            observation_identity,
        }
    }
}

/// Exact trigger, anchor, and complete capture-binding selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    trigger_identity: Identity,
    anchor: Anchor,
    bindings: Vec<Binding>,
}

impl Selection {
    /// Constructs a complete capture-environment selection.
    #[must_use]
    pub const fn new(trigger_identity: Identity, anchor: Anchor, bindings: Vec<Binding>) -> Self {
        Self {
            trigger_identity,
            anchor,
            bindings,
        }
    }
}

/// Borrowed complete capture binding exposed by a validated view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingRef<'a> {
    /// Exact capture identity.
    pub capture_identity: &'a str,
    /// Exact semantic binding identity.
    pub binding_identity: &'a str,
    /// Exact admitted observation identity.
    pub observation_identity: &'a str,
    /// Exact value-type identity.
    pub value_type: &'a str,
    /// Canonical captured value text.
    pub canonical_value: &'a str,
    /// Exact unit identity.
    pub unit: &'a str,
}

/// Read-only payload produced only inside a validated capture view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureView {
    trigger_identity: String,
    anchor: AnchorWire,
    bindings: Vec<BindingWire>,
}

impl CaptureView {
    /// Returns the exact captured trigger identity.
    #[must_use]
    pub fn trigger_identity(&self) -> &str {
        &self.trigger_identity
    }

    /// Returns the exact capture anchor.
    #[must_use]
    pub fn anchor(&self) -> super::observation::AnchorRef<'_> {
        match &self.anchor {
            AnchorWire::EventPosition { position } => {
                super::observation::AnchorRef::EventPosition { position }
            }
            AnchorWire::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            } => super::observation::AnchorRef::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            },
            AnchorWire::Timestamp { instant_nanos } => {
                super::observation::AnchorRef::Timestamp { instant_nanos }
            }
        }
    }

    /// Iterates every complete typed capture binding in canonical order.
    #[must_use]
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = BindingRef<'_>> {
        self.bindings.iter().map(|binding| BindingRef {
            capture_identity: &binding.capture_identity,
            binding_identity: &binding.binding_identity,
            observation_identity: &binding.observation_identity,
            value_type: &binding.value_type,
            canonical_value: &binding.canonical_value,
            unit: &binding.unit,
        })
    }
}

/// Constructor-private validated capture-environment view.
pub type View = Validated<CaptureView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum AnchorWire {
    EventPosition {
        position: String,
    },
    FixedSample {
        index: String,
        epoch_nanos: String,
        period_nanos: String,
    },
    Timestamp {
        instant_nanos: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    capture_identity: String,
    binding_identity: String,
    observation_identity: String,
    value_type: String,
    canonical_value: String,
    unit: String,
}

/// Derives canonical bounded capture-environment owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    if !selection.trigger_identity.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "capture trigger identity must be explicit",
            Usage::default(),
        ));
    }
    if selection.bindings.len() > effective.max_capture_bindings {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "capture population exceeds effective limit",
            Usage {
                capture_bindings: selection.bindings.len(),
                ..Usage::default()
            },
        ));
    }
    if selection
        .bindings
        .iter()
        .any(|binding| !binding.capture_identity.valid())
        || selection
            .bindings
            .windows(2)
            .any(|pair| pair[0].capture_identity >= pair[1].capture_identity)
    {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "capture bindings must be sorted and distinct",
            Usage::default(),
        ));
    }
    let qualified = context.history().qualified();
    if selection.bindings.len() != qualified.records().len() {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "capture environment must cover the complete qualified record set",
            Usage::default(),
        ));
    }
    let mut bindings = Vec::new();
    let mut observed = BTreeSet::new();
    bindings
        .try_reserve(selection.bindings.len())
        .map_err(|_| {
            Error::new(
                ErrorCode::ResourceIncomplete,
                "capture allocation failed",
                Usage::default(),
            )
        })?;
    for selected in &selection.bindings {
        if !observed.insert(&selected.observation_identity) {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "capture observation identities must be distinct",
                Usage::default(),
            ));
        }
        let record = qualified
            .records()
            .iter()
            .find(|record| record.identity == selected.observation_identity)
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "capture record is not qualified",
                    Usage::default(),
                )
            })?;
        if record.binding_identity != selected.binding_identity
            || record.trigger_identity != selection.trigger_identity
            || record.anchor != selection.anchor
        {
            return Err(Error::new(
                ErrorCode::ExpectedMismatch,
                "capture binding, trigger or anchor is cross-wired",
                Usage::default(),
            ));
        }
        let ValueState::Present {
            value_type,
            canonical_value,
        } = &record.value
        else {
            return Err(Error::new(
                ErrorCode::ExpectedMismatch,
                "capture cannot contain an absent valuation",
                Usage::default(),
            ));
        };
        bindings.push(BindingWire {
            capture_identity: selected.capture_identity.as_str().to_owned(),
            binding_identity: record.binding_identity.as_str().to_owned(),
            observation_identity: record.identity.as_str().to_owned(),
            value_type: value_type.as_str().to_owned(),
            canonical_value: canonical_value.clone(),
            unit: record.unit.as_str().to_owned(),
        });
    }
    let payload = CaptureView {
        trigger_identity: selection.trigger_identity.as_str().to_owned(),
        anchor: anchor_wire(selection.anchor),
        bindings,
    };
    let usage = Usage {
        depth: 5,
        string_bytes: payload
            .bindings
            .iter()
            .flat_map(|binding| {
                [
                    binding.binding_identity.len(),
                    binding.capture_identity.len(),
                    binding.observation_identity.len(),
                    binding.value_type.len(),
                    binding.canonical_value.len(),
                    binding.unit.len(),
                ]
            })
            .max()
            .unwrap_or(0)
            .max(payload.trigger_identity.len()),
        capture_bindings: payload.bindings.len(),
        visited_fields: 20usize.saturating_add(payload.bindings.len().saturating_mul(6)),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical capture bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

fn anchor_wire(value: Anchor) -> AnchorWire {
    match value {
        Anchor::EventPosition(position) => AnchorWire::EventPosition {
            position: position.to_string(),
        },
        Anchor::FixedSample {
            index,
            epoch_nanos,
            period_nanos,
        } => AnchorWire::FixedSample {
            index: index.to_string(),
            epoch_nanos: epoch_nanos.to_string(),
            period_nanos: period_nanos.to_string(),
        },
        Anchor::TimestampNanos(instant_nanos) => AnchorWire::Timestamp {
            instant_nanos: instant_nanos.to_string(),
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
