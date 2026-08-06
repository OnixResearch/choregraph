// Invariant checks mirror proof-linked state relations and deterministic collection shapes.
#![allow(
    unknown_lints,
    allocation_in_loop,
    assertion_density,
    compound_condition,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    unbounded_collection_growth,
    reason = "invariant checks mirror proof-linked state relations and deterministic collection shapes"
)]

// r[impl choregraph.machine.invariants]
// r[impl choregraph.machine.effect_intents]
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use crate::model::{
    AdmittedMachine, Edge, EffectDeclaration, EffectIntent, EffectKey, Event, InvariantIssue,
    LifecycleProfile, LocalState, MachineSchema, MachineState, TransitionSelector,
};
use crate::transition::{lifecycle_target, lifecycle_target_for_profile};

fn dependencies_complete(
    machine: &AdmittedMachine,
    state: &MachineState,
    node_id: &crate::NodeId,
) -> bool {
    let Some(dependencies) = machine.dependencies.get(node_id) else {
        return false;
    };
    for dependency in dependencies {
        if state.state(dependency) != Some(LocalState::Completed) {
            return false;
        }
    }
    true
}

fn indexes_match(machine: &AdmittedMachine) -> bool {
    let mut dependencies = BTreeMap::new();
    let mut dependents = BTreeMap::new();
    for node_id in machine.nodes.keys() {
        dependencies.insert(node_id.clone(), Vec::new());
        dependents.insert(node_id.clone(), Vec::new());
    }
    for edge in &machine.edges {
        let Some(node_dependencies) = dependencies.get_mut(&edge.target) else {
            return false;
        };
        node_dependencies.push(edge.source.clone());
        let Some(node_dependents) = dependents.get_mut(&edge.source) else {
            return false;
        };
        node_dependents.push(edge.target.clone());
    }
    dependencies == machine.dependencies && dependents == machine.dependents
}

fn indexed_views_match(machine: &AdmittedMachine) -> bool {
    let expected_node_indices = machine
        .node_order
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, node_id)| (node_id, index))
        .collect::<BTreeMap<_, _>>();
    if expected_node_indices != machine.node_indices {
        return false;
    }

    let mut expected_edges = Vec::with_capacity(machine.edges.len());
    for edge in &machine.edges {
        let (Some(source), Some(target)) = (
            machine.node_indices.get(&edge.source).copied(),
            machine.node_indices.get(&edge.target).copied(),
        ) else {
            return false;
        };
        expected_edges.push((source, target));
    }
    if expected_edges != machine.edge_indices {
        return false;
    }

    let mut expected_dependencies = Vec::with_capacity(machine.node_order.len());
    for node_id in &machine.node_order {
        let Some(dependencies) = machine.dependencies.get(node_id) else {
            return false;
        };
        let mut indexes = Vec::with_capacity(dependencies.len());
        for dependency in dependencies {
            let Some(index) = machine.node_indices.get(dependency).copied() else {
                return false;
            };
            indexes.push(index);
        }
        expected_dependencies.push(indexes);
    }
    if expected_dependencies != machine.dependency_indices {
        return false;
    }

    let mut expected_order = Vec::with_capacity(machine.topological_order.len());
    for node_id in &machine.topological_order {
        let Some(index) = machine.node_indices.get(node_id).copied() else {
            return false;
        };
        expected_order.push(index);
    }
    expected_order == machine.topological_indices
}

fn topology_matches(machine: &AdmittedMachine) -> bool {
    crate::verified_topology::admitted_graph_valid(
        &machine.edge_indices,
        machine.nodes.len(),
        &machine.topological_indices,
        &machine.dependency_indices,
    )
}

