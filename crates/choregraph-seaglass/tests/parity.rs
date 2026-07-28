#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "parity fixtures use legacy collection signatures directly for readable comparisons"
)]

// r[verify choregraph.conformance.sources]
// r[verify choregraph.conformance.inventory]
// r[verify choregraph.conformance.parity]
use std::collections::{BTreeMap, BTreeSet};

const NODE_A: &str = "a";
const NODE_B: &str = "b";
const NODE_C: &str = "c";
const NODE_D: &str = "d";
const MISSING_NODE: &str = "missing";
const INVALID_PROFILE_NODE: &str = "legacy accepts spaces";
const EXPECTED_DIAMOND_NODE_COUNT: usize = 4;
const EXPECTED_DIAMOND_EDGE_COUNT: usize = 4;
const EXPECTED_GROUP_COUNT: usize = 3;
const EXPECTED_ROOT_DEPENDENT_COUNT: u32 = 3;
const EXPECTED_MALFORMED_CODE_COUNT: usize = 6;

#[allow(
    unknown_lints,
    no_panic,
    reason = "a parity test must stop immediately when either compared implementation rejects a positive fixture"
)]
fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("expected success, got {error:?}"),
    }
}

fn legacy_diamond() -> legacy_execution_graph::ExecutionGraph {
    legacy_execution_graph::ExecutionGraph::new(
        vec![
            legacy_execution_graph::ExecutionNode::new(NODE_D),
            legacy_execution_graph::ExecutionNode::new(NODE_C),
            legacy_execution_graph::ExecutionNode::new(NODE_B),
            legacy_execution_graph::ExecutionNode::new(NODE_A),
        ],
        vec![
            legacy_execution_graph::ExecutionEdge::new(NODE_A, NODE_C),
            legacy_execution_graph::ExecutionEdge::new(NODE_C, NODE_D),
            legacy_execution_graph::ExecutionEdge::new(NODE_A, NODE_B),
            legacy_execution_graph::ExecutionEdge::new(NODE_B, NODE_D),
        ],
    )
}

fn choregraph_diamond() -> choregraph_seaglass::ExecutionGraph {
    choregraph_seaglass::ExecutionGraph::new(
        vec![
            choregraph_seaglass::ExecutionNode::new(NODE_D),
            choregraph_seaglass::ExecutionNode::new(NODE_C),
            choregraph_seaglass::ExecutionNode::new(NODE_B),
            choregraph_seaglass::ExecutionNode::new(NODE_A),
        ],
        vec![
            choregraph_seaglass::ExecutionEdge::new(NODE_A, NODE_C),
            choregraph_seaglass::ExecutionEdge::new(NODE_C, NODE_D),
            choregraph_seaglass::ExecutionEdge::new(NODE_A, NODE_B),
            choregraph_seaglass::ExecutionEdge::new(NODE_B, NODE_D),
        ],
    )
}

const fn compatibility_code(
    code: legacy_execution_graph::GraphDiagnosticCode,
) -> choregraph_seaglass::GraphDiagnosticCode {
    match code {
        legacy_execution_graph::GraphDiagnosticCode::EmptyNodeId => {
            choregraph_seaglass::GraphDiagnosticCode::EmptyNodeId
        }
        legacy_execution_graph::GraphDiagnosticCode::DuplicateNodeId => {
            choregraph_seaglass::GraphDiagnosticCode::DuplicateNodeId
        }
        legacy_execution_graph::GraphDiagnosticCode::DuplicateEdge => {
            choregraph_seaglass::GraphDiagnosticCode::DuplicateEdge
        }
        legacy_execution_graph::GraphDiagnosticCode::MissingSourceNode => {
            choregraph_seaglass::GraphDiagnosticCode::MissingSourceNode
        }
        legacy_execution_graph::GraphDiagnosticCode::MissingTargetNode => {
            choregraph_seaglass::GraphDiagnosticCode::MissingTargetNode
        }
        legacy_execution_graph::GraphDiagnosticCode::SelfLoop => {
            choregraph_seaglass::GraphDiagnosticCode::SelfLoop
        }
        legacy_execution_graph::GraphDiagnosticCode::CycleDetected => {
            choregraph_seaglass::GraphDiagnosticCode::CycleDetected
        }
        legacy_execution_graph::GraphDiagnosticCode::GraphTooLarge => {
            choregraph_seaglass::GraphDiagnosticCode::GraphTooLarge
        }
    }
}

