use quire_observation::handoff::*;
use quire_observation::replay::*;
use quire_observation::*;

fn id(value: &str) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn admitted_request() -> AdmissionRequest {
    let order = Subject {
        kind: SubjectKind::Order,
        identity: id("order:O1"),
    };
    AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.into(),
            identity: id("package:refund"),
            revision: id("1"),
            digest: digest(1),
        },
        producer: ProducerSelection {
            interface_version: PRODUCER_INTERFACE_VERSION.into(),
            document_identity: id("producer"),
            document_digest: digest(2),
            model_identity: id("model"),
            configuration_identity: id("configuration"),
            configuration_digest: digest(3),
        },
        binding: ObservationBinding {
            identity: id("binding:refund"),
            source_identity: id("source:provider"),
            schema_identity: id("schema:1"),
            signal_identity: id("signal:refund"),
            unit: id("USD"),
            subject_kind: SubjectKind::Order,
            required: true,
        },
        expected_subject: order.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("population"),
            membership_digest: digest(4),
            membership_complete: true,
            closure_identity: Some(id("closure")),
            closure_digest: Some(digest(5)),
            kind: ScopeKind::Window {
                window_identity: id("window:O1"),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id("member:O1"),
                record_identity: id("record:refund"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("record:refund"),
            binding_identity: id("binding:refund"),
            source_identity: id("source:provider"),
            schema_identity: id("schema:1"),
            subject: order,
            signal_identity: id("signal:refund"),
            unit: id("USD"),
            value: ValueState::Present {
                value_type: id("decimal"),
                canonical_value: "12.00".into(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
        },
    }
}

fn scope(name: &str) -> ScopeFact {
    ScopeFact {
        scope_identity: id(name),
        authority_identity: id(&format!("authority:{name}")),
        boundary_identity: id(&format!("boundary:{name}")),
    }
}

#[test]
fn it001_qualified_assessment_reaches_typed_consumer_without_loss() {
    let admission = admit(admitted_request());
    assert!(matches!(admission, AdmissionOutcome::Available { .. }));

    let replay_request = ReplayRequest {
        assessment_identity: id("assessment:refund"),
        scope_identity: id("window:O1"),
        scope_start_nanos: 0,
        deadline_nanos: 30,
        late_cutoff_nanos: 40,
        expected_progress_definition: id("progress-definition"),
        expected_progress_digest: digest(1),
        expected_source_set: id("sources"),
        expected_restoration: id("restore"),
        prior_result_identity: None,
        trigger: TriggerState::Activated {
            identity: id("trigger:refund"),
            event_time_nanos: 0,
        },
        required_history: true,
        history_available: true,
        membership_ambiguous: false,
        profile_supported: true,
        progress: None,
        rule: DecisionRule {
            witness_signal: id("signal:refund"),
            counterexample_signal: id("signal:chargeback"),
        },
        limits: ReplayLimits {
            max_events: 1,
            max_active_keys: 1,
        },
        observations: vec![TimedObservation {
            identity: id("record:refund"),
            subject_identity: id("order:O1"),
            signal_identity: id("signal:refund"),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            sequence: 1,
        }],
    };
    let batch = replay(&replay_request);
    assert_eq!(batch, incremental(&replay_request));
    assert_eq!(batch.disposition, Disposition::Satisfied);

    let handoff_result = AssessmentHandoff {
        result_identity: id("result:refund"),
        replay: batch,
        activation: Activation::Activated,
        participation: Participation::Complete,
        completeness: Completeness::Complete,
        decision_progress: scope("window:O1"),
        decision_closure: scope("window:O1"),
        surrounding_progress: scope("execution:O1"),
        surrounding_closure: scope("execution:O1"),
        global_closure: GlobalConformanceClosure::NotRequired,
        source_identity: id("source:provider"),
        binding_identity: id("binding:refund"),
        dependencies: vec![ImmutableDependency {
            identity: id("package:refund"),
            revision: id("1"),
            digest: digest(1),
        }],
        supersedes: None,
    };
    let capabilities = ConsumerCapabilities {
        preserves_activation: true,
        preserves_participation: true,
        preserves_completeness: true,
        preserves_dependencies: true,
        preserves_late_supersession: true,
    };
    assert!(matches!(
        handoff(handoff_result, capabilities),
        HandoffOutcome::Delivered(value) if value.mapping == MappingState::Preserved
    ));
}

#[test]
fn it001_incomplete_admission_cannot_reach_passed_handoff() {
    let mut request = admitted_request();
    request.scope.membership_complete = false;
    assert!(matches!(
        admit(request),
        AdmissionOutcome::Incomplete { reasons }
            if matches!(reasons.as_slice(), [IncompleteReason::MissingMembership])
    ));
}
