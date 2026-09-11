//! Typed result handoff for OB03.

use crate::replay::{Disposition, ReplayResult, SettlementBasis};
use crate::{Digest, Identity};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmutableDependency {
    pub identity: Identity,
    pub revision: Identity,
    pub digest: Digest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Activation {
    Activated,
    Untriggered,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Participation {
    Complete,
    MissingRequiredObservation,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Completeness {
    Complete,
    Incomplete,
    Refused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingState {
    Preserved,
    Conditional,
    Unrepresented,
    Refused,
}

/// One independently scoped progress or closure fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeFact {
    pub scope_identity: Identity,
    pub authority_identity: Identity,
    pub boundary_identity: Identity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalClosureState {
    Closed,
    Open,
    Incomplete,
    Contradicted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalConformanceClosure {
    NotRequired,
    Required {
        state: GlobalClosureState,
        execution_identity: Identity,
        branch_identity: Identity,
        workflow_identity: Identity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandoffRefusal {
    InvalidIdentity(&'static str),
    DuplicateDependency,
    ScopeCrossWiring,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentHandoff {
    pub result_identity: Identity,
    pub replay: ReplayResult,
    pub activation: Activation,
    pub participation: Participation,
    pub completeness: Completeness,
    pub decision_progress: ScopeFact,
    pub decision_closure: ScopeFact,
    pub surrounding_progress: ScopeFact,
    pub surrounding_closure: ScopeFact,
    pub global_closure: GlobalConformanceClosure,
    pub source_identity: Identity,
    pub binding_identity: Identity,
    pub dependencies: Vec<ImmutableDependency>,
    pub supersedes: Option<Identity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerCapabilities {
    pub preserves_activation: bool,
    pub preserves_participation: bool,
    pub preserves_completeness: bool,
    pub preserves_dependencies: bool,
    pub preserves_late_supersession: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumerHandoff {
    pub result: AssessmentHandoff,
    pub mapping: MappingState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandoffOutcome {
    Delivered(Box<ConsumerHandoff>),
    Refused(HandoffRefusal),
}

/// Maps an assessment only if every required fact remains independently readable.
#[must_use]
pub fn handoff(result: AssessmentHandoff, capabilities: ConsumerCapabilities) -> HandoffOutcome {
    if let Some(cause) = validate(&result) {
        return HandoffOutcome::Refused(cause);
    }
    let requires_late_link = !result.replay.late_records.is_empty()
        || result.replay.supersedes.is_some()
        || result.supersedes.is_some();
    let complete = capabilities.preserves_activation
        && capabilities.preserves_participation
        && capabilities.preserves_completeness
        && capabilities.preserves_dependencies
        && (!requires_late_link || capabilities.preserves_late_supersession);
    let mapping = if complete {
        MappingState::Preserved
    } else if matches!(
        result.replay.disposition,
        Disposition::Satisfied | Disposition::Violated
    ) && result.replay.basis != SettlementBasis::Unavailable
    {
        MappingState::Conditional
    } else {
        MappingState::Unrepresented
    };
    HandoffOutcome::Delivered(Box::new(ConsumerHandoff { result, mapping }))
}

fn validate(result: &AssessmentHandoff) -> Option<HandoffRefusal> {
    if !result.result_identity.valid()
        || !result.source_identity.valid()
        || !result.binding_identity.valid()
        || !valid_scope(&result.decision_progress)
        || !valid_scope(&result.decision_closure)
        || !valid_scope(&result.surrounding_progress)
        || !valid_scope(&result.surrounding_closure)
    {
        return Some(HandoffRefusal::InvalidIdentity("result axis"));
    }
    if result.decision_progress.scope_identity != result.decision_closure.scope_identity
        || result.surrounding_progress.scope_identity != result.surrounding_closure.scope_identity
    {
        return Some(HandoffRefusal::ScopeCrossWiring);
    }
    let mut dependencies = BTreeSet::new();
    for dependency in &result.dependencies {
        if !dependency.identity.valid() || !dependency.revision.valid() {
            return Some(HandoffRefusal::InvalidIdentity("dependency"));
        }
        if !dependencies.insert((dependency.identity.clone(), dependency.revision.clone())) {
            return Some(HandoffRefusal::DuplicateDependency);
        }
    }
    if let GlobalConformanceClosure::Required {
        execution_identity,
        branch_identity,
        workflow_identity,
        ..
    } = &result.global_closure
    {
        if !execution_identity.valid() || !branch_identity.valid() || !workflow_identity.valid() {
            return Some(HandoffRefusal::InvalidIdentity("global closure"));
        }
    }
    None
}

fn valid_scope(scope: &ScopeFact) -> bool {
    scope.scope_identity.valid()
        && scope.authority_identity.valid()
        && scope.boundary_identity.valid()
}
