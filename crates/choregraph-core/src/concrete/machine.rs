use crate::concrete::effect::ConcreteEffectDeclarationSpec;
use crate::model::graph::{EdgeSpec, NodeSpec};
use crate::model::pause::SuspensionDeclarationSpec;
use crate::model::semantics::{LINEAR_LIFECYCLE_V1, MACHINE_SCHEMA_V2};
use alloc::string::String;
use alloc::vec::Vec;

/// One untrusted v2 machine specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineSpecV2 {
    pub schema: String,
    pub lifecycle_profile: String,
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<EdgeSpec>,
    pub effects: Vec<ConcreteEffectDeclarationSpec>,
    pub suspensions: Vec<SuspensionDeclarationSpec>,
}

impl MachineSpecV2 {
    #[must_use]
    pub fn new(nodes: Vec<NodeSpec>, edges: Vec<EdgeSpec>) -> Self {
        Self {
            schema: String::from(MACHINE_SCHEMA_V2),
            lifecycle_profile: String::from(LINEAR_LIFECYCLE_V1),
            nodes,
            edges,
            effects: Vec::new(),
            suspensions: Vec::new(),
        }
    }
}
