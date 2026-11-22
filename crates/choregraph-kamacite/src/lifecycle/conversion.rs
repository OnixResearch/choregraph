use crate::codec::state::{FSM_SPEC_ROOT_LABEL, FSM_SPEC_SCHEMA_VERSION};
use crate::codec::{CanonicalArtifact, CodecBoundaryError, encode};
use crate::identity::{
    ArtifactDomain, ArtifactIdentity, KamaciteIdentity, KamaciteIdentityDomain, compute_identity,
    compute_kamacite_identity,
};
use crate::lifecycle::record::FsmCoverageEvidence;
use crate::lifecycle::record::FsmDenialRow;
use crate::lifecycle::record::FsmEvent;
use crate::lifecycle::record::FsmGuard;
use crate::lifecycle::record::FsmMatrixCell;
use crate::lifecycle::record::FsmSideEffect;
use crate::lifecycle::record::FsmState;
use crate::lifecycle::record::FsmTransitionRow;
use crate::lifecycle::record::LifecycleDescriptor;
use choregraph_core::{
    model::semantics::Action, model::semantics::DURABLE_SUSPENSION_V1,
    model::semantics::DURABLE_SUSPENSION_V2, model::semantics::LifecycleProfile,
    model::semantics::LocalState, model::semantics::LocalTransitionError,
    transition::lifecycle_target_for_profile,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

pub const LIFECYCLE_FSM_ID: &str = "choregraph.lifecycle.linear.v1";
pub const DURABLE_FSM_ID: &str = DURABLE_SUSPENSION_V1;
pub const DURABLE_FSM_V2_ID: &str = DURABLE_SUSPENSION_V2;
const SUPPORTED_SCHEMA_VERSION: u32 = 1;
const DETERMINISM_POLICY: &str = "deterministic";
const TOTALITY_POLICY: &str = "closed-matrix-with-explicit-denials";
const DEPENDENCIES_COMPLETE_GUARD: &str = "dependencies-complete";
const MACHINE_DECLARED_INTENTS_EFFECT: &str = "machine-declared-effect-intents";
const NO_SIDE_EFFECT: &str = "none";
const FSM_COVERAGE_SCHEMA_VERSION: &str = "kamacite.fsm-coverage.v1";
const FSM_COVERAGE_ROOT_LABEL: &str = "FsmCoverageMaterial";
const FSM_STATE_UNIVERSE_KIND: &str = "state-universe";
const FSM_EVENT_UNIVERSE_KIND: &str = "event-universe";
const FSM_TRANSITION_COVERAGE_KIND: &str = "transition-coverage";
const FSM_DENIAL_COVERAGE_KIND: &str = "denial-coverage";
const FSM_EXCLUSION_COVERAGE_KIND: &str = "exclusion-coverage";
const FSM_MISSING_CELLS_KIND: &str = "missing-cells";
const FSM_TRANSITION_KIND: &str = "transition";
const LOCAL_STATE_COUNT: usize = 6;
const ACTION_COUNT: usize = 5;
const MATRIX_CELL_COUNT: usize = LOCAL_STATE_COUNT * ACTION_COUNT;
const DURABLE_STATE_COUNT: usize = 7;
const DURABLE_ACTION_COUNT: usize = 7;
const DURABLE_MATRIX_CELL_COUNT: usize = DURABLE_STATE_COUNT * DURABLE_ACTION_COUNT;

const STATES: [LocalState; LOCAL_STATE_COUNT] = [
    LocalState::Pending,
    LocalState::Ready,
    LocalState::Running,
    LocalState::Completed,
    LocalState::Failed,
    LocalState::Cancelled,
];
const ACTIONS: [Action; ACTION_COUNT] = [
    Action::MarkReady,
    Action::Start,
    Action::Complete,
    Action::Fail,
    Action::Cancel,
];
const DURABLE_STATES: [LocalState; DURABLE_STATE_COUNT] = [
    LocalState::Pending,
    LocalState::Ready,
    LocalState::Running,
    LocalState::Completed,
    LocalState::Failed,
    LocalState::Cancelled,
    LocalState::Suspended,
];
const DURABLE_ACTIONS: [Action; DURABLE_ACTION_COUNT] = [
    Action::MarkReady,
    Action::Start,
    Action::Complete,
    Action::Fail,
    Action::Cancel,
    Action::Suspend,
    Action::ResumeSignal,
];
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleArtifact {
    pub descriptor: LifecycleDescriptor,
    pub canonical: CanonicalArtifact,
    pub identity: KamaciteIdentity,
    pub project_identity: ArtifactIdentity,
    pub coverage: FsmCoverageEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleError {
    Codec(CodecBoundaryError),
    Identity(String),
    UnsupportedProfile,
    ProfileMismatch,
    MatrixNotClosed { covered: usize, expected: usize },
}

impl Display for LifecycleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if let Self::Codec(error) = self {
            return Display::fmt(error, formatter);
        }
        if let Self::Identity(message) = self {
            return formatter.write_str(message);
        }
        if let Self::MatrixNotClosed { covered, expected } = self {
            return write!(
                formatter,
                "lifecycle matrix covers {covered} cells, expected {expected}"
            );
        }
        const DIAGNOSTICS: [(LifecycleError, &str); 2] = [
            (
                LifecycleError::UnsupportedProfile,
                "unsupported Kamacite lifecycle profile",
            ),
            (
                LifecycleError::ProfileMismatch,
                "lifecycle descriptor does not match the selected closed profile",
            ),
        ];
        if let Some((_, message)) = DIAGNOSTICS.iter().find(|(category, _)| category == self) {
            formatter.write_str(message)
        } else {
            // New categories retain their exact variant and payload in diagnostics.
            std::fmt::Debug::fmt(self, formatter)
        }
    }
}

