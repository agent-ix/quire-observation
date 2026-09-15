// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-007: partial Boolean values and exact event-time interval relations.

mod support;

use ix_trace_rs::trace;
use quire_observation::authority::{
    self, AuthoritySelection, Context, ErrorCode, History, Limits, SubjectSelection,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, ResourceLimits,
    ScopeKind, ScopeSelection, ValueState, Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
};

use authority::partial::{EventTimeInterval, PossibilitySet, ValueReason};
use support::{order, producer};

fn id(value: &str) -> Identity {
    Identity::new(value)
}

fn interval(earliest: i128, latest: i128) -> EventTimeInterval {
    EventTimeInterval::new(
        id("clock:event-time"),
        id("revision:1"),
        id("unit:nanosecond"),
        earliest,
        latest,
        Limits::owner_max(),
    )
    .expect("valid test interval")
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn qualified() -> Box<QualifiedObservation> {
    let producer = producer();
    let subject = order(&producer, "order:O1");
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id("package:boolean"),
            revision: id("1"),
            digest: digest(1),
        },
        binding: ObservationBinding {
            identity: id("binding:boolean"),
            source_identity: id("source:sensor"),
            schema_identity: id("schema:boolean/v1"),
            signal_identity: id("signal:active"),
            trigger_identity: id("trigger:sample"),
            unit: id("unit:boolean"),
            subject_kind: subject.kind().clone(),
            required: false,
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
            observation_sources: vec![id("source:sensor")],
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
            binding_identity: id("binding:boolean"),
            source_identity: id("source:sensor"),
            schema_identity: id("schema:boolean/v1"),
            subject,
            signal_identity: id("signal:active"),
            trigger_identity: id("trigger:sample"),
            unit: id("unit:boolean"),
            value: ValueState::Missing,
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
        definition_identity: id("definition:partial-owner"),
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

fn fact_interval() -> EventTimeInterval {
    EventTimeInterval::new(
        id("clock:event-time"),
        id("1"),
        id("unit:boolean"),
        8,
        12,
        Limits::owner_max(),
    )
    .expect("valid admitted-record interval")
}

#[trace("TC-007", "FR-007-AC-1")]
#[test]
fn tc007_coherent_value_states_preserve_exact_possibilities_and_reasons() {
    let cases = [
        (false, true, ValueReason::Known),
        (true, false, ValueReason::Known),
        (true, true, ValueReason::Missing),
        (true, true, ValueReason::Conflicting),
    ];

    for (can_be_false, can_be_true, reason) in cases {
        let value = PossibilitySet::new(can_be_false, can_be_true, reason)
            .expect("coherent possibility set");
        assert_eq!(value.can_be_false(), can_be_false);
        assert_eq!(value.can_be_true(), can_be_true);
        assert_eq!(value.reason(), reason);
    }
}

#[trace("TC-007", "FR-007-AC-2")]
#[test]
fn tc007_empty_and_reason_inconsistent_value_sets_have_stable_refusals() {
    let invalid = [
        (false, false, ValueReason::Known),
        (true, false, ValueReason::Missing),
        (false, true, ValueReason::Conflicting),
        (true, true, ValueReason::Known),
    ];

    for (can_be_false, can_be_true, reason) in invalid {
        let error = PossibilitySet::new(can_be_false, can_be_true, reason)
            .expect_err("incoherent possibility set must refuse");
        assert_eq!(error.code(), ErrorCode::InvalidPossibilitySet);
    }
}

#[trace("TC-007", "FR-007-AC-3")]
#[test]
fn tc007_interval_relation_retains_every_and_only_admissible_order() {
    for a_earliest in -3_i128..=3 {
        for a_latest in a_earliest..=3 {
            for b_earliest in -3_i128..=3 {
                for b_latest in b_earliest..=3 {
                    let actual = interval(a_earliest, a_latest)
                        .relation_to(&interval(b_earliest, b_latest), Limits::owner_max())
                        .expect("same-domain intervals compare");
                    assert_eq!(actual.before(), a_earliest < b_latest);
                    assert_eq!(
                        actual.equal(),
                        a_earliest <= b_latest && b_earliest <= a_latest
                    );
                    assert_eq!(actual.after(), a_latest > b_earliest);
                }
            }
        }
    }
}

#[trace("TC-007", "FR-007-AC-4")]
#[test]
fn tc007_reversed_and_cross_wired_intervals_refuse_before_comparison() {
    let reversed = EventTimeInterval::new(
        id("clock:event-time"),
        id("revision:1"),
        id("unit:nanosecond"),
        2,
        1,
        Limits::owner_max(),
    )
    .expect_err("reversed interval must refuse");
    assert_eq!(reversed.code(), ErrorCode::InvalidInterval);

    let mut four_bytes = Limits::owner_max();
    four_bytes.max_string_bytes = 4;
    let oversized = EventTimeInterval::new(
        id("clock:event-time"),
        id("revision:1"),
        id("unit:nanosecond"),
        0,
        1,
        four_bytes,
    )
    .expect_err("oversized interval identity must refuse before retention");
    assert_eq!(oversized.code(), ErrorCode::ResourceIncomplete);

    let comparison_error = interval(0, 1)
        .relation_to(&interval(1, 2), four_bytes)
        .expect_err("lowered limits must apply before comparison");
    assert_eq!(comparison_error.code(), ErrorCode::ResourceIncomplete);

    for cross_wired in [
        EventTimeInterval::new(
            id("clock:other"),
            id("revision:1"),
            id("unit:nanosecond"),
            0,
            1,
            Limits::owner_max(),
        ),
        EventTimeInterval::new(
            id("clock:event-time"),
            id("revision:2"),
            id("unit:nanosecond"),
            0,
            1,
            Limits::owner_max(),
        ),
        EventTimeInterval::new(
            id("clock:event-time"),
            id("revision:1"),
            id("unit:millisecond"),
            0,
            1,
            Limits::owner_max(),
        ),
    ] {
        let error = interval(0, 1)
            .relation_to(
                &cross_wired.expect("internally coherent interval"),
                Limits::owner_max(),
            )
            .expect_err("cross-domain relation must refuse");
        assert_eq!(error.code(), ErrorCode::IntervalDomainMismatch);
    }
}

#[trace("TC-007", "FR-007-AC-5")]
#[test]
fn tc007_ingestion_position_and_midpoint_are_not_semantic_order_inputs() {
    let left = interval(i128::MIN, 10);
    let right = interval(10, i128::MAX);
    let expected = left
        .relation_to(&right, Limits::owner_max())
        .expect("same-domain intervals");

    for ignored_ingestion_position in [i128::MIN, -1, 0, 1, i128::MAX] {
        let left_selection = authority::partial::Selection::new(
            id("observation:left"),
            PossibilitySet::new(false, true, ValueReason::Known).unwrap(),
            interval(i128::MIN, 10),
            ignored_ingestion_position,
        );
        let right_selection = authority::partial::Selection::new(
            id("observation:right"),
            PossibilitySet::new(true, false, ValueReason::Known).unwrap(),
            interval(10, i128::MAX),
            ignored_ingestion_position.saturating_neg(),
        );
        let actual = left_selection
            .relation_to(&right_selection, Limits::owner_max())
            .expect("same-domain intervals");
        assert_eq!(actual, expected);
    }

    let shifted_midpoints = interval(-1_000, 10)
        .relation_to(&interval(10, 1_000), Limits::owner_max())
        .expect("same-domain intervals");
    assert_eq!(shifted_midpoints, expected);

    assert!(expected.before());
    assert!(expected.equal());
    assert!(!expected.after());
}

#[trace("TC-007", "FR-007-AC-1", "FR-007-AC-4")]
#[test]
fn tc007_partial_fact_is_canonical_strictly_read_and_fail_closed() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let selection = authority::partial::Selection::new(
        qualified.records()[0].identity.clone(),
        PossibilitySet::new(true, true, ValueReason::Missing).expect("coherent missing value"),
        fact_interval(),
        15,
    );

    let first = authority::partial::derive(context, &selection, Limits::owner_max())
        .expect("derive partial fact");
    let second = authority::partial::derive(context, &selection, Limits::owner_max())
        .expect("repeat partial fact");
    assert_eq!(first.bytes(), second.bytes());
    let view = authority::partial::read(first.bytes(), context, &selection, Limits::owner_max())
        .expect("strict-read partial fact");
    assert_eq!(
        view.payload().observation_identity(),
        qualified.records()[0].identity.as_str()
    );
    assert!(view.payload().can_be_false());
    assert!(view.payload().can_be_true());
    assert_eq!(view.payload().reason(), ValueReason::Missing);
    assert_eq!(view.payload().interval().earliest, "8");
    assert_eq!(view.payload().interval().latest, "12");
    assert_eq!(view.payload().ingestion_position(), "15");

    let mut one_byte = Limits::owner_max();
    one_byte.max_output_bytes = 1;
    let error = authority::partial::derive(context, &selection, one_byte)
        .expect_err("bounded derivation must fail without a partial fact");
    assert_eq!(error.code(), ErrorCode::ResourceIncomplete);

    let cross_wired = authority::partial::Selection::new(
        qualified.records()[0].identity.clone(),
        PossibilitySet::new(true, true, ValueReason::Missing).expect("coherent missing value"),
        EventTimeInterval::new(
            id("clock:event-time"),
            id("1"),
            id("unit:other"),
            8,
            12,
            Limits::owner_max(),
        )
        .expect("internally valid interval"),
        15,
    );
    let error = authority::partial::derive(context, &cross_wired, Limits::owner_max())
        .expect_err("cross-wired fact must refuse");
    assert_eq!(error.code(), ErrorCode::ExpectedMismatch);
}
