use quire_observation::handoff::*;
use quire_observation::replay::*;
use quire_observation::{Digest, Identity};

fn id(value: &str) -> Identity {
    Identity::new(value)
}
fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn request() -> ReplayRequest {
    ReplayRequest {
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
            identity: id("trigger:refund-request"),
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
            max_events: 4,
            max_active_keys: 1,
        },
        observations: vec![],
    }
}

fn observation(signal: &str, event: i128, ingest: i128, sequence: u64) -> TimedObservation {
    TimedObservation {
        identity: id(&format!("record:{sequence}")),
        subject_identity: id("order:O1"),
        signal_identity: id(signal),
        event_time_nanos: event,
        ingestion_time_nanos: ingest,
        sequence,
    }
}

fn scope(name: &str) -> ScopeFact {
    ScopeFact {
        scope_identity: id(&format!("scope:{name}")),
        authority_identity: id(&format!("authority:{name}")),
        boundary_identity: id(&format!("boundary:{name}")),
    }
}

#[test]
fn tc002_replay_and_incremental_agree_for_decisive_cases() {
    let mut satisfied = request();
    satisfied
        .observations
        .push(observation("signal:refund", 10, 11, 1));
    assert_eq!(replay(&satisfied), incremental(&satisfied));
    assert_eq!(replay(&satisfied).disposition, Disposition::Satisfied);
    let mut violated = request();
    violated
        .observations
        .push(observation("signal:chargeback", 10, 11, 1));
    assert_eq!(replay(&violated), incremental(&violated));
    assert_eq!(replay(&violated).disposition, Disposition::Violated);
}

#[test]
fn tc002_quiet_deadline_needs_matching_progress_authority() {
    let open = request();
    assert_eq!(replay(&open).disposition, Disposition::Open);
    let mut settled = request();
    settled.progress = Some(ProgressAssertion {
        identity: id("progress:O1"),
        definition_identity: id("progress-definition"),
        definition_digest: digest(1),
        authority_identity: id("authority"),
        scope_identity: id("window:O1"),
        source_set_identity: id("sources"),
        restoration_identity: id("restore"),
        covered_through_nanos: 31,
    });
    assert_eq!(replay(&settled).disposition, Disposition::MissedDeadline);
    settled.progress.as_mut().unwrap().scope_identity = id("window:O2");
    assert_eq!(
        replay(&settled).disposition,
        Disposition::IncompleteProgress
    );
}

#[test]
fn tc002_non_success_and_late_inputs_stay_explicit() {
    let mut incomplete = request();
    incomplete.history_available = false;
    assert_eq!(
        replay(&incomplete).disposition,
        Disposition::IncompleteHistory
    );
    let mut ambiguous = request();
    ambiguous.membership_ambiguous = true;
    assert_eq!(
        replay(&ambiguous).disposition,
        Disposition::AmbiguousMembership
    );
    let mut late = request();
    late.prior_result_identity = Some(id("result:earlier"));
    late.observations
        .push(observation("signal:refund", 10, 41, 1));
    let result = replay(&late);
    assert_eq!(result.disposition, Disposition::Open);
    assert_eq!(result.late_records, vec![id("record:1")]);
    assert_eq!(result.supersedes, Some(id("result:earlier")));
    let mut exhausted = request();
    exhausted.limits.max_events = 0;
    exhausted
        .observations
        .push(observation("signal:refund", 10, 11, 1));
    assert_eq!(replay(&exhausted).disposition, Disposition::Exhausted);
}

#[test]
fn tc002_incremental_intake_enforces_event_and_active_key_bounds() {
    let mut retained = request();
    retained.limits.max_events = 1;
    let mut state = IncrementalAssessment::new(retained);
    state.push(observation("signal:refund", 10, 11, 1)).unwrap();
    assert_eq!(
        state.push(observation("signal:refund", 11, 12, 2)),
        Err(IncrementalError::RetentionExhausted)
    );
    let mut keys = request();
    keys.limits.max_events = 2;
    let mut state = IncrementalAssessment::new(keys);
    state.push(observation("signal:refund", 10, 11, 1)).unwrap();
    let mut other = observation("signal:refund", 11, 12, 2);
    other.subject_identity = id("order:O2");
    assert_eq!(state.push(other), Err(IncrementalError::ActiveKeyExhausted));
}

#[test]
fn tc002_untriggered_and_ambiguous_boundaries_stay_distinct() {
    let mut untriggered = request();
    untriggered.trigger = TriggerState::Untriggered;
    assert_eq!(replay(&untriggered).disposition, Disposition::Untriggered);
    let mut ambiguous = request();
    ambiguous.trigger = TriggerState::AmbiguousBoundary {
        identity: id("trigger:boundary"),
    };
    assert_eq!(
        replay(&ambiguous).disposition,
        Disposition::AmbiguousBoundary
    );
}

