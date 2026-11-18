//! Host-owned registry and operation declarations.

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetRegistry {
    pub schema: String,
    pub target: String,
    pub profile: String,
    pub implementation: String,
    pub types: Vec<ValueType>,
    pub operations: Vec<OperationSignature>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValueType {
    pub name: String,
    pub copyable: bool,
    pub droppable: bool,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationSignature {
    pub operation: String,
    pub implementation: String,
    pub inputs: Vec<super::interface::Boundary>,
    pub outputs: Vec<super::interface::Boundary>,
    pub invocation: Vec<super::interface::Boundary>,
    pub configuration: Vec<ConfigurationField>,
    pub effects: Vec<EffectRule>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationField {
    pub name: String,
    pub value_type: String,
    pub references: bool,
    pub operators: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRule {
    pub from_state: String,
    pub action: String,
    pub resource: String,
    pub ability: String,
}
