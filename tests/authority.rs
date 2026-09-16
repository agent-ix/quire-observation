// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-004: canonical observation-owner artifacts.

mod support;

use ix_trace_rs::trace;
use proptest::prelude::*;
use quire_observation::authority::{
    self, AuthoritySelection, Context, History, IncrementalHistory, Limits, OpenClosed,
    SubjectSelection, TemporalBoundary,
};
use quire_observation::{
    admit, AdmissionOutcome, AdmissionRequest, AdmittedRecord, Anchor, ClockRange, Digest,
    Identity, Member, ObservationBinding, PackageSelection, QualifiedObservation, Relationship,
    RelationshipIdentity, ResourceLimits, ScopeKind, ScopeSelection, ValueState, Visibility,
    NATIVE_LINKED_PACKAGE_FORMAT, PRODUCER_INTERFACE_VERSION,
};
use sha2::{Digest as _, Sha256};
use support::{order, producer, shipment, ORDER_SHIPMENT};

fn id(value: &str) -> Identity {
    Identity::new(value)
}

fn digest(value: u8) -> Digest {
    Digest::new([value; 32])
}

fn qualified() -> Box<QualifiedObservation> {
    qualified_with_window("window:O1")
}

fn qualified_with_window(window_identity: &str) -> Box<QualifiedObservation> {
    qualified_with_window_and_closure(window_identity, "closure-definition:windows")
}

fn qualified_with_window_and_closure(
    window_identity: &str,
    closure_definition_identity: &str,
) -> Box<QualifiedObservation> {
    let producer = producer();
    let subject = order(&producer, "order:O1");
    let causal_identity = RelationshipIdentity::new("relationship:O1-S1").unwrap();
    let relationship = Relationship::new(
        &producer,
        ORDER_SHIPMENT,
        causal_identity.clone(),
        subject.clone(),
        shipment(&producer, "shipment:S1"),
    )
    .unwrap();
    let mut request = AdmissionRequest {
        package: PackageSelection {
            format: NATIVE_LINKED_PACKAGE_FORMAT.to_owned(),
            identity: id("package:refund"),
            revision: id("1"),
            digest: digest(1),
        },
        binding: ObservationBinding {
            identity: id("binding:refund"),
            source_identity: id("source:payments"),
            schema_identity: id("schema:refund/v1"),
            signal_identity: id("signal:refund"),
            trigger_identity: id("trigger:refund-request"),
            unit: id("USD"),
            subject_kind: subject.kind().clone(),
            required: true,
        },
        expected_subject: subject.clone(),
        relationships: vec![relationship],
        required_relationships: vec![],
        scope: ScopeSelection {
            population_identity: id("unsealed-population"),
            membership_rule_identity: id("unsealed-membership"),
            membership_digest: digest(0),
            membership_document: vec![],
            required_member_identities: vec![id("member:O1")],
            observation_sources: vec![id("source:payments")],
            completeness_dependencies: vec![id("completeness:O1")],
            progress_dependencies: vec![id("progress:O1")],
            clock_identity: id("clock:event-time"),
            clock_revision: id("1"),
            membership_complete: true,
            closure_identity: Some(id(closure_definition_identity)),
            closure_digest: Some(digest(4)),
            kind: ScopeKind::Window {
                window_identity: id(window_identity),
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
            binding_identity: id("binding:refund"),
            source_identity: id("source:payments"),
            schema_identity: id("schema:refund/v1"),
            subject,
            signal_identity: id("signal:refund"),
            trigger_identity: id("trigger:refund-request"),
            unit: id("USD"),
            value: ValueState::Present {
                value_type: id("decimal"),
                canonical_value: "12.00".to_owned(),
            },
            visibility: Visibility::External,
            anchor: Anchor::TimestampNanos(10),
            event_time_nanos: 10,
            ingestion_time_nanos: 11,
            causal_relationship_identity: Some(causal_identity),
            clock_identity: id("clock:event-time"),
            clock_revision: id("1"),
            clock_uncertainty_nanos: 2,
        }],
        limits: ResourceLimits {
            max_records: 1,
            max_members: 1,
            max_relationships: 1,
            max_required_relationships: 0,
        },
        producer,
    };
    let record_identity =
        authority::observation::record_identity(&request.records[0], Limits::owner_max())
            .expect("derive FR-288 record identity");
    request.records[0].identity = record_identity.clone();
    request.scope.members[0].record_identity = record_identity;
    authority::population::assign_request_identities(&mut request, Limits::owner_max())
        .expect("derive FR-263/FR-264 identities");
    match admit(request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("fixture admission failed: {other:?}"),
    }
}

fn owner() -> AuthoritySelection {
    AuthoritySelection {
        definition_identity: id("definition:qobs-owner"),
        definition_revision: id("1"),
        definition_digest: digest(9),
    }
}

fn subject(qualified: &QualifiedObservation) -> SubjectSelection {
    let scope_identity = match &qualified.scope().kind {
        ScopeKind::Snapshot { snapshot_identity } => snapshot_identity.clone(),
        ScopeKind::Window { window_identity } => window_identity.clone(),
    };
    SubjectSelection {
        scope_identity,
        population_identity: qualified.scope().population_identity.clone(),
    }
}

fn selected_subject(qualified: &QualifiedObservation) -> SubjectSelection {
    subject(qualified)
}

fn boundary(state: OpenClosed) -> TemporalBoundary {
    TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: if state == OpenClosed::Closed { 30 } else { 20 },
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

fn partial_selection(record: &AdmittedRecord) -> authority::partial::Selection {
    authority::partial::Selection::new(
        record.identity.clone(),
        authority::partial::PossibilitySet::new(
            false,
            true,
            authority::partial::ValueReason::Known,
        )
        .expect("coherent known value"),
        authority::partial::EventTimeInterval::new(
            id("clock:event-time"),
            id("1"),
            id("USD"),
            8,
            12,
            Limits::owner_max(),
        )
        .expect("valid partial interval"),
        11,
    )
}

fn activation_interval(earliest: i128, latest: i128) -> authority::partial::EventTimeInterval {
    authority::partial::EventTimeInterval::new(
        id("clock:event-time"),
        id("1"),
        id("USD"),
        earliest,
        latest,
        Limits::owner_max(),
    )
    .expect("valid activation interval")
}

fn activation_selection(
    record: &AdmittedRecord,
    proofs: authority::activation::AuthorityProofs,
) -> authority::activation::Selection {
    authority::activation::Selection::new(
        authority::activation::ActivationSelection::new(
            id("obligation:refund"),
            record.identity.clone(),
            activation_interval(8, 12),
            vec![authority::activation::Capture::new(
                id("capture:refund-amount"),
                record.identity.clone(),
                id("decimal"),
                "12.00".to_owned(),
                id("source:payments"),
            )],
            authority::activation::ActivationState::Active,
        ),
        authority::activation::ProgressSelection::new(
            activation_interval(0, 29),
            activation_interval(30, 30),
        ),
        activation_interval(8, 12),
        activation_interval(10, 20),
        proofs,
    )
}

fn capture_selection(record: &AdmittedRecord) -> authority::capture::Selection {
    authority::capture::Selection::new(
        id("trigger:refund-request"),
        Anchor::TimestampNanos(10),
        vec![authority::capture::Binding::new(
            id("capture:refund-amount"),
            id("binding:refund"),
            record.identity.clone(),
        )],
    )
}

fn progress_authority_selection() -> authority::progress::Selection {
    authority::progress::Selection::new(
        authority::clock::Selection::new(id("clock:event-time"), id("1")),
        vec![id("source:payments")],
        boundary(OpenClosed::Closed),
        OpenClosed::Closed,
        id("trigger:refund-request"),
        cutoff(40),
        id("restoration:O1"),
    )
}

fn closure_selection() -> authority::closure::Selection {
    authority::closure::Selection::new(
        id("clock:event-time"),
        id("1"),
        vec![id("source:payments")],
        boundary(OpenClosed::Closed),
        OpenClosed::Closed,
    )
}

fn completeness_selection(record: &AdmittedRecord) -> authority::completeness::Selection {
    authority::completeness::Selection::new(
        id("boundary:O1"),
        vec![authority::completeness::Fact::new(
            id("member:O1"),
            Some(record.identity.clone()),
            authority::completeness::FactStatus::Available,
        )],
    )
}

fn activation_proofs(
    context: Context<'_>,
    record: &AdmittedRecord,
    limits: Limits,
) -> authority::activation::AuthorityProofs {
    let capture_selection = capture_selection(record);
    let capture_document = authority::capture::derive(context, &capture_selection, limits)
        .expect("derive capture proof");
    let capture = authority::capture::read(
        capture_document.bytes(),
        context,
        &capture_selection,
        limits,
    )
    .expect("strict-read capture proof");
    let progress_selection = progress_authority_selection();
    let progress_document = authority::progress::derive(context, &progress_selection, limits)
        .expect("derive progress proof");
    let progress = authority::progress::read(
        progress_document.bytes(),
        context,
        &progress_selection,
        limits,
    )
    .expect("strict-read progress proof");
    let closure_selection = closure_selection();
    let closure_document = authority::closure::derive(context, &closure_selection, limits)
        .expect("derive closure proof");
    let closure = authority::closure::read(
        closure_document.bytes(),
        context,
        &closure_selection,
        limits,
    )
    .expect("strict-read closure proof");
    let completeness_selection = completeness_selection(record);
    let completeness_document =
        authority::completeness::derive(context, &completeness_selection, limits)
            .expect("derive completeness proof");
    let completeness = authority::completeness::read(
        completeness_document.bytes(),
        context,
        &completeness_selection,
        limits,
    )
    .expect("strict-read completeness proof");
    authority::activation::AuthorityProofs::new(
        &capture,
        Some(&progress),
        Some(&closure),
        Some(&completeness),
    )
    .expect("compose strict-read activation proofs")
}

struct Documents {
    activation: authority::Document,
    observation: authority::Document,
    population: authority::Document,
    position: authority::Document,
    clock: authority::Document,
    partial: authority::Document,
    capture: authority::Document,
    progress: authority::Document,
    closure: authority::Document,
    completeness: authority::Document,
    availability: authority::Document,
}

#[derive(Clone, Copy)]
enum ArtifactKind {
    Activation,
    Observation,
    Population,
    Position,
    Clock,
    Partial,
    Capture,
    Progress,
    Closure,
    Completeness,
    Availability,
}

fn derive_all(
    history: History<'_>,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
) -> Documents {
    let limits = Limits::owner_max();
    let context = Context::new(history, owner, subject, 1, None);
    let record = &history.qualified().records()[0];
    let proofs = activation_proofs(context, record, limits);
    Documents {
        activation: authority::activation::derive(
            context,
            &activation_selection(record, proofs),
            limits,
        )
        .expect("derive activation authority"),
        observation: authority::observation::derive(
            context,
            &authority::observation::Selection::new(record.identity.clone(), cutoff(40)),
            limits,
        )
        .expect("derive record"),
        population: authority::population::derive(context, limits).expect("derive population"),
        position: authority::position::derive(
            context,
            &authority::position::Selection::new(
                id("ledger:O1"),
                id("clock:event-time"),
                id("1"),
                vec![authority::position::Position::new(
                    0,
                    record.identity.clone(),
                )],
            ),
            limits,
        )
        .expect("derive position ledger"),
        clock: authority::clock::derive(
            context,
            &authority::clock::Selection::new(id("clock:event-time"), id("1")),
            limits,
        )
        .expect("derive clock"),
        partial: authority::partial::derive(context, &partial_selection(record), limits)
            .expect("derive partial fact"),
        capture: authority::capture::derive(context, &capture_selection(record), limits)
            .expect("derive capture"),
        progress: authority::progress::derive(context, &progress_authority_selection(), limits)
            .expect("derive progress"),
        closure: authority::closure::derive(context, &closure_selection(), limits)
            .expect("derive closure"),
        completeness: authority::completeness::derive(
            context,
            &completeness_selection(record),
            limits,
        )
        .expect("derive completeness"),
        availability: authority::availability::derive(
            context,
            &authority::availability::Selection::new(
                vec![id("result:O1")],
                vec![id("result:O1")],
                authority::availability::DependencyState::Available,
                authority::availability::DependencyState::Available,
            ),
            limits,
        )
        .expect("derive availability"),
    }
}

fn derive_revised_all(
    history: History<'_>,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
    prior: &Documents,
) -> Documents {
    let limits = Limits::owner_max();
    let record = &history.qualified().records()[0];

    let capture_context = Context::new(history, owner, subject, 2, Some(&prior.capture));
    let capture_selection = capture_selection(record);
    let capture = authority::capture::derive(capture_context, &capture_selection, limits)
        .expect("derive revised capture");
    let capture_view =
        authority::capture::read(capture.bytes(), capture_context, &capture_selection, limits)
            .expect("read revised capture");

    let progress_context = Context::new(history, owner, subject, 2, Some(&prior.progress));
    let progress_selection = progress_authority_selection();
    let progress = authority::progress::derive(progress_context, &progress_selection, limits)
        .expect("derive revised progress");
    let progress_view = authority::progress::read(
        progress.bytes(),
        progress_context,
        &progress_selection,
        limits,
    )
    .expect("read revised progress");

    let closure_context = Context::new(history, owner, subject, 2, Some(&prior.closure));
    let closure_selection = closure_selection();
    let closure = authority::closure::derive(closure_context, &closure_selection, limits)
        .expect("derive revised closure");
    let closure_view =
        authority::closure::read(closure.bytes(), closure_context, &closure_selection, limits)
            .expect("read revised closure");

    let completeness_context = Context::new(history, owner, subject, 2, Some(&prior.completeness));
    let completeness_selection = completeness_selection(record);
    let completeness =
        authority::completeness::derive(completeness_context, &completeness_selection, limits)
            .expect("derive revised completeness");
    let completeness_view = authority::completeness::read(
        completeness.bytes(),
        completeness_context,
        &completeness_selection,
        limits,
    )
    .expect("read revised completeness");
    let proofs = authority::activation::AuthorityProofs::new(
        &capture_view,
        Some(&progress_view),
        Some(&closure_view),
        Some(&completeness_view),
    )
    .expect("compose revised strict-read activation proofs");

    Documents {
        activation: authority::activation::derive(
            Context::new(history, owner, subject, 2, Some(&prior.activation)),
            &activation_selection(record, proofs),
            limits,
        )
        .expect("derive revised activation"),
        observation: authority::observation::derive(
            Context::new(history, owner, subject, 2, Some(&prior.observation)),
            &authority::observation::Selection::new(record.identity.clone(), cutoff(40)),
            limits,
        )
        .expect("derive revised observation"),
        population: authority::population::derive(
            Context::new(history, owner, subject, 2, Some(&prior.population)),
            limits,
        )
        .expect("derive revised population"),
        position: authority::position::derive(
            Context::new(history, owner, subject, 2, Some(&prior.position)),
            &authority::position::Selection::new(
                id("ledger:O1"),
                id("clock:event-time"),
                id("1"),
                vec![authority::position::Position::new(
                    0,
                    record.identity.clone(),
                )],
            ),
            limits,
        )
        .expect("derive revised position"),
        clock: authority::clock::derive(
            Context::new(history, owner, subject, 2, Some(&prior.clock)),
            &authority::clock::Selection::new(id("clock:event-time"), id("1")),
            limits,
        )
        .expect("derive revised clock"),
        partial: authority::partial::derive(
            Context::new(history, owner, subject, 2, Some(&prior.partial)),
            &partial_selection(record),
            limits,
        )
        .expect("derive revised partial"),
        capture,
        progress,
        closure,
        completeness,
        availability: authority::availability::derive(
            Context::new(history, owner, subject, 2, Some(&prior.availability)),
            &authority::availability::Selection::new(
                vec![id("result:O1")],
                vec![],
                authority::availability::DependencyState::Available,
                authority::availability::DependencyState::Available,
            ),
            limits,
        )
        .expect("derive revised availability"),
    }
}

fn reader_accepts(
    kind: ArtifactKind,
    bytes: &[u8],
    qualified: &QualifiedObservation,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
) -> bool {
    let context = Context::new(History::batch(qualified), owner, subject, 1, None);
    let record = &qualified.records()[0];
    let limits = Limits::owner_max();
    match kind {
        ArtifactKind::Activation => {
            let proofs = activation_proofs(context, record, limits);
            authority::activation::read(
                bytes,
                context,
                &activation_selection(record, proofs),
                limits,
            )
            .is_ok()
        }
        ArtifactKind::Observation => authority::observation::read(
            bytes,
            context,
            &authority::observation::Selection::new(record.identity.clone(), cutoff(40)),
            limits,
        )
        .is_ok(),
        ArtifactKind::Population => authority::population::read(bytes, context, limits).is_ok(),
        ArtifactKind::Position => authority::position::read(
            bytes,
            context,
            &authority::position::Selection::new(
                id("ledger:O1"),
                id("clock:event-time"),
                id("1"),
                vec![authority::position::Position::new(
                    0,
                    record.identity.clone(),
                )],
            ),
            limits,
        )
        .is_ok(),
        ArtifactKind::Clock => authority::clock::read(
            bytes,
            context,
            &authority::clock::Selection::new(id("clock:event-time"), id("1")),
            limits,
        )
        .is_ok(),
        ArtifactKind::Partial => {
            authority::partial::read(bytes, context, &partial_selection(record), limits).is_ok()
        }
        ArtifactKind::Capture => authority::capture::read(
            bytes,
            context,
            &authority::capture::Selection::new(
                id("trigger:refund-request"),
                Anchor::TimestampNanos(10),
                vec![authority::capture::Binding::new(
                    id("capture:refund-amount"),
                    id("binding:refund"),
                    record.identity.clone(),
                )],
            ),
            limits,
        )
        .is_ok(),
        ArtifactKind::Progress => authority::progress::read(
            bytes,
            context,
            &authority::progress::Selection::new(
                authority::clock::Selection::new(id("clock:event-time"), id("1")),
                vec![id("source:payments")],
                boundary(OpenClosed::Closed),
                OpenClosed::Closed,
                id("trigger:refund-request"),
                cutoff(40),
                id("restoration:O1"),
            ),
            limits,
        )
        .is_ok(),
        ArtifactKind::Closure => authority::closure::read(
            bytes,
            context,
            &authority::closure::Selection::new(
                id("clock:event-time"),
                id("1"),
                vec![id("source:payments")],
                boundary(OpenClosed::Closed),
                OpenClosed::Closed,
            ),
            limits,
        )
        .is_ok(),
        ArtifactKind::Completeness => authority::completeness::read(
            bytes,
            context,
            &authority::completeness::Selection::new(
                id("boundary:O1"),
                vec![authority::completeness::Fact::new(
                    id("member:O1"),
                    Some(record.identity.clone()),
                    authority::completeness::FactStatus::Available,
                )],
            ),
            limits,
        )
        .is_ok(),
        ArtifactKind::Availability => authority::availability::read(
            bytes,
            context,
            &authority::availability::Selection::new(
                vec![id("result:O1")],
                vec![id("result:O1")],
                authority::availability::DependencyState::Available,
                authority::availability::DependencyState::Available,
            ),
            limits,
        )
        .is_ok(),
    }
}

fn remove_json_field(bytes: &[u8], payload: bool, field: &str) -> Vec<u8> {
    let mut value: serde_json::Value = serde_json::from_slice(bytes).expect("fixture JSON");
    let object = if payload {
        value["payload"].as_object_mut().expect("payload object")
    } else {
        value.as_object_mut().expect("envelope object")
    };
    assert!(object.remove(field).is_some(), "fixture field {field}");
    serde_json::to_vec(&value).expect("serialize mutation")
}

fn duplicate_json_field(bytes: &[u8], payload: bool, field: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_slice(bytes).expect("fixture JSON");
    let field_value = if payload {
        value["payload"]
            .as_object()
            .and_then(|object| object.get(field))
    } else {
        value.as_object().and_then(|object| object.get(field))
    }
    .expect("fixture field");
    let member = format!(
        "\"{field}\":{},",
        serde_json::to_string(field_value).expect("serialize field")
    );
    let insertion = if payload {
        let marker = b"\"payload\":{";
        bytes
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("payload marker")
            + marker.len()
    } else {
        1
    };
    let mut mutated = Vec::with_capacity(bytes.len() + member.len());
    mutated.extend_from_slice(&bytes[..insertion]);
    mutated.extend_from_slice(member.as_bytes());
    mutated.extend_from_slice(&bytes[insertion..]);
    mutated
}

fn insert_unknown_json_field(bytes: &[u8], payload: bool) -> Vec<u8> {
    let insertion = if payload {
        let marker = b"\"payload\":{";
        bytes
            .windows(marker.len())
            .position(|window| window == marker)
            .expect("payload marker")
            + marker.len()
    } else {
        1
    };
    let member = b"\"unknown_owner_field\":null,";
    let mut mutated = Vec::with_capacity(bytes.len() + member.len());
    mutated.extend_from_slice(&bytes[..insertion]);
    mutated.extend_from_slice(member);
    mutated.extend_from_slice(&bytes[insertion..]);
    mutated
}

fn replace_once(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let start = bytes
        .windows(from.len())
        .position(|window| window == from)
        .expect("mutation target");
    let mut mutated = Vec::with_capacity(bytes.len() - from.len() + to.len());
    mutated.extend_from_slice(&bytes[..start]);
    mutated.extend_from_slice(to);
    mutated.extend_from_slice(&bytes[start + from.len()..]);
    mutated
}

#[trace("TC-004", "FR-004-AC-1")]
#[test]
fn tc004_all_eleven_owner_contracts_derive_canonical_documents() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let documents = derive_all(History::batch(&qualified), &owner, &subject);
    assert_eq!(
        qualified.scope().membership_rule_identity.as_str(),
        "sha256-jcs:51daded8bd0ade3b3ce5addc3e5e58318cf661dad0e3f80b8a58be1e33eb5ccb"
    );
    let contracts = [
        documents.activation.contract(),
        documents.observation.contract(),
        documents.population.contract(),
        documents.position.contract(),
        documents.clock.contract(),
        documents.partial.contract(),
        documents.capture.contract(),
        documents.progress.contract(),
        documents.closure.contract(),
        documents.completeness.contract(),
        documents.availability.contract(),
    ];
    assert_eq!(contracts.len(), 11);
    assert!(contracts
        .iter()
        .all(|contract| contract.starts_with("quire.observation.")));
    for document in [
        &documents.activation,
        &documents.observation,
        &documents.population,
        &documents.position,
        &documents.clock,
        &documents.partial,
        &documents.capture,
        &documents.progress,
        &documents.closure,
        &documents.completeness,
        &documents.availability,
    ] {
        assert!(document.identity().as_str().starts_with("sha256:"));
        assert_eq!(document.bytes().len(), document.usage().wire_bytes);
    }
}

