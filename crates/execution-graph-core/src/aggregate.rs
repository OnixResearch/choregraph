use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::adapter;
use crate::lifecycle::StepState;
use crate::structure::{Bottleneck, Group, Topology};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub(super) graph: Topology,
    pub(super) machine: choregraph_core::model::automaton::AdmittedMachine,
    pub topological_order: Vec<String>,
}

impl Plan {
    pub fn graph(&self) -> &Topology {
        &self.graph
    }

    pub fn dependencies(&self, node_id: &str) -> Vec<String> {
        adapter::dependencies(&self.machine, node_id)
    }

    pub fn dependents(&self, node_id: &str) -> Vec<String> {
        adapter::dependents(&self.machine, node_id)
    }

    pub fn parallel_groups(&self) -> Vec<Group> {
        adapter::parallel_groups(&self.machine)
    }

    pub fn critical_path(&self) -> Vec<String> {
        adapter::critical_path(&self.machine)
    }

    pub fn bottlenecks(&self) -> Vec<Bottleneck> {
        adapter::bottlenecks(&self.machine)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeDecision {
    Execute,
    AwaitDependencies,
    SkipCompleted,
    RejectTerminalMutation,
    BlockedByFailedDependency,
}
// r[impl choregraph.execution_graph_compatibility.api]
pub fn ready_nodes(plan: &Plan, states: &BTreeMap<String, StepState>) -> Vec<String> {
    adapter::ready_nodes(&plan.machine, states)
}
pub fn resume_decision(
    plan: &Plan,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> ResumeDecision {
    adapter::resume_decision(&plan.machine, states, node_id)
}
