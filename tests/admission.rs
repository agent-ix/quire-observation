// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-001 and TC-005: admission-request validation and relationship
//! correlation.

mod support;

use ix_trace_rs::trace;
use quire_observation::*;
use support::{order, producer, producer_with_bundle_identity, shipment, ORDER_SHIPMENT};

fn id(value: &str) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn seal(mut request: AdmissionRequest) -> AdmissionRequest {
    let identity = authority::observation::record_identity(
        &request.records[0],
        authority::Limits::owner_max(),
    )
    .unwrap();
    request.records[0].identity = identity.clone();
    request.scope.members[0].record_identity = identity;
    authority::population::assign_request_identities(&mut request, authority::Limits::owner_max())
        .unwrap();
    request
}

fn request() -> AdmissionRequest {
    let producer = producer();
    let order = order(&producer, "order:O1");
    let request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.into(),
            identity: id("package:checkout"),
            revision: id("1-draft"),
            digest: digest(1),
        },
        binding: ObservationBinding {
            identity: id("binding:payment-effect"),
            source_identity: id("source:provider"),
            schema_identity: id("schema:payment/v1"),
            signal_identity: id("signal:effect"),
            trigger_identity: id("trigger:payment"),
            unit: id("USD"),
            subject_kind: order.kind().clone(),
            required: true,
        },
        expected_subject: order.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id("unsealed-membership"),
            membership_digest: digest(0),
            membership_document: vec![],
            required_member_identities: vec![id("member:O1")],
            observation_sources: vec![id("source:provider")],
            completeness_dependencies: vec![id("completeness:orders")],
            progress_dependencies: vec![id("progress:orders")],
            clock_identity: id("clock:positions"),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id("closure:complete")),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Snapshot {
                snapshot_identity: id("snapshot:1"),
            },
            range: ClockRange::EventPosition {
                start: 0,
                end_exclusive: 1,
            },
            members: vec![Member {
                object_identity: id("member:O1"),
                record_identity: id("record:1"),
                anchor: Anchor::EventPosition(0),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("record:1"),
            binding_identity: id("binding:payment-effect"),
            source_identity: id("source:provider"),
            schema_identity: id("schema:payment/v1"),
            subject: order,
            signal_identity: id("signal:effect"),
            trigger_identity: id("trigger:payment"),
            unit: id("USD"),
            value: ValueState::Present {
                value_type: id("decimal"),
                canonical_value: "12.00".into(),
            },
            visibility: Visibility::External,
            anchor: Anchor::EventPosition(0),
            event_time_nanos: 0,
            ingestion_time_nanos: 1,
            causal_relationship_identity: None,
            clock_identity: id("clock:positions"),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 0,
        }],
        limits: ResourceLimits {
            max_records: 2,
            max_members: 2,
            max_relationships: 2,
            max_required_relationships: 2,
        },
        producer,
    };
    seal(request)
}

fn relationship(
    input: &AdmissionRequest,
    identity: &str,
    source: QualifiedSubject,
    target: QualifiedSubject,
) -> Relationship {
    Relationship::new(
        &input.producer,
        ORDER_SHIPMENT,
        RelationshipIdentity::new(identity).unwrap(),
        source,
        target,
    )
    .unwrap()
}

fn required(
    input: &AdmissionRequest,
    source: QualifiedSubject,
    target: QualifiedSubject,
) -> RequiredRelationship {
    RequiredRelationship::new(&input.producer, ORDER_SHIPMENT, source, target).unwrap()
}

#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-7", "TC-005", "FR-005-AC-1")]
#[test]
fn tc001_valid_record_retains_all_selected_premises() {
    let outcome = admit(request());
    assert!(
        matches!(outcome, AdmissionOutcome::Available { observation }
            if observation.records()[0].visibility == Visibility::External
                && observation.package().identity == id("package:checkout")
                && observation.producer().interface_version() == PRODUCER_INTERFACE_VERSION
                && observation.producer().configuration().configuration_identity
                    == "ix://agent-ix/commerce/config/evaluation-default"
                && observation.scope().population_identity.as_str().starts_with("sha256-jcs:")
                && observation.scope().closure_identity == Some(id("closure:complete"))
                && observation.scope().closure_digest == Some(digest(5))
                && observation.limits().max_records == 2)
    );
}

