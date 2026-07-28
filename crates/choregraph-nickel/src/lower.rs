// Lowering preserves explicit closed-schema matches and established diagnostic paths.
#![allow(
    unknown_lints,
    assertion_density,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "lowering preserves explicit closed-schema matches and established diagnostic paths"
)]

// r[impl choregraph.nickel.lowerer]
use crate::diagnostic::{FrontendFailure, FrontendIssueCode};
use crate::imports::{ObservedImport, ValidatedImportClosure, validate_import_closure};
use crate::model::{
    CHOREGRAPH_EVALUATED_SCHEMA, EvaluatedConcreteReference, EvaluatedEffect,
    EvaluatedLatticeEffectV2, EvaluatedLimits, EvaluatedReferenceBinding, EvaluatedSource,
    LATTICE_EVALUATED_SCHEMA, LATTICE_V2_EVALUATED_SCHEMA, LatticeNode, LatticeNodeConfig,
    LatticeNodeConfigV2, LatticePortDirection,
};
use crate::profile::FrontendLimits;
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, ConcreteEffectDeclarationSpec, ConcreteReferenceSpec,
    EdgeSpec, EffectDeclarationSpec, LINEAR_LIFECYCLE_V1, LocalState, MachineSpec, MachineSpecV2,
    NodeSpec, ReferenceBindingSpec, TransitionSelector,
};
use std::collections::{BTreeMap, BTreeSet};