#[test]
fn tc003_handoff_never_promotes_loss_to_preservation() {
    let mut replay_result = replay(&request());
    replay_result.late_records.push(id("record:late"));
    let result = AssessmentHandoff {
        result_identity: id("result:1"),
        replay: replay_result,
        activation: Activation::Unknown,
        participation: Participation::MissingRequiredObservation,
        completeness: Completeness::Incomplete,
        decision_progress: scope("decision"),
        decision_closure: scope("decision"),
        surrounding_progress: scope("surrounding"),
        surrounding_closure: scope("surrounding"),
        global_closure: GlobalConformanceClosure::NotRequired,
        source_identity: id("source"),
        binding_identity: id("binding"),
        dependencies: vec![ImmutableDependency {
            identity: id("definition"),
            revision: id("1"),
            digest: digest(2),
        }],
        supersedes: Some(id("result:0")),
    };
    let missing = ConsumerCapabilities {
        preserves_activation: true,
        preserves_participation: true,
        preserves_completeness: true,
        preserves_dependencies: true,
        preserves_late_supersession: false,
    };
    assert!(matches!(
        handoff(result.clone(), missing),
        HandoffOutcome::Delivered(value) if value.mapping == MappingState::Unrepresented
    ));
    let full = ConsumerCapabilities {
        preserves_activation: true,
        preserves_participation: true,
        preserves_completeness: true,
        preserves_dependencies: true,
        preserves_late_supersession: true,
    };
    assert!(matches!(
        handoff(result, full),
        HandoffOutcome::Delivered(value) if value.mapping == MappingState::Preserved
    ));
}

#[test]
fn tc003_scope_cross_wiring_and_duplicate_dependencies_refuse() {
    let replay = replay(&request());
    let mut result = AssessmentHandoff {
        result_identity: id("result:axes"),
        replay,
        activation: Activation::Activated,
        participation: Participation::Complete,
        completeness: Completeness::Complete,
        decision_progress: scope("decision"),
        decision_closure: scope("decision"),
        surrounding_progress: scope("surrounding"),
        surrounding_closure: scope("surrounding"),
        global_closure: GlobalConformanceClosure::Required {
            state: GlobalClosureState::Open,
            execution_identity: id("execution"),
            branch_identity: id("branch"),
            workflow_identity: id("workflow"),
        },
        source_identity: id("source"),
        binding_identity: id("binding"),
        dependencies: vec![ImmutableDependency {
            identity: id("definition"),
            revision: id("1"),
            digest: digest(3),
        }],
        supersedes: None,
    };
    result.decision_closure.scope_identity = id("scope:wrong");
    let caps = ConsumerCapabilities {
        preserves_activation: true,
        preserves_participation: true,
        preserves_completeness: true,
        preserves_dependencies: true,
        preserves_late_supersession: true,
    };
    assert_eq!(
        handoff(result.clone(), caps.clone()),
        HandoffOutcome::Refused(HandoffRefusal::ScopeCrossWiring)
    );
    result.decision_closure = scope("decision");
    result.dependencies.push(result.dependencies[0].clone());
    assert_eq!(
        handoff(result, caps),
        HandoffOutcome::Refused(HandoffRefusal::DuplicateDependency)
    );
}

#[test]
fn tc003_handoff_preserves_each_non_boolean_disposition() {
    let capabilities = ConsumerCapabilities {
        preserves_activation: true,
        preserves_participation: true,
        preserves_completeness: true,
        preserves_dependencies: true,
        preserves_late_supersession: true,
    };
    for disposition in [
        Disposition::Satisfied,
        Disposition::Violated,
        Disposition::Untriggered,
        Disposition::IncompleteHistory,
        Disposition::Unsupported,
    ] {
        let result = AssessmentHandoff {
            result_identity: id("result:distinct"),
            replay: ReplayResult {
                assessment_identity: id("assessment"),
                disposition,
                basis: SettlementBasis::Unavailable,
                decision_support: vec![],
                progress_identity: None,
                late_records: vec![],
                supersedes: None,
                retained_events: 0,
            },
            activation: if disposition == Disposition::Untriggered {
                Activation::Untriggered
            } else {
                Activation::Activated
            },
            participation: Participation::Complete,
            completeness: if disposition == Disposition::IncompleteHistory {
                Completeness::Incomplete
            } else {
                Completeness::Complete
            },
            decision_progress: scope("decision"),
            decision_closure: scope("decision"),
            surrounding_progress: scope("surrounding"),
            surrounding_closure: scope("surrounding"),
            global_closure: GlobalConformanceClosure::NotRequired,
            source_identity: id("source"),
            binding_identity: id("binding"),
            dependencies: vec![ImmutableDependency {
                identity: id("definition"),
                revision: id("1"),
                digest: digest(4),
            }],
            supersedes: None,
        };
        assert!(matches!(
            handoff(result, capabilities.clone()),
            HandoffOutcome::Delivered(value) if value.result.replay.disposition == disposition
        ));
    }
}
