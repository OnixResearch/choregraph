// r[verify choregraph.effect_v2.seaglass]
// r[verify choregraph.effect_v2.lattice_handoff]
#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    non_trait_imports,
    numeric_units,
    reason = "the conversion-only v2 fixture keeps explicit handoff and wrong-schema cases"
)]

use choregraph_core::{
    concrete::effect::ConcreteEffectRequest, concrete::event::ConcreteEvent,
    concrete::reference::ConcreteBindingInput, concrete::reference::ConcreteReferenceSpec,
    model::automaton::AdmittedMachine, model::automaton::MachineSpec,
    model::capacity::AdmissionLimits, model::graph::NodeSpec, model::semantics::Action,
};
use choregraph_seaglass::lifecycle::ConversionError;
use choregraph_seaglass::v2::{
    ConcreteIntentHandoff, ConcreteMachineDecision, ConcretePlanError, admit_concrete_machine,
    convert_concrete_intent, step_concrete_machine,
};

#[path = "../../../fixtures/lattice-handoff/consumer.rs"]
mod lattice_fixture;

const LATTICE_EFFECT_V2_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/lattice-effect-v2.json");
const NODE: &str = "lattice-node/1";
const DECLARATION_KEY: &str = "request/main";
const REQUEST_SLOT: &str = "request/input";
const REQUEST_SOURCE: &str = "node/1/input/request";
const REQUEST_REFERENCE: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PAYLOAD_REFERENCE: &str =
    "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SINGLE_INTENT: usize = 1;
const FIRST_INTENT: usize = 0;
const CONTENT_IDENTITY: &str =
    "blake3:1111111111111111111111111111111111111111111111111111111111111111";
const MACHINE_IDENTITY: &str =
    "blake3:2222222222222222222222222222222222222222222222222222222222222222";
const WORKFLOW_REVISION: &str =
    "blake3:3333333333333333333333333333333333333333333333333333333333333333";
const RUN_IDENTITY_A: &str =
    "blake3:4444444444444444444444444444444444444444444444444444444444444444";
const RUN_IDENTITY_B: &str =
    "blake3:5555555555555555555555555555555555555555555555555555555555555555";
const POLICY_IDENTITY: &str =
    "blake3:6666666666666666666666666666666666666666666666666666666666666666";
const AUTHORITY_IDENTITY: &str =
    "blake3:7777777777777777777777777777777777777777777777777777777777777777";
const PERSISTENCE_IDENTITY: &str =
    "blake3:8888888888888888888888888888888888888888888888888888888888888888";
const ATTEMPT_IDENTITY: &str =
    "blake3:9999999999999999999999999999999999999999999999999999999999999999";

#[allow(no_panic, reason = "this test helper reports failed fixture setup")]
fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("expected success, got {error:?}"),
    }
}

fn reference(identity: &str) -> ConcreteReferenceSpec {
    ConcreteReferenceSpec {
        profile: String::from(choregraph_core::concrete::reference::CANONICAL_REFERENCE_PROFILE_V1),
        identity: String::from(identity),
    }
}

fn request() -> ConcreteEffectRequest {
    ConcreteEffectRequest {
        declaration_key: String::from(DECLARATION_KEY),
        request: ConcreteBindingInput {
            slot_id: Some(String::from(REQUEST_SLOT)),
            source_ref: Some(String::from(REQUEST_SOURCE)),
            reference: reference(REQUEST_REFERENCE),
        },
        payload: ConcreteBindingInput {
            slot_id: None,
            source_ref: None,
            reference: reference(PAYLOAD_REFERENCE),
        },
    }
}

#[allow(no_panic, reason = "this test helper reports failed fixture setup")]
fn concrete_intents() -> (
    choregraph_core::concrete::effect::ConcreteEffectIntent,
    ConcreteIntentHandoff,
) {
    let lowered = must(choregraph_nickel::lower::lower_evaluated_json(
        LATTICE_EFFECT_V2_JSON,
        &[],
        choregraph_nickel::profile::reviewed_frontend_limits(),
    ));
    assert_eq!(
        lowered.source_format,
        choregraph_nickel::lower::LoweredSourceFormat::LatticeWorkflowV2
    );
    let plan = must(admit_concrete_machine(lowered.machine));
    let initial = plan.machine().initial_state();
    let ready = match must(step_concrete_machine(
        &plan,
        &initial,
        &ConcreteEvent::new(NODE, Action::MarkReady),
    )) {
        ConcreteMachineDecision::Accept { next, intents } => {
            assert!(intents.is_empty());
            next
        }
        ConcreteMachineDecision::Deny { reason } => panic!("mark-ready denied: {reason:?}"),
    };
    let mut start = ConcreteEvent::new(NODE, Action::Start);
    start.effects.push(request());
    let ConcreteMachineDecision::Accept { intents, .. } =
        must(step_concrete_machine(&plan, &ready, &start))
    else {
        panic!("declared concrete effect was denied");
    };
    assert_eq!(intents.len(), SINGLE_INTENT);
    let choregraph_core::concrete::event::ConcreteDecision::Accept {
        intents: canonical, ..
    } = choregraph_core::transition::step_concrete(plan.machine(), &ready, &start)
    else {
        panic!("canonical concrete effect was denied");
    };
    assert_eq!(canonical.len(), SINGLE_INTENT);
    (
        canonical[FIRST_INTENT].clone(),
        intents[FIRST_INTENT].clone(),
    )
}

