// r[impl choregraph.machine.step]
// r[impl choregraph.machine.denial]
// r[impl choregraph.machine.effect_intents]
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::id::{AbilityId, NodeId, OperationId, PayloadRef, RequestId, ResourceId};
use crate::invariant::{effects_allowed, invariant, terminal_states_preserved};
use crate::model::{
    Action, AdmittedMachine, Decision, DenialReason, EffectDeclaration, EffectIntent, EffectKey,
    EffectRequest, Event, LocalState, LocalTransitionError, MachineState, TransitionSelector,
};

const EVENT_ACTION_TAG_BYTES: usize = 1;

/// Returns the target of one local lifecycle transition.
pub const fn lifecycle_target(
    from: LocalState,
    action: Action,
) -> Result<LocalState, LocalTransitionError> {
    if matches!(
        from,
        LocalState::Completed | LocalState::Failed | LocalState::Cancelled
    ) {
        return Err(LocalTransitionError::Terminal { from, action });
    }
    match (from, action) {
        (LocalState::Pending, Action::MarkReady) => Ok(LocalState::Ready),
        (LocalState::Ready, Action::Start) => Ok(LocalState::Running),
        (LocalState::Running, Action::Complete) => Ok(LocalState::Completed),
        (LocalState::Running, Action::Fail) => Ok(LocalState::Failed),
        (LocalState::Pending | LocalState::Ready | LocalState::Running, Action::Cancel) => {
            Ok(LocalState::Cancelled)
        }
        _ => Err(LocalTransitionError::Invalid { from, action }),
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

fn dependencies_complete(
    machine: &AdmittedMachine,
    state: &MachineState,
    node_id: &NodeId,
) -> bool {
    let Some(dependencies) = machine.dependencies.get(node_id) else {
        return false;
    };
    for dependency in dependencies {
        if state.state(dependency) != Some(LocalState::Completed) {
            return false;
        }
    }
    true
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

fn derive_intents(
    machine: &AdmittedMachine,
    node_id: &NodeId,
    selector: TransitionSelector,
    event: &Event,
) -> Result<Vec<EffectIntent>, DenialReason> {
    let key = EffectKey {
        node_id: node_id.clone(),
        transition: selector,
    };
    let declarations = machine
        .effect_index
        .get(&key)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut intents = BTreeSet::new();

    for request in &event.effects {
        validate_request_shape(machine, request)?;
        let Some(declaration) = declarations
            .iter()
            .find(|declaration| declaration_matches_request(declaration, request))
        else {
            return Err(DenialReason::UndeclaredEffect);
        };
        if !intents.insert(EffectIntent::from(declaration)) {
            return Err(DenialReason::DuplicateEffectRequest);
        }
    }
    Ok(intents.into_iter().collect())
}

fn denial_for_transition(error: LocalTransitionError) -> DenialReason {
    match error {
        LocalTransitionError::Terminal { from, action } => {
            DenialReason::TerminalState { from, action }
        }
        LocalTransitionError::Invalid { from, action } => {
            DenialReason::InvalidTransition { from, action }
        }
    }
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
    if !machine.nodes.contains_key(&node_id) {
        return Decision::Deny {
            reason: DenialReason::UnknownNode,
        };
    }
    let Some(from) = state.state(&node_id) else {
        return Decision::Deny {
            reason: DenialReason::InvalidState,
        };
    };
    let next_local = match lifecycle_target(from, event.action) {
        Ok(next) => next,
        Err(error) => {
            return Decision::Deny {
                reason: denial_for_transition(error),
            };
        }
    };
    if matches!(event.action, Action::MarkReady | Action::Start)
        && !dependencies_complete(machine, state, &node_id)
    {
        return Decision::Deny {
            reason: DenialReason::DependenciesIncomplete,
        };
    }

    let selector = TransitionSelector {
        from,
        action: event.action,
    };
    let intents = match derive_intents(machine, &node_id, selector, event) {
        Ok(intents) => intents,
        Err(reason) => return Decision::Deny { reason },
    };

    let mut next = state.clone();
    next.set(node_id, next_local);
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
