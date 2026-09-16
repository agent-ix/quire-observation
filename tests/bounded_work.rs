// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-013: revision-bound inventory of every C00 expansion path and its bound.

use ix_trace_rs::trace;
use sha2::{Digest as _, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceId {
    Common,
    Partial,
    Activation,
    Progress,
    Bundle,
    Repair,
    Coordination,
    Query,
    Admission,
    AuthorityEvidence,
    PartialEvidence,
    ActivationEvidence,
}

struct Source {
    id: SourceId,
    path: &'static str,
    text: &'static str,
    sha256: &'static str,
}

const SOURCES: &[Source] = &[
    Source {
        id: SourceId::Common,
        path: "src/authority/common.rs",
        text: include_str!("../src/authority/common.rs"),
        sha256: "379bcf5cd4bf014cb0b4633bc76b188e4aed308f2532db3fd5028d99a2d870bd",
    },
    Source {
        id: SourceId::Partial,
        path: "src/authority/partial.rs",
        text: include_str!("../src/authority/partial.rs"),
        sha256: "33de9f39f3e422bf2d364ceba7e302101b230d4fe6a3ce8d4f6ce4cf8a0e8237",
    },
    Source {
        id: SourceId::Activation,
        path: "src/authority/activation.rs",
        text: include_str!("../src/authority/activation.rs"),
        sha256: "ada51d5b9e7c5f16a8f1d15cdb691afed7cd615724db142d07834c882a854651",
    },
    Source {
        id: SourceId::Progress,
        path: "src/authority/progress.rs",
        text: include_str!("../src/authority/progress.rs"),
        sha256: "6e1b8892adc8a4b6371b1b316bdc2ed346173b48a3df475ddf524f988eeed0e6",
    },
    Source {
        id: SourceId::Bundle,
        path: "src/authority/bundle.rs",
        text: include_str!("../src/authority/bundle.rs"),
        sha256: "752c833a4103edca25f9e3ed029dce9fcaf1a98e5b701a12ab9c8e71067f0d2e",
    },
    Source {
        id: SourceId::Repair,
        path: "src/authority/repair.rs",
        text: include_str!("../src/authority/repair.rs"),
        sha256: "98e8781b95d32b7fd756c4465670ebc3ceba456bef30e48379424d378f19baa8",
    },
    Source {
        id: SourceId::Coordination,
        path: "src/authority/coordination.rs",
        text: include_str!("../src/authority/coordination.rs"),
        sha256: "5be19d0e20d935f75534fbaa19b17614b118e3952969768366d2a7be1207eec5",
    },
    Source {
        id: SourceId::Query,
        path: "src/authority/query.rs",
        text: include_str!("../src/authority/query.rs"),
        sha256: "26eaaf49db4113d5c011bf474a496e189351e9274de01be911ac8f1d82ec612f",
    },
    Source {
        id: SourceId::Admission,
        path: "src/lib.rs",
        text: include_str!("../src/lib.rs"),
        sha256: "6cc12016ce41d8da69e9816e2972ed91464119ed7ac2a8125948b793251e5ff2",
    },
    Source {
        id: SourceId::AuthorityEvidence,
        path: "tests/authority.rs",
        text: include_str!("authority.rs"),
        sha256: "5ceab5286f5e138d81d9bc617bb62ba779d833fc9fe033ab7cdd4f2223315edc",
    },
    Source {
        id: SourceId::PartialEvidence,
        path: "tests/partial_interval.rs",
        text: include_str!("partial_interval.rs"),
        sha256: "711adfb64cfb0827dd9d5a62ad65081fc1321eb5d6089d660b8bd6e33e1a1d59",
    },
    Source {
        id: SourceId::ActivationEvidence,
        path: "tests/activation_authority.rs",
        text: include_str!("activation_authority.rs"),
        sha256: "0a8e79c5dd29e20920d1d95753fd2bcbe5e2c17b11b9964b90c157ff0aeff5ca",
    },
];

struct AuditRow {
    name: &'static str,
    production: SourceId,
    entrypoint: &'static str,
    expansion: &'static str,
    retained: &'static str,
    bound_source: SourceId,
    limit: &'static str,
    precheck_or_charge: &'static str,
    checked_conversion_or_arithmetic: &'static str,
    evidence_source: SourceId,
    evidence_test: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceKind {
    StructurallyDominated,
    ExactOneOver,
}

const ROWS: &[AuditRow] = &[
    AuditRow {
        name: "common.strict-reader-preflight",
        production: SourceId::Common,
        entrypoint: "pub(crate) fn read_exact<P>",
        expansion: "preflight_for_contract(bytes, effective, contract)?",
        retained: "read_exact_preflighted(contract, bytes, expected, limits, observed)",
        bound_source: SourceId::Common,
        limit: "max_input_bytes",
        precheck_or_charge: "let observed = preflight_for_contract(bytes, effective, contract)?",
        checked_conversion_or_arithmetic:
            "read_exact_preflighted(contract, bytes, expected, limits, observed)",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "common.preflighted-strict-reader-deserialization",
        production: SourceId::Common,
        entrypoint: "pub(crate) fn read_exact_preflighted<P>",
        expansion: "serde_json::from_slice(bytes)",
        retained: "let envelope: Envelope<P> = serde_json::from_slice(bytes)",
        bound_source: SourceId::Common,
        limit: "max_input_bytes",
        precheck_or_charge: "let observed = preflight_for_contract(bytes, effective, contract)?",
        checked_conversion_or_arithmetic: "let canonical = to_bounded_json",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "common.parser-object-loop",
        production: SourceId::Common,
        entrypoint: "fn object(&mut self",
        expansion: "loop {",
        retained: "let key = self.string()?",
        bound_source: SourceId::Common,
        limit: "max_visited_fields",
        precheck_or_charge: "self.charge_visit()?",
        checked_conversion_or_arithmetic: "self.value(depth + 1, kind)?",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "common.parser-array-loop",
        production: SourceId::Common,
        entrypoint: "fn array(&mut self",
        expansion: "loop {",
        retained: "let mut count = 0usize",
        bound_source: SourceId::Common,
        limit: "max_visited_fields",
        precheck_or_charge: "self.charge_visit()?",
        checked_conversion_or_arithmetic: "count.checked_add(1)",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "common.canonical-output-writer",
        production: SourceId::Common,
        entrypoint: "pub(crate) fn to_bounded_json",
        expansion: "serde_json::to_writer",
        retained: ".try_reserve(buffer.len())",
        bound_source: SourceId::Common,
        limit: "limit: usize",
        precheck_or_charge: "if next > self.limit",
        checked_conversion_or_arithmetic: ".checked_add(buffer.len())",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "common.document-fixed-point",
        production: SourceId::Common,
        entrypoint: "pub(crate) fn build_document<P>",
        expansion: "for _ in 0..8",
        retained: "return Ok(Document {",
        bound_source: SourceId::Common,
        limit: "effective.max_output_bytes",
        precheck_or_charge: "validate_usage(usage, effective)?",
        checked_conversion_or_arithmetic: "to_bounded_json(&envelope, effective.max_output_bytes)?",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "common.escaped-collection-key-classification",
        production: SourceId::Common,
        entrypoint: "fn json_key_matches(",
        expansion: "while encoded_index < encoded.len()",
        retained: "let mut expected_index = 0usize",
        bound_source: SourceId::Common,
        limit: "max_bundle_components",
        precheck_or_charge: "ArrayKind::BundleComponents",
        checked_conversion_or_arithmetic: "encoded.get(encoded_index)",
        evidence_source: SourceId::Common,
        evidence_test: "",
    },
    AuditRow {
        name: "partial.qualified-history-lookup",
        production: SourceId::Partial,
        entrypoint: "pub fn derive(context: Context<'_>",
        expansion: ".find(|record| record.identity == selection.observation_identity)",
        retained: "let payload = PartialView {",
        bound_source: SourceId::Admission,
        limit: "pub max_records: usize",
        precheck_or_charge: "if request.records.len() > request.limits.max_records",
        checked_conversion_or_arithmetic: "selection.interval.validate_limits(limits)?",
        evidence_source: SourceId::Admission,
        evidence_test: "",
    },
    AuditRow {
        name: "partial.owner-document",
        production: SourceId::Partial,
        entrypoint: "pub fn derive(context: Context<'_>",
        expansion: "build_document(CONTRACT, context, &payload",
        retained: "let payload = PartialView {",
        bound_source: SourceId::Common,
        limit: "max_output_bytes",
        precheck_or_charge: "pub(crate) fn build_document<P>",
        checked_conversion_or_arithmetic: ".max(payload.ingestion_position.len())",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "partial.byte-first-strict-read",
        production: SourceId::Partial,
        entrypoint: "pub fn read(\n",
        expansion: "preflight_for_contract(bytes, limits.effective(), CONTRACT)?",
        retained: "let expected = derive(context, selection, limits)?",
        bound_source: SourceId::Common,
        limit: "max_input_bytes",
        precheck_or_charge: "if bytes.len() > limits.max_input_bytes",
        checked_conversion_or_arithmetic:
            "read_exact_preflighted(CONTRACT, bytes, &expected, limits, observed)",
        evidence_source: SourceId::PartialEvidence,
        evidence_test: "tc007_partial_fact_is_canonical_strictly_read_and_fail_closed",
    },
    AuditRow {
        name: "activation.strict-proof-copy",
        production: SourceId::Activation,
        entrypoint: "pub fn new(\n        capture: &super::capture::View",
        expansion: "capture_bindings.extend",
        retained: "let mut capture_bindings = Vec::new()",
        bound_source: SourceId::Common,
        limit: "max_capture_bindings",
        precheck_or_charge: "ArrayKind::Captures",
        checked_conversion_or_arithmetic: ".try_reserve(capture.payload().bindings().len())",
        evidence_source: SourceId::Activation,
        evidence_test: "",
    },
    AuditRow {
        name: "activation.capture-set",
        production: SourceId::Activation,
        entrypoint: "pub fn derive(context: Context<'_>",
        expansion: "for capture in &selection.activation.captures",
        retained: "let mut captures = Vec::new()",
        bound_source: SourceId::Activation,
        limit: "effective.max_capture_bindings",
        precheck_or_charge: "captures.len() > effective.max_capture_bindings",
        checked_conversion_or_arithmetic: ".try_reserve(selection.activation.captures.len())",
        evidence_source: SourceId::ActivationEvidence,
        evidence_test: "tc008_versioned_owner_round_trips_all_independent_authority",
    },
    AuditRow {
        name: "activation.qualified-record-index",
        production: SourceId::Activation,
        entrypoint: "pub fn derive(context: Context<'_>",
        expansion: "let record_index = qualified",
        retained: "collect::<BTreeMap<_, _>>()",
        bound_source: SourceId::Activation,
        limit: "max_population_entries",
        precheck_or_charge: "qualified.records().len() > effective.max_population_entries",
        checked_conversion_or_arithmetic: "qualified.records().len()",
        evidence_source: SourceId::ActivationEvidence,
        evidence_test: "tc008_versioned_owner_round_trips_all_independent_authority",
    },
    AuditRow {
        name: "activation.trigger-absence-scan",
        production: SourceId::Activation,
        entrypoint: "pub fn derive(context: Context<'_>",
        expansion:
            ".any(|record| record.trigger_identity == selection.activation.trigger_identity)",
        retained: "qualified.records()",
        bound_source: SourceId::Activation,
        limit: "max_population_entries",
        precheck_or_charge: "qualified.records().len() > effective.max_population_entries",
        checked_conversion_or_arithmetic: "qualified.records().len()",
        evidence_source: SourceId::Activation,
        evidence_test: "",
    },
    AuditRow {
        name: "activation.source-support-sets",
        production: SourceId::Activation,
        entrypoint: "fn exact_source_set(",
        expansion: "for source in sources",
        retained: "let mut exact = BTreeSet::new()",
        bound_source: SourceId::Activation,
        limit: "effective.max_required_sources",
        precheck_or_charge: "sources.len() > effective.max_required_sources",
        checked_conversion_or_arithmetic: "validate_string(source.as_str(), limits)?",
        evidence_source: SourceId::ActivationEvidence,
        evidence_test: "tc008_versioned_owner_round_trips_all_independent_authority",
    },
    AuditRow {
        name: "activation.byte-first-strict-read",
        production: SourceId::Activation,
        entrypoint: "pub fn read(\n",
        expansion: "preflight_for_contract(bytes, limits.effective(), CONTRACT)?",
        retained: "let expected = derive(context, selection, limits)?",
        bound_source: SourceId::Common,
        limit: "max_input_bytes",
        precheck_or_charge: "if bytes.len() > limits.max_input_bytes",
        checked_conversion_or_arithmetic:
            "read_exact_preflighted(CONTRACT, bytes, &expected, limits, observed)",
        evidence_source: SourceId::ActivationEvidence,
        evidence_test: "tc008_versioned_owner_round_trips_all_independent_authority",
    },
    AuditRow {
        name: "bundle.component-replacement-conflict-projection",
        production: SourceId::Bundle,
        entrypoint: "fn payload(\n    contract:",
        expansion: ".try_fold(0usize, |total, component|",
        retained: "let mut components: Vec<_> = selection.components.iter().collect()",
        bound_source: SourceId::Bundle,
        limit: "effective.max_bundle_components",
        precheck_or_charge: "max_bundle_components",
        checked_conversion_or_arithmetic: "checked_add(component.document_bytes)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc009_component_replacement_and_lineage_bounds_fail_closed",
    },
    AuditRow {
        name: "bundle.lineage-source-stamps",
        production: SourceId::Bundle,
        entrypoint: "pub fn from_views(head: &View",
        expansion: "for child in direct_children",
        retained: "let mut children = Vec::new()",
        bound_source: SourceId::Bundle,
        limit: "effective.max_lineage_children",
        precheck_or_charge: "if source_bytes > effective.max_input_bytes",
        checked_conversion_or_arithmetic: "total.checked_add(child.bytes().len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc009_component_replacement_and_lineage_bounds_fail_closed",
    },
    AuditRow {
        name: "bundle.supplied-lineage-replay",
        production: SourceId::Bundle,
        entrypoint: "fn payload(\n    contract:",
        expansion: "lineage.direct_children.len()",
        retained: "lineage.document.bytes().len()",
        bound_source: SourceId::Bundle,
        limit: "effective.max_lineage_children",
        precheck_or_charge: "lineage.direct_children.len() > effective.max_lineage_children",
        checked_conversion_or_arithmetic:
            "lineage.document.bytes().len() > effective.max_input_bytes",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc009_component_replacement_and_lineage_bounds_fail_closed",
    },
    AuditRow {
        name: "bundle.replacement-linear-merge",
        production: SourceId::Bundle,
        entrypoint: "fn validate_replacements(",
        expansion: "while prior_index < predecessor.components.len()",
        retained: "let mut removed = Vec::new()",
        bound_source: SourceId::Bundle,
        limit: "effective.max_bundle_components",
        precheck_or_charge: "selection.components.len() > effective.max_bundle_components",
        checked_conversion_or_arithmetic: ".try_reserve_exact(predecessor.components.len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc009_component_replacement_and_lineage_bounds_fail_closed",
    },
    AuditRow {
        name: "bundle.byte-first-strict-read",
        production: SourceId::Bundle,
        entrypoint: "fn read_for(\n",
        expansion: "preflight_for_contract(bytes, limits.effective(), contract)?",
        retained: "read_for_preflighted(",
        bound_source: SourceId::Common,
        limit: "max_input_bytes",
        precheck_or_charge: "if bytes.len() > limits.max_input_bytes",
        checked_conversion_or_arithmetic: "observed,",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc009_initial_bundle_is_canonical_complete_and_strictly_read",
    },
    AuditRow {
        name: "bundle.revision-byte-first-strict-read",
        production: SourceId::Bundle,
        entrypoint: "fn read_revision_for(\n",
        expansion: "preflight_for_contract(bytes, limits.effective(), contract)?",
        retained: "LineageView::from_views(view, &[], limits)?",
        bound_source: SourceId::Common,
        limit: "max_input_bytes",
        precheck_or_charge: "if bytes.len() > limits.max_input_bytes",
        checked_conversion_or_arithmetic: "read_for_preflighted(",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc009_successor_replay_and_same_key_contradiction_are_exact",
    },
    AuditRow {
        name: "repair.graph-indexes",
        production: SourceId::Repair,
        entrypoint: "pub fn plan(prior: &View",
        expansion: "for edge in &selection.edges",
        retained: "let mut outgoing: BTreeMap",
        bound_source: SourceId::Repair,
        limit: "effective.max_edges",
        precheck_or_charge: "selection.edges.len() > effective.max_edges",
        checked_conversion_or_arithmetic: ".checked_add(selection.prior_results.len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc010_each_planner_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "repair.affected-breadth-first-closure",
        production: SourceId::Repair,
        entrypoint: "fn affected_closure<'a>",
        expansion: "while let Some((node, changed_fact)) = queue.pop_front()",
        retained: "let mut affected = BTreeSet::new()",
        bound_source: SourceId::Repair,
        limit: "max_work",
        precheck_or_charge: "work.tick()?",
        checked_conversion_or_arithmetic: "work.tick()?",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc010_each_planner_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "repair.acyclic-and-topological-walks",
        production: SourceId::Repair,
        entrypoint: "fn validate_acyclic(",
        expansion: "while let Some(identity) = ready.pop_first()",
        retained: "let mut ready: BTreeSet<&str>",
        bound_source: SourceId::Repair,
        limit: "max_nodes",
        precheck_or_charge: "work.tick()?",
        checked_conversion_or_arithmetic: "entry.checked_add(1)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc010_each_planner_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "repair.dependency-retention",
        production: SourceId::Repair,
        entrypoint: "pub fn plan(prior: &View",
        expansion: ".filter(|edge| affected_identities.contains",
        retained: "let mut dependencies = Vec::new()",
        bound_source: SourceId::Repair,
        limit: "max_retained_bytes",
        precheck_or_charge: "ensure_state_bytes(&work)?",
        checked_conversion_or_arithmetic: ".checked_add(dependency_bytes)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc010_each_planner_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "coordination.job-input-preparation",
        production: SourceId::Coordination,
        entrypoint: "fn prepare<'a>(",
        expansion: "for identity in plan.recomputation_order()",
        retained: "let mut prepared = Vec::new()",
        bound_source: SourceId::Coordination,
        limit: "max_jobs",
        precheck_or_charge: "selection.jobs.len() > effective.max_jobs",
        checked_conversion_or_arithmetic: ".try_reserve_exact(selection.jobs.len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.dependency-preparation-scan",
        production: SourceId::Coordination,
        entrypoint: "fn prepare<'a>(",
        expansion: "for edge in plan.dependencies()",
        retained: "let mut expected_source_count = 0usize",
        bound_source: SourceId::Coordination,
        limit: "max_work",
        precheck_or_charge: "work.tick()?",
        checked_conversion_or_arithmetic: ".checked_add(1)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.duplicated-result-input-state",
        production: SourceId::Coordination,
        entrypoint: "fn job_state_bytes(job: &Job",
        expansion: "job.result_inputs.iter().try_fold",
        retained: "let result_input_bytes",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "work.charge_state(job.external_inputs.state_bytes)?",
        checked_conversion_or_arithmetic: ".checked_add(result_input_state_bytes(input, work)?)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.incremental-prefix-state",
        production: SourceId::Coordination,
        entrypoint: "pub fn push(&mut self",
        expansion: "self.current_inputs.push(input)",
        retained: "self.current_inputs",
        bound_source: SourceId::Coordination,
        limit: "external_inputs.state_bytes",
        precheck_or_charge: "next_state > self.prepared[current].job.external_inputs.state_bytes",
        checked_conversion_or_arithmetic: ".checked_add(input_bytes)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.prefix-preallocation",
        production: SourceId::Coordination,
        entrypoint: "fn ensure_prefix_capacity(",
        expansion: "prefix_state_bytes_for_outcome(",
        retained: "let bytes = prefix_state_bytes_for_outcome(",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "self.work.ensure_state_capacity(bytes)?",
        checked_conversion_or_arithmetic: ".checked_add(prepared.result_inputs.len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.push-terminal-clone-preallocation",
        production: SourceId::Coordination,
        entrypoint: "pub fn push(&mut self",
        expansion: "terminal.clone()",
        retained: "let outcome = if let Some(terminal)",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "ensure_state_capacity(outcome_state_bytes(terminal",
        checked_conversion_or_arithmetic: "outcome_state_bytes(terminal",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.close-terminal-clone-preallocation",
        production: SourceId::Coordination,
        entrypoint: "pub fn close(&mut self",
        expansion: "terminal.clone()",
        retained: "let outcome = if let Some(terminal)",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "ensure_state_capacity(outcome_state_bytes(terminal",
        checked_conversion_or_arithmetic: "outcome_state_bytes(terminal",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.pairwise-orders",
        production: SourceId::Coordination,
        entrypoint: "fn derive_orders(",
        expansion: "for left in 0..right",
        retained: "orders.push(IntervalOrder",
        bound_source: SourceId::Coordination,
        limit: "max_possible_orders",
        precheck_or_charge: "if next > work.limits.max_possible_orders",
        checked_conversion_or_arithmetic: ".checked_add(1)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.evaluator-output-retention",
        production: SourceId::Coordination,
        entrypoint: "fn convert_outcome(",
        expansion: "EvaluatorOutcome::Decisive",
        retained: "let replacement = build_replacement",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "work.ensure_state_capacity(retained_bytes)?",
        checked_conversion_or_arithmetic: "replacement_state_bytes_before_build",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.support-evidence-index",
        production: SourceId::Coordination,
        entrypoint: "fn validate_support_and_digest(",
        expansion: "for identity in &support.evidence_identities",
        retained: "let mut evidence = BTreeMap::new()",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "work.ensure_state_capacity(evidence_index_bytes)?",
        checked_conversion_or_arithmetic:
            ".checked_mul(std::mem::size_of::<(&str, EvaluatorInput<'_>)>())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.support-input-search",
        production: SourceId::Coordination,
        entrypoint: "fn find_input<'a>(",
        expansion: "while left < right",
        retained: "let mut right = inputs.len()",
        bound_source: SourceId::Coordination,
        limit: "max_work",
        precheck_or_charge: "work.tick()?",
        checked_conversion_or_arithmetic: "left + (right - left) / 2",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "coordination.unaffected-retention",
        production: SourceId::Coordination,
        entrypoint: "fn finish_run(",
        expansion: "for retained in plan.unaffected()",
        retained: "unaffected.push(RetainedResult",
        bound_source: SourceId::Coordination,
        limit: "max_state_bytes",
        precheck_or_charge: "work.charge_state(",
        checked_conversion_or_arithmetic: ".checked_add(retained.bytes().len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test:
            "tc010_each_coordinator_limit_admits_exact_and_refuses_one_over_on_both_paths",
    },
    AuditRow {
        name: "query.input-preflight",
        production: SourceId::Query,
        entrypoint: "pub fn evaluate(",
        expansion: "selection_input_bytes(selection)?",
        retained: "let mut work = Work",
        bound_source: SourceId::Query,
        limit: "max_input_bytes",
        precheck_or_charge: "if input_bytes > effective.max_input_bytes",
        checked_conversion_or_arithmetic: ".checked_add(lineage.head().bytes().len())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc011_each_query_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "query.exact-multi-role-selection",
        production: SourceId::Query,
        entrypoint: "fn selected_fact<'a>(",
        expansion: "for component in head",
        retained: "let mut facts = facts",
        bound_source: SourceId::Query,
        limit: "max_work",
        precheck_or_charge: "work.tick()?",
        checked_conversion_or_arithmetic: "work.tick()?",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc011_each_query_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "query.member-join-and-ordered-insert",
        production: SourceId::Query,
        entrypoint: "fn join_members<'a>(",
        expansion: "while insert_at != 0",
        retained: "joined.insert(insert_at, joined_member)",
        bound_source: SourceId::Query,
        limit: "max_members",
        precheck_or_charge: "work.ensure_state_capacity(structural_bytes)?",
        checked_conversion_or_arithmetic: ".checked_mul(std::mem::size_of::<Joined<'_>>())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc011_each_query_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "query.duplicate-fold-and-participation",
        production: SourceId::Query,
        entrypoint: "fn aggregate(",
        expansion: "for earlier in prior.iter().filter",
        retained: "participation.push(Participation",
        bound_source: SourceId::Query,
        limit: "max_state_bytes",
        precheck_or_charge: "work.ensure_state_capacity(structural_bytes)?",
        checked_conversion_or_arithmetic: ".checked_mul(std::mem::size_of::<Participation>())",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc011_each_query_limit_admits_exact_and_refuses_one_over",
    },
    AuditRow {
        name: "query.filter-count-exact-sum-folds",
        production: SourceId::Query,
        entrypoint: "fn aggregate(",
        expansion: "for member in joined",
        retained: "AggregateValue::Sum(sum)",
        bound_source: SourceId::Query,
        limit: "max_arithmetic_steps",
        precheck_or_charge: "work.arithmetic_step()?",
        checked_conversion_or_arithmetic: "sum.checked_add(operand)",
        evidence_source: SourceId::AuthorityEvidence,
        evidence_test: "tc011_each_query_limit_admits_exact_and_refuses_one_over",
    },
];

fn source(id: SourceId) -> &'static Source {
    SOURCES
        .iter()
        .find(|source| source.id == id)
        .expect("catalogued source")
}

fn function_scope<'a>(text: &'a str, anchor: &str) -> &'a str {
    let start = text
        .find(anchor)
        .unwrap_or_else(|| panic!("missing function anchor {anchor:?}"));
    let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
    let indent = text[line_start..start]
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let mut offset = text[start..]
        .find('\n')
        .map_or(text.len(), |index| start + index + 1);
    while offset < text.len() {
        let end = text[offset..]
            .find('\n')
            .map_or(text.len(), |index| offset + index);
        let line = &text[offset..end];
        let line_indent = line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        let trimmed = line.trim_start();
        if line_indent == indent
            && (trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn "))
        {
            return &text[start..offset];
        }
        offset = end.saturating_add(1);
    }
    &text[start..]
}

