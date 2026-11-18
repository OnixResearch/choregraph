use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::lifecycle::{StepAction, StepState, core_action, core_state};
use crate::topology::{
    Bottleneck, ExecutionGraph, ExecutionGroup, QueryError, parse_node_id, string_ids,
};

/// Resume decisions, including the stricter Choregraph failure classes.
///
/// `AwaitSignal` is observable only for durable-suspension machines. The
/// closed linear compatibility path never produces it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeDecision {
    Execute,
    AwaitDependencies,
    AwaitSignal,
    SkipCompleted,
    RejectTerminalMutation,
    BlockedByFailedDependency,
}

/// A checked resume conversion failure for new consumers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResumeDecisionError {
    UnknownNode,
    InvalidState(Vec<choregraph_core::model::outcome::InvariantIssue>),
}

/// A state conversion failure before a production decision call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateConversionError {
    InvalidNodeId,
    UnknownNode,
    InvalidState(Vec<choregraph_core::model::outcome::InvariantIssue>),
}
/// An aggregate machine decision with no effect-dispatch surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineDecision {
    Accept {
        next: choregraph_core::model::automaton::MachineState,
    },
    Deny {
        reason: choregraph_core::model::outcome::DenialReason,
    },
    AuthorityBoundaryViolation,
}
/// One admitted compatibility plan backed by the production Choregraph machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    pub(super) graph: ExecutionGraph,
    pub(super) machine: choregraph_core::model::automaton::AdmittedMachine,
    pub topological_order: Vec<String>,
}

impl ExecutionPlan {
    #[must_use]
    pub const fn graph(&self) -> &ExecutionGraph {
        &self.graph
    }

    #[must_use]
    pub const fn machine(&self) -> &choregraph_core::model::automaton::AdmittedMachine {
        &self.machine
    }

    /// Returns legacy-shaped dependencies and fails closed to no result on conversion failure.
    #[must_use]
    #[allow(
        clippy::manual_unwrap_or_default,
        reason = "explicit compatibility control flow keeps the fail-closed branch visible"
    )]
    pub fn dependencies(&self, node_id: &str) -> Vec<String> {
        match self.dependencies_checked(node_id) {
            Ok(results) => results,
            Err(_) => Vec::new(),
        }
    }

    /// Returns dependencies with explicit conversion failures for new consumers.
    pub fn dependencies_checked(&self, node_id: &str) -> Result<Vec<String>, QueryError> {
        let node_id = parse_node_id(&self.machine, node_id)?;
        self.machine
            .dependencies(&node_id)
            .map(string_ids)
            .ok_or(QueryError::UnknownNode)
    }

    /// Returns legacy-shaped dependents and fails closed to no result on conversion failure.
    #[must_use]
    #[allow(
        clippy::manual_unwrap_or_default,
        reason = "explicit compatibility control flow keeps the fail-closed branch visible"
    )]
    pub fn dependents(&self, node_id: &str) -> Vec<String> {
        match self.dependents_checked(node_id) {
            Ok(results) => results,
            Err(_) => Vec::new(),
        }
    }

    /// Returns dependents with explicit conversion failures for new consumers.
    pub fn dependents_checked(&self, node_id: &str) -> Result<Vec<String>, QueryError> {
        let node_id = parse_node_id(&self.machine, node_id)?;
        self.machine
            .dependents(&node_id)
            .map(string_ids)
            .ok_or(QueryError::UnknownNode)
    }

    /// Returns legacy-shaped groups and fails closed to no result on conversion failure.
    #[must_use]
    #[allow(
        clippy::manual_unwrap_or_default,
        reason = "explicit compatibility control flow keeps the fail-closed branch visible"
    )]
    pub fn parallel_groups(&self) -> Vec<ExecutionGroup> {
        match self.parallel_groups_checked() {
            Ok(results) => results,
            Err(_) => Vec::new(),
        }
    }

    /// Returns groups with explicit conversion failures for new consumers.
    pub fn parallel_groups_checked(&self) -> Result<Vec<ExecutionGroup>, QueryError> {
        choregraph_core::analytics::parallel_groups(&self.machine)
            .into_iter()
            .map(|group| {
                Ok(ExecutionGroup {
                    node_ids: string_ids(&group.node_ids),
                    level: u32::try_from(group.level)
                        .map_err(|_| QueryError::LevelOutsideProfile)?,
                    can_parallelize: group.can_parallelize,
                })
            })
            .collect()
    }

    #[must_use]
    pub fn critical_path(&self) -> Vec<String> {
        string_ids(&choregraph_core::analytics::critical_path(&self.machine))
    }

    /// Returns legacy-shaped bottlenecks and fails closed to no result on conversion failure.
    #[must_use]
    #[allow(
        clippy::manual_unwrap_or_default,
        reason = "explicit compatibility control flow keeps the fail-closed branch visible"
    )]
    pub fn bottlenecks(&self) -> Vec<Bottleneck> {
        match self.bottlenecks_checked() {
            Ok(results) => results,
            Err(_) => Vec::new(),
        }
    }

    /// Returns bottlenecks with explicit conversion failures for new consumers.
    pub fn bottlenecks_checked(&self) -> Result<Vec<Bottleneck>, QueryError> {
        choregraph_core::analytics::bottlenecks(&self.machine)
            .into_iter()
            .map(|item| {
                Ok(Bottleneck {
                    node_id: item.node_id.as_str().to_string(),
                    dependent_count: u32::try_from(item.dependent_count)
                        .map_err(|_| QueryError::CountOutsideProfile)?,
                })
            })
            .collect()
    }
}
/// Expands a sparse legacy state map to the exact Choregraph aggregate state.
pub fn expand_state(
    plan: &ExecutionPlan,
    sparse_states: &BTreeMap<String, StepState>,
) -> Result<choregraph_core::model::automaton::MachineState, StateConversionError> {
    let mut entries = plan.machine.initial_state().entries().clone();
    debug_assert_eq!(entries.len(), plan.machine.node_ids().len());
    for (raw_node_id, state) in sparse_states {
        let node_id = choregraph_core::id::bounded::NodeId::parse(
            raw_node_id,
            plan.machine.limits().max_identifier_bytes,
        )
        .map_err(|_| StateConversionError::InvalidNodeId)?;
        let Some(entry) = entries.get_mut(&node_id) else {
            return Err(StateConversionError::UnknownNode);
        };
        *entry = core_state(*state);
    }
    debug_assert!(
        plan.machine
            .node_ids()
            .iter()
            .all(|node_id| entries.contains_key(node_id))
    );
    debug_assert!(
        entries
            .values()
            .all(|value| { *value != choregraph_core::model::semantics::LocalState::Suspended })
    );
    let state = choregraph_core::model::automaton::MachineState::new(entries);
    let issues = choregraph_core::invariant::validate_invariant(&plan.machine, &state);
    if !issues.is_empty() {
        return Err(StateConversionError::InvalidState(issues));
    }
    Ok(state)
}