#[trace("TC-004", "FR-004-AC-3")]
#[test]
fn tc004_error_code_catalog_is_closed_and_round_trips_exactly() {
    let codes = authority::ErrorCode::all();
    assert_eq!(codes.len(), 25);
    let mut labels = codes
        .iter()
        .map(|code| {
            assert_eq!(authority::ErrorCode::from_code(code.as_str()), Some(*code));
            code.as_str()
        })
        .collect::<Vec<_>>();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), codes.len());
    assert_eq!(
        authority::ErrorCode::from_code("qobs-auth-invalid-json"),
        None
    );
    assert_eq!(authority::ErrorCode::from_code("QOBS-AUTH-UNKNOWN"), None);
}

#[trace("TC-004", "FR-004-AC-4")]
#[test]
fn tc004_owner_resource_maxima_are_pinned() {
    assert_eq!(
        Limits::owner_max(),
        Limits {
            max_input_bytes: 8_388_608,
            max_output_bytes: 8_388_608,
            max_depth: 64,
            max_string_bytes: 1_048_576,
            max_population_entries: 10_000,
            max_positions: 10_000,
            max_capture_bindings: 10_000,
            max_required_sources: 10_000,
            max_bundle_components: 10_000,
            max_replacements: 10_000,
            max_conflicts: 10_000,
            max_lineage_children: 10_000,
            max_visited_fields: 1_000_000,
        }
    );
}

#[trace(
    "TC-003",
    "FR-003-AC-1",
    "FR-003-AC-2",
    "FR-003-AC-3",
    "TC-004",
    "FR-004-AC-1",
    "FR-004-AC-3",
    "IT-001"
)]
#[test]
// Trace: NFR-002-AC-2, IT-001
fn tc004_each_public_reader_requires_exact_independent_selections() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let record = &qualified.records()[0];
    let documents = derive_all(history, &owner, &subject);
    assert!(authority::population::read_membership(
        &qualified.scope().membership_document,
        &qualified.scope().required_member_identities,
        Limits::owner_max(),
    )
    .is_ok());
    let observation_view = authority::observation::read(
        documents.observation.bytes(),
        context,
        &authority::observation::Selection::new(record.identity.clone(), cutoff(40)),
        Limits::owner_max(),
    )
    .expect("strict-read observation");
    assert_eq!(observation_view.authority(), &owner);
    assert_eq!(observation_view.subject(), &subject);
    assert_eq!(observation_view.revision(), 1);
    assert_eq!(observation_view.predecessor(), None);
    let observation = observation_view.payload();
    assert_eq!(observation.observation_identity(), record.identity.as_str());
    assert_eq!(observation.binding_identity(), "binding:refund");
    assert_eq!(observation.source_identity(), "source:payments");
    assert_eq!(observation.schema_identity(), "schema:refund/v1");
    assert_eq!(observation.subject().kind, support::ORDER_KIND.as_bytes());
    assert_eq!(observation.subject().identity, b"order:O1");
    assert_eq!(
        observation.subject().authority.identity,
        qualified.producer().bundle_identity()
    );
    assert_eq!(
        observation.subject().authority.revision,
        qualified.producer().bundle_revision()
    );
    assert_eq!(
        observation.subject().authority.digest,
        qualified.producer().digest()
    );
    assert_eq!(observation.signal_identity(), "signal:refund");
    assert_eq!(observation.trigger_identity(), "trigger:refund-request");
    assert_eq!(observation.unit(), "USD");
    assert_eq!(
        observation.value(),
        authority::observation::ValueRef::Present {
            value_type: "decimal",
            canonical_value: "12.00",
        }
    );
    assert_eq!(observation.visibility(), Visibility::External);
    assert_eq!(
        observation.anchor(),
        authority::observation::AnchorRef::Timestamp {
            instant_nanos: "10"
        }
    );
    assert_eq!(observation.event_time_nanos(), "10");
    assert_eq!(observation.ingestion_time_nanos(), "11");
    let relationship = observation
        .causal_relationship()
        .expect("the exact causal relationship is retained");
    assert_eq!(relationship.declaration_identity, ORDER_SHIPMENT);
    assert_eq!(relationship.interface_version, PRODUCER_INTERFACE_VERSION);
    assert_eq!(relationship.identity, b"relationship:O1-S1");
    assert_eq!(
        relationship.authority.identity,
        qualified.producer().bundle_identity()
    );
    assert_eq!(
        relationship.authority.revision,
        qualified.producer().bundle_revision()
    );
    assert_eq!(relationship.authority.digest, qualified.producer().digest());
    assert_eq!(relationship.source.kind, support::ORDER_KIND.as_bytes());
    assert_eq!(relationship.source.identity, b"order:O1");
    assert_eq!(relationship.target.kind, support::SHIPMENT_KIND.as_bytes());
    assert_eq!(relationship.target.identity, b"shipment:S1");
    assert_eq!(
        observation.clock(),
        authority::observation::ClockRef {
            identity: "clock:event-time",
            revision: "1",
            uncertainty_nanos: "2",
        }
    );
    assert_eq!(
        observation.lateness(),
        authority::observation::Lateness::OnTime
    );
    let expected_cutoff = authority::observation::cutoff_identity(&cutoff(40), Limits::owner_max())
        .expect("derive cutoff identity");
    assert_eq!(observation.cutoff_identity(), expected_cutoff.as_str());

    let population_view =
        authority::population::read(documents.population.bytes(), context, Limits::owner_max())
            .expect("strict-read population");
    let population = population_view.payload();
    assert_eq!(
        population.population_identity(),
        qualified.scope().population_identity.as_str()
    );
    let producer = population.producer();
    assert_eq!(producer.identity, qualified.producer().bundle_identity());
    assert_eq!(producer.version, PRODUCER_INTERFACE_VERSION);
    assert_eq!(producer.revision, qualified.producer().bundle_revision());
    assert_eq!(producer.digest, qualified.producer().digest());
    assert_eq!(
        population.membership_rule_identity(),
        qualified.scope().membership_rule_identity.as_str()
    );
    assert_eq!(
        population.membership_digest(),
        qualified
            .scope()
            .membership_rule_identity
            .as_str()
            .strip_prefix("sha256-jcs:")
            .expect("membership identity hash")
    );
    assert_eq!(
        population.selection(),
        authority::population::ScopeRef::EventTimeWindow {
            identity: "window:O1",
            start_nanos: "0",
            end_exclusive_nanos: "30",
        }
    );
    assert_eq!(population.required_members(), &["member:O1".to_owned()]);
    assert_eq!(
        population.observation_sources(),
        &["source:payments".to_owned()]
    );
    let configuration = population.configuration();
    assert_eq!(
        configuration.identity,
        qualified.producer().configuration().configuration_identity
    );
    assert_eq!(
        configuration.digest,
        &qualified.producer().configuration().digest
    );
    assert_eq!(
        population.closure(),
        authority::population::DefinitionRef {
            identity: "closure-definition:windows",
            digest: &"04".repeat(32),
        }
    );
    assert_eq!(
        population.completeness_dependencies(),
        &["completeness:O1".to_owned()]
    );
    assert_eq!(
        population.progress_dependencies(),
        &["progress:O1".to_owned()]
    );

    let position_view = authority::position::read(
        documents.position.bytes(),
        context,
        &authority::position::Selection::new(
            id("ledger:O1"),
            id("clock:event-time"),
            id("1"),
            vec![authority::position::Position::new(
                0,
                record.identity.clone(),
            )],
        ),
        Limits::owner_max(),
    )
    .expect("strict-read position ledger");
    let position = position_view.payload();
    assert_eq!(position.ledger_identity(), "ledger:O1");
    assert_eq!(position.clock_identity(), "clock:event-time");
    assert_eq!(position.clock_revision(), "1");
    assert_eq!(
        position.positions().collect::<Vec<_>>(),
        vec![("0", record.identity.as_str())]
    );

    let clock_view = authority::clock::read(
        documents.clock.bytes(),
        context,
        &authority::clock::Selection::new(id("clock:event-time"), id("1")),
        Limits::owner_max(),
    )
    .expect("strict-read clock binding");
    assert_eq!(clock_view.payload().clock_identity(), "clock:event-time");
    assert_eq!(clock_view.payload().clock_revision(), "1");
    assert_eq!(
        clock_view.payload().selection(),
        authority::clock::RangeRef::TimestampedEvent {
            start_nanos: "0",
            end_exclusive_nanos: "30",
        }
    );

    let capture_view = authority::capture::read(
        documents.capture.bytes(),
        context,
        &authority::capture::Selection::new(
            id("trigger:refund-request"),
            Anchor::TimestampNanos(10),
            vec![authority::capture::Binding::new(
                id("capture:refund-amount"),
                id("binding:refund"),
                record.identity.clone(),
            )],
        ),
        Limits::owner_max(),
    )
    .expect("strict-read capture environment");
    let capture = capture_view.payload();
    assert_eq!(capture.trigger_identity(), "trigger:refund-request");
    assert_eq!(
        capture.anchor(),
        authority::observation::AnchorRef::Timestamp {
            instant_nanos: "10"
        }
    );
    let bindings = capture.bindings().collect::<Vec<_>>();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].capture_identity, "capture:refund-amount");
    assert_eq!(bindings[0].binding_identity, "binding:refund");
    assert_eq!(bindings[0].observation_identity, record.identity.as_str());
    assert_eq!(bindings[0].value_type, "decimal");
    assert_eq!(bindings[0].canonical_value, "12.00");
    assert_eq!(bindings[0].unit, "USD");

    let progress_view = authority::progress::read(
        documents.progress.bytes(),
        context,
        &authority::progress::Selection::new(
            authority::clock::Selection::new(id("clock:event-time"), id("1")),
            vec![id("source:payments")],
            boundary(OpenClosed::Closed),
            OpenClosed::Closed,
            id("trigger:refund-request"),
            cutoff(40),
            id("restoration:O1"),
        ),
        Limits::owner_max(),
    )
    .expect("strict-read progress assertion");
    let progress = progress_view.payload();
    assert_eq!(progress.scope_identity(), "window:O1");
    assert_eq!(progress.clock_identity(), "clock:event-time");
    assert_eq!(progress.clock_revision(), "1");
    assert_eq!(progress.required_sources(), &["source:payments".to_owned()]);
    assert_eq!(
        progress.boundary(),
        authority::BoundaryRef::TimestampedEvent {
            lower_nanos: "0",
            upper_inclusive_nanos: "29",
            carrier_end_exclusive_nanos: "30",
            watermark_nanos: "30",
        }
    );
    assert_eq!(progress.state(), OpenClosed::Closed);
    assert_eq!(
        progress.captured_trigger_identity(),
        "trigger:refund-request"
    );
    assert_eq!(progress.cutoff_identity(), expected_cutoff.as_str());
    assert_eq!(progress.restoration_basis_identity(), "restoration:O1");
    assert!(progress.authority_identity().starts_with("sha256-jcs:"));

    let closure_view = authority::closure::read(
        documents.closure.bytes(),
        context,
        &authority::closure::Selection::new(
            id("clock:event-time"),
            id("1"),
            vec![id("source:payments")],
            boundary(OpenClosed::Closed),
            OpenClosed::Closed,
        ),
        Limits::owner_max(),
    )
    .expect("strict-read closure assertion");
    let closure = closure_view.payload();
    assert_eq!(closure.scope_identity(), "window:O1");
    assert_eq!(closure.clock_identity(), "clock:event-time");
    assert_eq!(closure.clock_revision(), "1");
    assert_eq!(closure.required_sources(), &["source:payments".to_owned()]);
    assert_eq!(closure.boundary(), progress.boundary());
    assert_eq!(closure.state(), OpenClosed::Closed);

    let completeness_view = authority::completeness::read(
        documents.completeness.bytes(),
        context,
        &authority::completeness::Selection::new(
            id("boundary:O1"),
            vec![authority::completeness::Fact::new(
                id("member:O1"),
                Some(record.identity.clone()),
                authority::completeness::FactStatus::Available,
            )],
        ),
        Limits::owner_max(),
    )
    .expect("strict-read completeness assertion");
    let completeness = completeness_view.payload();
    assert_eq!(
        completeness.population_identity(),
        qualified.scope().population_identity.as_str()
    );
    assert_eq!(completeness.boundary_identity(), "boundary:O1");
    assert_eq!(completeness.fact_count(), 1);
    let facts = completeness.facts().collect::<Vec<_>>();
    assert_eq!(facts[0].member_identity, "member:O1");
    assert_eq!(
        facts[0].observation_identity,
        Some(record.identity.as_str())
    );
    assert_eq!(
        facts[0].status,
        authority::completeness::FactStatus::Available
    );
    assert_eq!(
        completeness.state(),
        authority::completeness::State::Complete
    );

    let availability_view = authority::availability::read(
        documents.availability.bytes(),
        context,
        &authority::availability::Selection::new(
            vec![id("result:O1")],
            vec![id("result:O1")],
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        ),
        Limits::owner_max(),
    )
    .expect("strict-read result availability");
    let availability = availability_view.payload();
    assert_eq!(availability.required_results(), &["result:O1".to_owned()]);
    assert_eq!(availability.available_results(), &["result:O1".to_owned()]);
    assert_eq!(
        availability.state(),
        authority::availability::State::Available
    );

    let wrong = authority::observation::Selection::new(record.identity.clone(), cutoff(10));
    assert_eq!(
        authority::observation::read(
            documents.observation.bytes(),
            context,
            &wrong,
            Limits::owner_max()
        )
        .expect_err("cross-wired selection must refuse")
        .code(),
        authority::ErrorCode::ExpectedMismatch
    );
}

#[trace("TC-003", "FR-003-AC-2", "TC-004", "FR-004-AC-3")]
#[test]
fn tc004_every_required_field_is_missing_duplicate_and_order_strict() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let documents = derive_all(History::batch(&qualified), &owner, &subject);
    let artifacts = [
        (ArtifactKind::Activation, &documents.activation),
        (ArtifactKind::Observation, &documents.observation),
        (ArtifactKind::Population, &documents.population),
        (ArtifactKind::Position, &documents.position),
        (ArtifactKind::Clock, &documents.clock),
        (ArtifactKind::Partial, &documents.partial),
        (ArtifactKind::Capture, &documents.capture),
        (ArtifactKind::Progress, &documents.progress),
        (ArtifactKind::Closure, &documents.closure),
        (ArtifactKind::Completeness, &documents.completeness),
        (ArtifactKind::Availability, &documents.availability),
    ];
    let envelope_fields = [
        "contract",
        "identity",
        "revision",
        "authority",
        "subject",
        "payload",
        "predecessor",
        "limits",
    ];

    for (kind, document) in artifacts {
        for field in envelope_fields {
            assert!(
                !reader_accepts(
                    kind,
                    &remove_json_field(document.bytes(), false, field),
                    &qualified,
                    &owner,
                    &subject,
                ),
                "missing envelope field {field} was admitted for {}",
                document.contract()
            );
            assert!(
                !reader_accepts(
                    kind,
                    &duplicate_json_field(document.bytes(), false, field),
                    &qualified,
                    &owner,
                    &subject,
                ),
                "duplicate envelope field {field} was admitted for {}",
                document.contract()
            );
        }

        let value: serde_json::Value =
            serde_json::from_slice(document.bytes()).expect("fixture JSON");
        let payload_fields: Vec<_> = value["payload"]
            .as_object()
            .expect("payload object")
            .keys()
            .cloned()
            .collect();
        for field in payload_fields {
            assert!(
                !reader_accepts(
                    kind,
                    &remove_json_field(document.bytes(), true, &field),
                    &qualified,
                    &owner,
                    &subject,
                ),
                "missing payload field {field} was admitted for {}",
                document.contract()
            );
            assert!(
                !reader_accepts(
                    kind,
                    &duplicate_json_field(document.bytes(), true, &field),
                    &qualified,
                    &owner,
                    &subject,
                ),
                "duplicate payload field {field} was admitted for {}",
                document.contract()
            );
        }

        for payload in [false, true] {
            assert!(!reader_accepts(
                kind,
                &insert_unknown_json_field(document.bytes(), payload),
                &qualified,
                &owner,
                &subject,
            ));
        }

        let reordered = serde_json::to_vec(&value).expect("serialize reordered document");
        assert_ne!(reordered, document.bytes());
        assert!(!reader_accepts(
            kind, &reordered, &qualified, &owner, &subject,
        ));
    }

    let mut invalid_utf8 = documents.population.bytes().to_vec();
    let string_byte = invalid_utf8
        .iter()
        .position(|byte| *byte == b'q')
        .expect("document string byte");
    invalid_utf8[string_byte] = 0xff;
    assert!(!reader_accepts(
        ArtifactKind::Population,
        &invalid_utf8,
        &qualified,
        &owner,
        &subject,
    ));

    assert!(!reader_accepts(
        ArtifactKind::Population,
        documents.observation.bytes(),
        &qualified,
        &owner,
        &subject,
    ));
}

