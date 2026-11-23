//! Trace-validation checks replaying recorded Lattice workflow runs through the
//! production Choregraph kernel (`transition::step` and `transition::replay`).
//!
//! Each check verifies that every recorded step is an allowed kernel transition,
//! that the Verus ranking functions (`non_terminal_count` and
//! `linear_steps_remaining`) decrease as proved, and that malformed or
//! out-of-order recorded runs are rejected at the exact offending step index.

const DIAMOND_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/lattice-diamond.json");
const NODE_ROOT: &str = "lattice-node/1";
const NODE_LEFT: &str = "lattice-node/2";
const NODE_RIGHT: &str = "lattice-node/3";
const NODE_SINK: &str = "lattice-node/4";
const WAIT_CLASS: &str = "lattice/manual-gate";
const WAIT_REFERENCE: &str =
    "blake3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const TRACE_BINDING_BYTE: u8 = 0x3c;
const TRACE_LIMIT_COUNT: u64 = 64;
const EXPECTED_DIAMOND_STEPS_COUNT: usize = 12;
const EXPECTED_WAIT_STEPS_COUNT: usize = 8;
const EXPECTED_FAILURE_STEPS_COUNT: usize = 10;

const fn is_terminal_state(state: choregraph_core::model::semantics::LocalState) -> bool {
    matches!(
        state,
        choregraph_core::model::semantics::LocalState::Completed
            | choregraph_core::model::semantics::LocalState::Failed
            | choregraph_core::model::semantics::LocalState::Cancelled
    )
}

const fn linear_node_rank(state: choregraph_core::model::semantics::LocalState) -> usize {
    match state {
        choregraph_core::model::semantics::LocalState::Pending => 3,
        choregraph_core::model::semantics::LocalState::Ready
        | choregraph_core::model::semantics::LocalState::Suspended => 2,
        choregraph_core::model::semantics::LocalState::Running => 1,
        choregraph_core::model::semantics::LocalState::Completed
        | choregraph_core::model::semantics::LocalState::Failed
        | choregraph_core::model::semantics::LocalState::Cancelled => 0,
    }
}

const fn is_terminal_action(action: choregraph_core::model::semantics::Action) -> bool {
    matches!(
        action,
        choregraph_core::model::semantics::Action::Complete
            | choregraph_core::model::semantics::Action::Fail
            | choregraph_core::model::semantics::Action::Cancel
    )
}

const fn is_linear_progress_action(action: choregraph_core::model::semantics::Action) -> bool {
    matches!(
        action,
        choregraph_core::model::semantics::Action::MarkReady
            | choregraph_core::model::semantics::Action::Start
            | choregraph_core::model::semantics::Action::Complete
            | choregraph_core::model::semantics::Action::Fail
            | choregraph_core::model::semantics::Action::Cancel
    )
}

fn non_terminal_count(state: &choregraph_core::model::automaton::MachineState) -> usize {
    let count = state
        .entries()
        .values()
        .filter(|local_state| !is_terminal_state(**local_state))
        .count();
    debug_assert!(count <= state.entries().len());
    debug_assert!(
        (count == 0)
            == state
                .entries()
                .values()
                .all(|local_state| is_terminal_state(*local_state))
    );
    count
}

fn linear_steps_remaining(state: &choregraph_core::model::automaton::MachineState) -> usize {
    let total = state
        .entries()
        .values()
        .map(|local_state| linear_node_rank(*local_state))
        .sum();
    debug_assert!(total <= state.entries().len().saturating_mul(3));
    debug_assert!((total == 0) == (non_terminal_count(state) == 0));
    total
}

fn replay_bindings() -> choregraph_core::model::history::ReplayBindings {
    let digest = choregraph_core::model::history::TraceBinding::from_bytes(
        [TRACE_BINDING_BYTE; choregraph_core::model::history::TRACE_BINDING_BYTES],
    );
    choregraph_core::model::history::ReplayBindings::new(digest, digest)
}

fn replay_limits() -> choregraph_core::model::history::ReplayLimits {
    choregraph_core::model::history::ReplayLimits {
        max_events: TRACE_LIMIT_COUNT,
    }
}

fn lattice_diamond_machine() -> Result<choregraph_core::model::automaton::AdmittedMachine, String> {
    let lowered = choregraph_nickel::lower::lower_evaluated_json(
        DIAMOND_JSON,
        &[],
        choregraph_nickel::profile::reviewed_frontend_limits(),
    )
    .map_err(|error| format!("lower lattice diamond: {error:?}"))?;
    debug_assert_eq!(lowered.machine.node_ids().len(), 4);
    debug_assert_eq!(lowered.machine.edges().len(), 4);
    Ok(lowered.machine)
}

