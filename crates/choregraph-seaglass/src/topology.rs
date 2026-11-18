use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::admission::{GraphDiagnostic, admit_graph};

/// One legacy-shaped node declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionNode {
    pub id: String,
}

impl ExecutionNode {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// One legacy-shaped dependency edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutionEdge {
    pub source: String,
    pub target: String,
}

impl ExecutionEdge {
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

/// One untrusted compatibility graph.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionGraph {
    pub nodes: Vec<ExecutionNode>,
    pub edges: Vec<ExecutionEdge>,
}

impl ExecutionGraph {
    #[must_use]
    pub const fn new(nodes: Vec<ExecutionNode>, edges: Vec<ExecutionEdge>) -> Self {
        Self { nodes, edges }
    }
}
/// One converted graph level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionGroup {
    pub node_ids: Vec<String>,
    pub level: u32,
    pub can_parallelize: bool,
}

/// One converted bottleneck result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bottleneck {
    pub node_id: String,
    pub dependent_count: u32,
}
/// A graph-query conversion failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryError {
    InvalidNodeId,
    UnknownNode,
    LevelOutsideProfile,
    CountOutsideProfile,
}

/// Named inputs for one proposed-edge cycle probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CycleProbe<'a> {
    pub graph: &'a ExecutionGraph,
    pub source: &'a str,
    pub target: &'a str,
}
pub(super) fn string_ids(node_ids: &[choregraph_core::id::bounded::NodeId]) -> Vec<String> {
    node_ids
        .iter()
        .map(|node_id| node_id.as_str().to_string())
        .collect()
}

pub(super) fn parse_node_id(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
    value: &str,
) -> Result<choregraph_core::id::bounded::NodeId, QueryError> {
    choregraph_core::id::bounded::NodeId::parse(value, machine.limits().max_identifier_bytes)
        .map_err(|_| QueryError::InvalidNodeId)
}
/// Checks one proposed edge through the production Choregraph cycle probe.
#[must_use]
#[expect(
    clippy::manual_unwrap_or,
    reason = "explicit compatibility control flow keeps the conservative rejection branch visible"
)]
#[allow(
    unknown_lints,
    ambiguous_params,
    reason = "the exact legacy cycle-probe signature has two adjacent identifier parameters"
)]
pub fn would_create_cycle(graph: &ExecutionGraph, source: &str, target: &str) -> bool {
    match would_create_cycle_checked(CycleProbe {
        graph,
        source,
        target,
    }) {
        Ok(decision) => decision,
        Err(_) => true,
    }
}

/// Checks one proposed edge with explicit graph conversion failures for new consumers.
pub fn would_create_cycle_checked(probe: CycleProbe<'_>) -> Result<bool, Vec<GraphDiagnostic>> {
    let plan = admit_graph(probe.graph)?;
    let Ok(source) = parse_node_id(plan.machine(), probe.source) else {
        return Ok(false);
    };
    let Ok(target) = parse_node_id(plan.machine(), probe.target) else {
        return Ok(false);
    };
    Ok(choregraph_core::analytics::would_create_cycle(
        plan.machine(),
        &source,
        &target,
    ))
}
