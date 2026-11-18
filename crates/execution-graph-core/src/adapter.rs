// This adapter preserves the published compatibility API while delegating all decisions.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    explicit_defaults,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    renamed_imports,
    reason = "the compatibility adapter keeps published names and exact legacy mappings"
)]

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use choregraph_core::{
    id::bounded::NodeId, model::analysis::ResumeDecision as ChoregraphResumeDecision,
    model::automaton::AdmittedMachine, model::automaton::MachineSpec,
    model::automaton::MachineState, model::capacity::AdmissionLimits, model::graph::EdgeSpec,
    model::graph::NodeSpec, model::outcome::AdmissionDiagnostic as ChoregraphDiagnostic,
    model::outcome::AdmissionDiagnosticCode as ChoregraphDiagnosticCode,
    model::semantics::Action as ChoregraphAction, model::semantics::LocalState,
};

use crate::admission::{Diagnostic, DiagnosticCode};
use crate::aggregate::ResumeDecision;
use crate::lifecycle::{StepAction, StepState, StepTransitionError};
use crate::structure::{Bottleneck, Group, Topology};

pub(crate) fn machine_spec(graph: &Topology) -> MachineSpec {
    let nodes = graph
        .nodes
        .iter()
        .map(|node| NodeSpec::new(node.id.clone(), Vec::new()))
        .collect();
    let edges = graph
        .edges
        .iter()
        .map(|edge| EdgeSpec::new(edge.source.clone(), edge.target.clone()))
        .collect();
    MachineSpec::new(nodes, edges)
}

pub(crate) fn admit_machine(graph: &Topology) -> Result<AdmittedMachine, Vec<Diagnostic>> {
    AdmittedMachine::admit(machine_spec(graph), AdmissionLimits::default()).map_err(map_diagnostics)
}

fn diagnostic_code(code: ChoregraphDiagnosticCode) -> DiagnosticCode {
    match code {
        ChoregraphDiagnosticCode::EmptyIdentifier
        | ChoregraphDiagnosticCode::IdentifierTooLong
        | ChoregraphDiagnosticCode::MalformedIdentifier => DiagnosticCode::EmptyNodeId,
        ChoregraphDiagnosticCode::DuplicateNode => DiagnosticCode::DuplicateNodeId,
        ChoregraphDiagnosticCode::DuplicateEdge => DiagnosticCode::DuplicateEdge,
        ChoregraphDiagnosticCode::MissingSource => DiagnosticCode::MissingSourceNode,
        ChoregraphDiagnosticCode::MissingTarget => DiagnosticCode::MissingTargetNode,
        ChoregraphDiagnosticCode::SelfLoop => DiagnosticCode::SelfLoop,
        ChoregraphDiagnosticCode::CycleDetected => DiagnosticCode::CycleDetected,
        ChoregraphDiagnosticCode::InconsistentLimits
        | ChoregraphDiagnosticCode::TooManyNodes
        | ChoregraphDiagnosticCode::TooManyEdges
        | ChoregraphDiagnosticCode::TooManyOperations
        | ChoregraphDiagnosticCode::TooManyEffectDeclarations
        | ChoregraphDiagnosticCode::UnsupportedSchema
        | ChoregraphDiagnosticCode::UnsupportedLifecycleProfile
        | ChoregraphDiagnosticCode::DuplicateOperation
        | ChoregraphDiagnosticCode::EffectNodeMissing
        | ChoregraphDiagnosticCode::EffectOperationUndeclared
        | ChoregraphDiagnosticCode::InvalidEffectTransition
        | ChoregraphDiagnosticCode::DuplicateEffectDeclaration
        | ChoregraphDiagnosticCode::DuplicateDeclarationKey
        | ChoregraphDiagnosticCode::DuplicateConcreteEffectDeclaration
        | ChoregraphDiagnosticCode::DuplicateSuspensionDeclaration
        | ChoregraphDiagnosticCode::SuspensionDeclarationNodeMissing
        | ChoregraphDiagnosticCode::SuspensionDeclarationWithoutProfile
        | ChoregraphDiagnosticCode::MalformedSuspensionClass
        | ChoregraphDiagnosticCode::MalformedSuspensionReference
        | ChoregraphDiagnosticCode::SuspensionEffectTransition
        | ChoregraphDiagnosticCode::UnknownReferenceProfile
        | ChoregraphDiagnosticCode::MalformedBinding
        | ChoregraphDiagnosticCode::MalformedReference
        | ChoregraphDiagnosticCode::ReferenceTooLong
        | ChoregraphDiagnosticCode::IntentTooLarge
        | ChoregraphDiagnosticCode::TooManyIntentsForTransition
        | ChoregraphDiagnosticCode::NormalizationInvariant => DiagnosticCode::GraphTooLarge,
    }
}

