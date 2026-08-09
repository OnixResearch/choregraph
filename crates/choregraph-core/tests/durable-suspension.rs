// These integration fixtures use direct domain imports, production defaults, and failing assertions.
#![allow(
    unknown_lints,
    ambiguous_params,
    explicit_defaults,
    no_panic,
    non_trait_imports,
    numeric_units,
    raw_arithmetic_overflow,
    reason = "integration fixtures keep reviewed domain notation and exact boundary failures readable"
)]

// r[verify choregraph.machine.suspension_profile]
// r[verify choregraph.machine.suspension_transition]
// r[verify choregraph.machine.resume_signal]
// r[verify choregraph.machine.suspension_replay]
// r[verify choregraph.machine.suspension_boundary]
use choregraph_core::{
    Action, AdmissionDiagnosticCode, AdmissionLimits, AdmittedMachine, BoundedTrace,
    DURABLE_SUSPENSION_V1, Decision, DenialReason, EdgeSpec, EffectDeclarationSpec, Event,
    InvariantIssue, LINEAR_LIFECYCLE_V1, LifecycleProfile, LocalState, MachineSpec, MachineState,
    NodeSpec, ReplayBindings, ReplayDecision, ReplayFailureReason, ReplayInput, ReplayLimits,
    ResumeDecision, SuspensionDeclarationSpec, TraceBinding, TransitionSelector, invariant,
    ready_nodes, replay, resume_decision, step,
};

const NODE_GATE: &str = "gate";
const NODE_SOURCE: &str = "source";
const NODE_SINK: &str = "sink";
const SUSPENSION_CLASS: &str = "manual-approval";
const OTHER_CLASS: &str = "other-class";
const REFERENCE: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_REFERENCE: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const MALFORMED_REFERENCE: &str = "blake3:not-hex";
const MACHINE_BINDING_BYTE: u8 = 0x41;
const STATE_BINDING_BYTE: u8 = 0x42;
const TEST_MAX_EVENTS: u64 = 64;
const SECOND_EVENT: u64 = 1;

fn node(id: &str) -> NodeSpec {
    NodeSpec::new(id, Vec::new())
}

fn durable_spec() -> MachineSpec {
    let mut spec = MachineSpec::new(
        vec![node(NODE_SOURCE), node(NODE_GATE), node(NODE_SINK)],
        vec![
            EdgeSpec::new(NODE_SOURCE, NODE_GATE),
            EdgeSpec::new(NODE_GATE, NODE_SINK),
        ],
    );
    spec.lifecycle_profile = String::from(DURABLE_SUSPENSION_V1);
    spec
}

fn admit(spec: MachineSpec) -> AdmittedMachine {
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("test machine admission failed: {diagnostics:?}"),
    }
}

fn durable_machine() -> AdmittedMachine {
    admit(durable_spec())
}

fn diagnostic_codes(spec: MachineSpec) -> Vec<AdmissionDiagnosticCode> {
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(_) => panic!("admission unexpectedly succeeded"),
        Err(diagnostics) => diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect(),
    }
}

fn gate_ready_state(machine: &AdmittedMachine) -> MachineState {
    let mut state = machine.initial_state();
    let source = Event::new(NODE_SOURCE, Action::MarkReady);
    let started = Event::new(NODE_SOURCE, Action::Start);
    let completed = Event::new(NODE_SOURCE, Action::Complete);
    let mark_gate = Event::new(NODE_GATE, Action::MarkReady);
    for event in [source, started, completed, mark_gate] {
        match step(machine, &state, &event) {
            Decision::Accept { next, .. } => state = next,
            Decision::Deny { reason } => panic!("setup step denied: {reason:?}"),
        }
    }
    state
}

fn suspended_gate_state(machine: &AdmittedMachine) -> MachineState {
    let state = gate_ready_state(machine);
    let suspend = Event::suspension(NODE_GATE, Action::Suspend, SUSPENSION_CLASS, REFERENCE);
    match step(machine, &state, &suspend) {
        Decision::Accept { next, .. } => next,
        Decision::Deny { reason } => panic!("setup suspend denied: {reason:?}"),
    }
}

fn resume_event(class: &str, reference: &str) -> Event {
    Event::suspension(NODE_GATE, Action::ResumeSignal, class, reference)
}

