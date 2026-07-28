#![allow(
    unknown_lints,
    missing_const_fn,
    non_trait_imports,
    reason = "consumer failure fixtures use runtime test-harness entry points and explicit public boundary imports"
)]

// r[verify choregraph.conformance.negative]
use choregraph_core::{
    Action, BoundedTrace, ConsumedCount, Decision, DenialReason, EffectIntent, EffectRequest,
    Event, EventIndex, LocalState, MachineState, ReplayBindings, ReplayDecision,
    ReplayFailureReason, ReplayInput, ReplayLimits, TRACE_BINDING_BYTES, TraceBinding, replay,
    step,
};
use choregraph_nickel::{
    FrontendIssueCode, TranslationMaterials, TranslationReceipts, build_translation_receipts,
    lower_evaluated_json, reviewed_frontend_limits, validate_restricted,
    validate_translation_receipts,
};

const DIAMOND_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/lattice-diamond.json");
const EFFECT_AUTHORITY_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/effect-authority.json");
const FORBIDDEN_IMPORT_SOURCE: &[u8] =
    include_bytes!("../../../fixtures/nickel/negative/forbidden-runtime-import.ncl");
const NODE_ROOT: &str = "lattice-node/1";
const NODE_SINK: &str = "lattice-node/4";
const EFFECT_NODE: &str = "root";
const EFFECT_OPERATION: &str = "file/write";
const EFFECT_RESOURCE: &str = "root/output";
const EFFECT_ABILITY: &str = "file/write";
const EFFECT_REQUEST: &str = "request-1";
const EFFECT_PAYLOAD: &str =
    "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MACHINE_BINDING_BYTE: u8 = 0x11;
const STATE_BINDING_BYTE: u8 = 0x22;
const REPLAY_EVENT_LIMIT: u64 = 16;
const DENIED_EVENT_INDEX: u64 = 3;
const ACCEPTED_PREFIX_COUNT: u64 = 3;
const TEST_MAX_SOURCE_BYTES: u64 = 65_536;
const ONE_INTENT: usize = 1;
const NO_INTENTS: usize = 0;
const SOURCE_V1: &[u8] = b"source-v1";
const SOURCE_V2: &[u8] = b"source-v2";
const IMPORT_CLOSURE: &[u8] = b"imports-v1";
const EVALUATOR: &[u8] = b"nickel-evaluator-v1";
const EVALUATED_OUTPUT: &[u8] = b"evaluated-output-v1";
const LOWERER: &[u8] = b"choregraph-lowerer-v1";
const CANONICAL_DESCRIPTOR: &[u8] = b"canonical-descriptor-v1";
const ADMITTED_IR: &[u8] = b"admitted-ir-v1";
const PROOF_SET: &[u8] = b"proof-set-v1";
const TOOLCHAIN: &[u8] = b"toolchain-v1";

#[allow(
    unknown_lints,
    no_panic,
    reason = "a conformance test must stop when a positive fixture unexpectedly rejects input"
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
    reason = "a conformance test must stop when a required setup transition is denied"
)]
fn accept(
    machine: &choregraph_core::AdmittedMachine,
    state: &MachineState,
    event: &Event,
) -> (MachineState, Vec<EffectIntent>) {
    match step(machine, state, event) {
        Decision::Accept { next, intents } => (next, intents),
        Decision::Deny { reason } => panic!("expected acceptance, got {reason:?}"),
    }
}

fn bindings() -> ReplayBindings {
    ReplayBindings::new(
        TraceBinding::from_bytes([MACHINE_BINDING_BYTE; TRACE_BINDING_BYTES]),
        TraceBinding::from_bytes([STATE_BINDING_BYTE; TRACE_BINDING_BYTES]),
    )
}

fn materials(source: &[u8]) -> TranslationMaterials<'_> {
    TranslationMaterials {
        source,
        import_closure: IMPORT_CLOSURE,
        evaluator: EVALUATOR,
        evaluated_output: EVALUATED_OUTPUT,
        lowerer: LOWERER,
        canonical_descriptor: CANONICAL_DESCRIPTOR,
        admitted_ir: ADMITTED_IR,
        proof_set: PROOF_SET,
        toolchain: TOOLCHAIN,
    }
}

fn effect_request() -> EffectRequest {
    EffectRequest {
        operation: String::from(EFFECT_OPERATION),
        resource: String::from(EFFECT_RESOURCE),
        ability: String::from(EFFECT_ABILITY),
        request: String::from(EFFECT_REQUEST),
        payload: String::from(EFFECT_PAYLOAD),
    }
}

