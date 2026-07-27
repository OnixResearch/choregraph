// r[impl choregraph.machine.admission]
// r[impl choregraph.machine.bounds]
// r[impl choregraph.machine.determinism]
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::id::{
    BLAKE3_REFERENCE_BYTES, BoundedId, IdentifierError, NodeId, PayloadRef, PayloadRefError,
};
use crate::invariant::well_formed;
use crate::model::{
    AdmissionDiagnostic, AdmissionDiagnosticCode, AdmissionLimits, AdmittedMachine, AdmittedNode,
    Edge, EffectDeclaration, EffectDeclarationSpec, EffectKey, LINEAR_LIFECYCLE_V1,
    LifecycleProfile, LimitKind, MACHINE_SCHEMA_V1, MachineSpec,
};
use crate::transition::lifecycle_target;

const MINIMUM_BYTE_LIMIT: usize = 1;
const MINIMUM_EVENT_ACTION_TAG_BYTES: usize = 1;
const INTENT_IDENTIFIER_FIELD_COUNT: usize = 5;

type EffectIndex = BTreeMap<EffectKey, Vec<EffectDeclaration>>;
type EffectRouteIndices = BTreeMap<EffectKey, Vec<usize>>;
type AdmittedEffects = (Vec<EffectDeclaration>, EffectIndex, EffectRouteIndices);

fn diagnostic(
    code: AdmissionDiagnosticCode,
    subject: Option<&str>,
    related: Option<&str>,
) -> AdmissionDiagnostic {
    AdmissionDiagnostic {
        code,
        subject: subject.map(ToString::to_string),
        related: related.map(ToString::to_string),
        limit: None,
        observed: None,
        maximum: None,
    }
}

fn limit_diagnostic(
    code: AdmissionDiagnosticCode,
    kind: LimitKind,
    observed: usize,
    maximum: usize,
) -> AdmissionDiagnostic {
    AdmissionDiagnostic {
        code,
        subject: None,
        related: None,
        limit: Some(kind),
        observed: Some(observed),
        maximum: Some(maximum),
    }
}

fn identifier_diagnostic(error: IdentifierError, subject: &str) -> AdmissionDiagnostic {
    match error {
        IdentifierError::Empty => diagnostic(
            AdmissionDiagnosticCode::EmptyIdentifier,
            Some(subject),
            None,
        ),
        IdentifierError::TooLong { observed, maximum } => AdmissionDiagnostic {
            code: AdmissionDiagnosticCode::IdentifierTooLong,
            subject: Some(String::from(subject)),
            related: None,
            limit: Some(LimitKind::IdentifierBytes),
            observed: Some(observed),
            maximum: Some(maximum),
        },
        IdentifierError::InvalidByte { index } => AdmissionDiagnostic {
            code: AdmissionDiagnosticCode::MalformedIdentifier,
            subject: Some(String::from(subject)),
            related: None,
            limit: None,
            observed: Some(index),
            maximum: None,
        },
    }
}

fn reference_diagnostic(error: PayloadRefError, subject: &str) -> AdmissionDiagnostic {
    match error {
        PayloadRefError::TooLong { observed, maximum } => AdmissionDiagnostic {
            code: AdmissionDiagnosticCode::ReferenceTooLong,
            subject: Some(String::from(subject)),
            related: None,
            limit: Some(LimitKind::ReferenceBytes),
            observed: Some(observed),
            maximum: Some(maximum),
        },
        PayloadRefError::Malformed => diagnostic(
            AdmissionDiagnosticCode::MalformedReference,
            Some(subject),
            None,
        ),
    }
}

