// Lowering preserves explicit closed-schema matches and established diagnostic paths.
#![allow(
    unknown_lints,
    assertion_density,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    path_segment_repetition,
    reason = "lowering preserves explicit closed-schema matches and established diagnostic paths"
)]

// r[impl choregraph.nickel.lowerer]

const LATTICE_NODE_PREFIX: &str = "lattice-node/";
const LATTICE_OPERATION_PREFIX: &str = "lattice/";
const LIFECYCLE_EVENT_COUNT: usize = 5;
const LIFECYCLE_EVENTS: [&str; LIFECYCLE_EVENT_COUNT] =
    ["cancel", "complete", "fail", "mark-ready", "start"];
const DURABLE_EVENT_COUNT: usize = 7;
const DURABLE_EVENTS: [&str; DURABLE_EVENT_COUNT] = [
    "cancel",
    "complete",
    "fail",
    "mark-ready",
    "resume-signal",
    "start",
    "suspend",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweredSourceFormat {
    ChoregraphV1,
    LatticeWorkflowV1,
    LatticeWorkflowV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoweringOutput {
    pub machine: choregraph_core::model::automaton::AdmittedMachine,
    pub import_closure: crate::imports::ValidatedImportClosure,
    pub source_format: LoweredSourceFormat,
}

enum LoweredMachineSpec {
    V1(choregraph_core::model::automaton::MachineSpec),
    V2(choregraph_core::concrete::machine::MachineSpecV2),
}

fn fail(code: crate::diagnostic::FrontendIssueCode) -> crate::diagnostic::FrontendFailure {
    crate::diagnostic::FrontendFailure::one(code)
}

fn to_usize(value: u64) -> Result<usize, crate::diagnostic::FrontendFailure> {
    usize::try_from(value).map_err(|_| fail(crate::diagnostic::FrontendIssueCode::InvalidLimits))
}

fn admission_limits(
    requested: crate::model::graph::EvaluatedLimits,
    ceiling: choregraph_core::model::capacity::AdmissionLimits,
) -> Result<choregraph_core::model::capacity::AdmissionLimits, crate::diagnostic::FrontendFailure> {
    let limits = choregraph_core::model::capacity::AdmissionLimits {
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
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::MachineLimitExceedsProfile,
        ));
    }
    Ok(limits)
}

fn state(
    value: &str,
) -> Result<choregraph_core::model::semantics::LocalState, crate::diagnostic::FrontendFailure> {
    match value {
        "pending" => Ok(choregraph_core::model::semantics::LocalState::Pending),
        "ready" => Ok(choregraph_core::model::semantics::LocalState::Ready),
        "running" => Ok(choregraph_core::model::semantics::LocalState::Running),
        "completed" => Ok(choregraph_core::model::semantics::LocalState::Completed),
        "failed" => Ok(choregraph_core::model::semantics::LocalState::Failed),
        "cancelled" => Ok(choregraph_core::model::semantics::LocalState::Cancelled),
        _ => Err(fail(
            crate::diagnostic::FrontendIssueCode::MachineAdmissionRejected,
        )),
    }
}

fn valid_events<const COUNT: usize>(mut events: Vec<String>, expected: [&str; COUNT]) -> bool {
    events.sort();
    events.dedup();
    events.len() == COUNT && events.iter().map(String::as_str).eq(expected)
}

fn suspension_declaration(
    input: crate::model::graph::EvaluatedSuspension,
) -> choregraph_core::model::pause::SuspensionDeclarationSpec {
    choregraph_core::model::pause::SuspensionDeclarationSpec {
        node_id: input.node_id,
        class: input.class,
        reference: input.reference,
    }
}

struct SourceProfile<'a> {
    observed_schema: &'a str,
    expected_schema: &'a str,
    lifecycle_profile: &'a str,
    events: Vec<String>,
    allow_suspension: bool,
}

