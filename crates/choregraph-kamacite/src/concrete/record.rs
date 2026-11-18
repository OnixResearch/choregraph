use crate::aggregate::record::{DescriptorEdge, DescriptorLimits, DescriptorNode};
use crate::codec::CanonicalArtifact;
use crate::identity::ArtifactIdentity;
use choregraph_core::{concrete::effect::ConcreteEffectIntent, model::automaton::AdmittedMachine};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptorReferenceBindingV2 {
    pub mode: String,
    pub profile: Option<String>,
    pub identity: Option<String>,
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorConcreteEffectV2 {
    pub declaration_key: String,
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request_binding: DescriptorReferenceBindingV2,
    pub payload_binding: DescriptorReferenceBindingV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-machine-v2", deny_unknown_fields)]
pub struct ConcreteMachineDescriptorV2 {
    pub schema_version: u32,
    pub machine_schema: String,
    pub lifecycle_profile_id: String,
    pub limits: DescriptorLimits,
    pub nodes: Vec<DescriptorNode>,
    pub edges: Vec<DescriptorEdge>,
    pub effects: Vec<DescriptorConcreteEffectV2>,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorSelectedReferenceV2 {
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
    pub profile: String,
    pub identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-concrete-intent-v2", deny_unknown_fields)]
pub struct ConcreteIntentDescriptorV2 {
    pub schema_version: u32,
    pub declaration_key: String,
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: DescriptorSelectedReferenceV2,
    pub payload: DescriptorSelectedReferenceV2,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-concrete-intent-receipt-v1", deny_unknown_fields)]
pub struct ConcreteIntentReceiptBody {
    pub schema_version: u32,
    pub machine_identity: ArtifactIdentity,
    pub intent_identity: ArtifactIdentity,
    pub declaration_key: String,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
    pub semantic_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteMachineArtifact {
    pub machine: AdmittedMachine,
    pub descriptor: ConcreteMachineDescriptorV2,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteIntentArtifact {
    pub intent: ConcreteEffectIntent,
    pub descriptor: ConcreteIntentDescriptorV2,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteIntentReceiptArtifact {
    pub body: ConcreteIntentReceiptBody,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}