fn assert_evidence(row: &AuditRow) {
    let evidence_kind = if matches!(
        row.name,
        "common.escaped-collection-key-classification"
            | "partial.qualified-history-lookup"
            | "activation.strict-proof-copy"
            | "activation.trigger-absence-scan"
    ) {
        EvidenceKind::StructurallyDominated
    } else {
        EvidenceKind::ExactOneOver
    };
    if evidence_kind == EvidenceKind::StructurallyDominated {
        assert!(row.evidence_test.is_empty());
        return;
    }
    let evidence = source(row.evidence_source);
    let function = format!("fn {}", row.evidence_test);
    let function_at = evidence
        .text
        .find(&function)
        .unwrap_or_else(|| panic!("{} does not contain {}", evidence.path, function));
    let trace_at = evidence.text[..function_at]
        .rfind("#[trace(")
        .unwrap_or_else(|| panic!("{} has no trace before {}", evidence.path, function));
    let trace = &evidence.text[trace_at..function_at];
    assert!(
        trace.contains("TC-012"),
        "{} lacks TC-012 on {}",
        evidence.path,
        function
    );
    assert!(
        trace.contains("NFR-003-AC-2"),
        "{} lacks NFR-003-AC-2 on {}",
        evidence.path,
        function
    );
    let scope = function_scope(evidence.text, &function);
    let named_limit = row
        .limit
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .find(|word| word.starts_with("max_"));
    if let Some(named_limit) = named_limit {
        assert!(
            scope.contains(named_limit),
            "{}: {} does not exercise named limit {named_limit}",
            row.name,
            row.evidence_test
        );
    }
    assert!(
        scope.contains("exact"),
        "{}: {} lacks exact-bound evidence",
        row.name,
        row.evidence_test
    );
    assert!(
        scope.contains("one-over"),
        "{}: {} lacks one-over refusal evidence",
        row.name,
        row.evidence_test
    );
}

