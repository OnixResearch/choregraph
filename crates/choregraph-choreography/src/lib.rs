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

pub mod admission;
pub mod endpoint;
pub mod issue;
pub mod linkage;
pub mod model;

pub const CLAIM_BOUNDARY: &str = concat!(
    "bounded source admission, exact pinned Trellis projection, canonical identity linkage; ",
    "not runtime execution, host placement, transport delivery, persistence, authorization, ",
    "external-role correctness, exactly-once transfer, fault deadlock freedom, production readiness, ",
    "or release eligibility",
);

pub fn compile_evaluated(
    evaluated_bytes: &[u8],
    source_envelope_bytes: &[u8],
) -> Result<linkage::ProjectionBundle, admission::Error> {
    let source = admission::parse_evaluated_json(evaluated_bytes)?;
    let admitted = admission::admit_source(source)?;
    let materials = linkage::ProjectionMaterials::reviewed(source_envelope_bytes, evaluated_bytes);
    linkage::build_projection_bundle(&admitted, &materials).map_err(Into::into)
}

#[cfg(test)]
mod tests;
