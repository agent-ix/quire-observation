// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Agent-IX

use ix_trace_rs::trace;
use quire_observation::*;

fn id(value: &str) -> Identity {
    Identity::new(value)
}
fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn order(value: &str) -> Subject {
    Subject {
        kind: SubjectKind::Order,
        identity: id(value),
    }
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
    let order = order("order:O1");
    let request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.into(),
            identity: id("package:checkout"),
            revision: id("1-draft"),
            digest: digest(1),
        },
        producer: ProducerSelection {
            interface_version: PRODUCER_INTERFACE_VERSION.into(),
            document_identity: id("fcd:producer"),
            document_digest: digest(2),
            model_identity: id("model:commerce"),
            configuration_identity: id("config:1"),
            configuration_digest: digest(3),
        },
        binding: ObservationBinding {
            identity: id("binding:payment-effect"),
            source_identity: id("source:provider"),
            schema_identity: id("schema:payment/v1"),
            signal_identity: id("signal:effect"),
            trigger_identity: id("trigger:payment"),
            unit: id("USD"),
            subject_kind: SubjectKind::Order,
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
    };
    seal(request)
}

#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-7")]
#[test]
fn tc001_valid_record_retains_all_selected_premises() {
    let outcome = admit(request());
    assert!(
        matches!(outcome, AdmissionOutcome::Available { observation }
            if observation.records()[0].visibility == Visibility::External
                && observation.package().identity == id("package:checkout")
                && observation.producer().document_digest == digest(2)
                && observation.producer().configuration_identity == id("config:1")
                && observation.producer().configuration_digest == digest(3)
                && observation.scope().population_identity.as_str().starts_with("sha256-jcs:")
                && observation.scope().closure_identity == Some(id("closure:complete"))
                && observation.scope().closure_digest == Some(digest(5))
                && observation.limits().max_records == 2)
    );
}

#[test]
// Trace: TC-001, FR-001-AC-2
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

#[test]
// Trace: TC-001, FR-001-AC-4
fn tc001_refund_for_other_order_is_a_conflict() {
    let mut input = request();
    let refund = Subject {
        kind: SubjectKind::Refund,
        identity: id("refund:R1"),
    };
    input.required_relationships.push(RequiredRelationship {
        kind: RelationshipKind::RefundForOrder,
        from: refund.clone(),
        to: order("order:O1"),
    });
    input.relationships.push(Relationship {
        identity: id("relationship:R1-O2"),
        kind: RelationshipKind::RefundForOrder,
        from: refund,
        to: order("order:O2"),
    });
    assert!(matches!(
        admit(input),
        AdmissionOutcome::Refused {
            cause: RefusalCause::RelationshipConflict { .. }
        }
    ));
}

#[test]
// Trace: TC-001, FR-001-AC-3, FR-001-AC-4
fn tc001_missing_and_ambiguous_relationships_do_not_match_heuristically() {
    let refund = Subject {
        kind: SubjectKind::Refund,
        identity: id("refund:R1"),
    };
    let expected = RequiredRelationship {
        kind: RelationshipKind::RefundForOrder,
        from: refund.clone(),
        to: order("order:O1"),
    };
    let mut missing = request();
    missing.required_relationships.push(expected.clone());
    assert!(
        matches!(admit(missing), AdmissionOutcome::Incomplete { reasons } if matches!(reasons.as_slice(), [IncompleteReason::MissingRelationship { .. }]))
    );
    let mut ambiguous = request();
    ambiguous.required_relationships.push(expected);
    for suffix in ["a", "b"] {
        ambiguous.relationships.push(Relationship {
            identity: id(&format!("relationship:{suffix}")),
            kind: RelationshipKind::RefundForOrder,
            from: refund.clone(),
            to: order("order:O1"),
        });
    }
    assert!(matches!(
        admit(ambiguous),
        AdmissionOutcome::Refused {
            cause: RefusalCause::AmbiguousRelationship { .. }
        }
    ));

    let mut duplicate_identity = request();
    duplicate_identity.relationships.extend([
        Relationship {
            identity: id("relationship:duplicate"),
            kind: RelationshipKind::RefundForOrder,
            from: refund.clone(),
            to: order("order:O1"),
        },
        Relationship {
            identity: id("relationship:duplicate"),
            kind: RelationshipKind::RefundCompensatesEffect,
            from: refund,
            to: Subject {
                kind: SubjectKind::Effect,
                identity: id("effect:E1"),
            },
        },
    ]);
    assert!(matches!(
        admit(duplicate_identity),
        AdmissionOutcome::Refused {
            cause: RefusalCause::InvalidSelection(SelectionField::RelationshipIdentity)
        }
    ));
}

