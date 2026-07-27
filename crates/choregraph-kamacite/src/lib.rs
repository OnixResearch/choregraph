//! Pure canonical artifact and receipt adapter for the selected Kamacite profile.
//!
//! The crate owns no filesystem, process, network, clock, policy, or runtime-effect authority.

mod aggregate;
mod codec;
mod identity;
mod lifecycle;
mod receipt;

#[cfg(test)]
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
    GLOBAL_STATE_CARDINALITY, GLOBAL_STATE_MODEL, IdentityMaterials, KAMACITE_REVISION,
    SupportedKamaciteProfile, ValidatedMachineArtifact, build_artifact_set, build_profile_artifact,
    encode_machine_descriptor, validate_machine_artifact,
};
pub use codec::{
    CanonicalArtifact, CodecBoundaryError, FSM_RECEIPT_ROOT_LABEL, FSM_RECEIPT_SCHEMA_VERSION,
    FSM_SPEC_ROOT_LABEL, FSM_SPEC_SCHEMA_VERSION, MACHINE_ROOT_LABEL, MACHINE_SCHEMA_VERSION,
};
pub use identity::{
    ArtifactDomain, ArtifactIdentity, BLAKE3_ALGORITHM, BLAKE3_HEX_LENGTH, IdentityChain,
    IdentityError, KamaciteIdentity, KamaciteIdentityDomain, compute_identity,
    compute_kamacite_identity,
};
pub use lifecycle::{
    FsmCoverageEvidence, FsmDenialRow, FsmEvent, FsmGuard, FsmMatrixCell, FsmSideEffect, FsmState,
    FsmTransitionRow, LIFECYCLE_FSM_ID, LifecycleArtifact, LifecycleDescriptor, LifecycleError,
    build_lifecycle_artifact, transition_identity, validate_lifecycle_artifact,
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
