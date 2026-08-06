// The production shell keeps one proof-linked transition path and bounded deterministic output.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    sentinel_fallback,
    unbounded_collection_growth,
    reason = "the production shell keeps one proof-linked transition path and bounded deterministic output"
)]

// r[impl choregraph.machine.step]
// r[impl choregraph.machine.denial]
// r[impl choregraph.machine.effect_intents]
// r[impl choregraph.proof.single_step.production_linkage]
// r[impl choregraph.proof.trace.model]
// r[impl choregraph.proof.trace.replay]
// r[impl choregraph.proof.trace.failure]
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::concrete::{
    ConcreteDecision, ConcreteEvent, ConcreteReplayInput, concrete_effects_allowed,
    concrete_event_size, resolve_concrete_intents,
};
use crate::id::{AbilityId, NodeId, OperationId, PayloadRef, RequestId, ResourceId};
use crate::invariant::{effects_allowed, invariant, terminal_states_preserved};
use crate::model::{
    Action, AdmittedMachine, ConsumedCount, Decision, DenialReason, EffectDeclaration,
    EffectIntent, EffectKey, EffectRequest, Event, EventIndex, LastValidState, LifecycleProfile,
    LocalState, LocalTransitionError, MachineSchema, MachineState, ReplayDecision, ReplayFailure,
    ReplayFailureReason, ReplayInput, SuspensionRecord, TransitionSelector,
};
use crate::verified::{
    ACTION_CANCEL, ACTION_COMPLETE, ACTION_FAIL, ACTION_MARK_READY, ACTION_RESUME_SIGNAL,
    ACTION_START, ACTION_SUSPEND, DENIAL_DEPENDENCIES_INCOMPLETE, DENIAL_DUPLICATE_EFFECT,
    DENIAL_INVALID_STATE, DENIAL_INVALID_TRANSITION, DENIAL_INVARIANT_WOULD_BREAK,
    DENIAL_PROFILE_ACTION, DENIAL_SUSPENSION_EFFECTS, DENIAL_SUSPENSION_FACT_ABSENT,
    DENIAL_SUSPENSION_FACT_PRESENT, DENIAL_SUSPENSION_MISMATCH, DENIAL_TERMINAL_STATE,
    DENIAL_UNDECLARED_EFFECT, DENIAL_UNKNOWN_NODE, IndexedDecision, IndexedSuspensionFact,
    PROFILE_DURABLE_SUSPENSION_V1, PROFILE_LINEAR_V1, STATE_CANCELLED, STATE_COMPLETED,
    STATE_FAILED, STATE_PENDING, STATE_READY, STATE_RUNNING, STATE_SUSPENDED,
};

const EVENT_ACTION_TAG_BYTES: usize = 1;

const fn local_state_code(state: LocalState) -> u8 {
    match state {
        LocalState::Pending => STATE_PENDING,
        LocalState::Ready => STATE_READY,
        LocalState::Running => STATE_RUNNING,
        LocalState::Completed => STATE_COMPLETED,
        LocalState::Failed => STATE_FAILED,
        LocalState::Cancelled => STATE_CANCELLED,
        LocalState::Suspended => STATE_SUSPENDED,
    }
}

const fn local_state_from_code(state: u8) -> Option<LocalState> {
    match state {
        STATE_PENDING => Some(LocalState::Pending),
        STATE_READY => Some(LocalState::Ready),
        STATE_RUNNING => Some(LocalState::Running),
        STATE_COMPLETED => Some(LocalState::Completed),
        STATE_FAILED => Some(LocalState::Failed),
        STATE_CANCELLED => Some(LocalState::Cancelled),
        STATE_SUSPENDED => Some(LocalState::Suspended),
        _ => None,
    }
}

const fn action_code(action: Action) -> u8 {
    match action {
        Action::MarkReady => ACTION_MARK_READY,
        Action::Start => ACTION_START,
        Action::Complete => ACTION_COMPLETE,
        Action::Fail => ACTION_FAIL,
        Action::Cancel => ACTION_CANCEL,
        Action::Suspend => ACTION_SUSPEND,
        Action::ResumeSignal => ACTION_RESUME_SIGNAL,
    }
}

const fn profile_code(profile: LifecycleProfile) -> u8 {
    match profile {
        LifecycleProfile::LinearV1 => PROFILE_LINEAR_V1,
        LifecycleProfile::DurableSuspensionV1 => PROFILE_DURABLE_SUSPENSION_V1,
    }
}