const fn legacy_state(state: choregraph_seaglass::StepState) -> legacy_execution_graph::StepState {
    match state {
        choregraph_seaglass::StepState::Pending => legacy_execution_graph::StepState::Pending,
        choregraph_seaglass::StepState::Ready => legacy_execution_graph::StepState::Ready,
        choregraph_seaglass::StepState::Running => legacy_execution_graph::StepState::Running,
        choregraph_seaglass::StepState::Completed => legacy_execution_graph::StepState::Completed,
        choregraph_seaglass::StepState::Failed => legacy_execution_graph::StepState::Failed,
        choregraph_seaglass::StepState::Cancelled => legacy_execution_graph::StepState::Cancelled,
    }
}

const fn legacy_action(
    action: choregraph_seaglass::StepAction,
) -> legacy_execution_graph::StepAction {
    match action {
        choregraph_seaglass::StepAction::MarkReady => legacy_execution_graph::StepAction::MarkReady,
        choregraph_seaglass::StepAction::Start => legacy_execution_graph::StepAction::Start,
        choregraph_seaglass::StepAction::Complete => legacy_execution_graph::StepAction::Complete,
        choregraph_seaglass::StepAction::Fail => legacy_execution_graph::StepAction::Fail,
        choregraph_seaglass::StepAction::Cancel => legacy_execution_graph::StepAction::Cancel,
    }
}

const fn compatibility_state(
    state: legacy_execution_graph::StepState,
) -> choregraph_seaglass::StepState {
    match state {
        legacy_execution_graph::StepState::Pending => choregraph_seaglass::StepState::Pending,
        legacy_execution_graph::StepState::Ready => choregraph_seaglass::StepState::Ready,
        legacy_execution_graph::StepState::Running => choregraph_seaglass::StepState::Running,
        legacy_execution_graph::StepState::Completed => choregraph_seaglass::StepState::Completed,
        legacy_execution_graph::StepState::Failed => choregraph_seaglass::StepState::Failed,
        legacy_execution_graph::StepState::Cancelled => choregraph_seaglass::StepState::Cancelled,
    }
}

const fn compatibility_resume(
    decision: legacy_execution_graph::ResumeDecision,
) -> choregraph_seaglass::ResumeDecision {
    match decision {
        legacy_execution_graph::ResumeDecision::Execute => {
            choregraph_seaglass::ResumeDecision::Execute
        }
        legacy_execution_graph::ResumeDecision::AwaitDependencies => {
            choregraph_seaglass::ResumeDecision::AwaitDependencies
        }
        legacy_execution_graph::ResumeDecision::SkipCompleted => {
            choregraph_seaglass::ResumeDecision::SkipCompleted
        }
        legacy_execution_graph::ResumeDecision::RejectTerminalMutation => {
            choregraph_seaglass::ResumeDecision::RejectTerminalMutation
        }
        legacy_execution_graph::ResumeDecision::BlockedByFailedDependency => {
            choregraph_seaglass::ResumeDecision::BlockedByFailedDependency
        }
    }
}

#[test]
fn diamond_admission_queries_groups_and_analytics_match() {
    let legacy_graph = legacy_diamond();
    let compatibility_graph = choregraph_diamond();
    let legacy_plan = must(legacy_execution_graph::admit_graph(&legacy_graph));
    let compatibility_plan = must(choregraph_seaglass::admit_graph(&compatibility_graph));

    assert_eq!(
        compatibility_plan.graph().nodes.len(),
        EXPECTED_DIAMOND_NODE_COUNT
    );
    assert_eq!(
        compatibility_plan.graph().edges.len(),
        EXPECTED_DIAMOND_EDGE_COUNT
    );
    assert_eq!(
        compatibility_plan.topological_order,
        legacy_plan.topological_order
    );
    assert_eq!(
        compatibility_plan.dependencies(NODE_D),
        legacy_plan.dependencies(NODE_D)
    );
    assert_eq!(
        compatibility_plan.dependents(NODE_A),
        legacy_plan.dependents(NODE_A)
    );

    let legacy_groups = legacy_plan
        .parallel_groups()
        .into_iter()
        .map(|group| (group.node_ids, group.level, group.can_parallelize))
        .collect::<Vec<_>>();
    let compatibility_groups = compatibility_plan
        .parallel_groups()
        .into_iter()
        .map(|group| (group.node_ids, group.level, group.can_parallelize))
        .collect::<Vec<_>>();
    assert_eq!(compatibility_groups.len(), EXPECTED_GROUP_COUNT);
    assert_eq!(compatibility_groups, legacy_groups);
    assert_eq!(
        compatibility_plan.critical_path(),
        legacy_plan.critical_path()
    );

    let legacy_bottlenecks = legacy_plan
        .bottlenecks()
        .into_iter()
        .map(|item| (item.node_id, item.dependent_count))
        .collect::<Vec<_>>();
    let compatibility_bottlenecks = compatibility_plan
        .bottlenecks()
        .into_iter()
        .map(|item| (item.node_id, item.dependent_count))
        .collect::<Vec<_>>();
    assert_eq!(
        compatibility_bottlenecks[0].1,
        EXPECTED_ROOT_DEPENDENT_COUNT
    );
    assert_eq!(compatibility_bottlenecks, legacy_bottlenecks);
}

