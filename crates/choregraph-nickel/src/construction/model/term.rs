//! Construction expressions, references and effect bindings.

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "tag", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Expression {
    Literal {
        value: serde_json::Value,
    },
    Parameter {
        name: String,
    },
    Reference {
        reference: LocalReference,
    },
    Apply {
        operator: String,
        arguments: Vec<Expression>,
    },
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceDirection {
    Input,
    Output,
    Config,
    Invocation,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalReference {
    pub direction: ReferenceDirection,
    pub name: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExactReference {
    pub profile: String,
    pub identity: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Binding {
    Exact {
        reference: ExactReference,
    },
    Slot {
        slot: String,
        source: LocalReference,
    },
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub key: String,
    pub from_state: String,
    pub action: String,
    pub resource: String,
    pub ability: String,
    pub request: Binding,
    pub payload: Binding,
}
