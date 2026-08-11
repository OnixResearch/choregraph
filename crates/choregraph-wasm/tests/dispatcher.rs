#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    explicit_defaults,
    function_length,
    no_panic,
    non_trait_imports,
    numeric_units,
    raw_arithmetic_overflow,
    reason = "the bounded positive and negative integration corpus uses compact fixture builders, explicit boundary arithmetic, and panic-on-test-failure helpers"
)]

use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, CANONICAL_REFERENCE_PROFILE_V1,
    ConcreteEffectDeclarationSpec, DURABLE_SUSPENSION_V1, LocalState, MachineSpec, MachineSpecV2,
    NodeSpec, ReferenceBindingSpec, SuspensionDeclarationSpec, TransitionSelector,
};
use choregraph_kamacite::{
    ArtifactDomain, ArtifactIdentity, IdentityMaterials, build_artifact_set,
    build_concrete_machine_artifact, build_durable_lifecycle_artifact, build_profile_artifact,
    compute_identity, durable_machine_descriptor, encode_durable_machine_descriptor,
};
use preserves::PackedWriter;
use serde::Serialize;

use choregraph_wasm::{
    ABI_VERSION, ActionTag, AdapterErrorCode, ConcreteBindingValue, ConcreteEffectRequestValue,
    ConcreteEventValue, ConcreteReferenceValue, EventValue, IntentValue, KernelCall,
    MAX_OUTER_REQUEST_BYTES, MAX_TRACE_EVENTS, MachineArtifactKind, MachineArtifacts,
    OperationRequest, Outcome, ReplayBindingPairValue, ReplayBindingsValue, StateEntryValue,
    StateTag, StateValue, decode_result, dispatch, encode_call,
};

const NODE: &str = "effect-node";
const OPERATION: &str = "http/request";
const RESOURCE: &str = "service/example";
const ABILITY: &str = "http/request";
const DECLARATION_KEY: &str = "request/main";
const REQUEST_SLOT: &str = "request/input";
const REQUEST_SOURCE: &str = "node/input/request";
const REQUEST_REFERENCE: &str =
    "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SUSPENSION_CLASS: &str = "approval/manual";
const SUSPENSION_REFERENCE: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PAYLOAD_REFERENCE: &str =
    "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TRACE_BYTE: u8 = 7;
const WRONG_TRACE_BYTE: u8 = 8;
const REQUEST_PADDING_BYTE: u8 = 0;
const SOURCE_ENVELOPE: &[u8] = b"wasm-source-envelope-v1";
const PROOF_SET: &[u8] = b"wasm-proof-reference-v1";
const GENERATOR: &[u8] = b"wasm-generator-v1";
const ADAPTER: &[u8] = b"wasm-adapter-v1";
const TOOLCHAIN: &[u8] = b"wasm-toolchain-v1";

fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("expected success, got {error:?}"),
    }
}

fn materials() -> IdentityMaterials<'static> {
    IdentityMaterials {
        source_envelope: SOURCE_ENVELOPE,
        proof_set: PROOF_SET,
        generator: GENERATOR,
        adapter: ADAPTER,
        toolchain: TOOLCHAIN,
    }
}

fn linear_artifacts() -> MachineArtifacts {
    let machine = must(AdmittedMachine::admit(
        MachineSpec::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new()),
        AdmissionLimits::default(),
    ));
    let artifact_set = must(build_artifact_set(&machine, materials()));
    MachineArtifacts {
        kind: MachineArtifactKind::LinearV1,
        machine: artifact_set.machine_artifact.packed_bytes,
        machine_identity: artifact_set.identities.machine,
        lifecycle: Some(artifact_set.lifecycle.canonical.packed_bytes),
        lifecycle_identity: Some(artifact_set.identities.lifecycle),
        profile_identity: Some(artifact_set.identities.profile),
    }
}

fn durable_artifacts() -> MachineArtifacts {
    let mut spec = MachineSpec::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new());
    spec.lifecycle_profile = String::from(DURABLE_SUSPENSION_V1);
    spec.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from(NODE),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(SUSPENSION_REFERENCE),
    });
    let machine = must(AdmittedMachine::admit(spec, AdmissionLimits::default()));
    let lifecycle = must(build_durable_lifecycle_artifact(None));
    let (_profile, _profile_artifact, profile_identity) = must(build_profile_artifact());
    let descriptor = must(durable_machine_descriptor(
        &machine,
        lifecycle.project_identity.clone(),
        profile_identity.clone(),
    ));
    let artifact = must(encode_durable_machine_descriptor(&descriptor));
    let machine_identity = must(compute_identity(
        ArtifactDomain::Machine,
        &artifact.packed_bytes,
    ));
    MachineArtifacts {
        kind: MachineArtifactKind::DurableV1,
        machine: artifact.packed_bytes,
        machine_identity,
        lifecycle: Some(lifecycle.canonical.packed_bytes),
        lifecycle_identity: Some(lifecycle.project_identity),
        profile_identity: Some(profile_identity),
    }
}