fn concrete_handoff() -> ConcreteIntentHandoff {
    let (canonical, converted) = concrete_intents();
    assert_eq!(must(convert_concrete_intent(&canonical)), converted);
    converted
}

#[test]
fn intent_conversion_rejects_unrepresentable_states_and_actions() {
    let (canonical, _) = concrete_intents();
    let mut suspended = canonical.clone();
    suspended.transition.from = choregraph_core::model::semantics::LocalState::Suspended;
    assert_eq!(
        convert_concrete_intent(&suspended),
        Err(ConversionError::SuspendedState)
    );
    for action in [Action::Suspend, Action::ResumeSignal] {
        let mut unsupported = canonical.clone();
        unsupported.transition.action = action;
        assert_eq!(
            convert_concrete_intent(&unsupported),
            Err(ConversionError::SuspensionAction)
        );
    }
}

#[test]
fn concrete_denial_remains_a_machine_decision_not_a_conversion_failure() {
    let lowered = must(choregraph_nickel::lower::lower_evaluated_json(
        LATTICE_EFFECT_V2_JSON,
        &[],
        choregraph_nickel::profile::reviewed_frontend_limits(),
    ));
    let plan = must(admit_concrete_machine(lowered.machine));
    let initial = plan.machine().initial_state();
    assert!(matches!(
        step_concrete_machine(&plan, &initial, &ConcreteEvent::new(NODE, Action::Complete)),
        Ok(ConcreteMachineDecision::Deny { .. })
    ));
    assert_eq!(initial, plan.machine().initial_state());
}

fn binding_input<'a>(
    intent: Option<&'a ConcreteIntentHandoff>,
    run_identity: &'a str,
) -> lattice_fixture::BindingInput<'a> {
    lattice_fixture::BindingInput {
        handoff_schema: lattice_fixture::HANDOFF_SCHEMA,
        intent,
        intent_content_identity: CONTENT_IDENTITY,
        machine_identity: MACHINE_IDENTITY,
        workflow_revision: WORKFLOW_REVISION,
        run_identity,
        policy_identity: POLICY_IDENTITY,
        authority_identity: AUTHORITY_IDENTITY,
        persistence_identity: PERSISTENCE_IDENTITY,
        attempt_identity: ATTEMPT_IDENTITY,
    }
}

#[test]
fn exposes_exact_concrete_intent_data_without_downstream_authority_fields() {
    let intent = concrete_handoff();
    assert_eq!(intent.declaration_key, DECLARATION_KEY);
    assert_eq!(intent.request.slot_id.as_deref(), Some(REQUEST_SLOT));
    assert_eq!(intent.request.source_ref.as_deref(), Some(REQUEST_SOURCE));
    assert_eq!(intent.request.identity, REQUEST_REFERENCE);
    assert_eq!(intent.payload.identity, PAYLOAD_REFERENCE);
}

#[test]
fn disposable_lattice_binding_requires_intent_and_separates_occurrences() {
    let intent = concrete_handoff();
    let first = must(lattice_fixture::bind(binding_input(
        Some(&intent),
        RUN_IDENTITY_A,
    )));
    let second = must(lattice_fixture::bind(binding_input(
        Some(&intent),
        RUN_IDENTITY_B,
    )));
    assert_eq!(
        first.intent_content_identity,
        second.intent_content_identity
    );
    assert_ne!(first.occurrence_identity, second.occurrence_identity);
    assert_eq!(first.request_source_ref.as_deref(), Some(REQUEST_SOURCE));
    assert_eq!(
        lattice_fixture::bind(binding_input(None, RUN_IDENTITY_A)),
        Err(lattice_fixture::BindingError::MissingChoregraphIntent)
    );
    assert!(lattice_fixture::NON_CLAIM.contains("does not prove policy"));
}

#[test]
fn rejects_v1_machine_as_a_concrete_handoff_plan() {
    let machine = must(AdmittedMachine::admit(
        MachineSpec::new(vec![NodeSpec::new("node", Vec::new())], Vec::new()),
        AdmissionLimits::default(),
    ));
    assert_eq!(
        admit_concrete_machine(machine),
        Err(ConcretePlanError::WrongMachineSchema)
    );
}
