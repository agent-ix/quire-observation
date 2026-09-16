// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-008: activation identity and independent assessment-authority axes.

mod support;

use ix_trace_rs::trace;
use quire_observation::authority::{
    self, AuthoritySelection, Context, ErrorCode, History, Limits, OpenClosed, SubjectSelection,
    TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, ResourceLimits,
    ScopeKind, ScopeSelection, ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
};

use authority::activation::{
    ActivationSelection, ActivationState, AuthorityProofs, Capture, Contribution, EvidenceState,
    ExecutionState, Lateness, ProgressSelection, Selection, SilenceCoverage,
    SilenceIncompleteReason,
};
use authority::partial::EventTimeInterval;
use support::{order, producer};

fn id(value: &str) -> Identity {
    Identity::new(value)
}

fn interval(earliest: i128, latest: i128) -> EventTimeInterval {
    EventTimeInterval::new(
        id("clock:event-time"),
        id("1"),
        id("unit:nanosecond"),
        earliest,
        latest,
        Limits::owner_max(),
    )
    .expect("valid test interval")
}

fn capture(value: &str) -> Capture {
    Capture::new(
        id("capture:amount"),
        id("observation:amount-at-activation"),
        id("type:decimal"),
        value.to_owned(),
        id("provenance:source-a"),
    )
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn qualified() -> Box<QualifiedObservation> {
    qualified_with_value("12.00")
}

fn qualified_with_value(value: &str) -> Box<QualifiedObservation> {
    let producer = producer();
    let subject = order(&producer, "order:O1");
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id("package:activation"),
            revision: id("1"),
            digest: digest(1),
        },
        binding: ObservationBinding {
            identity: id("binding:amount"),
            source_identity: id("source:payments"),
            schema_identity: id("schema:amount/v1"),
            signal_identity: id("signal:amount"),
            trigger_identity: id("trigger:refund-request"),
            unit: id("USD"),
            subject_kind: subject.kind().clone(),
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id("unsealed-membership"),
            membership_digest: digest(0),
            membership_document: vec![],
            required_member_identities: vec![id("member:O1")],
            observation_sources: vec![id("source:payments")],
            completeness_dependencies: vec![],
            progress_dependencies: vec![],
            clock_identity: id("clock:event-time"),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id("closure-definition:windows")),
            closure_digest: Some(digest(4)),
            kind: ScopeKind::Window {
                window_identity: id("window:O1"),
            },
            range: ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 30,
            },
            members: vec![Member {
                object_identity: id("member:O1"),
                record_identity: id("unsealed-record"),
                anchor: Anchor::TimestampNanos(10),
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id("binding:amount"),
            source_identity: id("source:payments"),
            schema_identity: id("schema:amount/v1"),
            subject,
            signal_identity: id("signal:amount"),
            trigger_identity: id("trigger:refund-request"),
            unit: id("USD"),
            value: ValueState::Present {
                value_type: id("type:decimal"),
                canonical_value: value.to_owned(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 15,
            causal_relationship_identity: None,
            clock_identity: id("clock:event-time"),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 2,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
        producer,
    };
    let record_identity =
        authority::observation::record_identity(&request.records[0], Limits::owner_max())
            .expect("derive observation identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, Limits::owner_max())
        .expect("derive population identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("fixture admission failed: {other:?}"),
    }
}

fn owner() -> AuthoritySelection {
    AuthoritySelection {
        definition_identity: id("definition:activation-owner"),
        definition_revision: id("1"),
        definition_digest: digest(9),
    }
}

fn subject(qualified: &QualifiedObservation) -> SubjectSelection {
    SubjectSelection {
        scope_identity: id("window:O1"),
        population_identity: qualified.scope().population_identity.clone(),
    }
}

fn cutoff(instant_nanos: i128) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id("clock:event-time"),
        id("1"),
        instant_nanos,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}

fn boundary(state: OpenClosed) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: if state == OpenClosed::Closed { 30 } else { 20 },
    }
}