fn lattice_wait_machine() -> Result<choregraph_core::model::automaton::AdmittedMachine, String> {
    let mut spec = choregraph_core::model::automaton::MachineSpec::new(
        vec![
            choregraph_core::model::graph::NodeSpec::new(NODE_ROOT, Vec::new()),
            choregraph_core::model::graph::NodeSpec::new(NODE_LEFT, Vec::new()),
        ],
        vec![choregraph_core::model::graph::EdgeSpec::new(
            NODE_ROOT, NODE_LEFT,
        )],
    );
    spec.lifecycle_profile = String::from(choregraph_core::model::semantics::DURABLE_SUSPENSION_V1);
    let limits = choregraph_core::model::capacity::AdmissionLimits {
        max_nodes: 16,
        max_edges: 32,
        max_operations: 32,
        max_effect_declarations: 16,
        max_identifier_bytes: 256,
        max_reference_bytes: choregraph_core::id::reference::BLAKE3_REFERENCE_BYTES,
        max_event_bytes: 65_536,
        max_intent_bytes: 2_048,
        max_intents_per_transition: 8,
    };
    let machine = choregraph_core::model::automaton::AdmittedMachine::admit(spec, limits)
        .map_err(|diagnostics| format!("admit lattice wait machine: {diagnostics:?}"))?;
    debug_assert_eq!(machine.node_ids().len(), 2);
    debug_assert_eq!(machine.edges().len(), 1);
    Ok(machine)
}

fn replay_recorded_trace(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
    events: &[choregraph_core::model::dispatch::Event],
) -> choregraph_core::model::history::ReplayDecision {
    let initial = machine.initial_state();
    let bindings = replay_bindings();
    let trace = choregraph_core::model::history::BoundedTrace::new(bindings, events);
    let input = choregraph_core::model::history::ReplayInput::new(
        machine,
        &initial,
        bindings,
        trace,
        replay_limits(),
    );
    choregraph_core::transition::replay(input)
}

fn validate_recorded_run(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
    events: &[choregraph_core::model::dispatch::Event],
) -> Result<choregraph_core::model::automaton::MachineState, String> {
    let mut state = machine.initial_state();
    for (step_index, event) in events.iter().enumerate() {
        let before_non_terminal = non_terminal_count(&state);
        let before_rank = linear_steps_remaining(&state);
        match choregraph_core::transition::step(machine, &state, event) {
            choregraph_core::model::outcome::Decision::Accept { next, .. } => {
                let after_non_terminal = non_terminal_count(&next);
                let after_rank = linear_steps_remaining(&next);
                if after_non_terminal > before_non_terminal {
                    return Err(format!(
                        "step {step_index} increased non-terminal count: {before_non_terminal} -> {after_non_terminal}"
                    ));
                }
                if is_terminal_action(event.action)
                    && after_non_terminal.saturating_add(1) != before_non_terminal
                {
                    return Err(format!(
                        "terminal step {step_index} failed to decrease non-terminal count by 1"
                    ));
                }
                if is_linear_progress_action(event.action) && after_rank >= before_rank {
                    return Err(format!(
                        "linear progress step {step_index} failed to strictly decrease rank: {before_rank} -> {after_rank}"
                    ));
                }
                state = next;
            }
            choregraph_core::model::outcome::Decision::Deny { reason } => {
                return Err(format!("recorded step {step_index} denied: {reason:?}"));
            }
        }
    }
    match replay_recorded_trace(machine, events) {
        choregraph_core::model::history::ReplayDecision::Complete {
            final_state,
            consumed,
        } => {
            let consumed_count = consumed.get();
            if usize::try_from(consumed_count) != Ok(events.len()) {
                return Err(format!(
                    "replay consumed {consumed_count}, expected {}",
                    events.len()
                ));
            }
            if final_state != state {
                return Err(String::from("replay final state differed from live step"));
            }
            debug_assert_eq!(final_state, state);
            debug_assert_eq!(usize::try_from(consumed_count), Ok(events.len()));
            Ok(final_state)
        }
        choregraph_core::model::history::ReplayDecision::Failed(failure) => Err(format!(
            "replay rejected recorded run at {:?}: {:?}",
            failure.event_index, failure.reason
        )),
    }
}

