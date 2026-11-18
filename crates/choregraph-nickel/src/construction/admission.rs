//! Explicit v3 control/data validation and readmission for materialized Lattice graphs.
mod configuration;
mod effects;
mod links;
mod nodes;
mod preflight;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionOutput {
    pub machine: choregraph_core::model::automaton::AdmittedMachine,
    pub projection: crate::construction::model::projection::LatticeV3,
    pub artifact: Vec<u8>,
}
type PortIndex<'a> = std::collections::BTreeMap<
    u64,
    (
        u64,
        &'a crate::construction::model::projection::ExpandedPort,
    ),
>;
struct Indexes<'a> {
    nodes:
        std::collections::BTreeMap<u64, &'a crate::construction::model::projection::ExpandedNode>,
    ports: PortIndex<'a>,
}
fn count_use(
    uses: &mut std::collections::BTreeMap<u64, usize>,
    port: u64,
) -> Result<(), crate::construction::Failure> {
    let count = uses.entry(port).or_insert(0);
    *count = count
        .checked_add(1)
        .ok_or(crate::construction::Failure::LimitExceeded)?;
    Ok(())
}

pub(super) fn admit_projection(
    projection: crate::construction::model::projection::LatticeV3,
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<ProjectionOutput, crate::construction::Failure> {
    crate::construction::positive_limits(limits)?;
    header(&projection, registry, limits)?;
    artifact_bound(&projection, limits)?;
    let selected = super::registry::validate(registry, limits)?;
    let mut uses = std::collections::BTreeMap::new();
    let indexes =
        crate::construction::admission::nodes::validate(&projection, &selected, limits, &mut uses)?;
    let links =
        crate::construction::admission::links::validate(&projection, &indexes, &mut uses, limits)?;
    crate::construction::admission::links::interfaces(
        &projection,
        &selected,
        &indexes.ports,
        limits,
    )?;
    let mut spec = specification(&projection, &links);
    let rules = crate::construction::admission::effects::lower(
        &projection,
        &indexes,
        &mut uses,
        &mut spec,
        limits,
    )?;
    crate::construction::admission::effects::check_usage(
        &projection,
        &selected,
        rules,
        &links,
        &uses,
    )?;
    debug_assert_eq!(
        spec.nodes.len(),
        projection.nodes.len(),
        "lowering retains every numeric node"
    );
    debug_assert_eq!(
        spec.effects.len(),
        projection.effects.len(),
        "lowering retains every declaration"
    );
    let machine =
        choregraph_core::model::automaton::AdmittedMachine::admit_v2(spec, core_limits(limits))
            .map_err(|_| crate::construction::Failure::AdmissionRejected)?;
    let artifact = artifact(&machine, limits)?;
    Ok(ProjectionOutput {
        machine,
        projection,
        artifact,
    })
}
fn artifact_bound(
    projection: &crate::construction::model::projection::LatticeV3,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    let bytes = crate::construction::bounded_json(projection, limits.max_output_bytes)?;
    // Canonical encoding is not streaming. Bound its conservative tagged-row expansion first.
    const ARTIFACT_EXPANSION_FACTOR: usize = 4;
    const ARTIFACT_ENVELOPE_RESERVE: usize = 4096;
    let reserved = bytes
        .len()
        .checked_mul(ARTIFACT_EXPANSION_FACTOR)
        .and_then(|size| size.checked_add(ARTIFACT_ENVELOPE_RESERVE))
        .ok_or(crate::construction::Failure::LimitExceeded)?;
    if reserved > limits.max_output_bytes {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    Ok(())
}
fn header(
    projection: &crate::construction::model::projection::LatticeV3,
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    if projection.schema != crate::construction::model::profile::PROJECTION_SCHEMA {
        return Err(crate::construction::Failure::InvalidSchema);
    }
    if projection.target != registry.target || projection.registry != registry.profile {
        return Err(crate::construction::Failure::InvalidRegistry);
    }
    graph_counts(projection, limits)
}
fn graph_counts(
    projection: &crate::construction::model::projection::LatticeV3,
    limits: crate::construction::model::profile::Limits,
) -> Result<(), crate::construction::Failure> {
    if projection.nodes.len() > limits.max_nodes {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    let edges = projection
        .edges
        .len()
        .checked_add(projection.control_edges.len())
        .ok_or(crate::construction::Failure::LimitExceeded)?;
    if edges > limits.max_edges || projection.effects.len() > limits.max_effects {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    Ok(())
}
fn specification(
    projection: &crate::construction::model::projection::LatticeV3,
    links: &crate::construction::admission::links::LinkIndex,
) -> choregraph_core::concrete::machine::MachineSpecV2 {
    choregraph_core::concrete::machine::MachineSpecV2::new(
        projection
            .nodes
            .iter()
            .map(|node| {
                choregraph_core::model::graph::NodeSpec::new(
                    format!("lattice-node/{}", node.id),
                    vec![format!("lattice/{}", node.operation)],
                )
            })
            .collect(),
        links
            .dependencies
            .iter()
            .map(|(source, target)| {
                choregraph_core::model::graph::EdgeSpec::new(
                    format!("lattice-node/{source}"),
                    format!("lattice-node/{target}"),
                )
            })
            .collect(),
    )
}
const fn core_limits(
    limits: crate::construction::model::profile::Limits,
) -> choregraph_core::model::capacity::AdmissionLimits {
    choregraph_core::model::capacity::AdmissionLimits {
        max_nodes: if limits.max_nodes < choregraph_core::model::capacity::DEFAULT_MAX_NODES {
            limits.max_nodes
        } else {
            choregraph_core::model::capacity::DEFAULT_MAX_NODES
        },
        max_edges: if limits.max_edges < choregraph_core::model::capacity::DEFAULT_MAX_EDGES {
            limits.max_edges
        } else {
            choregraph_core::model::capacity::DEFAULT_MAX_EDGES
        },
        max_operations: if limits.max_nodes
            < choregraph_core::model::capacity::DEFAULT_MAX_OPERATIONS
        {
            limits.max_nodes
        } else {
            choregraph_core::model::capacity::DEFAULT_MAX_OPERATIONS
        },
        max_effect_declarations: if limits.max_effects
            < choregraph_core::model::capacity::DEFAULT_MAX_EFFECT_DECLARATIONS
        {
            limits.max_effects
        } else {
            choregraph_core::model::capacity::DEFAULT_MAX_EFFECT_DECLARATIONS
        },
        max_identifier_bytes: if limits.max_identifier_bytes
            < choregraph_core::model::capacity::DEFAULT_MAX_IDENTIFIER_BYTES
        {
            limits.max_identifier_bytes
        } else {
            choregraph_core::model::capacity::DEFAULT_MAX_IDENTIFIER_BYTES
        },
        max_reference_bytes: choregraph_core::model::capacity::DEFAULT_MAX_REFERENCE_BYTES,
        max_event_bytes: choregraph_core::model::capacity::DEFAULT_MAX_EVENT_BYTES,
        max_intent_bytes: choregraph_core::model::capacity::DEFAULT_MAX_INTENT_BYTES,
        max_intents_per_transition:
            choregraph_core::model::capacity::DEFAULT_MAX_INTENTS_PER_TRANSITION,
    }
}
fn artifact(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
    limits: crate::construction::model::profile::Limits,
) -> Result<Vec<u8>, crate::construction::Failure> {
    let artifact =
        choregraph_kamacite::concrete::conversion::build_concrete_machine_artifact(machine)
            .map_err(|_| crate::construction::Failure::AdmissionRejected)?
            .canonical
            .packed_bytes;
    if artifact.len() > limits.max_output_bytes {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    Ok(artifact)
}

/// Revalidate a persisted/materialized v3 graph against the CURRENT host registry.
/// No evaluator receipt, author origin or previous construction admission is authority.
pub fn lower_projection_json(
    projection: &[u8],
    registry: &crate::construction::model::host::TargetRegistry,
    limits: crate::construction::model::profile::Limits,
) -> Result<ProjectionOutput, crate::construction::Failure> {
    crate::construction::positive_limits(limits)?;
    if projection.len() > limits.max_output_bytes {
        return Err(crate::construction::Failure::LimitExceeded);
    }
    crate::construction::admission::preflight::scan_projection(projection, limits)?;
    let parsed = serde_json::from_slice(projection)
        .map_err(|_| crate::construction::Failure::InvalidJson)?;
    admit_projection(parsed, registry, limits)
}