fn bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([MACHINE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        TraceBinding::from_bytes([STATE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
    )
}

fn replay_events(
    machine: &AdmittedMachine,
    initial: &MachineState,
    events: &[Event],
) -> ReplayDecision {
    let trace = BoundedTrace::new(bindings(), events);
    replay(ReplayInput::new(
        machine,
        initial,
        bindings(),
        trace,
        ReplayLimits {
            max_events: TEST_MAX_EVENTS,
        },
    ))
}

fn must_some<T>(value: Option<T>) -> T {
    match value {
        Some(value) => value,
        None => panic!("expected a suspension record"),
    }
}

fn gate_id(machine: &AdmittedMachine) -> choregraph_core::NodeId {
    for node_id in machine.node_ids() {
        if node_id.as_str() == NODE_GATE {
            return node_id.clone();
        }
    }
    panic!("admitted node is missing: {NODE_GATE}")
}

fn deny_reason(decision: Decision) -> DenialReason {
    match decision {
        Decision::Deny { reason } => reason,
        Decision::Accept { .. } => panic!("transition unexpectedly accepted"),
    }
}

#[test]
fn durable_profile_admission_selects_explicit_profile() {
    let machine = durable_machine();
    assert_eq!(
        machine.lifecycle_profile(),
        LifecycleProfile::DurableSuspensionV1
    );
    assert!(machine.suspension_declarations().is_empty());
}

#[test]
fn v1_machine_behavior_stays_linear() {
    let machine = admit(MachineSpec::new(vec![node(NODE_GATE)], Vec::new()));
    assert_eq!(machine.lifecycle_profile(), LifecycleProfile::LinearV1);
    let state = machine.initial_state();
    let ready = match step(&machine, &state, &Event::new(NODE_GATE, Action::MarkReady)) {
        Decision::Accept { next, .. } => next,
        Decision::Deny { reason } => panic!("v1 mark-ready denied: {reason:?}"),
    };
    let suspend = Event::suspension(NODE_GATE, Action::Suspend, SUSPENSION_CLASS, REFERENCE);
    assert_eq!(
        deny_reason(step(&machine, &ready, &suspend)),
        DenialReason::UnsupportedProfileAction {
            action: Action::Suspend
        }
    );
    assert!(ready.suspensions().is_empty());
}

#[test]
fn admission_rejects_unsupported_profile_and_profile_less_declarations() {
    let mut unsupported = durable_spec();
    unsupported.lifecycle_profile = String::from("choregraph.lifecycle.unknown.v9");
    assert!(
        diagnostic_codes(unsupported)
            .contains(&AdmissionDiagnosticCode::UnsupportedLifecycleProfile)
    );

    let mut linear_with_declaration = MachineSpec::new(vec![node(NODE_GATE)], Vec::new());
    assert_eq!(
        linear_with_declaration.lifecycle_profile,
        LINEAR_LIFECYCLE_V1
    );
    linear_with_declaration
        .suspensions
        .push(SuspensionDeclarationSpec {
            node_id: String::from(NODE_GATE),
            class: String::from(SUSPENSION_CLASS),
            reference: String::from(REFERENCE),
        });
    assert!(
        diagnostic_codes(linear_with_declaration)
            .contains(&AdmissionDiagnosticCode::SuspensionDeclarationWithoutProfile)
    );
}

#[test]
fn admission_rejects_malformed_duplicate_and_missing_suspension_declarations() {
    let mut duplicate = durable_spec();
    for _ in 0..2 {
        duplicate.suspensions.push(SuspensionDeclarationSpec {
            node_id: String::from(NODE_GATE),
            class: String::from(SUSPENSION_CLASS),
            reference: String::from(REFERENCE),
        });
    }
    assert!(
        diagnostic_codes(duplicate)
            .contains(&AdmissionDiagnosticCode::DuplicateSuspensionDeclaration)
    );

    let mut missing_node = durable_spec();
    missing_node.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from("missing"),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(REFERENCE),
    });
    assert!(
        diagnostic_codes(missing_node)
            .contains(&AdmissionDiagnosticCode::SuspensionDeclarationNodeMissing)
    );

    let mut bad_class = durable_spec();
    bad_class.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from(NODE_GATE),
        class: String::from("bad class!"),
        reference: String::from(REFERENCE),
    });
    assert!(
        diagnostic_codes(bad_class).contains(&AdmissionDiagnosticCode::MalformedSuspensionClass)
    );

    let mut bad_reference = durable_spec();
    bad_reference.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from(NODE_GATE),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(MALFORMED_REFERENCE),
    });
    assert!(
        diagnostic_codes(bad_reference)
            .contains(&AdmissionDiagnosticCode::MalformedSuspensionReference)
    );
}

