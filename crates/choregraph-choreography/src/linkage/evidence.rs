use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "ProjectionEvidenceRole", rename_all = "kebab-case")]
pub enum ProjectionRole {
    Source,
    GlobalProtocol,
    LocalProtocol,
    CompleteLocalSet,
    ProofReference,
    ProjectionReceipt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "ProjectionEvidenceBinding", deny_unknown_fields)]
pub struct ProjectionBinding {
    pub role: ProjectionRole,
    pub identity: super::digest::Identity,
    pub claim: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionReceiptBody {
    pub source_envelope: super::digest::Identity,
    pub import_closure: super::digest::Identity,
    pub evaluator_output: super::digest::Identity,
    pub strict_dto: super::digest::Identity,
    pub global_artifact: super::digest::Identity,
    pub local_artifacts: Vec<super::digest::Identity>,
    pub complete_local_set: super::digest::Identity,
    pub trellis_revision: String,
    pub trellis_semantic_commit: String,
    pub trellis_source_set: super::digest::Identity,
    pub trellis_proof_set: super::digest::Identity,
    pub lowerer: super::digest::Identity,
    pub schema_set: super::digest::Identity,
    pub bounds: super::digest::Identity,
    pub diagnostics: super::digest::Identity,
    pub toolchain: super::digest::Identity,
    pub non_claims: super::digest::Identity,
    pub evidence_roles: Vec<ProjectionBinding>,
}
