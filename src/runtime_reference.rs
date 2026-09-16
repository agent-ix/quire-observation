// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Authority-qualified runtime subject and relationship values.
//!
//! FCD owns static Producer interface admission and relationship declarations.
//! This module consumes that constructor-private capability and owns only the
//! exact runtime identities and comparisons required by FR-287 and FR-262.

use agent_ix_baseline_producer::{
    AdmittedBundleKey, AdmittedStaticBundle, RelationshipDeclaration,
};

/// The component of a runtime reference that was absent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceComponent {
    /// Producer-local subject-kind discriminator.
    SubjectKind,
    /// Producer-local subject object identity.
    SubjectIdentity,
    /// Producer-supplied runtime relationship identity.
    RelationshipIdentity,
    /// FCD relationship declaration identity.
    RelationshipDeclaration,
}

/// The authored endpoint whose kind is incompatible with its FCD declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointSide {
    /// Authored source endpoint.
    Source,
    /// Authored target endpoint.
    Target,
}

/// Typed refusal returned before an invalid runtime reference can exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceRefusal {
    /// A required opaque runtime component is empty.
    Empty(ReferenceComponent),
    /// A subject or relationship names an authority other than the selected producer.
    ForeignAuthority,
    /// Semantic object comparison crossed a producer-local kind boundary.
    WrongSubjectKind,
    /// No exact relationship declaration exists in the selected producer bundle.
    UnknownRelationshipDeclaration {
        /// Exact requested declaration identity.
        identity: String,
    },
    /// The two authored endpoints are the declaration's endpoints in reverse order.
    ReversedEndpoints,
    /// One authored endpoint has a kind other than its declared endpoint type.
    WrongEndpointKind {
        /// Exact incompatible endpoint.
        endpoint: EndpointSide,
    },
}

/// Non-empty producer-local subject-kind bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubjectKind(Vec<u8>);

impl SubjectKind {
    /// Retains the exact producer-local discriminator without interpretation.
    pub fn new(value: impl Into<Vec<u8>>) -> Result<Self, ReferenceRefusal> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReferenceRefusal::Empty(ReferenceComponent::SubjectKind));
        }
        Ok(Self(value))
    }

    /// Returns the exact producer-local bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Non-empty producer-local runtime object identity bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SubjectIdentity(Vec<u8>);

impl SubjectIdentity {
    /// Retains the exact producer-local identity without interpretation.
    pub fn new(value: impl Into<Vec<u8>>) -> Result<Self, ReferenceRefusal> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReferenceRefusal::Empty(ReferenceComponent::SubjectIdentity));
        }
        Ok(Self(value))
    }

    /// Returns the exact producer-local bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Complete FR-287 authority-qualified runtime subject key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QualifiedSubject {
    authority: AdmittedBundleKey,
    kind: SubjectKind,
    identity: SubjectIdentity,
}

impl QualifiedSubject {
    /// Constructs a subject under one admitted Producer interface authority.
    pub fn new(
        producer: &AdmittedStaticBundle,
        kind: SubjectKind,
        identity: SubjectIdentity,
    ) -> Self {
        Self {
            authority: producer.key().clone(),
            kind,
            identity,
        }
    }

    /// Returns the exact FCD authority key.
    #[must_use]
    pub const fn authority(&self) -> &AdmittedBundleKey {
        &self.authority
    }

    /// Returns the exact producer-local kind.
    #[must_use]
    pub const fn kind(&self) -> &SubjectKind {
        &self.kind
    }

    /// Returns the exact producer-local object identity.
    #[must_use]
    pub const fn identity(&self) -> &SubjectIdentity {
        &self.identity
    }

    /// Compares semantic object identity only within one authority and kind.
    pub fn same_object(&self, other: &Self) -> Result<bool, ReferenceRefusal> {
        if self.authority != other.authority {
            return Err(ReferenceRefusal::ForeignAuthority);
        }
        if self.kind != other.kind {
            return Err(ReferenceRefusal::WrongSubjectKind);
        }
        Ok(self.identity == other.identity)
    }

    pub(crate) fn belongs_to(&self, producer: &AdmittedStaticBundle) -> bool {
        self.authority == *producer.key()
    }
}

/// Non-empty producer-supplied runtime relationship identity bytes.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RelationshipIdentity(Vec<u8>);

