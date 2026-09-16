// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-006: finite relationship topology and exact temporal-boundary anchoring.

mod support;

use std::collections::BTreeSet;

use ix_trace_rs::trace;
use quire_observation::authority::{
    self, AuthoritySelection, Context, History, Limits, OpenClosed, SubjectSelection,
    TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, Relationship,
    RelationshipIdentity, ResourceKind, ResourceLimits, ScopeKind, ScopeSelection, ValueState,
    Visibility, NATIVE_LINKED_PACKAGE_FORMAT,
};
use sha2::{Digest as _, Sha256};
use support::{order, producer, ORDER_SUCCESSOR};

fn id(value: &str) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn seal(mut request: AdmissionRequest) -> AdmissionRequest {
    let record_identity = authority::observation::record_identity(
        &request.records[0],
        authority::Limits::owner_max(),
    )
    .expect("derive the TC-006 record identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, authority::Limits::owner_max())
        .expect("derive the TC-006 population identities");
    request
}

fn request_for(
    scope_kind: ScopeKind,
    range: ClockRange,
    anchor: Anchor,
    clock_identity: &str,
) -> AdmissionRequest {
    let producer = producer();
    let subject = order(&producer, "order:O1");
    seal(AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id("package:topology-boundary"),
            revision: id("1"),
            digest: digest(1),
        },
        binding: ObservationBinding {
            identity: id("binding:topology-boundary"),
            source_identity: id("source:topology-boundary"),
            schema_identity: id("schema:topology-boundary/v1"),
            signal_identity: id("signal:topology-boundary"),
            trigger_identity: id("trigger:topology-boundary"),
            unit: id("unit:exact"),
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
            observation_sources: vec![id("source:topology-boundary")],
            completeness_dependencies: vec![id("completeness:topology-boundary")],
            progress_dependencies: vec![id("progress:topology-boundary")],
            clock_identity: id(clock_identity),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id("closure:topology-boundary")),
            closure_digest: Some(digest(4)),
            kind: scope_kind,
            range,
            members: vec![Member {
                object_identity: id("member:O1"),
                record_identity: id("unsealed-record"),
                anchor,
            }],
        },
        records: vec![AdmittedRecord {
            identity: id("unsealed-record"),
            binding_identity: id("binding:topology-boundary"),
            source_identity: id("source:topology-boundary"),
            schema_identity: id("schema:topology-boundary/v1"),
            subject,
            signal_identity: id("signal:topology-boundary"),
            trigger_identity: id("trigger:topology-boundary"),
            unit: id("unit:exact"),
            value: ValueState::Present {
                value_type: id("integer"),
                canonical_value: "1".to_owned(),
            },
            visibility: Visibility::External,
            anchor,
            event_time_nanos: 0,
            ingestion_time_nanos: 0,
            causal_relationship_identity: None,
            clock_identity: id(clock_identity),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 0,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 0,
            max_required_relationships: 0,
        },
        producer,
    })
}

fn qualified(request: AdmissionRequest) -> Box<QualifiedObservation> {
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("TC-006 fixture admission failed: {other:?}"),
    }
}

fn owner() -> AuthoritySelection {
    AuthoritySelection {
        definition_identity: id("definition:topology-boundary"),
        definition_revision: id("1"),
        definition_digest: digest(9),
    }
}

fn subject_selection(scope_identity: &str, qualified: &QualifiedObservation) -> SubjectSelection {
    SubjectSelection {
        scope_identity: id(scope_identity),
        population_identity: qualified.scope().population_identity.clone(),
    }
}

fn cutoff(clock_identity: &str, clock_revision: &str) -> authority::observation::CutoffSelection {
    authority::observation::CutoffSelection::new(
        id(clock_identity),
        id(clock_revision),
        100,
        authority::observation::CutoffRule::IngestionTimeAtOrBefore,
        id("1"),
    )
}

fn progress_selection(
    clock_identity: &str,
    clock_revision: &str,
    boundary: TemporalBoundary,
    state: OpenClosed,
) -> authority::progress::Selection {
    authority::progress::Selection::new(
        authority::clock::Selection::new(id(clock_identity), id(clock_revision)),
        vec![id("source:topology-boundary")],
        boundary,
        state,
        id("trigger:topology-boundary"),
        cutoff(clock_identity, clock_revision),
        id("restoration:topology-boundary"),
    )
}

