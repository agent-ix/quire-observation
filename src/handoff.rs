//! Typed result handoff for OB03.

use crate::replay::{Disposition, ReplayResult, SettlementBasis};
use crate::{Digest, Identity};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssessmentHandoff {
    pub result_identity: Identity,
    pub replay: ReplayResult,
    pub activation: Activation,
    pub participation: Participation,
    pub completeness: Completeness,
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

/// Maps an assessment only if every required fact remains independently readable.
#[must_use]
pub fn handoff(result: AssessmentHandoff, capabilities: ConsumerCapabilities) -> ConsumerHandoff {
    let requires_late_link = !result.replay.late_records.is_empty() || result.supersedes.is_some();
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
    ConsumerHandoff { result, mapping }
}