impl std::error::Error for LifecycleError {}

impl From<CodecBoundaryError> for LifecycleError {
    fn from(error: CodecBoundaryError) -> Self {
        Self::Codec(error)
    }
}

fn state_id(state: LocalState) -> &'static str {
    match state {
        LocalState::Pending => "pending",
        LocalState::Ready => "ready",
        LocalState::Running => "running",
        LocalState::Completed => "completed",
        LocalState::Failed => "failed",
        LocalState::Cancelled => "cancelled",
        LocalState::Suspended => "suspended",
    }
}

fn action_id(action: Action) -> &'static str {
    match action {
        Action::MarkReady => "mark-ready",
        Action::Start => "start",
        Action::Complete => "complete",
        Action::Fail => "fail",
        Action::Cancel => "cancel",
        Action::Suspend => "suspend",
        Action::ResumeSignal => "resume-signal",
    }
}

fn row_guards(state: LocalState, action: Action) -> Vec<String> {
    let is_dependency_gated = (state == LocalState::Pending && action == Action::MarkReady)
        || (state == LocalState::Suspended && action == Action::ResumeSignal);
    if is_dependency_gated {
        vec![String::from(DEPENDENCIES_COMPLETE_GUARD)]
    } else {
        Vec::new()
    }
}

fn row_side_effect(profile: LifecycleProfile, state: LocalState, action: Action) -> &'static str {
    if matches!(action, Action::Suspend | Action::ResumeSignal)
        || (profile == LifecycleProfile::DurableSuspensionV2
            && state == LocalState::Suspended
            && action == Action::Cancel)
    {
        NO_SIDE_EFFECT
    } else {
        MACHINE_DECLARED_INTENTS_EFFECT
    }
}

fn denial_reason(error: LocalTransitionError) -> String {
    match error {
        LocalTransitionError::Terminal { .. } => String::from("terminal-state"),
        LocalTransitionError::Invalid { .. } => String::from("invalid-transition"),
    }
}

