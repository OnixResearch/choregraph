//! Pure canonical artifact and receipt adapter for the selected Kamacite profile.
//!
//! The crate owns no filesystem, process, network, clock, policy, or runtime-effect authority.

mod aggregate;
mod codec;
mod concrete;
mod identity;
mod lifecycle;
mod profile;
mod receipt;

#[cfg(test)]
#[path = "../test-support/internal.rs"]
mod tests;

// The complete reviewed codec stays source-identical for reproducible canonical compatibility.
#[allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    dead_code,
    explicit_defaults,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    renamed_imports,
    unbounded_collection_growth,
    reason = "the complete reviewed codec stays source-identical for reproducible canonical compatibility"
)]
#[path = "../../../vendor/kamacite-core-selected/preserves/codec.rs"]
mod selected_kamacite_codec;

pub use aggregate::{
    AggregateError, AggregateMachineDescriptor, ArtifactSet, CompatibilityMetadata,
    DescriptorDependencySet, DescriptorEdge, DescriptorEffect, DescriptorLimits, DescriptorNode,
    DescriptorSuspension, DurableMachineDescriptor, GLOBAL_STATE_CARDINALITY, GLOBAL_STATE_MODEL,
    IdentityMaterials, KAMACITE_REVISION, SupportedKamaciteProfile,
    ValidatedDurableMachineArtifact, ValidatedMachineArtifact, build_artifact_set,
    build_profile_artifact, durable_machine_descriptor, encode_durable_machine_descriptor,
    encode_machine_descriptor, validate_durable_machine_artifact, validate_machine_artifact,
};
pub use codec::{
    CONCRETE_INTENT_RECEIPT_ROOT_LABEL, CONCRETE_INTENT_RECEIPT_SCHEMA_VERSION,
    CONCRETE_INTENT_ROOT_LABEL, CONCRETE_INTENT_SCHEMA_VERSION, CONCRETE_MACHINE_ROOT_LABEL,
    CONCRETE_MACHINE_SCHEMA_VERSION, CanonicalArtifact, CodecBoundaryError, FSM_RECEIPT_ROOT_LABEL,
    FSM_RECEIPT_SCHEMA_VERSION, FSM_SPEC_ROOT_LABEL, FSM_SPEC_SCHEMA_VERSION, MACHINE_ROOT_LABEL,
    MACHINE_SCHEMA_VERSION,
};
pub use concrete::{
    ConcreteArtifactError, ConcreteIntentArtifact, ConcreteIntentDescriptorV2,
    ConcreteIntentReceiptArtifact, ConcreteIntentReceiptBody, ConcreteMachineArtifact,
    ConcreteMachineDescriptorV2, DescriptorConcreteEffectV2, DescriptorReferenceBindingV2,
    DescriptorSelectedReferenceV2, build_concrete_intent_artifact, build_concrete_intent_receipt,
    build_concrete_machine_artifact, encode_concrete_intent_descriptor,
    encode_concrete_intent_receipt_body, encode_concrete_machine_descriptor,
    validate_concrete_intent_artifact, validate_concrete_intent_receipt,
    validate_concrete_machine_artifact,
};
pub use identity::{
    ArtifactDomain, ArtifactIdentity, BLAKE3_ALGORITHM, BLAKE3_HEX_LENGTH, IdentityChain,
    IdentityError, KamaciteIdentity, KamaciteIdentityDomain, compute_identity,
    compute_kamacite_identity,
};
pub use lifecycle::{
    DURABLE_FSM_ID, FsmCoverageEvidence, FsmDenialRow, FsmEvent, FsmGuard, FsmMatrixCell,
    FsmSideEffect, FsmState, FsmTransitionRow, LIFECYCLE_FSM_ID, LifecycleArtifact,
    LifecycleDescriptor, LifecycleError, build_durable_lifecycle_artifact,
    build_lifecycle_artifact, transition_identity, validate_durable_lifecycle_artifact,
    validate_lifecycle_artifact,
};
pub use profile::{
    ROLE_CHOREOGRAPHY_DIAGNOSTICS_ROOT, ROLE_CHOREOGRAPHY_DIAGNOSTICS_SCHEMA,
    ROLE_CHOREOGRAPHY_GLOBAL_ROOT, ROLE_CHOREOGRAPHY_GLOBAL_SCHEMA, ROLE_CHOREOGRAPHY_LOCAL_ROOT,
    ROLE_CHOREOGRAPHY_LOCAL_SCHEMA, ROLE_CHOREOGRAPHY_LOCAL_SET_ROOT,
    ROLE_CHOREOGRAPHY_LOCAL_SET_SCHEMA, ROLE_CHOREOGRAPHY_RECEIPT_ROOT,
    ROLE_CHOREOGRAPHY_RECEIPT_SCHEMA, ROLE_CHOREOGRAPHY_SOURCE_ROOT,
    ROLE_CHOREOGRAPHY_SOURCE_SCHEMA, encode_role_choreography_diagnostics,
    encode_role_choreography_global, encode_role_choreography_local,
    encode_role_choreography_local_set, encode_role_choreography_receipt,
    encode_role_choreography_source,
};
pub use receipt::{
    FsmGuardDecision, FsmReceiptArtifact, FsmReceiptBody, FsmTraceStep, LinkReceiptArtifact,
    LinkReceiptBody, LinkReceiptRole, ReceiptBundle, ReceiptError, TraceObservation,
    build_receipt_bundle, encode_fsm_receipt_body, encode_link_receipt_body, validate_fsm_receipt,
    validate_link_receipt, validate_receipt_bundle,
};

pub const CLAIM_BOUNDARY: &str = "canonical encoding and exact identity linkage; not behavioral proof, authority, or effect success";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterchangeClaim {
    CanonicalIdentityBinding,
    BehavioralProof,
    RuntimeAuthority,
}

#[must_use]
pub const fn supports(claim: InterchangeClaim) -> bool {
    matches!(claim, InterchangeClaim::CanonicalIdentityBinding)
}