fn reference(identity: &str) -> ConcreteReferenceValue {
    ConcreteReferenceValue {
        profile: String::from(CANONICAL_REFERENCE_PROFILE_V1),
        identity: String::from(identity),
    }
}

fn concrete_artifacts() -> MachineArtifacts {
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
            reference: choregraph_core::ConcreteReferenceSpec {
                profile: String::from(CANONICAL_REFERENCE_PROFILE_V1),
                identity: String::from(PAYLOAD_REFERENCE),
            },
        },
    });
    let machine = must(AdmittedMachine::admit_v2(spec, AdmissionLimits::default()));
    let artifact = must(build_concrete_machine_artifact(&machine));
    MachineArtifacts {
        kind: MachineArtifactKind::ConcreteV2,
        machine: artifact.canonical.packed_bytes,
        machine_identity: artifact.identity,
        lifecycle: None,
        lifecycle_identity: None,
        profile_identity: None,
    }
}

fn state(state: StateTag) -> StateValue {
    StateValue {
        entries: vec![StateEntryValue {
            node_id: String::from(NODE),
            state,
        }],
        suspensions: Vec::new(),
    }
}

fn event(action: ActionTag) -> EventValue {
    EventValue {
        node_id: String::from(NODE),
        action,
        effects: Vec::new(),
        suspension: None,
    }
}

fn concrete_event(action: ActionTag) -> ConcreteEventValue {
    ConcreteEventValue {
        node_id: String::from(NODE),
        action,
        effects: Vec::new(),
        suspension: None,
    }
}

