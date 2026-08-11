#![allow(
    unknown_lints,
    assertion_density,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    reason = "the pure ABI adapter returns bounded errors instead of panicking, exhaustively maps sealed wire enums, and keeps domain DTO names visible at conversion sites"
)]

// r[impl choregraph.wasm.module]
// r[impl choregraph.wasm.abi]
// r[impl choregraph.wasm.bounds]
use std::collections::BTreeMap;

use choregraph_core::{
    Action, AdmittedMachine, BoundedTrace, ConcreteBindingInput, ConcreteBoundedTrace,
    ConcreteDecision, ConcreteEffectIntent, ConcreteEffectRequest, ConcreteEvent,
    ConcreteReferenceSpec, ConcreteReplayInput, Decision, DenialReason, EffectIntent,
    EffectRequest, Event, LocalState, MachineState, NodeId, PayloadRef, ReplayBindings,
    ReplayDecision, ReplayFailureReason, ReplayInput, ReplayLimits, SuspensionClass,
    SuspensionFactSpec, SuspensionRecord, TRACE_BINDING_BYTES, TraceBinding, replay,
    replay_concrete, step, step_concrete,
};
use choregraph_kamacite::{
    ArtifactDomain, ArtifactIdentity, validate_concrete_machine_artifact,
    validate_durable_lifecycle_artifact, validate_durable_machine_artifact,
    validate_lifecycle_artifact, validate_machine_artifact,
};

use crate::codec::{decode_preserves, encode_preserves};
use crate::schema::{
    ABI_VERSION, ActionTag, AdapterErrorCode, AdapterErrorValue, AdapterStage, CALL_ROOT_LABEL,
    CALL_SCHEMA_VERSION, ConcreteBindingValue, ConcreteEffectRequestValue, ConcreteEventValue,
    DenialCode, DenialValue, EffectRequestValue, EventValue, IntentValue, KernelCall, KernelResult,
    MachineArtifactKind, MachineArtifacts, OperationRequest, Outcome, RESULT_ROOT_LABEL,
    RESULT_SCHEMA_VERSION, ReplayBindingPairValue, ReplayBindingsValue, ReplayFailureReasonValue,
    ReplayFailureValue, StateEntryValue, StateTag, StateValue, SuspensionFactValue,
    SuspensionValue, TransitionValue,
};

pub const MAX_OUTER_REQUEST_BYTES: usize = 1_048_576;
pub const MAX_OUTER_RESULT_BYTES: usize = 2_097_152;
pub const MAX_TRACE_EVENTS: u64 = 65_536;
pub const INPUT_IDENTITY_CONTEXT: &str = "choregraph wasm canonical call input v1";
pub const CLAIM_BOUNDARY: &str = "deterministic machine decisions and effect-intent data only; not authorization, persistence, scheduling, dispatch, isolation, engine correctness, or effect success";

struct MachineContext {
    machine: AdmittedMachine,
    identity: ArtifactIdentity,
    kind: MachineArtifactKind,
}

#[must_use]
pub fn dispatch(request: &[u8]) -> Vec<u8> {
    let input_identity = input_identity(request);
    if request.len() > MAX_OUTER_REQUEST_BYTES {
        return encode_result(KernelResult {
            abi_version: ABI_VERSION,
            operation: None,
            input_identity,
            outcome: adapter_error(
                AdapterErrorCode::RequestTooLarge,
                AdapterStage::OuterRequest,
            ),
        });
    }

    let call = match decode_preserves::<KernelCall>(request, CALL_SCHEMA_VERSION, CALL_ROOT_LABEL) {
        Ok((call, _canonical)) => call,
        Err(_error) => {
            return encode_result(KernelResult {
                abi_version: ABI_VERSION,
                operation: None,
                input_identity,
                outcome: adapter_error(AdapterErrorCode::MalformedPreserves, AdapterStage::Decode),
            });
        }
    };
    let operation = Some(call.operation.tag());
    if call.abi_version != ABI_VERSION {
        return encode_result(KernelResult {
            abi_version: ABI_VERSION,
            operation,
            input_identity,
            outcome: adapter_error(
                AdapterErrorCode::UnsupportedAbiVersion,
                AdapterStage::Version,
            ),
        });
    }

    let outcome = match execute(call.operation) {
        Ok(outcome) => outcome,
        Err(error) => Outcome::AdapterError { error },
    };
    encode_result(KernelResult {
        abi_version: ABI_VERSION,
        operation,
        input_identity,
        outcome,
    })
}