#[trace("TC-001", "FR-001-AC-2")]
#[test]
fn tc001_wrong_signal_unit_and_schema_refuse() {
    for (name, field) in [
        ("signal", BindingField::Signal),
        ("trigger", BindingField::Trigger),
        ("unit", BindingField::Unit),
        ("schema", BindingField::Schema),
    ] {
        let mut input = request();
        match name {
            "signal" => input.records[0].signal_identity = id("signal:wrong"),
            "trigger" => input.records[0].trigger_identity = id("trigger:wrong"),
            "unit" => input.records[0].unit = id("EUR"),
            "schema" => input.records[0].schema_identity = id("schema:stale"),
            _ => unreachable!(),
        }
        assert!(
            matches!(admit(input), AdmissionOutcome::Refused { cause: RefusalCause::BindingMismatch { field: actual, .. } } if actual == field)
        );
    }
}

#[trace("TC-001", "FR-001-AC-3", "IT-001")]
#[test]
fn tc001_missing_required_value_is_incomplete_not_false() {
    let mut input = request();
    input.records[0].value = ValueState::Missing;
    assert!(
        matches!(admit(input), AdmissionOutcome::Incomplete { reasons } if matches!(reasons.as_slice(), [IncompleteReason::MissingValuation { .. }]))
    );
}

#[trace("TC-001", "FR-001-AC-4", "TC-005", "FR-005-AC-5", "FR-005-AC-6")]
#[test]
fn tc005_relationship_correlation_has_four_exact_outcomes() {
    let mut missing = request();
    let source = missing.expected_subject.clone();
    let target = shipment(&missing.producer, "shipment:S1");
    missing
        .required_relationships
        .push(required(&missing, source.clone(), target.clone()));
    assert!(matches!(
        admit(missing),
        AdmissionOutcome::Incomplete { reasons }
            if matches!(reasons.as_slice(), [IncompleteReason::MissingRelationship { .. }])
    ));

    let mut conflicting = request();
    let source = conflicting.expected_subject.clone();
    let wanted = shipment(&conflicting.producer, "shipment:S1");
    let other = shipment(&conflicting.producer, "shipment:S2");
    let required_slot = required(&conflicting, source.clone(), wanted);
    let relation = relationship(&conflicting, "relationship:O1-S2", source, other);
    conflicting.required_relationships.push(required_slot);
    conflicting.relationships.push(relation);
    assert!(matches!(
        admit(conflicting),
        AdmissionOutcome::Refused {
            cause: RefusalCause::RelationshipConflict { .. }
        }
    ));

    let mut ambiguous = request();
    let source = ambiguous.expected_subject.clone();
    let target = shipment(&ambiguous.producer, "shipment:S1");
    let required_slot = required(&ambiguous, source.clone(), target.clone());
    ambiguous.required_relationships.push(required_slot);
    for suffix in ["a", "b"] {
        let relation = relationship(
            &ambiguous,
            &format!("relationship:{suffix}"),
            source.clone(),
            target.clone(),
        );
        ambiguous.relationships.push(relation);
    }
    assert!(matches!(
        admit(ambiguous),
        AdmissionOutcome::Refused {
            cause: RefusalCause::AmbiguousRelationship { .. }
        }
    ));

    let mut replay = request();
    let source = replay.expected_subject.clone();
    let target = shipment(&replay.producer, "shipment:S1");
    let required_slot = required(&replay, source.clone(), target.clone());
    let relation = relationship(&replay, "relationship:one", source, target);
    replay.required_relationships.push(required_slot);
    replay.relationships.extend([relation.clone(), relation]);
    assert!(matches!(
        admit(replay),
        AdmissionOutcome::Available { observation } if observation.relationships().len() == 1
    ));
}

