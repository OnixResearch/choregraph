// r[verify choregraph.effect_v2.kamacite]
// r[verify choregraph.effect_v2.occurrence_identity]
#![allow(
    unknown_lints,
    assertion_density,
    function_length,
    non_trait_imports,
    numeric_units,
    reason = "the canonical v2 corpus keeps explicit byte and linkage failures"
)]

use crate::{
    ArtifactDomain, ConcreteArtifactError, build_concrete_intent_artifact,
    build_concrete_intent_receipt, build_concrete_machine_artifact, compute_identity,
    encode_concrete_intent_descriptor, encode_concrete_intent_receipt_body,
    validate_concrete_intent_artifact, validate_concrete_intent_receipt,
    validate_concrete_machine_artifact,
};
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, CANONICAL_REFERENCE_PROFILE_V1, ConcreteBindingInput,
    ConcreteDecision, ConcreteEffectDeclarationSpec, ConcreteEffectRequest, ConcreteEvent,
    ConcreteReferenceSpec, LocalState, MachineSpecV2, NodeSpec, ReferenceBindingSpec,
    TransitionSelector, step_concrete,
};

const NODE: &str = "effect-node";
const OPERATION: &str = "http/request";
const RESOURCE: &str = "service/example";
const ABILITY: &str = "http/request";
const DECLARATION_KEY: &str = "request/main";
const REQUEST_SLOT: &str = "request/input";
const REQUEST_SOURCE: &str = "node/input/request";
const REQUEST_REFERENCE_A: &str =
    "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const REQUEST_REFERENCE_B: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PAYLOAD_REFERENCE: &str =
    "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("expected success, got {error:?}"),
    }
}

fn reference(identity: &str) -> ConcreteReferenceSpec {
    ConcreteReferenceSpec {
        profile: String::from(CANONICAL_REFERENCE_PROFILE_V1),
        identity: String::from(identity),
    }
}

fn machine() -> AdmittedMachine {
    let mut spec = MachineSpecV2::new(
        vec![NodeSpec::new(NODE, vec![String::from(OPERATION)])],
        Vec::new(),
    );
    spec.effects.push(ConcreteEffectDeclarationSpec {
        declaration_key: String::from(DECLARATION_KEY),
        node_id: String::from(NODE),
        transition: TransitionSelector {
            from: LocalState::Ready,
            action: Action::Start,
        },
        operation: String::from(OPERATION),
        resource: String::from(RESOURCE),
        ability: String::from(ABILITY),
        request_binding: ReferenceBindingSpec::Slot {
            slot_id: String::from(REQUEST_SLOT),
            source_ref: String::from(REQUEST_SOURCE),
        },
        payload_binding: ReferenceBindingSpec::Exact {
            reference: reference(PAYLOAD_REFERENCE),
        },
    });
    must(AdmittedMachine::admit_v2(spec, AdmissionLimits::default()))
}

fn effect_request(request_identity: &str) -> ConcreteEffectRequest {
    ConcreteEffectRequest {
        declaration_key: String::from(DECLARATION_KEY),
        request: ConcreteBindingInput {
            slot_id: Some(String::from(REQUEST_SLOT)),
            source_ref: Some(String::from(REQUEST_SOURCE)),
            reference: reference(request_identity),
        },
        payload: ConcreteBindingInput {
            slot_id: None,
            source_ref: None,
            reference: reference(PAYLOAD_REFERENCE),
        },
    }
}

fn intent(
    machine: &AdmittedMachine,
    request_identity: &str,
) -> choregraph_core::ConcreteEffectIntent {
    let ready = match step_concrete(
        machine,
        &machine.initial_state(),
        &ConcreteEvent::new(NODE, Action::MarkReady),
    ) {
        ConcreteDecision::Accept { next, intents } => {
            assert!(intents.is_empty());
            next
        }
        ConcreteDecision::Deny { reason } => panic!("mark-ready denied: {reason:?}"),
    };
    let mut start = ConcreteEvent::new(NODE, Action::Start);
    start.effects.push(effect_request(request_identity));
    match step_concrete(machine, &ready, &start) {
        ConcreteDecision::Accept { mut intents, .. } => match intents.pop() {
            Some(intent) => intent,
            None => panic!("accepted concrete step emitted no intent"),
        },
        ConcreteDecision::Deny { reason } => panic!("effectful start denied: {reason:?}"),
    }
}

