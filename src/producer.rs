// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! This crate's own local Producer-interface 1.2 admission surface.
//!
//! `quire-observation` previously consumed `AdmittedStaticBundle`,
//! `StaticProducerBundle` and `RelationshipDeclaration` from the
//! `agent-ix-baseline-producer` crate (`filament-core-data`). That dependency
//! is dropped entirely: this module is a decoupled, locally owned replacement
//! that preserves this crate's own admission behavior exactly — the same
//! JSON input shape (a Producer interface 1.2 static bundle document), the
//! same public API shape (`AdmittedBundleKey`, `AdmittedStaticBundle`,
//! `RelationshipDeclaration`, `PRODUCER_INTERFACE_VERSION`), and the same
//! validation outcomes this crate relies on: a non-empty bundle identity and
//! revision, the exact `1.2.0` interface version, a bundle digest that
//! recomputes to its declared value, and relationship declarations resolvable
//! by identity.
//!
//! This is a straight decoupling, not a redesign of the Producer interface.
//! `filament-core-data` issue #144 proposes moving admission toward "Semantic
//! IR documents lifted from spec artifacts"; that is a separate, larger
//! architectural question for whoever owns this crate's long-term admission
//! design, and is out of scope here.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// The only Producer interface version this crate accepts.
pub const PRODUCER_INTERFACE_VERSION: &str = "1.2.0";

/// The namespace of every producer-object revision on the wire.
pub const PRODUCER_REVISION_NAMESPACE: &str = "filament-core-data/producer-object-revision-1";

/// The canonical producer-object digest domain.
pub const CANONICAL_JSON_DOMAIN: &str = "filament-canonical-json-1";
/// The authored digest-domain normalization revision this crate selects.
pub const DIGEST_DOMAIN_VERSION: &str = "1";

/// The largest static bundle document this crate parses.
const MAX_STATIC_DOCUMENT_BYTES: usize = 1_048_576;

/// A refusal returned before an invalid static bundle can be admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal(String);

impl Refusal {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Refusal {}

/// A producer-authored two-member namespaced revision.
///
/// A drop-in of the pinned consumer `Revision { namespace, value }` shape:
/// same two members, same meaning, now owned locally.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    /// The explicitly selected namespace; never defaulted and never inferred.
    pub namespace: String,
    /// The opaque revision value within that namespace.
    pub value: String,
}

impl Revision {
    /// Authors a revision under an explicitly named namespace.
    pub fn new(namespace: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            value: value.into(),
        }
    }

    /// Authors a producer-object revision.
    pub fn producer(value: impl Into<String>) -> Self {
        Self::new(PRODUCER_REVISION_NAMESPACE, value)
    }
}

/// A SHA-256 digest in this crate's wire spelling: `sha256:` followed by 64
/// lowercase hexadecimal digits.
///
/// `quire_canonical::Sha256Digest` exposes no public constructor from raw
/// bytes, so a digest declared on the wire cannot be reconstructed as one.
/// This small local wrapper holds the same 32 bytes on both sides of the
/// round trip: `from_sha256` reads them out of a freshly computed
/// [`quire_canonical::Sha256Digest`], and `parse` reads them back out of a
/// declared wire string, so the two can be compared byte-for-byte.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct WireDigest([u8; 32]);

impl WireDigest {
    fn from_sha256(digest: quire_canonical::Sha256Digest) -> Self {
        Self(*digest.as_bytes())
    }

    fn parse(text: &str) -> Result<Self, Refusal> {
        let hex = text
            .strip_prefix("sha256:")
            .ok_or_else(|| Refusal::new(format!("{text} is not sha256:<hex>")))?;
        if hex.len() != 64 {
            return Err(Refusal::new(format!(
                "{text} is not sha256: followed by 64 hexadecimal digits"
            )));
        }
        let mut bytes = [0_u8; 32];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let position = index * 2;
            *byte = u8::from_str_radix(&hex[position..position + 2], 16)
                .map_err(|_| Refusal::new(format!("{text} is not valid lowercase hexadecimal")))?;
        }
        Ok(Self(bytes))
    }
}

impl std::fmt::Display for WireDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("sha256:")?;
        self.0
            .iter()
            .try_for_each(|byte| write!(formatter, "{byte:02x}"))
    }
}

