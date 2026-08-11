// r[impl choregraph.wasm.abi]
// r[impl choregraph.wasm.bounds]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "canonical ABI DTO definitions keep serde derives and artifact identity field types concise and reviewable"
)]

use choregraph_kamacite::ArtifactIdentity;
use serde::{Deserialize, Serialize};

pub const ABI_VERSION: u32 = 1;
pub const CALL_SCHEMA_VERSION: &str = "choregraph.wasm-call.v1";
pub const CALL_ROOT_LABEL: &str = "choregraph-wasm-call-v1";
pub const RESULT_SCHEMA_VERSION: &str = "choregraph.wasm-result.v1";
pub const RESULT_ROOT_LABEL: &str = "choregraph-wasm-result-v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-wasm-call-v1", deny_unknown_fields)]
pub struct KernelCall {
    pub abi_version: u32,
    pub operation: OperationRequest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationRequest {
    AdmitMachine {
        artifacts: MachineArtifacts,
    },
    Step {
        artifacts: MachineArtifacts,
        state: StateValue,
        event: EventValue,
    },
    StepConcrete {
        artifacts: MachineArtifacts,
        state: StateValue,
        event: ConcreteEventValue,
    },
    Replay {
        artifacts: MachineArtifacts,
        initial_state: StateValue,
        bindings: ReplayBindingPairValue,
        max_events: u64,
        events: Vec<EventValue>,
    },
    ReplayConcrete {
        artifacts: MachineArtifacts,
        initial_state: StateValue,
        bindings: ReplayBindingPairValue,
        max_events: u64,
        events: Vec<ConcreteEventValue>,
    },
}

impl OperationRequest {
    #[must_use]
    pub const fn tag(&self) -> OperationTag {
        match self {
            Self::AdmitMachine { .. } => OperationTag::AdmitMachine,
            Self::Step { .. } => OperationTag::Step,
            Self::StepConcrete { .. } => OperationTag::StepConcrete,
            Self::Replay { .. } => OperationTag::Replay,
            Self::ReplayConcrete { .. } => OperationTag::ReplayConcrete,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationTag {
    AdmitMachine,
    Step,
    StepConcrete,
    Replay,
    ReplayConcrete,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MachineArtifactKind {
    LinearV1,
    DurableV1,
    ConcreteV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MachineArtifacts {
    pub kind: MachineArtifactKind,
    #[serde(with = "serde_bytes")]
    pub machine: Vec<u8>,
    pub machine_identity: ArtifactIdentity,
    #[serde(with = "optional_bytes")]
    pub lifecycle: Option<Vec<u8>>,
    pub lifecycle_identity: Option<ArtifactIdentity>,
    pub profile_identity: Option<ArtifactIdentity>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateTag {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
    Suspended,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionTag {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
    Suspend,
    ResumeSignal,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateEntryValue {
    pub node_id: String,
    pub state: StateTag,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuspensionValue {
    pub node_id: String,
    pub class: String,
    pub reference: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateValue {
    pub entries: Vec<StateEntryValue>,
    pub suspensions: Vec<SuspensionValue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuspensionFactValue {
    pub class: String,
    pub reference: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRequestValue {
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventValue {
    pub node_id: String,
    pub action: ActionTag,
    pub effects: Vec<EffectRequestValue>,
    pub suspension: Option<SuspensionFactValue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConcreteReferenceValue {
    pub profile: String,
    pub identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConcreteBindingValue {
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
    pub reference: ConcreteReferenceValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConcreteEffectRequestValue {
    pub declaration_key: String,
    pub request: ConcreteBindingValue,
    pub payload: ConcreteBindingValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConcreteEventValue {
    pub node_id: String,
    pub action: ActionTag,
    pub effects: Vec<ConcreteEffectRequestValue>,
    pub suspension: Option<SuspensionFactValue>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayBindingsValue {
    #[serde(with = "serde_bytes")]
    pub machine: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub initial_state: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayBindingPairValue {
    pub expected: ReplayBindingsValue,
    pub trace: ReplayBindingsValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-wasm-result-v1", deny_unknown_fields)]
pub struct KernelResult {
    pub abi_version: u32,
    pub operation: Option<OperationTag>,
    pub input_identity: String,
    pub outcome: Outcome,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    Admitted {
        machine_identity: ArtifactIdentity,
        initial_state: StateValue,
    },
    Accepted {
        machine_identity: ArtifactIdentity,
        next: StateValue,
        intents: Vec<IntentValue>,
    },
    Denied {
        machine_identity: ArtifactIdentity,
        reason: DenialValue,
    },
    ReplayComplete {
        machine_identity: ArtifactIdentity,
        final_state: StateValue,
        consumed: u64,
    },
    ReplayFailed {
        machine_identity: ArtifactIdentity,
        failure: ReplayFailureValue,
    },
    AdapterError {
        error: AdapterErrorValue,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentValue {
    Declared {
        node_id: String,
        transition: TransitionValue,
        operation: String,
        resource: String,
        ability: String,
        request: String,
        payload: String,
    },
    Concrete {
        declaration_key: String,
        node_id: String,
        transition: TransitionValue,
        operation: String,
        resource: String,
        ability: String,
        request_slot: Option<String>,
        request_source_ref: Option<String>,
        request_profile: String,
        request_reference: String,
        payload_slot: Option<String>,
        payload_source_ref: Option<String>,
        payload_profile: String,
        payload_reference: String,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionValue {
    pub from: StateTag,
    pub action: ActionTag,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DenialValue {
    pub code: DenialCode,
    pub observed: Option<u64>,
    pub maximum: Option<u64>,
    pub from: Option<StateTag>,
    pub action: Option<ActionTag>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DenialCode {
    InvalidState,
    EventTooLarge,
    TooManyEffectRequests,
    InvalidNodeId,
    UnknownNode,
    TerminalState,
    InvalidTransition,
    DependenciesIncomplete,
    MalformedEffectRequest,
    DuplicateEffectRequest,
    UndeclaredEffect,
    UnknownDeclaration,
    ConcreteBindingMismatch,
    ConcreteIntentTooLarge,
    UnsupportedMachineSchema,
    UnsupportedProfileAction,
    SuspensionFactMissing,
    SuspensionFactUnexpected,
    MalformedSuspensionFact,
    SuspensionMismatch,
    SuspensionEffectsForbidden,
    InvariantWouldBreak,
    EffectsNotAllowed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayFailureValue {
    pub event_index: Option<u64>,
    pub consumed: u64,
    pub reason: ReplayFailureReasonValue,
    pub last_valid_state: StateValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReplayFailureReasonValue {
    TraceLimitExceeded { observed: u64, maximum: u64 },
    MachineBindingMismatch,
    InitialStateBindingMismatch,
    InitialStateInvalid,
    TransitionDenied { reason: DenialValue },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterErrorValue {
    pub code: AdapterErrorCode,
    pub stage: AdapterStage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterStage {
    OuterRequest,
    Decode,
    Version,
    Artifact,
    State,
    Event,
    Trace,
    Dispatch,
    Encode,
    OuterResult,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterErrorCode {
    RequestTooLarge,
    MalformedPreserves,
    UnsupportedAbiVersion,
    ArtifactKindMismatch,
    MissingArtifactField,
    UnexpectedArtifactField,
    InvalidArtifact,
    IdentityMismatch,
    InvalidState,
    InvalidEvent,
    InvalidTraceBinding,
    TraceLimitExceeded,
    OperationProfileMismatch,
    ResultTooLarge,
    InternalEncodingFailure,
}

mod optional_bytes {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde_bytes::{ByteBuf, Bytes};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => serializer.serialize_some(Bytes::new(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<ByteBuf>::deserialize(deserializer).map(|value| value.map(ByteBuf::into_vec))
    }
}