fn parse_id<Domain>(
    value: &str,
    limits: AdmissionLimits,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> Option<BoundedId<Domain>> {
    match BoundedId::parse(value, limits.max_identifier_bytes) {
        Ok(identifier) => Some(identifier),
        Err(error) => {
            diagnostics.push(identifier_diagnostic(error, value));
            None
        }
    }
}

fn validate_limits(limits: AdmissionLimits) -> Vec<AdmissionDiagnostic> {
    let mut diagnostics = Vec::new();
    let minimum_intent_bytes = MINIMUM_BYTE_LIMIT
        .checked_mul(INTENT_IDENTIFIER_FIELD_COUNT)
        .and_then(|bytes| bytes.checked_add(BLAKE3_REFERENCE_BYTES));

    let byte_limits = [
        (LimitKind::IdentifierBytes, limits.max_identifier_bytes),
        (LimitKind::ReferenceBytes, limits.max_reference_bytes),
        (LimitKind::EventBytes, limits.max_event_bytes),
        (LimitKind::IntentBytes, limits.max_intent_bytes),
    ];
    for (kind, value) in byte_limits {
        if value < MINIMUM_BYTE_LIMIT {
            diagnostics.push(limit_diagnostic(
                AdmissionDiagnosticCode::InconsistentLimits,
                kind,
                value,
                MINIMUM_BYTE_LIMIT,
            ));
        }
    }
    if limits.max_reference_bytes < BLAKE3_REFERENCE_BYTES {
        diagnostics.push(limit_diagnostic(
            AdmissionDiagnosticCode::InconsistentLimits,
            LimitKind::ReferenceBytes,
            limits.max_reference_bytes,
            BLAKE3_REFERENCE_BYTES,
        ));
    }
    let minimum_event_bytes = MINIMUM_BYTE_LIMIT.checked_add(MINIMUM_EVENT_ACTION_TAG_BYTES);
    match minimum_event_bytes {
        Some(minimum) if limits.max_event_bytes < minimum => diagnostics.push(limit_diagnostic(
            AdmissionDiagnosticCode::InconsistentLimits,
            LimitKind::EventBytes,
            limits.max_event_bytes,
            minimum,
        )),
        None => diagnostics.push(limit_diagnostic(
            AdmissionDiagnosticCode::InconsistentLimits,
            LimitKind::EventBytes,
            limits.max_event_bytes,
            usize::MAX,
        )),
        Some(_) => {}
    }
    match minimum_intent_bytes {
        Some(minimum) if limits.max_intent_bytes < minimum => diagnostics.push(limit_diagnostic(
            AdmissionDiagnosticCode::InconsistentLimits,
            LimitKind::IntentBytes,
            limits.max_intent_bytes,
            minimum,
        )),
        None => diagnostics.push(limit_diagnostic(
            AdmissionDiagnosticCode::InconsistentLimits,
            LimitKind::IntentBytes,
            limits.max_intent_bytes,
            usize::MAX,
        )),
        Some(_) => {}
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn count_operations(spec: &MachineSpec) -> usize {
    let mut count = 0usize;
    for node in &spec.nodes {
        count = count.saturating_add(node.operations.len());
    }
    count
}

fn validate_top_level_counts(
    spec: &MachineSpec,
    limits: AdmissionLimits,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) {
    let checks = [
        (
            spec.nodes.len(),
            limits.max_nodes,
            AdmissionDiagnosticCode::TooManyNodes,
            LimitKind::Nodes,
        ),
        (
            spec.edges.len(),
            limits.max_edges,
            AdmissionDiagnosticCode::TooManyEdges,
            LimitKind::Edges,
        ),
        (
            count_operations(spec),
            limits.max_operations,
            AdmissionDiagnosticCode::TooManyOperations,
            LimitKind::Operations,
        ),
        (
            spec.effects.len(),
            limits.max_effect_declarations,
            AdmissionDiagnosticCode::TooManyEffectDeclarations,
            LimitKind::EffectDeclarations,
        ),
    ];
    for (observed, maximum, code, kind) in checks {
        if observed > maximum {
            diagnostics.push(limit_diagnostic(code, kind, observed, maximum));
        }
    }
}

fn admit_nodes(
    spec: &MachineSpec,
    limits: AdmissionLimits,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> BTreeMap<NodeId, AdmittedNode> {
    let mut nodes = BTreeMap::new();
    for node in &spec.nodes {
        let Some(node_id) = parse_id::<crate::id::NodeDomain>(&node.id, limits, diagnostics) else {
            continue;
        };
        let mut operations = BTreeSet::new();
        for raw_operation in &node.operations {
            let Some(operation) =
                parse_id::<crate::id::OperationDomain>(raw_operation, limits, diagnostics)
            else {
                continue;
            };
            if !operations.insert(operation) {
                diagnostics.push(diagnostic(
                    AdmissionDiagnosticCode::DuplicateOperation,
                    Some(&node.id),
                    Some(raw_operation),
                ));
            }
        }
        if nodes.insert(node_id, AdmittedNode { operations }).is_some() {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::DuplicateNode,
                Some(&node.id),
                None,
            ));
        }
    }
    nodes
}

fn admit_edges(
    spec: &MachineSpec,
    limits: AdmissionLimits,
    nodes: &BTreeMap<NodeId, AdmittedNode>,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> BTreeSet<Edge> {
    let mut edges = BTreeSet::new();
    for edge in &spec.edges {
        let source = parse_id::<crate::id::NodeDomain>(&edge.source, limits, diagnostics);
        let target = parse_id::<crate::id::NodeDomain>(&edge.target, limits, diagnostics);
        let (Some(source), Some(target)) = (source, target) else {
            continue;
        };
        if !nodes.contains_key(&source) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::MissingSource,
                Some(&edge.source),
                Some(&edge.target),
            ));
        }
        if !nodes.contains_key(&target) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::MissingTarget,
                Some(&edge.target),
                Some(&edge.source),
            ));
        }
        if source == target {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::SelfLoop,
                Some(&edge.source),
                None,
            ));
        }
        if !edges.insert(Edge { source, target }) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::DuplicateEdge,
                Some(&edge.source),
                Some(&edge.target),
            ));
        }
    }
    edges
}

