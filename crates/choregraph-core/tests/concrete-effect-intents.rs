// r[verify choregraph.effect_v2.fixtures]
// r[verify choregraph.effect_v2.determinism]
// r[verify choregraph.effect_v2.denial]
// r[verify choregraph.effect_v2.replay]
#![allow(
    unknown_lints,
    assertion_density,
    function_length,
    non_trait_imports,
    numeric_units,
    reason = "the concrete-effect corpus keeps explicit positive and negative boundary cases"
)]

use choregraph_core::{
    Action, AdmissionDiagnosticCode, AdmissionLimits, AdmittedMachine,
    CANONICAL_REFERENCE_PROFILE_V1, ConcreteBindingInput, ConcreteBoundedTrace, ConcreteDecision,
    ConcreteEffectDeclarationSpec, ConcreteEffectRequest, ConcreteEvent, ConcreteReferenceSpec,
    ConcreteReplayInput, Decision, DenialReason, LocalState, MachineSpec, MachineSpecV2, NodeSpec,
    ReferenceBindingSpec, ReplayBindings, ReplayDecision, ReplayLimits, TraceBinding,
    TransitionSelector, concrete_effects_allowed, replay_concrete, step, step_concrete,
};

const NODE: &str = "effect-node";
const OPERATION_EXACT: &str = "effect/exact";
const OPERATION_SLOT: &str = "effect/slot";
const RESOURCE: &str = "resource/work";
const ABILITY: &str = "effect/run";
const EXACT_KEY: &str = "declaration/exact";
const SLOT_KEY: &str = "declaration/slot";
const REQUEST_SLOT: &str = "request/input";
const REQUEST_SOURCE: &str = "node/input/request";
const PAYLOAD_SLOT: &str = "payload/output";
const PAYLOAD_SOURCE: &str = "node/output/payload";
const REFERENCE_A: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REFERENCE_B: &str = "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const REFERENCE_C: &str = "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const REFERENCE_D: &str = "blake3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const MACHINE_BINDING_BYTE: u8 = 0x41;
const STATE_BINDING_BYTE: u8 = 0x42;
const TRACE_LIMIT: u64 = 8;
const TWO_EFFECTS: usize = 2;
const EVENT_ACTION_BYTES: usize = 1;
const MINIMUM_EVENT_LIMIT: usize = NODE.len() + EVENT_ACTION_BYTES;

fn reference(identity: &str) -> ConcreteReferenceSpec {
    ConcreteReferenceSpec {
        profile: String::from(CANONICAL_REFERENCE_PROFILE_V1),
        identity: String::from(identity),
    }
}

fn exact_input(identity: &str) -> ConcreteBindingInput {
    ConcreteBindingInput {
        slot_id: None,
        source_ref: None,
        reference: reference(identity),
    }
}

fn slot_input(slot_id: &str, source_ref: &str, identity: &str) -> ConcreteBindingInput {
    ConcreteBindingInput {
        slot_id: Some(String::from(slot_id)),
        source_ref: Some(String::from(source_ref)),
        reference: reference(identity),
    }
}

fn transition() -> TransitionSelector {
    TransitionSelector {
        from: LocalState::Ready,
        action: Action::Start,
    }
}

fn exact_declaration() -> ConcreteEffectDeclarationSpec {
    ConcreteEffectDeclarationSpec {
        declaration_key: String::from(EXACT_KEY),
        node_id: String::from(NODE),
        transition: transition(),
        operation: String::from(OPERATION_EXACT),
        resource: String::from(RESOURCE),
        ability: String::from(ABILITY),
        request_binding: ReferenceBindingSpec::Exact {
            reference: reference(REFERENCE_A),
        },
        payload_binding: ReferenceBindingSpec::Exact {
            reference: reference(REFERENCE_B),
        },
    }
}

