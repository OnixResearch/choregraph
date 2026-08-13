#![no_std]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the adapter keeps the exact reviewed machine and history boundary vocabulary"
)]

//! One-way history adapter that delegates to the production replay entrypoints.

extern crate alloc;

use alloc::vec::Vec;

use choregraph_core::{
    Action, AdmittedMachine, BoundedTrace, ConcreteBoundedTrace, ConcreteEvent,
    ConcreteReplayInput, Event, MachineState, ReplayBindings, ReplayDecision, ReplayInput,
    ReplayLimits,
};
use choregraph_history::{Digest, EventClass, EventId, HistoryGraph, Projection, ProjectionKind};

// r[impl choregraph.history.machine_replay]
// r[impl choregraph.history.machine_replay.valid]
// r[impl choregraph.history.machine_replay.rejects]
const EVENT_V1_DOMAIN: &[u8] = b"choregraph.history.machine-event.v1";
const EVENT_V2_DOMAIN: &[u8] = b"choregraph.history.machine-event.v2";
const ACTION_MARK_READY_TAG: u8 = 0;
const ACTION_START_TAG: u8 = 1;
const ACTION_COMPLETE_TAG: u8 = 2;
const ACTION_FAIL_TAG: u8 = 3;
const ACTION_CANCEL_TAG: u8 = 4;
const ACTION_SUSPEND_TAG: u8 = 5;
const ACTION_RESUME_SIGNAL_TAG: u8 = 6;
const LENGTH_U64_TAG: u8 = 0;
const LENGTH_NATIVE_TAG: u8 = 1;

/// One history event bound to its aggregate production-machine event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundEventV1 {
    pub history_event: EventId,
    pub event: Event,
}

/// One history event bound to its concrete production-machine event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundEventV2 {
    pub history_event: EventId,
    pub event: ConcreteEvent,
}

/// Explicit input for aggregate production replay.
pub struct ReplayHistoryInputV1<'a> {
    pub graph: &'a HistoryGraph,
    pub lineage: &'a Projection,
    pub machine: &'a AdmittedMachine,
    pub initial_state: &'a MachineState,
    pub expected_machine_identity: Digest,
    pub supplied_machine_identity: Digest,
    pub expected_schema_identity: Digest,
    pub expected_bindings: ReplayBindings,
    pub trace_bindings: ReplayBindings,
    pub events: &'a [BoundEventV1],
    pub limits: ReplayLimits,
}

/// Explicit input for concrete production replay.
pub struct ReplayHistoryInputV2<'a> {
    pub graph: &'a HistoryGraph,
    pub lineage: &'a Projection,
    pub machine: &'a AdmittedMachine,
    pub initial_state: &'a MachineState,
    pub expected_machine_identity: Digest,
    pub supplied_machine_identity: Digest,
    pub expected_schema_identity: Digest,
    pub expected_bindings: ReplayBindings,
    pub trace_bindings: ReplayBindings,
    pub events: &'a [BoundEventV2],
    pub limits: ReplayLimits,
}

/// Fail-closed adapter denials before production replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineHistoryError {
    NonLinearProjection,
    EventCountMismatch,
    HistoryEventMissing { event: EventId },
    UnsupportedEventClass { event: EventId },
    UnresolvedMerge { event: EventId },
    StaleMachineIdentity,
    StaleSchemaIdentity { event: EventId },
    StaleBinding,
    PayloadIdentityMismatch { event: EventId },
}

/// Replays aggregate events through [`choregraph_core::replay`].
pub fn replay_history(
    input: ReplayHistoryInputV1<'_>,
) -> Result<ReplayDecision, MachineHistoryError> {
    validate_common(CommonValidation {
        graph: input.graph,
        lineage: input.lineage,
        expected_machine_identity: input.expected_machine_identity,
        supplied_machine_identity: input.supplied_machine_identity,
        expected_bindings: input.expected_bindings,
        trace_bindings: input.trace_bindings,
        event_ids: input.events.iter().map(|binding| binding.history_event),
    })?;
    debug_assert_eq!(input.lineage.kind(), ProjectionKind::LinearLineage);
    debug_assert_eq!(input.lineage.events().len(), input.events.len());
    let mut events = Vec::with_capacity(input.events.len());
    for binding in input.events {
        validate_history_event(
            input.graph,
            binding.history_event,
            EventClass::MachineV1,
            input.expected_schema_identity,
            machine_event_payload_identity(&binding.event),
        )?;
        events.push(binding.event.clone());
    }
    let trace = BoundedTrace::new(input.trace_bindings, &events);
    let replay_input = ReplayInput::new(
        input.machine,
        input.initial_state,
        input.expected_bindings,
        trace,
        input.limits,
    );
    Ok(choregraph_core::replay(replay_input))
}

