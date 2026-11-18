//! Exact compilation association, separate from semantic proof and runtime authority.
use serde::{Deserialize, Serialize};

const BINDING_VERSION: &str = "choregraph.construction-binding.v1";
const HASH_CONTEXT: &str = "choregraph construction association v1";
#[derive(Clone, Copy, Debug)]
pub struct CapturedModule<'a> {
    pub logical_id: &'a str,
    pub bytes: &'a [u8],
}
/// Evaluation evidence is provided by the host only after native nickel-eval response validation.
/// Binding those exact bytes is not authentication of the host or proof of Nickel evaluation.
#[derive(Clone, Copy, Debug)]
pub struct CompilationMaterials<'a> {
    pub modules: &'a [CapturedModule<'a>],
    pub entry: &'a str,
    pub evaluator_identity: &'a str,
    pub evaluator_profile: &'a str,
    pub evaluator_binding: &'a [u8],
    pub evaluated_output_blake3: &'a str,
    pub library: &'a [u8],
    pub contracts: &'a [u8],
    pub lowerer_identity: &'a str,
    pub toolchain_identity: &'a str,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilationBinding {
    pub version: String,
    pub domains: std::collections::BTreeMap<String, String>,
    pub association: String,
}
fn hash(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(HASH_CONTEXT);
    hasher.update(&(domain.len() as u64).to_le_bytes());
    hasher.update(domain.as_bytes());
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

/// nickel-eval's exact submitted-map framing. This is identity, not evaluation evidence.
pub fn captured_map_blake3(modules: &[CapturedModule<'_>]) -> String {
    let sorted = modules
        .iter()
        .map(|module| (module.logical_id, module.bytes))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"nickel-eval.captured-map.v1\0");
    hasher.update(&(sorted.len() as u64).to_le_bytes());
    for (name, bytes) in sorted {
        for part in [name.as_bytes(), b"nickel-utf8".as_slice(), bytes] {
            hasher.update(&(part.len() as u64).to_le_bytes());
            hasher.update(part);
        }
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) struct Inputs<'a, 'm> {
    pub candidate: &'a [u8],
    pub parameters: &'a std::collections::BTreeMap<String, serde_json::Value>,
    pub registry: &'a crate::construction::model::host::TargetRegistry,
    pub materials: &'a CompilationMaterials<'m>,
    pub limits: crate::construction::model::profile::Limits,
}

pub(super) fn build(
    inputs: Inputs<'_, '_>,
    output: &crate::construction::admission::ProjectionOutput,
    origins: &[crate::construction::model::projection::Origin],
) -> Result<CompilationBinding, crate::construction::Failure> {
    let materials = inputs.materials;
    let modules = materials
        .modules
        .iter()
        .map(|module| (module.logical_id, hash("module-bytes", module.bytes)))
        .collect::<std::collections::BTreeMap<_, _>>();
    debug_assert_eq!(
        modules.len(),
        materials.modules.len(),
        "validated modules hash one-to-one"
    );
    let source = materials
        .modules
        .iter()
        .find(|module| module.logical_id == materials.entry)
        .ok_or(crate::construction::Failure::InvalidMaterials)?;
    let mut domains = material_domains(&inputs, source, output);
    domains.insert(
        "submitted-modules".to_owned(),
        serialized(
            "submitted-modules",
            &modules,
            inputs.limits.max_output_bytes,
        )?,
    );
    domains.insert(
        "parameters".to_owned(),
        serialized(
            "parameters",
            inputs.parameters,
            inputs.limits.max_parameter_bytes,
        )?,
    );
    domains.insert(
        "registry".to_owned(),
        serialized(
            "registry",
            inputs.registry,
            inputs.limits.max_candidate_bytes,
        )?,
    );
    domains.extend(
        graph_domains(&inputs, output, origins)?
            .into_iter()
            .map(|(name, digest)| (name.to_owned(), digest)),
    );
    debug_assert_eq!(
        domains.len(),
        ASSOCIATION_DOMAINS,
        "all distinct association domains are present"
    );
    let association = serialized("association", &domains, inputs.limits.max_output_bytes)?;
    Ok(CompilationBinding {
        version: BINDING_VERSION.to_owned(),
        domains,
        association,
    })
}

const ASSOCIATION_DOMAINS: usize = 19;
fn material_domains(
    inputs: &Inputs<'_, '_>,
    source: &CapturedModule<'_>,
    output: &crate::construction::admission::ProjectionOutput,
) -> std::collections::BTreeMap<String, String> {
    [
        ("source", source.bytes),
        ("entry", inputs.materials.entry.as_bytes()),
        ("evaluator", inputs.materials.evaluator_identity.as_bytes()),
        (
            "evaluator-profile",
            inputs.materials.evaluator_profile.as_bytes(),
        ),
        (
            "observed-evaluation-binding",
            inputs.materials.evaluator_binding,
        ),
        ("library", inputs.materials.library),
        ("contracts", inputs.materials.contracts),
        ("candidate", inputs.candidate),
        (
            "elaborator",
            crate::construction::model::profile::ELABORATOR_PROFILE.as_bytes(),
        ),
        ("lowerer", inputs.materials.lowerer_identity.as_bytes()),
        ("toolchain", inputs.materials.toolchain_identity.as_bytes()),
        ("artifact", output.artifact.as_slice()),
    ]
    .into_iter()
    .map(|(name, material)| (name.to_owned(), hash(name, material)))
    .collect()
}

fn serialized<T: Serialize + ?Sized>(
    domain: &str,
    value: &T,
    ceiling: usize,
) -> Result<String, crate::construction::Failure> {
    Ok(hash(
        domain,
        &crate::construction::bounded_json(value, ceiling)?,
    ))
}

fn graph_domains(
    inputs: &Inputs<'_, '_>,
    output: &crate::construction::admission::ProjectionOutput,
    origins: &[crate::construction::model::projection::Origin],
) -> Result<[(&'static str, String); 4], crate::construction::Failure> {
    Ok([
        (
            "profile",
            serialized(
                "profile",
                &(
                    crate::construction::model::profile::CONSTRUCTION_SCHEMA,
                    crate::construction::model::profile::PROJECTION_SCHEMA,
                    &inputs.registry.target,
                    &inputs.registry.profile,
                    inputs.limits,
                ),
                inputs.limits.max_candidate_bytes,
            )?,
        ),
        (
            "expanded-graph",
            serialized(
                "expanded-graph",
                &(
                    &output.projection.nodes,
                    &output.projection.edges,
                    &output.projection.control_edges,
                    &output.projection.effects,
                    &output.projection.interfaces,
                ),
                inputs.limits.max_output_bytes,
            )?,
        ),
        (
            "projection",
            serialized(
                "projection",
                &output.projection,
                inputs.limits.max_output_bytes,
            )?,
        ),
        (
            "diagnostic-origins",
            serialized(
                "diagnostic-origins",
                origins,
                inputs.limits.max_origin_bytes,
            )?,
        ),
    ])
}

/// Rebuild using current exact materials and native admission, then compare every returned field.
/// A matching author-provided digest is never sufficient for executable admission.
pub fn validate_compilation_binding(
    output: &crate::construction::Output,
    candidate: &[u8],
    registry: &crate::construction::model::host::TargetRegistry,
    materials: &CompilationMaterials<'_>,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let expected = crate::construction::compile_json(candidate, registry, materials, limits)?;
    if &expected != output {
        return Err(crate::construction::Failure::StaleBinding);
    }
    Ok(())
}