#[test]
#[allow(
    unknown_lints,
    no_panic,
    reason = "the negative replay fixture must stop if the expected failure variant is absent"
)]
fn replay_rejects_the_first_invalid_suffix_event_without_dispatch_authority() {
    let lowered = must(lower_evaluated_json(
        DIAMOND_JSON,
        &[],
        reviewed_frontend_limits(),
    ));
    let machine = lowered.machine;
    let initial = machine.initial_state();
    let events = [
        Event::new(NODE_ROOT, Action::MarkReady),
        Event::new(NODE_ROOT, Action::Start),
        Event::new(NODE_ROOT, Action::Complete),
        Event::new(NODE_SINK, Action::MarkReady),
        Event::new(NODE_SINK, Action::Start),
    ];
    let identity_bindings = bindings();
    let decision = replay(ReplayInput::new(
        &machine,
        &initial,
        identity_bindings,
        BoundedTrace::new(identity_bindings, &events),
        ReplayLimits {
            max_events: REPLAY_EVENT_LIMIT,
        },
    ));

    let ReplayDecision::Failed(failure) = decision else {
        panic!("invalid replay suffix unexpectedly completed");
    };
    assert_eq!(
        failure.event_index,
        Some(EventIndex::new(DENIED_EVENT_INDEX))
    );
    assert_eq!(failure.consumed, ConsumedCount::new(ACCEPTED_PREFIX_COUNT));
    assert_eq!(
        failure.reason,
        ReplayFailureReason::TransitionDenied(DenialReason::DependenciesIncomplete)
    );
    let root_state = failure
        .last_valid_state
        .state()
        .entries()
        .iter()
        .find(|(node_id, _)| node_id.as_str() == NODE_ROOT)
        .map(|(_, state)| *state);
    let sink_state = failure
        .last_valid_state
        .state()
        .entries()
        .iter()
        .find(|(node_id, _)| node_id.as_str() == NODE_SINK)
        .map(|(_, state)| *state);
    assert_eq!(root_state, Some(LocalState::Completed));
    assert_eq!(sink_state, Some(LocalState::Pending));
}

#[test]
fn authorization_denial_result_fails_the_node_without_reusing_the_intent() {
    let lowered = must(lower_evaluated_json(
        EFFECT_AUTHORITY_JSON,
        &[],
        reviewed_frontend_limits(),
    ));
    let machine = lowered.machine;
    let initial = machine.initial_state();
    let (ready, ready_intents) = accept(
        &machine,
        &initial,
        &Event::new(EFFECT_NODE, Action::MarkReady),
    );
    assert_eq!(ready_intents.len(), NO_INTENTS);

    let mut start = Event::new(EFFECT_NODE, Action::Start);
    start.effects.push(effect_request());
    let (running, start_intents) = accept(&machine, &ready, &start);
    assert_eq!(start_intents.len(), ONE_INTENT);

    let (failed, denial_result_intents) =
        accept(&machine, &running, &Event::new(EFFECT_NODE, Action::Fail));
    assert_eq!(denial_result_intents.len(), NO_INTENTS);
    assert_eq!(
        failed.entries().values().copied().collect::<Vec<_>>(),
        [LocalState::Failed]
    );
}

#[test]
fn stale_source_and_mismatched_translation_receipts_fail_closed() {
    let baseline = must(build_translation_receipts(materials(SOURCE_V1)));
    let stale_source = validate_translation_receipts(&baseline, materials(SOURCE_V2));
    assert_eq!(
        stale_source.err().map(|error| error.issues[0].code()),
        Some(FrontendIssueCode::StaleTranslationReceipt)
    );

    let alternate = must(build_translation_receipts(materials(SOURCE_V2)));
    let mismatched = TranslationReceipts {
        source: baseline.source,
        evaluator: alternate.evaluator,
    };
    let receipt_mismatch = validate_translation_receipts(&mismatched, materials(SOURCE_V1));
    assert_eq!(
        receipt_mismatch.err().map(|error| error.issues[0].code()),
        Some(FrontendIssueCode::StaleTranslationReceipt)
    );
}

#[test]
fn malformed_graph_and_forbidden_import_have_no_fallback_path() {
    let malformed = lower_evaluated_json(b"{}", &[], reviewed_frontend_limits());
    assert_eq!(
        malformed.err().map(|error| error.issues[0].code()),
        Some(FrontendIssueCode::MalformedEvaluatedValue)
    );

    let forbidden = validate_restricted(FORBIDDEN_IMPORT_SOURCE, TEST_MAX_SOURCE_BYTES);
    assert_eq!(
        forbidden.err().map(|error| error.issues[0].code()),
        Some(FrontendIssueCode::ForbiddenNickelAuthority)
    );
}