fn slot_declaration() -> ConcreteEffectDeclarationSpec {
    ConcreteEffectDeclarationSpec {
        declaration_key: String::from(SLOT_KEY),
        node_id: String::from(NODE),
        transition: transition(),
        operation: String::from(OPERATION_SLOT),
        resource: String::from(RESOURCE),
        ability: String::from(ABILITY),
        request_binding: ReferenceBindingSpec::Slot {
            slot_id: String::from(REQUEST_SLOT),
            source_ref: String::from(REQUEST_SOURCE),
        },
        payload_binding: ReferenceBindingSpec::Slot {
            slot_id: String::from(PAYLOAD_SLOT),
            source_ref: String::from(PAYLOAD_SOURCE),
        },
    }
}

fn machine_spec(reversed: bool) -> MachineSpecV2 {
    let mut spec = MachineSpecV2::new(
        vec![NodeSpec::new(
            NODE,
            vec![String::from(OPERATION_SLOT), String::from(OPERATION_EXACT)],
        )],
        Vec::new(),
    );
    spec.effects = if reversed {
        vec![slot_declaration(), exact_declaration()]
    } else {
        vec![exact_declaration(), slot_declaration()]
    };
    spec
}

fn admit_spec(spec: MachineSpecV2) -> AdmittedMachine {
    match AdmittedMachine::admit_v2(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("v2 admission failed: {diagnostics:?}"),
    }
}

fn machine() -> AdmittedMachine {
    admit_spec(machine_spec(false))
}

fn exact_request() -> ConcreteEffectRequest {
    ConcreteEffectRequest {
        declaration_key: String::from(EXACT_KEY),
        request: exact_input(REFERENCE_A),
        payload: exact_input(REFERENCE_B),
    }
}

fn slot_request(request_identity: &str, payload_identity: &str) -> ConcreteEffectRequest {
    ConcreteEffectRequest {
        declaration_key: String::from(SLOT_KEY),
        request: slot_input(REQUEST_SLOT, REQUEST_SOURCE, request_identity),
        payload: slot_input(PAYLOAD_SLOT, PAYLOAD_SOURCE, payload_identity),
    }
}

fn ready_state(machine: &AdmittedMachine) -> choregraph_core::MachineState {
    match step_concrete(
        machine,
        &machine.initial_state(),
        &ConcreteEvent::new(NODE, Action::MarkReady),
    ) {
        ConcreteDecision::Accept { next, intents } => {
            assert!(intents.is_empty());
            next
        }
        ConcreteDecision::Deny { reason } => panic!("mark-ready denied: {reason:?}"),
    }
}

fn start_event(reversed: bool, request_identity: &str, payload_identity: &str) -> ConcreteEvent {
    let mut event = ConcreteEvent::new(NODE, Action::Start);
    event.effects = if reversed {
        vec![
            slot_request(request_identity, payload_identity),
            exact_request(),
        ]
    } else {
        vec![
            exact_request(),
            slot_request(request_identity, payload_identity),
        ]
    };
    event
}

fn bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([MACHINE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
        TraceBinding::from_bytes([STATE_BINDING_BYTE; choregraph_core::TRACE_BINDING_BYTES]),
    )
}

fn diagnostic_codes(
    result: Result<AdmittedMachine, Vec<choregraph_core::AdmissionDiagnostic>>,
) -> Vec<AdmissionDiagnosticCode> {
    match result {
        Ok(_) => panic!("invalid v2 machine was admitted"),
        Err(diagnostics) => diagnostics.into_iter().map(|item| item.code).collect(),
    }
}

#[test]
fn exact_and_slot_bindings_emit_declared_concrete_intents() {
    let machine = machine();
    let ready = ready_state(&machine);
    let event = start_event(false, REFERENCE_C, REFERENCE_D);
    let decision = step_concrete(&machine, &ready, &event);
    let ConcreteDecision::Accept { next, intents } = decision else {
        panic!("valid concrete event was denied: {decision:?}");
    };
    assert_eq!(intents.len(), TWO_EFFECTS);
    assert_eq!(intents[0].declaration_key.as_str(), EXACT_KEY);
    assert_eq!(intents[1].declaration_key.as_str(), SLOT_KEY);
    assert_eq!(
        intents[1].request_slot.as_ref().map(|id| id.as_str()),
        Some(REQUEST_SLOT)
    );
    assert_eq!(
        intents[1].request_source_ref.as_ref().map(|id| id.as_str()),
        Some(REQUEST_SOURCE)
    );
    assert_eq!(intents[1].request_reference.identity.as_str(), REFERENCE_C);
    assert_eq!(intents[1].payload_reference.identity.as_str(), REFERENCE_D);
    assert!(concrete_effects_allowed(&machine, &ready, &event, &intents));
    assert_eq!(
        next.entries().values().copied().collect::<Vec<_>>(),
        [LocalState::Running]
    );
}