#[trace("TC-001", "FR-001-AC-2", "FR-001-AC-3", "NFR-001-AC-1")]
#[test]
fn tc001_scope_closure_clock_and_limits_are_explicit() {
    let mut incomplete = request();
    incomplete.scope.closure_identity = None;
    incomplete.scope.closure_digest = None;
    assert!(
        matches!(admit(incomplete), AdmissionOutcome::Incomplete { reasons } if matches!(reasons.as_slice(), [IncompleteReason::MissingClosure]))
    );

    let mut missing_membership = request();
    missing_membership.scope.membership_complete = false;
    assert!(
        matches!(admit(missing_membership), AdmissionOutcome::Incomplete { reasons } if matches!(reasons.as_slice(), [IncompleteReason::MissingMembership]))
    );

    let mut window = request();
    window.scope.kind = ScopeKind::Window {
        window_identity: id("window:1"),
    };
    window.scope.range = ClockRange::Timestamp {
        start_nanos: 0,
        end_nanos: 30,
    };
    window.records[0].anchor = Anchor::TimestampNanos(30);
    assert!(matches!(
        admit(window),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ClockMismatch { .. }
        }
    ));
}

#[trace("TC-001", "FR-001-AC-4", "NFR-001-AC-6", "TC-005", "FR-005-AC-8")]
#[test]
fn tc005_relationship_graph_bounds_are_exact_and_fail_closed() {
    let mut exact = request();
    exact.limits.max_relationships = 0;
    exact.limits.max_required_relationships = 0;
    assert!(matches!(admit(exact), AdmissionOutcome::Available { .. }));

    let mut relationships = request();
    relationships.limits.max_relationships = 0;
    let source = relationships.expected_subject.clone();
    let target = shipment(&relationships.producer, "shipment:S1");
    let relation = relationship(&relationships, "relationship:bounded", source, target);
    relationships.relationships.push(relation);
    assert!(matches!(
        admit(relationships),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ResourceLimit {
                limit: ResourceKind::Relationships
            }
        }
    ));

    let mut required_input = request();
    required_input.limits.max_required_relationships = 0;
    let source = required_input.expected_subject.clone();
    let target = shipment(&required_input.producer, "shipment:S1");
    let relation = required(&required_input, source, target);
    required_input.required_relationships.push(relation);
    assert!(matches!(
        admit(required_input),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ResourceLimit {
                limit: ResourceKind::RequiredRelationships
            }
        }
    ));
}

#[trace("TC-001", "FR-001-AC-2", "TC-005", "FR-005-AC-3")]
#[test]
fn tc005_subject_mismatch_and_cross_kind_comparison_refuse() {
    let mut input = request();
    input.records[0].subject = order(&input.producer, "order:O2");
    assert!(matches!(
        admit(input),
        AdmissionOutcome::Refused {
            cause: RefusalCause::SubjectMismatch { .. }
        }
    ));

    let input = request();
    let shipment = shipment(&input.producer, "order:O1");
    assert_eq!(
        input.expected_subject.same_object(&shipment),
        Err(ReferenceRefusal::WrongSubjectKind)
    );
}

#[trace("TC-005", "FR-005-AC-2", "FR-005-AC-3")]
#[test]
fn tc005_opaque_subject_components_and_authority_are_exact() {
    assert_eq!(
        SubjectKind::new(Vec::<u8>::new()),
        Err(ReferenceRefusal::Empty(ReferenceComponent::SubjectKind))
    );
    assert_eq!(
        SubjectIdentity::new(Vec::<u8>::new()),
        Err(ReferenceRefusal::Empty(ReferenceComponent::SubjectIdentity))
    );
    assert_eq!(
        RelationshipIdentity::new(Vec::<u8>::new()),
        Err(ReferenceRefusal::Empty(
            ReferenceComponent::RelationshipIdentity
        ))
    );

    let producer = producer();
    let exact = support::subject(&producer, b" Kind \0".to_vec(), b" Object \xff".to_vec());
    assert_eq!(exact.kind().as_bytes(), b" Kind \0");
    assert_eq!(exact.identity().as_bytes(), b" Object \xff");

    let foreign = producer_with_bundle_identity("ix://agent-ix/commerce/bundle/foreign");
    let foreign_subject = support::subject(
        &foreign,
        exact.kind().as_bytes(),
        exact.identity().as_bytes(),
    );
    assert_eq!(
        exact.same_object(&foreign_subject),
        Err(ReferenceRefusal::ForeignAuthority)
    );

    let mut input = request();
    input.expected_subject = order(&foreign, "order:O1");
    assert_eq!(
        admit(input),
        AdmissionOutcome::Refused {
            cause: RefusalCause::InvalidReference(ReferenceRefusal::ForeignAuthority)
        }
    );
}