impl RelationshipIdentity {
    /// Retains the exact relationship identity without interpretation.
    pub fn new(value: impl Into<Vec<u8>>) -> Result<Self, ReferenceRefusal> {
        let value = value.into();
        if value.is_empty() {
            return Err(ReferenceRefusal::Empty(
                ReferenceComponent::RelationshipIdentity,
            ));
        }
        Ok(Self(value))
    }

    /// Returns the exact producer-local bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One exact producer-qualified runtime relationship value.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Relationship {
    declaration_identity: String,
    authority: AdmittedBundleKey,
    interface_version: String,
    identity: RelationshipIdentity,
    source: QualifiedSubject,
    target: QualifiedSubject,
}

impl Relationship {
    /// Constructs and statically validates one runtime relationship.
    pub fn new(
        producer: &AdmittedStaticBundle,
        declaration_identity: impl Into<String>,
        identity: RelationshipIdentity,
        source: QualifiedSubject,
        target: QualifiedSubject,
    ) -> Result<Self, ReferenceRefusal> {
        let declaration_identity = declaration_identity.into();
        let declaration = resolve_declaration(producer, &declaration_identity)?;
        validate_endpoints(producer, declaration, &source, &target)?;
        Ok(Self {
            declaration_identity,
            authority: producer.key().clone(),
            interface_version: producer.interface_version().to_owned(),
            identity,
            source,
            target,
        })
    }

    /// Returns the selected FCD relationship declaration identity.
    #[must_use]
    pub fn declaration_identity(&self) -> &str {
        &self.declaration_identity
    }

    /// Returns the selected producer authority.
    #[must_use]
    pub const fn authority(&self) -> &AdmittedBundleKey {
        &self.authority
    }

    /// Returns the exact Producer interface version.
    #[must_use]
    pub fn interface_version(&self) -> &str {
        &self.interface_version
    }

    /// Returns the exact producer-supplied runtime identity.
    #[must_use]
    pub const fn identity(&self) -> &RelationshipIdentity {
        &self.identity
    }

    /// Returns the authored source endpoint.
    #[must_use]
    pub const fn source(&self) -> &QualifiedSubject {
        &self.source
    }

    /// Returns the authored target endpoint.
    #[must_use]
    pub const fn target(&self) -> &QualifiedSubject {
        &self.target
    }

    pub(crate) fn belongs_to(&self, producer: &AdmittedStaticBundle) -> bool {
        self.authority == *producer.key()
            && self.interface_version == producer.interface_version()
            && self.source.belongs_to(producer)
            && self.target.belongs_to(producer)
    }
}

/// One required relationship slot without a runtime relationship identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RequiredRelationship {
    declaration_identity: String,
    authority: AdmittedBundleKey,
    interface_version: String,
    source: QualifiedSubject,
    target: QualifiedSubject,
}

impl RequiredRelationship {
    /// Constructs and statically validates one required relationship slot.
    pub fn new(
        producer: &AdmittedStaticBundle,
        declaration_identity: impl Into<String>,
        source: QualifiedSubject,
        target: QualifiedSubject,
    ) -> Result<Self, ReferenceRefusal> {
        let declaration_identity = declaration_identity.into();
        let declaration = resolve_declaration(producer, &declaration_identity)?;
        validate_endpoints(producer, declaration, &source, &target)?;
        Ok(Self {
            declaration_identity,
            authority: producer.key().clone(),
            interface_version: producer.interface_version().to_owned(),
            source,
            target,
        })
    }

    /// Returns the selected FCD relationship declaration identity.
    #[must_use]
    pub fn declaration_identity(&self) -> &str {
        &self.declaration_identity
    }

    /// Returns the authored source endpoint.
    #[must_use]
    pub const fn source(&self) -> &QualifiedSubject {
        &self.source
    }

    /// Returns the authored target endpoint.
    #[must_use]
    pub const fn target(&self) -> &QualifiedSubject {
        &self.target
    }

    pub(crate) fn matches(&self, actual: &Relationship) -> bool {
        self.declaration_identity == actual.declaration_identity
            && self.authority == actual.authority
            && self.interface_version == actual.interface_version
            && self.source == actual.source
            && self.target == actual.target
    }

    pub(crate) fn conflicts_with(&self, actual: &Relationship) -> bool {
        self.declaration_identity == actual.declaration_identity
            && self.authority == actual.authority
            && self.interface_version == actual.interface_version
            && self.source == actual.source
            && self.target != actual.target
    }

    pub(crate) fn belongs_to(&self, producer: &AdmittedStaticBundle) -> bool {
        self.authority == *producer.key()
            && self.interface_version == producer.interface_version()
            && self.source.belongs_to(producer)
            && self.target.belongs_to(producer)
    }
}

