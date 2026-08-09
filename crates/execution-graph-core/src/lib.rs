#![no_std]
// This crate preserves the published compatibility surface and its exact legacy test fixtures.
#![allow(
    unknown_lints,
    ambiguous_params,
    explicit_defaults,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    no_panic,
    non_trait_imports,
    numeric_units,
    reason = "the compatibility facade retains published signatures and exact legacy fixtures"
)]

// r[impl choregraph.execution_graph_compatibility.boundary]
extern crate alloc;

mod adapter;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionNode {
    pub id: String,
}

impl ExecutionNode {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ExecutionEdge {
    pub source: String,
    pub target: String,
}

impl ExecutionEdge {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionGraph {
    pub nodes: Vec<ExecutionNode>,
    pub edges: Vec<ExecutionEdge>,
}

impl ExecutionGraph {
    pub fn new(nodes: Vec<ExecutionNode>, edges: Vec<ExecutionEdge>) -> Self {
        Self { nodes, edges }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum GraphDiagnosticCode {
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
pub struct GraphDiagnostic {
    pub code: GraphDiagnosticCode,
    pub node_id: Option<String>,
    pub related_node_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    graph: ExecutionGraph,
    machine: choregraph_core::AdmittedMachine,
    pub topological_order: Vec<String>,
}

impl ExecutionPlan {
    pub fn graph(&self) -> &ExecutionGraph {
        &self.graph
    }

    pub fn dependencies(&self, node_id: &str) -> Vec<String> {
        adapter::dependencies(&self.machine, node_id)
    }

    pub fn dependents(&self, node_id: &str) -> Vec<String> {
        adapter::dependents(&self.machine, node_id)
    }

    pub fn parallel_groups(&self) -> Vec<ExecutionGroup> {
        adapter::execution_groups(&self.machine)
    }

    pub fn critical_path(&self) -> Vec<String> {
        adapter::critical_path(&self.machine)
    }

    pub fn bottlenecks(&self) -> Vec<Bottleneck> {
        adapter::bottlenecks(&self.machine)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExecutionGroup {
    pub node_ids: Vec<String>,
    pub level: u32,
    pub can_parallelize: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bottleneck {
    pub node_id: String,
    pub dependent_count: u32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTransitionError {
    InvalidTransition { from: StepState, action: StepAction },
    TerminalState { from: StepState, action: StepAction },
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
// r[impl choregraph.execution_graph_compatibility.delegation]
pub fn admit_graph(graph: &ExecutionGraph) -> Result<ExecutionPlan, Vec<GraphDiagnostic>> {
    let machine = adapter::admit_machine(graph)?;
    let topological_order = adapter::node_strings(machine.topological_order());
    Ok(ExecutionPlan {
        graph: graph.clone(),
        machine,
        topological_order,
    })
}

pub fn validate_graph(graph: &ExecutionGraph) -> Vec<GraphDiagnostic> {
    match adapter::admit_machine(graph) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics,
    }
}

pub fn direct_dependencies(graph: &ExecutionGraph, node_id: &str) -> Vec<String> {
    match adapter::admit_machine(graph) {
        Ok(machine) => adapter::dependencies(&machine, node_id),
        Err(_) => Vec::new(),
    }
}

pub fn direct_dependents(graph: &ExecutionGraph, node_id: &str) -> Vec<String> {
    match adapter::admit_machine(graph) {
        Ok(machine) => adapter::dependents(&machine, node_id),
        Err(_) => Vec::new(),
    }
}

// r[impl choregraph.execution_graph_compatibility.api]
pub fn ready_nodes(plan: &ExecutionPlan, states: &BTreeMap<String, StepState>) -> Vec<String> {
    adapter::ready_nodes(&plan.machine, states)
}

pub fn transition_step_state(
    from: StepState,
    action: StepAction,
) -> Result<StepState, StepTransitionError> {
    adapter::transition(from, action)
}

pub fn is_terminal_state(state: StepState) -> bool {
    adapter::is_terminal(state)
}

pub fn resume_decision(
    plan: &ExecutionPlan,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> ResumeDecision {
    adapter::resume_decision(&plan.machine, states, node_id)
}

pub fn would_create_cycle(graph: &ExecutionGraph, source: &str, target: &str) -> bool {
    match adapter::admit_machine(graph) {
        Ok(machine) => adapter::would_create_cycle(&machine, source, target),
        Err(_) => false,
    }
}

impl core::fmt::Display for GraphDiagnostic {
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

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::string::ToString;

    use super::*;

    const NODE_A: &str = "a";
    const NODE_B: &str = "b";
    const NODE_C: &str = "c";
    const NODE_D: &str = "d";
    const MISSING_NODE: &str = "missing";
    const EXPECTED_DIAMOND_NODE_COUNT: usize = 4;
    const EXPECTED_DIAMOND_EDGE_COUNT: usize = 4;
    const EXPECTED_GROUP_COUNT: usize = 3;
    const EXPECTED_FANOUT_WIDTH: usize = 2;
    const EXPECTED_ROOT_DEPENDENT_COUNT: u32 = 3;

    #[allow(
        unknown_lints,
        no_panic,
        reason = "a positive compatibility test must stop when admission unexpectedly fails"
    )]
    fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected success, got {error:?}"),
        }
    }

    #[allow(
        unknown_lints,
        no_panic,
        reason = "a negative compatibility test must stop when admission unexpectedly succeeds"
    )]
    fn must_err<T: core::fmt::Debug, E>(result: Result<T, E>) -> E {
        match result {
            Ok(value) => panic!("expected failure, got {value:?}"),
            Err(error) => error,
        }
    }

    fn diamond_graph() -> ExecutionGraph {
        ExecutionGraph::new(
            alloc::vec![
                ExecutionNode::new(NODE_D),
                ExecutionNode::new(NODE_C),
                ExecutionNode::new(NODE_B),
                ExecutionNode::new(NODE_A),
            ],
            alloc::vec![
                ExecutionEdge::new(NODE_A, NODE_C),
                ExecutionEdge::new(NODE_C, NODE_D),
                ExecutionEdge::new(NODE_A, NODE_B),
                ExecutionEdge::new(NODE_B, NODE_D),
            ],
        )
    }

    #[test]
    // r[verify choregraph.execution_graph_compatibility.validation]
    fn admitted_graph_has_stable_order_groups_and_analytics() {
        let graph = diamond_graph();
        let plan = must(admit_graph(&graph));

        assert_eq!(plan.graph().nodes.len(), EXPECTED_DIAMOND_NODE_COUNT);
        assert_eq!(plan.graph().edges.len(), EXPECTED_DIAMOND_EDGE_COUNT);
        assert_eq!(
            plan.topological_order,
            alloc::vec![NODE_A, NODE_B, NODE_C, NODE_D]
        );
        assert_eq!(plan.dependencies(NODE_D), alloc::vec![NODE_B, NODE_C]);
        assert_eq!(plan.dependents(NODE_A), alloc::vec![NODE_B, NODE_C]);

        let groups = plan.parallel_groups();
        assert_eq!(groups.len(), EXPECTED_GROUP_COUNT);
        assert_eq!(groups[1].node_ids.len(), EXPECTED_FANOUT_WIDTH);
        assert!(groups[1].can_parallelize);
        assert_eq!(plan.critical_path(), alloc::vec![NODE_A, NODE_B, NODE_D]);

        let bottlenecks = plan.bottlenecks();
        assert_eq!(bottlenecks[0].node_id, NODE_A);
        assert_eq!(
            bottlenecks[0].dependent_count,
            EXPECTED_ROOT_DEPENDENT_COUNT
        );
    }

    #[test]
    fn empty_graph_is_a_valid_no_work_plan() {
        let plan = must(admit_graph(&ExecutionGraph::default()));
        assert!(plan.topological_order.is_empty());
        assert!(plan.parallel_groups().is_empty());
        assert!(plan.critical_path().is_empty());
        assert!(plan.bottlenecks().is_empty());
    }

    #[test]
    // r[verify choregraph.execution_graph_compatibility.validation]
    fn static_admission_rejects_malformed_graph_facts() {
        let graph = ExecutionGraph::new(
            alloc::vec![
                ExecutionNode::new(NODE_A),
                ExecutionNode::new(NODE_A),
                ExecutionNode::new(""),
            ],
            alloc::vec![
                ExecutionEdge::new(NODE_A, NODE_A),
                ExecutionEdge::new(NODE_A, MISSING_NODE),
                ExecutionEdge::new(MISSING_NODE, NODE_A),
                ExecutionEdge::new(NODE_A, MISSING_NODE),
            ],
        );

        let diagnostics = must_err(admit_graph(&graph));
        let codes = diagnostics
            .iter()
            .map(|item| item.code)
            .collect::<BTreeSet<_>>();
        assert!(codes.contains(&GraphDiagnosticCode::DuplicateNodeId));
        assert!(codes.contains(&GraphDiagnosticCode::EmptyNodeId));
        assert!(codes.contains(&GraphDiagnosticCode::SelfLoop));
        assert!(codes.contains(&GraphDiagnosticCode::MissingSourceNode));
        assert!(codes.contains(&GraphDiagnosticCode::MissingTargetNode));
        assert!(codes.contains(&GraphDiagnosticCode::DuplicateEdge));
    }

    #[test]
    fn admission_rejects_cycle() {
        let graph = ExecutionGraph::new(
            alloc::vec![ExecutionNode::new(NODE_A), ExecutionNode::new(NODE_B)],
            alloc::vec![
                ExecutionEdge::new(NODE_A, NODE_B),
                ExecutionEdge::new(NODE_B, NODE_A),
            ],
        );

        let diagnostics = must_err(admit_graph(&graph));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, GraphDiagnosticCode::CycleDetected);
    }

    #[test]
    fn ready_and_resume_decisions_require_completed_dependencies() {
        let plan = must(admit_graph(&diamond_graph()));
        let mut states = BTreeMap::new();
        assert_eq!(ready_nodes(&plan, &states), alloc::vec![NODE_A]);
        assert_eq!(
            resume_decision(&plan, &states, NODE_B),
            ResumeDecision::AwaitDependencies
        );

        states.insert(NODE_A.to_string(), StepState::Completed);
        assert_eq!(ready_nodes(&plan, &states), alloc::vec![NODE_B, NODE_C]);
        assert_eq!(
            resume_decision(&plan, &states, NODE_B),
            ResumeDecision::Execute
        );
        assert_eq!(
            resume_decision(&plan, &states, NODE_A),
            ResumeDecision::SkipCompleted
        );

        states.insert(NODE_A.to_string(), StepState::Failed);
        assert!(ready_nodes(&plan, &states).is_empty());
        assert_eq!(
            resume_decision(&plan, &states, NODE_B),
            ResumeDecision::BlockedByFailedDependency
        );
    }

    #[test]
    // r[verify choregraph.execution_graph_compatibility.validation]
    fn lifecycle_accepts_valid_paths_and_rejects_invalid_or_terminal_mutation() {
        let ready = must(transition_step_state(
            StepState::Pending,
            StepAction::MarkReady,
        ));
        let running = must(transition_step_state(ready, StepAction::Start));
        let completed = must(transition_step_state(running, StepAction::Complete));
        assert_eq!(completed, StepState::Completed);

        assert!(transition_step_state(StepState::Pending, StepAction::Complete).is_err());
        assert!(transition_step_state(StepState::Completed, StepAction::Cancel).is_err());
        assert_eq!(
            must(transition_step_state(StepState::Running, StepAction::Fail,)),
            StepState::Failed
        );
    }

    #[test]
    fn cycle_probe_checks_target_to_source_reachability() {
        let graph = ExecutionGraph::new(
            alloc::vec![ExecutionNode::new(NODE_A), ExecutionNode::new(NODE_B)],
            alloc::vec![ExecutionEdge::new(NODE_A, NODE_B)],
        );
        assert!(would_create_cycle(&graph, NODE_B, NODE_A));
        assert!(!would_create_cycle(&graph, NODE_A, NODE_B));
        assert!(!would_create_cycle(&graph, NODE_A, MISSING_NODE));
        assert!(would_create_cycle(&graph, MISSING_NODE, MISSING_NODE));
    }

    #[test]
    fn malformed_and_bounded_inputs_fail_closed() {
        let malformed = ExecutionGraph::new(
            alloc::vec![ExecutionNode::new("node with spaces")],
            Vec::new(),
        );
        assert_eq!(
            must_err(admit_graph(&malformed))[0].code,
            GraphDiagnosticCode::EmptyNodeId
        );

        let maximum_nodes = choregraph_core::DEFAULT_MAX_NODES;
        let nodes = (0..=maximum_nodes)
            .map(|index| ExecutionNode::new(format!("node-{index}")))
            .collect();
        let oversized = ExecutionGraph::new(nodes, Vec::new());
        assert!(
            must_err(admit_graph(&oversized))
                .iter()
                .any(|diagnostic| diagnostic.code == GraphDiagnosticCode::GraphTooLarge)
        );
    }
}