#[trace("TC-003", "FR-003-AC-1", "TC-004", "FR-004-AC-5")]
#[test]
fn tc004_independent_state_vocabularies_reject_cross_coercion() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let documents = derive_all(History::batch(&qualified), &owner, &subject);
    let mutations = [
        (
            ArtifactKind::Activation,
            replace_once(
                documents.activation.bytes(),
                b"\"activation\":\"active\"",
                b"\"activation\":\"complete\"",
            ),
        ),
        (
            ArtifactKind::Observation,
            replace_once(
                documents.observation.bytes(),
                b"\"lateness\":\"on-time\"",
                b"\"lateness\":\"true\"",
            ),
        ),
        (
            ArtifactKind::Progress,
            replace_once(
                documents.progress.bytes(),
                b"\"state\":\"closed\"",
                b"\"state\":\"complete\"",
            ),
        ),
        (
            ArtifactKind::Closure,
            replace_once(
                documents.closure.bytes(),
                b"\"state\":\"closed\"",
                b"\"state\":\"available\"",
            ),
        ),
        (
            ArtifactKind::Completeness,
            replace_once(
                documents.completeness.bytes(),
                b"\"state\":\"complete\"",
                b"\"state\":\"closed\"",
            ),
        ),
        (
            ArtifactKind::Availability,
            replace_once(
                documents.availability.bytes(),
                b"\"state\":\"available\"",
                b"\"state\":true",
            ),
        ),
    ];
    for (kind, bytes) in mutations {
        assert!(!reader_accepts(kind, &bytes, &qualified, &owner, &subject,));
    }
}

#[trace("TC-002", "FR-002-AC-1", "TC-004", "FR-004-AC-2", "IT-001")]
#[test]
// Trace: NFR-002-AC-1, NFR-001-AC-5
fn tc004_batch_and_incremental_history_are_byte_identical() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let batch = derive_all(History::batch(&qualified), &owner, &subject);
    let mut incremental = IncrementalHistory::new(&qualified);
    for record in qualified.records() {
        incremental.push(record).expect("admit incremental record");
    }
    let incremental = derive_all(
        incremental.finish().expect("complete incremental history"),
        &owner,
        &subject,
    );
    assert_eq!(batch.activation.bytes(), incremental.activation.bytes());
    assert_eq!(batch.observation.bytes(), incremental.observation.bytes());
    assert_eq!(batch.population.bytes(), incremental.population.bytes());
    assert_eq!(batch.position.bytes(), incremental.position.bytes());
    assert_eq!(batch.clock.bytes(), incremental.clock.bytes());
    assert_eq!(batch.capture.bytes(), incremental.capture.bytes());
    assert_eq!(batch.progress.bytes(), incremental.progress.bytes());
    assert_eq!(batch.closure.bytes(), incremental.closure.bytes());
    assert_eq!(batch.completeness.bytes(), incremental.completeness.bytes());
    assert_eq!(batch.availability.bytes(), incremental.availability.bytes());

    assert_eq!(
        IncrementalHistory::new(&qualified)
            .finish()
            .expect_err("incomplete history must refuse")
            .code(),
        authority::ErrorCode::ExpectedMismatch
    );
    let mut wrong_record = qualified.records()[0].clone();
    wrong_record.identity = id("record:out-of-order");
    let mut out_of_order = IncrementalHistory::new(&qualified);
    assert_eq!(
        out_of_order
            .push(&wrong_record)
            .expect_err("out-of-order record must refuse")
            .code(),
        authority::ErrorCode::ExpectedMismatch
    );
    let mut excess = IncrementalHistory::new(&qualified);
    excess
        .push(&qualified.records()[0])
        .expect("exact incremental record");
    assert_eq!(
        excess
            .push(&qualified.records()[0])
            .expect_err("excess incremental record must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );
}

#[trace("TC-003", "FR-003-AC-3")]
#[test]
// Trace: FR-003-AC-3
fn tc003_unsupported_owner_representation_refuses_without_partial_view() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let documents = derive_all(history, &owner, &subject);
    assert_eq!(
        authority::population::read(documents.observation.bytes(), context, Limits::owner_max())
            .expect_err("foreign owner contract must refuse")
            .code(),
        authority::ErrorCode::ContractMismatch
    );
}

#[trace("TC-004", "FR-004-AC-3", "FR-004-AC-4")]
#[test]
fn tc004_strict_reader_rejects_mutations_trailing_data_and_one_over_bytes() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let document = derive_all(history, &owner, &subject).population;

    let mut trailing = document.bytes().to_vec();
    trailing.push(b' ');
    assert!(authority::population::read(&trailing, context, Limits::owner_max()).is_err());

    let mut duplicate = b"{\"contract\":\"quire.observation.population/v1\",".to_vec();
    duplicate.extend_from_slice(&document.bytes()[1..]);
    assert!(authority::population::read(&duplicate, context, Limits::owner_max()).is_err());

    let mut unknown = document.bytes().to_vec();
    unknown.insert(unknown.len() - 1, b',');
    unknown.splice(
        unknown.len() - 1..unknown.len() - 1,
        b"\"unknown\":0".iter().copied(),
    );
    assert!(authority::population::read(&unknown, context, Limits::owner_max()).is_err());

    let mut one_under = Limits::owner_max();
    one_under.max_input_bytes = document.bytes().len() - 1;
    assert_eq!(
        authority::population::read(document.bytes(), context, one_under)
            .expect_err("one-over input must be resource-incomplete")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );
}

#[trace("TC-004", "FR-004-AC-4")]
#[test]
// Trace: NFR-001-AC-3
fn tc004_byte_depth_string_and_work_limits_are_exact_and_fail_closed() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let baseline = authority::population::derive(context, Limits::owner_max())
        .expect("derive baseline population");

    for (exact, lower) in [
        (
            Limits {
                max_depth: baseline.usage().depth,
                ..Limits::owner_max()
            },
            Limits {
                max_depth: baseline.usage().depth - 1,
                ..Limits::owner_max()
            },
        ),
        (
            Limits {
                max_string_bytes: baseline.usage().string_bytes,
                ..Limits::owner_max()
            },
            Limits {
                max_string_bytes: baseline.usage().string_bytes - 1,
                ..Limits::owner_max()
            },
        ),
        (
            Limits {
                max_visited_fields: baseline.usage().visited_fields,
                ..Limits::owner_max()
            },
            Limits {
                max_visited_fields: baseline.usage().visited_fields - 1,
                ..Limits::owner_max()
            },
        ),
    ] {
        let exact_result = authority::population::derive(context, exact);
        assert!(
            exact_result.is_ok(),
            "exact limit refused: {exact_result:?}"
        );
        assert_eq!(
            authority::population::derive(context, lower)
                .expect_err("one-over structural work must refuse")
                .code(),
            authority::ErrorCode::ResourceIncomplete
        );
    }

    let mut output_size = baseline.bytes().len();
    let output_exact = (0..8)
        .find_map(|_| {
            let limits = Limits {
                max_output_bytes: output_size,
                ..Limits::owner_max()
            };
            let document = authority::population::derive(context, limits)
                .expect("candidate exact output limit");
            if document.bytes().len() == output_size {
                Some((limits, document))
            } else {
                output_size = document.bytes().len();
                None
            }
        })
        .expect("output usage reaches a fixed point within eight encodings");
    assert_eq!(
        output_exact.1.bytes().len(),
        output_exact.0.max_output_bytes
    );
    let too_small_output = Limits {
        max_output_bytes: output_exact.0.max_output_bytes - 1,
        ..Limits::owner_max()
    };
    assert_eq!(
        authority::population::derive(context, too_small_output)
            .expect_err("one-over output must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );

    let mut input_size = baseline.bytes().len();
    let input_exact = (0..8)
        .find_map(|_| {
            let limits = Limits {
                max_input_bytes: input_size,
                ..Limits::owner_max()
            };
            let document = authority::population::derive(context, limits)
                .expect("candidate exact input limit");
            if document.bytes().len() == input_size {
                Some((limits, document))
            } else {
                input_size = document.bytes().len();
                None
            }
        })
        .expect("input usage reaches a fixed point within eight encodings");
    assert!(authority::population::read(input_exact.1.bytes(), context, input_exact.0).is_ok());
    let too_small_input = Limits {
        max_input_bytes: input_exact.0.max_input_bytes - 1,
        ..Limits::owner_max()
    };
    assert_eq!(
        authority::population::read(input_exact.1.bytes(), context, too_small_input)
            .expect_err("one-over input must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );
}

#[trace("TC-004", "FR-004-AC-4")]
#[test]
// Trace: NFR-001-AC-4
fn tc004_each_independent_semantic_limit_admits_exact_and_refuses_one_over() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let record = &qualified.records()[0];

    let mut exact = Limits::owner_max();
    exact.max_population_entries = 1;
    assert!(authority::population::derive(context, exact).is_ok());
    exact.max_population_entries = 0;
    assert_eq!(
        authority::population::derive(context, exact)
            .expect_err("one-over population must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );

    let positions = authority::position::Selection::new(
        id("ledger:O1"),
        id("clock:event-time"),
        id("1"),
        vec![authority::position::Position::new(
            0,
            record.identity.clone(),
        )],
    );
    let mut position_limits = Limits::owner_max();
    position_limits.max_positions = 1;
    assert!(authority::position::derive(context, &positions, position_limits).is_ok());
    position_limits.max_positions = 0;
    assert_eq!(
        authority::position::derive(context, &positions, position_limits)
            .expect_err("one-over positions must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );

    let captures = authority::capture::Selection::new(
        id("trigger:refund-request"),
        Anchor::TimestampNanos(10),
        vec![authority::capture::Binding::new(
            id("capture:refund-amount"),
            id("binding:refund"),
            record.identity.clone(),
        )],
    );
    let mut capture_limits = Limits::owner_max();
    capture_limits.max_capture_bindings = 1;
    assert!(authority::capture::derive(context, &captures, capture_limits).is_ok());
    capture_limits.max_capture_bindings = 0;
    assert_eq!(
        authority::capture::derive(context, &captures, capture_limits)
            .expect_err("one-over captures must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );

    let progress = authority::progress::Selection::new(
        authority::clock::Selection::new(id("clock:event-time"), id("1")),
        vec![id("source:payments")],
        boundary(OpenClosed::Closed),
        OpenClosed::Closed,
        id("trigger:refund-request"),
        cutoff(40),
        id("restoration:O1"),
    );
    let mut source_limits = Limits::owner_max();
    source_limits.max_required_sources = 1;
    assert!(authority::progress::derive(context, &progress, source_limits).is_ok());
    source_limits.max_required_sources = 0;
    assert_eq!(
        authority::progress::derive(context, &progress, source_limits)
            .expect_err("one-over sources must refuse")
            .code(),
        authority::ErrorCode::ResourceIncomplete
    );
}

#[trace("TC-002", "FR-002-AC-4", "IT-001")]
#[test]
fn tc002_duplicate_declared_position_is_ambiguous_not_unsupported() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let record = &qualified.records()[0];
    let ambiguous = authority::position::Selection::new(
        id("ledger:O1"),
        id("clock:event-time"),
        id("1"),
        vec![
            authority::position::Position::new(0, record.identity.clone()),
            authority::position::Position::new(0, record.identity.clone()),
        ],
    );
    assert_eq!(
        authority::position::derive(context, &ambiguous, Limits::owner_max())
            .expect_err("duplicate position must refuse")
            .code(),
        authority::ErrorCode::AmbiguousOrder
    );
}

#[trace("TC-004", "FR-004-AC-3")]
#[test]
fn tc004_position_ledger_refuses_a_foreign_clock_selection() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let record = &qualified.records()[0];
    let selection = authority::position::Selection::new(
        id("ledger:O1"),
        id("clock:foreign"),
        id("1"),
        vec![authority::position::Position::new(
            0,
            record.identity.clone(),
        )],
    );
    let error = authority::position::derive(
        Context::new(History::batch(&qualified), &owner, &subject, 1, None),
        &selection,
        Limits::owner_max(),
    )
    .expect_err("foreign position clock must refuse");
    assert_eq!(error.code(), authority::ErrorCode::ExpectedMismatch);
}

#[trace("TC-002", "FR-002-AC-2", "TC-004", "FR-004-AC-6", "IT-001")]
#[test]
fn tc004_boundary_mapping_is_exact_and_late_classification_is_owner_derived() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let record = &qualified.records()[0];

    let equal_watermark = TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: 29,
    };
    let invalid_progress = authority::progress::Selection::new(
        authority::clock::Selection::new(id("clock:event-time"), id("1")),
        vec![id("source:payments")],
        equal_watermark,
        OpenClosed::Closed,
        id("trigger:refund-request"),
        cutoff(40),
        id("restoration:O1"),
    );
    assert_eq!(
        authority::progress::derive(context, &invalid_progress, Limits::owner_max())
            .expect_err("watermark equality cannot cover an inclusive deadline")
            .code(),
        authority::ErrorCode::InvalidSelection
    );

    let foreign_carrier = TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 29,
        carrier_end_exclusive_nanos: 31,
        watermark_nanos: 31,
    };
    let cross_wired = authority::closure::Selection::new(
        id("clock:event-time"),
        id("1"),
        vec![id("source:payments")],
        foreign_carrier,
        OpenClosed::Closed,
    );
    assert_eq!(
        authority::closure::derive(context, &cross_wired, Limits::owner_max())
            .expect_err("foreign carrier cannot close this scope")
            .code(),
        authority::ErrorCode::ExpectedMismatch
    );

    let on_time_selection =
        authority::observation::Selection::new(record.identity.clone(), cutoff(40));
    let on_time = authority::observation::derive(context, &on_time_selection, Limits::owner_max())
        .expect("derive on-time record");
    let late_selection =
        authority::observation::Selection::new(record.identity.clone(), cutoff(10));
    let late = authority::observation::derive(
        Context::new(history, &owner, &subject, 2, Some(&on_time)),
        &late_selection,
        Limits::owner_max(),
    )
    .expect("derive late correction");
    let late_view = authority::observation::read(
        late.bytes(),
        Context::new(history, &owner, &subject, 2, Some(&on_time)),
        &late_selection,
        Limits::owner_max(),
    )
    .expect("strict-read late correction");
    assert_eq!(
        late_view.payload().lateness(),
        authority::observation::Lateness::Late
    );
    assert_eq!(
        late_view.payload().observation_identity(),
        record.identity.as_str()
    );
    assert_eq!(late.predecessor(), Some(on_time.identity()));
}

#[trace(
    "TC-002",
    "FR-002-AC-2",
    "FR-002-AC-3",
    "TC-004",
    "FR-004-AC-5",
    "IT-001"
)]
#[test]
fn tc004_state_axes_remain_independent_and_non_boolean() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let record = &qualified.records()[0];
    for progress_state in [OpenClosed::Open, OpenClosed::Closed] {
        for closure_state in [OpenClosed::Open, OpenClosed::Closed] {
            for fact_status in [
                authority::completeness::FactStatus::Available,
                authority::completeness::FactStatus::Incomplete,
                authority::completeness::FactStatus::Contradicted,
            ] {
                for (available_results, producer, contract, expected_availability) in [
                    (
                        vec![id("result:O1")],
                        authority::availability::DependencyState::Available,
                        authority::availability::DependencyState::Available,
                        authority::availability::State::Available,
                    ),
                    (
                        vec![],
                        authority::availability::DependencyState::Available,
                        authority::availability::DependencyState::Available,
                        authority::availability::State::NotYetObserved,
                    ),
                    (
                        vec![],
                        authority::availability::DependencyState::Unavailable,
                        authority::availability::DependencyState::Available,
                        authority::availability::State::ProducerUnavailable,
                    ),
                    (
                        vec![],
                        authority::availability::DependencyState::Available,
                        authority::availability::DependencyState::Unavailable,
                        authority::availability::State::ContractUnavailable,
                    ),
                ] {
                    let progress_selection = authority::progress::Selection::new(
                        authority::clock::Selection::new(id("clock:event-time"), id("1")),
                        vec![id("source:payments")],
                        boundary(progress_state),
                        progress_state,
                        id("trigger:refund-request"),
                        cutoff(40),
                        id("restoration:O1"),
                    );
                    let progress = authority::progress::derive(
                        context,
                        &progress_selection,
                        Limits::owner_max(),
                    )
                    .expect("independent progress state");
                    assert_eq!(
                        authority::progress::read(
                            progress.bytes(),
                            context,
                            &progress_selection,
                            Limits::owner_max(),
                        )
                        .expect("read independent progress state")
                        .payload()
                        .state(),
                        progress_state
                    );
                    let closure_selection = authority::closure::Selection::new(
                        id("clock:event-time"),
                        id("1"),
                        vec![id("source:payments")],
                        boundary(closure_state),
                        closure_state,
                    );
                    let closure = authority::closure::derive(
                        context,
                        &closure_selection,
                        Limits::owner_max(),
                    )
                    .expect("independent closure state");
                    assert_eq!(
                        authority::closure::read(
                            closure.bytes(),
                            context,
                            &closure_selection,
                            Limits::owner_max(),
                        )
                        .expect("read independent closure state")
                        .payload()
                        .state(),
                        closure_state
                    );
                    let observation =
                        if fact_status == authority::completeness::FactStatus::Incomplete {
                            None
                        } else {
                            Some(record.identity.clone())
                        };
                    let expected_completeness = match fact_status {
                        authority::completeness::FactStatus::Available => {
                            authority::completeness::State::Complete
                        }
                        authority::completeness::FactStatus::Incomplete => {
                            authority::completeness::State::Incomplete
                        }
                        authority::completeness::FactStatus::Contradicted => {
                            authority::completeness::State::Contradicted
                        }
                    };
                    let completeness_selection = authority::completeness::Selection::new(
                        id("boundary:O1"),
                        vec![authority::completeness::Fact::new(
                            id("member:O1"),
                            observation,
                            fact_status,
                        )],
                    );
                    let completeness = authority::completeness::derive(
                        context,
                        &completeness_selection,
                        Limits::owner_max(),
                    )
                    .expect("independent completeness state");
                    assert_eq!(
                        authority::completeness::read(
                            completeness.bytes(),
                            context,
                            &completeness_selection,
                            Limits::owner_max(),
                        )
                        .expect("read independent completeness state")
                        .payload()
                        .state(),
                        expected_completeness
                    );
                    let availability_selection = authority::availability::Selection::new(
                        vec![id("result:O1")],
                        available_results,
                        producer,
                        contract,
                    );
                    let availability = authority::availability::derive(
                        context,
                        &availability_selection,
                        Limits::owner_max(),
                    )
                    .expect("independent availability state");
                    assert_eq!(
                        authority::availability::read(
                            availability.bytes(),
                            context,
                            &availability_selection,
                            Limits::owner_max(),
                        )
                        .expect("read independent availability state")
                        .payload()
                        .state(),
                        expected_availability
                    );
                    for document in [&progress, &closure, &completeness, &availability] {
                        let text =
                            std::str::from_utf8(document.bytes()).expect("owner JSON is UTF-8");
                        assert!(!text.contains("truth"));
                        assert!(!text.contains("settlement"));
                        assert!(!text.contains("conformance"));
                        assert!(!text.contains("boolean"));
                    }
                }
            }
        }
    }
}

#[trace("TC-002", "FR-002-AC-3", "TC-004", "FR-004-AC-6")]
#[test]
fn tc004_corrections_are_new_direct_lineage_and_cross_wiring_refuses() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let original = authority::availability::derive(
        Context::new(history, &owner, &subject, 1, None),
        &authority::availability::Selection::new(
            vec![id("result:O1")],
            vec![],
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        ),
        Limits::owner_max(),
    )
    .expect("derive original");
    let correction = authority::availability::derive(
        Context::new(history, &owner, &subject, 2, Some(&original)),
        &authority::availability::Selection::new(
            vec![id("result:O1")],
            vec![id("result:O1")],
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        ),
        Limits::owner_max(),
    )
    .expect("derive correction");
    assert_ne!(original.identity(), correction.identity());
    assert_eq!(correction.predecessor(), Some(original.identity()));
    let original_bytes = original.bytes().to_vec();
    assert_eq!(original.bytes(), original_bytes);

    let foreign_subject = SubjectSelection {
        scope_identity: id("window:other"),
        population_identity: qualified.scope().population_identity.clone(),
    };
    assert_eq!(
        authority::availability::derive(
            Context::new(history, &owner, &foreign_subject, 2, Some(&original)),
            &authority::availability::Selection::new(
                vec![id("result:O1")],
                vec![id("result:O1")],
                authority::availability::DependencyState::Available,
                authority::availability::DependencyState::Available,
            ),
            Limits::owner_max()
        )
        .expect_err("cross-wired subject cannot correct an artifact")
        .code(),
        authority::ErrorCode::ExpectedMismatch
    );
}

