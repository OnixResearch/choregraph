// Evaluated-data DTOs keep explicit serde value imports for schema review.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "evaluated-data DTOs keep explicit serde value imports for schema review"
)]

// r[impl choregraph.nickel.contracts]
// r[impl choregraph.nickel.lattice_profile]
use serde::{Deserialize, Serialize};

pub const CHOREGRAPH_EVALUATED_SCHEMA: &str = "choregraph.evaluated-graph.v1";
pub const LATTICE_EVALUATED_SCHEMA: &str = "choregraph.lattice-workflow-projection.v1";
pub const RESTRICTED_EVALUATOR_PROFILE: &str = "choregraph.nickel-restricted.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportMediaClass {
    NickelData,
    Utf8Text,
    PreservesPacked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImportDeclaration {
    pub logical_id: String,
    pub root_id: String,
    pub relative_path: String,
    pub media_class: ImportMediaClass,
    pub max_bytes: u64,
    pub expected_blake3: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatedLimits {
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
#[serde(deny_unknown_fields)]
pub struct EvaluatedNode {
    pub id: String,
    pub operations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatedEdge {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluatedEffect {
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LatticePortDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LatticePort {
    pub id: u64,
    pub direction: LatticePortDirection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LatticeNode {
    pub id: u64,
    pub ports: Vec<LatticePort>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LatticePortEdge {
    pub source: u64,
    pub target: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LatticeNodeConfig {
    pub node_id: u64,
    pub operation: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "source_format", deny_unknown_fields)]
pub enum EvaluatedSource {
    #[serde(rename = "choregraph-v1")]
    Choregraph {
        schema: String,
        lifecycle_profile: String,
        events: Vec<String>,
        limits: EvaluatedLimits,
        imports: Vec<ImportDeclaration>,
        nodes: Vec<EvaluatedNode>,
        edges: Vec<EvaluatedEdge>,
        effects: Vec<EvaluatedEffect>,
    },
    #[serde(rename = "lattice-workflow-v1")]
    LatticeWorkflow {
        schema: String,
        lifecycle_profile: String,
        events: Vec<String>,
        limits: EvaluatedLimits,
        imports: Vec<ImportDeclaration>,
        nodes: Vec<LatticeNode>,
        edges: Vec<LatticePortEdge>,
        configs: Vec<LatticeNodeConfig>,
    },
}

impl EvaluatedSource {
    #[must_use]
    pub fn imports(&self) -> &[ImportDeclaration] {
        match self {
            Self::Choregraph { imports, .. } | Self::LatticeWorkflow { imports, .. } => imports,
        }
    }
}
