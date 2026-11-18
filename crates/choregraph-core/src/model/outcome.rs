use crate::id::bounded::NodeId;
use crate::model::automaton::MachineState;
use crate::model::dispatch::EffectIntent;
use crate::model::semantics::{Action, LocalState};
use alloc::string::String;
use alloc::vec::Vec;

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
    UnsupportedProfileAction { action: Action },
    SuspensionFactMissing,
    SuspensionFactUnexpected,
    MalformedSuspensionFact,
    SuspensionMismatch,
    SuspensionEffectsForbidden,
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
    DuplicateSuspensionDeclaration,
    SuspensionDeclarationNodeMissing,
    SuspensionDeclarationWithoutProfile,
    MalformedSuspensionClass,
    MalformedSuspensionReference,
    SuspensionEffectTransition,
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
    SuspensionRecordMissing(NodeId),
    SuspensionRecordUnexpected(NodeId),
    SuspensionWithoutProfile(NodeId),
}
