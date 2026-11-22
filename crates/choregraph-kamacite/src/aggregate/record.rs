use crate::identity::ArtifactIdentity;
use choregraph_core::id::bounded::{NodeId, OperationId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorLimits {
    pub max_nodes: u64,
    pub max_edges: u64,
    pub max_operations: u64,
    pub max_effect_declarations: u64,
    pub max_identifier_bytes: u64,
    pub max_reference_bytes: u64,
    pub max_event_bytes: u64,
    pub max_intent_bytes: u64,
    pub max_intents_per_transition: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorNode {
    pub node_id: String,
    pub operations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorEdge {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorDependencySet {
    pub node_id: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorEffect {
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompatibilityMetadata {
    pub execution_graph_revision: String,
    pub lattice_revision: String,
    pub kamacite_revision: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-machine-v1", deny_unknown_fields)]
pub struct AggregateMachineDescriptor {
    pub schema_version: u32,
    pub machine_schema: String,
    pub lifecycle_profile_id: String,
    pub lifecycle_profile_identity: ArtifactIdentity,
    pub kamacite_profile_identity: ArtifactIdentity,
    pub global_state_model: String,
    pub global_state_cardinality: String,
    pub limits: DescriptorLimits,
    pub nodes: Vec<DescriptorNode>,
    pub edges: Vec<DescriptorEdge>,
    pub dependencies: Vec<DescriptorDependencySet>,
    pub topological_order: Vec<String>,
    pub effects: Vec<DescriptorEffect>,
    pub compatibility: CompatibilityMetadata,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

/// One suspension declaration row in a durable machine descriptor.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DescriptorSuspension {
    pub node_id: String,
    pub class: String,
    pub reference: String,
}

/// The durable-suspension machine descriptor, reused unchanged by both durable
/// lifecycle profiles. Linear machines never gain this field set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-machine-durable-v1", deny_unknown_fields)]
pub struct DurableMachineDescriptor {
    pub schema_version: u32,
    pub machine_schema: String,
    pub lifecycle_profile_id: String,
    pub lifecycle_profile_identity: ArtifactIdentity,
    pub kamacite_profile_identity: ArtifactIdentity,
    pub global_state_model: String,
    pub global_state_cardinality: String,
    pub limits: DescriptorLimits,
    pub nodes: Vec<DescriptorNode>,
    pub edges: Vec<DescriptorEdge>,
    pub dependencies: Vec<DescriptorDependencySet>,
    pub topological_order: Vec<String>,
    pub effects: Vec<DescriptorEffect>,
    pub suspensions: Vec<DescriptorSuspension>,
    pub compatibility: CompatibilityMetadata,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

// Each wire row owns its strings; allocate once at this typed ownership boundary.
impl From<(&NodeId, &BTreeSet<OperationId>)> for DescriptorNode {
    fn from((node_id, operations): (&NodeId, &BTreeSet<OperationId>)) -> Self {
        Self {
            node_id: String::from(node_id.as_str()),
            operations: operations
                .iter()
                .map(|operation| String::from(operation.as_str()))
                .collect(),
        }
    }
}

impl From<(&NodeId, &[NodeId])> for DescriptorDependencySet {
    fn from((node_id, dependencies): (&NodeId, &[NodeId])) -> Self {
        Self {
            node_id: String::from(node_id.as_str()),
            dependencies: dependencies
                .iter()
                .map(|dependency| String::from(dependency.as_str()))
                .collect(),
        }
    }
}