fn authority_proofs(
    qualified: &QualifiedObservation,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
    progress: ExecutionState,
    closure: ExecutionState,
    evidence: EvidenceState,
    include_capture: bool,
) -> AuthorityProofs {
    let context = Context::new(History::batch(qualified), owner, subject, 1, None);
    let record = &qualified.records()[0];
    let capture_selection = authority::capture::Selection::new(
        id("trigger:refund-request"),
        Anchor::TimestampNanos(10),
        vec![authority::capture::Binding::new(
            id("capture:amount"),
            id("binding:amount"),
            record.identity.clone(),
        )],
    );
    let capture_document =
        authority::capture::derive(context, &capture_selection, Limits::owner_max())
            .expect("derive complete capture authority");
    let capture_view = authority::capture::read(
        capture_document.bytes(),
        context,
        &capture_selection,
        Limits::owner_max(),
    )
    .expect("strict-read complete capture authority");

    let progress_view = match progress {
        ExecutionState::Incomplete => None,
        ExecutionState::Open | ExecutionState::Closed => {
            let state = if progress == ExecutionState::Open {
                OpenClosed::Open
            } else {
                OpenClosed::Closed
            };
            let selected = authority::progress::Selection::new(
                authority::clock::Selection::new(id("clock:event-time"), id("1")),
                vec![id("source:payments")],
                boundary(state),
                state,
                id("trigger:refund-request"),
                cutoff(40),
                id("restoration:O1"),
            );
            let document = authority::progress::derive(context, &selected, Limits::owner_max())
                .expect("derive progress authority");
            Some(
                authority::progress::read(
                    document.bytes(),
                    context,
                    &selected,
                    Limits::owner_max(),
                )
                .expect("strict-read progress authority"),
            )
        }
    };

    let closure_view = match closure {
        ExecutionState::Incomplete => None,
        ExecutionState::Open | ExecutionState::Closed => {
            let state = if closure == ExecutionState::Open {
                OpenClosed::Open
            } else {
                OpenClosed::Closed
            };
            let selected = authority::closure::Selection::new(
                id("clock:event-time"),
                id("1"),
                vec![id("source:payments")],
                boundary(state),
                state,
            );
            let document = authority::closure::derive(context, &selected, Limits::owner_max())
                .expect("derive closure authority");
            Some(
                authority::closure::read(document.bytes(), context, &selected, Limits::owner_max())
                    .expect("strict-read closure authority"),
            )
        }
    };

    let (status, observation_identity) = match evidence {
        EvidenceState::Complete => (
            authority::completeness::FactStatus::Available,
            Some(record.identity.clone()),
        ),
        EvidenceState::Incomplete => (authority::completeness::FactStatus::Incomplete, None),
        EvidenceState::Contradicted => (
            authority::completeness::FactStatus::Contradicted,
            Some(record.identity.clone()),
        ),
    };
    let completeness_selection = authority::completeness::Selection::new(
        id("boundary:O1"),
        vec![authority::completeness::Fact::new(
            id("member:O1"),
            observation_identity,
            status,
        )],
    );
    let completeness_document =
        authority::completeness::derive(context, &completeness_selection, Limits::owner_max())
            .expect("derive completeness authority");
    let completeness_view = authority::completeness::read(
        completeness_document.bytes(),
        context,
        &completeness_selection,
        Limits::owner_max(),
    )
    .expect("strict-read completeness authority");

    if include_capture {
        AuthorityProofs::new(
            &capture_view,
            progress_view.as_ref(),
            closure_view.as_ref(),
            Some(&completeness_view),
        )
        .expect("compose strict-read authority proofs")
    } else {
        AuthorityProofs::without_capture(
            progress_view.as_ref(),
            closure_view.as_ref(),
            Some(&completeness_view),
        )
        .expect("compose trigger-absent authority proofs")
    }
}