fn declaration_size(declaration: &EffectDeclaration) -> Option<usize> {
    let fields = [
        declaration.node_id.as_str().len(),
        declaration.operation.as_str().len(),
        declaration.resource.as_str().len(),
        declaration.ability.as_str().len(),
        declaration.request.as_str().len(),
        declaration.payload.as_str().len(),
    ];
    let mut size = 0usize;
    for field in fields {
        size = size.checked_add(field)?;
    }
    Some(size)
}

fn parse_effect_declaration(
    raw: &EffectDeclarationSpec,
    limits: AdmissionLimits,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> Option<EffectDeclaration> {
    let node_id = parse_id::<crate::id::NodeDomain>(&raw.node_id, limits, diagnostics)?;
    let operation = parse_id::<crate::id::OperationDomain>(&raw.operation, limits, diagnostics)?;
    let resource = parse_id::<crate::id::ResourceDomain>(&raw.resource, limits, diagnostics)?;
    let ability = parse_id::<crate::id::AbilityDomain>(&raw.ability, limits, diagnostics)?;
    let request = parse_id::<crate::id::RequestDomain>(&raw.request, limits, diagnostics)?;
    let payload = match PayloadRef::parse(&raw.payload, limits.max_reference_bytes) {
        Ok(payload) => payload,
        Err(error) => {
            diagnostics.push(reference_diagnostic(error, &raw.payload));
            return None;
        }
    };
    if lifecycle_target(raw.transition.from, raw.transition.action).is_err() {
        diagnostics.push(diagnostic(
            AdmissionDiagnosticCode::InvalidEffectTransition,
            Some(&raw.node_id),
            Some(&raw.operation),
        ));
        return None;
    }
    Some(EffectDeclaration {
        node_id,
        transition: raw.transition,
        operation,
        resource,
        ability,
        request,
        payload,
    })
}

fn admit_effects(
    spec: &MachineSpec,
    limits: AdmissionLimits,
    nodes: &BTreeMap<NodeId, AdmittedNode>,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> AdmittedEffects {
    let mut unique = BTreeSet::new();
    for raw in &spec.effects {
        let Some(declaration) = parse_effect_declaration(raw, limits, diagnostics) else {
            continue;
        };
        let Some(node) = nodes.get(&declaration.node_id) else {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::EffectNodeMissing,
                Some(&raw.node_id),
                Some(&raw.operation),
            ));
            continue;
        };
        if !node.operations.contains(&declaration.operation) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::EffectOperationUndeclared,
                Some(&raw.node_id),
                Some(&raw.operation),
            ));
            continue;
        }
        let observed_size = declaration_size(&declaration).unwrap_or(usize::MAX);
        if observed_size > limits.max_intent_bytes {
            diagnostics.push(limit_diagnostic(
                AdmissionDiagnosticCode::IntentTooLarge,
                LimitKind::IntentBytes,
                observed_size,
                limits.max_intent_bytes,
            ));
            continue;
        }
        if !unique.insert(declaration) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::DuplicateEffectDeclaration,
                Some(&raw.node_id),
                Some(&raw.operation),
            ));
        }
    }

    let effects = unique.into_iter().collect::<Vec<_>>();
    let mut index = BTreeMap::<EffectKey, Vec<EffectDeclaration>>::new();
    let mut route_indices = BTreeMap::<EffectKey, Vec<usize>>::new();
    for (effect_index, declaration) in effects.iter().enumerate() {
        let key = EffectKey {
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
        };
        index
            .entry(key.clone())
            .or_default()
            .push(declaration.clone());
        route_indices.entry(key).or_default().push(effect_index);
    }
    for declarations in index.values() {
        if declarations.len() > limits.max_intents_per_transition {
            diagnostics.push(limit_diagnostic(
                AdmissionDiagnosticCode::TooManyIntentsForTransition,
                LimitKind::IntentsPerTransition,
                declarations.len(),
                limits.max_intents_per_transition,
            ));
        }
    }
    (effects, index, route_indices)
}

