//! Pure bounded role-choreography admission and Trellis-backed endpoint projection.
//!
//! The crate calls the pinned Trellis production projection. It does not execute
//! roles, grant authority, transport values, persist sessions, or claim release
//! eligibility.
//!
//! r[impl choregraph.role_choreography.source]
//! r[impl choregraph.role_choreography.bounds]
//! r[impl choregraph.role_choreography.admission]
//! r[impl choregraph.role_choreography.roles]
//! r[impl choregraph.role_choreography.lowering]
//! r[impl choregraph.role_choreography.trellis]
//! r[impl choregraph.role_choreography.projection]
//! r[impl choregraph.role_choreography.complete_set]
//! r[impl choregraph.role_choreography.identity]
//! r[impl choregraph.role_choreography.artifacts]
//! r[impl choregraph.role_choreography.diagnostics]
//! r[impl choregraph.role_choreography.reference]
//! r[impl choregraph.role_choreography.validation]
//! r[impl choregraph.role_choreography.fixtures]
//! r[impl choregraph.role_choreography.boundary]

mod admission;
mod endpoint;
mod issue;
mod linkage;
mod model;

pub use admission::{AdmittedChoreography, admit_source, parse_evaluated_json};
pub use endpoint::{
    GlobalProtocol, GlobalProtocolArtifactBody, GlobalProtocolBranch, LocalProtocol,
    LocalProtocolArtifactBody, LocalProtocolBranch, LocalProtocolSetBody, ProjectedChoreography,
    project_all, validate_complete_set,
};
pub use issue::{
    ChoreographyDiagnostic, DiagnosticCode, DiagnosticSeverity, SafeReplacement,
    diagnostics_are_safe, normalize_diagnostics, render_safe_diagnostics, safe_diagnostic_text,
    span_is_valid,
};
pub use linkage::{
    CHOREGRAPH_TOOLCHAIN, CanonicalChoreographyArtifact, CanonicalLocalArtifact,
    ChoreographyIdentity, ChoreographyIdentityDomain, LOWERER_IDENTITY_PREIMAGE, ProjectionBundle,
    ProjectionEvidenceBinding, ProjectionEvidenceRole, ProjectionMaterials, ProjectionReceiptBody,
    TRELLIS_PROOF_SET_PREIMAGE, TRELLIS_REVISION, TRELLIS_SEMANTIC_COMMIT,
    TRELLIS_SOURCE_MANIFEST_BLAKE3, TRELLIS_TOOLCHAIN, build_projection_bundle, compute_identity,
    identity_is_valid, prefixed_identity, verify_projection_receipt,
};
pub use model::{
    ChoiceBranch, ChoreographyLimits, ChoreographyNode, ImportIdentity, LabelId, MAX_BRANCHES,
    MAX_CANONICAL_OUTPUT_BYTES, MAX_DIAGNOSTIC_TEXT_BYTES, MAX_DIAGNOSTICS, MAX_IDENTIFIER_BYTES,
    MAX_IMPORTS, MAX_NESTING_DEPTH, MAX_ROLES, MAX_STEPS, MAX_VALUES, PayloadTag,
    REQUIRED_NON_CLAIMS, ROLE_CHOREOGRAPHY_SCHEMA, RoleChoreographySource, RoleDeclaration, RoleId,
    RoleKind, SourceSpan, ValueDeclaration, ValueId,
};
pub const CLAIM_BOUNDARY: &str = concat!(
    "bounded source admission, exact pinned Trellis projection, canonical identity linkage; ",
    "not runtime execution, host placement, transport delivery, persistence, authorization, ",
    "external-role correctness, exactly-once transfer, fault deadlock freedom, production readiness, ",
    "or release eligibility",
);

pub fn compile_evaluated(
    evaluated_bytes: &[u8],
    source_envelope_bytes: &[u8],
) -> Result<ProjectionBundle, Vec<ChoreographyDiagnostic>> {
    let source = parse_evaluated_json(evaluated_bytes)?;
    let admitted = admit_source(source)?;
    let materials = ProjectionMaterials::reviewed(source_envelope_bytes, evaluated_bytes);
    build_projection_bundle(&admitted, &materials)
}

#[cfg(test)]
mod tests;