#[trace("TC-005", "FR-005-AC-9")]
#[test]
fn tc005_qualified_owner_contracts_version_without_mutating_v1() {
    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    assert_eq!(
        sha256(include_bytes!(
            "../schemas/observation-record-v1.schema.json"
        )),
        "2922c6ad6bbca53f0800b7bd5159781632aa6ebd1e696567ae5639ec18dea3a3"
    );
    assert_eq!(
        sha256(include_bytes!(
            "../schemas/observation-population-v1.schema.json"
        )),
        "493b4c10701356007d055d351171ee1897de2b7226da03161fa86a9e52e524c2"
    );
    assert_eq!(
        authority::observation::CONTRACT,
        "quire.observation.record/v2"
    );
    assert_eq!(
        authority::population::CONTRACT,
        "quire.observation.population/v2"
    );
}

fn bundle_components(documents: &Documents) -> Vec<authority::bundle::Component> {
    use authority::bundle::{Component, ComponentRole};

    [
        (ComponentRole::Observation, &documents.observation),
        (ComponentRole::Population, &documents.population),
        (ComponentRole::Position, &documents.position),
        (ComponentRole::Clock, &documents.clock),
        (ComponentRole::Partial, &documents.partial),
        (ComponentRole::Capture, &documents.capture),
        (ComponentRole::Progress, &documents.progress),
        (ComponentRole::Activation, &documents.activation),
        (ComponentRole::Closure, &documents.closure),
        (ComponentRole::Completeness, &documents.completeness),
        (ComponentRole::Availability, &documents.availability),
    ]
    .into_iter()
    .map(|(role, document)| {
        Component::from_document(role, document).expect("owner document matches component role")
    })
    .collect()
}

#[trace("TC-009", "FR-009-AC-1", "FR-009-AC-5")]
#[test]
fn tc009_initial_bundle_is_canonical_complete_and_strictly_read() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let context = Context::new(History::batch(&qualified), &owner, &subject, 1, None);
    let documents = derive_all(History::batch(&qualified), &owner, &subject);
    let components = bundle_components(&documents);
    let selection = authority::bundle::Selection::new(components.clone(), vec![]);
    let first = authority::bundle::publish(context, &selection, None, Limits::owner_max())
        .expect("publish initial complete I07 bundle");
    let second = authority::bundle::publish(context, &selection, None, Limits::owner_max())
        .expect("repeat initial publication");
    assert_eq!(first.document().bytes(), second.document().bytes());
    assert_eq!(first.document().identity(), second.document().identity());
    assert_eq!(
        first.disposition(),
        authority::bundle::Disposition::Published
    );
    assert_eq!(first.view().payload().components().len(), 11);
    assert_eq!(first.view().payload().records().len(), 5);
    assert_eq!(first.view().payload().populations().len(), 2);
    assert_eq!(first.view().payload().positions().len(), 2);
    assert_eq!(first.view().payload().progress().len(), 2);
    assert_eq!(first.view().payload().conflicts().len(), 0);
    use authority::bundle::{ComponentRole, EmbeddedPayloadRef};
    for fact in first
        .view()
        .payload()
        .records()
        .chain(first.view().payload().populations())
        .chain(first.view().payload().positions())
        .chain(first.view().payload().progress())
    {
        let (expected, typed) = match fact.role() {
            ComponentRole::Observation => (
                &documents.observation,
                matches!(fact.payload(), EmbeddedPayloadRef::Observation(_)),
            ),
            ComponentRole::Population => (
                &documents.population,
                matches!(fact.payload(), EmbeddedPayloadRef::Population(_)),
            ),
            ComponentRole::Position => (
                &documents.position,
                matches!(fact.payload(), EmbeddedPayloadRef::Position(_)),
            ),
            ComponentRole::Clock => (
                &documents.clock,
                matches!(fact.payload(), EmbeddedPayloadRef::Clock(_)),
            ),
            ComponentRole::Partial => (
                &documents.partial,
                matches!(fact.payload(), EmbeddedPayloadRef::Partial(_)),
            ),
            ComponentRole::Capture => (
                &documents.capture,
                matches!(fact.payload(), EmbeddedPayloadRef::Capture(_)),
            ),
            ComponentRole::Progress => (
                &documents.progress,
                matches!(fact.payload(), EmbeddedPayloadRef::Progress(_)),
            ),
            ComponentRole::Activation => (
                &documents.activation,
                matches!(fact.payload(), EmbeddedPayloadRef::Activation(_)),
            ),
            ComponentRole::Closure => (
                &documents.closure,
                matches!(fact.payload(), EmbeddedPayloadRef::Closure(_)),
            ),
            ComponentRole::Completeness => (
                &documents.completeness,
                matches!(fact.payload(), EmbeddedPayloadRef::Completeness(_)),
            ),
            ComponentRole::Availability => (
                &documents.availability,
                matches!(fact.payload(), EmbeddedPayloadRef::Availability(_)),
            ),
        };
        assert!(typed, "embedded payload must retain its closed role type");
        assert_eq!(fact.canonical_document_bytes(), expected.bytes());
    }
    let replay = authority::bundle::publish(
        context,
        &selection,
        Some(first.lineage()),
        Limits::owner_max(),
    )
    .expect("initial current-head replay");
    assert_eq!(
        replay.disposition(),
        authority::bundle::Disposition::Replayed
    );

    let mut reversed = components;
    reversed.reverse();
    let permuted = authority::bundle::publish(
        context,
        &authority::bundle::Selection::new(reversed, vec![]),
        None,
        Limits::owner_max(),
    )
    .expect("component presentation order is non-semantic");
    assert_eq!(first.document().bytes(), permuted.document().bytes());
    for shift in 0..selection.components().len() {
        let mut generated = selection.components().to_vec();
        generated.rotate_left(shift);
        if shift % 2 == 1 {
            generated.reverse();
        }
        let candidate = authority::bundle::publish(
            context,
            &authority::bundle::Selection::new(generated, vec![]),
            None,
            Limits::owner_max(),
        )
        .expect("generated component permutation");
        assert_eq!(candidate.document().bytes(), first.document().bytes());
        assert_eq!(candidate.document().identity(), first.document().identity());
    }

    let alternate_record = authority::observation::derive(
        context,
        &authority::observation::Selection::new(
            qualified.records()[0].identity.clone(),
            cutoff(41),
        ),
        Limits::owner_max(),
    )
    .expect("second selected record document");
    let mut record_set = selection.components().to_vec();
    record_set.push(
        authority::bundle::Component::from_document(
            authority::bundle::ComponentRole::Observation,
            &alternate_record,
        )
        .expect("second record component"),
    );
    let duplicate_subject = authority::bundle::publish(
        context,
        &authority::bundle::Selection::new(record_set, vec![]),
        None,
        Limits::owner_max(),
    )
    .expect_err("same authority-qualified fact subject must not occur twice");
    assert_eq!(
        duplicate_subject.code(),
        authority::ErrorCode::InvalidSelection
    );

    use authority::bundle::{Conflict, ConflictKind};
    let conflicts = vec![
        Conflict::new(ConflictKind::Duplicate, &selection.components()[0]),
        Conflict::new(ConflictKind::Contradiction, &selection.components()[4]),
        Conflict::new(
            ConflictKind::UnresolvedCorrelation,
            &selection.components()[7],
        ),
    ];
    let conflict_selection = authority::bundle::Selection::with_conflicts(
        selection.components().to_vec(),
        vec![],
        conflicts.clone(),
    );
    let conflict_bundle =
        authority::bundle::publish(context, &conflict_selection, None, Limits::owner_max())
            .expect("bundle with every explicit conflict kind");
    let conflict_kinds = conflict_bundle
        .view()
        .payload()
        .conflicts()
        .map(|conflict| conflict.kind())
        .collect::<Vec<_>>();
    assert_eq!(
        conflict_kinds,
        vec![
            ConflictKind::Duplicate,
            ConflictKind::Contradiction,
            ConflictKind::UnresolvedCorrelation,
        ]
    );
    authority::bundle::read(
        conflict_bundle.document().bytes(),
        context,
        &conflict_selection,
        None,
        Limits::owner_max(),
    )
    .expect("strict-read explicit conflicts");
    let mut permuted_conflicts = conflicts;
    permuted_conflicts.reverse();
    let permuted_conflict_bundle = authority::bundle::publish(
        context,
        &authority::bundle::Selection::with_conflicts(
            selection.components().to_vec(),
            vec![],
            permuted_conflicts,
        ),
        None,
        Limits::owner_max(),
    )
    .expect("conflict presentation order is non-semantic");
    assert_eq!(
        conflict_bundle.document().bytes(),
        permuted_conflict_bundle.document().bytes()
    );
    authority::bundle::read(
        first.document().bytes(),
        context,
        &selection,
        None,
        Limits::owner_max(),
    )
    .expect("strict-read initial bundle");
    authority::bundle::read_lineage(
        first.lineage().document().bytes(),
        first.view(),
        &[],
        Limits::owner_max(),
    )
    .expect("strict-read canonical initial lineage");

    let alternate_availability = authority::availability::derive(
        context,
        &authority::availability::Selection::new(
            vec![id("result:O1")],
            vec![],
            authority::availability::DependencyState::Available,
            authority::availability::DependencyState::Available,
        ),
        Limits::owner_max(),
    )
    .expect("alternate valid availability");
    let mut alternate_components = selection.components().to_vec();
    let availability_index = alternate_components
        .iter()
        .position(|component| component.role() == authority::bundle::ComponentRole::Availability)
        .expect("availability role");
    alternate_components[availability_index] = authority::bundle::Component::from_document(
        authority::bundle::ComponentRole::Availability,
        &alternate_availability,
    )
    .expect("alternate availability component");
    let error = authority::bundle::read(
        first.document().bytes(),
        context,
        &authority::bundle::Selection::new(alternate_components, vec![]),
        None,
        Limits::owner_max(),
    )
    .expect_err("reader must revalidate the independently selected component set");
    assert_eq!(error.code(), authority::ErrorCode::ExpectedMismatch);
}

#[trace("TC-009", "FR-009-AC-1", "FR-009-AC-5")]
#[test]
fn tc009_bundle_schema_accepts_emitted_typed_wire_and_rejects_role_mismatches() {
    use serde_json::Value;

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let documents = derive_all(history, &owner, &subject);
    let selection = authority::bundle::Selection::new(bundle_components(&documents), vec![]);
    let publication = authority::bundle::publish(context, &selection, None, Limits::owner_max())
        .expect("canonical bundle for schema validation");

    let schema: Value =
        serde_json::from_slice(authority::bundle::SCHEMA_BYTES).expect("bundle schema is JSON");
    let validator = jsonschema::validator_for(&schema)
        .expect("standalone compound bundle schema and owner resources compile");
    let emitted: Value =
        serde_json::from_slice(publication.document().bytes()).expect("emitted bundle is JSON");
    assert!(
        validator.is_valid(&emitted),
        "emitted typed bundle must satisfy its advertised schema"
    );

    let observation_index = emitted["payload"]["records"]
        .as_array()
        .expect("record fact array")
        .iter()
        .position(|fact| fact["role"] == "observation")
        .expect("observation fact");
    let mut mismatched_kind = emitted.clone();
    mismatched_kind["payload"]["records"][observation_index]["payload"]["kind"] =
        Value::String("availability".to_owned());
    assert!(!validator.is_valid(&mismatched_kind));

    let mut empty_payload = emitted.clone();
    empty_payload["payload"]["records"][observation_index]["payload"]["value"] =
        Value::Object(serde_json::Map::new());
    assert!(!validator.is_valid(&empty_payload));

    let mut wrong_projection = emitted.clone();
    wrong_projection["payload"]["records"][observation_index] =
        emitted["payload"]["populations"][0].clone();
    assert!(!validator.is_valid(&wrong_projection));

    let mut invalid_subject = emitted;
    invalid_subject["payload"]["records"][observation_index]["fact_subject"]["semantic_key"] =
        Value::Null;
    assert!(!validator.is_valid(&invalid_subject));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[trace("TC-009", "FR-009-AC-1", "FR-009-AC-4")]
    #[test]
    fn tc009_generated_component_permutations_and_replays_are_idempotent(
        swaps in prop::collection::vec(0usize..11, 0..48)
    ) {
        let qualified = qualified();
        let owner = owner();
        let subject = subject(&qualified);
        let history = History::batch(&qualified);
        let context = Context::new(history, &owner, &subject, 1, None);
        let documents = derive_all(history, &owner, &subject);
        let components = bundle_components(&documents);
        let expected = authority::bundle::publish(
            context,
            &authority::bundle::Selection::new(components.clone(), vec![]),
            None,
            Limits::owner_max(),
        )
        .expect("canonical generated baseline");
        let mut generated = components;
        for (left, right) in swaps.into_iter().enumerate() {
            let len = generated.len();
            generated.swap(left % len, right % len);
        }
        let candidate = authority::bundle::publish(
            context,
            &authority::bundle::Selection::new(generated, vec![]),
            None,
            Limits::owner_max(),
        )
        .expect("generated permutation");
        prop_assert_eq!(candidate.document().bytes(), expected.document().bytes());
        prop_assert_eq!(candidate.document().identity(), expected.document().identity());
        let replay = authority::bundle::publish(
            context,
            &authority::bundle::Selection::new(bundle_components(&documents), vec![]),
            Some(expected.lineage()),
            Limits::owner_max(),
        )
        .expect("generated exact replay");
        prop_assert_eq!(replay.disposition(), authority::bundle::Disposition::Replayed);
    }
}

fn coordinator_interval(earliest: i128, latest: i128) -> authority::partial::EventTimeInterval {
    authority::partial::EventTimeInterval::new(
        id("clock:event-time"),
        id("1"),
        id("nanosecond"),
        earliest,
        latest,
        Limits::owner_max(),
    )
    .expect("valid coordinator interval")
}

fn coordinator_input(
    identity: &str,
    earliest: i128,
    latest: i128,
    bytes: &[u8],
) -> authority::coordination::Input {
    authority::coordination::Input::new(
        id(identity),
        id("window:O1"),
        coordinator_interval(earliest, latest),
        bytes.to_vec(),
    )
}

fn coordinator_plan() -> authority::repair::Plan {
    let (initial, successor) = repair_bundle_pair();
    authority::repair::plan(
        initial.view(),
        successor.view(),
        &repair_selection(&initial, &successor),
        authority::repair::Limits::owner_max(),
    )
    .expect("coordinator repair plan")
}

fn coordinator_closure_for(window_identity: &str, state: OpenClosed) -> authority::closure::View {
    let qualified = qualified_with_window(window_identity);
    let owner = owner();
    let subject = subject(&qualified);
    coordinator_closure_with(&qualified, &owner, &subject, state)
}

fn coordinator_closure_with(
    qualified: &QualifiedObservation,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
    state: OpenClosed,
) -> authority::closure::View {
    let history = History::batch(qualified);
    let context = Context::new(history, owner, subject, 1, None);
    let selection = authority::closure::Selection::new(
        id("clock:event-time"),
        id("1"),
        vec![id("source:payments")],
        boundary(state),
        state,
    );
    let document = authority::closure::derive(context, &selection, Limits::owner_max())
        .expect("derive coordinator closure");
    authority::closure::read(document.bytes(), context, &selection, Limits::owner_max())
        .expect("strict-read coordinator closure")
}

fn coordinator_batch_inputs() -> authority::coordination::BatchInputs {
    use authority::coordination::{BatchInputs, JobInputs};

    BatchInputs::new(vec![
        JobInputs::new(
            id("result:direct"),
            vec![
                coordinator_input("evidence:witness", 8, 12, b"witness"),
                coordinator_input("evidence:trailing", 5, 15, b"trailing"),
            ],
        ),
        JobInputs::new(
            id("result:composed"),
            vec![
                coordinator_input("evidence:unresolved", 0, 10, b"unresolved"),
                coordinator_input("evidence:tail", 5, 15, b"tail"),
            ],
        ),
        JobInputs::new(
            id("result:successor-only"),
            vec![coordinator_input(
                "evidence:unsupported",
                20,
                22,
                b"unsupported",
            )],
        ),
    ])
}

fn coordinator_profile() -> authority::coordination::PackageProfile {
    authority::coordination::PackageProfile::new(
        id("package:temporal"),
        id("7"),
        digest(7),
        id("profile:refund"),
        id("3"),
        digest(3),
    )
}

fn coordinator_selection_with_profile(
    batch: &authority::coordination::BatchInputs,
    closure: authority::closure::View,
    selected: authority::coordination::PackageProfile,
) -> authority::coordination::Selection {
    use authority::coordination::{InputManifest, Job, ResultInput, Selection};

    let manifest = |result_identity: &str| {
        let inputs = batch
            .jobs()
            .iter()
            .find(|job| job.result_identity().as_str() == result_identity)
            .expect("coordinator job inputs")
            .inputs();
        InputManifest::for_inputs(inputs).expect("exact coordinator input manifest")
    };
    Selection::new(vec![
        Job::new(
            id("result:direct"),
            selected.clone(),
            id("nanosecond"),
            manifest("result:direct"),
            vec![],
            None,
        ),
        Job::new(
            id("result:composed"),
            selected.clone(),
            id("nanosecond"),
            manifest("result:composed"),
            vec![ResultInput::new(
                id("result:direct"),
                id("window:O1"),
                coordinator_interval(5, 15),
            )],
            Some(closure),
        ),
        Job::new(
            id("result:successor-only"),
            selected,
            id("nanosecond"),
            manifest("result:successor-only"),
            vec![],
            None,
        ),
    ])
}

fn coordinator_selection(
    batch: &authority::coordination::BatchInputs,
) -> authority::coordination::Selection {
    coordinator_selection_with_profile(
        batch,
        coordinator_closure_for("window:O1", OpenClosed::Closed),
        coordinator_profile(),
    )
}

fn job_inputs<'a>(
    batch: &'a authority::coordination::BatchInputs,
    identity: &Identity,
) -> &'a [authority::coordination::Input] {
    batch
        .jobs()
        .iter()
        .find(|job| job.result_identity() == identity)
        .expect("scheduled coordinator inputs")
        .inputs()
}

fn drive_incremental<E: authority::coordination::Evaluator>(
    plan: &authority::repair::Plan,
    selection: &authority::coordination::Selection,
    batch: &authority::coordination::BatchInputs,
    evaluator: &mut E,
    limits: authority::coordination::Limits,
) -> authority::Result<authority::coordination::Run> {
    let mut run =
        authority::coordination::IncrementalRun::start(plan, selection, evaluator, limits)?;
    for identity in plan.recomputation_order() {
        for input in job_inputs(batch, identity) {
            run.push(identity, input.clone())?;
        }
        run.close(identity)?;
    }
    run.finish()
}

#[derive(Default)]
struct ScriptedEvaluator {
    composed_source_bytes: Vec<Vec<u8>>,
    lost_direct_order: bool,
}

impl authority::coordination::Evaluator for ScriptedEvaluator {
    fn evaluate(
        &mut self,
        request: authority::coordination::Request<'_>,
    ) -> authority::coordination::EvaluatorOutcome {
        use authority::coordination::{
            DecisionSupport, Disposition, EvaluatorOutcome, OrderRelation, SupportKind,
        };

        match request.result_identity().as_str() {
            "result:direct" => {
                if request.inputs().len() == 2 {
                    let relations = request
                        .orders()
                        .iter()
                        .map(|order| order.relation())
                        .collect::<Vec<_>>();
                    self.lost_direct_order |= relations
                        != [
                            OrderRelation::Before,
                            OrderRelation::Equal,
                            OrderRelation::After,
                        ];
                }
                if request
                    .inputs()
                    .iter()
                    .any(|input| input.identity().as_str() == "evidence:witness")
                {
                    EvaluatorOutcome::Decisive {
                        disposition: Disposition::Satisfied,
                        support: DecisionSupport::new(
                            SupportKind::Witness,
                            id("evidence:witness"),
                            vec![id("evidence:witness")],
                        ),
                        canonical_result: b"direct-v2".to_vec(),
                    }
                } else {
                    EvaluatorOutcome::Pending(id("reason:awaiting-witness"))
                }
            }
            "result:composed" => {
                let source = request
                    .inputs()
                    .iter()
                    .find(|input| input.identity().as_str() == "result:direct")
                    .expect("coordinator injected direct dependency");
                self.composed_source_bytes
                    .push(source.canonical_bytes().to_vec());
                if request.end_of_input() {
                    let closure = request.closure().expect("validated closure proof");
                    EvaluatorOutcome::Decisive {
                        disposition: Disposition::Violated,
                        support: DecisionSupport::new(
                            SupportKind::Closure,
                            closure.identity().clone(),
                            request
                                .inputs()
                                .iter()
                                .map(|input| input.identity().clone())
                                .collect(),
                        ),
                        canonical_result: b"composed-v2".to_vec(),
                    }
                } else {
                    EvaluatorOutcome::Pending(id("reason:awaiting-closure"))
                }
            }
            "result:successor-only" => {
                EvaluatorOutcome::Unsupported(id("reason:unsupported-profile"))
            }
            other => panic!("unexpected coordinator result: {other}"),
        }
    }
}

