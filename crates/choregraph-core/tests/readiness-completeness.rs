// Readiness completeness fixtures keep public production behavior explicit.
#![allow(
    unknown_lints,
    assertion_density,
    no_panic,
    non_trait_imports,
    unbounded_collection_growth,
    reason = "readiness fixtures exercise the public production graph and transition paths"
)]

// r[verify choregraph.proof.single_step.readiness_completeness_baseline]
// r[verify choregraph.proof.single_step.readiness_completeness]
// r[verify choregraph.proof.single_step.readiness_frontier]
// r[verify choregraph.proof.single_step.readiness_transition]
// r[verify choregraph.proof.single_step.readiness_completeness_negative]
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, DEFAULT_MAX_EDGES, DEFAULT_MAX_EFFECT_DECLARATIONS,
    DEFAULT_MAX_EVENT_BYTES, DEFAULT_MAX_IDENTIFIER_BYTES, DEFAULT_MAX_INTENT_BYTES,
    DEFAULT_MAX_INTENTS_PER_TRANSITION, DEFAULT_MAX_NODES, DEFAULT_MAX_OPERATIONS,
    DEFAULT_MAX_REFERENCE_BYTES, Decision, DenialReason, EdgeSpec, Event, MachineSpec,
    MachineState, NodeId, NodeSpec, ready_nodes, step,
};

const NODE_SOURCE: &str = "source";
const NODE_LEFT: &str = "left";
const NODE_RIGHT: &str = "right";
const NODE_SINK: &str = "sink";
const PARALLEL_READY_COUNT: usize = 2;

fn node(id: &str) -> NodeSpec {
    NodeSpec::new(id, Vec::new())
}

const fn limits() -> AdmissionLimits {
    AdmissionLimits {
        max_nodes: DEFAULT_MAX_NODES,
        max_edges: DEFAULT_MAX_EDGES,
        max_operations: DEFAULT_MAX_OPERATIONS,
        max_effect_declarations: DEFAULT_MAX_EFFECT_DECLARATIONS,
        max_identifier_bytes: DEFAULT_MAX_IDENTIFIER_BYTES,
        max_reference_bytes: DEFAULT_MAX_REFERENCE_BYTES,
        max_event_bytes: DEFAULT_MAX_EVENT_BYTES,
        max_intent_bytes: DEFAULT_MAX_INTENT_BYTES,
        max_intents_per_transition: DEFAULT_MAX_INTENTS_PER_TRANSITION,
    }
}

fn diamond_machine() -> AdmittedMachine {
    let spec = MachineSpec::new(
        vec![
            node(NODE_SINK),
            node(NODE_RIGHT),
            node(NODE_LEFT),
            node(NODE_SOURCE),
        ],
        vec![
            EdgeSpec::new(NODE_SOURCE, NODE_RIGHT),
            EdgeSpec::new(NODE_RIGHT, NODE_SINK),
            EdgeSpec::new(NODE_SOURCE, NODE_LEFT),
            EdgeSpec::new(NODE_LEFT, NODE_SINK),
        ],
    );
    match AdmittedMachine::admit(spec, limits()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("diamond admission failed: {diagnostics:?}"),
    }
}

fn accept(machine: &AdmittedMachine, state: &MachineState, event: Event) -> MachineState {
    match step(machine, state, &event) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            next
        }
        Decision::Deny { reason } => panic!("expected accepted transition, got {reason:?}"),
    }
}

fn complete(machine: &AdmittedMachine, state: &MachineState, node_id: &str) -> MachineState {
    let ready = accept(machine, state, Event::new(node_id, Action::MarkReady));
    let running = accept(machine, &ready, Event::new(node_id, Action::Start));
    accept(machine, &running, Event::new(node_id, Action::Complete))
}

fn names(nodes: Vec<NodeId>) -> Vec<String> {
    nodes
        .into_iter()
        .map(|node_id| String::from(node_id.as_str()))
        .collect()
}

#[test]
fn production_frontier_contains_every_eligible_node_once_in_topological_order() {
    let machine = diamond_machine();
    let initial = machine.initial_state();
    assert_eq!(
        ready_nodes(&machine, &initial).map(names),
        Ok(vec![String::from(NODE_SOURCE)])
    );

    let source_completed = complete(&machine, &initial, NODE_SOURCE);
    let parallel = match ready_nodes(&machine, &source_completed) {
        Ok(frontier) => frontier,
        Err(issues) => panic!("parallel frontier failed: {issues:?}"),
    };
    assert_eq!(parallel.len(), PARALLEL_READY_COUNT);
    assert_eq!(
        names(parallel),
        vec![String::from(NODE_LEFT), String::from(NODE_RIGHT)]
    );

    let left_completed = complete(&machine, &source_completed, NODE_LEFT);
    assert_eq!(
        ready_nodes(&machine, &left_completed).map(names),
        Ok(vec![String::from(NODE_RIGHT)])
    );
    let branches_completed = complete(&machine, &left_completed, NODE_RIGHT);
    assert_eq!(
        ready_nodes(&machine, &branches_completed).map(names),
        Ok(vec![String::from(NODE_SINK)])
    );
}

#[test]
fn mark_ready_accepts_exactly_when_the_pending_target_has_complete_dependencies() {
    let machine = diamond_machine();
    let initial = machine.initial_state();

    assert_eq!(
        step(
            &machine,
            &initial,
            &Event::new(NODE_SINK, Action::MarkReady),
        ),
        Decision::Deny {
            reason: DenialReason::DependenciesIncomplete,
        }
    );

    let source_completed = complete(&machine, &initial, NODE_SOURCE);
    match step(
        &machine,
        &source_completed,
        &Event::new(NODE_LEFT, Action::MarkReady),
    ) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            assert_eq!(
                ready_nodes(&machine, &next).map(names),
                Ok(vec![String::from(NODE_RIGHT)])
            );
        }
        Decision::Deny { reason } => panic!("eligible node was denied: {reason:?}"),
    }
}
