// Evaluated-data DTOs retain schema compatibility defaults.

// r[impl choregraph.nickel.contracts]
// r[impl choregraph.nickel.lattice_profile]
pub mod dependency {
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum ImportMediaClass {
        NickelData,
        Utf8Text,
        PreservesPacked,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ImportDeclaration {
        pub logical_id: String,
        pub root_id: String,
        pub relative_path: String,
        pub media_class: crate::model::dependency::ImportMediaClass,
        pub max_bytes: u64,
        pub expected_blake3: String,
    }
}

pub mod graph {
    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
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

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EvaluatedNode {
        pub id: String,
        pub operations: Vec<String>,
    }

    /// One durable-suspension declaration in an evaluated Choregraph source.
    ///
    /// The closed field set is the boundary defense: approver, policy, credential,
    /// clock, persistence, and notification fields fail deserialization.
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EvaluatedSuspension {
        pub node_id: String,
        pub class: String,
        pub reference: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EvaluatedEdge {
        pub source: String,
        pub target: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
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
}

pub mod projection {
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EvaluatedLatticeEffectV2 {
        pub declaration_key: String,
        pub node_id: u64,
        pub from_state: String,
        pub action: String,
        pub resource: String,
        pub ability: String,
        pub request_binding: crate::model::projection::EvaluatedReferenceBinding,
        pub payload_binding: crate::model::projection::EvaluatedReferenceBinding,
    }

    #[derive(
        Clone, Copy, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize,
    )]
    #[serde(rename_all = "kebab-case")]
    pub enum LatticePortDirection {
        Input,
        Output,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct LatticePort {
        pub id: u64,
        pub direction: crate::model::projection::LatticePortDirection,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct LatticeNode {
        pub id: u64,
        pub ports: Vec<crate::model::projection::LatticePort>,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct LatticePortEdge {
        pub source: u64,
        pub target: u64,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct LatticeNodeConfig {
        pub node_id: u64,
        pub operation: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct LatticeNodeConfigV2 {
        pub node_id: u64,
        pub operation: String,
        pub effectful: bool,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EvaluatedConcreteReference {
        pub profile: String,
        pub identity: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
    pub enum EvaluatedReferenceBinding {
        Exact {
            reference: crate::model::projection::EvaluatedConcreteReference,
        },
        Slot {
            slot_id: String,
            source_ref: String,
        },
    }
}

pub mod admission {
    pub const CHOREGRAPH_EVALUATED_SCHEMA: &str = "choregraph.evaluated-graph.v1";
    pub const LATTICE_EVALUATED_SCHEMA: &str = "choregraph.lattice-workflow-projection.v1";
    pub const LATTICE_V2_EVALUATED_SCHEMA: &str = "choregraph.lattice-workflow-projection.v2";
    pub const RESTRICTED_EVALUATOR_PROFILE: &str = "choregraph.nickel-restricted.v1";

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(tag = "source_format", deny_unknown_fields)]
    pub enum EvaluatedSource {
        #[serde(rename = "choregraph-v1")]
        Choregraph {
            schema: String,
            lifecycle_profile: String,
            events: Vec<String>,
            limits: crate::model::graph::EvaluatedLimits,
            imports: Vec<crate::model::dependency::ImportDeclaration>,
            nodes: Vec<crate::model::graph::EvaluatedNode>,
            edges: Vec<crate::model::graph::EvaluatedEdge>,
            effects: Vec<crate::model::graph::EvaluatedEffect>,
            #[serde(default = "Vec::new")]
            suspensions: Vec<crate::model::graph::EvaluatedSuspension>,
        },
        #[serde(rename = "lattice-workflow-v1")]
        LatticeWorkflow {
            schema: String,
            lifecycle_profile: String,
            events: Vec<String>,
            limits: crate::model::graph::EvaluatedLimits,
            imports: Vec<crate::model::dependency::ImportDeclaration>,
            nodes: Vec<crate::model::projection::LatticeNode>,
            edges: Vec<crate::model::projection::LatticePortEdge>,
            configs: Vec<crate::model::projection::LatticeNodeConfig>,
        },
        #[serde(rename = "lattice-workflow-v2")]
        LatticeWorkflowV2 {
            schema: String,
            lifecycle_profile: String,
            events: Vec<String>,
            limits: crate::model::graph::EvaluatedLimits,
            imports: Vec<crate::model::dependency::ImportDeclaration>,
            nodes: Vec<crate::model::projection::LatticeNode>,
            edges: Vec<crate::model::projection::LatticePortEdge>,
            configs: Vec<crate::model::projection::LatticeNodeConfigV2>,
            effects: Vec<crate::model::projection::EvaluatedLatticeEffectV2>,
        },
    }

    impl crate::model::admission::EvaluatedSource {
        /// Import I/O is available only for the reviewed source profiles.
        pub fn imports(
            &self,
        ) -> Result<
            &[crate::model::dependency::ImportDeclaration],
            crate::diagnostic::FrontendFailure,
        > {
            if let Self::Choregraph { imports, .. } = self {
                return Ok(imports);
            }
            if let Self::LatticeWorkflow { imports, .. } = self {
                return Ok(imports);
            }
            if let Self::LatticeWorkflowV2 { imports, .. } = self {
                return Ok(imports);
            }
            Err(crate::diagnostic::FrontendFailure::one(
                crate::diagnostic::FrontendIssueCode::UnsupportedSourceFormat,
            ))
        }
    }
}