pub fn encode_call(call: &KernelCall) -> Result<Vec<u8>, String> {
    encode_preserves(call, CALL_SCHEMA_VERSION, CALL_ROOT_LABEL)
        .map(|artifact| artifact.packed_bytes)
        .map_err(|error| error.to_string())
}

pub fn decode_result(input: &[u8]) -> Result<KernelResult, String> {
    decode_preserves(input, RESULT_SCHEMA_VERSION, RESULT_ROOT_LABEL)
        .map(|(result, _canonical)| result)
        .map_err(|error| error.to_string())
}

fn execute(operation: OperationRequest) -> Result<Outcome, AdapterErrorValue> {
    match operation {
        OperationRequest::AdmitMachine { artifacts } => {
            let context = admit_artifacts(&artifacts)?;
            Ok(Outcome::Admitted {
                machine_identity: context.identity,
                initial_state: state_value(&context.machine.initial_state()),
            })
        }
        OperationRequest::Step {
            artifacts,
            state,
            event,
        } => {
            let context = admit_artifacts(&artifacts)?;
            require_linear(context.kind)?;
            let state = state_from_value(&state, &context.machine)?;
            let event = event_from_value(event);
            Ok(decision_outcome(
                context.identity,
                step(&context.machine, &state, &event),
            ))
        }
        OperationRequest::StepConcrete {
            artifacts,
            state,
            event,
        } => {
            let context = admit_artifacts(&artifacts)?;
            require_concrete(context.kind)?;
            let state = state_from_value(&state, &context.machine)?;
            let event = concrete_event_from_value(event);
            Ok(concrete_decision_outcome(
                context.identity,
                step_concrete(&context.machine, &state, &event),
            ))
        }
        OperationRequest::Replay {
            artifacts,
            initial_state,
            bindings,
            max_events,
            events,
        } => {
            let context = admit_artifacts(&artifacts)?;
            require_linear(context.kind)?;
            validate_trace_limit(events.len(), max_events)?;
            let initial_state = state_from_value(&initial_state, &context.machine)?;
            let (expected_bindings, trace_bindings) = bindings_from_value(bindings)?;
            let events: Vec<Event> = events.into_iter().map(event_from_value).collect();
            let trace = BoundedTrace::new(trace_bindings, &events);
            let input = ReplayInput::new(
                &context.machine,
                &initial_state,
                expected_bindings,
                trace,
                ReplayLimits { max_events },
            );
            Ok(replay_outcome(context.identity, replay(input)))
        }
        OperationRequest::ReplayConcrete {
            artifacts,
            initial_state,
            bindings,
            max_events,
            events,
        } => {
            let context = admit_artifacts(&artifacts)?;
            require_concrete(context.kind)?;
            validate_trace_limit(events.len(), max_events)?;
            let initial_state = state_from_value(&initial_state, &context.machine)?;
            let (expected_bindings, trace_bindings) = bindings_from_value(bindings)?;
            let events: Vec<ConcreteEvent> =
                events.into_iter().map(concrete_event_from_value).collect();
            let trace = ConcreteBoundedTrace::new(trace_bindings, &events);
            let input = ConcreteReplayInput::new(
                &context.machine,
                &initial_state,
                expected_bindings,
                trace,
                ReplayLimits { max_events },
            );
            Ok(replay_outcome(context.identity, replay_concrete(input)))
        }
    }
}

