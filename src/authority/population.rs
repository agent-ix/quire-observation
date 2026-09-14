// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

//! `quire.observation.population/v2` owner artifact and FR-263/FR-264 identities.

use agent_ix_baseline_producer::{DigestSelection, Revision};
use serde::{Deserialize, Serialize};

use super::common::{
    self, build_document, digest_hex, ensure_sorted_unique, preflight, read_exact, to_bounded_json,
    Validated,
};
use super::{Context, Document, Error, ErrorCode, Result, Usage};
use crate::{AdmissionRequest, Digest, Identity, QualifiedObservation, ScopeKind};

pub use super::Limits;

/// Immutable population owner-contract label.
pub const CONTRACT: &str = "quire.observation.population/v2";
/// Immutable explicit-members input-contract label.
pub const MEMBERSHIP_CONTRACT: &str = "quire.observation.explicit-members/v1";
/// Pinned JSON Schema bytes for [`MEMBERSHIP_CONTRACT`].
pub const MEMBERSHIP_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-explicit-members-v1.schema.json");
/// Lowercase SHA-256 digest of [`MEMBERSHIP_SCHEMA_BYTES`].
pub const MEMBERSHIP_SCHEMA_SHA256: &str =
    "408c9d2908c3660d657cea574f52d8531e9ab749ec78c8d058fe5f163df23cce";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-population-v2.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "bceba1a2a69d150af05a7f7bea52769560849fd24a8582cb35b0937eee263c01";

/// Borrowed Producer-interface selection in a validated population.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProducerRef<'a> {
    /// Exact producer-bundle identity.
    pub identity: &'a str,
    /// Exact Producer-interface version.
    pub version: &'a str,
    /// Exact namespaced producer-bundle revision.
    pub revision: &'a Revision,
    /// Exact four-member producer-bundle digest selection.
    pub digest: &'a DigestSelection,
}

/// Borrowed immutable definition selection in a validated population.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefinitionRef<'a> {
    /// Exact definition identity.
    pub identity: &'a str,
    /// Lowercase SHA-256 digest text.
    pub digest: &'a str,
}

/// Borrowed exact FCD configuration selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfigurationRef<'a> {
    /// Exact configuration identity.
    pub identity: &'a str,
    /// Exact four-member configuration digest selection.
    pub digest: &'a DigestSelection,
}

/// Borrowed mutually exclusive scope selection in a validated population.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeRef<'a> {
    /// Selected immutable snapshot.
    Snapshot {
        /// Exact snapshot identity.
        identity: &'a str,
    },
    /// Selected half-open event-time window.
    EventTimeWindow {
        /// Exact window identity.
        identity: &'a str,
        /// Canonical inclusive-start nanoseconds text.
        start_nanos: &'a str,
        /// Canonical exclusive-end nanoseconds text.
        end_exclusive_nanos: &'a str,
    },
}

/// Read-only payload produced only inside a validated population view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationView {
    population_identity: String,
    producer: ProducerWire,
    membership_rule_identity: String,
    membership_digest: String,
    selection: SelectionWire,
    required_members: Vec<String>,
    observation_sources: Vec<String>,
    configuration: ConfigurationWire,
    closure: DefinitionWire,
    completeness_dependencies: Vec<String>,
    progress_dependencies: Vec<String>,
}

impl PopulationView {
    /// Returns the FR-263 population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }

    /// Returns the exact selected Producer-interface facts.
    #[must_use]
    pub fn producer(&self) -> ProducerRef<'_> {
        ProducerRef {
            identity: &self.producer.authority.bundle_identity,
            version: &self.producer.interface_version,
            revision: &self.producer.authority.bundle_revision,
            digest: &self.producer.authority.digest,
        }
    }

    /// Returns the FR-264 membership-rule identity.
    #[must_use]
    pub fn membership_rule_identity(&self) -> &str {
        &self.membership_rule_identity
    }

    /// Returns the exact membership-document digest text.
    #[must_use]
    pub fn membership_digest(&self) -> &str {
        &self.membership_digest
    }

    /// Returns the selected snapshot or event-time window.
    #[must_use]
    pub fn selection(&self) -> ScopeRef<'_> {
        match &self.selection {
            SelectionWire::Snapshot { identity } => ScopeRef::Snapshot { identity },
            SelectionWire::EventTimeWindow {
                identity,
                start_nanos,
                end_exclusive_nanos,
            } => ScopeRef::EventTimeWindow {
                identity,
                start_nanos,
                end_exclusive_nanos,
            },
        }
    }

    /// Returns the exact sorted required-member population.
    #[must_use]
    pub fn required_members(&self) -> &[String] {
        &self.required_members
    }

    /// Returns the exact sorted observation-source set.
    #[must_use]
    pub fn observation_sources(&self) -> &[String] {
        &self.observation_sources
    }

    /// Returns the selected configuration definition.
    #[must_use]
    pub fn configuration(&self) -> ConfigurationRef<'_> {
        ConfigurationRef {
            identity: &self.configuration.identity,
            digest: &self.configuration.digest,
        }
    }

    /// Returns the selected closure definition.
    #[must_use]
    pub fn closure(&self) -> DefinitionRef<'_> {
        DefinitionRef {
            identity: &self.closure.identity,
            digest: &self.closure.digest,
        }
    }

    /// Returns the exact sorted completeness-dependency set.
    #[must_use]
    pub fn completeness_dependencies(&self) -> &[String] {
        &self.completeness_dependencies
    }

    /// Returns the exact sorted progress-dependency set.
    #[must_use]
    pub fn progress_dependencies(&self) -> &[String] {
        &self.progress_dependencies
    }
}