#[test]
fn empty_graph_is_an_exact_no_work_plan() {
    let legacy_plan = must(legacy_execution_graph::admit_graph(
        &legacy_execution_graph::ExecutionGraph::new(Vec::new(), Vec::new()),
    ));
    let compatibility_plan = must(choregraph_seaglass::admit_graph(
        &choregraph_seaglass::ExecutionGraph::new(Vec::new(), Vec::new()),
    ));
    assert_eq!(
        compatibility_plan.topological_order,
        legacy_plan.topological_order
    );
    assert_eq!(compatibility_plan.parallel_groups(), Vec::new());
    assert_eq!(
        compatibility_plan.critical_path(),
        legacy_plan.critical_path()
    );
    assert_eq!(compatibility_plan.bottlenecks(), Vec::new());
}

#[test]
fn malformed_graph_diagnostic_classes_match() {
    let legacy_graph = legacy_execution_graph::ExecutionGraph::new(
        vec![
            legacy_execution_graph::ExecutionNode::new(NODE_A),
            legacy_execution_graph::ExecutionNode::new(NODE_A),
            legacy_execution_graph::ExecutionNode::new(""),
        ],
        vec![
            legacy_execution_graph::ExecutionEdge::new(NODE_A, NODE_A),
            legacy_execution_graph::ExecutionEdge::new(NODE_A, MISSING_NODE),
            legacy_execution_graph::ExecutionEdge::new(MISSING_NODE, NODE_A),
            legacy_execution_graph::ExecutionEdge::new(NODE_A, MISSING_NODE),
        ],
    );
    let compatibility_graph = choregraph_seaglass::ExecutionGraph::new(
        vec![
            choregraph_seaglass::ExecutionNode::new(NODE_A),
            choregraph_seaglass::ExecutionNode::new(NODE_A),
            choregraph_seaglass::ExecutionNode::new(""),
        ],
        vec![
            choregraph_seaglass::ExecutionEdge::new(NODE_A, NODE_A),
            choregraph_seaglass::ExecutionEdge::new(NODE_A, MISSING_NODE),
            choregraph_seaglass::ExecutionEdge::new(MISSING_NODE, NODE_A),
            choregraph_seaglass::ExecutionEdge::new(NODE_A, MISSING_NODE),
        ],
    );

    let legacy_codes = legacy_execution_graph::validate_graph(&legacy_graph)
        .into_iter()
        .map(|item| compatibility_code(item.code))
        .collect::<BTreeSet<_>>();
    let compatibility_codes = choregraph_seaglass::validate_graph(&compatibility_graph)
        .into_iter()
        .map(|item| item.code)
        .collect::<BTreeSet<_>>();
    assert_eq!(compatibility_codes.len(), EXPECTED_MALFORMED_CODE_COUNT);
    assert_eq!(compatibility_codes, legacy_codes);
}