fn admit_artifacts(artifacts: &MachineArtifacts) -> Result<MachineContext, AdapterErrorValue> {
    match artifacts.kind {
        MachineArtifactKind::LinearV1 => {
            let (lifecycle_bytes, lifecycle_identity, profile_identity) =
                required_aggregate_fields(artifacts)?;
            let lifecycle = validate_lifecycle_artifact(lifecycle_bytes)
                .map_err(|_error| invalid_artifact())?;
            if lifecycle.project_identity != *lifecycle_identity {
                return Err(identity_mismatch());
            }
            let validated =
                validate_machine_artifact(&artifacts.machine, &lifecycle, profile_identity)
                    .map_err(|_error| invalid_artifact())?;
            if validated.identity != artifacts.machine_identity {
                return Err(identity_mismatch());
            }
            Ok(MachineContext {
                machine: validated.machine,
                identity: validated.identity,
                kind: artifacts.kind,
            })
        }
        MachineArtifactKind::DurableV1 => {
            let (lifecycle_bytes, lifecycle_identity, profile_identity) =
                required_aggregate_fields(artifacts)?;
            let lifecycle = validate_durable_lifecycle_artifact(lifecycle_bytes)
                .map_err(|_error| invalid_artifact())?;
            if lifecycle.project_identity != *lifecycle_identity {
                return Err(identity_mismatch());
            }
            let validated =
                validate_durable_machine_artifact(&artifacts.machine, &lifecycle, profile_identity)
                    .map_err(|_error| invalid_artifact())?;
            if validated.identity != artifacts.machine_identity {
                return Err(identity_mismatch());
            }
            Ok(MachineContext {
                machine: validated.machine,
                identity: validated.identity,
                kind: artifacts.kind,
            })
        }
        MachineArtifactKind::ConcreteV2 => {
            if artifacts.lifecycle.is_some()
                || artifacts.lifecycle_identity.is_some()
                || artifacts.profile_identity.is_some()
            {
                return Err(adapter_error_value(
                    AdapterErrorCode::UnexpectedArtifactField,
                    AdapterStage::Artifact,
                ));
            }
            let validated =
                validate_concrete_machine_artifact(&artifacts.machine, &artifacts.machine_identity)
                    .map_err(|_error| invalid_artifact())?;
            Ok(MachineContext {
                machine: validated.machine,
                identity: validated.identity,
                kind: artifacts.kind,
            })
        }
    }
}

fn required_aggregate_fields(
    artifacts: &MachineArtifacts,
) -> Result<(&[u8], &ArtifactIdentity, &ArtifactIdentity), AdapterErrorValue> {
    let lifecycle = artifacts.lifecycle.as_deref().ok_or_else(|| {
        adapter_error_value(
            AdapterErrorCode::MissingArtifactField,
            AdapterStage::Artifact,
        )
    })?;
    let lifecycle_identity = artifacts.lifecycle_identity.as_ref().ok_or_else(|| {
        adapter_error_value(
            AdapterErrorCode::MissingArtifactField,
            AdapterStage::Artifact,
        )
    })?;
    let profile_identity = artifacts.profile_identity.as_ref().ok_or_else(|| {
        adapter_error_value(
            AdapterErrorCode::MissingArtifactField,
            AdapterStage::Artifact,
        )
    })?;
    lifecycle_identity
        .validate(ArtifactDomain::Lifecycle)
        .map_err(|_error| invalid_artifact())?;
    Ok((lifecycle, lifecycle_identity, profile_identity))
}

fn state_from_value(
    value: &StateValue,
    machine: &AdmittedMachine,
) -> Result<MachineState, AdapterErrorValue> {
    let admission_policy = machine.limits();
    if !strictly_sorted_by_node(&value.entries, |entry| &entry.node_id)
        || !strictly_sorted_by_node(&value.suspensions, |entry| &entry.node_id)
    {
        return Err(adapter_error_value(
            AdapterErrorCode::InvalidState,
            AdapterStage::State,
        ));
    }
    let mut entries = BTreeMap::new();
    for entry in &value.entries {
        let node_id = NodeId::parse(entry.node_id.clone(), admission_policy.max_identifier_bytes)
            .map_err(|_error| {
            adapter_error_value(AdapterErrorCode::InvalidState, AdapterStage::State)
        })?;
        if entries
            .insert(node_id, state_from_tag(entry.state))
            .is_some()
        {
            return Err(adapter_error_value(
                AdapterErrorCode::InvalidState,
                AdapterStage::State,
            ));
        }
    }
    let mut suspensions = BTreeMap::new();
    for entry in &value.suspensions {
        let node_id = NodeId::parse(entry.node_id.clone(), admission_policy.max_identifier_bytes)
            .map_err(|_error| {
            adapter_error_value(AdapterErrorCode::InvalidState, AdapterStage::State)
        })?;
        let class =
            SuspensionClass::parse(entry.class.clone(), admission_policy.max_identifier_bytes)
                .map_err(|_error| {
                    adapter_error_value(AdapterErrorCode::InvalidState, AdapterStage::State)
                })?;
        let reference = PayloadRef::parse(
            entry.reference.clone(),
            admission_policy.max_reference_bytes,
        )
        .map_err(|_error| {
            adapter_error_value(AdapterErrorCode::InvalidState, AdapterStage::State)
        })?;
        if suspensions
            .insert(node_id, SuspensionRecord { class, reference })
            .is_some()
        {
            return Err(adapter_error_value(
                AdapterErrorCode::InvalidState,
                AdapterStage::State,
            ));
        }
    }
    Ok(MachineState::with_suspensions(entries, suspensions))
}

