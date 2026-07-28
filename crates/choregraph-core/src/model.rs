// The canonical public IR remains in one reviewable file with explicit value imports.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the canonical public IR remains in one reviewable file with explicit value imports"
)]

// r[impl choregraph.machine.ir]
// r[impl choregraph.machine.bounds]
// r[impl choregraph.machine.parameterized_state]
// r[impl choregraph.machine.effect_intents]
// r[impl choregraph.machine.denial]
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::concrete::ConcreteEffectDeclaration;
use crate::id::{
    AbilityId, DeclarationKey, NodeId, OperationId, PayloadRef, RequestId, ResourceId,
};

pub const MACHINE_SCHEMA_V1: &str = "choregraph.machine.v1";
pub const MACHINE_SCHEMA_V2: &str = "choregraph.machine.v2";
pub const LINEAR_LIFECYCLE_V1: &str = "choregraph.lifecycle.linear.v1";

pub const DEFAULT_MAX_NODES: usize = 4_096;
pub const DEFAULT_MAX_EDGES: usize = 16_384;
pub const DEFAULT_MAX_OPERATIONS: usize = 16_384;
pub const DEFAULT_MAX_EFFECT_DECLARATIONS: usize = 16_384;
pub const DEFAULT_MAX_IDENTIFIER_BYTES: usize = 256;
pub const DEFAULT_MAX_REFERENCE_BYTES: usize = crate::id::BLAKE3_REFERENCE_BYTES;
pub const DEFAULT_MAX_EVENT_BYTES: usize = 65_536;
pub const DEFAULT_MAX_INTENT_BYTES: usize = 2_048;
pub const DEFAULT_MAX_INTENTS_PER_TRANSITION: usize = 64;
pub const DEFAULT_MAX_TRACE_EVENTS: u64 = 65_536;
pub const TRACE_BINDING_BYTES: usize = 32;

/// Bounded admission and transition limits selected by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionLimits {
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_operations: usize,
    pub max_effect_declarations: usize,
    pub max_identifier_bytes: usize,
    pub max_reference_bytes: usize,
    pub max_event_bytes: usize,
    pub max_intent_bytes: usize,
    pub max_intents_per_transition: usize,
}

impl Default for AdmissionLimits {
    fn default() -> Self {
        Self {
            max_nodes: DEFAULT_MAX_NODES,
            max_edges: DEFAULT_MAX_EDGES,
            max_operations: DEFAULT_MAX_OPERATIONS,
            max_effect_declarations: DEFAULT_MAX_EFFECT_DECLARATIONS,
            max_identifier_bytes: DEFAULT_MAX_IDENTIFIER_BYTES,
            max_reference_bytes: DEFAULT_MAX_REFERENCE_BYTES,
            max_event_bytes: DEFAULT_MAX_EVENT_BYTES,
            max_intent_bytes: DEFAULT_MAX_INTENT_BYTES,
            max_intents_per_transition: DEFAULT_MAX_INTENTS_PER_TRANSITION,
        }
    }
}

/// One untrusted node declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeSpec {
    pub id: String,
    pub operations: Vec<String>,
}

impl NodeSpec {
    #[must_use]
    pub fn new(id: impl Into<String>, operations: Vec<String>) -> Self {
        Self {
            id: id.into(),
            operations,
        }
    }
}

/// One untrusted directed dependency edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdgeSpec {
    pub source: String,
    pub target: String,
}

impl EdgeSpec {
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

/// Local lifecycle states for one admitted node.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LocalState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Events supported by the linear lifecycle profile.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
}

/// A local lifecycle transition failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalTransitionError {
    Terminal { from: LocalState, action: Action },
    Invalid { from: LocalState, action: Action },
}

/// One exact lifecycle transition selector.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TransitionSelector {
    pub from: LocalState,
    pub action: Action,
}

/// One untrusted effect declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDeclarationSpec {
    pub node_id: String,
    pub transition: TransitionSelector,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

/// One untrusted canonical machine specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineSpec {
    pub schema: String,
    pub lifecycle_profile: String,
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<EdgeSpec>,
    pub effects: Vec<EffectDeclarationSpec>,
}

