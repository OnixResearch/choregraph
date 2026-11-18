pub const DIAGNOSTICS_SCHEMA: &str = "choregraph.role-choreography-diagnostics-preserves.v1";
pub const DIAGNOSTICS_ROOT: &str = "DiagnosticArtifactBody";
pub const RECEIPT_SCHEMA: &str = "choregraph.role-choreography-projection-receipt-preserves.v1";
pub const RECEIPT_ROOT: &str = "ProjectionReceiptBody";
pub fn encode_diagnostics<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(value, DIAGNOSTICS_SCHEMA, DIAGNOSTICS_ROOT)
}

pub fn encode_receipt<T: serde::Serialize>(
    value: &T,
) -> Result<crate::codec::CanonicalArtifact, crate::codec::CodecBoundaryError> {
    crate::codec::encode(value, RECEIPT_SCHEMA, RECEIPT_ROOT)
}