fn strictly_sorted_by_node<T>(values: &[T], node_id: impl Fn(&T) -> &String) -> bool {
    values
        .windows(2)
        .all(|window| node_id(&window[0]) < node_id(&window[1]))
}

fn event_from_value(value: EventValue) -> Event {
    Event {
        node_id: value.node_id,
        action: action_from_tag(value.action),
        effects: value.effects.into_iter().map(effect_from_value).collect(),
        suspension: value.suspension.map(suspension_from_value),
    }
}

fn effect_from_value(value: EffectRequestValue) -> EffectRequest {
    EffectRequest {
        operation: value.operation,
        resource: value.resource,
        ability: value.ability,
        request: value.request,
        payload: value.payload,
    }
}

fn concrete_event_from_value(value: ConcreteEventValue) -> ConcreteEvent {
    ConcreteEvent {
        node_id: value.node_id,
        action: action_from_tag(value.action),
        effects: value
            .effects
            .into_iter()
            .map(concrete_effect_from_value)
            .collect(),
        suspension: value.suspension.map(suspension_from_value),
    }
}

fn concrete_effect_from_value(value: ConcreteEffectRequestValue) -> ConcreteEffectRequest {
    ConcreteEffectRequest {
        declaration_key: value.declaration_key,
        request: concrete_binding_from_value(value.request),
        payload: concrete_binding_from_value(value.payload),
    }
}

fn concrete_binding_from_value(value: ConcreteBindingValue) -> ConcreteBindingInput {
    ConcreteBindingInput {
        slot_id: value.slot_id,
        source_ref: value.source_ref,
        reference: ConcreteReferenceSpec {
            profile: value.reference.profile,
            identity: value.reference.identity,
        },
    }
}

fn suspension_from_value(value: SuspensionFactValue) -> SuspensionFactSpec {
    SuspensionFactSpec {
        class: value.class,
        reference: value.reference,
    }
}

fn bindings_from_value(
    value: ReplayBindingPairValue,
) -> Result<(ReplayBindings, ReplayBindings), AdapterErrorValue> {
    Ok((
        one_binding_from_value(value.expected)?,
        one_binding_from_value(value.trace)?,
    ))
}

fn one_binding_from_value(value: ReplayBindingsValue) -> Result<ReplayBindings, AdapterErrorValue> {
    let machine: [u8; TRACE_BINDING_BYTES] = value.machine.try_into().map_err(|_bytes| {
        adapter_error_value(AdapterErrorCode::InvalidTraceBinding, AdapterStage::Trace)
    })?;
    let initial_state: [u8; TRACE_BINDING_BYTES] =
        value.initial_state.try_into().map_err(|_bytes| {
            adapter_error_value(AdapterErrorCode::InvalidTraceBinding, AdapterStage::Trace)
        })?;
    Ok(ReplayBindings::new(
        TraceBinding::from_bytes(machine),
        TraceBinding::from_bytes(initial_state),
    ))
}

fn validate_trace_limit(
    observed_events: usize,
    requested_maximum: u64,
) -> Result<(), AdapterErrorValue> {
    if requested_maximum > MAX_TRACE_EVENTS || (observed_events as u64) > requested_maximum {
        return Err(adapter_error_value(
            AdapterErrorCode::TraceLimitExceeded,
            AdapterStage::Trace,
        ));
    }
    Ok(())
}