#[trace("TC-013", "NFR-003-AC-1")]
#[test]
fn tc013_each_c00_expansion_path_names_its_dominating_bound_and_evidence() {
    for source in SOURCES {
        assert_eq!(
            format!("{:x}", Sha256::digest(source.text.as_bytes())),
            source.sha256,
            "{} changed; re-audit its rows",
            source.path
        );
    }
    assert_eq!(
        ROWS.len(),
        43,
        "every inventoried expansion path must remain explicit"
    );
    for row in ROWS {
        let production = source(row.production);
        let bounds = source(row.bound_source);
        let scope = function_scope(production.text, row.entrypoint);
        for (kind, anchor) in [
            ("entrypoint", row.entrypoint),
            ("expansion", row.expansion),
            ("retained collection", row.retained),
            (
                "checked conversion/arithmetic",
                row.checked_conversion_or_arithmetic,
            ),
        ] {
            assert!(
                scope.contains(anchor),
                "{}: missing {kind} anchor {anchor:?} in scoped function in {}",
                row.name,
                production.path
            );
        }
        for (kind, anchor) in [
            ("finite limit", row.limit),
            ("precheck/charge", row.precheck_or_charge),
        ] {
            assert!(
                bounds.text.contains(anchor),
                "{}: missing {kind} anchor {anchor:?} in {}",
                row.name,
                bounds.path
            );
        }
        assert_evidence(row);
    }
}

