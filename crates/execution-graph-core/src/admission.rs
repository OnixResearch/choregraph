use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::adapter;
use crate::aggregate::Plan;
use crate::structure::Topology;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename = "GraphDiagnosticCode", rename_all = "snake_case")]
pub enum DiagnosticCode {
    EmptyNodeId,
    DuplicateNodeId,
    DuplicateEdge,
    MissingSourceNode,
    MissingTargetNode,
    SelfLoop,
    CycleDetected,
    GraphTooLarge,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename = "GraphDiagnostic")]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub node_id: Option<String>,
    pub related_node_id: Option<String>,
    pub message: String,
}
// r[impl choregraph.execution_graph_compatibility.api]
// r[impl choregraph.execution_graph_compatibility.delegation]
pub fn admit_graph(graph: &Topology) -> Result<Plan, Vec<Diagnostic>> {
    let machine = adapter::admit_machine(graph)?;
    let topological_order = adapter::node_strings(machine.topological_order());
    Ok(Plan {
        graph: graph.clone(),
        machine,
        topological_order,
    })
}

pub fn validate(graph: &Topology) -> Vec<Diagnostic> {
    match adapter::admit_machine(graph) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics,
    }
}
impl core::fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let context = match (&self.node_id, &self.related_node_id) {
            (Some(node_id), Some(related)) => format!(" ({node_id} -> {related})"),
            (Some(node_id), None) => format!(" ({node_id})"),
            (None, Some(related)) => format!(" ({related})"),
            (None, None) => String::new(),
        };
        write!(formatter, "{}{context}", self.message)
    }
}
