#![allow(
    unknown_lints,
    ambient_env,
    assertion_density,
    explicit_defaults,
    function_length,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    reason = "the bounded conformance shell captures one explicit component path and uses compact fixture builders to compare native and engine bytes"
)]

// r[verify choregraph.wasm.parity]
// r[verify choregraph.wasm.bounds]
// r[verify choregraph.wasm.boundary]
use std::error::Error;
use std::fs;
use std::path::Path;

use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, CANONICAL_REFERENCE_PROFILE_V1,
    ConcreteEffectDeclarationSpec, DURABLE_SUSPENSION_V1, EdgeSpec, LocalState, MachineSpec,
    MachineSpecV2, NodeSpec, ReferenceBindingSpec, SuspensionDeclarationSpec, TransitionSelector,
};
use choregraph_kamacite::{
    ArtifactDomain, IdentityMaterials, build_artifact_set, build_concrete_machine_artifact,
    build_durable_lifecycle_artifact, build_profile_artifact, compute_identity,
    durable_machine_descriptor, encode_durable_machine_descriptor,
};
use choregraph_wasm::{
    ABI_VERSION, ActionTag, ConcreteBindingValue, ConcreteEffectRequestValue, ConcreteEventValue,
    ConcreteReferenceValue, EffectRequestValue, EventValue, KernelCall, MAX_OUTER_REQUEST_BYTES,
    MAX_OUTER_RESULT_BYTES, MAX_TRACE_EVENTS, MachineArtifactKind, MachineArtifacts,
    OperationRequest, ReplayBindingPairValue, ReplayBindingsValue, StateEntryValue, StateTag,
    StateValue, dispatch, encode_call,
};
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};

wasmtime::component::bindgen!({
    path: "../../crates/choregraph-wasm/wit",
    world: "kernel",
});

const NODE: &str = "conformance-node";
const NODE_B: &str = "conformance-node-b";
const NODE_C: &str = "conformance-node-c";
const NODE_D: &str = "conformance-node-d";
const OPERATION: &str = "http/request";
const RESOURCE: &str = "service/example";
const ABILITY: &str = "http/request";
const DECLARATION_KEY: &str = "request/main";
const REQUEST_SLOT: &str = "request/input";
const REQUEST_SOURCE: &str = "node/input/request";
const REFERENCE: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SUSPENSION_CLASS: &str = "approval/manual";
const SUSPENSION_REFERENCE: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TRACE_BYTE: u8 = 17;
const ABI_MISMATCH_VERSION: u32 = ABI_VERSION.saturating_add(1);
const MAX_COMPONENT_BYTES: usize = 16 * 1_048_576;
const MAX_ENGINE_MEMORY_BYTES: usize = 64 * 1_048_576;
const MAX_ENGINE_INSTANCES: usize = 1;
const MAX_ENGINE_MEMORIES: usize = 2;
const MAX_ENGINE_TABLES: usize = 4;
const MAX_ENGINE_FUEL: u64 = 50_000_000;
const WASMTIME_VERSION: &str = "45.0.0";
const COMPONENT_IDENTITY_CONTEXT: &str = "choregraph wasm component bytes v1";
const CORPUS_IDENTITY_CONTEXT: &str = "choregraph wasm parity corpus v1";
const FIXTURE_IDENTITY_CONTEXT: &str = "choregraph wasm parity fixture v1";
const SOURCE_ENVELOPE: &[u8] = b"wasm-conformance-source-v1";
const PROOF_SET: &[u8] = b"wasm-conformance-proof-v1";
const GENERATOR: &[u8] = b"wasm-conformance-generator-v1";
const ADAPTER: &[u8] = b"wasm-conformance-adapter-v1";
const TOOLCHAIN: &[u8] = b"wasm-conformance-toolchain-v1";

struct HostState {
    limits: StoreLimits,
}

struct Fixture {
    name: &'static str,
    request: Vec<u8>,
}

type AppResult<T> = Result<T, Box<dyn Error>>;