/// Constructor-private validated population view.
pub type View = Validated<PopulationView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerWire {
    authority: ProducerAuthorityWire,
    interface_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProducerAuthorityWire {
    bundle_identity: String,
    bundle_revision: Revision,
    digest: DigestSelection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigurationWire {
    identity: String,
    digest: DigestSelection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefinitionWire {
    identity: String,
    digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum SelectionWire {
    Snapshot {
        identity: String,
    },
    EventTimeWindow {
        identity: String,
        start_nanos: String,
        end_exclusive_nanos: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MembershipPreimage {
    identity_version: &'static str,
    members: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MembershipWire {
    contract: String,
    identity: String,
    digest: String,
    members: Vec<String>,
}

/// Canonical immutable explicit-members document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipDocument {
    identity: Identity,
    digest: Digest,
    bytes: Vec<u8>,
}

impl MembershipDocument {
    /// Returns the derived FR-264 identity.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Returns the SHA-256 digest of the canonical document bytes.
    #[must_use]
    pub const fn digest(&self) -> Digest {
        self.digest
    }

    /// Returns the canonical immutable document bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Strict-reader output for the explicit-members input contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMembership(MembershipDocument, Vec<Identity>);

impl ValidatedMembership {
    /// Returns the validated canonical document.
    #[must_use]
    pub const fn document(&self) -> &MembershipDocument {
        &self.0
    }

    /// Returns the exact independently supplied member set.
    #[must_use]
    pub fn members(&self) -> &[Identity] {
        &self.1
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PopulationPreimage {
    identity_version: &'static str,
    producer: ProducerWire,
    membership_rule_identity: String,
    selection: SelectionWire,
    observation_sources: Vec<String>,
    configuration: ConfigurationWire,
    closure: DefinitionWire,
    completeness_dependencies: Vec<String>,
    progress_dependencies: Vec<String>,
}

/// Derives the FR-264 explicit-members identity and digest.
pub fn membership_identity(
    required_members: &[Identity],
    limits: Limits,
) -> Result<(Identity, Digest)> {
    let effective = limits.effective();
    ensure_sorted_unique(required_members, effective.max_population_entries)?;
    common::sha256_jcs(
        &MembershipPreimage {
            identity_version: "quire.observation.explicit-members-identity/v1-draft.1",
            members: strings(required_members),
        },
        effective,
    )
}

/// Derives canonical bounded explicit-members bytes and identity.
pub fn derive_membership(
    required_members: &[Identity],
    limits: Limits,
) -> Result<MembershipDocument> {
    let effective = limits.effective();
    let (identity, digest) = membership_identity(required_members, effective)?;
    let wire = MembershipWire {
        contract: MEMBERSHIP_CONTRACT.to_owned(),
        identity: identity.as_str().to_owned(),
        digest: digest_hex(&digest),
        members: strings(required_members),
    };
    let bytes = to_bounded_json(&wire, effective.max_output_bytes)?;
    let usage = preflight(
        &bytes,
        Limits {
            max_input_bytes: effective.max_output_bytes,
            ..effective
        },
    )?;
    if usage.population_entries != required_members.len() {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "membership work accounting is inconsistent",
            usage,
        ));
    }
    Ok(MembershipDocument {
        identity,
        digest,
        bytes,
    })
}

/// Strict-reads explicit-members bytes against an independent member set.
pub fn read_membership(
    bytes: &[u8],
    expected_members: &[Identity],
    limits: Limits,
) -> Result<ValidatedMembership> {
    let effective = limits.effective();
    let usage = preflight(bytes, effective)?;
    let wire: MembershipWire = serde_json::from_slice(bytes).map_err(|_| {
        Error::new(
            ErrorCode::InvalidJson,
            "membership is not the closed v1 contract",
            usage,
        )
    })?;
    if wire.contract != MEMBERSHIP_CONTRACT {
        return Err(Error::new(
            ErrorCode::ContractMismatch,
            "membership contract label is unknown",
            usage,
        ));
    }
    if to_bounded_json(&wire, effective.max_output_bytes)? != bytes {
        return Err(Error::new(
            ErrorCode::NonCanonical,
            "membership bytes are not canonical",
            usage,
        ));
    }
    let expected = derive_membership(expected_members, effective)?;
    if expected.bytes != bytes
        || expected.identity.as_str() != wire.identity
        || digest_hex(&expected.digest) != wire.digest
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "membership does not match independently supplied members",
            usage,
        ));
    }
    Ok(ValidatedMembership(expected, expected_members.to_vec()))
}

/// Derives the exact FR-263 population identity from an unqualified request.
pub fn population_identity(request: &AdmissionRequest, limits: Limits) -> Result<Identity> {
    let scope = &request.scope;
    let effective = limits.effective();
    if request.producer.interface_version() != crate::PRODUCER_INTERFACE_VERSION {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "population producer selection must be explicit and supported",
            Usage::default(),
        ));
    }
    validate_sets(scope, effective)?;
    let (membership_identity, membership_digest) =
        membership_identity(&scope.required_member_identities, effective)?;
    if scope.membership_rule_identity != membership_identity
        || scope.membership_digest != membership_digest
    {
        return Err(Error::new(
            ErrorCode::IdentityMismatch,
            "membership identity or digest does not match FR-264 preimage",
            Usage::default(),
        ));
    }
    let closure = closure_wire(scope)?;
    let preimage = PopulationPreimage {
        identity_version: "quire.observation.population-identity/v2",
        producer: ProducerWire {
            authority: ProducerAuthorityWire {
                bundle_identity: request.producer.bundle_identity().to_owned(),
                bundle_revision: request.producer.bundle_revision().clone(),
                digest: request.producer.digest().clone(),
            },
            interface_version: request.producer.interface_version().to_owned(),
        },
        membership_rule_identity: scope.membership_rule_identity.as_str().to_owned(),
        selection: selection_wire(scope)?,
        observation_sources: strings(&scope.observation_sources),
        configuration: ConfigurationWire {
            identity: request
                .producer
                .configuration()
                .configuration_identity
                .clone(),
            digest: request.producer.configuration().digest.clone(),
        },
        closure,
        completeness_dependencies: strings(&scope.completeness_dependencies),
        progress_dependencies: strings(&scope.progress_dependencies),
    };
    common::sha256_jcs(&preimage, effective).map(|value| value.0)
}

/// Atomically fills the owner-derived membership and population identities.
pub fn assign_request_identities(request: &mut AdmissionRequest, limits: Limits) -> Result<()> {
    let membership = derive_membership(&request.scope.required_member_identities, limits)?;
    let mut candidate = request.clone();
    candidate.scope.membership_rule_identity = membership.identity.clone();
    candidate.scope.membership_digest = membership.digest;
    candidate.scope.membership_document = membership.bytes.clone();
    let population = population_identity(&candidate, limits)?;
    request.scope.membership_rule_identity = membership.identity;
    request.scope.membership_digest = membership.digest;
    request.scope.membership_document = membership.bytes;
    request.scope.population_identity = population;
    Ok(())
}

/// Derives canonical bounded population owner bytes.
pub fn derive(context: Context<'_>, limits: Limits) -> Result<Document> {
    let qualified = context.history().qualified();
    let payload = payload_from_qualified(qualified, limits)?;
    let count = payload.required_members.len();
    let usage = Usage {
        depth: 5,
        string_bytes: max_string(&payload),
        population_entries: count,
        required_sources: payload.observation_sources.len(),
        visited_fields: 32usize
            .saturating_add(count)
            .saturating_add(payload.observation_sources.len())
            .saturating_add(payload.completeness_dependencies.len())
            .saturating_add(payload.progress_dependencies.len()),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical population bytes against qualified state.
pub fn read(bytes: &[u8], context: Context<'_>, limits: Limits) -> Result<View> {
    let expected = derive(context, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

fn payload_from_qualified(
    qualified: &QualifiedObservation,
    limits: Limits,
) -> Result<PopulationView> {
    let request = request_view(qualified);
    let expected = population_identity(&request, limits)?;
    if expected != qualified.scope().population_identity {
        return Err(Error::new(
            ErrorCode::IdentityMismatch,
            "qualified population identity does not match FR-263 preimage",
            Usage::default(),
        ));
    }
    Ok(PopulationView {
        population_identity: expected.as_str().to_owned(),
        producer: ProducerWire {
            authority: ProducerAuthorityWire {
                bundle_identity: qualified.producer().bundle_identity().to_owned(),
                bundle_revision: qualified.producer().bundle_revision().clone(),
                digest: qualified.producer().digest().clone(),
            },
            interface_version: qualified.producer().interface_version().to_owned(),
        },
        membership_rule_identity: qualified
            .scope()
            .membership_rule_identity
            .as_str()
            .to_owned(),
        membership_digest: digest_hex(&qualified.scope().membership_digest),
        selection: selection_wire(qualified.scope())?,
        required_members: strings(&qualified.scope().required_member_identities),
        observation_sources: strings(&qualified.scope().observation_sources),
        configuration: ConfigurationWire {
            identity: qualified
                .producer()
                .configuration()
                .configuration_identity
                .clone(),
            digest: qualified.producer().configuration().digest.clone(),
        },
        closure: closure_wire(qualified.scope())?,
        completeness_dependencies: strings(&qualified.scope().completeness_dependencies),
        progress_dependencies: strings(&qualified.scope().progress_dependencies),
    })
}

fn request_view(qualified: &QualifiedObservation) -> AdmissionRequest {
    AdmissionRequest {
        package: qualified.package().clone(),
        producer: qualified.producer().clone(),
        binding: qualified.binding().clone(),
        expected_subject: qualified.expected_subject().clone(),
        relationships: qualified.relationships().to_vec(),
        required_relationships: qualified.required_relationships().to_vec(),
        scope: qualified.scope().clone(),
        records: qualified.records().to_vec(),
        limits: qualified.limits(),
    }
}

fn validate_sets(scope: &crate::ScopeSelection, limits: Limits) -> Result<()> {
    ensure_sorted_unique(
        &scope.required_member_identities,
        limits.max_population_entries,
    )?;
    ensure_sorted_unique(&scope.observation_sources, limits.max_required_sources)?;
    ensure_sorted_unique(
        &scope.completeness_dependencies,
        limits.max_population_entries,
    )?;
    ensure_sorted_unique(&scope.progress_dependencies, limits.max_population_entries)?;
    Ok(())
}

fn selection_wire(scope: &crate::ScopeSelection) -> Result<SelectionWire> {
    match (&scope.kind, scope.range) {
        (ScopeKind::Snapshot { snapshot_identity }, _) if snapshot_identity.valid() => {
            Ok(SelectionWire::Snapshot {
                identity: snapshot_identity.as_str().to_owned(),
            })
        }
        (
            ScopeKind::Window { window_identity },
            crate::ClockRange::Timestamp {
                start_nanos,
                end_nanos,
            },
        ) if window_identity.valid() && start_nanos < end_nanos => {
            Ok(SelectionWire::EventTimeWindow {
                identity: window_identity.as_str().to_owned(),
                start_nanos: start_nanos.to_string(),
                end_exclusive_nanos: end_nanos.to_string(),
            })
        }
        _ => Err(Error::new(
            ErrorCode::InvalidSelection,
            "population scope identity and event-time window must be valid",
            Usage::default(),
        )),
    }
}

fn closure_wire(scope: &crate::ScopeSelection) -> Result<DefinitionWire> {
    let (Some(identity), Some(digest)) = (&scope.closure_identity, scope.closure_digest) else {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "population requires closure identity and digest",
            Usage::default(),
        ));
    };
    if !identity.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "population closure identity must be explicit",
            Usage::default(),
        ));
    }
    Ok(DefinitionWire {
        identity: identity.as_str().to_owned(),
        digest: digest_hex(&digest),
    })
}

fn strings(values: &[Identity]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.as_str().to_owned())
        .collect()
}

fn max_string(payload: &PopulationView) -> usize {
    payload
        .required_members
        .iter()
        .chain(&payload.observation_sources)
        .chain(&payload.completeness_dependencies)
        .chain(&payload.progress_dependencies)
        .map(String::len)
        .max()
        .unwrap_or(0)
        .max(payload.population_identity.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_digest_is_pinned() {
        assert_eq!(common::schema_sha256(SCHEMA_BYTES), SCHEMA_SHA256);
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
        assert_eq!(
            common::schema_sha256(MEMBERSHIP_SCHEMA_BYTES),
            MEMBERSHIP_SCHEMA_SHA256
        );
    }
}
