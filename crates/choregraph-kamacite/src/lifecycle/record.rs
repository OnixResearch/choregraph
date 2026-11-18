use crate::identity::KamaciteIdentity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmState {
    pub state_id: String,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmEvent {
    pub event_id: String,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmGuard {
    pub guard_id: String,
    pub required_evidence_refs: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmSideEffect {
    pub effect_id: String,
    pub class: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FsmTransitionRow {
    pub from_state: String,
    pub event: String,
    pub to_state: String,
    pub guards: Vec<String>,
    pub side_effect: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FsmDenialRow {
    pub from_state: String,
    pub event: String,
    pub reason: String,
    pub guards: Vec<String>,
    pub side_effect: String,
    pub preserves_state: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "kamacite-fsm-spec-v2")]
pub struct LifecycleDescriptor {
    pub schema_version: u32,
    pub fsm_id: String,
    pub source_envelope_identity: Option<KamaciteIdentity>,
    pub states: Vec<FsmState>,
    pub events: Vec<FsmEvent>,
    pub initial_state: String,
    pub terminal_states: Vec<String>,
    pub intentionally_unreachable_states: Vec<String>,
    pub determinism_policy: String,
    pub totality_policy: String,
    pub guards: Vec<FsmGuard>,
    pub side_effects: Vec<FsmSideEffect>,
    pub transitions: Vec<FsmTransitionRow>,
    pub denials: Vec<FsmDenialRow>,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FsmMatrixCell {
    pub state: String,
    pub event: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FsmCoverageEvidence {
    pub state_universe_identity: KamaciteIdentity,
    pub event_universe_identity: KamaciteIdentity,
    pub transition_coverage_identity: KamaciteIdentity,
    pub denial_coverage_identity: KamaciteIdentity,
    pub exclusion_coverage_identity: KamaciteIdentity,
    pub missing_cells_identity: KamaciteIdentity,
    pub missing_cells: Vec<FsmMatrixCell>,
}