fn main() -> AppResult<()> {
    let component_path = component_path()?;
    let component_bytes = fs::read(&component_path)?;
    if component_bytes.len() > MAX_COMPONENT_BYTES {
        return Err(failure("component exceeds the selected byte limit"));
    }
    let engine = engine()?;
    let component = Component::new(&engine, &component_bytes)?;
    let linker = Linker::new(&engine);
    let fixtures = corpus()?;
    let mut reused_store = store(&engine)?;
    let reused = Kernel::instantiate(&mut reused_store, &component, &linker)?;

    let mut corpus_hasher = blake3::Hasher::new_derive_key(CORPUS_IDENTITY_CONTEXT);
    for fixture in &fixtures {
        if fixture.request.len() > MAX_OUTER_REQUEST_BYTES {
            return Err(failure(
                "conformance request exceeds the host request limit",
            ));
        }
        let native = dispatch(&fixture.request);
        let fresh = call_fresh(&engine, &component, &linker, &fixture.request)?;
        reused_store.set_fuel(MAX_ENGINE_FUEL)?;
        let reused_result = reused.call_call(&mut reused_store, &fixture.request)?;
        if native.len() > MAX_OUTER_RESULT_BYTES
            || fresh.len() > MAX_OUTER_RESULT_BYTES
            || reused_result.len() > MAX_OUTER_RESULT_BYTES
        {
            return Err(failure("conformance result exceeds the host result limit"));
        }
        if native != fresh || native != reused_result {
            let fixture_identity = identity(FIXTURE_IDENTITY_CONTEXT, &fixture.request);
            return Err(failure(&format!(
                "fixture {} ({fixture_identity}) mismatch class packed-result-bytes",
                fixture.name
            )));
        }
        corpus_hasher.update(fixture.name.as_bytes());
        corpus_hasher.update(&usize_to_u64(fixture.request.len()).to_le_bytes());
        corpus_hasher.update(&fixture.request);
        println!(
            "fixture={} identity={} verdict=byte-equal",
            fixture.name,
            identity(FIXTURE_IDENTITY_CONTEXT, &fixture.request)
        );
    }

    println!(
        "component-identity={}",
        identity(COMPONENT_IDENTITY_CONTEXT, &component_bytes)
    );
    println!("corpus-identity={}", corpus_hasher.finalize().to_hex());
    println!("engine=wasmtime-{WASMTIME_VERSION}");
    println!("imports=none exports=call verdict=pass");
    println!("non-claim=engine-compiler-sandbox-effect-and-release-correctness");
    Ok(())
}

fn component_path() -> AppResult<String> {
    let mut arguments = std::env::args();
    let _program = arguments.next();
    let Some(path) = arguments.next() else {
        return Err(failure("usage: choregraph-wasm-conformance COMPONENT"));
    };
    if arguments.next().is_some() || !Path::new(&path).is_file() {
        return Err(failure("component argument must name one file"));
    }
    Ok(path)
}

fn engine() -> AppResult<Engine> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    Ok(Engine::new(&config)?)
}

fn store(engine: &Engine) -> AppResult<Store<HostState>> {
    let limits = StoreLimitsBuilder::new()
        .memory_size(MAX_ENGINE_MEMORY_BYTES)
        .instances(MAX_ENGINE_INSTANCES)
        .memories(MAX_ENGINE_MEMORIES)
        .tables(MAX_ENGINE_TABLES)
        .build();
    let mut store = Store::new(engine, HostState { limits });
    store.limiter(|state| &mut state.limits);
    store.set_fuel(MAX_ENGINE_FUEL)?;
    Ok(store)
}

fn call_fresh(
    engine: &Engine,
    component: &Component,
    linker: &Linker<HostState>,
    request: &[u8],
) -> AppResult<Vec<u8>> {
    let mut store = store(engine)?;
    let bindings = Kernel::instantiate(&mut store, component, linker)?;
    Ok(bindings.call_call(&mut store, request)?)
}