/// Replays concrete events through [`choregraph_core::replay_concrete`].
pub fn replay_history_concrete(
    input: ReplayHistoryInputV2<'_>,
) -> Result<ReplayDecision, MachineHistoryError> {
    validate_common(CommonValidation {
        graph: input.graph,
        lineage: input.lineage,
        expected_machine_identity: input.expected_machine_identity,
        supplied_machine_identity: input.supplied_machine_identity,
        expected_bindings: input.expected_bindings,
        trace_bindings: input.trace_bindings,
        event_ids: input.events.iter().map(|binding| binding.history_event),
    })?;
    debug_assert_eq!(input.lineage.kind(), ProjectionKind::LinearLineage);
    debug_assert_eq!(input.lineage.events().len(), input.events.len());
    let mut events = Vec::with_capacity(input.events.len());
    for binding in input.events {
        validate_history_event(
            input.graph,
            binding.history_event,
            EventClass::MachineV2,
            input.expected_schema_identity,
            concrete_event_payload_identity(&binding.event),
        )?;
        events.push(binding.event.clone());
    }
    let trace = ConcreteBoundedTrace::new(input.trace_bindings, &events);
    let replay_input = ConcreteReplayInput::new(
        input.machine,
        input.initial_state,
        input.expected_bindings,
        trace,
        input.limits,
    );
    Ok(choregraph_core::replay_concrete(replay_input))
}

/// Computes the canonical payload identity for one aggregate machine event.
#[must_use]
pub fn machine_event_payload_identity(event: &Event) -> Digest {
    let mut hasher = start_hasher(EVENT_V1_DOMAIN, &event.node_id, event.action);
    update_length(&mut hasher, event.effects.len());
    for effect in &event.effects {
        for value in [
            &effect.operation,
            &effect.resource,
            &effect.ability,
            &effect.request,
            &effect.payload,
        ] {
            update_bytes(&mut hasher, value.as_bytes());
        }
    }
    update_suspension(&mut hasher, event.suspension.as_ref());
    Digest::from_bytes(*hasher.finalize().as_bytes())
}

/// Computes the canonical payload identity for one concrete machine event.
#[must_use]
pub fn concrete_event_payload_identity(event: &ConcreteEvent) -> Digest {
    let mut hasher = start_hasher(EVENT_V2_DOMAIN, &event.node_id, event.action);
    update_length(&mut hasher, event.effects.len());
    for effect in &event.effects {
        update_bytes(&mut hasher, effect.declaration_key.as_bytes());
        update_binding_input(&mut hasher, &effect.request);
        update_binding_input(&mut hasher, &effect.payload);
    }
    update_suspension(&mut hasher, event.suspension.as_ref());
    Digest::from_bytes(*hasher.finalize().as_bytes())
}

struct CommonValidation<'a, I> {
    graph: &'a HistoryGraph,
    lineage: &'a Projection,
    expected_machine_identity: Digest,
    supplied_machine_identity: Digest,
    expected_bindings: ReplayBindings,
    trace_bindings: ReplayBindings,
    event_ids: I,
}