/// A producer-authored digest, folded to two wire members.
///
/// This is this crate's own wire shape: `Serialize` and `Deserialize` both
/// read and write the folded `{ label, value }` form, so a value this crate
/// emits round-trips back through this same type.
///
/// A *different* shape — reading a Producer interface 1.2 static-bundle
/// digest declaration, still `{ algorithm, domain, version, value }` (four
/// members, unchanged) — goes through `deserialize_bundle_digest` (below) instead,
/// at the one conversion site where a static bundle is parsed. `algorithm` is
/// never retained there: this crate's only digest algorithm is SHA-256,
/// hardcoded on the wire this crate itself emits. `domain` and `version` are
/// folded into one label joined as `"{domain}/{version}"` — for example
/// `domain: "filament-canonical-json-1"`, `version: "1"` becomes the label
/// `"filament-canonical-json-1/1"`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerDigest {
    /// `"{domain}/{version}"`, joined at the point this digest was read.
    pub label: String,
    /// `sha256:` followed by 64 lowercase hexadecimal digits.
    pub value: String,
}

impl ProducerDigest {
    /// Folds a separately authored `domain` and `version` into one label.
    fn from_domain_version(domain: &str, version: &str, value: impl Into<String>) -> Self {
        Self {
            label: format!("{domain}/{version}"),
            value: value.into(),
        }
    }

    /// Authors a canonical producer-object digest selection over an already computed value.
    fn canonical(value: impl Into<String>) -> Self {
        Self::from_domain_version(CANONICAL_JSON_DOMAIN, DIGEST_DOMAIN_VERSION, value)
    }
}

/// Reads a static-bundle digest declaration — `{ algorithm, domain, version,
/// value }`, unchanged from a Producer interface 1.2 document — and folds it
/// to a [`ProducerDigest`]. See [`ProducerDigest`]'s own documentation for the
/// join convention; `algorithm` is read and discarded.
fn deserialize_bundle_digest<'de, D>(deserializer: D) -> Result<ProducerDigest, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Wire {
        domain: String,
        version: String,
        value: String,
    }
    let wire = Wire::deserialize(deserializer)?;
    Ok(ProducerDigest::from_domain_version(
        &wire.domain,
        &wire.version,
        wire.value,
    ))
}

/// As [`deserialize_bundle_digest`], for an optional field.
fn deserialize_bundle_digest_opt<'de, D>(
    deserializer: D,
) -> Result<Option<ProducerDigest>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bundle_digest(deserializer).map(Some)
}

/// One independently identified relationship endpoint declaration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipEndpointDeclaration {
    /// The exported type identity.
    pub type_identity: String,
}

/// The Producer-declared relationship shape this crate resolves against.
///
/// A local replacement for FCD's `RelationshipDeclaration`, carrying only the
/// members this crate reads: the declaration identity and the two endpoints'
/// type identities. The document may declare a digest, ownership, semantics
/// and multiplicity per relationship; this crate does not read them and they
/// are ignored, not modeled.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipDeclaration {
    /// The declared relationship identity.
    pub relationship_identity: String,
    /// The declared source endpoint.
    pub source: RelationshipEndpointDeclaration,
    /// The declared target endpoint.
    pub target: RelationshipEndpointDeclaration,
}

/// The admitted-bundle key: identity, revision and digest together.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AdmittedBundleKey {
    /// The bundle identity.
    pub bundle_identity: String,
    /// The bundle's namespaced revision.
    pub bundle_revision: Revision,
    /// The bundle's producer-object digest.
    pub digest: ProducerDigest,
}

/// The declared configuration selection this crate reads from a bundle.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerConfiguration {
    /// The configuration identity.
    pub configuration_identity: String,
    /// The configuration digest.
    #[serde(deserialize_with = "deserialize_bundle_digest")]
    pub digest: ProducerDigest,
}

/// One admitted Producer interface 1.2 static bundle.
///
/// The only way to obtain one is [`StaticProducerBundle::admit`] or
/// [`StaticProducerBundle::admit_json`]: they construct and validate
/// indivisibly, and a refused admission yields no value of this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedStaticBundle {
    key: AdmittedBundleKey,
    interface_version: String,
    configuration: ProducerConfiguration,
    relationships: Vec<RelationshipDeclaration>,
}

impl AdmittedStaticBundle {
    /// The admitted-bundle key: identity, revision and digest together.
    #[must_use]
    pub const fn key(&self) -> &AdmittedBundleKey {
        &self.key
    }

    /// The bundle identity.
    #[must_use]
    pub fn bundle_identity(&self) -> &str {
        &self.key.bundle_identity
    }

    /// The bundle's namespaced revision.
    #[must_use]
    pub const fn bundle_revision(&self) -> &Revision {
        &self.key.bundle_revision
    }

    /// The bundle's producer-object digest.
    #[must_use]
    pub const fn digest(&self) -> &ProducerDigest {
        &self.key.digest
    }

    /// The admitted Producer interface version.
    #[must_use]
    pub fn interface_version(&self) -> &str {
        &self.interface_version
    }

    /// The declared configuration selection.
    #[must_use]
    pub const fn configuration(&self) -> &ProducerConfiguration {
        &self.configuration
    }

    /// The declared relationship shapes this crate resolves runtime relationships against.
    #[must_use]
    pub fn relationships(&self) -> &[RelationshipDeclaration] {
        &self.relationships
    }
}