fn successor(
    request: &AdmissionRequest,
    identity: &str,
    source_identity: &str,
    target_identity: &str,
) -> Relationship {
    Relationship::new(
        &request.producer,
        ORDER_SUCCESSOR,
        RelationshipIdentity::new(identity).expect("relationship identity is non-empty"),
        order(&request.producer, source_identity),
        order(&request.producer, target_identity),
    )
    .expect("the pinned FCD Order-successor declaration admits this edge")
}

#[trace("TC-006", "FR-006-AC-1", "FR-006-AC-2", "FR-006-AC-3")]
#[test]
fn tc006_retains_loops_cycles_and_disconnected_components_without_inference() {
    let mut request = request_for(
        ScopeKind::Snapshot {
            snapshot_identity: id("snapshot:topology"),
        },
        ClockRange::EventPosition {
            start: 0,
            end_exclusive: 1,
        },
        Anchor::EventPosition(0),
        "clock:topology",
    );
    request.limits.max_relationships = 4;

    let self_loop = successor(&request, "relationship:self", "order:O1", "order:O1");
    let forward = successor(&request, "relationship:forward", "order:O2", "order:O3");
    let backward = successor(&request, "relationship:backward", "order:O3", "order:O2");
    request.relationships.extend([
        self_loop.clone(),
        forward.clone(),
        backward.clone(),
        self_loop.clone(),
    ]);

    let observation = qualified(request);
    assert_eq!(observation.relationships().len(), 3);
    let retained: BTreeSet<_> = observation
        .relationships()
        .iter()
        .map(|edge| {
            (
                edge.identity().as_bytes().to_vec(),
                edge.source().identity().as_bytes().to_vec(),
                edge.target().identity().as_bytes().to_vec(),
            )
        })
        .collect();
    assert_eq!(
        retained,
        BTreeSet::from([
            (
                b"relationship:self".to_vec(),
                b"order:O1".to_vec(),
                b"order:O1".to_vec(),
            ),
            (
                b"relationship:forward".to_vec(),
                b"order:O2".to_vec(),
                b"order:O3".to_vec(),
            ),
            (
                b"relationship:backward".to_vec(),
                b"order:O3".to_vec(),
                b"order:O2".to_vec(),
            ),
        ])
    );

    let mut reordered = request_for(
        ScopeKind::Snapshot {
            snapshot_identity: id("snapshot:topology"),
        },
        ClockRange::EventPosition {
            start: 0,
            end_exclusive: 1,
        },
        Anchor::EventPosition(0),
        "clock:topology",
    );
    reordered.records[0].event_time_nanos = 999;
    reordered.records[0].ingestion_time_nanos = -999;
    reordered.relationships = vec![backward, self_loop, forward];
    reordered.limits.max_relationships = 3;
    let reordered = qualified(seal(reordered));
    assert_eq!(reordered.relationships(), observation.relationships());

    let mut replay_over = request_for(
        ScopeKind::Snapshot {
            snapshot_identity: id("snapshot:topology"),
        },
        ClockRange::EventPosition {
            start: 0,
            end_exclusive: 1,
        },
        Anchor::EventPosition(0),
        "clock:topology",
    );
    replay_over.limits.max_relationships = 1;
    let replay = successor(&replay_over, "relationship:replay", "order:O1", "order:O1");
    replay_over.relationships.extend([replay.clone(), replay]);
    assert!(matches!(
        admit(replay_over),
        AdmissionOutcome::Refused {
            cause: quire_observation::RefusalCause::ResourceLimit {
                limit: ResourceKind::Relationships
            }
        }
    ));
}