fn validate_source_profile(
    profile: SourceProfile<'_>,
) -> Result<(), crate::diagnostic::FrontendFailure> {
    if profile.observed_schema != profile.expected_schema {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::UnsupportedSchema,
        ));
    }
    if profile.lifecycle_profile == choregraph_core::model::semantics::LINEAR_LIFECYCLE_V1 {
        if !valid_events(profile.events, LIFECYCLE_EVENTS) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::UnsupportedEvents,
            ));
        }
        return Ok(());
    }
    if profile.lifecycle_profile == choregraph_core::model::semantics::DURABLE_SUSPENSION_V1
        && profile.allow_suspension
    {
        if !valid_events(profile.events, DURABLE_EVENTS) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::UnsupportedEvents,
            ));
        }
        return Ok(());
    }
    Err(fail(
        crate::diagnostic::FrontendIssueCode::UnsupportedLifecycle,
    ))
}

fn action(
    value: &str,
) -> Result<choregraph_core::model::semantics::Action, crate::diagnostic::FrontendFailure> {
    match value {
        "mark-ready" => Ok(choregraph_core::model::semantics::Action::MarkReady),
        "start" => Ok(choregraph_core::model::semantics::Action::Start),
        "complete" => Ok(choregraph_core::model::semantics::Action::Complete),
        "fail" => Ok(choregraph_core::model::semantics::Action::Fail),
        "cancel" => Ok(choregraph_core::model::semantics::Action::Cancel),
        _ => Err(fail(
            crate::diagnostic::FrontendIssueCode::MachineAdmissionRejected,
        )),
    }
}

fn effect(
    input: crate::model::graph::EvaluatedEffect,
) -> Result<
    choregraph_core::model::dispatch::EffectDeclarationSpec,
    crate::diagnostic::FrontendFailure,
> {
    Ok(choregraph_core::model::dispatch::EffectDeclarationSpec {
        node_id: input.node_id,
        transition: choregraph_core::model::semantics::TransitionSelector {
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

fn concrete_reference(
    input: crate::model::projection::EvaluatedConcreteReference,
) -> choregraph_core::concrete::reference::ConcreteReferenceSpec {
    choregraph_core::concrete::reference::ConcreteReferenceSpec {
        profile: input.profile,
        identity: input.identity,
    }
}

fn reference_binding(
    input: crate::model::projection::EvaluatedReferenceBinding,
) -> choregraph_core::concrete::reference::ReferenceBindingSpec {
    match input {
        crate::model::projection::EvaluatedReferenceBinding::Exact { reference } => {
            choregraph_core::concrete::reference::ReferenceBindingSpec::Exact {
                reference: concrete_reference(reference),
            }
        }
        crate::model::projection::EvaluatedReferenceBinding::Slot {
            slot_id,
            source_ref,
        } => choregraph_core::concrete::reference::ReferenceBindingSpec::Slot {
            slot_id,
            source_ref,
        },
    }
}

fn binding_source_is_local(
    node_id: u64,
    binding: &crate::model::projection::EvaluatedReferenceBinding,
) -> bool {
    let crate::model::projection::EvaluatedReferenceBinding::Slot { source_ref, .. } = binding
    else {
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
    nodes: Vec<crate::model::projection::LatticeNode>,
    edges: Vec<crate::model::projection::LatticePortEdge>,
    configs: Vec<crate::model::projection::LatticeNodeConfig>,
) -> Result<
    (
        Vec<choregraph_core::model::graph::NodeSpec>,
        Vec<choregraph_core::model::graph::EdgeSpec>,
    ),
    crate::diagnostic::FrontendFailure,
> {
    let node_ids = nodes
        .iter()
        .map(|node| node.id)
        .collect::<std::collections::BTreeSet<_>>();
    if node_ids.len() != nodes.len() {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::MachineAdmissionRejected,
        ));
    }
    let mut ports = std::collections::BTreeMap::new();
    for node in &nodes {
        for port in &node.ports {
            if ports.insert(port.id, (node.id, port.direction)).is_some() {
                return Err(fail(
                    crate::diagnostic::FrontendIssueCode::DuplicateLatticePort,
                ));
            }
        }
    }
    let mut operations = std::collections::BTreeMap::new();
    for config in configs {
        if !node_ids.contains(&config.node_id) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::UnknownLatticeConfigNode,
            ));
        }
        if operations
            .insert(config.node_id, config.operation)
            .is_some()
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::DuplicateLatticeConfig,
            ));
        }
    }
    let mut lowered_nodes = Vec::with_capacity(nodes.len());
    for node in nodes {
        let operation = operations
            .remove(&node.id)
            .ok_or_else(|| fail(crate::diagnostic::FrontendIssueCode::MissingLatticeConfig))?;
        lowered_nodes.push(choregraph_core::model::graph::NodeSpec::new(
            lattice_node_id(node.id),
            vec![lattice_operation(&operation)],
        ));
    }
    if !operations.is_empty() {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::UnknownLatticeConfigNode,
        ));
    }
    let mut lowered_edges = Vec::with_capacity(edges.len());
    for edge in edges {
        let (source_node, source_direction) = ports
            .get(&edge.source)
            .copied()
            .ok_or_else(|| fail(crate::diagnostic::FrontendIssueCode::MissingLatticePort))?;
        let (target_node, target_direction) = ports
            .get(&edge.target)
            .copied()
            .ok_or_else(|| fail(crate::diagnostic::FrontendIssueCode::MissingLatticePort))?;
        if source_direction != crate::model::projection::LatticePortDirection::Output
            || target_direction != crate::model::projection::LatticePortDirection::Input
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::WrongLatticePortDirection,
            ));
        }
        lowered_edges.push(choregraph_core::model::graph::EdgeSpec::new(
            lattice_node_id(source_node),
            lattice_node_id(target_node),
        ));
    }
    Ok((lowered_nodes, lowered_edges))
}