fn selection(
    qualified: &QualifiedObservation,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
    trigger: TriggerCase,
) -> Selection {
    selection_axes(
        qualified,
        owner,
        subject,
        AxesCase {
            trigger,
            progress: ExecutionState::Closed,
            closure: ExecutionState::Open,
            evidence: EvidenceState::Complete,
            include_contributions: true,
        },
    )
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum TriggerCase {
    Absent,
    Admitted,
}

#[derive(Clone, Copy)]
struct AxesCase {
    trigger: TriggerCase,
    progress: ExecutionState,
    closure: ExecutionState,
    evidence: EvidenceState,
    include_contributions: bool,
}

fn selection_axes(
    qualified: &QualifiedObservation,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
    axes: AxesCase,
) -> Selection {
    let record = &qualified.records()[0];
    let proofs = authority_proofs(
        qualified,
        owner,
        subject,
        axes.progress,
        axes.closure,
        axes.evidence,
        axes.trigger == TriggerCase::Admitted,
    );
    let selected = Selection::new(
        activation_selection(qualified, axes.trigger, "12.00", "source:payments"),
        progress_selection(axes.progress),
        interval(8, 12),
        interval(10, 20),
        proofs,
    );
    if axes.include_contributions {
        selected.with_contributions(
            Some(Contribution::new(
                id("verdict:eligible"),
                vec![record.identity.clone()],
            )),
            Some(Contribution::new(
                id("settlement:pending"),
                vec![record.identity.clone()],
            )),
        )
    } else {
        selected
    }
}

fn activation_selection(
    qualified: &QualifiedObservation,
    trigger: TriggerCase,
    canonical_value: &str,
    provenance_identity: &str,
) -> ActivationSelection {
    let record = &qualified.records()[0];
    if trigger == TriggerCase::Admitted {
        ActivationSelection::new(
            id("obligation:refund"),
            record.identity.clone(),
            interval(8, 12),
            vec![Capture::new(
                id("capture:amount"),
                record.identity.clone(),
                id("type:decimal"),
                canonical_value.to_owned(),
                id(provenance_identity),
            )],
        )
    } else {
        ActivationSelection::without_trigger(id("obligation:refund"), interval(8, 12))
    }
}

fn progress_selection(state: ExecutionState) -> ProgressSelection {
    let frontier = if state == ExecutionState::Closed {
        30
    } else {
        20
    };
    ProgressSelection::new(interval(0, 29), interval(frontier, frontier))
}

#[trace("TC-008", "FR-008-AC-1")]
#[test]
fn tc008_trigger_and_original_capture_provenance_bind_activation_identity() {
    let captured = capture("12.00");
    let first = authority::activation::activation_identity(
        &id("obligation:refund"),
        &id("trigger:first"),
        &interval(8, 12),
        std::slice::from_ref(&captured),
        Limits::owner_max(),
    )
    .expect("derive first activation");
    let second = authority::activation::activation_identity(
        &id("obligation:refund"),
        &id("trigger:second"),
        &interval(8, 12),
        std::slice::from_ref(&captured),
        Limits::owner_max(),
    )
    .expect("derive second activation");

    assert_ne!(first, second);
    assert_eq!(captured.canonical_value(), "12.00");
    assert_eq!(
        captured.source_observation_identity(),
        "observation:amount-at-activation"
    );
    assert_eq!(captured.provenance_identity(), "provenance:source-a");
}

#[trace("TC-008", "FR-008-AC-1")]
#[test]
fn tc008_later_observation_value_cannot_rewrite_original_activation_capture() {
    let original = qualified_with_value("12.00");
    let later = qualified_with_value("13.00");
    let owner = owner();
    let original_subject = subject(&original);
    let later_subject = subject(&later);
    let make_selection =
        |qualified: &QualifiedObservation, selected_subject: &SubjectSelection, value: &str| {
            let proofs = authority_proofs(
                qualified,
                &owner,
                selected_subject,
                ExecutionState::Closed,
                ExecutionState::Closed,
                EvidenceState::Complete,
                true,
            );
            Selection::new(
                activation_selection(qualified, TriggerCase::Admitted, value, "source:payments"),
                progress_selection(ExecutionState::Closed),
                interval(8, 12),
                interval(10, 20),
                proofs,
            )
        };
    let original_selection = make_selection(&original, &original_subject, "12.00");
    let later_selection = make_selection(&later, &later_subject, "13.00");
    let original_context = Context::new(
        History::batch(&original),
        &owner,
        &original_subject,
        1,
        None,
    );
    let later_context = Context::new(History::batch(&later), &owner, &later_subject, 1, None);
    let original_document =
        authority::activation::derive(original_context, &original_selection, Limits::owner_max())
            .expect("derive original activation");
    let later_document =
        authority::activation::derive(later_context, &later_selection, Limits::owner_max())
            .expect("derive later activation");
    let original_view = authority::activation::read(
        original_document.bytes(),
        original_context,
        &original_selection,
        Limits::owner_max(),
    )
    .expect("original authority remains independently readable");
    let later_view = authority::activation::read(
        later_document.bytes(),
        later_context,
        &later_selection,
        Limits::owner_max(),
    )
    .expect("later authority is independently readable");
    assert_eq!(
        original_view
            .payload()
            .captures()
            .next()
            .unwrap()
            .canonical_value,
        "12.00"
    );
    assert_eq!(
        later_view
            .payload()
            .captures()
            .next()
            .unwrap()
            .canonical_value,
        "13.00"
    );
    assert_ne!(
        original_view.payload().activation_identity(),
        later_view.payload().activation_identity()
    );
}

#[trace("TC-008", "FR-008-AC-1")]
#[test]
fn tc008_activation_capture_set_must_be_nonempty_sorted_and_distinct() {
    let first = Capture::new(
        id("capture:a"),
        id("observation:a"),
        id("type:decimal"),
        "12.00".to_owned(),
        id("source:a"),
    );
    let second = Capture::new(
        id("capture:b"),
        id("observation:b"),
        id("type:decimal"),
        "13.00".to_owned(),
        id("source:b"),
    );
    for captures in [
        vec![],
        vec![second.clone(), first.clone()],
        vec![first.clone(), first],
    ] {
        let error = authority::activation::activation_identity(
            &id("obligation:refund"),
            &id("trigger:first"),
            &interval(8, 12),
            &captures,
            Limits::owner_max(),
        )
        .expect_err("invalid capture set must refuse");
        assert_eq!(error.code(), ErrorCode::InvalidSelection);
    }
}

#[trace("TC-008", "FR-008-AC-2", "FR-008-AC-4")]
#[test]
fn tc008_derived_activation_crosses_independent_scope_and_contribution_axes() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    for trigger in [TriggerCase::Absent, TriggerCase::Admitted] {
        for progress in [
            ExecutionState::Open,
            ExecutionState::Closed,
            ExecutionState::Incomplete,
        ] {
            for closure in [
                ExecutionState::Open,
                ExecutionState::Closed,
                ExecutionState::Incomplete,
            ] {
                for evidence in [
                    EvidenceState::Complete,
                    EvidenceState::Incomplete,
                    EvidenceState::Contradicted,
                ] {
                    for include_contributions in [false, true] {
                        let selected = selection_axes(
                            &qualified,
                            &owner,
                            &subject,
                            AxesCase {
                                trigger,
                                progress,
                                closure,
                                evidence,
                                include_contributions,
                            },
                        );
                        let document =
                            authority::activation::derive(context, &selected, Limits::owner_max())
                                .expect("independent axis product derives");
                        let view = authority::activation::read(
                            document.bytes(),
                            context,
                            &selected,
                            Limits::owner_max(),
                        )
                        .expect("independent axis product strict-reads");
                        let payload = view.payload();
                        let expected_activation = if trigger == TriggerCase::Admitted {
                            ActivationState::Active
                        } else if closure == ExecutionState::Closed
                            && evidence == EvidenceState::Complete
                        {
                            ActivationState::Inactive
                        } else {
                            ActivationState::Unknown
                        };
                        assert_eq!(payload.activation(), expected_activation);
                        assert_eq!(payload.progress(), progress);
                        assert_eq!(payload.closure(), closure);
                        assert_eq!(payload.evidence(), evidence);
                        assert_eq!(
                            payload.silence_covered(),
                            progress == ExecutionState::Closed
                        );
                        assert_eq!(
                            payload.progress_authority().is_some(),
                            progress != ExecutionState::Incomplete
                        );
                        assert_eq!(
                            payload.silence_incomplete_reasons().any(|reason| reason
                                == SilenceIncompleteReason::MissingProgressAuthority),
                            progress == ExecutionState::Incomplete
                        );
                        assert_eq!(
                            payload
                                .silence_incomplete_reasons()
                                .any(|reason| reason
                                    == SilenceIncompleteReason::MissingRequiredSource),
                            progress == ExecutionState::Incomplete
                        );
                        assert_eq!(
                            payload.closure_authority().is_some(),
                            closure != ExecutionState::Incomplete
                        );
                        assert!(payload.completeness_authority().is_some());
                        assert_eq!(payload.verdict().is_some(), include_contributions);
                        assert_eq!(payload.settlement().is_some(), include_contributions);
                    }
                }
            }
        }
    }
}