#[test]
fn validates_recorded_lattice_diamond_parallel_completion_run() -> Result<(), String> {
    let machine = lattice_diamond_machine()?;
    let recorded_events = vec![
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_RIGHT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_RIGHT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_RIGHT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_SINK,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_SINK,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_SINK,
            choregraph_core::model::semantics::Action::Complete,
        ),
    ];
    assert_eq!(recorded_events.len(), EXPECTED_DIAMOND_STEPS_COUNT);
    let final_state = validate_recorded_run(&machine, &recorded_events)?;
    assert_eq!(non_terminal_count(&final_state), 0);
    assert_eq!(linear_steps_remaining(&final_state), 0);
    Ok(())
}

#[test]
fn validates_recorded_lattice_durable_wait_resume_and_completion_run() -> Result<(), String> {
    let machine = lattice_wait_machine()?;
    let recorded_events = vec![
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::suspension(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Suspend,
            WAIT_CLASS,
            WAIT_REFERENCE,
        ),
        choregraph_core::model::dispatch::Event::suspension(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::ResumeSignal,
            WAIT_CLASS,
            WAIT_REFERENCE,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Complete,
        ),
    ];
    assert_eq!(recorded_events.len(), EXPECTED_WAIT_STEPS_COUNT);
    let final_state = validate_recorded_run(&machine, &recorded_events)?;
    assert_eq!(non_terminal_count(&final_state), 0);
    assert_eq!(linear_steps_remaining(&final_state), 0);
    Ok(())
}

#[test]
fn validates_recorded_lattice_failure_with_downstream_cancellation_run() -> Result<(), String> {
    let machine = lattice_diamond_machine()?;
    let recorded_events = vec![
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Fail,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_RIGHT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_RIGHT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_RIGHT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_SINK,
            choregraph_core::model::semantics::Action::Cancel,
        ),
    ];
    assert_eq!(recorded_events.len(), EXPECTED_FAILURE_STEPS_COUNT);
    let final_state = validate_recorded_run(&machine, &recorded_events)?;
    assert_eq!(non_terminal_count(&final_state), 0);
    assert_eq!(linear_steps_remaining(&final_state), 0);
    Ok(())
}

#[test]
fn rejects_recorded_lattice_run_with_premature_sink_ready_at_exact_index() -> Result<(), String> {
    let machine = lattice_diamond_machine()?;
    let invalid_events = vec![
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_LEFT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_SINK,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
    ];
    match replay_recorded_trace(&machine, &invalid_events) {
        choregraph_core::model::history::ReplayDecision::Failed(failure) => {
            assert_eq!(
                failure.event_index,
                Some(choregraph_core::model::history::EventIndex::new(6))
            );
            assert_eq!(
                failure.consumed,
                choregraph_core::model::history::ConsumedCount::new(6)
            );
            assert_eq!(
                failure.reason,
                choregraph_core::model::history::ReplayFailureReason::TransitionDenied(
                    choregraph_core::model::outcome::DenialReason::DependenciesIncomplete
                )
            );
            Ok(())
        }
        choregraph_core::model::history::ReplayDecision::Complete { .. } => {
            Err(String::from("premature sink readiness unexpectedly passed"))
        }
    }
}

#[test]
fn rejects_recorded_lattice_run_with_post_terminal_mutation_at_exact_index() -> Result<(), String> {
    let machine = lattice_diamond_machine()?;
    let invalid_events = vec![
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::MarkReady,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Start,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Complete,
        ),
        choregraph_core::model::dispatch::Event::new(
            NODE_ROOT,
            choregraph_core::model::semantics::Action::Cancel,
        ),
    ];
    match replay_recorded_trace(&machine, &invalid_events) {
        choregraph_core::model::history::ReplayDecision::Failed(failure) => {
            assert_eq!(
                failure.event_index,
                Some(choregraph_core::model::history::EventIndex::new(3))
            );
            assert_eq!(
                failure.consumed,
                choregraph_core::model::history::ConsumedCount::new(3)
            );
            assert_eq!(
                failure.reason,
                choregraph_core::model::history::ReplayFailureReason::TransitionDenied(
                    choregraph_core::model::outcome::DenialReason::TerminalState {
                        from: choregraph_core::model::semantics::LocalState::Completed,
                        action: choregraph_core::model::semantics::Action::Cancel,
                    }
                )
            );
            Ok(())
        }
        choregraph_core::model::history::ReplayDecision::Complete { .. } => {
            Err(String::from("post-terminal mutation unexpectedly passed"))
        }
    }
}
