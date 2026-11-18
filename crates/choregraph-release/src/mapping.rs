// Linkage-only Valence projection of an admitted bundle.
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::linkage::Bundle;
use crate::profile::Role;
use crate::rejection::{Failure, IssueCode};
pub const VALENCE_PROJECTION_SCHEMA: &str = "choregraph.valence-release-projection.v1";
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValenceProjectionEntry {
    pub role: String,
    pub artifact_blake3: String,
    pub verification_role: String,
    pub claim: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValenceProjection {
    pub schema: String,
    pub bundle_blake3: String,
    pub entries: Vec<ValenceProjectionEntry>,
    pub non_claims: Vec<String>,
}

const fn projection_role(role: Role) -> &'static str {
    match role {
        Role::SingleStepProofResult | Role::TraceProofResult => "presented-proof-result",
        Role::MachineSchema
        | Role::ConcreteMachineSchema
        | Role::ConcreteIntentSchema
        | Role::LinkReceiptSchema
        | Role::KamaciteMachine
        | Role::KamaciteConcreteMachine
        | Role::KamaciteConcreteIntent
        | Role::KamaciteConcreteIntentReceipt
        | Role::KamaciteLifecycle
        | Role::KamaciteTraceReceipt
        | Role::KamaciteExhaustivenessReceipt
        | Role::KamaciteSourceReceipt
        | Role::KamaciteProofReferenceReceipt => "canonical-artifact",
        Role::ExecutionGraphParity | Role::LatticeDownstream | Role::LatticeConcreteHandoff => {
            "conformance-observation"
        }
        Role::CairnReceipt | Role::OctetReceipt | Role::NixReceipt => "gate-observation",
        _ => "identity-observation",
    }
}

pub fn project_valence(bundle: &Bundle) -> ValenceProjection {
    let entries = bundle
        .artifacts
        .iter()
        .map(|artifact| ValenceProjectionEntry {
            role: artifact.role.label().to_string(),
            artifact_blake3: artifact.digest.hex(),
            verification_role: projection_role(artifact.role).to_string(),
            claim: String::from("linkage-only-no-claim-promotion"),
        })
        .collect();
    ValenceProjection {
        schema: String::from(VALENCE_PROJECTION_SCHEMA),
        bundle_blake3: bundle.identity.hex(),
        entries,
        non_claims: bundle.non_claims.clone(),
    }
}

pub fn verify_valence_projection(
    bundle: &Bundle,
    projection: &ValenceProjection,
) -> Result<(), Failure> {
    if projection != &project_valence(bundle) {
        return Err(Failure::one(IssueCode::ProjectionMismatch, None));
    }
    Ok(())
}