#[trace("TC-008", "FR-008-AC-3", "FR-008-AC-4", "TC-012")]
#[test]
fn tc008_silence_requires_definitely_beyond_progress_and_every_source() {
    let required = [id("source:a"), id("source:b")];
    assert_eq!(
        authority::activation::silence_coverage(
            &interval(0, 10),
            &interval(11, 20),
            &required,
            &required,
            Limits::owner_max(),
        )
        .expect("same-domain progress"),
        SilenceCoverage::Covered,
    );
    assert_eq!(
        authority::activation::silence_coverage(
            &interval(0, 10),
            &interval(11, 20),
            &required,
            &[id("source:b"), id("source:a")],
            Limits::owner_max(),
        )
        .expect("source presentation order is not semantic"),
        SilenceCoverage::Covered,
    );

    let overlap = authority::activation::silence_coverage(
        &interval(0, 10),
        &interval(10, 20),
        &required,
        &required,
        Limits::owner_max(),
    )
    .expect("overlap is an incomplete fact");
    assert!(overlap.has_reason(SilenceIncompleteReason::ProgressNotDefinitelyBeyond));

    let missing = authority::activation::silence_coverage(
        &interval(0, 10),
        &interval(11, 20),
        &required,
        &[id("source:a")],
        Limits::owner_max(),
    )
    .expect("missing source is an incomplete fact");
    assert!(missing.has_reason(SilenceIncompleteReason::MissingRequiredSource));

    let both = authority::activation::silence_coverage(
        &interval(0, 10),
        &interval(10, 20),
        &required,
        &[id("source:a")],
        Limits::owner_max(),
    )
    .expect("every failed silence premise is retained");
    assert_eq!(
        both.reasons().collect::<Vec<_>>(),
        [
            SilenceIncompleteReason::ProgressNotDefinitelyBeyond,
            SilenceIncompleteReason::MissingRequiredSource,
        ]
    );
}

