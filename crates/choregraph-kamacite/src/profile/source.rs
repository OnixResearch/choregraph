pub const SCHEMA: &str = "choregraph.role-choreography-source-preserves.v1";
pub const ROOT: &str = "RoleChoreographySource";
pub fn encode<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(value, SCHEMA, ROOT)
}