impl MachineSpec {
    #[must_use]
    pub fn new(nodes: Vec<NodeSpec>, edges: Vec<EdgeSpec>) -> Self {
        Self {
            schema: String::from(MACHINE_SCHEMA_V1),
            lifecycle_profile: String::from(LINEAR_LIFECYCLE_V1),
            nodes,
            edges,
            effects: Vec::new(),
        }
    }
}

/// The selected machine schema after admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MachineSchema {
    V1,
    V2,
}

/// The selected local lifecycle profile after admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleProfile {
    LinearV1,
}

/// One admitted immutable node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedNode {
    pub(crate) operations: BTreeSet<OperationId>,
}

/// One normalized immutable edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
}

/// One exact admitted effect declaration.
///
/// A declaration permits a matching event request. It does not require every event to request it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EffectDeclaration {
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request: RequestId,
    pub payload: PayloadRef,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct EffectKey {
    pub(crate) node_id: NodeId,
    pub(crate) transition: TransitionSelector,
}

/// An admitted machine with normalized immutable indexes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedMachine {
    pub(crate) limits: AdmissionLimits,
    pub(crate) schema: MachineSchema,
    pub(crate) lifecycle_profile: LifecycleProfile,
    pub(crate) nodes: BTreeMap<NodeId, AdmittedNode>,
    pub(crate) node_order: Vec<NodeId>,
    pub(crate) node_indices: BTreeMap<NodeId, usize>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) edge_indices: Vec<(usize, usize)>,
    pub(crate) dependencies: BTreeMap<NodeId, Vec<NodeId>>,
    pub(crate) dependency_indices: Vec<Vec<usize>>,
    pub(crate) dependents: BTreeMap<NodeId, Vec<NodeId>>,
    pub(crate) topological_order: Vec<NodeId>,
    pub(crate) topological_indices: Vec<usize>,
    pub(crate) effects: Vec<EffectDeclaration>,
    pub(crate) effect_index: BTreeMap<EffectKey, Vec<EffectDeclaration>>,
    pub(crate) effect_route_indices: BTreeMap<EffectKey, Vec<usize>>,
    pub(crate) concrete_effects: Vec<ConcreteEffectDeclaration>,
    pub(crate) concrete_effect_key_indices: BTreeMap<DeclarationKey, usize>,
    pub(crate) concrete_effect_route_indices: BTreeMap<EffectKey, Vec<usize>>,
}

impl AdmittedMachine {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        match self.schema {
            MachineSchema::V1 => MACHINE_SCHEMA_V1,
            MachineSchema::V2 => MACHINE_SCHEMA_V2,
        }
    }

    #[must_use]
    pub const fn schema_version(&self) -> MachineSchema {
        self.schema
    }

    #[must_use]
    pub const fn lifecycle_profile(&self) -> LifecycleProfile {
        self.lifecycle_profile
    }

    #[must_use]
    pub const fn limits(&self) -> AdmissionLimits {
        self.limits
    }

    #[must_use]
    pub fn node_ids(&self) -> &[NodeId] {
        &self.node_order
    }

    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    #[must_use]
    pub fn topological_order(&self) -> &[NodeId] {
        &self.topological_order
    }

    #[must_use]
    pub fn dependencies(&self, node_id: &NodeId) -> Option<&[NodeId]> {
        self.dependencies.get(node_id).map(Vec::as_slice)
    }

    #[must_use]
    pub fn operations(&self, node_id: &NodeId) -> Option<&BTreeSet<OperationId>> {
        self.nodes.get(node_id).map(|node| &node.operations)
    }

    #[must_use]
    pub fn dependents(&self, node_id: &NodeId) -> Option<&[NodeId]> {
        self.dependents.get(node_id).map(Vec::as_slice)
    }

    #[must_use]
    pub fn effect_declarations(&self) -> &[EffectDeclaration] {
        &self.effects
    }

    #[must_use]
    pub fn concrete_effect_declarations(&self) -> &[ConcreteEffectDeclaration] {
        &self.concrete_effects
    }

    #[must_use]
    pub fn initial_state(&self) -> MachineState {
        let entries = self
            .node_order
            .iter()
            .cloned()
            .map(|node_id| (node_id, LocalState::Pending))
            .collect();
        MachineState { entries }
    }
}

/// Parameterized aggregate state with one entry per admitted node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineState {
    entries: BTreeMap<NodeId, LocalState>,
}