#[trace("TC-010", "FR-010-AC-3", "FR-010-AC-4")]
#[test]
fn tc010_streaming_prefix_is_observable_before_tail_and_matches_batch() {
    use authority::coordination::{Outcome, PrefixOutcome};

    let plan = coordinator_plan();
    let batch_inputs = coordinator_batch_inputs();
    let selection = coordinator_selection(&batch_inputs);
    let mut incremental_evaluator = ScriptedEvaluator::default();
    let mut stream = authority::coordination::IncrementalRun::start(
        &plan,
        &selection,
        &mut incremental_evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("start without future input");

    let direct_inputs = job_inputs(&batch_inputs, &id("result:direct"));
    let decisive = stream
        .push(&id("result:direct"), direct_inputs[0].clone())
        .expect("witness prefix is exposed immediately");
    assert!(!decisive.end_of_input());
    let PrefixOutcome::Settled(early) = decisive.outcome() else {
        panic!("witness prefix did not settle: {decisive:?}");
    };
    let early = early.clone();

    let trailing = stream
        .push(&id("result:direct"), direct_inputs[1].clone())
        .expect("unrelated tail arrives only after settlement was observed");
    let PrefixOutcome::Settled(after_tail) = trailing.outcome() else {
        panic!("settled prefix regressed after tail: {trailing:?}");
    };
    assert_eq!(after_tail, &early);
    stream
        .close(&id("result:direct"))
        .expect("close already-settled direct job");

    let composed_inputs = job_inputs(&batch_inputs, &id("result:composed"));
    let unresolved = stream
        .push(&id("result:composed"), composed_inputs[0].clone())
        .expect("unresolved composed prefix");
    assert!(matches!(unresolved.outcome(), PrefixOutcome::Pending(_)));
    assert!(!unresolved.end_of_input());
    stream
        .push(&id("result:composed"), composed_inputs[1].clone())
        .expect("composed tail remains unresolved before closure");
    let closed = stream
        .close(&id("result:composed"))
        .expect("matching validated closure settles the job");
    assert!(matches!(closed.outcome(), PrefixOutcome::Settled(_)));

    let successor_input = job_inputs(&batch_inputs, &id("result:successor-only"))[0].clone();
    stream
        .push(&id("result:successor-only"), successor_input)
        .expect("successor-only prefix");
    stream
        .close(&id("result:successor-only"))
        .expect("close successor-only job");
    let incremental = stream.finish().expect("complete incremental run");

    let mut batch_evaluator = ScriptedEvaluator::default();
    let batch = authority::coordination::run_batch(
        &plan,
        &selection,
        &batch_inputs,
        &mut batch_evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("complete batch run");
    assert_eq!(batch.bytes(), incremental.bytes());
    assert_eq!(batch.identity(), incremental.identity());
    assert_eq!(
        batch.outcomes().collect::<Vec<_>>(),
        incremental.outcomes().collect::<Vec<_>>()
    );
    assert!(incremental_evaluator
        .composed_source_bytes
        .iter()
        .all(|bytes| bytes == b"direct-v2"));
    assert!(batch_evaluator
        .composed_source_bytes
        .iter()
        .all(|bytes| bytes == b"direct-v2"));
    let batch_direct = batch.outcome(&id("result:direct")).expect("direct outcome");
    let Outcome::Replaced(batch_replacement) = batch_direct else {
        panic!("batch direct result was not replaced");
    };
    assert_eq!(batch_replacement.disposition(), early.disposition());
    assert_eq!(batch_replacement.support(), early.support());
    assert_eq!(batch_replacement.evidence_digest(), early.evidence_digest());
    assert_eq!(
        batch_replacement.evaluator_digest(),
        early.evaluator_digest()
    );
    assert_eq!(batch_replacement.evaluator_bytes(), early.evaluator_bytes());
    assert_eq!(batch_replacement.prior_identity().as_str(), "result:direct");
    assert_eq!(
        batch_replacement.support().evidence_identities(),
        [id("evidence:witness")]
    );
}

struct CounterexampleEvaluator;

impl authority::coordination::Evaluator for CounterexampleEvaluator {
    fn evaluate(
        &mut self,
        request: authority::coordination::Request<'_>,
    ) -> authority::coordination::EvaluatorOutcome {
        authority::coordination::EvaluatorOutcome::Decisive {
            disposition: authority::coordination::Disposition::Violated,
            support: authority::coordination::DecisionSupport::new(
                authority::coordination::SupportKind::Counterexample,
                request.inputs()[0].identity().clone(),
                vec![request.inputs()[0].identity().clone()],
            ),
            canonical_result: b"violated-v2".to_vec(),
        }
    }
}

#[trace("TC-010", "FR-010-AC-4")]
#[test]
fn tc010_counterexample_prefix_settles_before_end_of_input() {
    let plan = coordinator_plan();
    let batch = coordinator_batch_inputs();
    let selection = coordinator_selection(&batch);
    let mut evaluator = CounterexampleEvaluator;
    let mut stream = authority::coordination::IncrementalRun::start(
        &plan,
        &selection,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("start counterexample stream");
    let event = stream
        .push(
            &id("result:direct"),
            job_inputs(&batch, &id("result:direct"))[0].clone(),
        )
        .expect("counterexample prefix");
    assert!(!event.end_of_input());
    assert!(matches!(
        event.outcome(),
        authority::coordination::PrefixOutcome::Settled(settled)
            if settled.disposition() == authority::coordination::Disposition::Violated
                && settled.support().kind()
                    == authority::coordination::SupportKind::Counterexample
    ));
}

#[derive(Clone, Copy)]
enum FailureMode {
    Incomplete,
    Indeterminate,
    Unsupported,
    Refused,
    Failed,
    Exhausted,
}

struct IsolatedFailureEvaluator {
    mode: FailureMode,
}

impl authority::coordination::Evaluator for IsolatedFailureEvaluator {
    fn evaluate(
        &mut self,
        request: authority::coordination::Request<'_>,
    ) -> authority::coordination::EvaluatorOutcome {
        use authority::coordination::{
            DecisionSupport, Disposition, EvaluatorOutcome, SupportKind,
        };

        match request.result_identity().as_str() {
            "result:direct" => EvaluatorOutcome::Decisive {
                disposition: Disposition::Satisfied,
                support: DecisionSupport::new(
                    SupportKind::Witness,
                    id("evidence:witness"),
                    vec![id("evidence:witness")],
                ),
                canonical_result: b"direct-v2".to_vec(),
            },
            "result:composed" => match self.mode {
                FailureMode::Incomplete => EvaluatorOutcome::Incomplete(id("reason:incomplete")),
                FailureMode::Indeterminate => {
                    EvaluatorOutcome::Indeterminate(id("reason:indeterminate"))
                }
                FailureMode::Unsupported => EvaluatorOutcome::Unsupported(id("reason:unsupported")),
                FailureMode::Refused => EvaluatorOutcome::Refused(id("reason:refused")),
                FailureMode::Failed => EvaluatorOutcome::Failed(id("reason:failed")),
                FailureMode::Exhausted => EvaluatorOutcome::Exhausted(id("reason:exhausted")),
            },
            "result:successor-only" => EvaluatorOutcome::Pending(id("reason:pending")),
            other => panic!("unexpected result: {other}"),
        }
    }
}

#[trace("TC-010", "FR-010-AC-3", "FR-010-AC-6")]
#[test]
fn tc010_item_local_failure_preserves_successful_sibling_and_path_parity() {
    use authority::coordination::Outcome;

    for mode in [
        FailureMode::Incomplete,
        FailureMode::Indeterminate,
        FailureMode::Unsupported,
        FailureMode::Refused,
        FailureMode::Failed,
        FailureMode::Exhausted,
    ] {
        let plan = coordinator_plan();
        let expected_unaffected = plan
            .unaffected()
            .map(|result| (result.identity().clone(), result.bytes().to_vec()))
            .collect::<Vec<_>>();
        let inputs = coordinator_batch_inputs();
        let selection = coordinator_selection(&inputs);
        let mut batch_evaluator = IsolatedFailureEvaluator { mode };
        let batch = authority::coordination::run_batch(
            &plan,
            &selection,
            &inputs,
            &mut batch_evaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("batch item-local failure");
        let mut incremental_evaluator = IsolatedFailureEvaluator { mode };
        let incremental = drive_incremental(
            &plan,
            &selection,
            &inputs,
            &mut incremental_evaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("incremental item-local failure");
        assert_eq!(batch.bytes(), incremental.bytes());
        assert!(matches!(
            batch.outcome(&id("result:direct")),
            Some(Outcome::Replaced(_))
        ));
        assert!(!matches!(
            batch.outcome(&id("result:composed")),
            Some(Outcome::Replaced(_))
        ));
        assert!(matches!(
            (mode, batch.outcome(&id("result:composed"))),
            (FailureMode::Incomplete, Some(Outcome::Incomplete(_)))
                | (FailureMode::Indeterminate, Some(Outcome::Indeterminate(_)))
                | (FailureMode::Unsupported, Some(Outcome::Unsupported(_)))
                | (FailureMode::Refused, Some(Outcome::Refused(_)))
                | (FailureMode::Failed, Some(Outcome::Failed(_)))
                | (FailureMode::Exhausted, Some(Outcome::Exhausted(_)))
        ));
        assert!(matches!(
            batch.outcome(&id("result:successor-only")),
            Some(Outcome::Pending(_))
        ));
        let batch_unaffected = batch
            .unaffected()
            .map(|result| (result.identity().clone(), result.bytes().to_vec()))
            .collect::<Vec<_>>();
        let incremental_unaffected = incremental
            .unaffected()
            .map(|result| (result.identity().clone(), result.bytes().to_vec()))
            .collect::<Vec<_>>();
        assert_eq!(batch_unaffected, expected_unaffected);
        assert_eq!(incremental_unaffected, expected_unaffected);
    }
}

fn permuted_coordinator_inputs(
    reverse_jobs: bool,
    input_reversals: u8,
) -> authority::coordination::BatchInputs {
    let baseline = coordinator_batch_inputs();
    let mut jobs = baseline.jobs().to_vec();
    for (index, job) in jobs.iter_mut().enumerate() {
        if input_reversals & (1 << index) != 0 {
            let mut inputs = job.inputs().to_vec();
            inputs.reverse();
            *job = authority::coordination::JobInputs::new(job.result_identity().clone(), inputs);
        }
    }
    if reverse_jobs {
        jobs.reverse();
    }
    authority::coordination::BatchInputs::new(jobs)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[trace("TC-010", "FR-010-AC-3", "FR-010-AC-5")]
    #[test]
    fn tc010_generated_arrival_permutations_preserve_semantic_bytes(
        reverse_jobs in any::<bool>(),
        input_reversals in 0u8..8,
    ) {
        let plan = coordinator_plan();
        let baseline_inputs = coordinator_batch_inputs();
        let baseline_selection = coordinator_selection(&baseline_inputs);
        let mut baseline_evaluator = ScriptedEvaluator::default();
        let baseline = authority::coordination::run_batch(
            &plan,
            &baseline_selection,
            &baseline_inputs,
            &mut baseline_evaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("baseline batch run");

        let inputs = permuted_coordinator_inputs(reverse_jobs, input_reversals);
        let mut selection = coordinator_selection(&inputs);
        if reverse_jobs {
            let mut jobs = selection.jobs().to_vec();
            jobs.reverse();
            selection = authority::coordination::Selection::new(jobs);
        }
        let mut batch_evaluator = ScriptedEvaluator::default();
        let batch = authority::coordination::run_batch(
            &plan,
            &selection,
            &inputs,
            &mut batch_evaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("permuted batch run");
        let mut incremental_evaluator = ScriptedEvaluator::default();
        let incremental = drive_incremental(
            &plan,
            &selection,
            &inputs,
            &mut incremental_evaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("permuted incremental run");

        prop_assert!(!batch_evaluator.lost_direct_order);
        prop_assert!(!incremental_evaluator.lost_direct_order);
        prop_assert_eq!(batch.bytes(), baseline.bytes());
        prop_assert_eq!(incremental.bytes(), baseline.bytes());
        prop_assert_eq!(batch.identity(), incremental.identity());
    }
}

struct PendingEvaluator;

impl authority::coordination::Evaluator for PendingEvaluator {
    fn evaluate(
        &mut self,
        _request: authority::coordination::Request<'_>,
    ) -> authority::coordination::EvaluatorOutcome {
        authority::coordination::EvaluatorOutcome::Pending(id("reason:pending"))
    }
}

#[trace("TC-010", "FR-010-AC-3")]
#[test]
fn tc010_run_and_replacement_bind_selection_evidence_profile_and_lineage() {
    use authority::coordination::Outcome;

    let plan = coordinator_plan();
    let baseline_inputs = coordinator_batch_inputs();
    let baseline_selection = coordinator_selection(&baseline_inputs);
    let mut evaluator = ScriptedEvaluator::default();
    let baseline = authority::coordination::run_batch(
        &plan,
        &baseline_selection,
        &baseline_inputs,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("baseline bound run");
    let Outcome::Replaced(baseline_direct) = baseline
        .outcome(&id("result:direct"))
        .expect("baseline direct outcome")
    else {
        panic!("baseline direct outcome not replaced");
    };
    let replacement_json = std::str::from_utf8(baseline_direct.bytes()).expect("replacement JSON");
    assert!(replacement_json.contains(plan.identity().as_str()));
    assert!(replacement_json.contains("result:direct"));

    let profile_variants = [
        authority::coordination::PackageProfile::new(
            id("package:other"),
            id("7"),
            digest(7),
            id("profile:refund"),
            id("3"),
            digest(3),
        ),
        authority::coordination::PackageProfile::new(
            id("package:temporal"),
            id("8"),
            digest(7),
            id("profile:refund"),
            id("3"),
            digest(3),
        ),
        authority::coordination::PackageProfile::new(
            id("package:temporal"),
            id("7"),
            digest(8),
            id("profile:refund"),
            id("3"),
            digest(3),
        ),
        authority::coordination::PackageProfile::new(
            id("package:temporal"),
            id("7"),
            digest(7),
            id("profile:other"),
            id("3"),
            digest(3),
        ),
        authority::coordination::PackageProfile::new(
            id("package:temporal"),
            id("7"),
            digest(7),
            id("profile:refund"),
            id("4"),
            digest(3),
        ),
        authority::coordination::PackageProfile::new(
            id("package:temporal"),
            id("7"),
            digest(7),
            id("profile:refund"),
            id("3"),
            digest(4),
        ),
    ];
    for profile in profile_variants {
        let selection = coordinator_selection_with_profile(
            &baseline_inputs,
            coordinator_closure_for("window:O1", OpenClosed::Closed),
            profile,
        );
        let mut evaluator = ScriptedEvaluator::default();
        let changed = authority::coordination::run_batch(
            &plan,
            &selection,
            &baseline_inputs,
            &mut evaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("profile-bound run");
        assert_ne!(baseline.identity(), changed.identity());
        let Outcome::Replaced(changed_direct) = changed
            .outcome(&id("result:direct"))
            .expect("changed direct outcome")
        else {
            panic!("changed direct outcome not replaced");
        };
        assert_ne!(baseline_direct.identity(), changed_direct.identity());
    }

    let mut unrelated_jobs = baseline_inputs.jobs().to_vec();
    let direct = unrelated_jobs
        .iter_mut()
        .find(|job| job.result_identity().as_str() == "result:direct")
        .expect("direct inputs");
    let mut unrelated_inputs = direct.inputs().to_vec();
    unrelated_inputs[1] = coordinator_input("evidence:trailing", 5, 15, b"changed-tail");
    *direct = authority::coordination::JobInputs::new(id("result:direct"), unrelated_inputs);
    let unrelated_batch = authority::coordination::BatchInputs::new(unrelated_jobs);
    let unrelated_selection = coordinator_selection(&unrelated_batch);
    let mut evaluator = ScriptedEvaluator::default();
    let unrelated = authority::coordination::run_batch(
        &plan,
        &unrelated_selection,
        &unrelated_batch,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("unrelated-tail run");
    let Outcome::Replaced(unrelated_direct) = unrelated
        .outcome(&id("result:direct"))
        .expect("unrelated direct outcome")
    else {
        panic!("unrelated direct outcome not replaced");
    };
    assert_eq!(baseline_direct.identity(), unrelated_direct.identity());
    assert_ne!(baseline.identity(), unrelated.identity());

    let mut evidence_jobs = baseline_inputs.jobs().to_vec();
    let direct = evidence_jobs
        .iter_mut()
        .find(|job| job.result_identity().as_str() == "result:direct")
        .expect("direct inputs");
    let mut evidence_inputs = direct.inputs().to_vec();
    evidence_inputs[0] = coordinator_input("evidence:witness", 7, 12, b"changed-witness");
    *direct = authority::coordination::JobInputs::new(id("result:direct"), evidence_inputs);
    let evidence_batch = authority::coordination::BatchInputs::new(evidence_jobs);
    let evidence_selection = coordinator_selection(&evidence_batch);
    let mut evaluator = ScriptedEvaluator::default();
    let changed_evidence = authority::coordination::run_batch(
        &plan,
        &evidence_selection,
        &evidence_batch,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("changed certified evidence run");
    let Outcome::Replaced(changed_direct) = changed_evidence
        .outcome(&id("result:direct"))
        .expect("changed evidence direct outcome")
    else {
        panic!("changed evidence direct outcome not replaced");
    };
    assert_ne!(baseline_direct.identity(), changed_direct.identity());
    assert_ne!(
        baseline_direct.evidence_digest(),
        changed_direct.evidence_digest()
    );

    let mut pending_evaluator = PendingEvaluator;
    let pending_baseline = authority::coordination::run_batch(
        &plan,
        &baseline_selection,
        &baseline_inputs,
        &mut pending_evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("pending baseline selection");
    for axis in 0..3 {
        let mut jobs = baseline_inputs.jobs().to_vec();
        let direct = jobs
            .iter_mut()
            .find(|job| job.result_identity().as_str() == "result:direct")
            .expect("direct inputs");
        let mut inputs = direct.inputs().to_vec();
        inputs[0] = match axis {
            0 => coordinator_input("evidence:renamed", 8, 12, b"witness"),
            1 => coordinator_input("evidence:witness", 7, 12, b"witness"),
            _ => coordinator_input("evidence:witness", 8, 12, b"changed-witness"),
        };
        *direct = authority::coordination::JobInputs::new(id("result:direct"), inputs);
        let changed_inputs = authority::coordination::BatchInputs::new(jobs);
        let changed_selection = coordinator_selection(&changed_inputs);
        let changed = authority::coordination::run_batch(
            &plan,
            &changed_selection,
            &changed_inputs,
            &mut PendingEvaluator,
            authority::coordination::Limits::owner_max(),
        )
        .expect("independently changed input axis");
        assert_ne!(pending_baseline.identity(), changed.identity());
    }

    let (initial, successor) = repair_bundle_pair();
    let original_selection = repair_selection(&initial, &successor);
    let mut changed_prior_results = original_selection.prior_results().to_vec();
    let direct = changed_prior_results
        .iter_mut()
        .find(|result| result.identity().as_str() == "result:direct")
        .expect("direct prior result");
    *direct = authority::repair::PriorResult::new(
        direct.identity().clone(),
        direct.region().clone(),
        b"direct-v1-lineage-change".to_vec(),
    );
    let changed_plan_selection = authority::repair::Selection::new(
        original_selection.authority_revision().clone(),
        original_selection.fact_regions().to_vec(),
        changed_prior_results,
        original_selection.edges().to_vec(),
    );
    let changed_plan = authority::repair::plan(
        initial.view(),
        successor.view(),
        &changed_plan_selection,
        authority::repair::Limits::owner_max(),
    )
    .expect("plan with independently changed prior lineage bytes");
    let changed_selection = coordinator_selection(&baseline_inputs);
    let mut evaluator = ScriptedEvaluator::default();
    let changed_lineage = authority::coordination::run_batch(
        &changed_plan,
        &changed_selection,
        &baseline_inputs,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("changed lineage run");
    let Outcome::Replaced(changed_direct) = changed_lineage
        .outcome(&id("result:direct"))
        .expect("changed lineage direct outcome")
    else {
        panic!("changed lineage direct outcome not replaced");
    };
    assert_ne!(plan.identity(), changed_plan.identity());
    assert_ne!(baseline_direct.identity(), changed_direct.identity());
}

struct ForgedClosureEvaluator;

impl authority::coordination::Evaluator for ForgedClosureEvaluator {
    fn evaluate(
        &mut self,
        request: authority::coordination::Request<'_>,
    ) -> authority::coordination::EvaluatorOutcome {
        authority::coordination::EvaluatorOutcome::Decisive {
            disposition: authority::coordination::Disposition::Violated,
            support: authority::coordination::DecisionSupport::new(
                authority::coordination::SupportKind::Closure,
                id("closure:fake"),
                vec![request.inputs()[0].identity().clone()],
            ),
            canonical_result: b"forged".to_vec(),
        }
    }
}

#[trace("TC-010", "FR-010-AC-4", "FR-010-AC-6")]
#[test]
fn tc010_foreign_input_open_closure_and_forged_support_refuse() {
    let plan = coordinator_plan();
    let batch = coordinator_batch_inputs();

    let accepted_closure = coordinator_closure_for("window:O1", OpenClosed::Closed);
    assert_eq!(accepted_closure.identity(), plan.closure_identity());

    let qualified = qualified();
    let exact_subject = subject(&qualified);
    let foreign_owners = [
        AuthoritySelection {
            definition_identity: id("definition:foreign-closure-owner"),
            ..owner()
        },
        AuthoritySelection {
            definition_revision: id("2"),
            ..owner()
        },
        AuthoritySelection {
            definition_digest: digest(8),
            ..owner()
        },
    ];
    let mut foreign_closures = foreign_owners
        .iter()
        .map(|foreign_owner| {
            coordinator_closure_with(
                &qualified,
                foreign_owner,
                &exact_subject,
                OpenClosed::Closed,
            )
        })
        .collect::<Vec<_>>();
    let foreign_population =
        qualified_with_window_and_closure("window:O1", "closure-definition:foreign");
    let foreign_population_subject = subject(&foreign_population);
    assert_ne!(
        foreign_population_subject.population_identity,
        exact_subject.population_identity
    );
    foreign_closures.push(coordinator_closure_with(
        &foreign_population,
        &owner(),
        &foreign_population_subject,
        OpenClosed::Closed,
    ));
    for closure in foreign_closures {
        assert_ne!(closure.identity(), plan.closure_identity());
        let selection = coordinator_selection_with_profile(&batch, closure, coordinator_profile());
        let error = match authority::coordination::IncrementalRun::start(
            &plan,
            &selection,
            &mut PendingEvaluator,
            authority::coordination::Limits::owner_max(),
        ) {
            Ok(_) => panic!("foreign exact closure identity cannot authorize settlement"),
            Err(error) => error,
        };
        assert_eq!(error.code(), authority::ErrorCode::InvalidSelection);
    }

    let open_selection = coordinator_selection_with_profile(
        &batch,
        coordinator_closure_for("window:O1", OpenClosed::Open),
        coordinator_profile(),
    );
    let open_error = match authority::coordination::IncrementalRun::start(
        &plan,
        &open_selection,
        &mut PendingEvaluator,
        authority::coordination::Limits::owner_max(),
    ) {
        Ok(_) => panic!("open closure cannot authorize settlement"),
        Err(error) => error,
    };
    assert_eq!(open_error.code(), authority::ErrorCode::InvalidSelection);

    let foreign_closure_selection = coordinator_selection_with_profile(
        &batch,
        coordinator_closure_for("window:foreign", OpenClosed::Closed),
        coordinator_profile(),
    );
    let foreign_closure_error = match authority::coordination::IncrementalRun::start(
        &plan,
        &foreign_closure_selection,
        &mut PendingEvaluator,
        authority::coordination::Limits::owner_max(),
    ) {
        Ok(_) => panic!("foreign closure scope cannot authorize settlement"),
        Err(error) => error,
    };
    assert_eq!(
        foreign_closure_error.code(),
        authority::ErrorCode::InvalidSelection
    );

    let selection = coordinator_selection(&batch);
    let witness_state_bytes = job_inputs(&batch, &id("result:direct"))[0]
        .retained_state_bytes()
        .expect("baseline witness state bytes");
    let mut pending_evaluator = PendingEvaluator;
    let mut stream = authority::coordination::IncrementalRun::start(
        &plan,
        &selection,
        &mut pending_evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("valid stream");
    let foreign = authority::coordination::Input::new(
        id("evidence:witness"),
        id("window:X2"),
        coordinator_interval(8, 12),
        b"witness".to_vec(),
    );
    assert_eq!(
        foreign.retained_state_bytes().expect("foreign-scope bytes"),
        witness_state_bytes
    );
    assert_eq!(
        stream
            .push(&id("result:direct"), foreign)
            .expect_err("foreign scope cannot reach evaluator")
            .code(),
        authority::ErrorCode::InvalidSelection
    );
    let foreign_clock = authority::coordination::Input::new(
        id("evidence:witness"),
        id("window:O1"),
        authority::partial::EventTimeInterval::new(
            id("clock:other-time"),
            id("1"),
            id("nanosecond"),
            8,
            12,
            Limits::owner_max(),
        )
        .expect("valid foreign-clock interval"),
        b"witness".to_vec(),
    );
    assert_eq!(
        foreign_clock
            .retained_state_bytes()
            .expect("foreign-clock bytes"),
        witness_state_bytes
    );
    assert_eq!(
        stream
            .push(&id("result:direct"), foreign_clock)
            .expect_err("foreign clock cannot reach evaluator")
            .code(),
        authority::ErrorCode::InvalidSelection
    );
    for (revision, unit) in [("2", "nanosecond"), ("1", "time-unitx")] {
        let cross_wired = authority::coordination::Input::new(
            id("evidence:witness"),
            id("window:O1"),
            authority::partial::EventTimeInterval::new(
                id("clock:event-time"),
                id(revision),
                id(unit),
                8,
                12,
                Limits::owner_max(),
            )
            .expect("valid cross-wired interval"),
            b"witness".to_vec(),
        );
        assert_eq!(
            cross_wired
                .retained_state_bytes()
                .expect("cross-wired input bytes"),
            witness_state_bytes
        );
        assert_eq!(
            stream
                .push(&id("result:direct"), cross_wired)
                .expect_err("foreign clock revision or unit cannot reach evaluator")
                .code(),
            authority::ErrorCode::InvalidSelection
        );
    }

    let mut forged_evaluator = ForgedClosureEvaluator;
    let mut forged = authority::coordination::IncrementalRun::start(
        &plan,
        &selection,
        &mut forged_evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("valid selection with forged evaluator response");
    assert_eq!(
        forged
            .push(
                &id("result:direct"),
                job_inputs(&batch, &id("result:direct"))[0].clone(),
            )
            .expect_err("unbound closure support cannot promote")
            .code(),
        authority::ErrorCode::SupportMismatch
    );
}

struct DependencyUnavailableEvaluator {
    composed_calls: usize,
}

impl authority::coordination::Evaluator for DependencyUnavailableEvaluator {
    fn evaluate(
        &mut self,
        request: authority::coordination::Request<'_>,
    ) -> authority::coordination::EvaluatorOutcome {
        if request.result_identity().as_str() == "result:composed" {
            self.composed_calls += 1;
        }
        authority::coordination::EvaluatorOutcome::Pending(id("reason:pending"))
    }
}

#[trace("TC-010", "FR-010-AC-3", "FR-010-AC-6")]
#[test]
fn tc010_unreplaced_dependency_propagates_incomplete_without_stale_evaluation() {
    let plan = coordinator_plan();
    let batch = coordinator_batch_inputs();
    let selection = coordinator_selection(&batch);
    let mut evaluator = DependencyUnavailableEvaluator { composed_calls: 0 };
    let run = drive_incremental(
        &plan,
        &selection,
        &batch,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("dependency-unavailable run remains typed");
    assert_eq!(evaluator.composed_calls, 0);
    assert!(matches!(
        run.outcome(&id("result:composed")),
        Some(authority::coordination::Outcome::Incomplete(reason))
            if reason.as_str() == "dependency-unavailable:result:direct"
    ));
}

#[trace("TC-010", "FR-010-AC-6")]
#[test]
fn tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths() {
    let plan = coordinator_plan();
    let batch = coordinator_batch_inputs();
    let selection = coordinator_selection(&batch);
    let mut evaluator = ScriptedEvaluator::default();
    let baseline = authority::coordination::run_batch(
        &plan,
        &selection,
        &batch,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("batch usage baseline");
    let usage = baseline.usage();
    let declared_inputs = selection
        .jobs()
        .iter()
        .map(|job| job.external_inputs().count() + job.result_inputs().len())
        .sum::<usize>();
    assert_eq!(usage.jobs, selection.jobs().len());
    assert_eq!(usage.inputs, declared_inputs);
    assert_eq!(usage.output_bytes, baseline.bytes().len());
    assert_eq!(
        usage,
        authority::coordination::Usage {
            jobs: 3,
            inputs: 6,
            possible_orders: 12,
            evaluator_calls: 3,
            work: 137,
            state_bytes: 7_116,
            output_bytes: 2_252,
        }
    );
    let exact = authority::coordination::Limits {
        max_jobs: usage.jobs,
        max_inputs: usage.inputs,
        max_possible_orders: usage.possible_orders,
        max_work: usage.work,
        max_state_bytes: usage.state_bytes,
        max_output_bytes: usage.output_bytes,
    };
    assert!(authority::coordination::run_batch(
        &plan,
        &selection,
        &batch,
        &mut ScriptedEvaluator::default(),
        exact,
    )
    .is_ok());
    for lower in [
        authority::coordination::Limits {
            max_jobs: usage.jobs - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_inputs: usage.inputs - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_possible_orders: usage.possible_orders - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_work: usage.work - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_state_bytes: usage.state_bytes - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_output_bytes: usage.output_bytes - 1,
            ..exact
        },
    ] {
        assert_eq!(
            authority::coordination::run_batch(
                &plan,
                &selection,
                &batch,
                &mut ScriptedEvaluator::default(),
                lower,
            )
            .expect_err("one-over batch limit")
            .code(),
            authority::ErrorCode::ResourceIncomplete
        );
    }

    let mut evaluator = ScriptedEvaluator::default();
    let incremental = drive_incremental(
        &plan,
        &selection,
        &batch,
        &mut evaluator,
        authority::coordination::Limits::owner_max(),
    )
    .expect("incremental usage baseline");
    let usage = incremental.usage();
    assert_eq!(usage.jobs, selection.jobs().len());
    assert_eq!(usage.inputs, declared_inputs);
    assert_eq!(usage.output_bytes, incremental.bytes().len());
    assert_eq!(
        usage,
        authority::coordination::Usage {
            jobs: 3,
            inputs: 6,
            possible_orders: 21,
            evaluator_calls: 5,
            work: 144,
            state_bytes: 7_964,
            output_bytes: 2_252,
        }
    );
    let exact = authority::coordination::Limits {
        max_jobs: usage.jobs,
        max_inputs: usage.inputs,
        max_possible_orders: usage.possible_orders,
        max_work: usage.work,
        max_state_bytes: usage.state_bytes,
        max_output_bytes: usage.output_bytes,
    };
    assert!(drive_incremental(
        &plan,
        &selection,
        &batch,
        &mut ScriptedEvaluator::default(),
        exact,
    )
    .is_ok());
    for lower in [
        authority::coordination::Limits {
            max_jobs: usage.jobs - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_inputs: usage.inputs - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_possible_orders: usage.possible_orders - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_work: usage.work - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_state_bytes: usage.state_bytes - 1,
            ..exact
        },
        authority::coordination::Limits {
            max_output_bytes: usage.output_bytes - 1,
            ..exact
        },
    ] {
        assert_eq!(
            drive_incremental(
                &plan,
                &selection,
                &batch,
                &mut ScriptedEvaluator::default(),
                lower,
            )
            .expect_err("one-over incremental limit")
            .code(),
            authority::ErrorCode::ResourceIncomplete
        );
    }
}

#[trace("TC-010", "FR-010-AC-4", "FR-010-AC-6")]
#[test]
fn tc010_late_incremental_refusal_exposes_only_nonpromoted_settlement() {
    use authority::coordination::PrefixOutcome;

    let plan = coordinator_plan();
    let batch = coordinator_batch_inputs();
    let selection = coordinator_selection(&batch);
    let mut evaluator = ScriptedEvaluator::default();
    let mut stream = authority::coordination::IncrementalRun::start(
        &plan,
        &selection,
        &mut evaluator,
        authority::coordination::Limits {
            max_possible_orders: 0,
            ..authority::coordination::Limits::owner_max()
        },
    )
    .expect("zero-order stream starts from preflighted manifests");
    let direct_inputs = job_inputs(&batch, &id("result:direct"));
    let early = stream
        .push(&id("result:direct"), direct_inputs[0].clone())
        .expect("one-input witness has no pairwise-order work");
    assert!(matches!(early.outcome(), PrefixOutcome::Settled(_)));
    stream
        .push(&id("result:direct"), direct_inputs[1].clone())
        .expect("terminal direct certificate remains nonpromoted");
    stream
        .close(&id("result:direct"))
        .expect("close direct certificate");
    let error = stream
        .push(
            &id("result:composed"),
            job_inputs(&batch, &id("result:composed"))[0].clone(),
        )
        .expect_err("later pairwise-order work exceeds zero bound");
    assert_eq!(error.code(), authority::ErrorCode::ResourceIncomplete);
    // No `Run` exists and PrefixOutcome has no promoted-replacement variant.
}

#[trace("TC-009", "FR-009-AC-1", "FR-009-AC-5")]
#[test]
fn tc009_bundle_and_lineage_strict_readers_reject_wire_mutations() {
    use serde_json::Value;

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let context = Context::new(history, &owner, &subject, 1, None);
    let documents = derive_all(history, &owner, &subject);
    let selection = authority::bundle::Selection::new(bundle_components(&documents), vec![]);
    let publication = authority::bundle::publish(context, &selection, None, Limits::owner_max())
        .expect("initial publication");

    let mut contract: Value =
        serde_json::from_slice(publication.document().bytes()).expect("canonical bundle JSON");
    contract["contract"] = Value::String("quire.observation-authority/v0".to_owned());
    let error = authority::bundle::read(
        &serde_json::to_vec(&contract).expect("encode contract mutation"),
        context,
        &selection,
        None,
        Limits::owner_max(),
    )
    .expect_err("foreign contract");
    assert_eq!(error.code(), authority::ErrorCode::ContractMismatch);

    for index in 0..selection.components().len() {
        let digest = publication
            .view()
            .payload()
            .components()
            .nth(index)
            .expect("component at generated index")
            .digest();
        let canonical = std::str::from_utf8(publication.document().bytes())
            .expect("canonical owner bytes are UTF-8");
        let mutated = canonical.replacen(digest, &"0".repeat(64), 1);
        let error = authority::bundle::read(
            mutated.as_bytes(),
            context,
            &selection,
            None,
            Limits::owner_max(),
        )
        .expect_err("every component mutation must refuse");
        assert_eq!(error.code(), authority::ErrorCode::IdentityMismatch);
    }

    let mut unknown: Value =
        serde_json::from_slice(publication.document().bytes()).expect("canonical bundle JSON");
    unknown["unexpected"] = Value::Bool(true);
    let error = authority::bundle::read(
        &serde_json::to_vec(&unknown).expect("encode unknown-field mutation"),
        context,
        &selection,
        None,
        Limits::owner_max(),
    )
    .expect_err("unknown bundle field");
    assert_eq!(error.code(), authority::ErrorCode::InvalidJson);

    let pretty = serde_json::to_string_pretty(
        &serde_json::from_slice::<Value>(publication.document().bytes())
            .expect("canonical bundle JSON"),
    )
    .expect("encode pretty bundle mutation");
    let error = authority::bundle::read(
        pretty.as_bytes(),
        context,
        &selection,
        None,
        Limits::owner_max(),
    )
    .expect_err("noncanonical bundle encoding");
    assert_eq!(error.code(), authority::ErrorCode::NonCanonical);

    let lineage_value: Value = serde_json::from_slice(publication.lineage().document().bytes())
        .expect("canonical lineage JSON");
    let head_digest = lineage_value["head"]["digest"]
        .as_str()
        .expect("lineage head digest");
    let lineage_text = std::str::from_utf8(publication.lineage().document().bytes())
        .expect("canonical lineage bytes are UTF-8");
    let lineage = lineage_text.replacen(head_digest, &"0".repeat(64), 1);
    let error = authority::bundle::read_lineage(
        lineage.as_bytes(),
        publication.view(),
        &[],
        Limits::owner_max(),
    )
    .expect_err("lineage stamp mutation");
    assert_eq!(error.code(), authority::ErrorCode::IdentityMismatch);
}

fn successor_bundle_selection(
    history: History<'_>,
    owner: &AuthoritySelection,
    subject: &SubjectSelection,
    documents: &Documents,
    revision: u64,
    available_results: Vec<Identity>,
    producer: authority::availability::DependencyState,
) -> authority::bundle::Selection {
    use authority::bundle::{Component, ComponentRole, Replacement, Selection};

    let context = Context::new(
        history,
        owner,
        subject,
        revision,
        Some(&documents.availability),
    );
    let revised = authority::availability::derive(
        context,
        &authority::availability::Selection::new(
            vec![id("result:O1")],
            available_results,
            producer,
            authority::availability::DependencyState::Available,
        ),
        Limits::owner_max(),
    )
    .expect("derive revised availability");
    let mut components = bundle_components(documents);
    let index = components
        .iter()
        .position(|component| component.role() == ComponentRole::Availability)
        .expect("availability component");
    let prior = components[index].clone();
    let successor = Component::from_document(ComponentRole::Availability, &revised)
        .expect("revised availability component");
    let replacement = Replacement::new(&prior, &successor).expect("same-role replacement");
    components[index] = successor;
    Selection::new(components, vec![replacement])
}

fn repair_bundle_pair() -> (
    authority::bundle::Publication,
    authority::bundle::Publication,
) {
    use authority::availability::DependencyState;
    use authority::bundle::Selection;

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let documents = derive_all(history, &owner, &subject);
    let initial = authority::bundle::publish(
        Context::new(history, &owner, &subject, 1, None),
        &Selection::new(bundle_components(&documents), vec![]),
        None,
        Limits::owner_max(),
    )
    .expect("publish initial repair bundle");
    let successor_selection = successor_bundle_selection(
        history,
        &owner,
        &subject,
        &documents,
        2,
        vec![],
        DependencyState::Available,
    );
    let successor = authority::bundle::publish(
        Context::new(history, &owner, &subject, 2, Some(initial.document())),
        &successor_selection,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect("publish successor repair bundle");
    (initial, successor)
}

fn repair_region(start_nanos: i128, end_nanos: i128) -> authority::repair::Region {
    authority::repair::Region::new(
        id("window:O1"),
        id("clock:event-time"),
        id("1"),
        ClockRange::Timestamp {
            start_nanos,
            end_nanos,
        },
    )
    .expect("valid repair region")
}

fn repair_selection(
    initial: &authority::bundle::Publication,
    successor: &authority::bundle::Publication,
) -> authority::repair::Selection {
    use authority::repair::{DependencyEdge, FactRegion, PriorResult, Selection};

    let replacement = successor
        .view()
        .payload()
        .replacements()
        .next()
        .expect("availability replacement");
    Selection::new(
        initial.view().identity().clone(),
        vec![
            FactRegion::new(
                Identity::new(replacement.prior_identity()),
                repair_region(8, 12),
            ),
            FactRegion::new(
                Identity::new(replacement.successor_identity()),
                repair_region(20, 24),
            ),
        ],
        vec![
            PriorResult::new(
                id("result:direct"),
                repair_region(0, 20),
                b"direct-v1".to_vec(),
            ),
            PriorResult::new(
                id("result:composed"),
                repair_region(5, 25),
                b"composed-v1".to_vec(),
            ),
            PriorResult::new(
                id("result:outside"),
                repair_region(24, 30),
                b"outside-v1".to_vec(),
            ),
            PriorResult::new(
                id("result:isolated"),
                repair_region(0, 20),
                b"isolated-v1".to_vec(),
            ),
            PriorResult::new(
                id("result:successor-only"),
                repair_region(20, 24),
                b"successor-only-v1".to_vec(),
            ),
        ],
        vec![
            DependencyEdge::new(
                Identity::new(replacement.prior_identity()),
                id("result:direct"),
            ),
            DependencyEdge::new(id("result:direct"), id("result:composed")),
            DependencyEdge::new(
                Identity::new(replacement.prior_identity()),
                id("result:outside"),
            ),
            DependencyEdge::new(
                Identity::new(replacement.prior_identity()),
                id("result:successor-only"),
            ),
        ],
    )
}

#[trace("TC-010", "FR-010-AC-1", "FR-010-AC-2")]
#[test]
fn tc010_plan_contains_only_explicit_observable_closure_and_retains_other_bytes() {
    let (initial, successor) = repair_bundle_pair();
    let selection = repair_selection(&initial, &successor);
    let first = authority::repair::plan(
        initial.view(),
        successor.view(),
        &selection,
        authority::repair::Limits::owner_max(),
    )
    .expect("derive repair plan");
    let second = authority::repair::plan(
        initial.view(),
        successor.view(),
        &selection,
        authority::repair::Limits::owner_max(),
    )
    .expect("repeat repair plan");

    assert_eq!(first.bytes(), second.bytes());
    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        first.affected().map(Identity::as_str).collect::<Vec<_>>(),
        ["result:composed", "result:direct", "result:successor-only"]
    );
    assert_eq!(
        first
            .recomputation_order()
            .map(Identity::as_str)
            .collect::<Vec<_>>(),
        ["result:direct", "result:composed", "result:successor-only"]
    );
    let unaffected = first
        .unaffected()
        .map(|result| (result.identity().as_str(), result.bytes()))
        .collect::<Vec<_>>();
    assert_eq!(
        unaffected,
        [
            ("result:isolated", b"isolated-v1".as_slice()),
            ("result:outside", b"outside-v1".as_slice()),
        ]
    );
    let replacement = successor
        .view()
        .payload()
        .replacements()
        .next()
        .expect("one replacement");
    let changed = first
        .changed_facts()
        .next()
        .expect("changed fact inventory");
    assert_eq!(first.changed_facts().len(), 1);
    assert_eq!(
        changed.role(),
        authority::bundle::ComponentRole::Availability
    );
    assert_eq!(
        changed.prior_identity().as_str(),
        replacement.prior_identity()
    );
    assert_eq!(
        changed.successor_identity().as_str(),
        replacement.successor_identity()
    );
    assert_eq!(
        changed.prior_region().range(),
        ClockRange::Timestamp {
            start_nanos: 8,
            end_nanos: 12,
        }
    );
    assert_eq!(
        changed.successor_region().range(),
        ClockRange::Timestamp {
            start_nanos: 20,
            end_nanos: 24,
        }
    );
    let shifted_facts = selection
        .fact_regions()
        .iter()
        .map(|fact| {
            if fact.identity().as_str() == replacement.successor_identity() {
                authority::repair::FactRegion::new(fact.identity().clone(), repair_region(20, 23))
            } else {
                fact.clone()
            }
        })
        .collect();
    let shifted = authority::repair::Selection::new(
        selection.authority_revision().clone(),
        shifted_facts,
        selection.prior_results().to_vec(),
        selection.edges().to_vec(),
    );
    let shifted_plan = authority::repair::plan(
        initial.view(),
        successor.view(),
        &shifted,
        authority::repair::Limits::owner_max(),
    )
    .expect("same affected set with a distinct successor region");
    assert_eq!(
        first.affected().collect::<Vec<_>>(),
        shifted_plan.affected().collect::<Vec<_>>()
    );
    assert_ne!(first.bytes(), shifted_plan.bytes());
    assert_ne!(first.identity(), shifted_plan.identity());
}

#[trace("TC-010", "FR-010-AC-1")]
#[test]
fn tc010_half_open_overlap_is_exact_for_every_clock_family_and_authority_axis() {
    use authority::repair::Region;

    let event = |scope: &str, clock: &str, revision: &str, start, end_exclusive| {
        Region::new(
            id(scope),
            id(clock),
            id(revision),
            ClockRange::EventPosition {
                start,
                end_exclusive,
            },
        )
        .expect("valid event-position region")
    };
    let fixed = |epoch_nanos, period_nanos, start, end_exclusive| {
        Region::new(
            id("window:O1"),
            id("clock:fixed"),
            id("1"),
            ClockRange::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            },
        )
        .expect("valid fixed-sample region")
    };
    let timestamp = |start_nanos, end_nanos| repair_region(start_nanos, end_nanos);

    assert!(
        event("window:O1", "clock:event", "1", 0, 10).overlaps(&event(
            "window:O1",
            "clock:event",
            "1",
            9,
            11
        ))
    );
    assert!(
        !event("window:O1", "clock:event", "1", 0, 10).overlaps(&event(
            "window:O1",
            "clock:event",
            "1",
            10,
            20
        ))
    );
    assert!(fixed(100, 5, 0, 10).overlaps(&fixed(100, 5, 9, 11)));
    assert!(!fixed(100, 5, 0, 10).overlaps(&fixed(100, 5, 10, 20)));
    assert!(!fixed(100, 5, 0, 10).overlaps(&fixed(101, 5, 0, 10)));
    assert!(timestamp(0, 10).overlaps(&timestamp(9, 11)));
    assert!(!timestamp(0, 10).overlaps(&timestamp(10, 20)));
    assert!(
        !event("window:O1", "clock:event", "1", 0, 10).overlaps(&event(
            "window:other",
            "clock:event",
            "1",
            0,
            10
        ))
    );
    assert!(
        !event("window:O1", "clock:event", "1", 0, 10).overlaps(&event(
            "window:O1",
            "clock:other",
            "1",
            0,
            10
        ))
    );
    assert!(
        !event("window:O1", "clock:event", "1", 0, 10).overlaps(&event(
            "window:O1",
            "clock:event",
            "2",
            0,
            10
        ))
    );
}

#[trace("TC-010", "FR-010-AC-1")]
#[test]
fn tc010_independent_results_schedule_by_scope_window_then_identity() {
    use authority::repair::{DependencyEdge, FactRegion, PriorResult, Selection};

    let (initial, successor) = repair_bundle_pair();
    let replacement = successor
        .view()
        .payload()
        .replacements()
        .next()
        .expect("replacement seed");
    let results = [
        ("result:b", repair_region(5, 15)),
        ("result:z", repair_region(0, 20)),
        ("result:a", repair_region(5, 15)),
    ];
    let selection = Selection::new(
        initial.view().identity().clone(),
        vec![
            FactRegion::new(
                Identity::new(replacement.prior_identity()),
                repair_region(8, 12),
            ),
            FactRegion::new(
                Identity::new(replacement.successor_identity()),
                repair_region(8, 12),
            ),
        ],
        results
            .iter()
            .map(|(identity, region)| {
                PriorResult::new(id(identity), region.clone(), identity.as_bytes().to_vec())
            })
            .collect(),
        results
            .iter()
            .map(|(identity, _)| {
                DependencyEdge::new(Identity::new(replacement.prior_identity()), id(identity))
            })
            .collect(),
    );
    let plan = authority::repair::plan(
        initial.view(),
        successor.view(),
        &selection,
        authority::repair::Limits::owner_max(),
    )
    .expect("independent result schedule");
    assert_eq!(
        plan.recomputation_order()
            .map(Identity::as_str)
            .collect::<Vec<_>>(),
        ["result:z", "result:a", "result:b"]
    );
}

#[trace("TC-010", "FR-010-AC-6")]
#[test]
fn tc010_cycle_unknown_identity_and_foreign_revision_refuse_without_plan() {
    use authority::repair::{DependencyEdge, Selection};

    let (initial, successor) = repair_bundle_pair();
    let baseline = repair_selection(&initial, &successor);
    let cycle = Selection::new(
        baseline.authority_revision().clone(),
        baseline.fact_regions().to_vec(),
        baseline.prior_results().to_vec(),
        vec![
            DependencyEdge::new(id("result:direct"), id("result:composed")),
            DependencyEdge::new(id("result:composed"), id("result:direct")),
        ],
    );
    assert_eq!(
        authority::repair::plan(
            initial.view(),
            successor.view(),
            &cycle,
            authority::repair::Limits::owner_max(),
        )
        .expect_err("cycle must refuse without a plan")
        .code(),
        authority::ErrorCode::DependencyCycle
    );

    let unknown = Selection::new(
        baseline.authority_revision().clone(),
        baseline.fact_regions().to_vec(),
        baseline.prior_results().to_vec(),
        vec![DependencyEdge::new(
            id("result:missing"),
            id("result:direct"),
        )],
    );
    assert_eq!(
        authority::repair::plan(
            initial.view(),
            successor.view(),
            &unknown,
            authority::repair::Limits::owner_max(),
        )
        .expect_err("unknown dependency identity must refuse")
        .code(),
        authority::ErrorCode::InvalidDependency
    );

    let foreign = Selection::new(
        id("bundle:foreign-revision"),
        baseline.fact_regions().to_vec(),
        baseline.prior_results().to_vec(),
        baseline.edges().to_vec(),
    );
    assert_eq!(
        authority::repair::plan(
            initial.view(),
            successor.view(),
            &foreign,
            authority::repair::Limits::owner_max(),
        )
        .expect_err("foreign graph revision must refuse")
        .code(),
        authority::ErrorCode::ForeignRevision
    );
}

#[trace("TC-010", "FR-010-AC-1", "FR-010-AC-6")]
#[test]
fn tc010_only_replacements_seed_invalidation_and_clock_domains_cannot_cross_wire() {
    use authority::repair::{DependencyEdge, FactRegion, PriorResult, Selection};

    let (initial, successor) = repair_bundle_pair();
    let baseline = repair_selection(&initial, &successor);
    let non_replaced = initial
        .view()
        .payload()
        .components()
        .find(|component| component.role() == authority::bundle::ComponentRole::Observation)
        .expect("non-replaced observation component");
    let mut facts = baseline.fact_regions().to_vec();
    facts.push(FactRegion::new(
        Identity::new(non_replaced.identity()),
        repair_region(8, 12),
    ));
    let non_seeded = Selection::new(
        baseline.authority_revision().clone(),
        facts.clone(),
        baseline.prior_results().to_vec(),
        vec![DependencyEdge::new(
            Identity::new(non_replaced.identity()),
            id("result:isolated"),
        )],
    );
    let plan = authority::repair::plan(
        initial.view(),
        successor.view(),
        &non_seeded,
        authority::repair::Limits::owner_max(),
    )
    .expect("non-replaced facts cannot seed invalidation");
    assert_eq!(plan.affected().len(), 0);
    assert_eq!(plan.unaffected().len(), baseline.prior_results().len());

    let contradictory_results = baseline
        .prior_results()
        .iter()
        .enumerate()
        .map(|(index, result)| {
            if index == 0 {
                PriorResult::new(
                    result.identity().clone(),
                    authority::repair::Region::new(
                        id("window:O1"),
                        id("clock:event-time"),
                        id("1"),
                        ClockRange::EventPosition {
                            start: 0,
                            end_exclusive: 20,
                        },
                    )
                    .expect("individually valid cross-wired domain"),
                    result.bytes().to_vec(),
                )
            } else {
                result.clone()
            }
        })
        .collect();
    let contradictory = Selection::new(
        baseline.authority_revision().clone(),
        baseline.fact_regions().to_vec(),
        contradictory_results,
        baseline.edges().to_vec(),
    );
    assert_eq!(
        authority::repair::plan(
            initial.view(),
            successor.view(),
            &contradictory,
            authority::repair::Limits::owner_max(),
        )
        .expect_err("one clock revision cannot mix clock families")
        .code(),
        authority::ErrorCode::IntervalDomainMismatch
    );

    assert_eq!(
        authority::repair::Region::new(
            id("window:O1"),
            id("clock:event-time"),
            id("1"),
            ClockRange::Timestamp {
                start_nanos: 10,
                end_nanos: 10,
            },
        )
        .expect_err("empty half-open repair window")
        .code(),
        authority::ErrorCode::InvalidInterval
    );

    for foreign_region in [
        authority::repair::Region::new(
            id("window:foreign"),
            id("clock:event-time"),
            id("1"),
            ClockRange::Timestamp {
                start_nanos: 8,
                end_nanos: 12,
            },
        )
        .expect("valid foreign scope region"),
        authority::repair::Region::new(
            id("window:O1"),
            id("clock:foreign"),
            id("99"),
            ClockRange::Timestamp {
                start_nanos: 8,
                end_nanos: 12,
            },
        )
        .expect("valid foreign clock region"),
    ] {
        let cross_wired_facts = baseline
            .fact_regions()
            .iter()
            .map(|fact| FactRegion::new(fact.identity().clone(), foreign_region.clone()))
            .collect();
        let cross_wired = Selection::new(
            baseline.authority_revision().clone(),
            cross_wired_facts,
            baseline.prior_results().to_vec(),
            baseline.edges().to_vec(),
        );
        assert_eq!(
            authority::repair::plan(
                initial.view(),
                successor.view(),
                &cross_wired,
                authority::repair::Limits::owner_max(),
            )
            .expect_err("uniformly cross-wired fact regions must refuse")
            .code(),
            authority::ErrorCode::AuthorityMismatch
        );
    }
    let foreign_results = baseline
        .prior_results()
        .iter()
        .map(|result| {
            PriorResult::new(
                result.identity().clone(),
                authority::repair::Region::new(
                    id("window:foreign"),
                    id("clock:event-time"),
                    id("1"),
                    result.region().range(),
                )
                .expect("valid but foreign result scope"),
                result.bytes().to_vec(),
            )
        })
        .collect();
    let cross_wired_results = Selection::new(
        baseline.authority_revision().clone(),
        baseline.fact_regions().to_vec(),
        foreign_results,
        baseline.edges().to_vec(),
    );
    assert_eq!(
        authority::repair::plan(
            initial.view(),
            successor.view(),
            &cross_wired_results,
            authority::repair::Limits::owner_max(),
        )
        .expect_err("prior results require prior bundle authority")
        .code(),
        authority::ErrorCode::AuthorityMismatch
    );
}

#[trace("TC-010", "FR-010-AC-6")]
#[test]
fn tc010_each_planner_limit_admits_exact_and_refuses_one_over() {
    let (initial, successor) = repair_bundle_pair();
    let selection = repair_selection(&initial, &successor);
    let baseline = authority::repair::plan(
        initial.view(),
        successor.view(),
        &selection,
        authority::repair::Limits::owner_max(),
    )
    .expect("baseline repair plan");
    let usage = baseline.usage();
    let region_identity_bytes = |region: &authority::repair::Region| {
        region.scope_identity().as_str().len()
            + region.clock_identity().as_str().len()
            + region.clock_revision().as_str().len()
    };
    let expected_state_bytes = selection
        .fact_regions()
        .iter()
        .map(|fact| fact.identity().as_str().len() + region_identity_bytes(fact.region()))
        .sum::<usize>()
        + selection
            .prior_results()
            .iter()
            .map(|result| {
                result.bytes().len()
                    + result.identity().as_str().len()
                    + region_identity_bytes(result.region())
            })
            .sum::<usize>()
        + baseline
            .dependencies()
            .map(|edge| {
                edge.source_identity().as_str().len() + edge.dependent_identity().as_str().len()
            })
            .sum::<usize>()
        + baseline.closure_identity().as_str().len();
    assert_eq!(
        usage.nodes,
        selection.fact_regions().len() + selection.prior_results().len()
    );
    assert_eq!(usage.edges, selection.edges().len());
    assert_eq!(usage.retained_results, selection.prior_results().len());
    assert_eq!(usage.retained_bytes, expected_state_bytes);
    let exact = authority::repair::Limits {
        max_nodes: usage.nodes,
        max_edges: usage.edges,
        max_work: usage.work,
        max_retained_results: usage.retained_results,
        max_retained_bytes: usage.retained_bytes,
        max_output_bytes: baseline.bytes().len(),
    };
    assert!(authority::repair::plan(initial.view(), successor.view(), &selection, exact).is_ok());
    let count_preflight = authority::repair::plan(
        initial.view(),
        successor.view(),
        &selection,
        authority::repair::Limits {
            max_nodes: usage.nodes - 1,
            ..exact
        },
    )
    .expect_err("count preflight must refuse before state traversal");
    assert_eq!(
        count_preflight.code(),
        authority::ErrorCode::ResourceIncomplete
    );
    assert_eq!(count_preflight.usage().visited_fields, 0);

    for lower in [
        authority::repair::Limits {
            max_nodes: usage.nodes - 1,
            ..exact
        },
        authority::repair::Limits {
            max_edges: usage.edges - 1,
            ..exact
        },
        authority::repair::Limits {
            max_work: usage.work - 1,
            ..exact
        },
        authority::repair::Limits {
            max_retained_results: usage.retained_results - 1,
            ..exact
        },
        authority::repair::Limits {
            max_retained_bytes: usage.retained_bytes - 1,
            ..exact
        },
        authority::repair::Limits {
            max_output_bytes: baseline.bytes().len() - 1,
            ..exact
        },
    ] {
        assert_eq!(
            authority::repair::plan(initial.view(), successor.view(), &selection, lower)
                .expect_err("one-over planner resource must refuse")
                .code(),
            authority::ErrorCode::ResourceIncomplete
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[trace("TC-010", "FR-010-AC-1", "FR-010-AC-2")]
    #[test]
    fn tc010_generated_dags_match_reference_closure_and_permutation(
        edge_flags in any::<[bool; 15]>(),
        observable in any::<[bool; 5]>(),
    ) {
        use authority::repair::{DependencyEdge, FactRegion, PriorResult, Selection};

        let (initial, successor) = repair_bundle_pair();
        let replacement = successor
            .view()
            .payload()
            .replacements()
            .next()
            .expect("replacement seed");
        let result_ids = [
            "result:generated:0",
            "result:generated:1",
            "result:generated:2",
            "result:generated:3",
            "result:generated:4",
        ];
        let mut edges = Vec::new();
        for (index, result_identity) in result_ids.iter().enumerate() {
            if edge_flags[index] {
                edges.push(DependencyEdge::new(
                    Identity::new(replacement.prior_identity()),
                    id(result_identity),
                ));
            }
        }
        let mut flag_index = result_ids.len();
        for source in 0..result_ids.len() {
            for dependent in source + 1..result_ids.len() {
                if edge_flags[flag_index] {
                    edges.push(DependencyEdge::new(
                        id(result_ids[source]),
                        id(result_ids[dependent]),
                    ));
                }
                flag_index += 1;
            }
        }
        let results = result_ids
            .iter()
            .enumerate()
            .map(|(index, identity)| {
                PriorResult::new(
                    id(identity),
                    if observable[index] {
                        repair_region(0, 20)
                    } else {
                        repair_region(24, 30)
                    },
                    format!("generated-prior-{index}").into_bytes(),
                )
            })
            .collect::<Vec<_>>();
        let selection = Selection::new(
            initial.view().identity().clone(),
            vec![
                FactRegion::new(
                    Identity::new(replacement.prior_identity()),
                    repair_region(8, 12),
                ),
                FactRegion::new(
                    Identity::new(replacement.successor_identity()),
                    repair_region(8, 12),
                ),
            ],
            results.clone(),
            edges.clone(),
        );
        let plan = authority::repair::plan(
            initial.view(),
            successor.view(),
            &selection,
            authority::repair::Limits::owner_max(),
        )
        .expect("generated acyclic repair graph");

        let mut reachable = [false; 5];
        reachable.copy_from_slice(&edge_flags[..result_ids.len()]);
        let mut flag_index = result_ids.len();
        for source in 0..result_ids.len() {
            for dependent in source + 1..result_ids.len() {
                if edge_flags[flag_index] && reachable[source] {
                    reachable[dependent] = true;
                }
                flag_index += 1;
            }
        }
        let mut expected = result_ids
            .iter()
            .enumerate()
            .filter_map(|(index, identity)| {
                (reachable[index] && observable[index]).then_some(*identity)
            })
            .collect::<Vec<_>>();
        expected.sort_unstable();
        prop_assert_eq!(
            plan.affected().map(Identity::as_str).collect::<Vec<_>>(),
            expected
        );

        edges.reverse();
        let mut permuted_facts = selection.fact_regions().to_vec();
        permuted_facts.reverse();
        let mut permuted_results = results;
        permuted_results.reverse();
        let permuted = Selection::new(
            initial.view().identity().clone(),
            permuted_facts,
            permuted_results,
            edges,
        );
        let permuted_plan = authority::repair::plan(
            initial.view(),
            successor.view(),
            &permuted,
            authority::repair::Limits::owner_max(),
        )
        .expect("edge permutation");
        prop_assert_eq!(plan.bytes(), permuted_plan.bytes());
        prop_assert_eq!(plan.identity(), permuted_plan.identity());
    }
}

#[trace("TC-009", "FR-009-AC-2", "FR-009-AC-4")]
#[test]
fn tc009_successor_replay_and_same_key_contradiction_are_exact() {
    use authority::availability::DependencyState;
    use authority::bundle::{Disposition, LineageView, Selection};

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let documents = derive_all(history, &owner, &subject);
    let initial_context = Context::new(history, &owner, &subject, 1, None);
    let initial_selection = Selection::new(bundle_components(&documents), vec![]);
    let initial = authority::bundle::publish(
        initial_context,
        &initial_selection,
        None,
        Limits::owner_max(),
    )
    .expect("initial bundle");
    let prior_bytes = initial.document().bytes().to_vec();

    let selection = successor_bundle_selection(
        history,
        &owner,
        &subject,
        &documents,
        2,
        vec![],
        DependencyState::Available,
    );
    let context = Context::new(history, &owner, &subject, 2, Some(initial.document()));
    let successor = authority::bundle::publish(
        context,
        &selection,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect("valid direct successor");
    assert_eq!(successor.disposition(), Disposition::Published);
    assert_eq!(
        successor.document().predecessor(),
        Some(initial.document().identity())
    );
    assert_eq!(successor.view().payload().replacements().len(), 1);
    assert_eq!(initial.document().bytes(), prior_bytes);
    authority::bundle::read_revision(
        successor.document().bytes(),
        context,
        &selection,
        Some(initial.view()),
        Limits::owner_max(),
    )
    .expect("strict-read historical successor from its exact predecessor view");

    let known_child = LineageView::from_views(
        initial.view(),
        std::slice::from_ref(successor.view()),
        Limits::owner_max(),
    )
    .expect("known child lineage");
    let replay =
        authority::bundle::publish(context, &selection, Some(&known_child), Limits::owner_max())
            .expect("known exact child replay");
    assert_eq!(replay.disposition(), Disposition::Replayed);
    assert_eq!(replay.document().bytes(), successor.document().bytes());

    let head_replay = authority::bundle::publish(
        context,
        &selection,
        Some(successor.lineage()),
        Limits::owner_max(),
    )
    .expect("current-head replay");
    assert_eq!(head_replay.disposition(), Disposition::Replayed);

    let conflicting_selection = successor_bundle_selection(
        history,
        &owner,
        &subject,
        &documents,
        2,
        vec![id("result:O1")],
        DependencyState::Unavailable,
    );
    let conflicting = authority::bundle::publish(
        context,
        &conflicting_selection,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect("derive competing candidate against empty lineage");
    let conflicting_lineage = LineageView::from_views(
        initial.view(),
        std::slice::from_ref(conflicting.view()),
        Limits::owner_max(),
    )
    .expect("competing keyed child lineage");
    let error = authority::bundle::publish(
        context,
        &selection,
        Some(&conflicting_lineage),
        Limits::owner_max(),
    )
    .expect_err("same key with unequal bytes must refuse");
    assert_eq!(error.code(), authority::ErrorCode::IdentityContradiction);
    assert_eq!(initial.document().bytes(), prior_bytes);
}

#[trace("TC-009", "FR-009-AC-2", "FR-009-AC-3")]
#[test]
fn tc009_every_component_role_replaces_one_fact_at_a_time() {
    use authority::bundle::{Replacement, Selection};

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let prior_documents = derive_all(history, &owner, &subject);
    let revised_documents = derive_revised_all(history, &owner, &subject, &prior_documents);
    let prior_components = bundle_components(&prior_documents);
    let revised_components = bundle_components(&revised_documents);
    let initial = authority::bundle::publish(
        Context::new(history, &owner, &subject, 1, None),
        &Selection::new(prior_components.clone(), vec![]),
        None,
        Limits::owner_max(),
    )
    .expect("initial complete bundle");
    let predecessor_bytes = initial.document().bytes().to_vec();

    for (index, (prior, revised)) in prior_components.iter().zip(&revised_components).enumerate() {
        assert_eq!(prior.role(), revised.role());
        let replacement =
            Replacement::new(prior, revised).expect("same-role and same-fact-subject replacement");
        let mut selected = prior_components.clone();
        selected[index] = revised.clone();
        let publication = authority::bundle::publish(
            Context::new(history, &owner, &subject, 2, Some(initial.document())),
            &Selection::new(selected, vec![replacement]),
            Some(initial.lineage()),
            Limits::owner_max(),
        )
        .expect("publish one independently replaced role");
        let edge = publication
            .view()
            .payload()
            .replacements()
            .next()
            .expect("one replacement edge");
        assert_eq!(edge.role(), prior.role());
        assert_eq!(edge.prior_identity(), prior.identity());
        assert_eq!(edge.successor_identity(), revised.identity());
        assert_eq!(initial.document().bytes(), predecessor_bytes);
    }
}

#[trace("TC-009", "FR-009-AC-3")]
#[test]
fn tc009_stale_and_known_sibling_lineage_refuse_distinctly() {
    use authority::availability::DependencyState;
    use authority::bundle::{LineageView, Selection};

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let documents = derive_all(history, &owner, &subject);
    let initial_context = Context::new(history, &owner, &subject, 1, None);
    let initial_selection = Selection::new(bundle_components(&documents), vec![]);
    let initial = authority::bundle::publish(
        initial_context,
        &initial_selection,
        None,
        Limits::owner_max(),
    )
    .expect("initial bundle");
    let selection_two = successor_bundle_selection(
        history,
        &owner,
        &subject,
        &documents,
        2,
        vec![],
        DependencyState::Available,
    );
    let context_two = Context::new(history, &owner, &subject, 2, Some(initial.document()));
    let successor = authority::bundle::publish(
        context_two,
        &selection_two,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect("revision two");
    let missing =
        authority::bundle::publish(context_two, &selection_two, None, Limits::owner_max())
            .expect_err("later publication requires lineage");
    assert_eq!(missing.code(), authority::ErrorCode::StaleHead);
    let missing_predecessor = authority::bundle::publish(
        Context::new(history, &owner, &subject, 2, None),
        &selection_two,
        None,
        Limits::owner_max(),
    )
    .expect_err("later revision without a predecessor must refuse");
    assert_eq!(
        missing_predecessor.code(),
        authority::ErrorCode::RevisionMismatch
    );

    let self_link = LineageView::from_views(
        initial.view(),
        std::slice::from_ref(initial.view()),
        Limits::owner_max(),
    )
    .expect_err("lineage cannot retain its head as its own child");
    assert_eq!(self_link.code(), authority::ErrorCode::InvalidSelection);
    let non_increasing = authority::bundle::publish(
        Context::new(history, &owner, &subject, 1, Some(initial.document())),
        &initial_selection,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect_err("non-increasing revision must refuse");
    assert_eq!(
        non_increasing.code(),
        authority::ErrorCode::PredecessorMismatch
    );

    let mut foreign_owner = owner.clone();
    foreign_owner.definition_identity = id("definition:foreign-lineage");
    let foreign_documents = derive_all(history, &foreign_owner, &subject);
    let foreign = authority::bundle::publish(
        Context::new(history, &foreign_owner, &subject, 1, None),
        &Selection::new(bundle_components(&foreign_documents), vec![]),
        None,
        Limits::owner_max(),
    )
    .expect("foreign initial lineage");
    let cross_authority = authority::bundle::publish(
        context_two,
        &selection_two,
        Some(foreign.lineage()),
        Limits::owner_max(),
    )
    .expect_err("cross-authority lineage must refuse");
    assert_eq!(
        cross_authority.code(),
        authority::ErrorCode::AuthorityMismatch
    );

    let foreign_qualified = qualified_with_window("window:O2");
    let foreign_subject = selected_subject(&foreign_qualified);
    let foreign_history = History::batch(&foreign_qualified);
    let foreign_scope_documents = derive_all(foreign_history, &owner, &foreign_subject);
    let foreign_scope = authority::bundle::publish(
        Context::new(foreign_history, &owner, &foreign_subject, 1, None),
        &Selection::new(bundle_components(&foreign_scope_documents), vec![]),
        None,
        Limits::owner_max(),
    )
    .expect("foreign-scope initial lineage");
    let cross_scope = authority::bundle::publish(
        context_two,
        &selection_two,
        Some(foreign_scope.lineage()),
        Limits::owner_max(),
    )
    .expect_err("cross-scope lineage must refuse");
    assert_eq!(cross_scope.code(), authority::ErrorCode::AuthorityMismatch);

    let selection_three = successor_bundle_selection(
        history,
        &owner,
        &subject,
        &documents,
        3,
        vec![id("result:O1")],
        DependencyState::Unavailable,
    );
    let context_three = Context::new(history, &owner, &subject, 3, Some(initial.document()));
    let stale = authority::bundle::publish(
        context_three,
        &selection_three,
        Some(successor.lineage()),
        Limits::owner_max(),
    )
    .expect_err("old predecessor is stale against newer head");
    assert_eq!(stale.code(), authority::ErrorCode::StaleHead);

    let sibling = authority::bundle::publish(
        context_three,
        &selection_three,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect("revision-three competing direct child");
    let branched = LineageView::from_views(
        initial.view(),
        std::slice::from_ref(sibling.view()),
        Limits::owner_max(),
    )
    .expect("known sibling lineage");
    let error = authority::bundle::publish(
        context_two,
        &selection_two,
        Some(&branched),
        Limits::owner_max(),
    )
    .expect_err("different known child must refuse as sibling");
    assert_eq!(error.code(), authority::ErrorCode::KnownSibling);
}

#[trace("TC-009", "FR-009-AC-5")]
#[test]
fn tc009_component_replacement_and_lineage_bounds_fail_closed() {
    use authority::availability::DependencyState;
    use authority::bundle::{Component, ComponentRole, LineageView, Replacement, Selection};

    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let documents = derive_all(history, &owner, &subject);
    let context = Context::new(history, &owner, &subject, 1, None);
    let components = bundle_components(&documents);

    for index in 0..components.len() {
        let mut omitted = components.clone();
        omitted.remove(index);
        let error = authority::bundle::publish(
            context,
            &Selection::new(omitted, vec![]),
            None,
            Limits::owner_max(),
        )
        .expect_err("every missing component family must refuse");
        assert_eq!(error.code(), authority::ErrorCode::InvalidSelection);

        let mut duplicated = components.clone();
        duplicated.push(components[index].clone());
        let error = authority::bundle::publish(
            context,
            &Selection::new(duplicated, vec![]),
            None,
            Limits::owner_max(),
        )
        .expect_err("every duplicated component family must refuse");
        assert_eq!(error.code(), authority::ErrorCode::InvalidSelection);
    }
    assert_eq!(
        Component::from_document(ComponentRole::Availability, &documents.observation)
            .expect_err("cross-role component")
            .code(),
        authority::ErrorCode::InvalidSelection
    );
    assert_eq!(
        Replacement::new(&components[0], &components[1])
            .expect_err("cross-role replacement")
            .code(),
        authority::ErrorCode::InvalidReplacement
    );
    assert_eq!(
        Replacement::new(&components[0], &components[0])
            .expect_err("self replacement")
            .code(),
        authority::ErrorCode::InvalidReplacement
    );
    let changed_subject_document = authority::availability::derive(
        Context::new(history, &owner, &subject, 2, Some(&documents.availability)),
        &authority::availability::Selection::new(
            vec![id("result:other")],
            vec![id("result:other")],
            DependencyState::Available,
            DependencyState::Available,
        ),
        Limits::owner_max(),
    )
    .expect("different result-subject availability");
    let prior_availability = components
        .iter()
        .find(|component| component.role() == ComponentRole::Availability)
        .expect("prior availability component");
    let changed_subject =
        Component::from_document(ComponentRole::Availability, &changed_subject_document)
            .expect("changed result-subject component");
    assert_eq!(
        Replacement::new(prior_availability, &changed_subject)
            .expect_err("replacement cannot change its fact-level subject")
            .code(),
        authority::ErrorCode::InvalidReplacement
    );

    let mut foreign_owner = owner.clone();
    foreign_owner.definition_identity = id("definition:foreign");
    let foreign_documents = derive_all(history, &foreign_owner, &subject);
    let foreign_availability =
        Component::from_document(ComponentRole::Availability, &foreign_documents.availability)
            .expect("foreign-authority availability component");
    assert_eq!(
        Replacement::new(prior_availability, &foreign_availability)
            .expect_err("replacement cannot cross authority")
            .code(),
        authority::ErrorCode::InvalidReplacement
    );
    let mut cross_wired = components.clone();
    let availability_index = cross_wired
        .iter()
        .position(|component| component.role() == ComponentRole::Availability)
        .expect("availability role");
    cross_wired[availability_index] =
        Component::from_document(ComponentRole::Availability, &foreign_documents.availability)
            .expect("foreign but internally valid component");
    let error = authority::bundle::publish(
        context,
        &Selection::new(cross_wired, vec![]),
        None,
        Limits::owner_max(),
    )
    .expect_err("foreign authority component must refuse");
    assert_eq!(error.code(), authority::ErrorCode::AuthorityMismatch);

    let foreign_qualified = qualified_with_window("window:foreign-replacement");
    let foreign_subject = selected_subject(&foreign_qualified);
    let foreign_history = History::batch(&foreign_qualified);
    let foreign_scope_documents = derive_all(foreign_history, &owner, &foreign_subject);
    let foreign_scope_availability = Component::from_document(
        ComponentRole::Availability,
        &foreign_scope_documents.availability,
    )
    .expect("foreign-scope availability component");
    assert_eq!(
        Replacement::new(prior_availability, &foreign_scope_availability)
            .expect_err("replacement cannot cross scope")
            .code(),
        authority::ErrorCode::InvalidReplacement
    );

    let component_limits = Limits {
        max_bundle_components: 10,
        ..Limits::owner_max()
    };
    let error = authority::bundle::publish(
        context,
        &Selection::new(components.clone(), vec![]),
        None,
        component_limits,
    )
    .expect_err("one-over component bound");
    assert_eq!(error.code(), authority::ErrorCode::ResourceIncomplete);
    assert_eq!(error.usage().bundle_components, 11);

    authority::bundle::publish(
        context,
        &Selection::new(components.clone(), vec![]),
        None,
        Limits {
            max_bundle_components: 11,
            ..Limits::owner_max()
        },
    )
    .expect("exact component bound admits");
    authority::bundle::publish(
        context,
        &Selection::new(components.clone(), vec![]),
        None,
        Limits {
            max_positions: 1,
            max_bundle_components: 11,
            ..Limits::owner_max()
        },
    )
    .expect("bundle position-fact projection does not consume position-ledger entry capacity");

    use authority::bundle::{Conflict, ConflictKind};
    let explicit_conflict = Conflict::new(ConflictKind::Duplicate, &components[0]);
    let conflict_selection =
        Selection::with_conflicts(components.clone(), vec![], vec![explicit_conflict.clone()]);
    let error = authority::bundle::publish(
        context,
        &conflict_selection,
        None,
        Limits {
            max_conflicts: 0,
            ..Limits::owner_max()
        },
    )
    .expect_err("one-over conflict bound");
    assert_eq!(error.code(), authority::ErrorCode::ResourceIncomplete);
    assert_eq!(error.usage().conflicts, 1);
    authority::bundle::publish(
        context,
        &conflict_selection,
        None,
        Limits {
            max_conflicts: 1,
            ..Limits::owner_max()
        },
    )
    .expect("exact conflict bound admits");
    let duplicate_conflicts = Selection::with_conflicts(
        components.clone(),
        vec![],
        vec![explicit_conflict.clone(), explicit_conflict],
    );
    assert_eq!(
        authority::bundle::publish(context, &duplicate_conflicts, None, Limits::owner_max())
            .expect_err("duplicate explicit conflict must refuse")
            .code(),
        authority::ErrorCode::InvalidSelection
    );

    let initial = authority::bundle::publish(
        context,
        &Selection::new(components, vec![]),
        None,
        Limits::owner_max(),
    )
    .expect("initial bundle");
    let successor_selection = successor_bundle_selection(
        history,
        &owner,
        &subject,
        &documents,
        2,
        vec![],
        DependencyState::Available,
    );
    let successor_context = Context::new(history, &owner, &subject, 2, Some(initial.document()));
    let omitted_replacement = Selection::new(successor_selection.components().to_vec(), vec![]);
    let error = authority::bundle::publish(
        successor_context,
        &omitted_replacement,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect_err("changed component requires an explicit replacement");
    assert_eq!(error.code(), authority::ErrorCode::InvalidReplacement);

    let duplicated_replacement = Selection::new(
        successor_selection.components().to_vec(),
        vec![
            successor_selection.replacements()[0].clone(),
            successor_selection.replacements()[0].clone(),
        ],
    );
    let error = authority::bundle::publish(
        successor_context,
        &duplicated_replacement,
        Some(initial.lineage()),
        Limits::owner_max(),
    )
    .expect_err("duplicate replacement must refuse");
    assert_eq!(error.code(), authority::ErrorCode::InvalidReplacement);
    let replacement_limits = Limits {
        max_replacements: 0,
        ..Limits::owner_max()
    };
    let error = authority::bundle::publish(
        successor_context,
        &successor_selection,
        Some(initial.lineage()),
        replacement_limits,
    )
    .expect_err("one-over replacement bound");
    assert_eq!(error.code(), authority::ErrorCode::ResourceIncomplete);
    assert_eq!(error.usage().replacements, 1);

    let successor = authority::bundle::publish(
        successor_context,
        &successor_selection,
        Some(initial.lineage()),
        Limits {
            max_replacements: 1,
            ..Limits::owner_max()
        },
    )
    .expect("exact replacement bound admits");
    LineageView::from_views(
        initial.view(),
        std::slice::from_ref(successor.view()),
        Limits {
            max_lineage_children: 1,
            ..Limits::owner_max()
        },
    )
    .expect("exact lineage-child bound admits");
    let lineage_limits = Limits {
        max_lineage_children: 0,
        ..Limits::owner_max()
    };
    let error = LineageView::from_views(
        initial.view(),
        std::slice::from_ref(successor.view()),
        lineage_limits,
    )
    .expect_err("one-over lineage-child bound");
    assert_eq!(error.code(), authority::ErrorCode::ResourceIncomplete);
    assert_eq!(error.usage().lineage_children, 1);
}