fn diagnostic_message(code: DiagnosticCode) -> &'static str {
    match code {
        DiagnosticCode::EmptyNodeId => "execution graph node id is invalid",
        DiagnosticCode::DuplicateNodeId => "execution graph node id must be unique",
        DiagnosticCode::DuplicateEdge => "execution graph edge must be unique",
        DiagnosticCode::MissingSourceNode => "execution graph edge source node is missing",
        DiagnosticCode::MissingTargetNode => "execution graph edge target node is missing",
        DiagnosticCode::SelfLoop => "execution graph edge cannot connect a node to itself",
        DiagnosticCode::CycleDetected => "execution graph contains a cycle",
        DiagnosticCode::GraphTooLarge => "execution graph exceeds the supported bounds",
    }
}

fn map_diagnostic(diagnostic: ChoregraphDiagnostic) -> Diagnostic {
    let code = diagnostic_code(diagnostic.code);
    Diagnostic {
        code,
        node_id: diagnostic.subject,
        related_node_id: diagnostic.related,
        message: diagnostic_message(code).to_string(),
    }
}

fn map_diagnostics(diagnostics: Vec<ChoregraphDiagnostic>) -> Vec<Diagnostic> {
    diagnostics.into_iter().map(map_diagnostic).collect()
}

pub(crate) fn node_strings(nodes: &[NodeId]) -> Vec<String> {
    nodes.iter().map(ToString::to_string).collect()
}

pub(crate) fn dependencies(machine: &AdmittedMachine, node_id: &str) -> Vec<String> {
    let Some(node_id) = parse_node(machine, node_id) else {
        return Vec::new();
    };
    machine
        .dependencies(&node_id)
        .map(node_strings)
        .unwrap_or_default()
}

pub(crate) fn dependents(machine: &AdmittedMachine, node_id: &str) -> Vec<String> {
    let Some(node_id) = parse_node(machine, node_id) else {
        return Vec::new();
    };
    machine
        .dependents(&node_id)
        .map(node_strings)
        .unwrap_or_default()
}

pub(crate) fn parallel_groups(machine: &AdmittedMachine) -> Vec<Group> {
    choregraph_core::analytics::parallel_groups(machine)
        .into_iter()
        .map(|group| Group {
            node_ids: node_strings(&group.node_ids),
            level: level_to_u32(group.level),
            can_parallelize: group.can_parallelize,
        })
        .collect()
}

// The explicit branch makes the fail-closed saturation boundary visible.
#[allow(
    clippy::manual_unwrap_or,
    reason = "the explicit branch documents the fail-closed saturation boundary"
)]
fn level_to_u32(level: usize) -> u32 {
    match u32::try_from(level) {
        Ok(level) => level,
        Err(_) => u32::MAX,
    }
}

pub(crate) fn critical_path(machine: &AdmittedMachine) -> Vec<String> {
    node_strings(&choregraph_core::analytics::critical_path(machine))
}

pub(crate) fn bottlenecks(machine: &AdmittedMachine) -> Vec<Bottleneck> {
    choregraph_core::analytics::bottlenecks(machine)
        .into_iter()
        .map(|bottleneck| Bottleneck {
            node_id: bottleneck.node_id.to_string(),
            dependent_count: level_to_u32(bottleneck.dependent_count),
        })
        .collect()
}

fn to_local_state(state: StepState) -> LocalState {
    match state {
        StepState::Pending => LocalState::Pending,
        StepState::Ready => LocalState::Ready,
        StepState::Running => LocalState::Running,
        StepState::Completed => LocalState::Completed,
        StepState::Failed => LocalState::Failed,
        StepState::Cancelled => LocalState::Cancelled,
    }
}

fn from_local_state(state: LocalState) -> Option<StepState> {
    match state {
        LocalState::Pending => Some(StepState::Pending),
        LocalState::Ready => Some(StepState::Ready),
        LocalState::Running => Some(StepState::Running),
        LocalState::Completed => Some(StepState::Completed),
        LocalState::Failed => Some(StepState::Failed),
        LocalState::Cancelled => Some(StepState::Cancelled),
        LocalState::Suspended => None,
    }
}