fn graph_indexes(
    nodes: &BTreeMap<NodeId, AdmittedNode>,
    edges: &BTreeSet<Edge>,
) -> (BTreeMap<NodeId, Vec<NodeId>>, BTreeMap<NodeId, Vec<NodeId>>) {
    let mut dependencies = BTreeMap::new();
    let mut dependents = BTreeMap::new();
    for node_id in nodes.keys() {
        dependencies.insert(node_id.clone(), Vec::new());
        dependents.insert(node_id.clone(), Vec::new());
    }
    for edge in edges {
        if let Some(values) = dependencies.get_mut(&edge.target) {
            values.push(edge.source.clone());
        }
        if let Some(values) = dependents.get_mut(&edge.source) {
            values.push(edge.target.clone());
        }
    }
    (dependencies, dependents)
}

fn topological_order(
    nodes: &BTreeMap<NodeId, AdmittedNode>,
    dependencies: &BTreeMap<NodeId, Vec<NodeId>>,
    dependents: &BTreeMap<NodeId, Vec<NodeId>>,
) -> Vec<NodeId> {
    let mut incoming = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for node_id in nodes.keys() {
        let count = dependencies.get(node_id).map_or(0, Vec::len);
        incoming.insert(node_id.clone(), count);
        if count == 0 {
            ready.insert(node_id.clone());
        }
    }

    let mut order = Vec::with_capacity(nodes.len());
    while let Some(node_id) = ready.pop_first() {
        order.push(node_id.clone());
        let Some(children) = dependents.get(&node_id) else {
            continue;
        };
        for child in children {
            let Some(count) = incoming.get_mut(child) else {
                continue;
            };
            if *count == 0 {
                continue;
            }
            *count = count.saturating_sub(1);
            if *count == 0 {
                ready.insert(child.clone());
            }
        }
    }
    order
}

