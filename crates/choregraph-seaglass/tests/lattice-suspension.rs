// Pinned Lattice downstream fixture for durable suspension decisions.
//
// This fixture mirrors how a Lattice generic wait node (pinned consumer
// revision c8d56ad83a57287064edb85f58808f53e4bf4536) converts through the
// adapter vocabulary and observes Choregraph decisions. It proves conversion
// and decision shape only, not Lattice approval behavior.
#![allow(
    unknown_lints,
    explicit_defaults,
    no_panic,
    non_trait_imports,
    numeric_units,
    reason = "the pinned integration fixture keeps domain notation and exact boundary failures readable"
)]

// r[verify choregraph.machine.suspension_replay]
// r[verify choregraph.machine.resume_signal]
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, BoundedTrace, DURABLE_SUSPENSION_V1, Decision,
    DenialReason, EdgeSpec, Event, LocalState, MachineSpec, NodeSpec, ReplayBindings,
    ReplayDecision, ReplayInput, ReplayLimits, ResumeDecision, TraceBinding, replay,
    resume_decision, step,
};

const LATTICE_WAIT_NODE: &str = "lattice-node/2";
const LATTICE_SOURCE_NODE: &str = "lattice-node/1";
const WAIT_CLASS: &str = "lattice/manual-gate";
const WAIT_REFERENCE: &str =
    "blake3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const STALE_REFERENCE: &str =
    "blake3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const BINDING_BYTE: u8 = 0x51;
const TRACE_EVENT_LIMIT: u64 = 16;

fn lattice_wait_machine() -> AdmittedMachine {
    let mut spec = MachineSpec::new(
        vec![
            NodeSpec::new(LATTICE_SOURCE_NODE, Vec::new()),
            NodeSpec::new(LATTICE_WAIT_NODE, Vec::new()),
        ],
        vec![EdgeSpec::new(LATTICE_SOURCE_NODE, LATTICE_WAIT_NODE)],
    );
    spec.lifecycle_profile = String::from(DURABLE_SUSPENSION_V1);
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("lattice wait machine admission failed: {diagnostics:?}"),
    }
}

fn node_id(machine: &AdmittedMachine, expected: &str) -> choregraph_core::NodeId {
    for node_id in machine.node_ids() {
        if node_id.as_str() == expected {
            return node_id.clone();
        }
    }
    panic!("admitted node is missing: {expected}")
}

fn drive_to_gate_ready(machine: &AdmittedMachine) -> choregraph_core::MachineState {
    let mut state = machine.initial_state();
    for event in [
        Event::new(LATTICE_SOURCE_NODE, Action::MarkReady),
        Event::new(LATTICE_SOURCE_NODE, Action::Start),
        Event::new(LATTICE_SOURCE_NODE, Action::Complete),
        Event::new(LATTICE_WAIT_NODE, Action::MarkReady),
    ] {
        match step(machine, &state, &event) {
            Decision::Accept { next, .. } => state = next,
            Decision::Deny { reason } => panic!("setup step denied: {reason:?}"),
        }
    }
    state
}

#[test]
fn lattice_wait_node_observes_suspended_wait_resume_and_denial_decisions() {
    let machine = lattice_wait_machine();
    let gate = node_id(&machine, LATTICE_WAIT_NODE);
    let ready = drive_to_gate_ready(&machine);

    let suspend = Event::suspension(
        LATTICE_WAIT_NODE,
        Action::Suspend,
        WAIT_CLASS,
        WAIT_REFERENCE,
    );
    let suspended = match step(&machine, &ready, &suspend) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            next
        }
        Decision::Deny { reason } => panic!("suspend denied: {reason:?}"),
    };
    assert_eq!(suspended.state(&gate), Some(LocalState::Suspended));

    // The wait decision: dependencies are complete, yet the node waits.
    assert_eq!(
        resume_decision(&machine, &suspended, &gate),
        ResumeDecision::AwaitSignal
    );

    // A mismatched signal cannot release the wait.
    let mismatched = Event::suspension(
        LATTICE_WAIT_NODE,
        Action::ResumeSignal,
        WAIT_CLASS,
        STALE_REFERENCE,
    );
    match step(&machine, &suspended, &mismatched) {
        Decision::Deny { reason } => assert_eq!(reason, DenialReason::SuspensionMismatch),
        Decision::Accept { .. } => panic!("mismatched signal released the wait node"),
    }
    assert_eq!(
        resume_decision(&machine, &suspended, &gate),
        ResumeDecision::AwaitSignal
    );

    // The one matching signal releases the node to ready without effects.
    let matching = Event::suspension(
        LATTICE_WAIT_NODE,
        Action::ResumeSignal,
        WAIT_CLASS,
        WAIT_REFERENCE,
    );
    match step(&machine, &suspended, &matching) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            assert_eq!(next.state(&gate), Some(LocalState::Ready));
        }
        Decision::Deny { reason } => panic!("matching resume denied: {reason:?}"),
    }
}

#[test]
fn lattice_wait_trace_replays_suspended_and_resumes_deterministically() {
    let machine = lattice_wait_machine();
    let initial = machine.initial_state();
    let events = vec![
        Event::new(LATTICE_SOURCE_NODE, Action::MarkReady),
        Event::new(LATTICE_SOURCE_NODE, Action::Start),
        Event::new(LATTICE_SOURCE_NODE, Action::Complete),
        Event::new(LATTICE_WAIT_NODE, Action::MarkReady),
        Event::suspension(
            LATTICE_WAIT_NODE,
            Action::Suspend,
            WAIT_CLASS,
            WAIT_REFERENCE,
        ),
    ];
    let bindings = ReplayBindings::new(
        TraceBinding::from_bytes([BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        TraceBinding::from_bytes([BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
    );
    let replay_limits = ReplayLimits {
        max_events: TRACE_EVENT_LIMIT,
    };
    let gate = node_id(&machine, LATTICE_WAIT_NODE);

    let suspended_replay = replay(ReplayInput::new(
        &machine,
        &initial,
        bindings,
        BoundedTrace::new(bindings, &events),
        replay_limits,
    ));
    match suspended_replay {
        ReplayDecision::Complete { final_state, .. } => {
            assert_eq!(final_state.state(&gate), Some(LocalState::Suspended));
        }
        ReplayDecision::Failed(failure) => panic!("suspended replay failed: {failure:?}"),
    }

    let mut extended = events.clone();
    extended.push(Event::suspension(
        LATTICE_WAIT_NODE,
        Action::ResumeSignal,
        WAIT_CLASS,
        WAIT_REFERENCE,
    ));
    let resumed_replay = replay(ReplayInput::new(
        &machine,
        &initial,
        bindings,
        BoundedTrace::new(bindings, &extended),
        replay_limits,
    ));
    match resumed_replay {
        ReplayDecision::Complete { final_state, .. } => {
            assert_eq!(final_state.state(&gate), Some(LocalState::Ready));
            assert!(final_state.suspension(&gate).is_none());
        }
        ReplayDecision::Failed(failure) => panic!("resumed replay failed: {failure:?}"),
    }
}
