//! Bounded deterministic replay and incremental assessment for OB02.

use crate::{Digest, Identity};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayLimits {
    pub max_events: usize,
    pub max_active_keys: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressAssertion {
    pub identity: Identity,
    pub definition_identity: Identity,
    pub definition_digest: Digest,
    pub authority_identity: Identity,
    pub scope_identity: Identity,
    pub source_set_identity: Identity,
    pub restoration_identity: Identity,
    /// Timestamp through which the authority asserts coverage, exclusively.
    pub covered_through_nanos: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimedObservation {
    pub identity: Identity,
    pub subject_identity: Identity,
    pub signal_identity: Identity,
    pub event_time_nanos: i128,
    pub ingestion_time_nanos: i128,
    /// Caller-declared semantic order. Timestamps do not establish causality.
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRule {
    pub witness_signal: Identity,
    pub counterexample_signal: Identity,
}

/// The activation selection is explicit; absence of an activation is not a
/// failed assessment, and a boundary ambiguity is not an inferred timestamp.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TriggerState {
    Activated {
        identity: Identity,
        event_time_nanos: i128,
    },
    Untriggered,
    AmbiguousBoundary {
        identity: Identity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayRequest {
    pub assessment_identity: Identity,
    pub scope_identity: Identity,
    pub scope_start_nanos: i128,
    pub deadline_nanos: i128,
    pub late_cutoff_nanos: i128,
    /// Earlier immutable result that a late contradiction may supersede.
    pub prior_result_identity: Option<Identity>,
    pub trigger: TriggerState,
    pub required_history: bool,
    pub history_available: bool,
    pub membership_ambiguous: bool,
    pub profile_supported: bool,
    pub progress: Option<ProgressAssertion>,
    pub rule: DecisionRule,
    pub limits: ReplayLimits,
    pub observations: Vec<TimedObservation>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    Satisfied,
    Violated,
    MissedDeadline,
    Untriggered,
    AmbiguousBoundary,
    Open,
    IncompleteHistory,
    AmbiguousMembership,
    Unsupported,
    Exhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettlementBasis {
    DecisiveWitness,
    DecisiveCounterexample,
    EligibleDeadline,
    NotSettled,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayResult {
    pub assessment_identity: Identity,
    pub disposition: Disposition,
    pub basis: SettlementBasis,
    pub decision_support: Vec<Identity>,
    pub progress_identity: Option<Identity>,
    pub late_records: Vec<Identity>,
    pub supersedes: Option<Identity>,
    pub retained_events: usize,
}

/// Stateful bounded intake for callers receiving observations incrementally.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncrementalAssessment {
    request: ReplayRequest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IncrementalError {
    RetentionExhausted,
    ActiveKeyExhausted,
}

impl IncrementalAssessment {
    #[must_use]
    pub fn new(mut request: ReplayRequest) -> Self {
        request.observations.clear();
        Self { request }
    }

    /// Retains a record only when both declared resource limits remain valid.
    pub fn push(&mut self, observation: TimedObservation) -> Result<(), IncrementalError> {
        if self.request.observations.len() >= self.request.limits.max_events {
            return Err(IncrementalError::RetentionExhausted);
        }
        let mut subjects: BTreeSet<_> = self
            .request
            .observations
            .iter()
            .map(|record| record.subject_identity.clone())
            .collect();
        subjects.insert(observation.subject_identity.clone());
        if subjects.len() > self.request.limits.max_active_keys {
            return Err(IncrementalError::ActiveKeyExhausted);
        }
        self.request.observations.push(observation);
        Ok(())
    }

    #[must_use]
    pub fn finish(&self) -> ReplayResult {
        replay(&self.request)
    }
}

/// Evaluates one caller-selected rule over a finite, explicitly ordered history.
#[must_use]
pub fn replay(request: &ReplayRequest) -> ReplayResult {
    match &request.trigger {
        TriggerState::Untriggered => return unavailable(request, Disposition::Untriggered),
        TriggerState::AmbiguousBoundary { .. } => {
            return unavailable(request, Disposition::AmbiguousBoundary);
        }
        TriggerState::Activated { .. } => {}
    }
    if !request.profile_supported {
        return unavailable(request, Disposition::Unsupported);
    }
    if request.required_history && !request.history_available {
        return unavailable(request, Disposition::IncompleteHistory);
    }
    if request.membership_ambiguous {
        return unavailable(request, Disposition::AmbiguousMembership);
    }
    let active_keys: BTreeSet<_> = request
        .observations
        .iter()
        .map(|record| record.subject_identity.clone())
        .collect();
    if request.observations.len() > request.limits.max_events
        || active_keys.len() > request.limits.max_active_keys
    {
        return unavailable(request, Disposition::Exhausted);
    }

    let mut ordered = request.observations.clone();
    ordered.sort_by_key(|record| record.sequence);
    if ordered
        .windows(2)
        .any(|pair| pair[0].sequence == pair[1].sequence)
    {
        return unavailable(request, Disposition::Unsupported);
    }

    let mut late_records = Vec::new();
    for record in &ordered {
        if record.ingestion_time_nanos > request.late_cutoff_nanos
            && record.event_time_nanos <= request.deadline_nanos
        {
            late_records.push(record.identity.clone());
            continue;
        }
        if record.event_time_nanos < request.scope_start_nanos
            || record.event_time_nanos >= request.deadline_nanos
        {
            continue;
        }
        if record.signal_identity == request.rule.witness_signal {
            return ReplayResult {
                assessment_identity: request.assessment_identity.clone(),
                disposition: Disposition::Satisfied,
                basis: SettlementBasis::DecisiveWitness,
                decision_support: vec![record.identity.clone()],
                progress_identity: request
                    .progress
                    .as_ref()
                    .map(|value| value.identity.clone()),
                supersedes: request
                    .prior_result_identity
                    .clone()
                    .filter(|_| !late_records.is_empty()),
                late_records,
                retained_events: ordered.len(),
            };
        }
        if record.signal_identity == request.rule.counterexample_signal {
            return ReplayResult {
                assessment_identity: request.assessment_identity.clone(),
                disposition: Disposition::Violated,
                basis: SettlementBasis::DecisiveCounterexample,
                decision_support: vec![record.identity.clone()],
                progress_identity: request
                    .progress
                    .as_ref()
                    .map(|value| value.identity.clone()),
                supersedes: request
                    .prior_result_identity
                    .clone()
                    .filter(|_| !late_records.is_empty()),
                late_records,
                retained_events: ordered.len(),
            };
        }
    }

    if let Some(progress) = &request.progress {
        if progress.scope_identity == request.scope_identity
            && progress.covered_through_nanos > request.deadline_nanos
        {
            return ReplayResult {
                assessment_identity: request.assessment_identity.clone(),
                disposition: Disposition::MissedDeadline,
                basis: SettlementBasis::EligibleDeadline,
                decision_support: Vec::new(),
                progress_identity: Some(progress.identity.clone()),
                supersedes: request
                    .prior_result_identity
                    .clone()
                    .filter(|_| !late_records.is_empty()),
                late_records,
                retained_events: ordered.len(),
            };
        }
    }

    ReplayResult {
        assessment_identity: request.assessment_identity.clone(),
        disposition: Disposition::Open,
        basis: SettlementBasis::NotSettled,
        decision_support: Vec::new(),
        progress_identity: request
            .progress
            .as_ref()
            .map(|value| value.identity.clone()),
        supersedes: request
            .prior_result_identity
            .clone()
            .filter(|_| !late_records.is_empty()),
        late_records,
        retained_events: ordered.len(),
    }
}

/// Incremental execution retains no different semantics from batch replay.
#[must_use]
pub fn incremental(request: &ReplayRequest) -> ReplayResult {
    let mut state = IncrementalAssessment::new(request.clone());
    for observation in &request.observations {
        if state.push(observation.clone()).is_err() {
            return unavailable(request, Disposition::Exhausted);
        }
    }
    state.finish()
}

fn unavailable(request: &ReplayRequest, disposition: Disposition) -> ReplayResult {
    ReplayResult {
        assessment_identity: request.assessment_identity.clone(),
        disposition,
        basis: SettlementBasis::Unavailable,
        decision_support: Vec::new(),
        progress_identity: request
            .progress
            .as_ref()
            .map(|value| value.identity.clone()),
        late_records: Vec::new(),
        supersedes: None,
        retained_events: request.observations.len().min(request.limits.max_events),
    }
}