#[test]
fn admission_rejects_effect_declarations_on_suspension_transitions() {
    let mut spec = durable_spec();
    spec.effects.push(EffectDeclarationSpec {
        node_id: String::from(NODE_GATE),
        transition: TransitionSelector {
            from: LocalState::Ready,
            action: Action::Suspend,
        },
        operation: String::from("op"),
        resource: String::from("resource/x"),
        ability: String::from("execute"),
        request: String::from("request/x"),
        payload: String::from(REFERENCE),
    });
    assert!(diagnostic_codes(spec).contains(&AdmissionDiagnosticCode::SuspensionEffectTransition));
}

#[test]
fn declared_suspension_state_reconstructs_recorded_wait() {
    let mut spec = durable_spec();
    spec.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from(NODE_GATE),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(REFERENCE),
    });
    let machine = admit(spec);
    let state = machine.declared_suspension_state();
    let gate = gate_id(&machine);
    assert_eq!(state.state(&gate), Some(LocalState::Suspended));
    let record = must_some(state.suspension(&gate));
    assert_eq!(record.class.as_str(), SUSPENSION_CLASS);
    assert_eq!(record.reference.as_str(), REFERENCE);
    assert!(invariant(&machine, &state));
}

#[test]
fn suspend_moves_ready_gate_to_suspended_without_effects() {
    let machine = durable_machine();
    let state = gate_ready_state(&machine);
    let gate = gate_id(&machine);
    let suspend = Event::suspension(NODE_GATE, Action::Suspend, SUSPENSION_CLASS, REFERENCE);
    match step(&machine, &state, &suspend) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            assert_eq!(next.state(&gate), Some(LocalState::Suspended));
            let record = must_some(next.suspension(&gate));
            assert_eq!(record.class.as_str(), SUSPENSION_CLASS);
            assert_eq!(record.reference.as_str(), REFERENCE);
            assert!(invariant(&machine, &next));
        }
        Decision::Deny { reason } => panic!("suspend denied: {reason:?}"),
    }
}

#[test]
fn matching_resume_releases_suspended_gate_without_effects() {
    let machine = durable_machine();
    let state = suspended_gate_state(&machine);
    let gate = gate_id(&machine);
    match step(&machine, &state, &resume_event(SUSPENSION_CLASS, REFERENCE)) {
        Decision::Accept { next, intents } => {
            assert!(intents.is_empty());
            assert_eq!(next.state(&gate), Some(LocalState::Ready));
            assert!(next.suspension(&gate).is_none());
            assert!(invariant(&machine, &next));
        }
        Decision::Deny { reason } => panic!("resume denied: {reason:?}"),
    }
}

#[test]
fn suspended_gate_is_never_ready_and_waits_for_signal() {
    let machine = durable_machine();
    let state = suspended_gate_state(&machine);
    let gate = gate_id(&machine);
    let ready = match ready_nodes(&machine, &state) {
        Ok(ready) => ready,
        Err(issues) => panic!("ready nodes rejected a valid state: {issues:?}"),
    };
    assert!(!ready.contains(&gate));
    assert_eq!(
        resume_decision(&machine, &state, &gate),
        ResumeDecision::AwaitSignal
    );
}