#[trace("TC-008", "FR-008-AC-5")]
#[test]
fn tc008_lateness_uses_interval_endpoints_not_ingestion_order() {
    assert_eq!(
        authority::activation::classify_lateness(
            &interval(0, 10),
            &interval(10, 20),
            Limits::owner_max(),
        )
        .unwrap(),
        Lateness::DefinitelyTimely,
    );
    assert_eq!(
        authority::activation::classify_lateness(
            &interval(21, 30),
            &interval(10, 20),
            Limits::owner_max(),
        )
        .unwrap(),
        Lateness::DefinitelyLate,
    );
    assert_eq!(
        authority::activation::classify_lateness(
            &interval(10, 30),
            &interval(10, 20),
            Limits::owner_max(),
        )
        .unwrap(),
        Lateness::Uncertain,
    );
}

#[trace("TC-008", "FR-008-AC-6")]
#[test]
fn tc008_cross_wired_clock_authority_refuses_without_fallback() {
    let foreign = EventTimeInterval::new(
        id("clock:foreign"),
        id("1"),
        id("unit:nanosecond"),
        11,
        20,
        Limits::owner_max(),
    )
    .unwrap();
    let error = authority::activation::silence_coverage(
        &interval(0, 10),
        &foreign,
        &[id("source:a")],
        &[id("source:a")],
        Limits::owner_max(),
    )
    .expect_err("foreign clock must refuse");
    assert_eq!(error.code(), ErrorCode::IntervalDomainMismatch);
}

