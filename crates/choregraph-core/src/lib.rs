#![no_std]

extern crate alloc;

mod admission;
mod analytics;
mod boundary;
mod concrete;
mod id;
mod invariant;
mod model;
mod transition;
mod verified;
#[path = "verified/topology.rs"]
mod verified_topology;
#[path = "verified/trace.rs"]
mod verified_trace;

pub use analytics::{
    bottlenecks, critical_path, parallel_groups, ready_nodes, resume_decision, would_create_cycle,
};
pub use boundary::{
    Capability, FOUNDATION_CLAIM, MACHINE_CLAIM, SUSPENSION_NON_CLAIMS,
    TRANSITION_IMPLEMENTATION_POLICY, TransitionImplementationPolicy, excluded_capabilities,
    permits,
};
pub use concrete::{
    CANONICAL_REFERENCE_PROFILE_V1, CanonicalReferenceProfile, ConcreteBindingInput,
    ConcreteBoundedTrace, ConcreteDecision, ConcreteEffectDeclaration,
    ConcreteEffectDeclarationSpec, ConcreteEffectIntent, ConcreteEffectRequest, ConcreteEvent,
    ConcreteReference, ConcreteReferenceSpec, ConcreteReplayInput, MachineSpecV2, ReferenceBinding,
    ReferenceBindingSpec, concrete_effects_allowed,
};
pub use id::{
    AbilityId, BLAKE3_DIGEST_HEX_BYTES, BLAKE3_REFERENCE_BYTES, BLAKE3_REFERENCE_PREFIX, BoundedId,
    DeclarationKey, IdentifierError, NodeId, OperationId, PayloadRef, PayloadRefError, RequestId,
    ResourceId, SlotId, SourceRef, SuspensionClass,
};
pub use invariant::{
    effects_allowed, invariant, terminal_states_preserved, validate_invariant, well_formed,
};
pub use model::{
    Action, AdmissionDiagnostic, AdmissionDiagnosticCode, AdmissionLimits, AdmittedMachine,
    Bottleneck, BoundedTrace, ConsumedCount, DEFAULT_MAX_EDGES, DEFAULT_MAX_EFFECT_DECLARATIONS,
    DEFAULT_MAX_EVENT_BYTES, DEFAULT_MAX_IDENTIFIER_BYTES, DEFAULT_MAX_INTENT_BYTES,
    DEFAULT_MAX_INTENTS_PER_TRANSITION, DEFAULT_MAX_NODES, DEFAULT_MAX_OPERATIONS,
    DEFAULT_MAX_REFERENCE_BYTES, DEFAULT_MAX_TRACE_EVENTS,
    DURABLE_SUSPENSION_V1, Decision, DenialReason, Edge, EdgeSpec, EffectDeclaration,
    EffectDeclarationSpec, EffectIntent, EffectRequest, Event, EventIndex, InvariantIssue,
    LINEAR_LIFECYCLE_V1, LastValidState, LifecycleProfile, LimitKind, LocalState,
    LocalTransitionError, MACHINE_SCHEMA_V1, MACHINE_SCHEMA_V2, MachineSchema, MachineSpec,
    MachineState, NodeSpec, ParallelGroup, ReplayBindings, ReplayDecision, ReplayFailure,
    ReplayFailureReason, ReplayInput, ReplayLimits, ResumeDecision, SuspensionDeclaration,
    SuspensionDeclarationSpec, SuspensionFactSpec, SuspensionRecord, TRACE_BINDING_BYTES,
    TraceBinding, TransitionSelector,
};
pub use transition::{
    lifecycle_target, lifecycle_target_for_profile, replay, replay_concrete, step, step_concrete,
};