fn descriptor_for(
    profile: LifecycleProfile,
    state_table: &[LocalState],
    action_table: &[Action],
    fsm_id: &str,
    source_envelope_identity: Option<KamaciteIdentity>,
) -> LifecycleDescriptor {
    let states = state_table
        .iter()
        .map(|state| FsmState {
            state_id: String::from(state_id(*state)),
            label: String::from(state_id(*state)),
        })
        .collect();
    let events = action_table
        .iter()
        .map(|action| FsmEvent {
            event_id: String::from(action_id(*action)),
            label: String::from(action_id(*action)),
        })
        .collect();
    let mut transitions = Vec::new();
    let mut denials = Vec::new();
    for state in state_table {
        for action in action_table {
            match lifecycle_target_for_profile(profile, *state, *action) {
                Ok(target) => transitions.push(FsmTransitionRow {
                    from_state: String::from(state_id(*state)),
                    event: String::from(action_id(*action)),
                    to_state: String::from(state_id(target)),
                    guards: row_guards(*state, *action),
                    side_effect: String::from(row_side_effect(profile, *state, *action)),
                }),
                Err(error) => denials.push(FsmDenialRow {
                    from_state: String::from(state_id(*state)),
                    event: String::from(action_id(*action)),
                    reason: denial_reason(error),
                    guards: Vec::new(),
                    side_effect: String::from(NO_SIDE_EFFECT),
                    preserves_state: true,
                }),
            }
        }
    }
    LifecycleDescriptor {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        fsm_id: String::from(fsm_id),
        source_envelope_identity,
        states,
        events,
        initial_state: String::from(state_id(LocalState::Pending)),
        terminal_states: vec![
            String::from(state_id(LocalState::Completed)),
            String::from(state_id(LocalState::Failed)),
            String::from(state_id(LocalState::Cancelled)),
        ],
        intentionally_unreachable_states: Vec::new(),
        determinism_policy: String::from(DETERMINISM_POLICY),
        totality_policy: String::from(TOTALITY_POLICY),
        guards: vec![FsmGuard {
            guard_id: String::from(DEPENDENCIES_COMPLETE_GUARD),
            required_evidence_refs: vec![String::from("admitted-machine-dependency-index")],
            non_claims: vec![String::from(
                "dependency evidence does not prove event truth",
            )],
        }],
        side_effects: vec![
            FsmSideEffect {
                effect_id: String::from(MACHINE_DECLARED_INTENTS_EFFECT),
                class: String::from("pure-zero-or-more-effect-intents"),
            },
            FsmSideEffect {
                effect_id: String::from(NO_SIDE_EFFECT),
                class: String::from("no-side-effect"),
            },
        ],
        transitions,
        denials,
        checked_laws: vec![
            String::from("closed-state-event-matrix"),
            String::from("deterministic-local-transition"),
            String::from("denial-preserves-local-state"),
            String::from("denial-emits-no-effect-intents"),
        ],
        non_claims: vec![
            String::from("does not enumerate aggregate product states"),
            String::from("does not prove transition implementation correctness"),
            String::from("does not authorize or execute effects"),
            String::from("does not prove source equivalence"),
        ],
    }
}

fn descriptor(source_envelope_identity: Option<KamaciteIdentity>) -> LifecycleDescriptor {
    descriptor_for(
        LifecycleProfile::LinearV1,
        &STATES,
        &ACTIONS,
        LIFECYCLE_FSM_ID,
        source_envelope_identity,
    )
}

pub(crate) const fn durable_fsm_id(profile: LifecycleProfile) -> Option<&'static str> {
    match profile {
        LifecycleProfile::LinearV1 => None,
        LifecycleProfile::DurableSuspensionV1 => Some(DURABLE_FSM_ID),
        LifecycleProfile::DurableSuspensionV2 => Some(DURABLE_FSM_V2_ID),
    }
}

pub(crate) fn durable_profile_from_fsm_id(id: &str) -> Option<LifecycleProfile> {
    match id {
        DURABLE_FSM_ID => Some(LifecycleProfile::DurableSuspensionV1),
        DURABLE_FSM_V2_ID => Some(LifecycleProfile::DurableSuspensionV2),
        _ => None,
    }
}

fn durable_descriptor(
    profile: LifecycleProfile,
    fsm_id: &str,
    source_envelope_identity: Option<KamaciteIdentity>,
) -> LifecycleDescriptor {
    let mut descriptor = descriptor_for(
        profile,
        &DURABLE_STATES,
        &DURABLE_ACTIONS,
        fsm_id,
        source_envelope_identity,
    );
    descriptor.checked_laws.push(String::from(
        "suspension-transitions-emit-no-effect-intents",
    ));
    descriptor.non_claims.push(String::from(
        "does not prove approval authority or signal truth",
    ));
    descriptor
}

fn build_artifact(
    descriptor: LifecycleDescriptor,
    expected_cells: usize,
) -> Result<LifecycleArtifact, LifecycleError> {
    let canonical = encode(&descriptor, FSM_SPEC_SCHEMA_VERSION, FSM_SPEC_ROOT_LABEL)?;
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::FsmSpec, &canonical.packed_bytes);
    let project_identity = compute_identity(ArtifactDomain::Lifecycle, &canonical.packed_bytes)
        .map_err(|error| LifecycleError::Identity(error.to_string()))?;
    let coverage = coverage_evidence(&descriptor)?;
    if !coverage.missing_cells.is_empty() {
        return Err(LifecycleError::MatrixNotClosed {
            covered: expected_cells.saturating_sub(coverage.missing_cells.len()),
            expected: expected_cells,
        });
    }
    Ok(LifecycleArtifact {
        descriptor,
        canonical,
        identity,
        project_identity,
        coverage,
    })
}