#[trace(
    "TC-008",
    "FR-008-AC-1",
    "FR-008-AC-2",
    "FR-008-AC-3",
    "FR-008-AC-4",
    "FR-008-AC-5",
    "FR-008-AC-6",
    "TC-012",
    "NFR-003-AC-2"
)]
#[test]
fn tc008_versioned_owner_round_trips_all_independent_authority() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let selected = selection(&qualified, &owner, &subject, TriggerCase::Admitted);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let first = authority::activation::derive(context, &selected, Limits::owner_max())
        .expect("derive activation authority");
    let second = authority::activation::derive(context, &selected, Limits::owner_max())
        .expect("derive activation authority again");
    assert_eq!(first.bytes(), second.bytes());

    let population_exact = Limits {
        max_population_entries: qualified.records().len(),
        ..Limits::owner_max()
    };
    authority::activation::derive(context, &selected, population_exact)
        .expect("exact activation record-index bound admits");
    let population_one_over = Limits {
        max_population_entries: qualified.records().len() - 1,
        ..Limits::owner_max()
    };
    let first_error = authority::activation::derive(context, &selected, population_one_over)
        .expect_err("one-over activation record-index bound");
    let repeated_error = authority::activation::derive(context, &selected, population_one_over)
        .expect_err("repeated one-over activation record-index bound");
    assert_eq!(first_error, repeated_error);
    assert_eq!(first_error.code(), ErrorCode::ResourceIncomplete);

    let view = authority::activation::read(first.bytes(), context, &selected, Limits::owner_max())
        .expect("strict-read activation authority");
    let payload = view.payload();
    assert_eq!(payload.obligation_identity(), "obligation:refund");
    assert_eq!(payload.activation(), ActivationState::Active);
    assert_eq!(payload.progress(), ExecutionState::Closed);
    assert_eq!(payload.closure(), ExecutionState::Open);
    assert_eq!(payload.evidence(), EvidenceState::Complete);
    assert_eq!(payload.lateness(), Lateness::Uncertain);
    assert!(payload.silence_covered());
    assert_eq!(payload.required_sources(), ["source:payments"]);
    assert_eq!(payload.covered_sources(), ["source:payments"]);
    assert_eq!(payload.activation_interval().earliest, "8");
    assert_eq!(payload.deadline_interval().latest, "29");
    assert_eq!(payload.progress_interval().earliest, "30");
    assert_eq!(payload.event_interval().earliest, "8");
    assert_eq!(payload.cutoff_interval().latest, "20");
    let captures = payload.captures().collect::<Vec<_>>();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].canonical_value, "12.00");
    assert_eq!(captures[0].provenance_identity, "source:payments");
    assert_eq!(
        payload
            .capture_authority()
            .expect("capture authority")
            .revision,
        1
    );
    assert_eq!(payload.progress_authority().unwrap().revision, 1);
    assert_eq!(payload.closure_authority().unwrap().revision, 1);
    assert_eq!(payload.completeness_authority().unwrap().revision, 1);
    assert_eq!(payload.verdict().unwrap().identity, "verdict:eligible");
    assert_eq!(payload.settlement().unwrap().identity, "settlement:pending");

    for (exact, lower) in [
        (
            Limits {
                max_depth: first.usage().depth,
                ..Limits::owner_max()
            },
            Limits {
                max_depth: first.usage().depth - 1,
                ..Limits::owner_max()
            },
        ),
        (
            Limits {
                max_string_bytes: first.usage().string_bytes,
                ..Limits::owner_max()
            },
            Limits {
                max_string_bytes: first.usage().string_bytes - 1,
                ..Limits::owner_max()
            },
        ),
        (
            Limits {
                max_visited_fields: first.usage().visited_fields,
                ..Limits::owner_max()
            },
            Limits {
                max_visited_fields: first.usage().visited_fields - 1,
                ..Limits::owner_max()
            },
        ),
        (
            Limits {
                max_capture_bindings: first.usage().capture_bindings,
                ..Limits::owner_max()
            },
            Limits {
                max_capture_bindings: first.usage().capture_bindings - 1,
                ..Limits::owner_max()
            },
        ),
        (
            Limits {
                max_required_sources: first.usage().required_sources,
                ..Limits::owner_max()
            },
            Limits {
                max_required_sources: first.usage().required_sources - 1,
                ..Limits::owner_max()
            },
        ),
    ] {
        authority::activation::derive(context, &selected, exact)
            .expect("exact activation resource bound admits");
        let first_error = authority::activation::derive(context, &selected, lower)
            .expect_err("one-over activation resource bound");
        let repeated_error = authority::activation::derive(context, &selected, lower)
            .expect_err("repeated one-over activation resource bound");
        assert_eq!(first_error, repeated_error);
        assert_eq!(first_error.code(), ErrorCode::ResourceIncomplete);
    }

    let mut output_size = first.bytes().len();
    let output_exact = (0..8)
        .find_map(|_| {
            let limits = Limits {
                max_output_bytes: output_size,
                ..Limits::owner_max()
            };
            let document = authority::activation::derive(context, &selected, limits)
                .expect("candidate exact activation output limit");
            if document.bytes().len() == output_size {
                Some((limits, document))
            } else {
                output_size = document.bytes().len();
                None
            }
        })
        .expect("activation output usage reaches a fixed point");
    let output_lower = Limits {
        max_output_bytes: output_exact.0.max_output_bytes - 1,
        ..Limits::owner_max()
    };
    let first_error = authority::activation::derive(context, &selected, output_lower)
        .expect_err("one-over activation output bound");
    let repeated_error = authority::activation::derive(context, &selected, output_lower)
        .expect_err("repeated one-over activation output bound");
    assert_eq!(first_error, repeated_error);
    assert_eq!(first_error.code(), ErrorCode::ResourceIncomplete);

    let mut input_size = first.bytes().len();
    let input_exact = (0..8)
        .find_map(|_| {
            let limits = Limits {
                max_input_bytes: input_size,
                ..Limits::owner_max()
            };
            let document = authority::activation::derive(context, &selected, limits)
                .expect("candidate exact activation input limit");
            if document.bytes().len() == input_size {
                Some((limits, document))
            } else {
                input_size = document.bytes().len();
                None
            }
        })
        .expect("activation input usage reaches a fixed point");
    authority::activation::read(input_exact.1.bytes(), context, &selected, input_exact.0)
        .expect("exact activation input bound admits");
    let input_lower = Limits {
        max_input_bytes: input_exact.0.max_input_bytes - 1,
        ..input_exact.0
    };
    let first_error =
        authority::activation::read(input_exact.1.bytes(), context, &selected, input_lower)
            .expect_err("one-over activation input bound");
    let repeated_error =
        authority::activation::read(input_exact.1.bytes(), context, &selected, input_lower)
            .expect_err("repeated one-over activation input bound");
    assert_eq!(first_error, repeated_error);
    assert_eq!(first_error.code(), ErrorCode::ResourceIncomplete);

    let mismatched = selection(&qualified, &owner, &subject, TriggerCase::Absent);
    let byte_first = authority::activation::read(
        first.bytes(),
        context,
        &mismatched,
        Limits {
            max_input_bytes: first.bytes().len() - 1,
            ..Limits::owner_max()
        },
    )
    .expect_err("one-over input refuses before mismatched semantic selection");
    assert_eq!(byte_first.code(), ErrorCode::ResourceIncomplete);
    let error =
        authority::activation::read(first.bytes(), context, &mismatched, Limits::owner_max())
            .expect_err("strict reader rejects a different independent selection");
    assert_eq!(error.code(), ErrorCode::ExpectedMismatch);
}