#[trace("TC-006", "FR-006-AC-4", "FR-006-AC-6", "FR-006-AC-7")]
#[test]
fn tc006_timestamp_boundary_uses_exact_scope_clock_and_half_open_carrier() {
    let exact_boundary = TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 30,
        carrier_end_exclusive_nanos: 31,
        watermark_nanos: 31,
    };
    for anchor in [0, 30] {
        let qualified = qualified(request_for(
            ScopeKind::Window {
                window_identity: id("window:timestamp"),
            },
            ClockRange::Timestamp {
                start_nanos: 0,
                end_nanos: 31,
            },
            Anchor::TimestampNanos(anchor),
            "clock:timestamp",
        ));
        let owner = owner();
        let subject = subject_selection("window:timestamp", &qualified);
        let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
        authority::progress::derive(
            context,
            &progress_selection("clock:timestamp", "1", exact_boundary, OpenClosed::Closed),
            Limits::owner_max(),
        )
        .expect("inclusive start and upper endpoint map to the exact carrier");
    }

    let at_carrier_end = request_for(
        ScopeKind::Window {
            window_identity: id("window:timestamp"),
        },
        ClockRange::Timestamp {
            start_nanos: 0,
            end_nanos: 31,
        },
        Anchor::TimestampNanos(31),
        "clock:timestamp",
    );
    assert!(matches!(
        admit(at_carrier_end),
        AdmissionOutcome::Refused {
            cause: quire_observation::RefusalCause::ClockMismatch { .. }
        }
    ));

    let slack = qualified(request_for(
        ScopeKind::Window {
            window_identity: id("window:timestamp"),
        },
        ClockRange::Timestamp {
            start_nanos: 0,
            end_nanos: 31,
        },
        Anchor::TimestampNanos(29),
        "clock:timestamp",
    ));
    let owner = owner();
    let subject = subject_selection("window:timestamp", &slack);
    let context = Context::new(History::batch(&slack), &owner, &subject, 1, None);
    authority::closure::derive(
        context,
        &authority::closure::Selection::new(
            id("clock:timestamp"),
            id("1"),
            vec![id("source:topology-boundary")],
            TemporalBoundary::TimestampedEvent {
                lower_nanos: 0,
                upper_inclusive_nanos: 20,
                carrier_end_exclusive_nanos: 31,
                watermark_nanos: 21,
            },
            OpenClosed::Closed,
        ),
        Limits::owner_max(),
    )
    .expect("an explicit timestamp carrier may extend beyond the native upper bound");

    for (selection, expected) in [
        (
            progress_selection("clock:foreign", "1", exact_boundary, OpenClosed::Closed),
            authority::ErrorCode::ExpectedMismatch,
        ),
        (
            progress_selection("clock:timestamp", "2", exact_boundary, OpenClosed::Closed),
            authority::ErrorCode::ExpectedMismatch,
        ),
        (
            progress_selection(
                "clock:timestamp",
                "1",
                TemporalBoundary::EventPosition {
                    lower: 0,
                    upper_inclusive: 29,
                    carrier_end_exclusive: 30,
                    watermark: 30,
                },
                OpenClosed::Closed,
            ),
            authority::ErrorCode::ExpectedMismatch,
        ),
        (
            progress_selection(
                "clock:timestamp",
                "1",
                TemporalBoundary::TimestampedEvent {
                    lower_nanos: 1,
                    upper_inclusive_nanos: 30,
                    carrier_end_exclusive_nanos: 31,
                    watermark_nanos: 31,
                },
                OpenClosed::Closed,
            ),
            authority::ErrorCode::ExpectedMismatch,
        ),
        (
            progress_selection(
                "clock:timestamp",
                "1",
                TemporalBoundary::TimestampedEvent {
                    lower_nanos: 0,
                    upper_inclusive_nanos: 30,
                    carrier_end_exclusive_nanos: 32,
                    watermark_nanos: 31,
                },
                OpenClosed::Closed,
            ),
            authority::ErrorCode::ExpectedMismatch,
        ),
        (
            progress_selection(
                "clock:timestamp",
                "1",
                TemporalBoundary::TimestampedEvent {
                    lower_nanos: 0,
                    upper_inclusive_nanos: 30,
                    carrier_end_exclusive_nanos: 31,
                    watermark_nanos: 30,
                },
                OpenClosed::Closed,
            ),
            authority::ErrorCode::InvalidSelection,
        ),
    ] {
        assert_eq!(
            authority::progress::derive(context, &selection, Limits::owner_max())
                .expect_err("a cross-wired or non-covering timestamp boundary must refuse")
                .code(),
            expected
        );
    }

    let wrong_subject = subject_selection("window:foreign", &slack);
    let error = authority::progress::derive(
        Context::new(History::batch(&slack), &owner, &wrong_subject, 1, None),
        &progress_selection("clock:timestamp", "1", exact_boundary, OpenClosed::Closed),
        Limits::owner_max(),
    )
    .expect_err("a foreign scope identity must refuse");
    assert_eq!(error.code(), authority::ErrorCode::ExpectedMismatch);
}