/// Returns the target of one local lifecycle transition under the linear v1 profile.
///
/// This compatibility entrypoint keeps the exact pre-suspension table. New code
/// that must honor the admitted machine profile uses
/// [`lifecycle_target_for_profile`].
pub const fn lifecycle_target(
    from: LocalState,
    action: Action,
) -> Result<LocalState, LocalTransitionError> {
    lifecycle_target_for_profile(LifecycleProfile::LinearV1, from, action)
}

/// Returns the target of one local lifecycle transition under an explicit profile.
pub const fn lifecycle_target_for_profile(
    profile: LifecycleProfile,
    from: LocalState,
    action: Action,
) -> Result<LocalState, LocalTransitionError> {
    let target = crate::verified::lifecycle_target(
        profile_code(profile),
        local_state_code(from),
        action_code(action),
    );
    match target {
        Some(target) => match local_state_from_code(target) {
            Some(target) => Ok(target),
            None => Err(LocalTransitionError::Invalid { from, action }),
        },
        None if matches!(
            from,
            LocalState::Completed | LocalState::Failed | LocalState::Cancelled
        ) =>
        {
            Err(LocalTransitionError::Terminal { from, action })
        }
        None => Err(LocalTransitionError::Invalid { from, action }),
    }
}

fn checked_add(total: usize, value: usize) -> Option<usize> {
    total.checked_add(value)
}

fn request_size(request: &EffectRequest) -> Option<usize> {
    let mut size = 0usize;
    for field in [
        request.operation.len(),
        request.resource.len(),
        request.ability.len(),
        request.request.len(),
        request.payload.len(),
    ] {
        size = checked_add(size, field)?;
    }
    Some(size)
}

fn event_size(event: &Event) -> Option<usize> {
    let mut size = checked_add(event.node_id.len(), EVENT_ACTION_TAG_BYTES)?;
    if let Some(fact) = &event.suspension {
        size = checked_add(size, fact.class.len())?;
        size = checked_add(size, fact.reference.len())?;
    }
    for request in &event.effects {
        size = checked_add(size, request_size(request)?)?;
    }
    Some(size)
}

fn declaration_matches_request(declaration: &EffectDeclaration, request: &EffectRequest) -> bool {
    declaration.operation.as_str() == request.operation
        && declaration.resource.as_str() == request.resource
        && declaration.ability.as_str() == request.ability
        && declaration.request.as_str() == request.request
        && declaration.payload.as_str() == request.payload
}

fn validate_request_shape(
    machine: &AdmittedMachine,
    request: &EffectRequest,
) -> Result<(), DenialReason> {
    OperationId::parse(&request.operation, machine.limits.max_identifier_bytes)
        .map_err(|_| DenialReason::MalformedEffectRequest)?;
    ResourceId::parse(&request.resource, machine.limits.max_identifier_bytes)
        .map_err(|_| DenialReason::MalformedEffectRequest)?;
    AbilityId::parse(&request.ability, machine.limits.max_identifier_bytes)
        .map_err(|_| DenialReason::MalformedEffectRequest)?;
    RequestId::parse(&request.request, machine.limits.max_identifier_bytes)
        .map_err(|_| DenialReason::MalformedEffectRequest)?;
    PayloadRef::parse(&request.payload, machine.limits.max_reference_bytes)
        .map_err(|_| DenialReason::MalformedEffectRequest)?;
    Ok(())
}

fn resolve_intent_indices(
    machine: &AdmittedMachine,
    allowed_indices: &[usize],
    event: &Event,
) -> Result<Vec<usize>, DenialReason> {
    let mut requested_indices = Vec::with_capacity(event.effects.len());
    for request in &event.effects {
        validate_request_shape(machine, request)?;
        let mut matched = None;
        for effect_index in allowed_indices {
            let Some(declaration) = machine.effects.get(*effect_index) else {
                return Err(DenialReason::InvalidState);
            };
            if declaration_matches_request(declaration, request) {
                matched = Some(*effect_index);
                break;
            }
        }
        let Some(effect_index) = matched else {
            return Err(DenialReason::UndeclaredEffect);
        };
        requested_indices.push(effect_index);
    }
    Ok(requested_indices)
}

fn indexed_states(machine: &AdmittedMachine, state: &MachineState) -> Option<Vec<u8>> {
    if state.entries().len() != machine.node_order.len() {
        return None;
    }
    let mut states = Vec::with_capacity(machine.node_order.len());
    for node_id in &machine.node_order {
        states.push(local_state_code(state.state(node_id)?));
    }
    Some(states)
}

