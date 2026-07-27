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
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::id::{AbilityId, NodeId, OperationId, PayloadRef, RequestId, ResourceId};
use crate::invariant::{effects_allowed, invariant, terminal_states_preserved};
use crate::model::{
    Action, AdmittedMachine, Decision, DenialReason, EffectDeclaration, EffectIntent, EffectKey,
    EffectRequest, Event, LocalState, LocalTransitionError, MachineState, TransitionSelector,
};
use crate::verified::{
    ACTION_CANCEL, ACTION_COMPLETE, ACTION_FAIL, ACTION_MARK_READY, ACTION_START,
    DENIAL_DEPENDENCIES_INCOMPLETE, DENIAL_DUPLICATE_EFFECT, DENIAL_INVALID_STATE,
    DENIAL_INVALID_TRANSITION, DENIAL_INVARIANT_WOULD_BREAK, DENIAL_TERMINAL_STATE,
    DENIAL_UNDECLARED_EFFECT, DENIAL_UNKNOWN_NODE, IndexedDecision, STATE_CANCELLED,
    STATE_COMPLETED, STATE_FAILED, STATE_PENDING, STATE_READY, STATE_RUNNING,
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
    }
}

/// Returns the target of one local lifecycle transition.
pub const fn lifecycle_target(
    from: LocalState,
    action: Action,
) -> Result<LocalState, LocalTransitionError> {
    let target = crate::verified::lifecycle_target(local_state_code(from), action_code(action));
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
        _ => DenialReason::InvariantWouldBreak,
    }
}

fn public_decision_from_indexed(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
    from: LocalState,
    indexed: IndexedDecision,
) -> Decision {
    if !indexed.accepted {
        return Decision::Deny {
            reason: denial_from_indexed(indexed.reason, from, event.action),
        };
    }
    let Some(next) = state_from_indexed(machine, &indexed.next_states) else {
        return Decision::Deny {
            reason: DenialReason::InvariantWouldBreak,
        };
    };
    let Some(intents) = intents_from_indices(machine, &indexed.intent_indices) else {
        return Decision::Deny {
            reason: DenialReason::EffectsNotAllowed,
        };
    };
    if !invariant(machine, &next) || !terminal_states_preserved(state, &next) {
        return Decision::Deny {
            reason: DenialReason::InvariantWouldBreak,
        };
    }
    if !effects_allowed(machine, state, event, &intents) {
        return Decision::Deny {
            reason: DenialReason::EffectsNotAllowed,
        };
    }
    Decision::Accept { next, intents }
}

/// Applies one deterministic aggregate machine transition.
#[must_use]
pub fn step(machine: &AdmittedMachine, state: &MachineState, event: &Event) -> Decision {
    if !invariant(machine, state) {
        return Decision::Deny {
            reason: DenialReason::InvalidState,
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
    let Ok(node_id) = NodeId::parse(&event.node_id, machine.limits.max_identifier_bytes) else {
        return Decision::Deny {
            reason: DenialReason::InvalidNodeId,
        };
    };
    let Some(node_index) = machine.node_indices.get(&node_id).copied() else {
        return Decision::Deny {
            reason: DenialReason::UnknownNode,
        };
    };
    let Some(from) = state.state(&node_id) else {
        return Decision::Deny {
            reason: DenialReason::InvalidState,
        };
    };
    let Some(states) = indexed_states(machine, state) else {
        return Decision::Deny {
            reason: DenialReason::InvalidState,
        };
    };

    let empty_intents = Vec::new();
    let precheck = crate::verified::indexed_step(
        &machine.dependency_indices,
        &states,
        node_index,
        action_code(event.action),
        &empty_intents,
        &empty_intents,
    );
    if !precheck.accepted {
        return public_decision_from_indexed(machine, state, event, from, precheck);
    }

    let selector = TransitionSelector {
        from,
        action: event.action,
    };
    let key = EffectKey {
        node_id,
        transition: selector,
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
    let indexed = crate::verified::indexed_step(
        &machine.dependency_indices,
        &states,
        node_index,
        action_code(event.action),
        allowed_indices,
        &requested_indices,
    );
    public_decision_from_indexed(machine, state, event, from, indexed)
}