/// A candidate Producer interface 1.2 static bundle, not yet validated.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticProducerBundle {
    /// The bundle's own identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_identity: Option<String>,
    /// The bundle's namespaced revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_revision: Option<Revision>,
    /// The bundle's own producer-object digest.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_bundle_digest_opt"
    )]
    pub digest: Option<ProducerDigest>,
    /// The declared Producer interface version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface_version: Option<String>,
    /// The declared configuration selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration: Option<ProducerConfiguration>,
    /// The declared relationship shapes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Vec<RelationshipDeclaration>>,
    /// Every other member this crate does not read (components, endpoints,
    /// correspondences, inventory, model, profile, static closure, ...),
    /// retained only so the document round-trips for digest recomputation.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl StaticProducerBundle {
    /// Constructs and validates one admitted static bundle, indivisibly.
    ///
    /// # Errors
    ///
    /// A [`Refusal`] naming the first missing or mismatched member.
    pub fn admit(self) -> Result<AdmittedStaticBundle, Refusal> {
        let bundle_identity = self
            .bundle_identity
            .clone()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| Refusal::new("bundle declares no bundleIdentity"))?;
        let bundle_revision = self
            .bundle_revision
            .clone()
            .filter(|revision| !revision.namespace.is_empty() && !revision.value.is_empty())
            .ok_or_else(|| Refusal::new("bundle declares no bundleRevision"))?;
        let interface_version = self
            .interface_version
            .clone()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| Refusal::new("bundle declares no interfaceVersion"))?;
        if interface_version != PRODUCER_INTERFACE_VERSION {
            return Err(Refusal::new(format!(
                "{bundle_identity} declares interfaceVersion {interface_version}, not {PRODUCER_INTERFACE_VERSION}"
            )));
        }
        let declared_digest = self
            .digest
            .clone()
            .ok_or_else(|| Refusal::new("bundle declares no digest"))?;
        let configuration = self
            .configuration
            .clone()
            .ok_or_else(|| Refusal::new("bundle declares no configuration"))?;
        let relationships = self.relationships.clone().unwrap_or_default();

        let recomputed = self.canonical_digest_selection()?;
        let declared_bytes = WireDigest::parse(&declared_digest.value)?;
        let recomputed_bytes = WireDigest::parse(&recomputed.value)?;
        if declared_digest.label != recomputed.label || declared_bytes != recomputed_bytes {
            return Err(Refusal::new(format!(
                "{bundle_identity} declares digest {}/{} but recomputes {}/{}",
                declared_digest.label, declared_digest.value, recomputed.label, recomputed.value
            )));
        }

        Ok(AdmittedStaticBundle {
            key: AdmittedBundleKey {
                bundle_identity,
                bundle_revision,
                digest: declared_digest,
            },
            interface_version,
            configuration,
            relationships,
        })
    }

    /// Constructs and validates one admitted static bundle from its JSON bytes.
    ///
    /// # Errors
    ///
    /// A [`Refusal`] when the document exceeds the maximum static document
    /// size, is not valid JSON in the declared shape, or fails [`Self::admit`].
    pub fn admit_json(bytes: &[u8]) -> Result<AdmittedStaticBundle, Refusal> {
        if bytes.len() > MAX_STATIC_DOCUMENT_BYTES {
            return Err(Refusal::new(format!(
                "static bundle of {} bytes exceeds the maximum of {MAX_STATIC_DOCUMENT_BYTES}",
                bytes.len()
            )));
        }
        let bundle: Self = serde_json::from_slice(bytes)
            .map_err(|error| Refusal::new(format!("static bundle is not valid JSON: {error}")))?;
        bundle.admit()
    }

    /// Recomputes this bundle's canonical producer-object digest, excluding
    /// its own top-level `digest` member.
    ///
    /// # Errors
    ///
    /// A [`Refusal`] when the bundle cannot be canonicalized.
    pub fn canonical_digest_selection(&self) -> Result<ProducerDigest, Refusal> {
        let mut value = serde_json::to_value(self)
            .map_err(|error| Refusal::new(format!("bundle could not be serialized: {error}")))?;
        if let Some(object) = value.as_object_mut() {
            object.remove("digest");
        }
        let limits = quire_canonical::Limits::new(
            MAX_STATIC_DOCUMENT_BYTES as u64,
            quire_canonical::Limits::MAX_DEPTH,
        )
        .expect("the static document byte and depth ceilings fit quire_canonical's bounds");
        let digest = quire_canonical::sha256(&value, limits)
            .map_err(|error| Refusal::new(format!("bundle canonicalization failed: {error}")))?;
        Ok(ProducerDigest::canonical(
            WireDigest::from_sha256(digest).to_string(),
        ))
    }
}