fn state_from_indexed(machine: &AdmittedMachine, states: &[u8]) -> Option<MachineState> {
    if states.len() != machine.node_order.len() {
        return None;
    }
    let mut entries = BTreeMap::new();
    for (node_index, node_id) in machine.node_order.iter().enumerate() {
        let local_state = local_state_from_code(states[node_index])?;
        entries.insert(node_id.clone(), local_state);
    }
    Some(MachineState::new(entries))
}

fn intents_from_indices(
    machine: &AdmittedMachine,
    intent_indices: &[usize],
) -> Option<Vec<EffectIntent>> {
    let mut intents = BTreeSet::new();
    for effect_index in intent_indices {
        intents.insert(EffectIntent::from(machine.effects.get(*effect_index)?));
    }
    if intents.len() != intent_indices.len() {
        return None;
    }
    Some(intents.into_iter().collect())
}

fn denial_from_indexed(reason: u8, from: LocalState, action: Action) -> DenialReason {
    match reason {
        DENIAL_INVALID_STATE => DenialReason::InvalidState,
        DENIAL_UNKNOWN_NODE => DenialReason::UnknownNode,
        DENIAL_TERMINAL_STATE => DenialReason::TerminalState { from, action },
        DENIAL_INVALID_TRANSITION => DenialReason::InvalidTransition { from, action },
        DENIAL_DEPENDENCIES_INCOMPLETE => DenialReason::DependenciesIncomplete,
        DENIAL_DUPLICATE_EFFECT => DenialReason::DuplicateEffectRequest,
        DENIAL_UNDECLARED_EFFECT => DenialReason::UndeclaredEffect,
        DENIAL_INVARIANT_WOULD_BREAK => DenialReason::InvariantWouldBreak,
        DENIAL_SUSPENSION_MISMATCH => DenialReason::SuspensionMismatch,
        DENIAL_SUSPENSION_FACT_ABSENT => DenialReason::SuspensionFactMissing,
        DENIAL_SUSPENSION_FACT_PRESENT => DenialReason::SuspensionFactUnexpected,
        DENIAL_SUSPENSION_EFFECTS => DenialReason::SuspensionEffectsForbidden,
        DENIAL_PROFILE_ACTION => DenialReason::UnsupportedProfileAction { action },
        _ => DenialReason::InvariantWouldBreak,
    }
}

fn public_decision_from_indexed(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
    prepared: &PreparedStep,
    indexed: IndexedDecision,
) -> Decision {
    if !indexed.accepted {
        return Decision::Deny {
            reason: denial_from_indexed(indexed.reason, prepared.from, event.action),
        };
    }
    let Some(mut next) = state_from_indexed(machine, &indexed.next_states) else {
        return Decision::Deny {
            reason: DenialReason::InvariantWouldBreak,
        };
    };
    apply_suspension_delta(
        state,
        &mut next,
        event.action,
        &prepared.node_id,
        prepared.record.clone(),
    );
    if !invariant(machine, &next) || !terminal_states_preserved(state, &next) {
        return Decision::Deny {
            reason: DenialReason::InvariantWouldBreak,
        };
    };
    let Some(intents) = intents_from_indices(machine, &indexed.intent_indices) else {
        return Decision::Deny {
            reason: DenialReason::EffectsNotAllowed,
        };
    };
    if !effects_allowed(machine, state, event, &intents) {
        return Decision::Deny {
            reason: DenialReason::EffectsNotAllowed,
        };
    }
    Decision::Accept { next, intents }
}

struct PreparedStep {
    node_id: NodeId,
    node_index: usize,
    from: LocalState,
    action: Action,
    event_fact: Option<IndexedSuspensionFact>,
    record: Option<SuspensionRecord>,
    states: Vec<u8>,
    suspensions: Vec<Option<IndexedSuspensionFact>>,
}

fn indexed_suspension_fact(fact: &crate::model::SuspensionFactSpec) -> IndexedSuspensionFact {
    IndexedSuspensionFact {
        class: Vec::from(fact.class.as_bytes()),
        reference: Vec::from(fact.reference.as_bytes()),
    }
}