fn to_action(action: StepAction) -> ChoregraphAction {
    match action {
        StepAction::MarkReady => ChoregraphAction::MarkReady,
        StepAction::Start => ChoregraphAction::Start,
        StepAction::Complete => ChoregraphAction::Complete,
        StepAction::Fail => ChoregraphAction::Fail,
        StepAction::Cancel => ChoregraphAction::Cancel,
    }
}

pub(crate) fn transition(
    from: StepState,
    action: StepAction,
) -> Result<StepState, StepTransitionError> {
    match choregraph_core::transition::lifecycle_target(to_local_state(from), to_action(action)) {
        Ok(target) => {
            from_local_state(target).ok_or(StepTransitionError::InvalidTransition { from, action })
        }
        Err(choregraph_core::model::semantics::LocalTransitionError::Terminal { .. }) => {
            Err(StepTransitionError::TerminalState { from, action })
        }
        Err(choregraph_core::model::semantics::LocalTransitionError::Invalid { .. }) => {
            Err(StepTransitionError::InvalidTransition { from, action })
        }
    }
}

pub(crate) fn is_terminal(state: StepState) -> bool {
    matches!(
        choregraph_core::transition::lifecycle_target(
            to_local_state(state),
            ChoregraphAction::Cancel
        ),
        Err(choregraph_core::model::semantics::LocalTransitionError::Terminal { .. })
    )
}

fn machine_state(machine: &AdmittedMachine, states: &BTreeMap<String, StepState>) -> MachineState {
    let entries = machine
        .node_ids()
        .iter()
        .cloned()
        .map(|node_id| {
            let state = states
                .get(node_id.as_str())
                .copied()
                .unwrap_or(StepState::Pending);
            (node_id, to_local_state(state))
        })
        .collect();
    MachineState::new(entries)
}

pub(crate) fn ready_nodes(
    machine: &AdmittedMachine,
    states: &BTreeMap<String, StepState>,
) -> Vec<String> {
    let state = machine_state(machine, states);
    match choregraph_core::analytics::ready_nodes(machine, &state) {
        Ok(nodes) => node_strings(&nodes),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn resume_decision(
    machine: &AdmittedMachine,
    states: &BTreeMap<String, StepState>,
    node_id: &str,
) -> ResumeDecision {
    let Some(node_id) = parse_node(machine, node_id) else {
        return ResumeDecision::AwaitDependencies;
    };
    let state = machine_state(machine, states);
    match choregraph_core::analytics::resume_decision(machine, &state, &node_id) {
        ChoregraphResumeDecision::Execute => ResumeDecision::Execute,
        ChoregraphResumeDecision::AwaitDependencies | ChoregraphResumeDecision::AwaitSignal => {
            ResumeDecision::AwaitDependencies
        }
        ChoregraphResumeDecision::SkipCompleted => ResumeDecision::SkipCompleted,
        ChoregraphResumeDecision::RejectTerminalMutation => ResumeDecision::RejectTerminalMutation,
        ChoregraphResumeDecision::BlockedByFailedDependency => {
            ResumeDecision::BlockedByFailedDependency
        }
        ChoregraphResumeDecision::UnknownNode | ChoregraphResumeDecision::InvalidState => {
            ResumeDecision::AwaitDependencies
        }
    }
}

pub(crate) fn would_create_cycle(machine: &AdmittedMachine, source: &str, target: &str) -> bool {
    let maximum_bytes = machine.limits().max_identifier_bytes;
    let (Ok(source), Ok(target)) = (
        NodeId::parse(source, maximum_bytes),
        NodeId::parse(target, maximum_bytes),
    ) else {
        return false;
    };
    choregraph_core::analytics::would_create_cycle(machine, &source, &target)
}

fn parse_node(machine: &AdmittedMachine, node_id: &str) -> Option<NodeId> {
    let node_id = NodeId::parse(node_id, machine.limits().max_identifier_bytes).ok()?;
    machine
        .node_ids()
        .binary_search(&node_id)
        .ok()
        .map(|_| node_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_state_mapping_accepts_linear_states_and_rejects_suspension() {
        assert_eq!(from_local_state(LocalState::Ready), Some(StepState::Ready));
        assert_eq!(from_local_state(LocalState::Suspended), None);
    }

    #[test]
    fn suspension_admission_diagnostics_map_to_a_bounded_compatibility_failure() {
        assert_eq!(
            diagnostic_code(ChoregraphDiagnosticCode::MalformedSuspensionReference),
            DiagnosticCode::GraphTooLarge
        );
    }
}
