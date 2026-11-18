//! Versioned finite graph construction over a host-selected registry.
//!
//! This module has no evaluator or runtime. Captured evaluation belongs to nickel-eval;
//! successful elaboration always ends in the existing machine admission.
pub mod admission;
pub mod association;
mod expand;
pub mod model;
mod observation;
mod registry;

use serde::Serialize;
use std::io::Write;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    InvalidJson,
    InvalidSchema,
    InvalidRegistry,
    InvalidIdentifier,
    LimitExceeded,
    UnknownOperation,
    InvalidConfiguration,
    InvalidReference,
    InvalidEffect,
    DuplicateKey,
    UnknownTemplate,
    InvalidParameters,
    InvalidHandle,
    WrongDirection,
    UnknownBoundary,
    IncompatibleType,
    OpenBoundary,
    UsageViolation,
    AdmissionRejected,
    InvalidMaterials,
    InvalidLocation,
    StaleBinding,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for Failure {}

#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub machine: choregraph_core::model::automaton::AdmittedMachine,
    pub projection: crate::construction::model::projection::LatticeV3,
    pub origins: Vec<crate::construction::model::projection::Origin>,
    pub binding: crate::construction::association::CompilationBinding,
    pub artifact: Vec<u8>,
}

pub(super) fn identifier(
    value: &str,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), Failure> {
    if value.is_empty()
        || value.len() > limits.max_identifier_bytes
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.:/".contains(&byte))
    {
        return Err(Failure::InvalidIdentifier);
    }
    Ok(())
}

struct BoundedWriter {
    bytes: Vec<u8>,
    ceiling: usize,
}
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.ceiling.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("bounded serialization"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn bounded_json<T: Serialize + ?Sized>(
    value: &T,
    ceiling: usize,
) -> Result<Vec<u8>, Failure> {
    let mut writer = BoundedWriter {
        bytes: Vec::new(),
        ceiling,
    };
    serde_json::to_writer(&mut writer, value).map_err(|_| Failure::LimitExceeded)?;
    Ok(writer.bytes)
}
pub(super) const MAX_DEPTH: usize = 64;
const fn limit_fields(limits: crate::construction::model::profile::Limits) -> [usize; 12] {
    [
        limits.max_candidate_bytes,
        limits.max_nodes,
        limits.max_ports,
        limits.max_edges,
        limits.max_effects,
        limits.max_instances,
        limits.max_depth,
        limits.max_boundaries,
        limits.max_identifier_bytes,
        limits.max_origin_bytes,
        limits.max_parameter_bytes,
        limits.max_output_bytes,
    ]
}
const fn positive_limits(
    limits: crate::construction::model::profile::Limits,
) -> Result<(), Failure> {
    let fields = limit_fields(limits);
    let mut remaining = fields.as_slice();
    while let Some((value, rest)) = remaining.split_first() {
        if *value == 0 {
            return Err(Failure::LimitExceeded);
        }
        remaining = rest;
    }
    if limits.max_depth > MAX_DEPTH {
        return Err(Failure::LimitExceeded);
    }
    Ok(())
}

/// Compile exact evaluated candidate bytes. No partially admitted result escapes on failure.
/// `registry` and evaluation materials are selected and authenticated by the host, not Nickel.
// r[impl choregraph.construction.captured]
// r[impl choregraph.construction.composition]
// r[impl choregraph.construction.registry]
// r[impl choregraph.construction.bounds]
// r[impl choregraph.construction.binding]
pub fn compile_json(
    candidate: &[u8],
    registry: &crate::construction::model::host::TargetRegistry,
    materials: &crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<Output, Failure> {
    let parsed = parse_inputs(candidate, registry, materials, limits)?;
    let (projected, origins) = elaborate(&parsed, registry, materials, limits)?;
    let inputs = crate::construction::association::Inputs {
        candidate,
        parameters: &parsed.parameters,
        registry,
        materials,
        limits,
    };
    complete_output(inputs, projected, origins)
}

fn parse_inputs(
    candidate: &[u8],
    registry: &crate::construction::model::host::TargetRegistry,
    materials: &crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<crate::construction::model::program::Candidate, Failure> {
    positive_limits(limits)?;
    if candidate.len() > limits.max_candidate_bytes {
        return Err(Failure::LimitExceeded);
    }
    crate::construction::observation::validate_materials(candidate, materials, limits)?;
    parse_candidate(candidate, registry, limits)
}

fn elaborate(
    parsed: &crate::construction::model::program::Candidate,
    registry: &crate::construction::model::host::TargetRegistry,
    materials: &crate::construction::association::CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<
    (
        crate::construction::admission::ProjectionOutput,
        Vec<crate::construction::model::projection::Origin>,
    ),
    Failure,
> {
    let selected = crate::construction::registry::validate(registry, limits)?;
    let (projection, origins) =
        crate::construction::expand::Expansion::new(parsed, selected, materials.modules, limits)?
            .finish()?;
    let projected = crate::construction::admission::admit_projection(projection, registry, limits)?;
    Ok((projected, origins))
}

fn complete_output(
    inputs: crate::construction::association::Inputs<'_, '_>,
    projected: crate::construction::admission::ProjectionOutput,
    origins: Vec<crate::construction::model::projection::Origin>,
) -> Result<Output, Failure> {
    let limits = inputs.limits;
    let binding = crate::construction::association::build(inputs, &projected, &origins)?;
    output_bound(&projected, &origins, &binding, limits)?;
    Ok(Output {
        machine: projected.machine,
        projection: projected.projection,
        origins,
        binding,
        artifact: projected.artifact,
    })
}

fn parse_candidate(
    candidate: &[u8],
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<crate::construction::model::program::Candidate, Failure> {
    let parsed: crate::construction::model::program::Candidate =
        serde_json::from_slice(candidate).map_err(|_| Failure::InvalidJson)?;
    if parsed.schema != crate::construction::model::profile::CONSTRUCTION_SCHEMA {
        return Err(Failure::InvalidSchema);
    }
    identifier(&parsed.root, limits)?;
    if parsed.root.contains('/') {
        return Err(Failure::InvalidIdentifier);
    }
    for name in parsed.parameters.keys() {
        identifier(name, limits)?;
    }
    if parsed.target != registry.target || parsed.registry != registry.profile {
        return Err(Failure::InvalidRegistry);
    }
    Ok(parsed)
}

fn output_bound(
    projected: &crate::construction::admission::ProjectionOutput,
    origins: &[crate::construction::model::projection::Origin],
    binding: &crate::construction::association::CompilationBinding,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), Failure> {
    // All returned serialized material has an aggregate output bound, not just per-field bounds.
    let mut bytes = bounded_json(&projected.projection, limits.max_output_bytes)?.len();
    for size in [
        bounded_json(&origins, limits.max_origin_bytes)?.len(),
        bounded_json(&binding, limits.max_output_bytes)?.len(),
        projected.artifact.len(),
    ] {
        bytes = bytes.checked_add(size).ok_or(Failure::LimitExceeded)?;
        if bytes > limits.max_output_bytes {
            return Err(Failure::LimitExceeded);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
