use crate::codec::CanonicalArtifact;
use crate::identity::{ArtifactIdentity, KamaciteIdentity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkReceiptRole {
    Admission,
    Lifecycle,
    Source,
    ProofReference,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-link-receipt-v1", deny_unknown_fields)]
pub struct LinkReceiptBody {
    pub schema_version: u32,
    pub role: LinkReceiptRole,
    pub subject: ArtifactIdentity,
    pub dependencies: Vec<ArtifactIdentity>,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
    pub semantic_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkReceiptArtifact {
    pub body: LinkReceiptBody,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmGuardDecision {
    pub guard_id: String,
    pub allowed: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmTraceStep {
    pub from_state: String,
    pub event: String,
    pub to_state: String,
    pub transition_identity: Option<KamaciteIdentity>,
    pub guard_decisions: Vec<FsmGuardDecision>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "kamacite-fsm-receipt-v3", deny_unknown_fields)]
pub struct FsmReceiptBody {
    pub schema_version: u32,
    pub receipt_kind: String,
    pub spec_identity: KamaciteIdentity,
    pub state_universe_identity: KamaciteIdentity,
    pub event_universe_identity: KamaciteIdentity,
    pub transition_coverage_identity: KamaciteIdentity,
    pub denial_coverage_identity: KamaciteIdentity,
    pub adapter_identity: KamaciteIdentity,
    pub dependency_identities: Vec<KamaciteIdentity>,
    pub octet_receipt_identities: Vec<KamaciteIdentity>,
    pub valence_export_identity: Option<KamaciteIdentity>,
    pub checked_laws: Vec<String>,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
    pub semantic_claims: Vec<String>,
    pub trace_start_state: Option<String>,
    pub trace_events: Vec<String>,
    pub trace_steps: Vec<FsmTraceStep>,
    pub trace_final_state: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FsmReceiptArtifact {
    pub body: FsmReceiptBody,
    pub canonical: CanonicalArtifact,
    pub identity: KamaciteIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceObservation {
    pub from_state: String,
    pub event: String,
    pub to_state: String,
    pub guard_evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptBundle {
    pub admission: LinkReceiptArtifact,
    pub lifecycle: LinkReceiptArtifact,
    pub source: LinkReceiptArtifact,
    pub proof_reference: LinkReceiptArtifact,
    pub exhaustiveness: FsmReceiptArtifact,
    pub trace: FsmReceiptArtifact,
}
