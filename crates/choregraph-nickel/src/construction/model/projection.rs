//! Materialized v3 projection and exact origin data.

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LatticeV3 {
    pub schema: String,
    pub target: String,
    pub registry: String,
    pub nodes: Vec<ExpandedNode>,
    pub edges: Vec<DataEdge>,
    pub control_edges: Vec<ControlEdge>,
    pub effects: Vec<ExpandedEffect>,
    pub interfaces: Vec<ExpandedInterface>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandedInterface {
    pub scope: String,
    pub inputs: Vec<ExpandedBoundary>,
    pub outputs: Vec<ExpandedBoundary>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandedBoundary {
    pub name: String,
    pub value_type: String,
    pub ports: Vec<u64>,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandedNode {
    pub id: u64,
    pub scoped_id: String,
    pub operation: String,
    pub implementation: String,
    pub ports: Vec<ExpandedPort>,
    pub invocation: Vec<super::interface::Boundary>,
    pub configuration: std::collections::BTreeMap<String, ExpandedExpression>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandedPort {
    pub id: u64,
    pub name: String,
    pub direction: super::interface::Direction,
    pub value_type: String,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "tag", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ExpandedExpression {
    Literal {
        value: serde_json::Value,
    },
    Reference {
        node: u64,
        direction: super::term::ReferenceDirection,
        name: String,
        port: Option<u64>,
    },
    Apply {
        operator: String,
        arguments: Vec<ExpandedExpression>,
    },
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DataEdge {
    pub source: u64,
    pub target: u64,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ControlEdge {
    pub source: u64,
    pub target: u64,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ExpandedBinding {
    Exact {
        reference: super::term::ExactReference,
    },
    Slot {
        slot_id: String,
        source_ref: String,
    },
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpandedEffect {
    pub declaration_key: String,
    pub node_id: u64,
    pub from_state: String,
    pub action: String,
    pub resource: String,
    pub ability: String,
    pub request_binding: ExpandedBinding,
    pub payload_binding: ExpandedBinding,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub element: String,
    pub template: String,
    pub instance_path: Vec<String>,
    pub local: String,
    pub generation: Option<String>,
    pub location: Option<super::program::SourceLocation>,
    pub supplied: bool,
}