/// Builds the original durable-suspension v1 lifecycle descriptor.
pub fn build_durable_lifecycle_artifact(
    source_envelope_identity: Option<KamaciteIdentity>,
) -> Result<LifecycleArtifact, LifecycleError> {
    build_durable_lifecycle_artifact_for_profile(
        LifecycleProfile::DurableSuspensionV1,
        source_envelope_identity,
    )
}

/// Builds the closed lifecycle descriptor for an explicitly selected durable profile.
pub fn build_durable_lifecycle_artifact_for_profile(
    profile: LifecycleProfile,
    source_envelope_identity: Option<KamaciteIdentity>,
) -> Result<LifecycleArtifact, LifecycleError> {
    let fsm_id = durable_fsm_id(profile).ok_or(LifecycleError::UnsupportedProfile)?;
    if let Some(identity) = &source_envelope_identity {
        identity
            .validate(KamaciteIdentityDomain::Source)
            .map_err(|error| LifecycleError::Identity(error.to_string()))?;
    }
    build_artifact(
        durable_descriptor(profile, fsm_id, source_envelope_identity),
        DURABLE_MATRIX_CELL_COUNT,
    )
}

pub fn validate_durable_lifecycle_artifact(
    input: &[u8],
) -> Result<LifecycleArtifact, LifecycleError> {
    let (decoded, canonical): (LifecycleDescriptor, CanonicalArtifact) =
        crate::codec::decode(input, FSM_SPEC_SCHEMA_VERSION, FSM_SPEC_ROOT_LABEL)?;
    if decoded.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(LifecycleError::UnsupportedProfile);
    }
    let profile =
        durable_profile_from_fsm_id(&decoded.fsm_id).ok_or(LifecycleError::UnsupportedProfile)?;
    let expected = durable_descriptor(
        profile,
        &decoded.fsm_id,
        decoded.source_envelope_identity.clone(),
    );
    if decoded != expected {
        return Err(LifecycleError::ProfileMismatch);
    }
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::FsmSpec, &canonical.packed_bytes);
    let project_identity = compute_identity(ArtifactDomain::Lifecycle, &canonical.packed_bytes)
        .map_err(|error| LifecycleError::Identity(error.to_string()))?;
    let coverage = coverage_evidence(&decoded)?;
    if !coverage.missing_cells.is_empty() {
        return Err(LifecycleError::MatrixNotClosed {
            covered: DURABLE_MATRIX_CELL_COUNT.saturating_sub(coverage.missing_cells.len()),
            expected: DURABLE_MATRIX_CELL_COUNT,
        });
    }
    Ok(LifecycleArtifact {
        descriptor: decoded,
        canonical,
        identity,
        project_identity,
        coverage,
    })
}

pub fn build_lifecycle_artifact(
    source_envelope_identity: Option<KamaciteIdentity>,
) -> Result<LifecycleArtifact, LifecycleError> {
    if let Some(identity) = &source_envelope_identity {
        identity
            .validate(KamaciteIdentityDomain::Source)
            .map_err(|error| LifecycleError::Identity(error.to_string()))?;
    }
    let descriptor = descriptor(source_envelope_identity);
    let canonical = encode(&descriptor, FSM_SPEC_SCHEMA_VERSION, FSM_SPEC_ROOT_LABEL)?;
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::FsmSpec, &canonical.packed_bytes);
    let project_identity = compute_identity(ArtifactDomain::Lifecycle, &canonical.packed_bytes)
        .map_err(|error| LifecycleError::Identity(error.to_string()))?;
    let coverage = coverage_evidence(&descriptor)?;
    if !coverage.missing_cells.is_empty() {
        return Err(LifecycleError::MatrixNotClosed {
            covered: MATRIX_CELL_COUNT.saturating_sub(coverage.missing_cells.len()),
            expected: MATRIX_CELL_COUNT,
        });
    }
    Ok(LifecycleArtifact {
        descriptor,
        canonical,
        identity,
        project_identity,
        coverage,
    })
}