#[trace("TC-013", "NFR-003-AC-1")]
#[test]
fn tc013_open_form_loops_are_consuming_or_metered_without_factorial_materialization() {
    let common = source(SourceId::Common).text;
    assert_eq!(
        common.matches("loop {").count(),
        2,
        "the two JSON object/array loops are the only shared open-form loops"
    );
    assert!(common.contains("fn object(&mut self"));
    assert!(common.contains("fn array(&mut self"));
    assert!(common.contains("self.charge_visit()?"));

    let repair = source(SourceId::Repair).text;
    assert_eq!(repair.matches("fn affected_closure<'a>").count(), 1);
    assert_eq!(
        repair.matches("affected_closure").count(),
        2,
        "one definition and one bounded call"
    );
    assert!(repair.contains("while let Some((node, changed_fact)) = queue.pop_front()"));
    assert!(repair.contains("let mut visited = BTreeSet::new()"));

    let coordination = source(SourceId::Coordination).text;
    assert!(coordination.contains("for right in 0..inputs.len()"));
    assert!(coordination.contains("for left in 0..right"));
    assert!(!coordination.contains("permutations("));
    assert!(!coordination.contains("factorial"));

    let bundle = source(SourceId::Bundle).text;
    let revision_read = function_scope(bundle, "fn read_revision_for(");
    let byte_preflight = revision_read
        .find("preflight_for_contract(bytes, limits.effective(), contract)?")
        .expect("historical read has a byte preflight");
    let lineage_build = revision_read
        .find("LineageView::from_views")
        .expect("historical read constructs predecessor lineage");
    assert!(
        byte_preflight < lineage_build,
        "historical byte preflight must dominate lineage construction"
    );

    for entrypoint in ["pub fn push(&mut self", "pub fn close(&mut self"] {
        let scope = function_scope(coordination, entrypoint);
        let preflight = scope
            .find("ensure_state_capacity(outcome_state_bytes(terminal")
            .expect("terminal clone has an exact state preflight");
        let clone = scope
            .find("terminal.clone()")
            .expect("terminal outcome clone remains explicit");
        assert!(
            preflight < clone,
            "terminal clone preflight must dominate allocation"
        );
    }
    let convert = function_scope(coordination, "fn convert_outcome(");
    let retained_preflight = convert
        .find("work.ensure_state_capacity(retained_bytes)")
        .expect("replacement retained bytes are guarded before construction");
    let replacement_build = convert
        .find("let replacement = build_replacement")
        .expect("replacement construction remains explicit");
    assert!(
        retained_preflight < replacement_build,
        "replacement state preflight must dominate wire/result allocation"
    );
}