#[test]
// Trace: TC-001, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, NFR-001-AC-1
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

    let mut excess = request();
    excess.limits.max_records = 0;
    assert!(matches!(
        admit(excess),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ResourceLimit {
                limit: ResourceKind::Records
            }
        }
    ));
}

#[trace("TC-001", "FR-001-AC-4", "NFR-001-AC-6")]
#[test]
fn tc001_relationship_graph_bounds_are_exact_and_fail_closed() {
    let mut exact = request();
    exact.limits.max_relationships = 0;
    exact.limits.max_required_relationships = 0;
    assert!(matches!(admit(exact), AdmissionOutcome::Available { .. }));

    let mut relationships = request();
    relationships.limits.max_relationships = 0;
    relationships.relationships.push(Relationship {
        identity: id("relationship:bounded"),
        kind: RelationshipKind::PaymentAttemptForOrder,
        from: Subject {
            kind: SubjectKind::PaymentAttempt,
            identity: id("payment:P1"),
        },
        to: order("order:O1"),
    });
    assert!(matches!(
        admit(relationships),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ResourceLimit {
                limit: ResourceKind::Relationships
            }
        }
    ));

    let mut required = request();
    required.limits.max_required_relationships = 0;
    required.required_relationships.push(RequiredRelationship {
        kind: RelationshipKind::PaymentAttemptForOrder,
        from: Subject {
            kind: SubjectKind::PaymentAttempt,
            identity: id("payment:P1"),
        },
        to: order("order:O1"),
    });
    assert!(matches!(
        admit(required),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ResourceLimit {
                limit: ResourceKind::RequiredRelationships
            }
        }
    ));
}

#[test]
// Trace: TC-001, FR-001-AC-2
fn tc001_clock_families_are_not_interchangeable() {
    let mut fixed = request();
    fixed.scope.range = ClockRange::FixedSample {
        start: 4,
        end_exclusive: 6,
        epoch_nanos: 10,
        period_nanos: 5,
    };
    fixed.records[0].anchor = Anchor::FixedSample {
        index: 4,
        epoch_nanos: 10,
        period_nanos: 5,
    };
    fixed.scope.members[0].anchor = fixed.records[0].anchor;
    fixed = seal(fixed);
    assert!(matches!(
        admit(fixed.clone()),
        AdmissionOutcome::Available { .. }
    ));
    fixed.records[0].anchor = Anchor::EventPosition(4);
    assert!(matches!(
        admit(fixed),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ClockMismatch { .. }
        }
    ));
}

#[test]
// Trace: TC-001, FR-001-AC-2
fn tc001_producer_version_and_subject_mismatch_refuse() {
    let mut version = request();
    version.producer.interface_version = "1.1.0".into();
    assert!(matches!(
        admit(version),
        AdmissionOutcome::Refused {
            cause: RefusalCause::ProducerVersion
        }
    ));
    let mut subject = request();
    subject.records[0].subject = order("order:O2");
    assert!(matches!(
        admit(subject),
        AdmissionOutcome::Refused {
            cause: RefusalCause::SubjectMismatch { .. }
        }
    ));
}

#[test]
// Trace: TC-001, FR-001-AC-1, FR-001-AC-3, NFR-001-AC-2
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

    let mut invalid_helper_input = request();
    invalid_helper_input.producer.document_identity = id("   ");
    let original_scope = invalid_helper_input.scope.clone();
    let error = authority::population::assign_request_identities(
        &mut invalid_helper_input,
        authority::Limits::owner_max(),
    )
    .expect_err("invalid producer selection must fail closed");
    assert_eq!(error.code(), authority::ErrorCode::InvalidSelection);
    assert_eq!(invalid_helper_input.scope, original_scope);
}