#[test]
fn v2_machine_intent_and_receipt_round_trip_canonically() {
    let machine = machine();
    let machine_artifact = must(build_concrete_machine_artifact(&machine));
    let validated_machine = must(validate_concrete_machine_artifact(
        &machine_artifact.canonical.packed_bytes,
        &machine_artifact.identity,
    ));
    assert_eq!(validated_machine.machine, machine);

    let intent = intent(&machine, REQUEST_REFERENCE_A);
    let intent_artifact = must(build_concrete_intent_artifact(&machine, &intent));
    let validated_intent = must(validate_concrete_intent_artifact(
        &intent_artifact.canonical.packed_bytes,
        &intent_artifact.identity,
        &machine,
    ));
    assert_eq!(validated_intent.intent, intent);

    let receipt = must(build_concrete_intent_receipt(
        &machine_artifact,
        &intent_artifact,
    ));
    must(validate_concrete_intent_receipt(
        &receipt.canonical.packed_bytes,
        &receipt.identity,
        &machine_artifact,
        &intent_artifact,
    ));
    assert!(
        intent_artifact
            .descriptor
            .non_claims
            .iter()
            .any(|claim| claim.contains("not a run occurrence"))
    );
}

#[test]
fn machine_identity_stays_stable_when_content_changes() {
    let machine = machine();
    let machine_artifact = must(build_concrete_machine_artifact(&machine));
    let first = must(build_concrete_intent_artifact(
        &machine,
        &intent(&machine, REQUEST_REFERENCE_A),
    ));
    let same = must(build_concrete_intent_artifact(
        &machine,
        &intent(&machine, REQUEST_REFERENCE_A),
    ));
    let changed = must(build_concrete_intent_artifact(
        &machine,
        &intent(&machine, REQUEST_REFERENCE_B),
    ));
    assert_eq!(first.identity, same.identity);
    assert_ne!(first.identity, changed.identity);
    assert_eq!(
        machine_artifact.identity,
        must(build_concrete_machine_artifact(&machine)).identity
    );
}

#[test]
fn stale_json_overclaim_and_exact_reference_mutation_fail_closed() {
    let machine = machine();
    let machine_artifact = must(build_concrete_machine_artifact(&machine));
    let intent_artifact = must(build_concrete_intent_artifact(
        &machine,
        &intent(&machine, REQUEST_REFERENCE_A),
    ));
    let receipt = must(build_concrete_intent_receipt(
        &machine_artifact,
        &intent_artifact,
    ));

    let stale_machine = must(compute_identity(ArtifactDomain::Machine, b"stale-machine"));
    assert!(matches!(
        validate_concrete_machine_artifact(
            &machine_artifact.canonical.packed_bytes,
            &stale_machine,
        ),
        Err(ConcreteArtifactError::StaleIdentity)
    ));

    let stale_intent = must(compute_identity(
        ArtifactDomain::ConcreteIntent,
        b"stale-intent",
    ));
    assert!(matches!(
        validate_concrete_intent_artifact(
            &intent_artifact.canonical.packed_bytes,
            &stale_intent,
            &machine,
        ),
        Err(ConcreteArtifactError::StaleIdentity)
    ));

    assert!(matches!(
        validate_concrete_machine_artifact(
            br#"{\"schema_version\":1}"#,
            &machine_artifact.identity
        ),
        Err(ConcreteArtifactError::Codec(_))
    ));

    let mut mutated = intent_artifact.descriptor.clone();
    mutated.payload.identity = String::from(REQUEST_REFERENCE_B);
    let mutated_canonical = must(encode_concrete_intent_descriptor(&mutated));
    let mutated_identity = must(compute_identity(
        ArtifactDomain::ConcreteIntent,
        &mutated_canonical.packed_bytes,
    ));
    assert!(matches!(
        validate_concrete_intent_artifact(
            &mutated_canonical.packed_bytes,
            &mutated_identity,
            &machine,
        ),
        Err(ConcreteArtifactError::IntentMismatch)
    ));

    let mut overclaim = receipt.body.clone();
    overclaim
        .semantic_claims
        .push(String::from("runtime effect is authorized"));
    let overclaim_canonical = must(encode_concrete_intent_receipt_body(&overclaim));
    let overclaim_identity = must(compute_identity(
        ArtifactDomain::Receipt,
        &overclaim_canonical.packed_bytes,
    ));
    assert!(matches!(
        validate_concrete_intent_receipt(
            &overclaim_canonical.packed_bytes,
            &overclaim_identity,
            &machine_artifact,
            &intent_artifact,
        ),
        Err(ConcreteArtifactError::Overclaim)
    ));
}