fn indexed_suspensions(
    machine: &AdmittedMachine,
    state: &MachineState,
) -> Option<Vec<Option<IndexedSuspensionFact>>> {
    let mut suspensions = Vec::with_capacity(machine.node_order.len());
    for node_id in &machine.node_order {
        let record = state.suspension(node_id);
        match (state.state(node_id), record) {
            (Some(LocalState::Suspended), Some(record)) => {
                suspensions.push(Some(IndexedSuspensionFact {
                    class: Vec::from(record.class.as_str().as_bytes()),
                    reference: Vec::from(record.reference.as_str().as_bytes()),
                }));
            }
            (Some(state), None) if state != LocalState::Suspended => {
                suspensions.push(None);
            }
            _ => return None,
        }
    }
    Some(suspensions)
}

fn prepare_step(
    machine: &AdmittedMachine,
    state: &MachineState,
    node_id: &str,
    action: Action,
    suspension: Option<&crate::model::SuspensionFactSpec>,
) -> Result<PreparedStep, DenialReason> {
    if !invariant(machine, state) {
        return Err(DenialReason::InvalidState);
    }
    let node_id = NodeId::parse(node_id, machine.limits.max_identifier_bytes)
        .map_err(|_| DenialReason::InvalidNodeId)?;
    let node_index = machine
        .node_indices
        .get(&node_id)
        .copied()
        .ok_or(DenialReason::UnknownNode)?;
    let from = state.state(&node_id).ok_or(DenialReason::InvalidState)?;
    let states = indexed_states(machine, state).ok_or(DenialReason::InvalidState)?;
    let suspensions = indexed_suspensions(machine, state).ok_or(DenialReason::InvalidState)?;
    let event_fact = suspension.map(indexed_suspension_fact);
    let empty_intents = Vec::new();
    let precheck = crate::verified::indexed_step(
        &machine.dependency_indices,
        &states,
        &suspensions,
        profile_code(machine.lifecycle_profile),
        node_index,
        action_code(action),
        &event_fact,
        &empty_intents,
        &empty_intents,
    );
    if !precheck.accepted {
        return Err(denial_from_indexed(precheck.reason, from, action));
    }
    Ok(PreparedStep {
        node_id,
        node_index,
        from,
        action,
        event_fact,
        record: None,
        states,
        suspensions,
    })
}

fn run_indexed_step(
    machine: &AdmittedMachine,
    prepared: &PreparedStep,
    allowed_indices: &[usize],
    requested_indices: &[usize],
) -> IndexedDecision {
    crate::verified::indexed_step(
        &machine.dependency_indices,
        &prepared.states,
        &prepared.suspensions,
        profile_code(machine.lifecycle_profile),
        prepared.node_index,
        action_code(prepared.action),
        &prepared.event_fact,
        allowed_indices,
        requested_indices,
    )
}

/// Parses the typed suspension record for an accepted suspend event.
fn parse_suspension_fact(
    machine: &AdmittedMachine,
    fact: Option<&crate::model::SuspensionFactSpec>,
) -> Result<Option<SuspensionRecord>, DenialReason> {
    let Some(fact) = fact else {
        return Ok(None);
    };
    let class =
        crate::id::SuspensionClass::parse(fact.class.clone(), machine.limits.max_identifier_bytes)
            .map_err(|_| DenialReason::MalformedSuspensionFact)?;
    let reference = PayloadRef::parse(fact.reference.clone(), machine.limits.max_reference_bytes)
        .map_err(|_| DenialReason::MalformedSuspensionFact)?;
    Ok(Some(SuspensionRecord { class, reference }))
}

/// Applies the accepted suspension delta to the next aggregate state.
fn apply_suspension_delta(
    state: &MachineState,
    next: &mut MachineState,
    action: Action,
    node_id: &NodeId,
    record: Option<SuspensionRecord>,
) {
    next.suspensions = state.suspensions().clone();
    match action {
        Action::Suspend => {
            if let Some(record) = record {
                next.suspensions.insert(node_id.clone(), record);
            }
        }
        Action::ResumeSignal => {
            next.suspensions.remove(node_id);
        }
        _ => {}
    }
}

fn accepted_next_state(
    machine: &AdmittedMachine,
    state: &MachineState,
    prepared: &PreparedStep,
    indexed: &IndexedDecision,
) -> Result<MachineState, DenialReason> {
    if !indexed.accepted {
        return Err(denial_from_indexed(
            indexed.reason,
            prepared.from,
            prepared.action,
        ));
    }
    let mut next = state_from_indexed(machine, &indexed.next_states)
        .ok_or(DenialReason::InvariantWouldBreak)?;
    apply_suspension_delta(
        state,
        &mut next,
        prepared.action,
        &prepared.node_id,
        prepared.record.clone(),
    );
    if !invariant(machine, &next) || !terminal_states_preserved(state, &next) {
        return Err(DenialReason::InvariantWouldBreak);
    }
    Ok(next)
}