fn decision_outcome(identity: ArtifactIdentity, decision: Decision) -> Outcome {
    match decision {
        Decision::Accept { next, intents } => Outcome::Accepted {
            machine_identity: identity,
            next: state_value(&next),
            intents: intents.into_iter().map(intent_value).collect(),
        },
        Decision::Deny { reason } => Outcome::Denied {
            machine_identity: identity,
            reason: denial_value(reason),
        },
    }
}

fn concrete_decision_outcome(identity: ArtifactIdentity, decision: ConcreteDecision) -> Outcome {
    match decision {
        ConcreteDecision::Accept { next, intents } => Outcome::Accepted {
            machine_identity: identity,
            next: state_value(&next),
            intents: intents.into_iter().map(concrete_intent_value).collect(),
        },
        ConcreteDecision::Deny { reason } => Outcome::Denied {
            machine_identity: identity,
            reason: denial_value(reason),
        },
    }
}

fn replay_outcome(identity: ArtifactIdentity, decision: ReplayDecision) -> Outcome {
    match decision {
        ReplayDecision::Complete {
            final_state,
            consumed,
        } => Outcome::ReplayComplete {
            machine_identity: identity,
            final_state: state_value(&final_state),
            consumed: consumed.get(),
        },
        ReplayDecision::Failed(failure) => Outcome::ReplayFailed {
            machine_identity: identity,
            failure: ReplayFailureValue {
                event_index: failure.event_index.map(|index| index.get()),
                consumed: failure.consumed.get(),
                reason: replay_failure_reason_value(failure.reason),
                last_valid_state: state_value(failure.last_valid_state.state()),
            },
        },
    }
}

fn replay_failure_reason_value(reason: ReplayFailureReason) -> ReplayFailureReasonValue {
    match reason {
        ReplayFailureReason::TraceLimitExceeded { observed, maximum } => {
            ReplayFailureReasonValue::TraceLimitExceeded { observed, maximum }
        }
        ReplayFailureReason::MachineBindingMismatch => {
            ReplayFailureReasonValue::MachineBindingMismatch
        }
        ReplayFailureReason::InitialStateBindingMismatch => {
            ReplayFailureReasonValue::InitialStateBindingMismatch
        }
        ReplayFailureReason::InitialStateInvalid => ReplayFailureReasonValue::InitialStateInvalid,
        ReplayFailureReason::TransitionDenied(reason) => {
            ReplayFailureReasonValue::TransitionDenied {
                reason: denial_value(reason),
            }
        }
    }
}

fn state_value(state: &MachineState) -> StateValue {
    let entries = state
        .entries()
        .iter()
        .map(|(node_id, state)| StateEntryValue {
            node_id: node_id.as_str().to_owned(),
            state: state_tag(*state),
        })
        .collect();
    let suspensions = state
        .suspensions()
        .iter()
        .map(|(node_id, record)| SuspensionValue {
            node_id: node_id.as_str().to_owned(),
            class: record.class.as_str().to_owned(),
            reference: record.reference.as_str().to_owned(),
        })
        .collect();
    StateValue {
        entries,
        suspensions,
    }
}

fn intent_value(intent: EffectIntent) -> IntentValue {
    IntentValue::Declared {
        node_id: intent.node_id.as_str().to_owned(),
        transition: transition_value(intent.transition.from, intent.transition.action),
        operation: intent.operation.as_str().to_owned(),
        resource: intent.resource.as_str().to_owned(),
        ability: intent.ability.as_str().to_owned(),
        request: intent.request.as_str().to_owned(),
        payload: intent.payload.as_str().to_owned(),
    }
}

fn concrete_intent_value(intent: ConcreteEffectIntent) -> IntentValue {
    IntentValue::Concrete {
        declaration_key: intent.declaration_key.as_str().to_owned(),
        node_id: intent.node_id.as_str().to_owned(),
        transition: transition_value(intent.transition.from, intent.transition.action),
        operation: intent.operation.as_str().to_owned(),
        resource: intent.resource.as_str().to_owned(),
        ability: intent.ability.as_str().to_owned(),
        request_slot: intent
            .request_slot
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        request_source_ref: intent
            .request_source_ref
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        request_profile: intent.request_reference.profile.as_str().to_owned(),
        request_reference: intent.request_reference.identity.as_str().to_owned(),
        payload_slot: intent
            .payload_slot
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        payload_source_ref: intent
            .payload_source_ref
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        payload_profile: intent.payload_reference.profile.as_str().to_owned(),
        payload_reference: intent.payload_reference.identity.as_str().to_owned(),
    }
}