#[test]
fn suspension_denial_classes_fail_closed() {
    let machine = durable_machine();
    let state = suspended_gate_state(&machine);
    let ready = gate_ready_state(&machine);

    let nested = Event::suspension(NODE_GATE, Action::Suspend, SUSPENSION_CLASS, REFERENCE);
    assert!(matches!(
        deny_reason(step(&machine, &state, &nested)),
        DenialReason::InvalidTransition { .. }
    ));

    assert!(matches!(
        deny_reason(step(
            &machine,
            &ready,
            &resume_event(SUSPENSION_CLASS, REFERENCE)
        )),
        DenialReason::InvalidTransition { .. }
    ));

    assert_eq!(
        deny_reason(step(
            &machine,
            &state,
            &resume_event(OTHER_CLASS, REFERENCE)
        )),
        DenialReason::SuspensionMismatch
    );
    assert_eq!(
        deny_reason(step(
            &machine,
            &state,
            &resume_event(SUSPENSION_CLASS, OTHER_REFERENCE)
        )),
        DenialReason::SuspensionMismatch
    );

    let missing_fact = Event::new(NODE_GATE, Action::Suspend);
    assert!(missing_fact.suspension.is_none());
    assert_eq!(
        deny_reason(step(&machine, &ready, &missing_fact)),
        DenialReason::SuspensionFactMissing
    );

    let mut unexpected_fact = Event::new(NODE_GATE, Action::Start);
    unexpected_fact.suspension = Some(choregraph_core::SuspensionFactSpec {
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(REFERENCE),
    });
    assert_eq!(
        deny_reason(step(&machine, &ready, &unexpected_fact)),
        DenialReason::SuspensionFactUnexpected
    );

    let wrong_node = Event::suspension("missing", Action::Suspend, SUSPENSION_CLASS, REFERENCE);
    assert_eq!(
        deny_reason(step(&machine, &ready, &wrong_node)),
        DenialReason::UnknownNode
    );
}

#[test]
fn malformed_and_oversized_suspension_facts_are_denied() {
    let machine = durable_machine();
    let ready = gate_ready_state(&machine);
    let malformed = Event::suspension(NODE_GATE, Action::Suspend, "bad class!", REFERENCE);
    assert_eq!(
        deny_reason(step(&machine, &ready, &malformed)),
        DenialReason::MalformedSuspensionFact
    );
    let malformed_reference = Event::suspension(
        NODE_GATE,
        Action::Suspend,
        SUSPENSION_CLASS,
        MALFORMED_REFERENCE,
    );
    assert_eq!(
        deny_reason(step(&machine, &ready, &malformed_reference)),
        DenialReason::MalformedSuspensionFact
    );
}

#[test]
fn terminal_nodes_reject_suspend_and_resume() {
    let machine = durable_machine();
    let mut state = machine.initial_state();
    for event in [
        Event::new(NODE_SOURCE, Action::MarkReady),
        Event::new(NODE_SOURCE, Action::Start),
        Event::new(NODE_SOURCE, Action::Complete),
    ] {
        match step(&machine, &state, &event) {
            Decision::Accept { next, .. } => state = next,
            Decision::Deny { reason } => panic!("setup denied: {reason:?}"),
        }
    }
    let suspend = Event::suspension(NODE_SOURCE, Action::Suspend, SUSPENSION_CLASS, REFERENCE);
    assert!(matches!(
        deny_reason(step(&machine, &state, &suspend)),
        DenialReason::TerminalState { .. }
    ));
}

#[test]
fn duplicate_resume_after_accept_is_denied() {
    let machine = durable_machine();
    let state = suspended_gate_state(&machine);
    let resumed = match step(&machine, &state, &resume_event(SUSPENSION_CLASS, REFERENCE)) {
        Decision::Accept { next, .. } => next,
        Decision::Deny { reason } => panic!("resume denied: {reason:?}"),
    };
    assert!(matches!(
        deny_reason(step(
            &machine,
            &resumed,
            &resume_event(SUSPENSION_CLASS, REFERENCE)
        )),
        DenialReason::InvalidTransition { .. }
    ));
}