const LATTICE_NODE_PREFIX: &str = "lattice-node/";
const LATTICE_OPERATION_PREFIX: &str = "lattice/";
const LIFECYCLE_EVENT_COUNT: usize = 5;
const LIFECYCLE_EVENTS: [&str; LIFECYCLE_EVENT_COUNT] =
    ["cancel", "complete", "fail", "mark-ready", "start"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweredSourceFormat {
    ChoregraphV1,
    LatticeWorkflowV1,
    LatticeWorkflowV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweringOutput {
    pub machine: AdmittedMachine,
    pub import_closure: ValidatedImportClosure,
    pub source_format: LoweredSourceFormat,
}

enum LoweredMachineSpec {
    V1(MachineSpec),
    V2(MachineSpecV2),
}

fn fail(code: FrontendIssueCode) -> FrontendFailure {
    FrontendFailure::one(code)
}

fn to_usize(value: u64) -> Result<usize, FrontendFailure> {
    usize::try_from(value).map_err(|_| fail(FrontendIssueCode::InvalidLimits))
}

fn admission_limits(
    requested: EvaluatedLimits,
    ceiling: AdmissionLimits,
) -> Result<AdmissionLimits, FrontendFailure> {
    let limits = AdmissionLimits {
        max_nodes: to_usize(requested.max_nodes)?,
        max_edges: to_usize(requested.max_edges)?,
        max_operations: to_usize(requested.max_operations)?,
        max_effect_declarations: to_usize(requested.max_effect_declarations)?,
        max_identifier_bytes: to_usize(requested.max_identifier_bytes)?,
        max_reference_bytes: to_usize(requested.max_reference_bytes)?,
        max_event_bytes: to_usize(requested.max_event_bytes)?,
        max_intent_bytes: to_usize(requested.max_intent_bytes)?,
        max_intents_per_transition: to_usize(requested.max_intents_per_transition)?,
    };
    let selected = [
        (limits.max_nodes, ceiling.max_nodes),
        (limits.max_edges, ceiling.max_edges),
        (limits.max_operations, ceiling.max_operations),
        (
            limits.max_effect_declarations,
            ceiling.max_effect_declarations,
        ),
        (limits.max_identifier_bytes, ceiling.max_identifier_bytes),
        (limits.max_reference_bytes, ceiling.max_reference_bytes),
        (limits.max_event_bytes, ceiling.max_event_bytes),
        (limits.max_intent_bytes, ceiling.max_intent_bytes),
        (
            limits.max_intents_per_transition,
            ceiling.max_intents_per_transition,
        ),
    ];
    if selected
        .iter()
        .any(|(value, maximum)| *value == 0 || value > maximum)
    {
        return Err(fail(FrontendIssueCode::MachineLimitExceedsProfile));
    }
    Ok(limits)
}

fn state(value: &str) -> Result<LocalState, FrontendFailure> {
    match value {
        "pending" => Ok(LocalState::Pending),
        "ready" => Ok(LocalState::Ready),
        "running" => Ok(LocalState::Running),
        "completed" => Ok(LocalState::Completed),
        "failed" => Ok(LocalState::Failed),
        "cancelled" => Ok(LocalState::Cancelled),
        _ => Err(fail(FrontendIssueCode::MachineAdmissionRejected)),
    }
}

fn valid_events(mut events: Vec<String>) -> bool {
    events.sort();
    events.dedup();
    events.len() == LIFECYCLE_EVENT_COUNT && events.iter().map(String::as_str).eq(LIFECYCLE_EVENTS)
}

fn validate_source_profile(
    schema: &str,
    expected_schema: &str,
    lifecycle_profile: &str,
    events: Vec<String>,
) -> Result<(), FrontendFailure> {
    if schema != expected_schema {
        return Err(fail(FrontendIssueCode::UnsupportedSchema));
    }
    if lifecycle_profile != LINEAR_LIFECYCLE_V1 {
        return Err(fail(FrontendIssueCode::UnsupportedLifecycle));
    }
    if !valid_events(events) {
        return Err(fail(FrontendIssueCode::UnsupportedEvents));
    }
    Ok(())
}

fn action(value: &str) -> Result<Action, FrontendFailure> {
    match value {
        "mark-ready" => Ok(Action::MarkReady),
        "start" => Ok(Action::Start),
        "complete" => Ok(Action::Complete),
        "fail" => Ok(Action::Fail),
        "cancel" => Ok(Action::Cancel),
        _ => Err(fail(FrontendIssueCode::MachineAdmissionRejected)),
    }
}

fn effect(input: EvaluatedEffect) -> Result<EffectDeclarationSpec, FrontendFailure> {
    Ok(EffectDeclarationSpec {
        node_id: input.node_id,
        transition: TransitionSelector {
            from: state(&input.from_state)?,
            action: action(&input.action)?,
        },
        operation: input.operation,
        resource: input.resource,
        ability: input.ability,
        request: input.request,
        payload: input.payload,
    })
}

fn concrete_reference(input: EvaluatedConcreteReference) -> ConcreteReferenceSpec {
    ConcreteReferenceSpec {
        profile: input.profile,
        identity: input.identity,
    }
}

fn reference_binding(input: EvaluatedReferenceBinding) -> ReferenceBindingSpec {
    match input {
        EvaluatedReferenceBinding::Exact { reference } => ReferenceBindingSpec::Exact {
            reference: concrete_reference(reference),
        },
        EvaluatedReferenceBinding::Slot {
            slot_id,
            source_ref,
        } => ReferenceBindingSpec::Slot {
            slot_id,
            source_ref,
        },
    }
}

fn binding_source_is_local(node_id: u64, binding: &EvaluatedReferenceBinding) -> bool {
    let EvaluatedReferenceBinding::Slot { source_ref, .. } = binding else {
        return true;
    };
    let node_prefix = format!("node/{node_id}/");
    let Some(remainder) = source_ref.strip_prefix(&node_prefix) else {
        return false;
    };
    remainder.starts_with("input/")
        || remainder.starts_with("output/")
        || remainder.starts_with("config/")
}

fn lattice_node_id(node_id: u64) -> String {
    format!("{LATTICE_NODE_PREFIX}{node_id}")
}

fn lattice_operation(value: &str) -> String {
    format!("{LATTICE_OPERATION_PREFIX}{value}")
}

fn lower_lattice(
    nodes: Vec<LatticeNode>,
    edges: Vec<crate::model::LatticePortEdge>,
    configs: Vec<LatticeNodeConfig>,
) -> Result<(Vec<NodeSpec>, Vec<EdgeSpec>), FrontendFailure> {
    let node_ids = nodes.iter().map(|node| node.id).collect::<BTreeSet<_>>();
    if node_ids.len() != nodes.len() {
        return Err(fail(FrontendIssueCode::MachineAdmissionRejected));
    }
    let mut ports = BTreeMap::new();
    for node in &nodes {
        for port in &node.ports {
            if ports.insert(port.id, (node.id, port.direction)).is_some() {
                return Err(fail(FrontendIssueCode::DuplicateLatticePort));
            }
        }
    }
    let mut operations = BTreeMap::new();
    for config in configs {
        if !node_ids.contains(&config.node_id) {
            return Err(fail(FrontendIssueCode::UnknownLatticeConfigNode));
        }
        if operations
            .insert(config.node_id, config.operation)
            .is_some()
        {
            return Err(fail(FrontendIssueCode::DuplicateLatticeConfig));
        }
    }
    let mut lowered_nodes = Vec::with_capacity(nodes.len());
    for node in nodes {
        let operation = operations
            .remove(&node.id)
            .ok_or_else(|| fail(FrontendIssueCode::MissingLatticeConfig))?;
        lowered_nodes.push(NodeSpec::new(
            lattice_node_id(node.id),
            vec![lattice_operation(&operation)],
        ));
    }
    if !operations.is_empty() {
        return Err(fail(FrontendIssueCode::UnknownLatticeConfigNode));
    }
    let mut lowered_edges = Vec::with_capacity(edges.len());
    for edge in edges {
        let (source_node, source_direction) = ports
            .get(&edge.source)
            .copied()
            .ok_or_else(|| fail(FrontendIssueCode::MissingLatticePort))?;
        let (target_node, target_direction) = ports
            .get(&edge.target)
            .copied()
            .ok_or_else(|| fail(FrontendIssueCode::MissingLatticePort))?;
        if source_direction != LatticePortDirection::Output
            || target_direction != LatticePortDirection::Input
        {
            return Err(fail(FrontendIssueCode::WrongLatticePortDirection));
        }
        lowered_edges.push(EdgeSpec::new(
            lattice_node_id(source_node),
            lattice_node_id(target_node),
        ));
    }
    Ok((lowered_nodes, lowered_edges))
}

fn lower_lattice_v2(
    nodes: Vec<LatticeNode>,
    edges: Vec<crate::model::LatticePortEdge>,
    configs: Vec<LatticeNodeConfigV2>,
    effects: Vec<EvaluatedLatticeEffectV2>,
) -> Result<MachineSpecV2, FrontendFailure> {
    let mut operations = BTreeMap::new();
    let mut effectful_nodes = BTreeSet::new();
    let mut legacy_configs = Vec::with_capacity(configs.len());
    for config in configs {
        if operations
            .insert(config.node_id, config.operation.clone())
            .is_some()
        {
            return Err(fail(FrontendIssueCode::DuplicateLatticeConfig));
        }
        if config.effectful {
            effectful_nodes.insert(config.node_id);
        }
        legacy_configs.push(LatticeNodeConfig {
            node_id: config.node_id,
            operation: config.operation,
        });
    }
    let (nodes, edges) = lower_lattice(nodes, edges, legacy_configs)?;
    let mut declared_effectful_nodes = BTreeSet::new();
    let mut lowered_effects = Vec::with_capacity(effects.len());
    for effect in effects {
        let Some(operation) = operations.get(&effect.node_id) else {
            return Err(fail(FrontendIssueCode::UnknownLatticeEffectNode));
        };
        if !effectful_nodes.contains(&effect.node_id) {
            return Err(fail(FrontendIssueCode::UnknownLatticeEffectNode));
        }
        if !binding_source_is_local(effect.node_id, &effect.request_binding)
            || !binding_source_is_local(effect.node_id, &effect.payload_binding)
        {
            return Err(fail(FrontendIssueCode::InvalidLatticeSourceRef));
        }
        declared_effectful_nodes.insert(effect.node_id);
        lowered_effects.push(ConcreteEffectDeclarationSpec {
            declaration_key: effect.declaration_key,
            node_id: lattice_node_id(effect.node_id),
            transition: TransitionSelector {
                from: state(&effect.from_state)?,
                action: action(&effect.action)?,
            },
            operation: lattice_operation(operation),
            resource: effect.resource,
            ability: effect.ability,
            request_binding: reference_binding(effect.request_binding),
            payload_binding: reference_binding(effect.payload_binding),
        });
    }
    if declared_effectful_nodes != effectful_nodes {
        return Err(fail(FrontendIssueCode::MissingLatticeEffect));
    }
    let mut spec = MachineSpecV2::new(nodes, edges);
    spec.effects = lowered_effects;
    Ok(spec)
}

pub fn parse_evaluated_json(
    evaluated_json: &[u8],
    max_evaluated_bytes: u64,
) -> Result<EvaluatedSource, FrontendFailure> {
    let evaluated_bytes = u64::try_from(evaluated_json.len())
        .map_err(|_| fail(FrontendIssueCode::EvaluatedInputTooLarge))?;
    if evaluated_json.is_empty() || evaluated_bytes > max_evaluated_bytes {
        return Err(fail(FrontendIssueCode::EvaluatedInputTooLarge));
    }
    serde_json::from_slice(evaluated_json)
        .map_err(|_| fail(FrontendIssueCode::MalformedEvaluatedValue))
}

pub fn lower_evaluated_json(
    evaluated_json: &[u8],
    observations: &[ObservedImport<'_>],
    frontend_limits: FrontendLimits,
) -> Result<LoweringOutput, FrontendFailure> {
    let max_evaluated_bytes = u64::try_from(frontend_limits.max_evaluated_bytes)
        .map_err(|_| fail(FrontendIssueCode::EvaluatedInputTooLarge))?;
    let source = parse_evaluated_json(evaluated_json, max_evaluated_bytes)?;
    let import_closure = validate_import_closure(source.imports(), observations, frontend_limits)?;

    let (spec, limits, source_format) = match source {
        EvaluatedSource::Choregraph {
            schema,
            lifecycle_profile,
            events,
            limits,
            imports: _,
            nodes,
            edges,
            effects,
        } => {
            validate_source_profile(
                &schema,
                CHOREGRAPH_EVALUATED_SCHEMA,
                &lifecycle_profile,
                events,
            )?;
            let admission_limits = admission_limits(limits, frontend_limits.machine_ceiling)?;
            let mut spec = MachineSpec::new(
                nodes
                    .into_iter()
                    .map(|node| NodeSpec::new(node.id, node.operations))
                    .collect(),
                edges
                    .into_iter()
                    .map(|edge| EdgeSpec::new(edge.source, edge.target))
                    .collect(),
            );
            spec.effects = effects
                .into_iter()
                .map(effect)
                .collect::<Result<Vec<_>, FrontendFailure>>()?;
            (
                LoweredMachineSpec::V1(spec),
                admission_limits,
                LoweredSourceFormat::ChoregraphV1,
            )
        }
        EvaluatedSource::LatticeWorkflow {
            schema,
            lifecycle_profile,
            events,
            limits,
            imports: _,
            nodes,
            edges,
            configs,
        } => {
            validate_source_profile(
                &schema,
                LATTICE_EVALUATED_SCHEMA,
                &lifecycle_profile,
                events,
            )?;
            let admission_limits = admission_limits(limits, frontend_limits.machine_ceiling)?;
            let (nodes, edges) = lower_lattice(nodes, edges, configs)?;
            (
                LoweredMachineSpec::V1(MachineSpec::new(nodes, edges)),
                admission_limits,
                LoweredSourceFormat::LatticeWorkflowV1,
            )
        }
        EvaluatedSource::LatticeWorkflowV2 {
            schema,
            lifecycle_profile,
            events,
            limits,
            imports: _,
            nodes,
            edges,
            configs,
            effects,
        } => {
            validate_source_profile(
                &schema,
                LATTICE_V2_EVALUATED_SCHEMA,
                &lifecycle_profile,
                events,
            )?;
            let admission_limits = admission_limits(limits, frontend_limits.machine_ceiling)?;
            let spec = lower_lattice_v2(nodes, edges, configs, effects)?;
            (
                LoweredMachineSpec::V2(spec),
                admission_limits,
                LoweredSourceFormat::LatticeWorkflowV2,
            )
        }
    };
    let machine = match spec {
        LoweredMachineSpec::V1(spec) => AdmittedMachine::admit(spec, limits),
        LoweredMachineSpec::V2(spec) => AdmittedMachine::admit_v2(spec, limits),
    }
    .map_err(|_| fail(FrontendIssueCode::MachineAdmissionRejected))?;
    Ok(LoweringOutput {
        machine,
        import_closure,
        source_format,
    })
}