fn denial_value(reason: DenialReason) -> DenialValue {
    let mut value = DenialValue {
        code: DenialCode::InvalidState,
        observed: None,
        maximum: None,
        from: None,
        action: None,
    };
    match reason {
        DenialReason::InvalidState => {}
        DenialReason::EventTooLarge { observed, maximum } => {
            value.code = DenialCode::EventTooLarge;
            value.observed = Some(usize_to_u64(observed));
            value.maximum = Some(usize_to_u64(maximum));
        }
        DenialReason::TooManyEffectRequests { observed, maximum } => {
            value.code = DenialCode::TooManyEffectRequests;
            value.observed = Some(usize_to_u64(observed));
            value.maximum = Some(usize_to_u64(maximum));
        }
        DenialReason::InvalidNodeId => value.code = DenialCode::InvalidNodeId,
        DenialReason::UnknownNode => value.code = DenialCode::UnknownNode,
        DenialReason::TerminalState { from, action } => {
            value.code = DenialCode::TerminalState;
            value.from = Some(state_tag(from));
            value.action = Some(action_tag(action));
        }
        DenialReason::InvalidTransition { from, action } => {
            value.code = DenialCode::InvalidTransition;
            value.from = Some(state_tag(from));
            value.action = Some(action_tag(action));
        }
        DenialReason::DependenciesIncomplete => value.code = DenialCode::DependenciesIncomplete,
        DenialReason::MalformedEffectRequest => value.code = DenialCode::MalformedEffectRequest,
        DenialReason::DuplicateEffectRequest => value.code = DenialCode::DuplicateEffectRequest,
        DenialReason::UndeclaredEffect => value.code = DenialCode::UndeclaredEffect,
        DenialReason::UnknownDeclaration => value.code = DenialCode::UnknownDeclaration,
        DenialReason::ConcreteBindingMismatch => value.code = DenialCode::ConcreteBindingMismatch,
        DenialReason::ConcreteIntentTooLarge { observed, maximum } => {
            value.code = DenialCode::ConcreteIntentTooLarge;
            value.observed = Some(usize_to_u64(observed));
            value.maximum = Some(usize_to_u64(maximum));
        }
        DenialReason::UnsupportedMachineSchema => {
            value.code = DenialCode::UnsupportedMachineSchema;
        }
        DenialReason::UnsupportedProfileAction { action } => {
            value.code = DenialCode::UnsupportedProfileAction;
            value.action = Some(action_tag(action));
        }
        DenialReason::SuspensionFactMissing => value.code = DenialCode::SuspensionFactMissing,
        DenialReason::SuspensionFactUnexpected => value.code = DenialCode::SuspensionFactUnexpected,
        DenialReason::MalformedSuspensionFact => value.code = DenialCode::MalformedSuspensionFact,
        DenialReason::SuspensionMismatch => value.code = DenialCode::SuspensionMismatch,
        DenialReason::SuspensionEffectsForbidden => {
            value.code = DenialCode::SuspensionEffectsForbidden;
        }
        DenialReason::InvariantWouldBreak => value.code = DenialCode::InvariantWouldBreak,
        DenialReason::EffectsNotAllowed => value.code = DenialCode::EffectsNotAllowed,
    }
    value
}

const fn state_from_tag(state: StateTag) -> LocalState {
    match state {
        StateTag::Pending => LocalState::Pending,
        StateTag::Ready => LocalState::Ready,
        StateTag::Running => LocalState::Running,
        StateTag::Completed => LocalState::Completed,
        StateTag::Failed => LocalState::Failed,
        StateTag::Cancelled => LocalState::Cancelled,
        StateTag::Suspended => LocalState::Suspended,
    }
}

const fn state_tag(state: LocalState) -> StateTag {
    match state {
        LocalState::Pending => StateTag::Pending,
        LocalState::Ready => StateTag::Ready,
        LocalState::Running => StateTag::Running,
        LocalState::Completed => StateTag::Completed,
        LocalState::Failed => StateTag::Failed,
        LocalState::Cancelled => StateTag::Cancelled,
        LocalState::Suspended => StateTag::Suspended,
    }
}