#[test]
fn cycle_rejection_and_cycle_probes_match() {
    let legacy_cycle = legacy_execution_graph::ExecutionGraph::new(
        vec![
            legacy_execution_graph::ExecutionNode::new(NODE_A),
            legacy_execution_graph::ExecutionNode::new(NODE_B),
        ],
        vec![
            legacy_execution_graph::ExecutionEdge::new(NODE_A, NODE_B),
            legacy_execution_graph::ExecutionEdge::new(NODE_B, NODE_A),
        ],
    );
    let compatibility_cycle = choregraph_seaglass::ExecutionGraph::new(
        vec![
            choregraph_seaglass::ExecutionNode::new(NODE_A),
            choregraph_seaglass::ExecutionNode::new(NODE_B),
        ],
        vec![
            choregraph_seaglass::ExecutionEdge::new(NODE_A, NODE_B),
            choregraph_seaglass::ExecutionEdge::new(NODE_B, NODE_A),
        ],
    );
    assert_eq!(
        choregraph_seaglass::validate_graph(&compatibility_cycle)[0].code,
        compatibility_code(legacy_execution_graph::validate_graph(&legacy_cycle)[0].code)
    );

    let legacy_graph = legacy_execution_graph::ExecutionGraph::new(
        vec![
            legacy_execution_graph::ExecutionNode::new(NODE_A),
            legacy_execution_graph::ExecutionNode::new(NODE_B),
        ],
        vec![legacy_execution_graph::ExecutionEdge::new(NODE_A, NODE_B)],
    );
    let compatibility_graph = choregraph_seaglass::ExecutionGraph::new(
        vec![
            choregraph_seaglass::ExecutionNode::new(NODE_A),
            choregraph_seaglass::ExecutionNode::new(NODE_B),
        ],
        vec![choregraph_seaglass::ExecutionEdge::new(NODE_A, NODE_B)],
    );
    assert_eq!(
        choregraph_seaglass::would_create_cycle(&compatibility_graph, NODE_B, NODE_A),
        legacy_execution_graph::would_create_cycle(&legacy_graph, NODE_B, NODE_A)
    );
    assert_eq!(
        choregraph_seaglass::would_create_cycle(&compatibility_graph, NODE_A, MISSING_NODE),
        legacy_execution_graph::would_create_cycle(&legacy_graph, NODE_A, MISSING_NODE)
    );
}

#[test]
fn lifecycle_matrix_matches_the_legacy_kernel() {
    let states = [
        choregraph_seaglass::StepState::Pending,
        choregraph_seaglass::StepState::Ready,
        choregraph_seaglass::StepState::Running,
        choregraph_seaglass::StepState::Completed,
        choregraph_seaglass::StepState::Failed,
        choregraph_seaglass::StepState::Cancelled,
    ];
    let actions = [
        choregraph_seaglass::StepAction::MarkReady,
        choregraph_seaglass::StepAction::Start,
        choregraph_seaglass::StepAction::Complete,
        choregraph_seaglass::StepAction::Fail,
        choregraph_seaglass::StepAction::Cancel,
    ];

    for state in states {
        for action in actions {
            let legacy = legacy_execution_graph::transition_step_state(
                legacy_state(state),
                legacy_action(action),
            )
            .map(compatibility_state)
            .map_err(|error| match error {
                legacy_execution_graph::StepTransitionError::InvalidTransition { .. } => {
                    choregraph_seaglass::StepTransitionError::InvalidTransition {
                        from: state,
                        action,
                    }
                }
                legacy_execution_graph::StepTransitionError::TerminalState { .. } => {
                    choregraph_seaglass::StepTransitionError::TerminalState {
                        from: state,
                        action,
                    }
                }
            });
            assert_eq!(
                choregraph_seaglass::transition_step_state(state, action),
                legacy
            );
            assert_eq!(
                choregraph_seaglass::is_terminal_state(state),
                legacy_execution_graph::is_terminal_state(legacy_state(state))
            );
        }
    }
}