fn corpus() -> AppResult<Vec<Fixture>> {
    let linear = linear_artifacts()?;
    let empty = empty_artifacts()?;
    let diamond = diamond_artifacts(false)?;
    let reordered_diamond = diamond_artifacts(true)?;
    if diamond.machine_identity != reordered_diamond.machine_identity {
        return Err(failure("reordered diamond artifact identity mismatch"));
    }
    let durable = durable_artifacts()?;
    let concrete = concrete_artifacts()?;
    let mut stale = linear.clone();
    stale.machine_identity.hex =
        String::from("0000000000000000000000000000000000000000000000000000000000000000");
    let mut wrong_schema = linear.clone();
    wrong_schema.kind = MachineArtifactKind::DurableV1;
    Ok(vec![
        Fixture {
            name: "admit-empty",
            request: call(OperationRequest::AdmitMachine { artifacts: empty })?,
        },
        Fixture {
            name: "admit-linear",
            request: call(OperationRequest::AdmitMachine {
                artifacts: linear.clone(),
            })?,
        },
        Fixture {
            name: "admit-diamond",
            request: call(OperationRequest::AdmitMachine { artifacts: diamond })?,
        },
        Fixture {
            name: "admit-reordered-diamond",
            request: call(OperationRequest::AdmitMachine {
                artifacts: reordered_diamond,
            })?,
        },
        Fixture {
            name: "admit-durable",
            request: call(OperationRequest::AdmitMachine { artifacts: durable })?,
        },
        Fixture {
            name: "step-accepted",
            request: call(OperationRequest::Step {
                artifacts: linear.clone(),
                state: state(StateTag::Pending),
                event: event(ActionTag::MarkReady),
            })?,
        },
        Fixture {
            name: "step-denied",
            request: call(OperationRequest::Step {
                artifacts: linear.clone(),
                state: state(StateTag::Completed),
                event: event(ActionTag::Start),
            })?,
        },
        Fixture {
            name: "replay-complete",
            request: call(OperationRequest::Replay {
                artifacts: linear.clone(),
                initial_state: state(StateTag::Pending),
                bindings: bindings(),
                max_events: MAX_TRACE_EVENTS,
                events: vec![event(ActionTag::MarkReady)],
            })?,
        },
        Fixture {
            name: "replay-first-denial",
            request: call(OperationRequest::Replay {
                artifacts: linear.clone(),
                initial_state: state(StateTag::Pending),
                bindings: bindings(),
                max_events: MAX_TRACE_EVENTS,
                events: vec![event(ActionTag::Start)],
            })?,
        },
        Fixture {
            name: "replay-binding-mismatch",
            request: call(OperationRequest::Replay {
                artifacts: linear.clone(),
                initial_state: state(StateTag::Pending),
                bindings: mismatched_bindings(),
                max_events: MAX_TRACE_EVENTS,
                events: Vec::new(),
            })?,
        },
        Fixture {
            name: "replay-limit-rejected",
            request: call(OperationRequest::Replay {
                artifacts: linear.clone(),
                initial_state: state(StateTag::Pending),
                bindings: bindings(),
                max_events: MAX_TRACE_EVENTS.saturating_add(1),
                events: Vec::new(),
            })?,
        },
        Fixture {
            name: "invalid-state",
            request: call(OperationRequest::Step {
                artifacts: linear.clone(),
                state: StateValue {
                    entries: Vec::new(),
                    suspensions: Vec::new(),
                },
                event: event(ActionTag::MarkReady),
            })?,
        },
        Fixture {
            name: "undeclared-effect",
            request: call(OperationRequest::Step {
                artifacts: linear.clone(),
                state: state(StateTag::Pending),
                event: undeclared_effect_event(),
            })?,
        },
        Fixture {
            name: "duplicate-effect",
            request: call(OperationRequest::Step {
                artifacts: linear,
                state: state(StateTag::Pending),
                event: duplicate_effect_event(),
            })?,
        },
        Fixture {
            name: "stale-machine-identity",
            request: call(OperationRequest::AdmitMachine { artifacts: stale })?,
        },
        Fixture {
            name: "wrong-machine-schema",
            request: call(OperationRequest::AdmitMachine {
                artifacts: wrong_schema,
            })?,
        },
        Fixture {
            name: "admit-concrete",
            request: call(OperationRequest::AdmitMachine {
                artifacts: concrete.clone(),
            })?,
        },
        Fixture {
            name: "step-concrete-accepted",
            request: call(OperationRequest::StepConcrete {
                artifacts: concrete.clone(),
                state: state(StateTag::Pending),
                event: concrete_event(ActionTag::MarkReady),
            })?,
        },
        Fixture {
            name: "step-concrete-intent",
            request: call(OperationRequest::StepConcrete {
                artifacts: concrete.clone(),
                state: state(StateTag::Ready),
                event: concrete_intent_event(),
            })?,
        },
        Fixture {
            name: "replay-concrete-complete",
            request: call(OperationRequest::ReplayConcrete {
                artifacts: concrete,
                initial_state: state(StateTag::Pending),
                bindings: bindings(),
                max_events: MAX_TRACE_EVENTS,
                events: vec![concrete_event(ActionTag::MarkReady)],
            })?,
        },
        Fixture {
            name: "malformed-json",
            request: br#"{"abi_version":1}"#.to_vec(),
        },
        Fixture {
            name: "unsupported-version",
            request: encode_call(&KernelCall {
                abi_version: ABI_MISMATCH_VERSION,
                operation: OperationRequest::AdmitMachine {
                    artifacts: linear_artifacts()?,
                },
            })
            .map_err(|message| failure(&message))?,
        },
    ])
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

fn linear_artifacts_from(spec: MachineSpec) -> AppResult<MachineArtifacts> {
    let machine = must(AdmittedMachine::admit(spec, AdmissionLimits::default()))?;
    let artifact_set = must(build_artifact_set(&machine, materials()))?;
    Ok(MachineArtifacts {
        kind: MachineArtifactKind::LinearV1,
        machine: artifact_set.machine_artifact.packed_bytes,
        machine_identity: artifact_set.identities.machine,
        lifecycle: Some(artifact_set.lifecycle.canonical.packed_bytes),
        lifecycle_identity: Some(artifact_set.identities.lifecycle),
        profile_identity: Some(artifact_set.identities.profile),
    })
}

fn empty_artifacts() -> AppResult<MachineArtifacts> {
    linear_artifacts_from(MachineSpec::new(Vec::new(), Vec::new()))
}

fn linear_artifacts() -> AppResult<MachineArtifacts> {
    linear_artifacts_from(MachineSpec::new(
        vec![NodeSpec::new(NODE, Vec::new())],
        Vec::new(),
    ))
}

fn diamond_artifacts(reordered: bool) -> AppResult<MachineArtifacts> {
    let nodes = if reordered {
        vec![
            NodeSpec::new(NODE_D, Vec::new()),
            NodeSpec::new(NODE_C, Vec::new()),
            NodeSpec::new(NODE_B, Vec::new()),
            NodeSpec::new(NODE, Vec::new()),
        ]
    } else {
        vec![
            NodeSpec::new(NODE, Vec::new()),
            NodeSpec::new(NODE_B, Vec::new()),
            NodeSpec::new(NODE_C, Vec::new()),
            NodeSpec::new(NODE_D, Vec::new()),
        ]
    };
    let edges = if reordered {
        vec![
            EdgeSpec::new(NODE_C, NODE_D),
            EdgeSpec::new(NODE_B, NODE_D),
            EdgeSpec::new(NODE, NODE_C),
            EdgeSpec::new(NODE, NODE_B),
        ]
    } else {
        vec![
            EdgeSpec::new(NODE, NODE_B),
            EdgeSpec::new(NODE, NODE_C),
            EdgeSpec::new(NODE_B, NODE_D),
            EdgeSpec::new(NODE_C, NODE_D),
        ]
    };
    linear_artifacts_from(MachineSpec::new(nodes, edges))
}

fn durable_artifacts() -> AppResult<MachineArtifacts> {
    let mut spec = MachineSpec::new(vec![NodeSpec::new(NODE, Vec::new())], Vec::new());
    spec.lifecycle_profile = String::from(DURABLE_SUSPENSION_V1);
    spec.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from(NODE),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(SUSPENSION_REFERENCE),
    });
    let machine = must(AdmittedMachine::admit(spec, AdmissionLimits::default()))?;
    let lifecycle = must(build_durable_lifecycle_artifact(None))?;
    let (_profile, _profile_artifact, profile_identity) = must(build_profile_artifact())?;
    let descriptor = must(durable_machine_descriptor(
        &machine,
        lifecycle.project_identity.clone(),
        profile_identity.clone(),
    ))?;
    let artifact = must(encode_durable_machine_descriptor(&descriptor))?;
    let machine_identity = must(compute_identity(
        ArtifactDomain::Machine,
        &artifact.packed_bytes,
    ))?;
    Ok(MachineArtifacts {
        kind: MachineArtifactKind::DurableV1,
        machine: artifact.packed_bytes,
        machine_identity,
        lifecycle: Some(lifecycle.canonical.packed_bytes),
        lifecycle_identity: Some(lifecycle.project_identity),
        profile_identity: Some(profile_identity),
    })
}