const fn action_from_tag(action: ActionTag) -> Action {
    match action {
        ActionTag::MarkReady => Action::MarkReady,
        ActionTag::Start => Action::Start,
        ActionTag::Complete => Action::Complete,
        ActionTag::Fail => Action::Fail,
        ActionTag::Cancel => Action::Cancel,
        ActionTag::Suspend => Action::Suspend,
        ActionTag::ResumeSignal => Action::ResumeSignal,
    }
}

const fn action_tag(action: Action) -> ActionTag {
    match action {
        Action::MarkReady => ActionTag::MarkReady,
        Action::Start => ActionTag::Start,
        Action::Complete => ActionTag::Complete,
        Action::Fail => ActionTag::Fail,
        Action::Cancel => ActionTag::Cancel,
        Action::Suspend => ActionTag::Suspend,
        Action::ResumeSignal => ActionTag::ResumeSignal,
    }
}

const fn transition_value(from: LocalState, action: Action) -> TransitionValue {
    TransitionValue {
        from: state_tag(from),
        action: action_tag(action),
    }
}

fn require_linear(kind: MachineArtifactKind) -> Result<(), AdapterErrorValue> {
    if matches!(
        kind,
        MachineArtifactKind::LinearV1 | MachineArtifactKind::DurableV1
    ) {
        Ok(())
    } else {
        Err(adapter_error_value(
            AdapterErrorCode::OperationProfileMismatch,
            AdapterStage::Dispatch,
        ))
    }
}

fn require_concrete(kind: MachineArtifactKind) -> Result<(), AdapterErrorValue> {
    if kind == MachineArtifactKind::ConcreteV2 {
        Ok(())
    } else {
        Err(adapter_error_value(
            AdapterErrorCode::OperationProfileMismatch,
            AdapterStage::Dispatch,
        ))
    }
}

fn invalid_artifact() -> AdapterErrorValue {
    adapter_error_value(AdapterErrorCode::InvalidArtifact, AdapterStage::Artifact)
}

fn identity_mismatch() -> AdapterErrorValue {
    adapter_error_value(AdapterErrorCode::IdentityMismatch, AdapterStage::Artifact)
}

const fn adapter_error_value(code: AdapterErrorCode, stage: AdapterStage) -> AdapterErrorValue {
    AdapterErrorValue { code, stage }
}

const fn adapter_error(code: AdapterErrorCode, stage: AdapterStage) -> Outcome {
    Outcome::AdapterError {
        error: adapter_error_value(code, stage),
    }
}

fn input_identity(input: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(INPUT_IDENTITY_CONTEXT);
    hasher.update(input);
    hasher.finalize().to_hex().to_string()
}

fn encode_result(result: KernelResult) -> Vec<u8> {
    let encoded = match encode_preserves(&result, RESULT_SCHEMA_VERSION, RESULT_ROOT_LABEL) {
        Ok(artifact) => artifact.packed_bytes,
        Err(_error) => return emergency_encoding_failure(&result.input_identity),
    };
    if encoded.len() <= MAX_OUTER_RESULT_BYTES {
        return encoded;
    }
    let bounded = KernelResult {
        abi_version: ABI_VERSION,
        operation: result.operation,
        input_identity: result.input_identity,
        outcome: adapter_error(AdapterErrorCode::ResultTooLarge, AdapterStage::OuterResult),
    };
    match encode_preserves(&bounded, RESULT_SCHEMA_VERSION, RESULT_ROOT_LABEL) {
        Ok(artifact) => artifact.packed_bytes,
        Err(_error) => Vec::new(),
    }
}

fn emergency_encoding_failure(input_identity: &str) -> Vec<u8> {
    let emergency = KernelResult {
        abi_version: ABI_VERSION,
        operation: None,
        input_identity: input_identity.to_owned(),
        outcome: adapter_error(
            AdapterErrorCode::InternalEncodingFailure,
            AdapterStage::Encode,
        ),
    };
    match encode_preserves(&emergency, RESULT_SCHEMA_VERSION, RESULT_ROOT_LABEL) {
        Ok(artifact) => artifact.packed_bytes,
        Err(_error) => Vec::new(),
    }
}

fn usize_to_u64(value: usize) -> u64 {
    value as u64
}