/// Returns legacy-shaped ready nodes and fails closed to no result on invalid state input.
#[must_use]
#[expect(
    clippy::manual_unwrap_or_default,
    reason = "explicit compatibility control flow keeps the fail-closed branch visible"
)]
pub fn ready_nodes(
    plan: &ExecutionPlan,
    sparse_states: &BTreeMap<String, StepState>,
) -> Vec<String> {
    match ready_nodes_checked(plan, sparse_states) {
        Ok(results) => results,
        Err(_) => Vec::new(),
    }
}

/// Returns ready nodes with explicit state conversion failures for new consumers.
pub fn ready_nodes_checked(
    plan: &ExecutionPlan,
    sparse_states: &BTreeMap<String, StepState>,
) -> Result<Vec<String>, StateConversionError> {
    let state = expand_state(plan, sparse_states)?;
    let ready = choregraph_core::analytics::ready_nodes(&plan.machine, &state)
        .map_err(StateConversionError::InvalidState)?;
    Ok(string_ids(&ready))
}

/// Returns one legacy-shaped resume decision and rejects conversion failures as terminal mutation.
#[must_use]
pub fn resume_decision(
    plan: &ExecutionPlan,
    sparse_states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> ResumeDecision {
    match resume_decision_checked(plan, sparse_states, node_id) {
        Ok(decision) => decision,
        Err(_) => ResumeDecision::RejectTerminalMutation,
    }
}

/// Returns one strict resume decision with explicit conversion failures for new consumers.
#[allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    reason = "the public core decision vocabulary requires explicit fail-closed compatibility mapping"
)]
pub fn resume_decision_checked(
    plan: &ExecutionPlan,
    sparse_states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> Result<ResumeDecision, ResumeDecisionError> {
    debug_assert!(choregraph_core::invariant::well_formed(&plan.machine));
    debug_assert_eq!(plan.machine.node_ids().len(), plan.graph.nodes.len());
    let state = expand_state(plan, sparse_states).map_err(|error| match error {
        StateConversionError::InvalidState(issues) => ResumeDecisionError::InvalidState(issues),
        StateConversionError::InvalidNodeId | StateConversionError::UnknownNode => {
            ResumeDecisionError::UnknownNode
        }
    })?;
    let node_id =
        parse_node_id(&plan.machine, node_id).map_err(|_| ResumeDecisionError::UnknownNode)?;
    match choregraph_core::analytics::resume_decision(&plan.machine, &state, &node_id) {
        choregraph_core::model::analysis::ResumeDecision::Execute => Ok(ResumeDecision::Execute),
        choregraph_core::model::analysis::ResumeDecision::AwaitDependencies => {
            Ok(ResumeDecision::AwaitDependencies)
        }
        choregraph_core::model::analysis::ResumeDecision::SkipCompleted => {
            Ok(ResumeDecision::SkipCompleted)
        }
        choregraph_core::model::analysis::ResumeDecision::RejectTerminalMutation => {
            Ok(ResumeDecision::RejectTerminalMutation)
        }
        choregraph_core::model::analysis::ResumeDecision::BlockedByFailedDependency => {
            Ok(ResumeDecision::BlockedByFailedDependency)
        }
        choregraph_core::model::analysis::ResumeDecision::AwaitSignal => {
            Ok(ResumeDecision::AwaitSignal)
        }
        choregraph_core::model::analysis::ResumeDecision::UnknownNode => {
            Err(ResumeDecisionError::UnknownNode)
        }
        choregraph_core::model::analysis::ResumeDecision::InvalidState => {
            Err(ResumeDecisionError::InvalidState(Vec::new()))
        }
    }
}

/// Applies one aggregate event through production `step` and exposes no effect intent.
#[must_use]
pub fn step_machine(
    plan: &ExecutionPlan,
    state: &choregraph_core::model::automaton::MachineState,
    node_id: &str,
    action: StepAction,
) -> MachineDecision {
    let event = choregraph_core::model::dispatch::Event::new(node_id, core_action(action));
    match choregraph_core::transition::step(&plan.machine, state, &event) {
        choregraph_core::model::outcome::Decision::Accept { next, intents }
            if intents.is_empty() =>
        {
            MachineDecision::Accept { next }
        }
        choregraph_core::model::outcome::Decision::Accept { .. } => {
            MachineDecision::AuthorityBoundaryViolation
        }
        choregraph_core::model::outcome::Decision::Deny { reason } => {
            MachineDecision::Deny { reason }
        }
    }
}