#[test]
fn ready_and_resume_decisions_match_for_valid_sparse_states() {
    let legacy_plan = must(legacy_execution_graph::admit_graph(&legacy_diamond()));
    let compatibility_plan = must(choregraph_seaglass::admit_graph(&choregraph_diamond()));
    let mut legacy_states = BTreeMap::new();
    let mut compatibility_states = BTreeMap::new();

    assert_eq!(
        choregraph_seaglass::ready_nodes(&compatibility_plan, &compatibility_states),
        legacy_execution_graph::ready_nodes(&legacy_plan, &legacy_states)
    );
    assert_eq!(
        choregraph_seaglass::resume_decision(&compatibility_plan, &compatibility_states, NODE_B),
        compatibility_resume(legacy_execution_graph::resume_decision(
            &legacy_plan,
            &legacy_states,
            NODE_B,
        ))
    );

    legacy_states.insert(
        NODE_A.to_string(),
        legacy_execution_graph::StepState::Completed,
    );
    compatibility_states.insert(
        NODE_A.to_string(),
        choregraph_seaglass::StepState::Completed,
    );
    assert_eq!(
        choregraph_seaglass::ready_nodes(&compatibility_plan, &compatibility_states),
        legacy_execution_graph::ready_nodes(&legacy_plan, &legacy_states)
    );
    assert_eq!(
        choregraph_seaglass::resume_decision(&compatibility_plan, &compatibility_states, NODE_B),
        compatibility_resume(legacy_execution_graph::resume_decision(
            &legacy_plan,
            &legacy_states,
            NODE_B,
        ))
    );

    legacy_states.insert(
        NODE_A.to_string(),
        legacy_execution_graph::StepState::Failed,
    );
    compatibility_states.insert(NODE_A.to_string(), choregraph_seaglass::StepState::Failed);
    assert_eq!(
        choregraph_seaglass::ready_nodes(&compatibility_plan, &compatibility_states),
        legacy_execution_graph::ready_nodes(&legacy_plan, &legacy_states)
    );
    assert_eq!(
        choregraph_seaglass::resume_decision(&compatibility_plan, &compatibility_states, NODE_B),
        compatibility_resume(legacy_execution_graph::resume_decision(
            &legacy_plan,
            &legacy_states,
            NODE_B,
        ))
    );
}

#[test]
fn aggregate_steps_delegate_without_effect_authority() {
    let plan = must(choregraph_seaglass::admit_graph(&choregraph_diamond()));
    let initial = plan.machine().initial_state();
    assert!(choregraph_core::invariant(plan.machine(), &initial));
    let decision = choregraph_seaglass::step_machine(
        &plan,
        &initial,
        NODE_A,
        choregraph_seaglass::StepAction::MarkReady,
    );
    assert!(matches!(
        decision,
        choregraph_seaglass::MachineDecision::Accept { .. }
    ));
    let choregraph_seaglass::MachineDecision::Accept { next } = decision else {
        return;
    };
    let node_id = must(choregraph_core::NodeId::parse(
        NODE_A,
        plan.machine().limits().max_identifier_bytes,
    ));
    assert_eq!(
        next.state(&node_id),
        Some(choregraph_core::LocalState::Ready)
    );
}

#[test]
fn stricter_identifier_state_and_unknown_node_cases_fail_closed() {
    let legacy_graph = legacy_execution_graph::ExecutionGraph::new(
        vec![legacy_execution_graph::ExecutionNode::new(
            INVALID_PROFILE_NODE,
        )],
        Vec::new(),
    );
    let compatibility_graph = choregraph_seaglass::ExecutionGraph::new(
        vec![choregraph_seaglass::ExecutionNode::new(
            INVALID_PROFILE_NODE,
        )],
        Vec::new(),
    );
    assert!(legacy_execution_graph::admit_graph(&legacy_graph).is_ok());
    assert!(
        choregraph_seaglass::validate_graph(&compatibility_graph)
            .iter()
            .any(|item| item.code == choregraph_seaglass::GraphDiagnosticCode::GraphTooLarge)
    );

    let legacy_plan = must(legacy_execution_graph::admit_graph(&legacy_diamond()));
    let compatibility_plan = must(choregraph_seaglass::admit_graph(&choregraph_diamond()));
    assert_eq!(
        legacy_execution_graph::resume_decision(&legacy_plan, &BTreeMap::new(), MISSING_NODE),
        legacy_execution_graph::ResumeDecision::Execute
    );
    assert_eq!(
        choregraph_seaglass::resume_decision(&compatibility_plan, &BTreeMap::new(), MISSING_NODE),
        choregraph_seaglass::ResumeDecision::RejectTerminalMutation
    );
    assert_eq!(
        choregraph_seaglass::resume_decision_checked(
            &compatibility_plan,
            &BTreeMap::new(),
            MISSING_NODE,
        ),
        Err(choregraph_seaglass::ResumeDecisionError::UnknownNode)
    );

    let inconsistent_states = BTreeMap::from([(
        NODE_D.to_string(),
        choregraph_seaglass::StepState::Completed,
    )]);
    assert!(choregraph_seaglass::ready_nodes(&compatibility_plan, &inconsistent_states).is_empty());
    assert!(matches!(
        choregraph_seaglass::ready_nodes_checked(&compatibility_plan, &inconsistent_states),
        Err(choregraph_seaglass::StateConversionError::InvalidState(_))
    ));
}