fn concrete_artifacts() -> AppResult<MachineArtifacts> {
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
                identity: String::from(REFERENCE),
            },
        },
    });
    let machine = must(AdmittedMachine::admit_v2(spec, AdmissionLimits::default()))?;
    let artifact = must(build_concrete_machine_artifact(&machine))?;
    Ok(MachineArtifacts {
        kind: MachineArtifactKind::ConcreteV2,
        machine: artifact.canonical.packed_bytes,
        machine_identity: artifact.identity,
        lifecycle: None,
        lifecycle_identity: None,
        profile_identity: None,
    })
}

fn state(value: StateTag) -> StateValue {
    StateValue {
        entries: vec![StateEntryValue {
            node_id: String::from(NODE),
            state: value,
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

fn undeclared_effect_request() -> EffectRequestValue {
    EffectRequestValue {
        operation: String::from(OPERATION),
        resource: String::from(RESOURCE),
        ability: String::from(ABILITY),
        request: String::from(REFERENCE),
        payload: String::from(REFERENCE),
    }
}

fn undeclared_effect_event() -> EventValue {
    EventValue {
        node_id: String::from(NODE),
        action: ActionTag::MarkReady,
        effects: vec![undeclared_effect_request()],
        suspension: None,
    }
}

fn duplicate_effect_event() -> EventValue {
    let request = undeclared_effect_request();
    EventValue {
        node_id: String::from(NODE),
        action: ActionTag::MarkReady,
        effects: vec![request.clone(), request],
        suspension: None,
    }
}

fn concrete_reference() -> ConcreteReferenceValue {
    ConcreteReferenceValue {
        profile: String::from(CANONICAL_REFERENCE_PROFILE_V1),
        identity: String::from(REFERENCE),
    }
}

fn concrete_intent_event() -> ConcreteEventValue {
    ConcreteEventValue {
        node_id: String::from(NODE),
        action: ActionTag::Start,
        effects: vec![ConcreteEffectRequestValue {
            declaration_key: String::from(DECLARATION_KEY),
            request: ConcreteBindingValue {
                slot_id: Some(String::from(REQUEST_SLOT)),
                source_ref: Some(String::from(REQUEST_SOURCE)),
                reference: concrete_reference(),
            },
            payload: ConcreteBindingValue {
                slot_id: None,
                source_ref: None,
                reference: concrete_reference(),
            },
        }],
        suspension: None,
    }
}

fn bindings() -> ReplayBindingPairValue {
    let value = ReplayBindingsValue {
        machine: vec![TRACE_BYTE; choregraph_core::TRACE_BINDING_BYTES],
        initial_state: vec![TRACE_BYTE; choregraph_core::TRACE_BINDING_BYTES],
    };
    ReplayBindingPairValue {
        expected: value.clone(),
        trace: value,
    }
}

fn mismatched_bindings() -> ReplayBindingPairValue {
    let mut pair = bindings();
    pair.trace.machine[0] = TRACE_BYTE.saturating_add(1);
    pair
}

fn call(operation: OperationRequest) -> AppResult<Vec<u8>> {
    encode_call(&KernelCall {
        abi_version: ABI_VERSION,
        operation,
    })
    .map_err(|message| failure(&message))
}

fn identity(context: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> AppResult<T> {
    result.map_err(|error| failure(&format!("fixture construction failed: {error:?}")))
}

fn failure(message: &str) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message.to_owned()))
}

fn usize_to_u64(value: usize) -> u64 {
    value as u64
}