impl MachineState {
    #[must_use]
    pub fn new(entries: BTreeMap<NodeId, LocalState>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub fn entries(&self) -> &BTreeMap<NodeId, LocalState> {
        &self.entries
    }

    #[must_use]
    pub fn state(&self, node_id: &NodeId) -> Option<LocalState> {
        self.entries.get(node_id).copied()
    }
}

/// One raw effect request carried by an event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRequest {
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

/// One bounded lifecycle event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub node_id: String,
    pub action: Action,
    pub effects: Vec<EffectRequest>,
}

impl Event {
    #[must_use]
    pub fn new(node_id: impl Into<String>, action: Action) -> Self {
        Self {
            node_id: node_id.into(),
            action,
            effects: Vec::new(),
        }
    }
}

/// One declared intent emitted by an accepted transition.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EffectIntent {
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request: RequestId,
    pub payload: PayloadRef,
}

impl From<&EffectDeclaration> for EffectIntent {
    fn from(declaration: &EffectDeclaration) -> Self {
        Self {
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
            operation: declaration.operation.clone(),
            resource: declaration.resource.clone(),
            ability: declaration.ability.clone(),
            request: declaration.request.clone(),
            payload: declaration.payload.clone(),
        }
    }
}

/// One caller-supplied canonical BLAKE3 binding.
///
/// The core compares bindings but does not compute or authorize them. A consumer adapter must
/// derive these bytes from its admitted canonical machine and initial-state artifacts.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TraceBinding([u8; TRACE_BINDING_BYTES]);

impl TraceBinding {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; TRACE_BINDING_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn bytes(self) -> [u8; TRACE_BINDING_BYTES] {
        self.0
    }
}

/// Exact machine and initial-state identities carried by a trace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayBindings {
    pub machine: TraceBinding,
    pub initial_state: TraceBinding,
}

impl ReplayBindings {
    #[must_use]
    pub const fn new(machine: TraceBinding, initial_state: TraceBinding) -> Self {
        Self {
            machine,
            initial_state,
        }
    }
}

/// Caller-selected finite replay limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayLimits {
    pub max_events: u64,
}

impl Default for ReplayLimits {
    fn default() -> Self {
        Self {
            max_events: DEFAULT_MAX_TRACE_EVENTS,
        }
    }
}

/// An ordered event slice with exact canonical identity bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedTrace<'a> {
    bindings: ReplayBindings,
    events: &'a [Event],
}

impl<'a> BoundedTrace<'a> {
    #[must_use]
    pub const fn new(bindings: ReplayBindings, events: &'a [Event]) -> Self {
        Self { bindings, events }
    }

    #[must_use]
    pub const fn bindings(&self) -> ReplayBindings {
        self.bindings
    }

    #[must_use]
    pub const fn events(&self) -> &'a [Event] {
        self.events
    }
}

/// All explicit inputs to one deterministic replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayInput<'a> {
    pub(crate) machine: &'a AdmittedMachine,
    pub(crate) initial_state: &'a MachineState,
    pub(crate) expected_bindings: ReplayBindings,
    pub(crate) trace: BoundedTrace<'a>,
    pub(crate) limits: ReplayLimits,
}

impl<'a> ReplayInput<'a> {
    #[must_use]
    pub const fn new(
        machine: &'a AdmittedMachine,
        initial_state: &'a MachineState,
        expected_bindings: ReplayBindings,
        trace: BoundedTrace<'a>,
        limits: ReplayLimits,
    ) -> Self {
        Self {
            machine,
            initial_state,
            expected_bindings,
            trace,
            limits,
        }
    }
}

/// The number of accepted events in a replay prefix.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConsumedCount(u64);

impl ConsumedCount {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn from_usize(value: usize) -> Self {
        match u64::try_from(value) {
            Ok(value) => Self(value),
            Err(_) => Self(u64::MAX),
        }
    }
}

/// The zero-based position of the first denied event.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EventIndex(u64);

impl EventIndex {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn from_usize(value: usize) -> Self {
        match u64::try_from(value) {
            Ok(value) => Self(value),
            Err(_) => Self(u64::MAX),
        }
    }
}

/// The state after the accepted prefix and before a denied event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LastValidState(MachineState);