#[test]
fn declaration_and_request_source_order_do_not_change_the_decision() {
    let first = machine();
    let second = admit_spec(machine_spec(true));
    assert_eq!(first, second);
    let ready = ready_state(&first);
    let forward = step_concrete(
        &first,
        &ready,
        &start_event(false, REFERENCE_C, REFERENCE_D),
    );
    let reversed = step_concrete(
        &second,
        &ready,
        &start_event(true, REFERENCE_C, REFERENCE_D),
    );
    assert_eq!(forward, reversed);
}

#[test]
fn runtime_references_do_not_change_the_admitted_machine() {
    let machine = machine();
    let baseline = machine.clone();
    let ready = ready_state(&machine);
    let first = step_concrete(
        &machine,
        &ready,
        &start_event(false, REFERENCE_C, REFERENCE_D),
    );
    let second = step_concrete(
        &machine,
        &ready,
        &start_event(false, REFERENCE_D, REFERENCE_C),
    );
    assert_ne!(first, second);
    assert_eq!(machine, baseline);
}

#[test]
fn duplicate_keys_and_unknown_profiles_fail_admission_without_raw_values() {
    let mut duplicate = machine_spec(false);
    duplicate.effects[1].declaration_key = String::from(EXACT_KEY);
    let diagnostics = match AdmittedMachine::admit_v2(duplicate, AdmissionLimits::default()) {
        Ok(_) => panic!("duplicate declaration key was admitted"),
        Err(diagnostics) => diagnostics,
    };
    assert!(
        diagnostics
            .iter()
            .any(|item| item.code == AdmissionDiagnosticCode::DuplicateDeclarationKey)
    );
    assert!(
        diagnostics
            .iter()
            .all(|item| item.subject.as_deref() != Some(REFERENCE_A))
    );

    let mut unknown_profile = machine_spec(false);
    unknown_profile.effects[0].request_binding = ReferenceBindingSpec::Exact {
        reference: ConcreteReferenceSpec {
            profile: String::from("unknown-profile"),
            identity: String::from("credential=secret"),
        },
    };
    let diagnostics = match AdmittedMachine::admit_v2(unknown_profile, AdmissionLimits::default()) {
        Ok(_) => panic!("unknown profile was admitted"),
        Err(diagnostics) => diagnostics,
    };
    assert!(
        diagnostics
            .iter()
            .any(|item| item.code == AdmissionDiagnosticCode::UnknownReferenceProfile)
    );
    assert!(
        diagnostics
            .iter()
            .all(|item| item.subject.as_deref() != Some("credential=secret"))
    );
}

