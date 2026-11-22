// r[impl choregraph.wasm.abi]
// r[impl choregraph.wasm.bounds]
pub mod profile {
    pub const ABI_VERSION: u32 = 1;
    pub const CALL_SCHEMA_VERSION: &str = "choregraph.wasm-call.v1";
    pub const CALL_ROOT_LABEL: &str = "choregraph-wasm-call-v1";
    pub const RESULT_SCHEMA_VERSION: &str = "choregraph.wasm-result.v1";
    pub const RESULT_ROOT_LABEL: &str = "choregraph-wasm-result-v1";

    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum OperationTag {
        AdmitMachine,
        Step,
        StepConcrete,
        Replay,
        ReplayConcrete,
    }
}

pub mod input {
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename = "choregraph-wasm-call-v1", deny_unknown_fields)]
    pub struct KernelCall {
        pub abi_version: u32,
        pub operation: crate::schema::input::OperationRequest,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum OperationRequest {
        AdmitMachine {
            artifacts: crate::schema::input::MachineArtifacts,
        },
        Step {
            artifacts: crate::schema::input::MachineArtifacts,
            state: crate::schema::snapshot::StateValue,
            event: crate::schema::invocation::EventValue,
        },
        StepConcrete {
            artifacts: crate::schema::input::MachineArtifacts,
            state: crate::schema::snapshot::StateValue,
            event: crate::schema::invocation::ConcreteEventValue,
        },
        Replay {
            artifacts: crate::schema::input::MachineArtifacts,
            initial_state: crate::schema::snapshot::StateValue,
            bindings: crate::schema::history::ReplayBindingPairValue,
            max_events: u64,
            events: Vec<crate::schema::invocation::EventValue>,
        },
        ReplayConcrete {
            artifacts: crate::schema::input::MachineArtifacts,
            initial_state: crate::schema::snapshot::StateValue,
            bindings: crate::schema::history::ReplayBindingPairValue,
            max_events: u64,
            events: Vec<crate::schema::invocation::ConcreteEventValue>,
        },
    }

    impl OperationRequest {
        /// Returns the operation identity admitted by ABI version 1.
        #[must_use]
        pub const fn tag(&self) -> Option<crate::schema::profile::OperationTag> {
            if matches!(self, Self::AdmitMachine { .. }) {
                Some(crate::schema::profile::OperationTag::AdmitMachine)
            } else if matches!(self, Self::Step { .. }) {
                Some(crate::schema::profile::OperationTag::Step)
            } else if matches!(self, Self::StepConcrete { .. }) {
                Some(crate::schema::profile::OperationTag::StepConcrete)
            } else if matches!(self, Self::Replay { .. }) {
                Some(crate::schema::profile::OperationTag::Replay)
            } else if matches!(self, Self::ReplayConcrete { .. }) {
                Some(crate::schema::profile::OperationTag::ReplayConcrete)
            } else {
                None
            }
        }
    }

    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum MachineArtifactKind {
        LinearV1,
        DurableV1,
        ConcreteV2,
        DurableV2,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct MachineArtifacts {
        pub kind: crate::schema::input::MachineArtifactKind,
        #[serde(with = "serde_bytes")]
        pub machine: Vec<u8>,
        pub machine_identity: choregraph_kamacite::identity::ArtifactIdentity,
        #[serde(with = "crate::schema::optional_bytes")]
        pub lifecycle: Option<Vec<u8>>,
        pub lifecycle_identity: Option<choregraph_kamacite::identity::ArtifactIdentity>,
        pub profile_identity: Option<choregraph_kamacite::identity::ArtifactIdentity>,
    }
}

pub mod snapshot {
    #[derive(
        Clone, Copy, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize,
    )]
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

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct StateEntryValue {
        pub node_id: String,
        pub state: crate::schema::snapshot::StateTag,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct SuspensionValue {
        pub node_id: String,
        pub class: String,
        pub reference: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct StateValue {
        pub entries: Vec<crate::schema::snapshot::StateEntryValue>,
        pub suspensions: Vec<crate::schema::snapshot::SuspensionValue>,
    }
}

pub mod invocation {
    #[derive(
        Clone, Copy, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize,
    )]
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

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct SuspensionFactValue {
        pub class: String,
        pub reference: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EffectRequestValue {
        pub operation: String,
        pub resource: String,
        pub ability: String,
        pub request: String,
        pub payload: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct EventValue {
        pub node_id: String,
        pub action: crate::schema::invocation::ActionTag,
        pub effects: Vec<crate::schema::invocation::EffectRequestValue>,
        pub suspension: Option<crate::schema::invocation::SuspensionFactValue>,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ConcreteReferenceValue {
        pub profile: String,
        pub identity: String,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ConcreteBindingValue {
        pub slot_id: Option<String>,
        pub source_ref: Option<String>,
        pub reference: crate::schema::invocation::ConcreteReferenceValue,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ConcreteEffectRequestValue {
        pub declaration_key: String,
        pub request: crate::schema::invocation::ConcreteBindingValue,
        pub payload: crate::schema::invocation::ConcreteBindingValue,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ConcreteEventValue {
        pub node_id: String,
        pub action: crate::schema::invocation::ActionTag,
        pub effects: Vec<crate::schema::invocation::ConcreteEffectRequestValue>,
        pub suspension: Option<crate::schema::invocation::SuspensionFactValue>,
    }
}

pub mod history {
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ReplayBindingsValue {
        #[serde(with = "serde_bytes")]
        pub machine: Vec<u8>,
        #[serde(with = "serde_bytes")]
        pub initial_state: Vec<u8>,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ReplayBindingPairValue {
        pub expected: crate::schema::history::ReplayBindingsValue,
        pub trace: crate::schema::history::ReplayBindingsValue,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct ReplayFailureValue {
        pub event_index: Option<u64>,
        pub consumed: u64,
        pub reason: crate::schema::history::ReplayFailureReasonValue,
        pub last_valid_state: crate::schema::snapshot::StateValue,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum ReplayFailureReasonValue {
        TraceLimitExceeded {
            observed: u64,
            maximum: u64,
        },
        MachineBindingMismatch,
        InitialStateBindingMismatch,
        InitialStateInvalid,
        TransitionDenied {
            reason: crate::schema::decision::DenialValue,
        },
    }
}

pub mod decision {
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename = "choregraph-wasm-result-v1", deny_unknown_fields)]
    pub struct KernelResult {
        pub abi_version: u32,
        pub operation: Option<crate::schema::profile::OperationTag>,
        pub input_identity: String,
        pub outcome: crate::schema::decision::Outcome,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Outcome {
        Admitted {
            machine_identity: choregraph_kamacite::identity::ArtifactIdentity,
            initial_state: crate::schema::snapshot::StateValue,
        },
        Accepted {
            machine_identity: choregraph_kamacite::identity::ArtifactIdentity,
            next: crate::schema::snapshot::StateValue,
            intents: Vec<crate::schema::decision::IntentValue>,
        },
        Denied {
            machine_identity: choregraph_kamacite::identity::ArtifactIdentity,
            reason: crate::schema::decision::DenialValue,
        },
        ReplayComplete {
            machine_identity: choregraph_kamacite::identity::ArtifactIdentity,
            final_state: crate::schema::snapshot::StateValue,
            consumed: u64,
        },
        ReplayFailed {
            machine_identity: choregraph_kamacite::identity::ArtifactIdentity,
            failure: crate::schema::history::ReplayFailureValue,
        },
        AdapterError {
            error: crate::schema::diagnostic::AdapterErrorValue,
        },
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum IntentValue {
        Declared {
            node_id: String,
            transition: crate::schema::decision::TransitionValue,
            operation: String,
            resource: String,
            ability: String,
            request: String,
            payload: String,
        },
        Concrete {
            declaration_key: String,
            node_id: String,
            transition: crate::schema::decision::TransitionValue,
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

    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct TransitionValue {
        pub from: crate::schema::snapshot::StateTag,
        pub action: crate::schema::invocation::ActionTag,
    }

    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct DenialValue {
        pub code: crate::schema::decision::DenialCode,
        pub observed: Option<u64>,
        pub maximum: Option<u64>,
        pub from: Option<crate::schema::snapshot::StateTag>,
        pub action: Option<crate::schema::invocation::ActionTag>,
    }

    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
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
        InternalInconsistency,
    }
}

pub mod diagnostic {
    #[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    pub struct AdapterErrorValue {
        pub code: crate::schema::diagnostic::AdapterErrorCode,
        pub stage: crate::schema::diagnostic::AdapterStage,
    }

    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
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

    #[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
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
}

mod optional_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => serializer.serialize_some(serde_bytes::Bytes::new(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<serde_bytes::ByteBuf>::deserialize(deserializer)
            .map(|value| value.map(serde_bytes::ByteBuf::into_vec))
    }
}