impl LastValidState {
    #[must_use]
    pub fn new(state: MachineState) -> Self {
        Self(state)
    }

    #[must_use]
    pub const fn state(&self) -> &MachineState {
        &self.0
    }

    #[must_use]
    pub fn into_state(self) -> MachineState {
        self.0
    }
}

/// Stable reasons that stop replay before the full trace is consumed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayFailureReason {
    TraceLimitExceeded { observed: u64, maximum: u64 },
    MachineBindingMismatch,
    InitialStateBindingMismatch,
    InitialStateInvalid,
    TransitionDenied(DenialReason),
}

/// Exact first-failure information with the accepted-prefix state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayFailure {
    pub event_index: Option<EventIndex>,
    pub consumed: ConsumedCount,
    pub reason: ReplayFailureReason,
    pub last_valid_state: LastValidState,
}

/// Pure replay output. This type contains no effect intent or dispatch authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayDecision {
    Complete {
        final_state: MachineState,
        consumed: ConsumedCount,
    },
    Failed(ReplayFailure),
}

/// The result of one pure transition request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    Accept {
        next: MachineState,
        intents: Vec<EffectIntent>,
    },
    Deny {
        reason: DenialReason,
    },
}

/// Stable denial reasons. Denials contain no replacement state or intents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DenialReason {
    InvalidState,
    EventTooLarge { observed: usize, maximum: usize },
    TooManyEffectRequests { observed: usize, maximum: usize },
    InvalidNodeId,
    UnknownNode,
    TerminalState { from: LocalState, action: Action },
    InvalidTransition { from: LocalState, action: Action },
    DependenciesIncomplete,
    MalformedEffectRequest,
    DuplicateEffectRequest,
    UndeclaredEffect,
    UnknownDeclaration,
    ConcreteBindingMismatch,
    ConcreteIntentTooLarge { observed: usize, maximum: usize },
    UnsupportedMachineSchema,
    InvariantWouldBreak,
    EffectsNotAllowed,
}

/// Stable admission limit categories.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LimitKind {
    Nodes,
    Edges,
    Operations,
    EffectDeclarations,
    IdentifierBytes,
    ReferenceBytes,
    EventBytes,
    IntentBytes,
    IntentsPerTransition,
}

/// Stable admission diagnostic categories.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AdmissionDiagnosticCode {
    InconsistentLimits,
    TooManyNodes,
    TooManyEdges,
    TooManyOperations,
    TooManyEffectDeclarations,
    UnsupportedSchema,
    UnsupportedLifecycleProfile,
    EmptyIdentifier,
    IdentifierTooLong,
    MalformedIdentifier,
    DuplicateNode,
    DuplicateOperation,
    DuplicateEdge,
    MissingSource,
    MissingTarget,
    SelfLoop,
    CycleDetected,
    EffectNodeMissing,
    EffectOperationUndeclared,
    InvalidEffectTransition,
    DuplicateEffectDeclaration,
    DuplicateDeclarationKey,
    DuplicateConcreteEffectDeclaration,
    UnknownReferenceProfile,
    MalformedBinding,
    MalformedReference,
    ReferenceTooLong,
    IntentTooLarge,
    TooManyIntentsForTransition,
    NormalizationInvariant,
}

/// One deterministic admission diagnostic.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AdmissionDiagnostic {
    pub code: AdmissionDiagnosticCode,
    pub subject: Option<String>,
    pub related: Option<String>,
    pub limit: Option<LimitKind>,
    pub observed: Option<usize>,
    pub maximum: Option<usize>,
}

/// Executable state-invariant failures.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InvariantIssue {
    MissingNode(NodeId),
    ExtraNode(NodeId),
    DependenciesIncomplete(NodeId),
}

/// Resume behavior compatible with the current execution-graph vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeDecision {
    Execute,
    AwaitDependencies,
    SkipCompleted,
    RejectTerminalMutation,
    BlockedByFailedDependency,
    UnknownNode,
    InvalidState,
}

/// One deterministic graph level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParallelGroup {
    pub node_ids: Vec<NodeId>,
    pub level: usize,
    pub can_parallelize: bool,
}

/// One node ranked by transitive dependent count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bottleneck {
    pub node_id: NodeId,
    pub dependent_count: usize,
}