/// Applies one deterministic aggregate v1 machine transition.
#[must_use]
pub fn step(machine: &AdmittedMachine, state: &MachineState, event: &Event) -> Decision {
    if machine.schema != MachineSchema::V1 {
        return Decision::Deny {
            reason: DenialReason::UnsupportedMachineSchema,
        };
    }
    if event.effects.len() > machine.limits.max_intents_per_transition {
        return Decision::Deny {
            reason: DenialReason::TooManyEffectRequests {
                observed: event.effects.len(),
                maximum: machine.limits.max_intents_per_transition,
            },
        };
    }
    let observed_event_bytes = event_size(event).unwrap_or(usize::MAX);
    if observed_event_bytes > machine.limits.max_event_bytes {
        return Decision::Deny {
            reason: DenialReason::EventTooLarge {
                observed: observed_event_bytes,
                maximum: machine.limits.max_event_bytes,
            },
        };
    }
    let mut prepared = match prepare_step(
        machine,
        state,
        &event.node_id,
        event.action,
        event.suspension.as_ref(),
    ) {
        Ok(prepared) => prepared,
        Err(reason) => return Decision::Deny { reason },
    };
    prepared.record = match parse_suspension_fact(machine, event.suspension.as_ref()) {
        Ok(record) => record,
        Err(reason) => return Decision::Deny { reason },
    };
    if matches!(event.action, Action::Suspend | Action::ResumeSignal) && !event.effects.is_empty() {
        return Decision::Deny {
            reason: DenialReason::SuspensionEffectsForbidden,
        };
    }
    let key = EffectKey {
        node_id: prepared.node_id.clone(),
        transition: TransitionSelector {
            from: prepared.from,
            action: event.action,
        },
    };
    let allowed_indices = machine
        .effect_route_indices
        .get(&key)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let requested_indices = match resolve_intent_indices(machine, allowed_indices, event) {
        Ok(indices) => indices,
        Err(reason) => return Decision::Deny { reason },
    };
    let indexed = run_indexed_step(machine, &prepared, allowed_indices, &requested_indices);
    public_decision_from_indexed(machine, state, event, &prepared, indexed)
}

/// Applies one deterministic aggregate v2 machine transition through the same indexed kernel.
#[must_use]
pub fn step_concrete(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &ConcreteEvent,
) -> ConcreteDecision {
    if machine.schema != MachineSchema::V2 {
        return ConcreteDecision::Deny {
            reason: DenialReason::UnsupportedMachineSchema,
        };
    }
    if event.effects.len() > machine.limits.max_intents_per_transition {
        return ConcreteDecision::Deny {
            reason: DenialReason::TooManyEffectRequests {
                observed: event.effects.len(),
                maximum: machine.limits.max_intents_per_transition,
            },
        };
    }
    let observed_event_bytes = concrete_event_size(event).unwrap_or(usize::MAX);
    if observed_event_bytes > machine.limits.max_event_bytes {
        return ConcreteDecision::Deny {
            reason: DenialReason::EventTooLarge {
                observed: observed_event_bytes,
                maximum: machine.limits.max_event_bytes,
            },
        };
    }
    let mut prepared = match prepare_step(
        machine,
        state,
        &event.node_id,
        event.action,
        event.suspension.as_ref(),
    ) {
        Ok(prepared) => prepared,
        Err(reason) => return ConcreteDecision::Deny { reason },
    };
    prepared.record = match parse_suspension_fact(machine, event.suspension.as_ref()) {
        Ok(record) => record,
        Err(reason) => return ConcreteDecision::Deny { reason },
    };
    if matches!(event.action, Action::Suspend | Action::ResumeSignal) && !event.effects.is_empty() {
        return ConcreteDecision::Deny {
            reason: DenialReason::SuspensionEffectsForbidden,
        };
    }
    let key = EffectKey {
        node_id: prepared.node_id.clone(),
        transition: TransitionSelector {
            from: prepared.from,
            action: event.action,
        },
    };
    let allowed_indices = machine
        .concrete_effect_route_indices
        .get(&key)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let (requested_indices, intents) =
        match resolve_concrete_intents(machine, allowed_indices, event) {
            Ok(resolved) => resolved,
            Err(reason) => return ConcreteDecision::Deny { reason },
        };
    let indexed = run_indexed_step(machine, &prepared, allowed_indices, &requested_indices);
    let next = match accepted_next_state(machine, state, &prepared, &indexed) {
        Ok(next) => next,
        Err(reason) => return ConcreteDecision::Deny { reason },
    };
    if indexed.intent_indices != requested_indices
        || !concrete_effects_allowed(machine, state, event, &intents)
    {
        return ConcreteDecision::Deny {
            reason: DenialReason::EffectsNotAllowed,
        };
    }
    ConcreteDecision::Accept { next, intents }
}

