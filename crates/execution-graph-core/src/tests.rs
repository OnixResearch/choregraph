use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::admission::{DiagnosticCode, admit_graph};
use crate::aggregate::{ResumeDecision, ready_nodes, resume_decision};
use crate::lifecycle::{StepAction, StepState, transition_step_state};
use crate::queries::would_create_cycle;
use crate::structure::{Edge, Node, Topology};

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

fn diamond() -> Topology {
    Topology::new(
        alloc::vec![
            Node::new(NODE_D),
            Node::new(NODE_C),
            Node::new(NODE_B),
            Node::new(NODE_A),
        ],
        alloc::vec![
            Edge::new(NODE_A, NODE_C),
            Edge::new(NODE_C, NODE_D),
            Edge::new(NODE_A, NODE_B),
            Edge::new(NODE_B, NODE_D),
        ],
    )
}

#[test]
// r[verify choregraph.execution_graph_compatibility.validation]
fn admitted_topology_has_stable_order_groups_and_analytics() {
    let graph = diamond();
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
fn empty_topology_is_a_valid_no_work_plan() {
    let plan = must(admit_graph(&Topology::default()));
    assert!(plan.topological_order.is_empty());
    assert!(plan.parallel_groups().is_empty());
    assert!(plan.critical_path().is_empty());
    assert!(plan.bottlenecks().is_empty());
}

#[test]
// r[verify choregraph.execution_graph_compatibility.validation]
fn static_admission_rejects_malformed_topology_facts() {
    let graph = Topology::new(
        alloc::vec![Node::new(NODE_A), Node::new(NODE_A), Node::new(""),],
        alloc::vec![
            Edge::new(NODE_A, NODE_A),
            Edge::new(NODE_A, MISSING_NODE),
            Edge::new(MISSING_NODE, NODE_A),
            Edge::new(NODE_A, MISSING_NODE),
        ],
    );

    let diagnostics = must_err(admit_graph(&graph));
    let codes = diagnostics
        .iter()
        .map(|item| item.code)
        .collect::<BTreeSet<_>>();
    assert!(codes.contains(&DiagnosticCode::DuplicateNodeId));
    assert!(codes.contains(&DiagnosticCode::EmptyNodeId));
    assert!(codes.contains(&DiagnosticCode::SelfLoop));
    assert!(codes.contains(&DiagnosticCode::MissingSourceNode));
    assert!(codes.contains(&DiagnosticCode::MissingTargetNode));
    assert!(codes.contains(&DiagnosticCode::DuplicateEdge));
}

#[test]
fn admission_rejects_cycle() {
    let graph = Topology::new(
        alloc::vec![Node::new(NODE_A), Node::new(NODE_B)],
        alloc::vec![Edge::new(NODE_A, NODE_B), Edge::new(NODE_B, NODE_A),],
    );

    let diagnostics = must_err(admit_graph(&graph));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, DiagnosticCode::CycleDetected);
}

#[test]
fn ready_and_resume_decisions_require_completed_dependencies() {
    let plan = must(admit_graph(&diamond()));
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
    let graph = Topology::new(
        alloc::vec![Node::new(NODE_A), Node::new(NODE_B)],
        alloc::vec![Edge::new(NODE_A, NODE_B)],
    );
    assert!(would_create_cycle(&graph, NODE_B, NODE_A));
    assert!(!would_create_cycle(&graph, NODE_A, NODE_B));
    assert!(!would_create_cycle(&graph, NODE_A, MISSING_NODE));
    assert!(would_create_cycle(&graph, MISSING_NODE, MISSING_NODE));
}

#[test]
fn malformed_and_bounded_inputs_fail_closed() {
    let malformed = Topology::new(alloc::vec![Node::new("node with spaces")], Vec::new());
    assert_eq!(
        must_err(admit_graph(&malformed))[0].code,
        DiagnosticCode::EmptyNodeId
    );

    let maximum_nodes = choregraph_core::model::capacity::DEFAULT_MAX_NODES;
    let nodes = (0..=maximum_nodes)
        .map(|index| Node::new(format!("node-{index}")))
        .collect();
    let oversized = Topology::new(nodes, Vec::new());
    assert!(
        must_err(admit_graph(&oversized))
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::GraphTooLarge)
    );
}

#[test]
fn renamed_types_preserve_serialized_fields_and_round_trip() {
    let topology = Topology::new(
        alloc::vec![Node::new(NODE_A), Node::new(NODE_B)],
        alloc::vec![Edge::new(NODE_A, NODE_B)],
    );
    let encoded = r#"{"nodes":[{"id":"a"},{"id":"b"}],"edges":[{"source":"a","target":"b"}]}"#;
    assert_eq!(must(serde_json::to_string(&topology)), encoded);
    assert_eq!(must(serde_json::from_str::<Topology>(encoded)), topology);
    let plan = must(admit_graph(&topology));
    assert_eq!(
        must(serde_json::to_string(&plan.parallel_groups())),
        r#"[{"node_ids":["a"],"level":0,"can_parallelize":false},{"node_ids":["b"],"level":1,"can_parallelize":false}]"#
    );
    let malformed = Topology::new(alloc::vec![Node::new("")], Vec::new());
    let diagnostics = must_err(admit_graph(&malformed));
    assert_eq!(
        must(serde_json::to_string(&diagnostics)),
        r#"[{"code":"empty_node_id","node_id":"","related_node_id":null,"message":"execution graph node id is invalid"}]"#
    );
}

#[test]
fn lifecycle_and_resume_wire_vocabulary_remains_closed() {
    assert_eq!(
        must(serde_json::to_string(&[
            StepState::Pending,
            StepState::Ready,
            StepState::Running,
            StepState::Completed,
            StepState::Failed,
            StepState::Cancelled,
        ])),
        r#"["pending","ready","running","completed","failed","cancelled"]"#
    );
    assert_eq!(
        must(serde_json::to_string(&[
            StepAction::MarkReady,
            StepAction::Start,
            StepAction::Complete,
            StepAction::Fail,
            StepAction::Cancel,
        ])),
        r#"["mark_ready","start","complete","fail","cancel"]"#
    );
    assert_eq!(
        must(serde_json::to_string(&[
            ResumeDecision::Execute,
            ResumeDecision::AwaitDependencies,
            ResumeDecision::SkipCompleted,
            ResumeDecision::RejectTerminalMutation,
            ResumeDecision::BlockedByFailedDependency,
        ])),
        r#"["execute","await_dependencies","skip_completed","reject_terminal_mutation","blocked_by_failed_dependency"]"#
    );
}