fn lower_lattice_v2(
    nodes: Vec<crate::model::projection::LatticeNode>,
    edges: Vec<crate::model::projection::LatticePortEdge>,
    configs: Vec<crate::model::projection::LatticeNodeConfigV2>,
    effects: Vec<crate::model::projection::EvaluatedLatticeEffectV2>,
) -> Result<choregraph_core::concrete::machine::MachineSpecV2, crate::diagnostic::FrontendFailure> {
    let mut operations = std::collections::BTreeMap::new();
    let mut effectful_nodes = std::collections::BTreeSet::new();
    let mut legacy_configs = Vec::with_capacity(configs.len());
    for config in configs {
        if operations
            .insert(config.node_id, config.operation.clone())
            .is_some()
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::DuplicateLatticeConfig,
            ));
        }
        if config.effectful {
            effectful_nodes.insert(config.node_id);
        }
        legacy_configs.push(crate::model::projection::LatticeNodeConfig {
            node_id: config.node_id,
            operation: config.operation,
        });
    }
    let (nodes, edges) = lower_lattice(nodes, edges, legacy_configs)?;
    let mut declared_effectful_nodes = std::collections::BTreeSet::new();
    let mut lowered_effects = Vec::with_capacity(effects.len());
    for effect in effects {
        let Some(operation) = operations.get(&effect.node_id) else {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::UnknownLatticeEffectNode,
            ));
        };
        if !effectful_nodes.contains(&effect.node_id) {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::UnknownLatticeEffectNode,
            ));
        }
        if !binding_source_is_local(effect.node_id, &effect.request_binding)
            || !binding_source_is_local(effect.node_id, &effect.payload_binding)
        {
            return Err(fail(
                crate::diagnostic::FrontendIssueCode::InvalidLatticeSourceRef,
            ));
        }
        declared_effectful_nodes.insert(effect.node_id);
        lowered_effects.push(
            choregraph_core::concrete::effect::ConcreteEffectDeclarationSpec {
                declaration_key: effect.declaration_key,
                node_id: lattice_node_id(effect.node_id),
                transition: choregraph_core::model::semantics::TransitionSelector {
                    from: state(&effect.from_state)?,
                    action: action(&effect.action)?,
                },
                operation: lattice_operation(operation),
                resource: effect.resource,
                ability: effect.ability,
                request_binding: reference_binding(effect.request_binding),
                payload_binding: reference_binding(effect.payload_binding),
            },
        );
    }
    if declared_effectful_nodes != effectful_nodes {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::MissingLatticeEffect,
        ));
    }
    let mut spec = choregraph_core::concrete::machine::MachineSpecV2::new(nodes, edges);
    spec.effects = lowered_effects;
    Ok(spec)
}