pub fn validate_lifecycle_artifact(input: &[u8]) -> Result<LifecycleArtifact, LifecycleError> {
    let (decoded, canonical): (LifecycleDescriptor, CanonicalArtifact) =
        crate::codec::decode(input, FSM_SPEC_SCHEMA_VERSION, FSM_SPEC_ROOT_LABEL)?;
    if decoded.fsm_id != LIFECYCLE_FSM_ID || decoded.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(LifecycleError::UnsupportedProfile);
    }
    let expected = descriptor(decoded.source_envelope_identity.clone());
    if decoded != expected {
        return Err(LifecycleError::ProfileMismatch);
    }
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::FsmSpec, &canonical.packed_bytes);
    let project_identity = compute_identity(ArtifactDomain::Lifecycle, &canonical.packed_bytes)
        .map_err(|error| LifecycleError::Identity(error.to_string()))?;
    let coverage = coverage_evidence(&decoded)?;
    if !coverage.missing_cells.is_empty() {
        return Err(LifecycleError::MatrixNotClosed {
            covered: MATRIX_CELL_COUNT.saturating_sub(coverage.missing_cells.len()),
            expected: MATRIX_CELL_COUNT,
        });
    }
    Ok(LifecycleArtifact {
        descriptor: decoded,
        canonical,
        identity,
        project_identity,
        coverage,
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct FsmCoverageMaterial<T> {
    kind: String,
    entries: Vec<T>,
}

fn coverage_identity<T: Serialize>(
    domain: KamaciteIdentityDomain,
    kind: &str,
    entries: Vec<T>,
) -> Result<KamaciteIdentity, LifecycleError> {
    let material = FsmCoverageMaterial {
        kind: String::from(kind),
        entries,
    };
    let artifact = encode(
        &material,
        FSM_COVERAGE_SCHEMA_VERSION,
        FSM_COVERAGE_ROOT_LABEL,
    )?;
    Ok(compute_kamacite_identity(domain, &artifact.packed_bytes))
}

fn coverage_evidence(input: &LifecycleDescriptor) -> Result<FsmCoverageEvidence, LifecycleError> {
    let mut states = input
        .states
        .iter()
        .map(|state| state.state_id.clone())
        .collect::<Vec<_>>();
    states.sort();
    let mut events = input
        .events
        .iter()
        .map(|event| event.event_id.clone())
        .collect::<Vec<_>>();
    events.sort();
    let mut transitions = input.transitions.clone();
    transitions.sort();
    let mut denials = input.denials.clone();
    denials.sort();
    let mut exclusions = input.intentionally_unreachable_states.clone();
    exclusions.sort();
    exclusions.dedup();

    let covered = input
        .transitions
        .iter()
        .map(|row| (row.from_state.clone(), row.event.clone()))
        .chain(
            input
                .denials
                .iter()
                .map(|row| (row.from_state.clone(), row.event.clone())),
        )
        .collect::<BTreeSet<_>>();
    let mut missing_cells = Vec::new();
    for state in &states {
        for event in &events {
            if !covered.contains(&(state.clone(), event.clone())) {
                missing_cells.push(FsmMatrixCell {
                    state: state.clone(),
                    event: event.clone(),
                });
            }
        }
    }

    Ok(FsmCoverageEvidence {
        state_universe_identity: coverage_identity(
            KamaciteIdentityDomain::FsmUniverse,
            FSM_STATE_UNIVERSE_KIND,
            states,
        )?,
        event_universe_identity: coverage_identity(
            KamaciteIdentityDomain::FsmUniverse,
            FSM_EVENT_UNIVERSE_KIND,
            events,
        )?,
        transition_coverage_identity: coverage_identity(
            KamaciteIdentityDomain::FsmCoverage,
            FSM_TRANSITION_COVERAGE_KIND,
            transitions,
        )?,
        denial_coverage_identity: coverage_identity(
            KamaciteIdentityDomain::FsmCoverage,
            FSM_DENIAL_COVERAGE_KIND,
            denials,
        )?,
        exclusion_coverage_identity: coverage_identity(
            KamaciteIdentityDomain::FsmCoverage,
            FSM_EXCLUSION_COVERAGE_KIND,
            exclusions,
        )?,
        missing_cells_identity: coverage_identity(
            KamaciteIdentityDomain::FsmCoverage,
            FSM_MISSING_CELLS_KIND,
            missing_cells.clone(),
        )?,
        missing_cells,
    })
}

pub fn transition_identity(
    transition: &FsmTransitionRow,
) -> Result<KamaciteIdentity, LifecycleError> {
    coverage_identity(
        KamaciteIdentityDomain::FsmCoverage,
        FSM_TRANSITION_KIND,
        vec![transition.clone()],
    )
}
