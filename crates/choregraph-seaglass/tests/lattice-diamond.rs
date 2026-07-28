#![allow(
    unknown_lints,
    missing_const_fn,
    non_trait_imports,
    reason = "the conformance fixture uses runtime test-harness entry points and explicit public boundary imports"
)]

// r[verify choregraph.conformance.lattice]
use choregraph_core::{
    Action, AdmittedMachine, Decision, DenialReason, EffectRequest, Event, LocalState,
    MachineState, ready_nodes, step,
};
use choregraph_nickel::{LoweredSourceFormat, lower_evaluated_json, reviewed_frontend_limits};

const DIAMOND_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/lattice-diamond.json");
const NODE_ROOT: &str = "lattice-node/1";
const NODE_LEFT: &str = "lattice-node/2";
const NODE_RIGHT: &str = "lattice-node/3";
const NODE_SINK: &str = "lattice-node/4";
const EXPECTED_NODE_COUNT: usize = 4;
const EXPECTED_EDGE_COUNT: usize = 4;
const EMPTY_INTENT_COUNT: usize = 0;
const UNDECLARED_OPERATION: &str = "lattice/root";
const UNDECLARED_RESOURCE: &str = "root/work";
const UNDECLARED_ABILITY: &str = "file/write";
const UNDECLARED_REQUEST: &str = "request-1";
const EMPTY_BLAKE3_REFERENCE: &str =
    "blake3:0000000000000000000000000000000000000000000000000000000000000000";

#[allow(
    unknown_lints,
    no_panic,
    reason = "a conformance test must stop when a positive boundary unexpectedly rejects input"
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
    reason = "a conformance test must stop when a reviewed transition is denied"
)]
fn accept_without_intents(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
) -> MachineState {
    match step(machine, state, event) {
        Decision::Accept { next, intents } => {
            assert_eq!(intents.len(), EMPTY_INTENT_COUNT);
            next
        }
        Decision::Deny { reason } => panic!("expected acceptance, got {reason:?}"),
    }
}

fn complete_node(machine: &AdmittedMachine, state: &MachineState, node_id: &str) -> MachineState {
    let ready = accept_without_intents(machine, state, &Event::new(node_id, Action::MarkReady));
    let running = accept_without_intents(machine, &ready, &Event::new(node_id, Action::Start));
    accept_without_intents(machine, &running, &Event::new(node_id, Action::Complete))
}

fn ready_names(machine: &AdmittedMachine, state: &MachineState) -> Vec<String> {
    must(ready_nodes(machine, state))
        .iter()
        .map(|node_id| String::from(node_id.as_str()))
        .collect()
}

#[test]
fn lowers_and_executes_the_reviewed_lattice_diamond() {
    let lowered = must(lower_evaluated_json(
        DIAMOND_JSON,
        &[],
        reviewed_frontend_limits(),
    ));
    assert_eq!(
        lowered.source_format,
        LoweredSourceFormat::LatticeWorkflowV1
    );
    assert_eq!(lowered.machine.node_ids().len(), EXPECTED_NODE_COUNT);
    assert_eq!(lowered.machine.edges().len(), EXPECTED_EDGE_COUNT);
    assert!(lowered.machine.effect_declarations().is_empty());

    let machine = lowered.machine;
    let mut state = machine.initial_state();
    assert_eq!(ready_names(&machine, &state), [NODE_ROOT]);

    state = complete_node(&machine, &state, NODE_ROOT);
    assert_eq!(ready_names(&machine, &state), [NODE_LEFT, NODE_RIGHT]);

    state = complete_node(&machine, &state, NODE_LEFT);
    assert_eq!(ready_names(&machine, &state), [NODE_RIGHT]);

    state = complete_node(&machine, &state, NODE_RIGHT);
    assert_eq!(ready_names(&machine, &state), [NODE_SINK]);

    state = complete_node(&machine, &state, NODE_SINK);
    assert!(ready_names(&machine, &state).is_empty());
    assert!(
        state
            .entries()
            .values()
            .all(|local_state| *local_state == LocalState::Completed)
    );

    assert_eq!(
        step(&machine, &state, &Event::new(NODE_ROOT, Action::Cancel),),
        Decision::Deny {
            reason: DenialReason::TerminalState {
                from: LocalState::Completed,
                action: Action::Cancel,
            },
        }
    );
}

#[test]
fn rejects_premature_sink_readiness_and_undeclared_effects() {
    let lowered = must(lower_evaluated_json(
        DIAMOND_JSON,
        &[],
        reviewed_frontend_limits(),
    ));
    let machine = lowered.machine;
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

    let mut effect_event = Event::new(NODE_ROOT, Action::MarkReady);
    effect_event.effects.push(EffectRequest {
        operation: String::from(UNDECLARED_OPERATION),
        resource: String::from(UNDECLARED_RESOURCE),
        ability: String::from(UNDECLARED_ABILITY),
        request: String::from(UNDECLARED_REQUEST),
        payload: String::from(EMPTY_BLAKE3_REFERENCE),
    });
    assert_eq!(
        step(&machine, &initial, &effect_event),
        Decision::Deny {
            reason: DenialReason::UndeclaredEffect,
        }
    );
}
