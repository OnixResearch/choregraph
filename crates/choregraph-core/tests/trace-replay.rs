// The bounded fixture corpus keeps explicit imports, panic diagnostics, and fixed test vectors.
#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    function_length,
    non_trait_imports,
    no_panic,
    numeric_units,
    too_many_parameters,
    unbounded_collection_growth,
    reason = "the bounded fixture corpus keeps explicit imports, panic diagnostics, and fixed test vectors"
)]

// r[verify choregraph.proof.trace.model]
// r[verify choregraph.proof.trace.replay]
// r[verify choregraph.proof.trace.failure]
// r[verify choregraph.proof.trace.fixtures]
use std::collections::BTreeMap;

use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, BoundedTrace, ConsumedCount, Decision, DenialReason,
    EdgeSpec, EffectDeclarationSpec, EffectRequest, Event, EventIndex, LocalState, MachineSpec,
    MachineState, NodeSpec, ReplayBindings, ReplayDecision, ReplayFailureReason, ReplayInput,
    ReplayLimits, TraceBinding, TransitionSelector, invariant, replay, step,
};

const NODE_SOURCE: &str = "source";
const NODE_LEFT: &str = "left";
const NODE_RIGHT: &str = "right";
const NODE_SINK: &str = "sink";
const NODE_MISSING: &str = "missing";
const OPERATION_WRITE: &str = "file/write";
const RESOURCE_OUTPUT: &str = "root/output";
const ABILITY_WRITE: &str = "file/write";
const REQUEST_WRITE: &str = "request-1";
const PAYLOAD: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MACHINE_BINDING_BYTE: u8 = 0x11;
const STATE_BINDING_BYTE: u8 = 0x22;
const STALE_BINDING_BYTE: u8 = 0x33;
const TEST_MAX_EVENTS: u64 = 64;
const ONE_EVENT_LIMIT: u64 = 1;
const SINGLE_INTENT_COUNT: usize = 1;
const FIRST_EVENT: u64 = 0;
const SECOND_EVENT: u64 = 1;
const THIRD_EVENT: u64 = 2;
const NO_EVENTS: u64 = 0;

fn node(id: &str) -> NodeSpec {
    NodeSpec::new(id, Vec::new())
}

fn admit(spec: MachineSpec) -> AdmittedMachine {
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("test machine admission failed: {diagnostics:?}"),
    }
}

fn linear_machine() -> AdmittedMachine {
    admit(MachineSpec::new(vec![node(NODE_SOURCE)], Vec::new()))
}

fn diamond_machine() -> AdmittedMachine {
    admit(MachineSpec::new(
        vec![
            node(NODE_SOURCE),
            node(NODE_LEFT),
            node(NODE_RIGHT),
            node(NODE_SINK),
        ],
        vec![
            EdgeSpec::new(NODE_SOURCE, NODE_LEFT),
            EdgeSpec::new(NODE_SOURCE, NODE_RIGHT),
            EdgeSpec::new(NODE_LEFT, NODE_SINK),
            EdgeSpec::new(NODE_RIGHT, NODE_SINK),
        ],
    ))
}

fn effect_machine() -> AdmittedMachine {
    let mut spec = MachineSpec::new(
        vec![NodeSpec::new(
            NODE_SOURCE,
            vec![String::from(OPERATION_WRITE)],
        )],
        Vec::new(),
    );
    spec.effects.push(EffectDeclarationSpec {
        node_id: String::from(NODE_SOURCE),
        transition: TransitionSelector {
            from: LocalState::Ready,
            action: Action::Start,
        },
        operation: String::from(OPERATION_WRITE),
        resource: String::from(RESOURCE_OUTPUT),
        ability: String::from(ABILITY_WRITE),
        request: String::from(REQUEST_WRITE),
        payload: String::from(PAYLOAD),
    });
    admit(spec)
}

