// Graph analytics retain deterministic collection order and reviewed compatibility signatures.
#![allow(
    unknown_lints,
    assertion_density,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    unbounded_collection_growth,
    reason = "graph analytics retain deterministic collection order and reviewed compatibility signatures"
)]

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::NodeId;
use crate::invariant::{invariant, validate_invariant};
use crate::model::{
    AdmittedMachine, Bottleneck, InvariantIssue, LocalState, MachineState, ParallelGroup,
    ResumeDecision,
};
use crate::verified::{
    STATE_CANCELLED, STATE_COMPLETED, STATE_FAILED, STATE_PENDING, STATE_READY, STATE_RUNNING,
    STATE_SUSPENDED,
};

const ROOT_LEVEL: usize = 0;
const NEXT_LEVEL_INCREMENT: usize = 1;
const MINIMUM_PARALLEL_NODE_COUNT: usize = 2;

const fn local_state_code(state: LocalState) -> u8 {
    match state {
        LocalState::Pending => STATE_PENDING,
        LocalState::Ready => STATE_READY,
        LocalState::Running => STATE_RUNNING,
        LocalState::Completed => STATE_COMPLETED,
        LocalState::Failed => STATE_FAILED,
        LocalState::Cancelled => STATE_CANCELLED,
        LocalState::Suspended => STATE_SUSPENDED,
    }
}

fn indexed_states(machine: &AdmittedMachine, state: &MachineState) -> Vec<u8> {
    machine
        .node_order
        .iter()
        .map(|node_id| {
            state
                .state(node_id)
                .map_or(STATE_CANCELLED, local_state_code)
        })
        .collect()
}

fn dependencies_complete(
    machine: &AdmittedMachine,
    state: &MachineState,
    node_id: &NodeId,
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

fn has_failed_dependency(
    machine: &AdmittedMachine,
    state: &MachineState,
    node_id: &NodeId,
) -> bool {
    let Some(dependencies) = machine.dependencies.get(node_id) else {
        return false;
    };
    for dependency in dependencies {
        if matches!(
            state.state(dependency),
            Some(LocalState::Failed | LocalState::Cancelled)
        ) {
            return true;
        }
    }
    false
}

/// Returns pending nodes whose direct dependencies are complete.
// r[impl choregraph.proof.single_step.readiness_completeness]
// r[impl choregraph.proof.single_step.readiness_frontier]
pub fn ready_nodes(
    machine: &AdmittedMachine,
    state: &MachineState,
) -> Result<Vec<NodeId>, Vec<InvariantIssue>> {
    let issues = validate_invariant(machine, state);
    if !issues.is_empty() {
        return Err(issues);
    }
    let states = indexed_states(machine, state);
    let ready_indices = crate::verified::ready_frontier(
        &machine.topological_indices,
        &machine.dependency_indices,
        &states,
    );
    let mut ready = Vec::with_capacity(ready_indices.len());
    for node_index in ready_indices {
        ready.push(machine.node_order[node_index].clone());
    }
    Ok(ready)
}

/// Returns one deterministic resume decision for an admitted node.
#[must_use]
pub fn resume_decision(
    machine: &AdmittedMachine,
    state: &MachineState,
    node_id: &NodeId,
) -> ResumeDecision {
    if !invariant(machine, state) {
        return ResumeDecision::InvalidState;
    }
    let Some(local_state) = state.state(node_id) else {
        return ResumeDecision::UnknownNode;
    };
    match local_state {
        LocalState::Completed => ResumeDecision::SkipCompleted,
        LocalState::Failed | LocalState::Cancelled => ResumeDecision::RejectTerminalMutation,
        LocalState::Suspended => ResumeDecision::AwaitSignal,
        LocalState::Pending | LocalState::Ready | LocalState::Running => {
            if has_failed_dependency(machine, state, node_id) {
                ResumeDecision::BlockedByFailedDependency
            } else if dependencies_complete(machine, state, node_id) {
                ResumeDecision::Execute
            } else {
                ResumeDecision::AwaitDependencies
            }
        }
    }
}

/// Groups admitted nodes by their deterministic dependency level.
#[must_use]
pub fn parallel_groups(machine: &AdmittedMachine) -> Vec<ParallelGroup> {
    let mut levels = machine
        .node_order
        .iter()
        .cloned()
        .map(|node_id| (node_id, ROOT_LEVEL))
        .collect::<BTreeMap<_, _>>();

    for node_id in &machine.topological_order {
        let Some(level) = levels.get(node_id).copied() else {
            continue;
        };
        let Some(dependents) = machine.dependents.get(node_id) else {
            continue;
        };
        for dependent in dependents {
            let candidate = level.saturating_add(NEXT_LEVEL_INCREMENT);
            if let Some(current) = levels.get_mut(dependent) {
                *current = (*current).max(candidate);
            }
        }
    }

    let mut grouped = BTreeMap::<usize, Vec<NodeId>>::new();
    for node_id in &machine.topological_order {
        let Some(level) = levels.get(node_id).copied() else {
            continue;
        };
        grouped.entry(level).or_default().push(node_id.clone());
    }
    grouped
        .into_iter()
        .map(|(level, node_ids)| ParallelGroup {
            can_parallelize: node_ids.len() >= MINIMUM_PARALLEL_NODE_COUNT,
            node_ids,
            level,
        })
        .collect()
}

fn path_precedes(candidate: &[NodeId], current: &[NodeId]) -> bool {
    match candidate.len().cmp(&current.len()) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => candidate < current,
    }
}