fn bindings(machine_byte: u8, trace_byte: u8) -> ReplayBindingPairValue {
    ReplayBindingPairValue {
        expected: ReplayBindingsValue {
            machine: vec![machine_byte; choregraph_core::TRACE_BINDING_BYTES],
            initial_state: vec![TRACE_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        },
        trace: ReplayBindingsValue {
            machine: vec![trace_byte; choregraph_core::TRACE_BINDING_BYTES],
            initial_state: vec![TRACE_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        },
    }
}

fn call(operation: OperationRequest) -> Vec<u8> {
    must(encode_call(&KernelCall {
        abi_version: ABI_VERSION,
        operation,
    }))
}

fn outcome(operation: OperationRequest) -> Outcome {
    must(decode_result(&dispatch(&call(operation)))).outcome
}

#[derive(Serialize)]
#[serde(rename = "choregraph-wasm-call-v1")]
struct UnknownOperationCall {
    abi_version: u32,
    operation: UnknownOperation,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum UnknownOperation {
    Unknown,
}

fn unknown_operation_call() -> Vec<u8> {
    let value = UnknownOperationCall {
        abi_version: ABI_VERSION,
        operation: UnknownOperation::Unknown,
    };
    let mut bytes = Vec::new();
    {
        let mut writer = PackedWriter::new(&mut bytes);
        must(value.serialize(&mut preserves::serde::ser::Serializer::new(&mut writer)));
    }
    bytes
}

#[test]
fn all_production_operations_return_canonical_deterministic_results() {
    let linear = linear_artifacts();
    let admission_call = call(OperationRequest::AdmitMachine {
        artifacts: linear.clone(),
    });
    assert_eq!(dispatch(&admission_call), dispatch(&admission_call));
    assert!(matches!(
        must(decode_result(&dispatch(&admission_call))).outcome,
        Outcome::Admitted { .. }
    ));

    assert!(matches!(
        outcome(OperationRequest::AdmitMachine {
            artifacts: durable_artifacts(),
        }),
        Outcome::Admitted { .. }
    ));

    assert!(matches!(
        outcome(OperationRequest::Step {
            artifacts: linear.clone(),
            state: state(StateTag::Pending),
            event: event(ActionTag::MarkReady),
        }),
        Outcome::Accepted { intents, .. } if intents.is_empty()
    ));
    assert!(matches!(
        outcome(OperationRequest::Replay {
            artifacts: linear,
            initial_state: state(StateTag::Pending),
            bindings: bindings(TRACE_BYTE, TRACE_BYTE),
            max_events: MAX_TRACE_EVENTS,
            events: vec![event(ActionTag::MarkReady)],
        }),
        Outcome::ReplayComplete { consumed, .. } if consumed == 1
    ));

    let concrete = concrete_artifacts();
    assert!(matches!(
        outcome(OperationRequest::StepConcrete {
            artifacts: concrete.clone(),
            state: state(StateTag::Pending),
            event: concrete_event(ActionTag::MarkReady),
        }),
        Outcome::Accepted { intents, .. } if intents.is_empty()
    ));
    let mut start = concrete_event(ActionTag::Start);
    start.effects.push(ConcreteEffectRequestValue {
        declaration_key: String::from(DECLARATION_KEY),
        request: ConcreteBindingValue {
            slot_id: Some(String::from(REQUEST_SLOT)),
            source_ref: Some(String::from(REQUEST_SOURCE)),
            reference: reference(REQUEST_REFERENCE),
        },
        payload: ConcreteBindingValue {
            slot_id: None,
            source_ref: None,
            reference: reference(PAYLOAD_REFERENCE),
        },
    });
    assert!(matches!(
        outcome(OperationRequest::StepConcrete {
            artifacts: concrete.clone(),
            state: state(StateTag::Ready),
            event: start,
        }),
        Outcome::Accepted { intents, .. }
            if matches!(intents.as_slice(), [IntentValue::Concrete { .. }])
    ));
    assert!(matches!(
        outcome(OperationRequest::ReplayConcrete {
            artifacts: concrete,
            initial_state: state(StateTag::Pending),
            bindings: bindings(TRACE_BYTE, TRACE_BYTE),
            max_events: MAX_TRACE_EVENTS,
            events: vec![concrete_event(ActionTag::MarkReady)],
        }),
        Outcome::ReplayComplete { consumed, .. } if consumed == 1
    ));
}

#[test]
fn malformed_stale_mismatched_and_denied_inputs_fail_closed() {
    let json = br#"{"abi_version":1}"#;
    assert!(matches!(
        must(decode_result(&dispatch(json))).outcome,
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::MalformedPreserves
    ));

    assert!(matches!(
        must(decode_result(&dispatch(&unknown_operation_call()))).outcome,
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::MalformedPreserves
    ));
    let unsupported = KernelCall {
        abi_version: ABI_VERSION + 1,
        operation: OperationRequest::AdmitMachine {
            artifacts: linear_artifacts(),
        },
    };
    assert!(matches!(
        must(decode_result(&dispatch(&must(encode_call(&unsupported))))).outcome,
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::UnsupportedAbiVersion
    ));

    let mut stale = linear_artifacts();
    stale.machine_identity = ArtifactIdentity {
        domain: stale.machine_identity.domain,
        algorithm: stale.machine_identity.algorithm.clone(),
        hex: String::from("0000000000000000000000000000000000000000000000000000000000000000"),
    };
    assert!(matches!(
        outcome(OperationRequest::AdmitMachine { artifacts: stale }),
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::IdentityMismatch
                || error.code == AdapterErrorCode::InvalidArtifact
    ));

    assert!(matches!(
        outcome(OperationRequest::Replay {
            artifacts: linear_artifacts(),
            initial_state: state(StateTag::Pending),
            bindings: bindings(TRACE_BYTE, WRONG_TRACE_BYTE),
            max_events: MAX_TRACE_EVENTS,
            events: Vec::new(),
        }),
        Outcome::ReplayFailed { .. }
    ));
    assert!(matches!(
        outcome(OperationRequest::Step {
            artifacts: linear_artifacts(),
            state: state(StateTag::Completed),
            event: event(ActionTag::Start),
        }),
        Outcome::Denied { .. }
    ));
    let mut duplicate_state = state(StateTag::Pending);
    duplicate_state.entries.push(StateEntryValue {
        node_id: String::from(NODE),
        state: StateTag::Ready,
    });
    assert!(matches!(
        outcome(OperationRequest::Step {
            artifacts: linear_artifacts(),
            state: duplicate_state,
            event: event(ActionTag::MarkReady),
        }),
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::InvalidState
    ));
    assert!(matches!(
        outcome(OperationRequest::StepConcrete {
            artifacts: linear_artifacts(),
            state: state(StateTag::Pending),
            event: concrete_event(ActionTag::MarkReady),
        }),
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::OperationProfileMismatch
    ));
}

#[test]
fn outer_and_trace_limits_stop_before_semantic_dispatch() {
    let oversized = vec![REQUEST_PADDING_BYTE; MAX_OUTER_REQUEST_BYTES + 1];
    assert!(matches!(
        must(decode_result(&dispatch(&oversized))).outcome,
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::RequestTooLarge
    ));
    assert!(matches!(
        outcome(OperationRequest::Replay {
            artifacts: linear_artifacts(),
            initial_state: state(StateTag::Pending),
            bindings: bindings(TRACE_BYTE, TRACE_BYTE),
            max_events: MAX_TRACE_EVENTS + 1,
            events: Vec::new(),
        }),
        Outcome::AdapterError { error }
            if error.code == AdapterErrorCode::TraceLimitExceeded
    ));
}