#[trace("TC-008", "FR-008-AC-6")]
#[test]
fn tc008_owner_refuses_foreign_scope_clock_even_when_intervals_agree() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let record = &qualified.records()[0];
    let proofs = authority_proofs(
        &qualified,
        &owner,
        &subject,
        ExecutionState::Closed,
        ExecutionState::Closed,
        EvidenceState::Complete,
        true,
    );
    let foreign = |earliest, latest| {
        EventTimeInterval::new(
            id("clock:foreign"),
            id("1"),
            id("unit:nanosecond"),
            earliest,
            latest,
            Limits::owner_max(),
        )
        .unwrap()
    };
    let selected = Selection::new(
        ActivationSelection::new(
            id("obligation:refund"),
            record.identity.clone(),
            foreign(8, 12),
            vec![Capture::new(
                id("capture:amount"),
                record.identity.clone(),
                id("type:decimal"),
                "12.00".to_owned(),
                id("source:payments"),
            )],
        ),
        ProgressSelection::new(foreign(0, 29), foreign(30, 30)),
        foreign(8, 12),
        foreign(10, 20),
        proofs,
    );
    let error = authority::activation::derive(
        Context::new(History::batch(&qualified), &owner, &subject, 1, None),
        &selected,
        Limits::owner_max(),
    )
    .expect_err("foreign scope clock must refuse before output");
    assert_eq!(error.code(), ErrorCode::AuthorityMismatch);
}

