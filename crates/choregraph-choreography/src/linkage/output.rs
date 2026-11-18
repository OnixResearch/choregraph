#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalArtifact {
    pub schema_version: String,
    pub packed_bytes: Vec<u8>,
    pub text_projection: String,
    pub identity: super::digest::Identity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalLocalArtifact {
    pub role: crate::model::participants::RoleId,
    pub artifact: CanonicalArtifact,
}