fn intent_size(declaration: &EffectDeclaration) -> Option<usize> {
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

fn effects_match(machine: &AdmittedMachine) -> bool {
    let mut normalized = machine.effects.clone();
    normalized.sort();
    normalized.dedup();
    if normalized != machine.effects {
        return false;
    }

    let mut expected_index = BTreeMap::<EffectKey, Vec<EffectDeclaration>>::new();
    let mut expected_route_indices = BTreeMap::<EffectKey, Vec<usize>>::new();
    for (effect_index, declaration) in machine.effects.iter().enumerate() {
        let Some(node) = machine.nodes.get(&declaration.node_id) else {
            return false;
        };
        if !node.operations.contains(&declaration.operation) {
            return false;
        }
        if lifecycle_target(declaration.transition.from, declaration.transition.action).is_err() {
            return false;
        }
        if declaration.node_id.as_str().len() > machine.limits.max_identifier_bytes
            || declaration.operation.as_str().len() > machine.limits.max_identifier_bytes
            || declaration.resource.as_str().len() > machine.limits.max_identifier_bytes
            || declaration.ability.as_str().len() > machine.limits.max_identifier_bytes
            || declaration.request.as_str().len() > machine.limits.max_identifier_bytes
            || declaration.payload.as_str().len() > machine.limits.max_reference_bytes
            || intent_size(declaration).is_none_or(|size| size > machine.limits.max_intent_bytes)
        {
            return false;
        }
        if matches!(
            declaration.transition.action,
            crate::Action::Suspend | crate::Action::ResumeSignal
        ) {
            return false;
        }
        if lifecycle_target_for_profile(
            machine.lifecycle_profile,
            declaration.transition.from,
            declaration.transition.action,
        )
        .is_err()
        {
            return false;
        }
        let key = EffectKey {
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
        };
        expected_index
            .entry(key.clone())
            .or_default()
            .push(declaration.clone());
        expected_route_indices
            .entry(key)
            .or_default()
            .push(effect_index);
    }
    for declarations in expected_index.values() {
        if declarations.len() > machine.limits.max_intents_per_transition {
            return false;
        }
    }
    expected_index == machine.effect_index && expected_route_indices == machine.effect_route_indices
}

fn normalized_edges(machine: &AdmittedMachine) -> bool {
    let mut normalized = machine.edges.clone();
    normalized.sort();
    normalized.dedup();
    if normalized != machine.edges {
        return false;
    }
    for Edge { source, target } in &machine.edges {
        if source == target {
            return false;
        }
        if !machine.nodes.contains_key(source) || !machine.nodes.contains_key(target) {
            return false;
        }
    }
    true
}

fn suspension_declarations_match(machine: &AdmittedMachine) -> bool {
    let mut normalized = machine.suspension_declarations.clone();
    normalized.sort();
    normalized.dedup();
    if normalized != machine.suspension_declarations {
        return false;
    }
    if !machine.suspension_declarations.is_empty()
        && machine.lifecycle_profile != LifecycleProfile::DurableSuspensionV1
    {
        return false;
    }
    let mut expected_indices = BTreeMap::new();
    for (index, declaration) in machine.suspension_declarations.iter().enumerate() {
        if !machine.nodes.contains_key(&declaration.node_id) {
            return false;
        }
        if declaration.class.as_str().len() > machine.limits.max_identifier_bytes
            || declaration.reference.as_str().len() > machine.limits.max_reference_bytes
        {
            return false;
        }
        if expected_indices
            .insert(declaration.node_id.clone(), index)
            .is_some()
        {
            return false;
        }
    }
    expected_indices == machine.suspension_declaration_indices
}

/// Returns true when all immutable admitted-machine facts remain closed and normalized.
#[must_use]
pub fn well_formed(machine: &AdmittedMachine) -> bool {
    if !matches!(
        machine.lifecycle_profile,
        LifecycleProfile::LinearV1 | LifecycleProfile::DurableSuspensionV1
    ) {
        return false;
    }
    let effect_count = machine
        .effects
        .len()
        .saturating_add(machine.concrete_effects.len());
    if machine.nodes.len() > machine.limits.max_nodes
        || machine.edges.len() > machine.limits.max_edges
        || effect_count > machine.limits.max_effect_declarations
    {
        return false;
    }
    let expected_node_order = machine.nodes.keys().cloned().collect::<Vec<_>>();
    if expected_node_order != machine.node_order {
        return false;
    }
    let mut operation_count = 0usize;
    for (node_id, node) in &machine.nodes {
        if node_id.as_str().len() > machine.limits.max_identifier_bytes {
            return false;
        }
        operation_count = operation_count.saturating_add(node.operations.len());
        for operation in &node.operations {
            if operation.as_str().len() > machine.limits.max_identifier_bytes {
                return false;
            }
        }
    }
    if operation_count > machine.limits.max_operations {
        return false;
    }
    let is_effect_set_valid = match machine.schema {
        MachineSchema::V1 => {
            machine.concrete_effects.is_empty()
                && machine.concrete_effect_key_indices.is_empty()
                && machine.concrete_effect_route_indices.is_empty()
                && effects_match(machine)
        }
        MachineSchema::V2 => {
            machine.effects.is_empty()
                && machine.effect_index.is_empty()
                && machine.effect_route_indices.is_empty()
                && crate::concrete::concrete_effects_match(machine)
        }
    };
    normalized_edges(machine)
        && indexes_match(machine)
        && indexed_views_match(machine)
        && topology_matches(machine)
        && is_effect_set_valid
        && suspension_declarations_match(machine)
}

/// Returns deterministic state-invariant failures.
#[must_use]
pub fn validate_invariant(machine: &AdmittedMachine, state: &MachineState) -> Vec<InvariantIssue> {
    let mut issues = BTreeSet::new();
    for node_id in machine.nodes.keys() {
        if !state.entries().contains_key(node_id) {
            issues.insert(InvariantIssue::MissingNode(node_id.clone()));
        }
    }
    for node_id in state.entries().keys() {
        if !machine.nodes.contains_key(node_id) {
            issues.insert(InvariantIssue::ExtraNode(node_id.clone()));
        }
    }
    for (node_id, local_state) in state.entries() {
        if matches!(
            local_state,
            LocalState::Ready | LocalState::Running | LocalState::Completed | LocalState::Failed
        ) && !dependencies_complete(machine, state, node_id)
        {
            issues.insert(InvariantIssue::DependenciesIncomplete(node_id.clone()));
        }
        match local_state {
            LocalState::Suspended => {
                if machine.lifecycle_profile != LifecycleProfile::DurableSuspensionV1 {
                    issues.insert(InvariantIssue::SuspensionWithoutProfile(node_id.clone()));
                }
                if !state.suspensions().contains_key(node_id) {
                    issues.insert(InvariantIssue::SuspensionRecordMissing(node_id.clone()));
                }
            }
            _ => {
                if state.suspensions().contains_key(node_id) {
                    issues.insert(InvariantIssue::SuspensionRecordUnexpected(node_id.clone()));
                }
            }
        }
    }
    for node_id in state.suspensions().keys() {
        if !state.entries().contains_key(node_id) {
            issues.insert(InvariantIssue::SuspensionRecordUnexpected(node_id.clone()));
        }
    }
    issues.into_iter().collect()
}

/// Returns true when aggregate state exactly covers the admitted nodes and dependencies.
#[must_use]
pub fn invariant(machine: &AdmittedMachine, state: &MachineState) -> bool {
    validate_invariant(machine, state).is_empty()
}

/// Returns true when no terminal local state changes across an aggregate transition.
#[must_use]
pub fn terminal_states_preserved(before: &MachineState, after: &MachineState) -> bool {
    for (node_id, local_state) in before.entries() {
        if matches!(
            local_state,
            LocalState::Completed | LocalState::Failed | LocalState::Cancelled
        ) && after.state(node_id) != Some(*local_state)
        {
            return false;
        }
    }
    true
}

fn declaration_matches_intent(declaration: &EffectDeclaration, intent: &EffectIntent) -> bool {
    declaration.node_id == intent.node_id
        && declaration.transition == intent.transition
        && declaration.operation == intent.operation
        && declaration.resource == intent.resource
        && declaration.ability == intent.ability
        && declaration.request == intent.request
        && declaration.payload == intent.payload
}

fn request_matches_intent(request: &crate::EffectRequest, intent: &EffectIntent) -> bool {
    request.operation == intent.operation.as_str()
        && request.resource == intent.resource.as_str()
        && request.ability == intent.ability.as_str()
        && request.request == intent.request.as_str()
        && request.payload == intent.payload.as_str()
}

/// Returns true when every emitted intent exactly matches the event and admitted declaration.
#[must_use]
pub fn effects_allowed(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
    intents: &[EffectIntent],
) -> bool {
    if !invariant(machine, state) || intents.len() != event.effects.len() {
        return false;
    }
    let Ok(node_id) = crate::NodeId::parse(&event.node_id, machine.limits.max_identifier_bytes)
    else {
        return false;
    };
    let Some(local_state) = state.state(&node_id) else {
        return false;
    };
    let selector = TransitionSelector {
        from: local_state,
        action: event.action,
    };
    if lifecycle_target_for_profile(machine.lifecycle_profile, local_state, event.action).is_err() {
        return false;
    }

    let mut normalized = intents.to_vec();
    normalized.sort();
    normalized.dedup();
    if normalized != intents {
        return false;
    }

    let key = EffectKey {
        node_id,
        transition: selector,
    };
    let declarations = machine
        .effect_index
        .get(&key)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    for intent in intents {
        if !declarations
            .iter()
            .any(|declaration| declaration_matches_intent(declaration, intent))
        {
            return false;
        }
        if !event
            .effects
            .iter()
            .any(|request| request_matches_intent(request, intent))
        {
            return false;
        }
    }
    true
}