/// Returns one deterministic longest dependency path.
#[must_use]
pub fn critical_path(machine: &AdmittedMachine) -> Vec<NodeId> {
    let mut paths = BTreeMap::<NodeId, Vec<NodeId>>::new();
    for node_id in &machine.topological_order {
        let mut best = alloc::vec![node_id.clone()];
        let Some(dependencies) = machine.dependencies.get(node_id) else {
            continue;
        };
        for dependency in dependencies {
            let Some(prefix) = paths.get(dependency) else {
                continue;
            };
            let mut candidate = prefix.clone();
            candidate.push(node_id.clone());
            if path_precedes(&candidate, &best) {
                best = candidate;
            }
        }
        paths.insert(node_id.clone(), best);
    }

    let mut best = Vec::new();
    for candidate in paths.into_values() {
        if path_precedes(&candidate, &best) {
            best = candidate;
        }
    }
    best
}

fn transitive_dependent_count(machine: &AdmittedMachine, node_id: &NodeId) -> usize {
    let mut visited = BTreeSet::new();
    let mut pending = machine.dependents.get(node_id).cloned().unwrap_or_default();
    let maximum_visits = machine.nodes.len();
    let mut visits = 0usize;

    while let Some(candidate) = pending.pop() {
        if !visited.insert(candidate.clone()) {
            continue;
        }
        visits = visits.saturating_add(NEXT_LEVEL_INCREMENT);
        if visits > maximum_visits {
            break;
        }
        if let Some(dependents) = machine.dependents.get(&candidate) {
            pending.extend(dependents.iter().cloned());
        }
    }
    visited.len()
}

/// Returns whether adding `source -> target` would create a cycle.
#[must_use]
pub fn would_create_cycle(machine: &AdmittedMachine, source: &NodeId, target: &NodeId) -> bool {
    if source == target {
        return true;
    }
    if !machine.nodes.contains_key(source) || !machine.nodes.contains_key(target) {
        return false;
    }
    let mut visited = BTreeSet::new();
    let mut pending = alloc::vec![target.clone()];
    while let Some(candidate) = pending.pop() {
        if &candidate == source {
            return true;
        }
        if !visited.insert(candidate.clone()) {
            continue;
        }
        if visited.len() > machine.nodes.len() {
            return true;
        }
        if let Some(dependents) = machine.dependents.get(&candidate) {
            pending.extend(dependents.iter().cloned());
        }
    }
    false
}

/// Ranks nodes by transitive dependent count, then identifier.
#[must_use]
pub fn bottlenecks(machine: &AdmittedMachine) -> Vec<Bottleneck> {
    let mut result = machine
        .topological_order
        .iter()
        .cloned()
        .map(|node_id| Bottleneck {
            dependent_count: transitive_dependent_count(machine, &node_id),
            node_id,
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| {
        right
            .dependent_count
            .cmp(&left.dependent_count)
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    result
}