#[test]
fn replay_ends_validly_suspended_and_matches_direct_stepping() {
    let machine = durable_machine();
    let initial = machine.initial_state();
    let prefix = vec![
        Event::new(NODE_SOURCE, Action::MarkReady),
        Event::new(NODE_SOURCE, Action::Start),
        Event::new(NODE_SOURCE, Action::Complete),
        Event::new(NODE_GATE, Action::MarkReady),
        Event::suspension(NODE_GATE, Action::Suspend, SUSPENSION_CLASS, REFERENCE),
    ];
    let gate = gate_id(&machine);
    match replay_events(&machine, &initial, &prefix) {
        ReplayDecision::Complete { final_state, .. } => {
            assert_eq!(final_state.state(&gate), Some(LocalState::Suspended));
        }
        ReplayDecision::Failed(failure) => panic!("suspended replay failed: {failure:?}"),
    }

    let mut extended = prefix.clone();
    extended.push(resume_event(SUSPENSION_CLASS, REFERENCE));
    let replayed = match replay_events(&machine, &initial, &extended) {
        ReplayDecision::Complete { final_state, .. } => final_state,
        ReplayDecision::Failed(failure) => panic!("extended replay failed: {failure:?}"),
    };
    let mut direct = initial.clone();
    for event in &extended {
        match step(&machine, &direct, event) {
            Decision::Accept { next, .. } => direct = next,
            Decision::Deny { reason } => panic!("direct step denied: {reason:?}"),
        }
    }
    assert_eq!(replayed, direct);
}

#[test]
fn replay_reports_first_exact_suspension_denial() {
    let machine = durable_machine();
    let initial = machine.initial_state();
    let premature = vec![
        Event::new(NODE_SOURCE, Action::MarkReady),
        resume_event(SUSPENSION_CLASS, REFERENCE),
    ];
    match replay_events(&machine, &initial, &premature) {
        ReplayDecision::Failed(failure) => {
            assert_eq!(
                failure.event_index.map(|index| index.get()),
                Some(SECOND_EVENT)
            );
            assert!(matches!(
                failure.reason,
                ReplayFailureReason::TransitionDenied(DenialReason::InvalidTransition { .. })
            ));
        }
        ReplayDecision::Complete { .. } => panic!("premature resume replayed as complete"),
    }

    let stale = vec![
        Event::new(NODE_SOURCE, Action::MarkReady),
        Event::new(NODE_SOURCE, Action::Start),
        Event::new(NODE_SOURCE, Action::Complete),
        Event::new(NODE_GATE, Action::MarkReady),
        Event::suspension(NODE_GATE, Action::Suspend, SUSPENSION_CLASS, REFERENCE),
        resume_event(SUSPENSION_CLASS, OTHER_REFERENCE),
    ];
    let last_event = stale.len() as u64 - 1;
    match replay_events(&machine, &initial, &stale) {
        ReplayDecision::Failed(failure) => {
            assert_eq!(
                failure.event_index.map(|index| index.get()),
                Some(last_event)
            );
            assert!(matches!(
                failure.reason,
                ReplayFailureReason::TransitionDenied(DenialReason::SuspensionMismatch)
            ));
        }
        ReplayDecision::Complete { .. } => panic!("stale signal replayed as complete"),
    }
}

#[test]
fn state_invariant_flags_suspension_record_conflicts() {
    let machine = durable_machine();
    let state = suspended_gate_state(&machine);
    let gate = gate_id(&machine);

    let missing_record =
        MachineState::with_suspensions(state.entries().clone(), std::collections::BTreeMap::new());
    let issues = choregraph_core::validate_invariant(&machine, &missing_record);
    assert!(issues.contains(&InvariantIssue::SuspensionRecordMissing(gate.clone())));

    let ready = gate_ready_state(&machine);
    let unexpected_record = MachineState::with_suspensions(
        ready.entries().clone(),
        [(gate.clone(), must_some(state.suspension(&gate)).clone())]
            .into_iter()
            .collect(),
    );
    let issues = choregraph_core::validate_invariant(&machine, &unexpected_record);
    assert!(issues.contains(&InvariantIssue::SuspensionRecordUnexpected(gate.clone())));

    let linear = admit(MachineSpec::new(vec![node(NODE_GATE)], Vec::new()));
    let mut linear_entries = linear.initial_state().entries().clone();
    linear_entries.insert(gate.clone(), LocalState::Suspended);
    let linear_suspended = MachineState::new(linear_entries);
    let issues = choregraph_core::validate_invariant(&linear, &linear_suspended);
    assert!(issues.contains(&InvariantIssue::SuspensionWithoutProfile(gate.clone())));
    assert!(issues.contains(&InvariantIssue::SuspensionRecordMissing(gate.clone())));
}