impl AdmittedMachine {
    /// Admits and normalizes one untrusted machine specification.
    pub fn admit(
        spec: MachineSpec,
        limits: AdmissionLimits,
    ) -> Result<Self, Vec<AdmissionDiagnostic>> {
        let mut diagnostics = validate_limits(limits);
        validate_top_level_counts(&spec, limits, &mut diagnostics);
        diagnostics.sort();
        diagnostics.dedup();
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        if spec.schema != MACHINE_SCHEMA_V1 {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::UnsupportedSchema,
                Some(&spec.schema),
                None,
            ));
        }
        let lifecycle_profile = if spec.lifecycle_profile == LINEAR_LIFECYCLE_V1 {
            Some(LifecycleProfile::LinearV1)
        } else {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::UnsupportedLifecycleProfile,
                Some(&spec.lifecycle_profile),
                None,
            ));
            None
        };
        diagnostics.sort();
        diagnostics.dedup();
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let nodes = admit_nodes(&spec, limits, &mut diagnostics);
        let edges = admit_edges(&spec, limits, &nodes, &mut diagnostics);
        let (effects, effect_index, effect_route_indices) =
            admit_effects(&spec, limits, &nodes, &mut diagnostics);
        diagnostics.sort();
        diagnostics.dedup();
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let Some(lifecycle_profile) = lifecycle_profile else {
            return Err(alloc::vec![diagnostic(
                AdmissionDiagnosticCode::UnsupportedLifecycleProfile,
                Some(&spec.lifecycle_profile),
                None,
            )]);
        };
        let (dependencies, dependents) = graph_indexes(&nodes, &edges);
        let topological_order = topological_order(&nodes, &dependencies, &dependents);
        if topological_order.len() != nodes.len() {
            return Err(alloc::vec![diagnostic(
                AdmissionDiagnosticCode::CycleDetected,
                None,
                None,
            )]);
        }

        let node_order = nodes.keys().cloned().collect::<Vec<_>>();
        let node_indices = node_order
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, node_id)| (node_id, index))
            .collect::<BTreeMap<_, _>>();
        let normalized_edges = edges.into_iter().collect::<Vec<_>>();
        let mut edge_indices = Vec::with_capacity(normalized_edges.len());
        for edge in &normalized_edges {
            let (Some(source), Some(target)) = (
                node_indices.get(&edge.source).copied(),
                node_indices.get(&edge.target).copied(),
            ) else {
                return Err(alloc::vec![diagnostic(
                    AdmissionDiagnosticCode::NormalizationInvariant,
                    None,
                    None,
                )]);
            };
            edge_indices.push((source, target));
        }
        let mut dependency_indices = Vec::with_capacity(node_order.len());
        for node_id in &node_order {
            let Some(node_dependencies) = dependencies.get(node_id) else {
                return Err(alloc::vec![diagnostic(
                    AdmissionDiagnosticCode::NormalizationInvariant,
                    None,
                    None,
                )]);
            };
            let mut indexes = Vec::with_capacity(node_dependencies.len());
            for dependency in node_dependencies {
                let Some(index) = node_indices.get(dependency).copied() else {
                    return Err(alloc::vec![diagnostic(
                        AdmissionDiagnosticCode::NormalizationInvariant,
                        None,
                        None,
                    )]);
                };
                indexes.push(index);
            }
            dependency_indices.push(indexes);
        }
        let mut topological_indices = Vec::with_capacity(topological_order.len());
        for node_id in &topological_order {
            let Some(index) = node_indices.get(node_id).copied() else {
                return Err(alloc::vec![diagnostic(
                    AdmissionDiagnosticCode::NormalizationInvariant,
                    None,
                    None,
                )]);
            };
            topological_indices.push(index);
        }

        let machine = Self {
            limits,
            lifecycle_profile,
            node_order,
            node_indices,
            nodes,
            edges: normalized_edges,
            edge_indices,
            dependencies,
            dependency_indices,
            dependents,
            topological_order,
            topological_indices,
            effects,
            effect_index,
            effect_route_indices,
        };
        if !well_formed(&machine) {
            return Err(alloc::vec![diagnostic(
                AdmissionDiagnosticCode::NormalizationInvariant,
                None,
                None,
            )]);
        }
        Ok(machine)
    }
}