fn replay_failure(
    event_index: Option<EventIndex>,
    consumed: ConsumedCount,
    reason: ReplayFailureReason,
    last_valid_state: MachineState,
) -> ReplayDecision {
    ReplayDecision::Failed(ReplayFailure {
        event_index,
        consumed,
        reason,
        last_valid_state: LastValidState::new(last_valid_state),
    })
}

/// Replays a bounded event trace through the production transition relation.
///
/// Accepted effect intents are checked by [`step`] and then discarded. Replay output contains no
/// intent field and grants no dispatch authority.
#[must_use]
pub fn replay(input: ReplayInput<'_>) -> ReplayDecision {
    let trace_events = input.trace.events();
    let observed_event_count = ConsumedCount::from_usize(trace_events.len()).get();
    let initial_state = input.initial_state.clone();
    if observed_event_count > input.limits.max_events {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::TraceLimitExceeded {
                observed: observed_event_count,
                maximum: input.limits.max_events,
            },
            initial_state,
        );
    }
    let trace_bindings = input.trace.bindings();
    if trace_bindings.machine != input.expected_bindings.machine {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::MachineBindingMismatch,
            initial_state,
        );
    }
    if trace_bindings.initial_state != input.expected_bindings.initial_state {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::InitialStateBindingMismatch,
            initial_state,
        );
    }
    if !invariant(input.machine, input.initial_state) {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::InitialStateInvalid,
            initial_state,
        );
    }

    let mut last_valid_state = initial_state;
    for (event_index, event) in trace_events.iter().enumerate() {
        match step(input.machine, &last_valid_state, event) {
            Decision::Accept { next, intents: _ } => {
                last_valid_state = next;
            }
            Decision::Deny { reason } => {
                return replay_failure(
                    Some(EventIndex::from_usize(event_index)),
                    ConsumedCount::from_usize(event_index),
                    ReplayFailureReason::TransitionDenied(reason),
                    last_valid_state,
                );
            }
        }
    }

    ReplayDecision::Complete {
        final_state: last_valid_state,
        consumed: ConsumedCount::from_usize(trace_events.len()),
    }
}

/// Replays a bounded v2 trace through [`step_concrete`] and discards every concrete intent.
#[must_use]
pub fn replay_concrete(input: ConcreteReplayInput<'_>) -> ReplayDecision {
    let trace_events = input.trace.events();
    let observed_event_count = ConsumedCount::from_usize(trace_events.len()).get();
    let initial_state = input.initial_state.clone();
    if observed_event_count > input.limits.max_events {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::TraceLimitExceeded {
                observed: observed_event_count,
                maximum: input.limits.max_events,
            },
            initial_state,
        );
    }
    let trace_bindings = input.trace.bindings();
    if trace_bindings.machine != input.expected_bindings.machine {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::MachineBindingMismatch,
            initial_state,
        );
    }
    if trace_bindings.initial_state != input.expected_bindings.initial_state {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::InitialStateBindingMismatch,
            initial_state,
        );
    }
    if !invariant(input.machine, input.initial_state) {
        return replay_failure(
            None,
            ConsumedCount::new(0),
            ReplayFailureReason::InitialStateInvalid,
            initial_state,
        );
    }

    let mut last_valid_state = initial_state;
    for (event_index, event) in trace_events.iter().enumerate() {
        match step_concrete(input.machine, &last_valid_state, event) {
            ConcreteDecision::Accept { next, intents: _ } => {
                last_valid_state = next;
            }
            ConcreteDecision::Deny { reason } => {
                return replay_failure(
                    Some(EventIndex::from_usize(event_index)),
                    ConsumedCount::from_usize(event_index),
                    ReplayFailureReason::TransitionDenied(reason),
                    last_valid_state,
                );
            }
        }
    }

    ReplayDecision::Complete {
        final_state: last_valid_state,
        consumed: ConsumedCount::from_usize(trace_events.len()),
    }
}