#[trace("TC-008", "FR-008-AC-6")]
#[test]
fn tc008_owner_refuses_cross_wired_source_capture_and_support_authority() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let record = &qualified.records()[0];
    let proofs = authority_proofs(
        &qualified,
        &owner,
        &subject,
        ExecutionState::Closed,
        ExecutionState::Closed,
        EvidenceState::Complete,
        true,
    );
    let foreign_owner = AuthoritySelection {
        definition_identity: owner.definition_identity.clone(),
        definition_revision: id("2"),
        definition_digest: digest(8),
    };
    let foreign_proofs = authority_proofs(
        &qualified,
        &foreign_owner,
        &subject,
        ExecutionState::Closed,
        ExecutionState::Closed,
        EvidenceState::Complete,
        true,
    );
    let base = |activation, progress| {
        Selection::new(
            activation,
            progress,
            interval(8, 12),
            interval(10, 20),
            proofs.clone(),
        )
    };
    let selections = [
        base(
            ActivationSelection::new(
                id("obligation:refund"),
                id("observation:absent-trigger"),
                interval(8, 12),
                vec![Capture::new(
                    id("capture:amount"),
                    record.identity.clone(),
                    id("type:decimal"),
                    "12.00".to_owned(),
                    id("source:payments"),
                )],
            ),
            progress_selection(ExecutionState::Closed),
        ),
        base(
            ActivationSelection::new(
                id("obligation:refund"),
                record.identity.clone(),
                interval(8, 12),
                vec![Capture::new(
                    id("capture:foreign"),
                    record.identity.clone(),
                    id("type:decimal"),
                    "12.00".to_owned(),
                    id("source:payments"),
                )],
            ),
            progress_selection(ExecutionState::Closed),
        ),
        base(
            activation_selection(
                &qualified,
                TriggerCase::Admitted,
                "13.00",
                "source:payments",
            ),
            progress_selection(ExecutionState::Closed),
        ),
        base(
            activation_selection(&qualified, TriggerCase::Admitted, "12.00", "source:foreign"),
            progress_selection(ExecutionState::Closed),
        ),
        base(
            activation_selection(
                &qualified,
                TriggerCase::Admitted,
                "12.00",
                "source:payments",
            ),
            progress_selection(ExecutionState::Closed),
        )
        .with_contributions(
            Some(Contribution::new(
                id("verdict:foreign"),
                vec![id("observation:foreign")],
            )),
            Some(Contribution::new(
                id("settlement:valid"),
                vec![record.identity.clone()],
            )),
        ),
        Selection::new(
            activation_selection(
                &qualified,
                TriggerCase::Admitted,
                "12.00",
                "source:payments",
            ),
            progress_selection(ExecutionState::Closed),
            interval(8, 12),
            interval(10, 20),
            foreign_proofs,
        ),
    ];

    let expected = [
        ErrorCode::MissingPremise,
        ErrorCode::CaptureMismatch,
        ErrorCode::CaptureMismatch,
        ErrorCode::CaptureMismatch,
        ErrorCode::SupportMismatch,
        ErrorCode::AuthorityMismatch,
    ];
    for (selected, expected) in selections.into_iter().zip(expected) {
        let error = authority::activation::derive(context, &selected, Limits::owner_max())
            .expect_err("cross-wired authority must refuse without output");
        assert_eq!(error.code(), expected);
    }
}