#[trace("TC-005", "FR-005-AC-3", "FR-005-AC-7")]
#[test]
fn tc005_presentation_is_not_a_key_member_and_sorting_is_noncausal() {
    #[derive(Clone)]
    struct Presented {
        subject: QualifiedSubject,
        display: &'static str,
        trace: &'static str,
        timestamp: i64,
        arrival: u64,
        transport_ok: bool,
    }

    let producer = producer();
    let o2 = order(&producer, "order:O2");
    let o1 = order(&producer, "order:O1");
    let first = Presented {
        subject: o1.clone(),
        display: "first",
        trace: "trace-a",
        timestamp: 99,
        arrival: 1,
        transport_ok: true,
    };
    let changed = Presented {
        subject: o1,
        display: "renamed",
        trace: "trace-z",
        timestamp: -1,
        arrival: 500,
        transport_ok: false,
    };
    assert_eq!(first.subject, changed.subject);
    assert_ne!(first.display, changed.display);
    assert_ne!(first.trace, changed.trace);
    assert_ne!(first.timestamp, changed.timestamp);
    assert_ne!(first.arrival, changed.arrival);
    assert_ne!(first.transport_ok, changed.transport_ok);

    let mut subjects = vec![o2, changed.subject, shipment(&producer, "shipment:S1")];
    subjects.sort();
    let sorted = subjects.clone();
    subjects.reverse();
    subjects.sort();
    assert_eq!(subjects, sorted);

    let distinct: std::collections::BTreeSet<_> = [
        "workflow",
        "role-instance",
        "channel",
        "node",
        "message",
        "send",
        "receive",
        "delivery",
        "attempt",
        "effect",
        "receipt",
        "configuration",
    ]
    .into_iter()
    .map(|kind| support::subject(&producer, kind, "same-opaque-bytes"))
    .collect();
    assert_eq!(distinct.len(), 12);
}

#[trace("TC-005", "FR-005-AC-4")]
#[test]
fn tc005_relationship_construction_refuses_unknown_reversed_and_foreign_inputs() {
    let producer = producer();
    let local_order = order(&producer, "order:O1");
    let local_shipment = shipment(&producer, "shipment:S1");
    let identity = || RelationshipIdentity::new("relationship:one").unwrap();

    assert!(matches!(
        Relationship::new(
            &producer,
            "relationship:unknown",
            identity(),
            local_order.clone(),
            local_shipment.clone(),
        ),
        Err(ReferenceRefusal::UnknownRelationshipDeclaration { .. })
    ));
    assert_eq!(
        Relationship::new(
            &producer,
            "",
            identity(),
            local_order.clone(),
            local_shipment.clone(),
        ),
        Err(ReferenceRefusal::Empty(
            ReferenceComponent::RelationshipDeclaration
        ))
    );
    assert_eq!(
        Relationship::new(
            &producer,
            ORDER_SHIPMENT,
            identity(),
            local_order.clone(),
            local_order.clone(),
        ),
        Err(ReferenceRefusal::WrongEndpointKind {
            endpoint: EndpointSide::Target
        })
    );
    assert_eq!(
        Relationship::new(
            &producer,
            ORDER_SHIPMENT,
            identity(),
            local_shipment,
            local_order.clone(),
        ),
        Err(ReferenceRefusal::ReversedEndpoints)
    );

    let foreign = producer_with_bundle_identity("ix://agent-ix/commerce/bundle/foreign");
    let foreign_order = order(&foreign, "order:O1");
    let local_shipment = shipment(&producer, "shipment:S1");
    assert_eq!(
        Relationship::new(
            &producer,
            ORDER_SHIPMENT,
            identity(),
            foreign_order,
            local_shipment,
        ),
        Err(ReferenceRefusal::ForeignAuthority)
    );
}

