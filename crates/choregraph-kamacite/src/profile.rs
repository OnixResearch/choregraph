pub const ROLE_CHOREOGRAPHY_SOURCE_SCHEMA: &str =
    "choregraph.role-choreography-source-preserves.v1";
pub const ROLE_CHOREOGRAPHY_SOURCE_ROOT: &str = "RoleChoreographySource";
pub const ROLE_CHOREOGRAPHY_GLOBAL_SCHEMA: &str =
    "choregraph.role-choreography-global-preserves.v1";
pub const ROLE_CHOREOGRAPHY_GLOBAL_ROOT: &str = "GlobalProtocolArtifactBody";
pub const ROLE_CHOREOGRAPHY_LOCAL_SCHEMA: &str = "choregraph.role-choreography-local-preserves.v1";
pub const ROLE_CHOREOGRAPHY_LOCAL_ROOT: &str = "LocalProtocolArtifactBody";
pub const ROLE_CHOREOGRAPHY_LOCAL_SET_SCHEMA: &str =
    "choregraph.role-choreography-local-set-preserves.v1";
pub const ROLE_CHOREOGRAPHY_LOCAL_SET_ROOT: &str = "LocalProtocolSetBody";
pub const ROLE_CHOREOGRAPHY_DIAGNOSTICS_SCHEMA: &str =
    "choregraph.role-choreography-diagnostics-preserves.v1";
pub const ROLE_CHOREOGRAPHY_DIAGNOSTICS_ROOT: &str = "DiagnosticArtifactBody";
pub const ROLE_CHOREOGRAPHY_RECEIPT_SCHEMA: &str =
    "choregraph.role-choreography-projection-receipt-preserves.v1";
pub const ROLE_CHOREOGRAPHY_RECEIPT_ROOT: &str = "ProjectionReceiptBody";

pub fn encode_role_choreography_source<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(
        value,
        ROLE_CHOREOGRAPHY_SOURCE_SCHEMA,
        ROLE_CHOREOGRAPHY_SOURCE_ROOT,
    )
}

pub fn encode_role_choreography_global<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(
        value,
        ROLE_CHOREOGRAPHY_GLOBAL_SCHEMA,
        ROLE_CHOREOGRAPHY_GLOBAL_ROOT,
    )
}

pub fn encode_role_choreography_local<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(
        value,
        ROLE_CHOREOGRAPHY_LOCAL_SCHEMA,
        ROLE_CHOREOGRAPHY_LOCAL_ROOT,
    )
}

pub fn encode_role_choreography_local_set<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(
        value,
        ROLE_CHOREOGRAPHY_LOCAL_SET_SCHEMA,
        ROLE_CHOREOGRAPHY_LOCAL_SET_ROOT,
    )
}

pub fn encode_role_choreography_diagnostics<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(
        value,
        ROLE_CHOREOGRAPHY_DIAGNOSTICS_SCHEMA,
        ROLE_CHOREOGRAPHY_DIAGNOSTICS_ROOT,
    )
}

pub fn encode_role_choreography_receipt<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(
        value,
        ROLE_CHOREOGRAPHY_RECEIPT_SCHEMA,
        ROLE_CHOREOGRAPHY_RECEIPT_ROOT,
    )
}
