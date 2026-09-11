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

fn request() -> AdmissionRequest {
    let order = order("order:O1");
    AdmissionRequest {
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
            unit: id("USD"),
            subject_kind: SubjectKind::Order,
            required: true,
        },
        expected_subject: order.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("population:orders"),
            membership_digest: digest(4),
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
            unit: id("USD"),
            value: ValueState::Present {
                value_type: id("decimal"),
                canonical_value: "12.00".into(),
            },
            visibility: Visibility::External,
            anchor: Anchor::EventPosition(0),
        }],
        limits: ResourceLimits {
            max_records: 2,
            max_members: 2,
        },
    }
}

#[test]
fn tc140_valid_record_retains_provenance() {
    let outcome = admit(request());
    assert!(
        matches!(outcome, AdmissionOutcome::Available { records } if records[0].visibility == Visibility::External)
    );
}

#[test]
fn tc140_wrong_signal_unit_and_schema_refuse() {
    for field in ["signal", "unit", "schema"] {
        let mut input = request();
        match field {
            "signal" => input.records[0].signal_identity = id("signal:wrong"),
            "unit" => input.records[0].unit = id("EUR"),
            "schema" => input.records[0].schema_identity = id("schema:stale"),
            _ => unreachable!(),
        }
        assert!(
            matches!(admit(input), AdmissionOutcome::Refused { cause: RefusalCause::BindingMismatch { field: actual, .. } } if actual == field)
        );
    }
}

#[test]
fn tc140_missing_required_value_is_incomplete_not_false() {
    let mut input = request();
    input.records[0].value = ValueState::Missing;
    assert!(
        matches!(admit(input), AdmissionOutcome::Incomplete { reasons } if matches!(reasons.as_slice(), [IncompleteReason::MissingValuation { .. }]))
    );
}

#[test]
fn tc141_refund_for_other_order_is_a_conflict() {
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
fn tc141_missing_and_ambiguous_relationships_do_not_match_heuristically() {
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
}

#[test]
fn tc142_scope_closure_clock_and_limits_are_explicit() {
    let mut incomplete = request();
    incomplete.scope.closure_identity = None;
    incomplete.scope.closure_digest = None;
    assert!(
        matches!(admit(incomplete), AdmissionOutcome::Incomplete { reasons } if matches!(reasons.as_slice(), [IncompleteReason::MissingClosure]))
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
            cause: RefusalCause::ResourceLimit { limit: "records" }
        }
    ));
}

#[test]
fn tc142_clock_families_are_not_interchangeable() {
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
fn producer_version_and_subject_mismatch_refuse() {
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
