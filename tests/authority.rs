// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-004: canonical observation-owner artifacts.

mod support;

use ix_trace_rs::trace;
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
    SubjectSelection {
        scope_identity: id("window:O1"),
        population_identity: qualified.scope().population_identity.clone(),
    }
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

struct Documents {
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
    Documents {
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
        capture: authority::capture::derive(
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
        .expect("derive capture"),
        progress: authority::progress::derive(
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
        .expect("derive progress"),
        closure: authority::closure::derive(
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
        .expect("derive closure"),
        completeness: authority::completeness::derive(
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
fn tc004_all_ten_owner_contracts_derive_canonical_documents() {
    let qualified = qualified();
    let owner = owner();
    let subject = subject(&qualified);
    let documents = derive_all(History::batch(&qualified), &owner, &subject);
    assert_eq!(
        qualified.scope().membership_rule_identity.as_str(),
        "sha256-jcs:51daded8bd0ade3b3ce5addc3e5e58318cf661dad0e3f80b8a58be1e33eb5ccb"
    );
    let contracts = [
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
    assert_eq!(contracts.len(), 10);
    assert!(contracts
        .iter()
        .all(|contract| contract.starts_with("quire.observation.")));
    for document in [
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
    assert_eq!(codes.len(), 14);
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