fn bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([MACHINE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        TraceBinding::from_bytes([STATE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
    )
}

fn stale_machine_bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([STALE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        bindings().initial_state,
    )
}

fn stale_state_bindings() -> ReplayBindings {
    ReplayBindings::new(
        bindings().machine,
        TraceBinding::from_bytes([STALE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
    )
}

fn count(value: usize) -> ConsumedCount {
    match u64::try_from(value) {
        Ok(value) => ConsumedCount::new(value),
        Err(_) => panic!("test count does not fit u64"),
    }
}

fn index(value: usize) -> EventIndex {
    match u64::try_from(value) {
        Ok(value) => EventIndex::new(value),
        Err(_) => panic!("test index does not fit u64"),
    }
}

fn replay_events(
    machine: &AdmittedMachine,
    initial_state: &MachineState,
    events: &[Event],
    trace_bindings: ReplayBindings,
    expected_bindings: ReplayBindings,
    max_events: u64,
) -> ReplayDecision {
    replay(ReplayInput::new(
        machine,
        initial_state,
        expected_bindings,
        BoundedTrace::new(trace_bindings, events),
        ReplayLimits { max_events },
    ))
}

fn event(node_id: &str, action: Action) -> Event {
    Event::new(node_id, action)
}

fn completion_events(node_id: &str) -> Vec<Event> {
    vec![
        event(node_id, Action::MarkReady),
        event(node_id, Action::Start),
        event(node_id, Action::Complete),
    ]
}

fn diamond_events() -> Vec<Event> {
    let mut events = completion_events(NODE_SOURCE);
    events.extend(completion_events(NODE_LEFT));
    events.extend(completion_events(NODE_RIGHT));
    events.extend(completion_events(NODE_SINK));
    events
}

fn write_request() -> EffectRequest {
    EffectRequest {
        operation: String::from(OPERATION_WRITE),
        resource: String::from(RESOURCE_OUTPUT),
        ability: String::from(ABILITY_WRITE),
        request: String::from(REQUEST_WRITE),
        payload: String::from(PAYLOAD),
    }
}

#[test]
fn empty_trace_preserves_the_initial_state() {
    let machine = linear_machine();
    let initial_state = machine.initial_state();
    let decision = replay_events(
        &machine,
        &initial_state,
        &[],
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );

    assert_eq!(
        decision,
        ReplayDecision::Complete {
            final_state: initial_state,
            consumed: ConsumedCount::new(NO_EVENTS),
        }
    );
}

#[test]
fn linear_and_terminal_traces_end_in_invariant_states() {
    let machine = linear_machine();
    let initial_state = machine.initial_state();
    let completed = completion_events(NODE_SOURCE);
    let completed_decision = replay_events(
        &machine,
        &initial_state,
        &completed,
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );
    let cancelled = [event(NODE_SOURCE, Action::Cancel)];
    let cancelled_decision = replay_events(
        &machine,
        &initial_state,
        &cancelled,
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );

    match completed_decision {
        ReplayDecision::Complete {
            final_state,
            consumed,
        } => {
            assert_eq!(consumed, count(completed.len()));
            assert!(invariant(&machine, &final_state));
        }
        ReplayDecision::Failed(failure) => panic!("valid linear trace failed: {failure:?}"),
    }
    match cancelled_decision {
        ReplayDecision::Complete {
            final_state,
            consumed,
        } => {
            assert_eq!(consumed, count(cancelled.len()));
            assert!(invariant(&machine, &final_state));
        }
        ReplayDecision::Failed(failure) => panic!("valid terminal trace failed: {failure:?}"),
    }
}

#[test]
fn diamond_trace_respects_declared_dependency_order() {
    let machine = diamond_machine();
    let initial_state = machine.initial_state();
    let events = diamond_events();
    let decision = replay_events(
        &machine,
        &initial_state,
        &events,
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );

    match decision {
        ReplayDecision::Complete {
            final_state,
            consumed,
        } => {
            assert_eq!(consumed, count(events.len()));
            assert!(invariant(&machine, &final_state));
            assert!(
                final_state
                    .entries()
                    .values()
                    .all(|state| *state == LocalState::Completed)
            );
        }
        ReplayDecision::Failed(failure) => panic!("valid diamond trace failed: {failure:?}"),
    }
}

#[test]
fn denied_trace_stops_at_the_first_failure() {
    let machine = linear_machine();
    let initial_state = machine.initial_state();
    let events = [
        event(NODE_SOURCE, Action::MarkReady),
        event(NODE_SOURCE, Action::Start),
        event(NODE_SOURCE, Action::Start),
        event(NODE_SOURCE, Action::Complete),
    ];
    let decision = replay_events(
        &machine,
        &initial_state,
        &events,
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );

    match decision {
        ReplayDecision::Failed(failure) => {
            assert_eq!(failure.event_index, Some(EventIndex::new(THIRD_EVENT)));
            assert_eq!(failure.consumed, ConsumedCount::new(THIRD_EVENT));
            assert_eq!(
                failure.reason,
                ReplayFailureReason::TransitionDenied(DenialReason::InvalidTransition {
                    from: LocalState::Running,
                    action: Action::Start,
                })
            );
            assert_eq!(
                failure
                    .last_valid_state
                    .state()
                    .entries()
                    .values()
                    .copied()
                    .collect::<Vec<_>>(),
                vec![LocalState::Running]
            );
        }
        ReplayDecision::Complete { final_state, .. } => {
            panic!("invalid trace completed: {final_state:?}")
        }
    }
}

#[test]
fn replay_discards_historical_effect_intents() {
    let machine = effect_machine();
    let initial_state = machine.initial_state();
    let ready = match step(
        &machine,
        &initial_state,
        &event(NODE_SOURCE, Action::MarkReady),
    ) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            next
        }
        Decision::Deny { reason } => panic!("mark-ready denied: {reason:?}"),
    };
    let mut start = event(NODE_SOURCE, Action::Start);
    start.effects.push(write_request());
    match step(&machine, &ready, &start) {
        Decision::Accept { intents, .. } => assert_eq!(intents.len(), SINGLE_INTENT_COUNT),
        Decision::Deny { reason } => panic!("effectful start denied: {reason:?}"),
    }

    let events = [event(NODE_SOURCE, Action::MarkReady), start];
    let replayed = replay_events(
        &machine,
        &initial_state,
        &events,
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );
    assert!(matches!(replayed, ReplayDecision::Complete { .. }));
}

#[test]
fn malformed_unknown_invalid_terminal_and_reordered_traces_fail_exactly() {
    let linear = linear_machine();
    let linear_initial = linear.initial_state();
    let malformed = [event("bad node", Action::MarkReady)];
    let unknown = [event(NODE_MISSING, Action::MarkReady)];
    let invalid = [event(NODE_SOURCE, Action::Complete)];
    let terminal = [
        event(NODE_SOURCE, Action::Cancel),
        event(NODE_SOURCE, Action::MarkReady),
    ];
    let diamond = diamond_machine();
    let diamond_initial = diamond.initial_state();
    let reordered = [event(NODE_SINK, Action::MarkReady)];

    let cases = [
        (
            replay_events(
                &linear,
                &linear_initial,
                &malformed,
                bindings(),
                bindings(),
                TEST_MAX_EVENTS,
            ),
            FIRST_EVENT,
            DenialReason::InvalidNodeId,
        ),
        (
            replay_events(
                &linear,
                &linear_initial,
                &unknown,
                bindings(),
                bindings(),
                TEST_MAX_EVENTS,
            ),
            FIRST_EVENT,
            DenialReason::UnknownNode,
        ),
        (
            replay_events(
                &linear,
                &linear_initial,
                &invalid,
                bindings(),
                bindings(),
                TEST_MAX_EVENTS,
            ),
            FIRST_EVENT,
            DenialReason::InvalidTransition {
                from: LocalState::Pending,
                action: Action::Complete,
            },
        ),
        (
            replay_events(
                &linear,
                &linear_initial,
                &terminal,
                bindings(),
                bindings(),
                TEST_MAX_EVENTS,
            ),
            SECOND_EVENT,
            DenialReason::TerminalState {
                from: LocalState::Cancelled,
                action: Action::MarkReady,
            },
        ),
        (
            replay_events(
                &diamond,
                &diamond_initial,
                &reordered,
                bindings(),
                bindings(),
                TEST_MAX_EVENTS,
            ),
            FIRST_EVENT,
            DenialReason::DependenciesIncomplete,
        ),
    ];

    for (decision, expected_index, expected_reason) in cases {
        match decision {
            ReplayDecision::Failed(failure) => {
                assert_eq!(failure.event_index, Some(EventIndex::new(expected_index)));
                assert_eq!(
                    failure.reason,
                    ReplayFailureReason::TransitionDenied(expected_reason)
                );
            }
            ReplayDecision::Complete { final_state, .. } => {
                panic!("invalid trace completed: {final_state:?}")
            }
        }
    }
}

#[test]
fn oversized_and_stale_bound_traces_fail_before_processing() {
    let machine = linear_machine();
    let initial_state = machine.initial_state();
    let events = completion_events(NODE_SOURCE);
    let oversized = replay_events(
        &machine,
        &initial_state,
        &events,
        bindings(),
        bindings(),
        ONE_EVENT_LIMIT,
    );
    let stale_machine = replay_events(
        &machine,
        &initial_state,
        &events,
        stale_machine_bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );
    let stale_state = replay_events(
        &machine,
        &initial_state,
        &events,
        stale_state_bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );

    let expected = [
        ReplayFailureReason::TraceLimitExceeded {
            observed: count(events.len()).get(),
            maximum: ONE_EVENT_LIMIT,
        },
        ReplayFailureReason::MachineBindingMismatch,
        ReplayFailureReason::InitialStateBindingMismatch,
    ];
    for (decision, reason) in [oversized, stale_machine, stale_state]
        .into_iter()
        .zip(expected)
    {
        match decision {
            ReplayDecision::Failed(failure) => {
                assert_eq!(failure.event_index, None);
                assert_eq!(failure.consumed, ConsumedCount::new(NO_EVENTS));
                assert_eq!(failure.reason, reason);
                assert_eq!(failure.last_valid_state.state(), &initial_state);
            }
            ReplayDecision::Complete { final_state, .. } => {
                panic!("inadmissible trace completed: {final_state:?}")
            }
        }
    }
}

#[test]
fn invalid_initial_state_fails_before_processing() {
    let machine = diamond_machine();
    let initial_state = MachineState::new(BTreeMap::new());
    let events = diamond_events();
    let decision = replay_events(
        &machine,
        &initial_state,
        &events,
        bindings(),
        bindings(),
        TEST_MAX_EVENTS,
    );

    match decision {
        ReplayDecision::Failed(failure) => {
            assert_eq!(failure.event_index, None);
            assert_eq!(failure.consumed, ConsumedCount::new(NO_EVENTS));
            assert_eq!(failure.reason, ReplayFailureReason::InitialStateInvalid);
            assert_eq!(failure.last_valid_state.state(), &initial_state);
        }
        ReplayDecision::Complete { final_state, .. } => {
            panic!("invalid initial state completed: {final_state:?}")
        }
    }
}

fn repeated_step_prefix(
    machine: &AdmittedMachine,
    initial_state: &MachineState,
    events: &[Event],
) -> (MachineState, usize, Option<(usize, DenialReason)>) {
    let mut state = initial_state.clone();
    for (event_index, replay_event) in events.iter().enumerate() {
        match step(machine, &state, replay_event) {
            Decision::Accept { next, intents: _ } => state = next,
            Decision::Deny { reason } => return (state, event_index, Some((event_index, reason))),
        }
    }
    (state, events.len(), None)
}

#[test]
fn property_generated_traces_match_repeated_production_steps() {
    let machine = diamond_machine();
    let initial_state = machine.initial_state();
    let full_trace = diamond_events();
    let mut traces = Vec::new();
    for prefix_length in 0..=full_trace.len() {
        traces.push(full_trace[..prefix_length].to_vec());
    }
    for invalid_action in [
        Action::MarkReady,
        Action::Start,
        Action::Complete,
        Action::Fail,
        Action::Cancel,
    ] {
        let mut candidate = full_trace.clone();
        candidate.push(event(NODE_SOURCE, invalid_action));
        traces.push(candidate);
    }

    for trace in traces {
        let expected = repeated_step_prefix(&machine, &initial_state, &trace);
        let observed = replay_events(
            &machine,
            &initial_state,
            &trace,
            bindings(),
            bindings(),
            TEST_MAX_EVENTS,
        );
        match (observed, expected.2) {
            (
                ReplayDecision::Complete {
                    final_state,
                    consumed,
                },
                None,
            ) => {
                assert_eq!(final_state, expected.0);
                assert_eq!(consumed, count(expected.1));
            }
            (ReplayDecision::Failed(failure), Some((event_index, reason))) => {
                assert_eq!(failure.last_valid_state.state(), &expected.0);
                assert_eq!(failure.consumed, count(expected.1));
                assert_eq!(failure.event_index, Some(index(event_index)));
                assert_eq!(
                    failure.reason,
                    ReplayFailureReason::TransitionDenied(reason)
                );
            }
            (observed, expected_failure) => {
                panic!("replay mismatch: observed={observed:?} expected={expected_failure:?}")
            }
        }
    }
}
