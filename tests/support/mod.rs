// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

#![allow(dead_code)]

use agent_ix_baseline_producer::{AdmittedStaticBundle, StaticProducerBundle};
use quire_observation::{QualifiedSubject, SubjectIdentity, SubjectKind};

pub const ORDER_KIND: &str = "ix://agent-ix/commerce/type/Order";
pub const SHIPMENT_KIND: &str = "ix://agent-ix/commerce/type/Shipment";
pub const ORDER_SHIPMENT: &str = "ix://agent-ix/commerce/relationship/Order-shipment";
pub const ORDER_SUCCESSOR: &str = "ix://agent-ix/commerce/relationship/Order-successor";

pub fn producer() -> AdmittedStaticBundle {
    StaticProducerBundle::admit_json(include_bytes!("../fixtures/fcd-static-bundle-1.2.json"))
        .expect("the pinned FCD 1.2 fixture is admitted")
}

pub fn producer_with_bundle_identity(identity: &str) -> AdmittedStaticBundle {
    let mut document: serde_json::Value =
        serde_json::from_slice(include_bytes!("../fixtures/fcd-static-bundle-1.2.json"))
            .expect("the FCD fixture is JSON");
    document["bundleIdentity"] = identity.into();
    let mut bundle: StaticProducerBundle =
        serde_json::from_value(document).expect("the mutated fixture retains its shape");
    bundle.digest = Some(
        bundle
            .canonical_digest_selection()
            .expect("the mutated fixture has a declared canonicalization policy"),
    );
    bundle.admit().expect("the mutated FCD bundle is admitted")
}

pub fn subject(
    producer: &AdmittedStaticBundle,
    kind: impl Into<Vec<u8>>,
    identity: impl Into<Vec<u8>>,
) -> QualifiedSubject {
    QualifiedSubject::new(
        producer,
        SubjectKind::new(kind).expect("test subject kind is non-empty"),
        SubjectIdentity::new(identity).expect("test subject identity is non-empty"),
    )
}

pub fn order(producer: &AdmittedStaticBundle, identity: &str) -> QualifiedSubject {
    subject(producer, ORDER_KIND, identity)
}

pub fn shipment(producer: &AdmittedStaticBundle, identity: &str) -> QualifiedSubject {
    subject(producer, SHIPMENT_KIND, identity)
}