fn validate_common<I: Iterator<Item = EventId>>(
    input: CommonValidation<'_, I>,
) -> Result<(), MachineHistoryError> {
    if input.lineage.kind() != ProjectionKind::LinearLineage {
        return Err(MachineHistoryError::NonLinearProjection);
    }
    if input.expected_machine_identity != input.supplied_machine_identity {
        return Err(MachineHistoryError::StaleMachineIdentity);
    }
    if input.expected_bindings != input.trace_bindings {
        return Err(MachineHistoryError::StaleBinding);
    }
    debug_assert_eq!(
        input.expected_machine_identity,
        input.supplied_machine_identity
    );
    debug_assert_eq!(input.expected_bindings, input.trace_bindings);
    let supplied: Vec<EventId> = input.event_ids.collect();
    if input.lineage.events() != supplied {
        return Err(MachineHistoryError::EventCountMismatch);
    }
    for event in input.lineage.events() {
        if !input.graph.contains(*event) {
            return Err(MachineHistoryError::HistoryEventMissing { event: *event });
        }
    }
    Ok(())
}

fn validate_history_event(
    graph: &HistoryGraph,
    identity: EventId,
    expected_class: EventClass,
    expected_schema: Digest,
    expected_payload: Digest,
) -> Result<(), MachineHistoryError> {
    let event = graph
        .event(identity)
        .ok_or(MachineHistoryError::HistoryEventMissing { event: identity })?;
    if event.content().class == EventClass::Merge {
        return Err(MachineHistoryError::UnresolvedMerge { event: identity });
    }
    if event.content().class != expected_class {
        return Err(MachineHistoryError::UnsupportedEventClass { event: identity });
    }
    if event.content().schema_identity != expected_schema {
        return Err(MachineHistoryError::StaleSchemaIdentity { event: identity });
    }
    if event.content().payload_identity != expected_payload {
        return Err(MachineHistoryError::PayloadIdentityMismatch { event: identity });
    }
    Ok(())
}

fn start_hasher(domain: &[u8], node_id: &str, action: Action) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    update_bytes(&mut hasher, domain);
    update_bytes(&mut hasher, node_id.as_bytes());
    hasher.update(&[action_tag(action)]);
    hasher
}

const fn action_tag(action: Action) -> u8 {
    match action {
        Action::MarkReady => ACTION_MARK_READY_TAG,
        Action::Start => ACTION_START_TAG,
        Action::Complete => ACTION_COMPLETE_TAG,
        Action::Fail => ACTION_FAIL_TAG,
        Action::Cancel => ACTION_CANCEL_TAG,
        Action::Suspend => ACTION_SUSPEND_TAG,
        Action::ResumeSignal => ACTION_RESUME_SIGNAL_TAG,
    }
}

fn update_binding_input(
    hasher: &mut blake3::Hasher,
    input: &choregraph_core::ConcreteBindingInput,
) {
    update_optional_text(hasher, input.slot_id.as_deref());
    update_optional_text(hasher, input.source_ref.as_deref());
    update_bytes(hasher, input.reference.profile.as_bytes());
    update_bytes(hasher, input.reference.identity.as_bytes());
}

#[allow(
    missing_const_fn,
    reason = "BLAKE3 hasher mutation is not a const operation"
)]
fn update_suspension(
    hasher: &mut blake3::Hasher,
    suspension: Option<&choregraph_core::SuspensionFactSpec>,
) {
    match suspension {
        Some(fact) => {
            hasher.update(&[1]);
            update_bytes(hasher, fact.class.as_bytes());
            update_bytes(hasher, fact.reference.as_bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

#[allow(
    missing_const_fn,
    reason = "BLAKE3 hasher mutation is not a const operation"
)]
fn update_optional_text(hasher: &mut blake3::Hasher, value: Option<&str>) {
    match value {
        Some(text) => {
            hasher.update(&[1]);
            update_bytes(hasher, text.as_bytes());
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

#[allow(
    missing_const_fn,
    reason = "BLAKE3 hasher mutation is not a const operation"
)]
fn update_length(hasher: &mut blake3::Hasher, length: usize) {
    match u64::try_from(length) {
        Ok(framed) => {
            hasher.update(&[LENGTH_U64_TAG]);
            hasher.update(&framed.to_le_bytes());
        }
        Err(_) => {
            hasher.update(&[LENGTH_NATIVE_TAG]);
            hasher.update(&length.to_le_bytes());
        }
    }
}

fn update_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    update_length(hasher, bytes.len());
    hasher.update(bytes);
}