#[trace("TC-006", "FR-006-AC-5", "FR-006-AC-7")]
#[test]
fn tc006_discrete_boundaries_require_checked_successor_and_admit_points() {
    let event = qualified(request_for(
        ScopeKind::Snapshot {
            snapshot_identity: id("snapshot:event-point"),
        },
        ClockRange::EventPosition {
            start: 4,
            end_exclusive: 5,
        },
        Anchor::EventPosition(4),
        "clock:event-position",
    ));
    let owner = owner();
    let subject = subject_selection("snapshot:event-point", &event);
    let context = Context::new(History::batch(&event), &owner, &subject, 1, None);
    authority::progress::derive(
        context,
        &progress_selection(
            "clock:event-position",
            "1",
            TemporalBoundary::EventPosition {
                lower: 4,
                upper_inclusive: 4,
                carrier_end_exclusive: 5,
                watermark: 5,
            },
            OpenClosed::Closed,
        ),
        Limits::owner_max(),
    )
    .expect("one event position is a valid inclusive point interval");

    for boundary in [
        TemporalBoundary::EventPosition {
            lower: 4,
            upper_inclusive: 4,
            carrier_end_exclusive: 6,
            watermark: 6,
        },
        TemporalBoundary::EventPosition {
            lower: 4,
            upper_inclusive: 4,
            carrier_end_exclusive: 5,
            watermark: 4,
        },
    ] {
        assert_eq!(
            authority::progress::derive(
                context,
                &progress_selection("clock:event-position", "1", boundary, OpenClosed::Closed,),
                Limits::owner_max(),
            )
            .expect_err("a non-successor carrier or equal point watermark must refuse")
            .code(),
            authority::ErrorCode::InvalidSelection
        );
    }

    let sample = qualified(request_for(
        ScopeKind::Snapshot {
            snapshot_identity: id("snapshot:sample-point"),
        },
        ClockRange::FixedSample {
            start: 7,
            end_exclusive: 8,
            epoch_nanos: 100,
            period_nanos: 5,
        },
        Anchor::FixedSample {
            index: 7,
            epoch_nanos: 100,
            period_nanos: 5,
        },
        "clock:fixed-sample",
    ));
    let subject = subject_selection("snapshot:sample-point", &sample);
    let context = Context::new(History::batch(&sample), &owner, &subject, 1, None);
    authority::closure::derive(
        context,
        &authority::closure::Selection::new(
            id("clock:fixed-sample"),
            id("1"),
            vec![id("source:topology-boundary")],
            TemporalBoundary::FixedSample {
                lower: 7,
                upper_inclusive: 7,
                carrier_end_exclusive: 8,
                watermark: 8,
            },
            OpenClosed::Closed,
        ),
        Limits::owner_max(),
    )
    .expect("one fixed sample is a valid inclusive point interval");

    let overflow = qualified(request_for(
        ScopeKind::Snapshot {
            snapshot_identity: id("snapshot:overflow"),
        },
        ClockRange::EventPosition {
            start: u64::MAX - 1,
            end_exclusive: u64::MAX,
        },
        Anchor::EventPosition(u64::MAX - 1),
        "clock:overflow",
    ));
    let subject = subject_selection("snapshot:overflow", &overflow);
    let context = Context::new(History::batch(&overflow), &owner, &subject, 1, None);
    let error = authority::progress::derive(
        context,
        &progress_selection(
            "clock:overflow",
            "1",
            TemporalBoundary::EventPosition {
                lower: u64::MAX - 1,
                upper_inclusive: u64::MAX,
                carrier_end_exclusive: u64::MAX,
                watermark: 0,
            },
            OpenClosed::Open,
        ),
        Limits::owner_max(),
    )
    .expect_err("an overflowing discrete successor must refuse");
    assert_eq!(error.code(), authority::ErrorCode::InvalidSelection);

    let reversed = progress_selection(
        "clock:event-position",
        "1",
        TemporalBoundary::EventPosition {
            lower: 5,
            upper_inclusive: 4,
            carrier_end_exclusive: 5,
            watermark: 5,
        },
        OpenClosed::Closed,
    );
    let subject = subject_selection("snapshot:event-point", &event);
    let context = Context::new(History::batch(&event), &owner, &subject, 1, None);
    assert_eq!(
        authority::progress::derive(context, &reversed, Limits::owner_max())
            .expect_err("a reversed discrete interval must refuse")
            .code(),
        authority::ErrorCode::InvalidSelection
    );
}

#[trace("TC-006", "FR-006-AC-8")]
#[test]
fn tc006_progress_and_closure_v1_schemas_remain_immutable() {
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(
                "../schemas/observation-progress-assertion-v1.schema.json"
            ))
        ),
        "91223fb982b68d4fb7c66b3737370429bd4841321f5b561ebc6c1e9691bd7f1c"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(
                "../schemas/observation-closure-assertion-v1.schema.json"
            ))
        ),
        "8638fb9f4dd5a26d44ae9975388fc734670992914422e23c27eaae82149245f4"
    );
}