pub fn parse_evaluated_json(
    evaluated_json: &[u8],
    max_evaluated_bytes: u64,
) -> Result<crate::model::admission::EvaluatedSource, crate::diagnostic::FrontendFailure> {
    let evaluated_bytes = u64::try_from(evaluated_json.len())
        .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::EvaluatedInputTooLarge))?;
    if evaluated_json.is_empty() || evaluated_bytes > max_evaluated_bytes {
        return Err(fail(
            crate::diagnostic::FrontendIssueCode::EvaluatedInputTooLarge,
        ));
    }
    serde_json::from_slice(evaluated_json)
        .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::MalformedEvaluatedValue))
}

pub fn lower_evaluated_json(
    evaluated_json: &[u8],
    observations: &[crate::imports::ObservedImport<'_>],
    frontend_limits: crate::profile::FrontendLimits,
) -> Result<LoweringOutput, crate::diagnostic::FrontendFailure> {
    let max_evaluated_bytes = u64::try_from(frontend_limits.max_evaluated_bytes)
        .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::EvaluatedInputTooLarge))?;
    let source = parse_evaluated_json(evaluated_json, max_evaluated_bytes)?;
    let import_closure =
        crate::imports::validate_import_closure(source.imports()?, observations, frontend_limits)?;

    let (spec, limits, source_format) = match source {
        crate::model::admission::EvaluatedSource::Choregraph {
            schema,
            lifecycle_profile,
            events,
            limits,
            imports: _,
            nodes,
            edges,
            effects,
            suspensions,
        } => {
            validate_source_profile(SourceProfile {
                observed_schema: &schema,
                expected_schema: crate::model::admission::CHOREGRAPH_EVALUATED_SCHEMA,
                lifecycle_profile: &lifecycle_profile,
                events,
                allow_suspension: true,
            })?;
            let admission_limits = admission_limits(limits, frontend_limits.machine_ceiling)?;
            let mut spec = choregraph_core::model::automaton::MachineSpec::new(
                nodes
                    .into_iter()
                    .map(|node| {
                        choregraph_core::model::graph::NodeSpec::new(node.id, node.operations)
                    })
                    .collect(),
                edges
                    .into_iter()
                    .map(|edge| {
                        choregraph_core::model::graph::EdgeSpec::new(edge.source, edge.target)
                    })
                    .collect(),
            );
            spec.lifecycle_profile = lifecycle_profile;
            spec.effects = effects
                .into_iter()
                .map(effect)
                .collect::<Result<Vec<_>, crate::diagnostic::FrontendFailure>>()?;
            spec.suspensions = suspensions
                .into_iter()
                .map(suspension_declaration)
                .collect();
            (
                LoweredMachineSpec::V1(spec),
                admission_limits,
                LoweredSourceFormat::ChoregraphV1,
            )
        }
        crate::model::admission::EvaluatedSource::LatticeWorkflow {
            schema,
            lifecycle_profile,
            events,
            limits,
            imports: _,
            nodes,
            edges,
            configs,
        } => {
            validate_source_profile(SourceProfile {
                observed_schema: &schema,
                expected_schema: crate::model::admission::LATTICE_EVALUATED_SCHEMA,
                lifecycle_profile: &lifecycle_profile,
                events,
                allow_suspension: false,
            })?;
            let admission_limits = admission_limits(limits, frontend_limits.machine_ceiling)?;
            let (nodes, edges) = lower_lattice(nodes, edges, configs)?;
            (
                LoweredMachineSpec::V1(choregraph_core::model::automaton::MachineSpec::new(
                    nodes, edges,
                )),
                admission_limits,
                LoweredSourceFormat::LatticeWorkflowV1,
            )
        }
        crate::model::admission::EvaluatedSource::LatticeWorkflowV2 {
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
            validate_source_profile(SourceProfile {
                observed_schema: &schema,
                expected_schema: crate::model::admission::LATTICE_V2_EVALUATED_SCHEMA,
                lifecycle_profile: &lifecycle_profile,
                events,
                allow_suspension: false,
            })?;
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
        LoweredMachineSpec::V1(spec) => {
            choregraph_core::model::automaton::AdmittedMachine::admit(spec, limits)
        }
        LoweredMachineSpec::V2(spec) => {
            choregraph_core::model::automaton::AdmittedMachine::admit_v2(spec, limits)
        }
    }
    .map_err(|_| fail(crate::diagnostic::FrontendIssueCode::MachineAdmissionRejected))?;
    Ok(LoweringOutput {
        machine,
        import_closure,
        source_format,
    })
}
