//! Exact construction profiles and resource bounds.

use serde::{Deserialize, Serialize};
pub const CONSTRUCTION_SCHEMA: &str = "choregraph.graph-construction.v1";
pub const REGISTRY_SCHEMA: &str = "choregraph.target-registry.v1";
pub const PROJECTION_SCHEMA: &str = "choregraph.lattice-workflow-projection.v3";
pub const ELABORATOR_PROFILE: &str = "choregraph.graph-elaborator.v1";
pub const CAPTURED_EVALUATOR_PROFILE: &str = "nickel-eval.captured-modules.v1";
pub const LIBRARY_MODULE: &str = "choregraph/graph-construction-v1.ncl";
pub const CONTRACTS_MODULE: &str = "choregraph/graph-construction-contracts-v1.ncl";
pub const LIBRARY_BYTES: &[u8] =
    include_bytes!("../../../../../library/nickel/graph-construction-v1.ncl");
pub const CONTRACTS_BYTES: &[u8] =
    include_bytes!("../../../../../contracts/nickel/graph-construction-contracts-v1.ncl");
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_candidate_bytes: usize,
    pub max_nodes: usize,
    pub max_ports: usize,
    pub max_edges: usize,
    pub max_effects: usize,
    pub max_instances: usize,
    pub max_depth: usize,
    pub max_boundaries: usize,
    pub max_identifier_bytes: usize,
    pub max_origin_bytes: usize,
    pub max_parameter_bytes: usize,
    pub max_output_bytes: usize,
}
const DEFAULT_LIMITS: Limits = Limits {
    max_candidate_bytes: 1_048_576,
    max_nodes: choregraph_core::model::capacity::DEFAULT_MAX_NODES,
    max_ports: 16_384,
    max_edges: choregraph_core::model::capacity::DEFAULT_MAX_EDGES,
    max_effects: choregraph_core::model::capacity::DEFAULT_MAX_EFFECT_DECLARATIONS,
    max_instances: choregraph_core::model::capacity::DEFAULT_MAX_NODES,
    max_depth: 64,
    max_boundaries: 16_384,
    max_identifier_bytes: choregraph_core::model::capacity::DEFAULT_MAX_IDENTIFIER_BYTES,
    max_origin_bytes: 1_048_576,
    max_parameter_bytes: 1_048_576,
    max_output_bytes: 8_388_608,
};
impl Default for Limits {
    fn default() -> Self {
        DEFAULT_LIMITS
    }
}