#[trace("TC-005", "FR-005-AC-5")]
#[test]
fn tc005_same_relationship_identity_cannot_rebind() {
    let mut input = request();
    let source = input.expected_subject.clone();
    let s1 = shipment(&input.producer, "shipment:S1");
    let s2 = shipment(&input.producer, "shipment:S2");
    let first = relationship(&input, "relationship:fixed", source.clone(), s1);
    let rebound = relationship(&input, "relationship:fixed", source, s2);
    input.relationships.extend([first, rebound]);
    assert!(matches!(
        admit(input),
        AdmissionOutcome::Refused {
            cause: RefusalCause::RelationshipIdentityContradiction { .. }
        }
    ));
}

#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-3", "NFR-001-AC-2")]
#[test]
fn tc001_members_must_bind_exactly_to_accepted_records() {
    let mut unrelated = request();
    unrelated.scope.members[0].record_identity = id("record:other");
    assert!(matches!(
        admit(unrelated),
        AdmissionOutcome::Refused {
            cause: RefusalCause::MemberRecordMismatch { .. }
        }
    ));

    let mut missing = request();
    missing.scope.members.clear();
    assert!(matches!(
        admit(missing),
        AdmissionOutcome::Incomplete { reasons }
            if matches!(reasons.as_slice(), [IncompleteReason::MissingMembership])
    ));
}

#[trace("TC-001", "FR-001-AC-5")]
#[test]
fn tc001_member_identity_is_an_exact_opaque_key() {
    let opaque = id("not-a-uri::member[Ω]/%2f#Exact ");
    let mut input = request();
    input.scope.required_member_identities[0] = opaque.clone();
    input.scope.members[0].object_identity = opaque;
    let input = seal(input);
    assert!(matches!(admit(input), AdmissionOutcome::Available { .. }));

    let mut case_mismatch = request();
    case_mismatch.scope.required_member_identities[0] = id("member:o1");
    let case_mismatch = seal(case_mismatch);
    assert!(matches!(
        admit(case_mismatch),
        AdmissionOutcome::Refused {
            cause: RefusalCause::DerivedIdentityMismatch {
                artifact: OwnerArtifact::MembershipPopulation
            }
        }
    ));
}

#[trace("TC-001", "FR-001-AC-6")]
#[test]
fn tc001_owner_identities_sources_and_clock_are_derived_not_trusted() {
    let mut record = request();
    record.records[0].identity = id("sha256-jcs:stale");
    record.scope.members[0].record_identity = record.records[0].identity.clone();
    assert!(matches!(
        admit(record),
        AdmissionOutcome::Refused {
            cause: RefusalCause::DerivedIdentityMismatch {
                artifact: OwnerArtifact::Observation
            }
        }
    ));

    let mut membership = request();
    membership.scope.membership_rule_identity = id("sha256-jcs:stale");
    assert!(matches!(
        admit(membership),
        AdmissionOutcome::Refused {
            cause: RefusalCause::DerivedIdentityMismatch {
                artifact: OwnerArtifact::Membership
            }
        }
    ));

    let mut population = request();
    population.scope.population_identity = id("sha256-jcs:stale");
    assert!(matches!(
        admit(population),
        AdmissionOutcome::Refused {
            cause: RefusalCause::DerivedIdentityMismatch {
                artifact: OwnerArtifact::Population
            }
        }
    ));

    let mut source = request();
    source.scope.observation_sources.push(id("source:unused"));
    source.scope.observation_sources.sort();
    authority::population::assign_request_identities(&mut source, authority::Limits::owner_max())
        .unwrap();
    assert!(matches!(
        admit(source),
        AdmissionOutcome::Refused {
            cause: RefusalCause::DerivedIdentityMismatch {
                artifact: OwnerArtifact::ObservationSources
            }
        }
    ));

    let mut clock = request();
    clock.scope.clock_identity = id("clock:foreign");
    assert!(matches!(
        admit(clock),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ClockMismatch { .. }
        }
    ));
}