fn resolve_declaration<'a>(
    producer: &'a AdmittedStaticBundle,
    identity: &str,
) -> Result<&'a RelationshipDeclaration, ReferenceRefusal> {
    if identity.is_empty() {
        return Err(ReferenceRefusal::Empty(
            ReferenceComponent::RelationshipDeclaration,
        ));
    }
    producer
        .relationships()
        .iter()
        .find(|declaration| declaration.relationship_identity == identity)
        .ok_or_else(|| ReferenceRefusal::UnknownRelationshipDeclaration {
            identity: identity.to_owned(),
        })
}

fn validate_endpoints(
    producer: &AdmittedStaticBundle,
    declaration: &RelationshipDeclaration,
    source: &QualifiedSubject,
    target: &QualifiedSubject,
) -> Result<(), ReferenceRefusal> {
    if !source.belongs_to(producer) || !target.belongs_to(producer) {
        return Err(ReferenceRefusal::ForeignAuthority);
    }
    let source_kind = source.kind.as_bytes();
    let target_kind = target.kind.as_bytes();
    let declared_source = declaration.source.type_identity.as_bytes();
    let declared_target = declaration.target.type_identity.as_bytes();
    if source_kind == declared_target
        && target_kind == declared_source
        && (declared_source != declared_target)
    {
        return Err(ReferenceRefusal::ReversedEndpoints);
    }
    if source_kind != declared_source {
        return Err(ReferenceRefusal::WrongEndpointKind {
            endpoint: EndpointSide::Source,
        });
    }
    if target_kind != declared_target {
        return Err(ReferenceRefusal::WrongEndpointKind {
            endpoint: EndpointSide::Target,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use agent_ix_baseline_producer::{DigestSelection, Revision};

    use super::*;

    fn authority() -> AdmittedBundleKey {
        AdmittedBundleKey {
            bundle_identity: "producer:a".into(),
            bundle_revision: Revision::producer("1"),
            digest: DigestSelection::canonical(format!("sha256:{}", "1".repeat(64))),
        }
    }

    fn subject(authority: AdmittedBundleKey) -> QualifiedSubject {
        QualifiedSubject {
            authority,
            kind: SubjectKind::new(b"kind:a".to_vec()).unwrap(),
            identity: SubjectIdentity::new(b"object:a".to_vec()).unwrap(),
        }
    }

    // Trace: TC-005, FR-005-AC-2
    #[test]
    fn every_authority_member_participates_in_the_subject_key() {
        let base = subject(authority());
        let mut mutations = Vec::new();

        let mut value = authority();
        value.bundle_identity = "producer:b".into();
        mutations.push(subject(value));

        let mut value = authority();
        value.bundle_revision.namespace = "revision:other".into();
        mutations.push(subject(value));

        let mut value = authority();
        value.bundle_revision.value = "2".into();
        mutations.push(subject(value));

        let mut value = authority();
        value.digest.algorithm = "other".into();
        mutations.push(subject(value));

        let mut value = authority();
        value.digest.domain = "digest:other".into();
        mutations.push(subject(value));

        let mut value = authority();
        value.digest.version = "2".into();
        mutations.push(subject(value));

        let mut value = authority();
        value.digest.value = format!("sha256:{}", "2".repeat(64));
        mutations.push(subject(value));

        mutations.push(QualifiedSubject {
            authority: authority(),
            kind: SubjectKind::new(b"kind:b".to_vec()).unwrap(),
            identity: base.identity.clone(),
        });
        mutations.push(QualifiedSubject {
            authority: authority(),
            kind: base.kind.clone(),
            identity: SubjectIdentity::new(b"object:b".to_vec()).unwrap(),
        });

        assert!(mutations.iter().all(|mutation| mutation != &base));
        assert!(mutations
            .iter()
            .take(7)
            .all(|mutation| base.same_object(mutation) == Err(ReferenceRefusal::ForeignAuthority)));
    }

    // Trace: TC-005, FR-005-AC-4
    #[test]
    fn producer_interface_version_participates_in_relationship_value_equality() {
        let subject = subject(authority());
        let base = Relationship {
            declaration_identity: "declaration:a".into(),
            authority: authority(),
            interface_version: "1.2.0".into(),
            identity: RelationshipIdentity::new(b"relationship:a".to_vec()).unwrap(),
            source: subject.clone(),
            target: subject,
        };
        let mut other = base.clone();
        other.interface_version = "1.1.0".into();
        assert_ne!(base, other);
    }
}
