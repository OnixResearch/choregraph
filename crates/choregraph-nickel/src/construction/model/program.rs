//! Finite captured construction programs and source attribution.

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub schema: String,
    pub root: String,
    pub target: String,
    pub registry: String,
    pub parameters: std::collections::BTreeMap<String, serde_json::Value>,
    pub templates: Vec<Template>,
    pub body: Fragment,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Fragment,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fragment {
    pub key: String,
    #[serde(default = "no_attribution")]
    pub origin: Option<Attribution>,
    pub kind: FragmentKind,
}
const fn no_attribution() -> Option<Attribution> {
    None
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "tag", rename_all = "kebab-case", deny_unknown_fields)]
pub enum FragmentKind {
    Operation {
        operation: String,
        configuration: std::collections::BTreeMap<String, super::term::Expression>,
        effects: Vec<super::term::Effect>,
    },
    Instantiate {
        template: String,
        arguments: std::collections::BTreeMap<String, super::term::Expression>,
    },
    Identity {
        boundaries: Vec<super::interface::Boundary>,
    },
    Sequence {
        parts: Vec<Fragment>,
    },
    Parallel {
        parts: Vec<Fragment>,
    },
    Wire {
        parts: Vec<Fragment>,
        connections: Vec<super::interface::Connection>,
        inputs: Vec<super::interface::Exposure>,
        outputs: Vec<super::interface::Exposure>,
    },
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Attribution {
    pub template: String,
    pub generation: Option<String>,
    pub location: Option<SourceLocation>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLocation {
    pub logical_module: String,
    pub start: usize,
    pub end: usize,
}