#[test]
fn wrong_exact_slot_source_unknown_duplicate_and_oversized_requests_fail_closed() {
    let machine = machine();
    let ready = ready_state(&machine);

    let mut wrong_exact = ConcreteEvent::new(NODE, Action::Start);
    let mut changed_exact = exact_request();
    changed_exact.request.reference = reference(REFERENCE_C);
    wrong_exact.effects.push(changed_exact);
    assert_eq!(
        step_concrete(&machine, &ready, &wrong_exact),
        ConcreteDecision::Deny {
            reason: DenialReason::ConcreteBindingMismatch,
        }
    );

    let mut wrong_slot = ConcreteEvent::new(NODE, Action::Start);
    let mut changed_slot = slot_request(REFERENCE_C, REFERENCE_D);
    changed_slot.request.slot_id = Some(String::from("request/wrong"));
    wrong_slot.effects.push(changed_slot);
    assert_eq!(
        step_concrete(&machine, &ready, &wrong_slot),
        ConcreteDecision::Deny {
            reason: DenialReason::ConcreteBindingMismatch,
        }
    );

    let mut wrong_source = ConcreteEvent::new(NODE, Action::Start);
    let mut changed_source = slot_request(REFERENCE_C, REFERENCE_D);
    changed_source.payload.source_ref = Some(String::from("node/output/wrong"));
    wrong_source.effects.push(changed_source);
    assert_eq!(
        step_concrete(&machine, &ready, &wrong_source),
        ConcreteDecision::Deny {
            reason: DenialReason::ConcreteBindingMismatch,
        }
    );

    let mut unknown = ConcreteEvent::new(NODE, Action::Start);
    let mut unknown_request = exact_request();
    unknown_request.declaration_key = String::from("declaration/unknown");
    unknown.effects.push(unknown_request);
    assert_eq!(
        step_concrete(&machine, &ready, &unknown),
        ConcreteDecision::Deny {
            reason: DenialReason::UnknownDeclaration,
        }
    );

    let mut duplicate = ConcreteEvent::new(NODE, Action::Start);
    duplicate.effects = vec![exact_request(), exact_request()];
    assert_eq!(
        step_concrete(&machine, &ready, &duplicate),
        ConcreteDecision::Deny {
            reason: DenialReason::DuplicateEffectRequest,
        }
    );

    let small_limits = AdmissionLimits {
        max_event_bytes: MINIMUM_EVENT_LIMIT,
        ..AdmissionLimits::default()
    };
    let small_machine = match AdmittedMachine::admit_v2(machine_spec(false), small_limits) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("small-event machine admission failed: {diagnostics:?}"),
    };
    let small_ready = ready_state(&small_machine);
    assert!(matches!(
        step_concrete(
            &small_machine,
            &small_ready,
            &start_event(false, REFERENCE_C, REFERENCE_D),
        ),
        ConcreteDecision::Deny {
            reason: DenialReason::EventTooLarge { .. },
        }
    ));
}

#[test]
fn v1_and_v2_wrappers_reject_the_wrong_schema() {
    let v2 = machine();
    assert_eq!(
        step(
            &v2,
            &v2.initial_state(),
            &choregraph_core::Event::new(NODE, Action::MarkReady),
        ),
        Decision::Deny {
            reason: DenialReason::UnsupportedMachineSchema,
        }
    );

    let v1 = match AdmittedMachine::admit(
        MachineSpec::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new()),
        AdmissionLimits::default(),
    ) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("v1 admission failed: {diagnostics:?}"),
    };
    assert_eq!(
        step_concrete(
            &v1,
            &v1.initial_state(),
            &ConcreteEvent::new(NODE, Action::MarkReady),
        ),
        ConcreteDecision::Deny {
            reason: DenialReason::UnsupportedMachineSchema,
        }
    );
}

#[test]
fn replay_uses_the_concrete_step_and_returns_no_intent_surface() {
    let machine = machine();
    let initial = machine.initial_state();
    let events = [
        ConcreteEvent::new(NODE, Action::MarkReady),
        start_event(false, REFERENCE_C, REFERENCE_D),
    ];
    let replayed = replay_concrete(ConcreteReplayInput::new(
        &machine,
        &initial,
        bindings(),
        ConcreteBoundedTrace::new(bindings(), &events),
        ReplayLimits {
            max_events: TRACE_LIMIT,
        },
    ));
    assert!(matches!(replayed, ReplayDecision::Complete { .. }));
}

#[test]
fn malformed_schema_and_reference_fail_before_intent_output() {
    let mut schema = machine_spec(false);
    schema.schema = String::from("choregraph.machine.v9");
    assert!(
        diagnostic_codes(AdmittedMachine::admit_v2(
            schema,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::UnsupportedSchema)
    );

    let machine = machine();
    let ready = ready_state(&machine);
    let mut malformed = ConcreteEvent::new(NODE, Action::Start);
    let mut request = slot_request(REFERENCE_C, REFERENCE_D);
    request.payload.reference.identity = String::from("sha256:not-canonical");
    malformed.effects.push(request);
    assert_eq!(
        step_concrete(&machine, &ready, &malformed),
        ConcreteDecision::Deny {
            reason: DenialReason::MalformedEffectRequest,
        }
    );
}
