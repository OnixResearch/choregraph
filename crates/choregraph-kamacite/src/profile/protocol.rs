pub const GLOBAL_SCHEMA: &str = "choregraph.role-choreography-global-preserves.v1";
pub const GLOBAL_ROOT: &str = "GlobalProtocolArtifactBody";
pub const LOCAL_SCHEMA: &str = "choregraph.role-choreography-local-preserves.v1";
pub const LOCAL_ROOT: &str = "LocalProtocolArtifactBody";
pub const LOCAL_SET_SCHEMA: &str = "choregraph.role-choreography-local-set-preserves.v1";
pub const LOCAL_SET_ROOT: &str = "LocalProtocolSetBody";
pub fn encode_global<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(value, GLOBAL_SCHEMA, GLOBAL_ROOT)
}

pub fn encode_local<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(value, LOCAL_SCHEMA, LOCAL_ROOT)
}

pub fn encode_local_set<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(value, LOCAL_SET_SCHEMA, LOCAL_SET_ROOT)
}
