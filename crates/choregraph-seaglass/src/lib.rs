#![no_std]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "legacy-shaped DTO names remain concise while owner paths stay explicit at conversion boundaries"
)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

const LEGACY_EMPTY_NODE_MESSAGE: &str = "execution graph node id cannot be empty";
const LEGACY_DUPLICATE_NODE_MESSAGE: &str = "execution graph node id must be unique";
const LEGACY_DUPLICATE_EDGE_MESSAGE: &str = "execution graph edge must be unique";
const LEGACY_MISSING_SOURCE_MESSAGE: &str = "execution graph edge source node is missing";
const LEGACY_MISSING_TARGET_MESSAGE: &str = "execution graph edge target node is missing";
const LEGACY_SELF_LOOP_MESSAGE: &str = "execution graph edge cannot connect a node to itself";
const LEGACY_CYCLE_MESSAGE: &str = "execution graph contains a cycle";
const LEGACY_GRAPH_TOO_LARGE_MESSAGE: &str =
    "execution graph has more nodes than the supported level and analytics range";

/// A possible implementation mode for the Seaglass compatibility surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityMode {
    DelegateToChoregraphCore,
    IndependentTransitionImplementation,
}

/// One legacy-shaped node declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionNode {
    pub id: String,
}

impl ExecutionNode {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

/// One legacy-shaped dependency edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExecutionEdge {
    pub source: String,
    pub target: String,
}

impl ExecutionEdge {
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

/// One untrusted compatibility graph.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExecutionGraph {
    pub nodes: Vec<ExecutionNode>,
    pub edges: Vec<ExecutionEdge>,
}

impl ExecutionGraph {
    #[must_use]
    pub const fn new(nodes: Vec<ExecutionNode>, edges: Vec<ExecutionEdge>) -> Self {
        Self { nodes, edges }
    }
}

/// Stable compatibility admission categories.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GraphDiagnosticCode {
    EmptyNodeId,
    DuplicateNodeId,
    DuplicateEdge,
    MissingSourceNode,
    MissingTargetNode,
    SelfLoop,
    CycleDetected,
    GraphTooLarge,
}

/// One stable compatibility admission diagnostic.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GraphDiagnostic {
    pub code: GraphDiagnosticCode,
    pub node_id: Option<String>,
    pub related_node_id: Option<String>,
    pub message: String,
}

impl core::fmt::Display for GraphDiagnostic {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// One converted graph level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionGroup {
    pub node_ids: Vec<String>,
    pub level: u32,
    pub can_parallelize: bool,
}

/// One converted bottleneck result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bottleneck {
    pub node_id: String,
    pub dependent_count: u32,
}

/// Legacy lifecycle states mapped to Choregraph production states.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StepState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Legacy lifecycle actions mapped to Choregraph production actions.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum StepAction {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
}

/// Legacy local-transition error categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepTransitionError {
    InvalidTransition { from: StepState, action: StepAction },
    TerminalState { from: StepState, action: StepAction },
}

/// Resume decisions, including the stricter Choregraph failure classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeDecision {
    Execute,
    AwaitDependencies,
    SkipCompleted,
    RejectTerminalMutation,
    BlockedByFailedDependency,
}

/// A checked resume conversion failure for new consumers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResumeDecisionError {
    UnknownNode,
    InvalidState(Vec<choregraph_core::InvariantIssue>),
}

/// A state conversion failure before a production decision call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateConversionError {
    InvalidNodeId,
    UnknownNode,
    InvalidState(Vec<choregraph_core::InvariantIssue>),
}

/// A graph-query conversion failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryError {
    InvalidNodeId,
    UnknownNode,
    LevelOutsideProfile,
    CountOutsideProfile,
}

/// Named inputs for one proposed-edge cycle probe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CycleProbe<'a> {
    pub graph: &'a ExecutionGraph,
    pub source: &'a str,
    pub target: &'a str,
}

/// An aggregate machine decision with no effect-dispatch surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MachineDecision {
    Accept {
        next: choregraph_core::MachineState,
    },
    Deny {
        reason: choregraph_core::DenialReason,
    },
    AuthorityBoundaryViolation,
}

/// One admitted compatibility plan backed by the production Choregraph machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    graph: ExecutionGraph,
    machine: choregraph_core::AdmittedMachine,
    pub topological_order: Vec<String>,
}

impl ExecutionPlan {
    #[must_use]
    pub const fn graph(&self) -> &ExecutionGraph {
        &self.graph
    }

    #[must_use]
    pub const fn machine(&self) -> &choregraph_core::AdmittedMachine {
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
        choregraph_core::parallel_groups(&self.machine)
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
        string_ids(&choregraph_core::critical_path(&self.machine))
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
        choregraph_core::bottlenecks(&self.machine)
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

/// Returns the core capability that the adapter delegates to.
#[must_use]
pub const fn delegated_capability() -> choregraph_core::Capability {
    choregraph_core::Capability::DeterministicDecision
}

/// Returns true only for a conversion-only delegate.
#[must_use]
pub const fn accepts(mode: CompatibilityMode) -> bool {
    matches!(mode, CompatibilityMode::DelegateToChoregraphCore)
}

const fn compatibility_limits() -> choregraph_core::AdmissionLimits {
    choregraph_core::AdmissionLimits {
        max_nodes: choregraph_core::DEFAULT_MAX_NODES,
        max_edges: choregraph_core::DEFAULT_MAX_EDGES,
        max_operations: choregraph_core::DEFAULT_MAX_OPERATIONS,
        max_effect_declarations: choregraph_core::DEFAULT_MAX_EFFECT_DECLARATIONS,
        max_identifier_bytes: choregraph_core::DEFAULT_MAX_IDENTIFIER_BYTES,
        max_reference_bytes: choregraph_core::DEFAULT_MAX_REFERENCE_BYTES,
        max_event_bytes: choregraph_core::DEFAULT_MAX_EVENT_BYTES,
        max_intent_bytes: choregraph_core::DEFAULT_MAX_INTENT_BYTES,
        max_intents_per_transition: choregraph_core::DEFAULT_MAX_INTENTS_PER_TRANSITION,
    }
}

fn machine_spec(graph: &ExecutionGraph) -> choregraph_core::MachineSpec {
    let nodes = graph
        .nodes
        .iter()
        .map(|node| choregraph_core::NodeSpec::new(node.id.clone(), Vec::new()))
        .collect();
    let edges = graph
        .edges
        .iter()
        .map(|edge| choregraph_core::EdgeSpec::new(edge.source.clone(), edge.target.clone()))
        .collect();
    choregraph_core::MachineSpec::new(nodes, edges)
}

#[allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    reason = "the closed compatibility diagnostic vocabulary requires an explicit message for every class"
)]
const fn diagnostic_message(code: GraphDiagnosticCode) -> &'static str {
    match code {
        GraphDiagnosticCode::EmptyNodeId => LEGACY_EMPTY_NODE_MESSAGE,
        GraphDiagnosticCode::DuplicateNodeId => LEGACY_DUPLICATE_NODE_MESSAGE,
        GraphDiagnosticCode::DuplicateEdge => LEGACY_DUPLICATE_EDGE_MESSAGE,
        GraphDiagnosticCode::MissingSourceNode => LEGACY_MISSING_SOURCE_MESSAGE,
        GraphDiagnosticCode::MissingTargetNode => LEGACY_MISSING_TARGET_MESSAGE,
        GraphDiagnosticCode::SelfLoop => LEGACY_SELF_LOOP_MESSAGE,
        GraphDiagnosticCode::CycleDetected => LEGACY_CYCLE_MESSAGE,
        GraphDiagnosticCode::GraphTooLarge => LEGACY_GRAPH_TOO_LARGE_MESSAGE,
    }
}

#[allow(
    unknown_lints,
    missing_const_fn,
    reason = "diagnostic compatibility inspects owned optional text with non-const string methods"
)]
fn diagnostic_code(diagnostic: &choregraph_core::AdmissionDiagnostic) -> GraphDiagnosticCode {
    match diagnostic.code {
        choregraph_core::AdmissionDiagnosticCode::EmptyIdentifier => {
            GraphDiagnosticCode::EmptyNodeId
        }
        choregraph_core::AdmissionDiagnosticCode::MalformedIdentifier
            if diagnostic
                .subject
                .as_deref()
                .is_some_and(|subject| subject.trim().is_empty()) =>
        {
            GraphDiagnosticCode::EmptyNodeId
        }
        choregraph_core::AdmissionDiagnosticCode::DuplicateNode => {
            GraphDiagnosticCode::DuplicateNodeId
        }
        choregraph_core::AdmissionDiagnosticCode::DuplicateEdge => {
            GraphDiagnosticCode::DuplicateEdge
        }
        choregraph_core::AdmissionDiagnosticCode::MissingSource => {
            GraphDiagnosticCode::MissingSourceNode
        }
        choregraph_core::AdmissionDiagnosticCode::MissingTarget => {
            GraphDiagnosticCode::MissingTargetNode
        }
        choregraph_core::AdmissionDiagnosticCode::SelfLoop => GraphDiagnosticCode::SelfLoop,
        choregraph_core::AdmissionDiagnosticCode::CycleDetected => {
            GraphDiagnosticCode::CycleDetected
        }
        choregraph_core::AdmissionDiagnosticCode::TooManyNodes
        | choregraph_core::AdmissionDiagnosticCode::TooManyEdges => {
            GraphDiagnosticCode::GraphTooLarge
        }
        choregraph_core::AdmissionDiagnosticCode::IdentifierTooLong
        | choregraph_core::AdmissionDiagnosticCode::MalformedIdentifier => {
            GraphDiagnosticCode::GraphTooLarge
        }
        choregraph_core::AdmissionDiagnosticCode::InconsistentLimits
        | choregraph_core::AdmissionDiagnosticCode::TooManyOperations
        | choregraph_core::AdmissionDiagnosticCode::TooManyEffectDeclarations
        | choregraph_core::AdmissionDiagnosticCode::UnsupportedSchema
        | choregraph_core::AdmissionDiagnosticCode::UnsupportedLifecycleProfile
        | choregraph_core::AdmissionDiagnosticCode::DuplicateOperation
        | choregraph_core::AdmissionDiagnosticCode::EffectNodeMissing
        | choregraph_core::AdmissionDiagnosticCode::EffectOperationUndeclared
        | choregraph_core::AdmissionDiagnosticCode::InvalidEffectTransition
        | choregraph_core::AdmissionDiagnosticCode::DuplicateEffectDeclaration
        | choregraph_core::AdmissionDiagnosticCode::MalformedReference
        | choregraph_core::AdmissionDiagnosticCode::ReferenceTooLong
        | choregraph_core::AdmissionDiagnosticCode::IntentTooLarge
        | choregraph_core::AdmissionDiagnosticCode::TooManyIntentsForTransition
        | choregraph_core::AdmissionDiagnosticCode::NormalizationInvariant => {
            GraphDiagnosticCode::GraphTooLarge
        }
    }
}

fn compatibility_diagnostics(
    diagnostics: Vec<choregraph_core::AdmissionDiagnostic>,
) -> Vec<GraphDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| {
            let code = diagnostic_code(&diagnostic);
            GraphDiagnostic {
                code,
                node_id: diagnostic.subject,
                related_node_id: diagnostic.related,
                message: diagnostic_message(code).to_string(),
            }
        })
        .collect()
}

/// Admits one legacy-shaped graph through the production Choregraph admission path.
pub fn admit_graph(graph: &ExecutionGraph) -> Result<ExecutionPlan, Vec<GraphDiagnostic>> {
    let machine =
        choregraph_core::AdmittedMachine::admit(machine_spec(graph), compatibility_limits())
            .map_err(compatibility_diagnostics)?;
    let topological_order = string_ids(machine.topological_order());
    Ok(ExecutionPlan {
        graph: graph.clone(),
        machine,
        topological_order,
    })
}

/// Returns stable compatibility diagnostics for one graph.
#[must_use]
pub fn validate_graph(graph: &ExecutionGraph) -> Vec<GraphDiagnostic> {
    match admit_graph(graph) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics,
    }
}

fn string_ids(node_ids: &[choregraph_core::NodeId]) -> Vec<String> {
    node_ids
        .iter()
        .map(|node_id| node_id.as_str().to_string())
        .collect()
}

fn parse_node_id(
    machine: &choregraph_core::AdmittedMachine,
    value: &str,
) -> Result<choregraph_core::NodeId, QueryError> {
    choregraph_core::NodeId::parse(value, machine.limits().max_identifier_bytes)
        .map_err(|_| QueryError::InvalidNodeId)
}

#[allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    reason = "the closed compatibility lifecycle requires total mapping to the verified core vocabulary"
)]
const fn core_state(state: StepState) -> choregraph_core::LocalState {
    match state {
        StepState::Pending => choregraph_core::LocalState::Pending,
        StepState::Ready => choregraph_core::LocalState::Ready,
        StepState::Running => choregraph_core::LocalState::Running,
        StepState::Completed => choregraph_core::LocalState::Completed,
        StepState::Failed => choregraph_core::LocalState::Failed,
        StepState::Cancelled => choregraph_core::LocalState::Cancelled,
    }
}

const fn compatibility_state(state: choregraph_core::LocalState) -> StepState {
    match state {
        choregraph_core::LocalState::Pending => StepState::Pending,
        choregraph_core::LocalState::Ready => StepState::Ready,
        choregraph_core::LocalState::Running => StepState::Running,
        choregraph_core::LocalState::Completed => StepState::Completed,
        choregraph_core::LocalState::Failed => StepState::Failed,
        choregraph_core::LocalState::Cancelled => StepState::Cancelled,
    }
}

#[allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    reason = "the closed compatibility lifecycle requires total mapping to the verified core vocabulary"
)]
const fn core_action(action: StepAction) -> choregraph_core::Action {
    match action {
        StepAction::MarkReady => choregraph_core::Action::MarkReady,
        StepAction::Start => choregraph_core::Action::Start,
        StepAction::Complete => choregraph_core::Action::Complete,
        StepAction::Fail => choregraph_core::Action::Fail,
        StepAction::Cancel => choregraph_core::Action::Cancel,
    }
}

/// Applies one local compatibility transition through the verified production lifecycle kernel.
pub const fn transition_step_state(
    from: StepState,
    action: StepAction,
) -> Result<StepState, StepTransitionError> {
    match choregraph_core::lifecycle_target(core_state(from), core_action(action)) {
        Ok(target) => Ok(compatibility_state(target)),
        Err(choregraph_core::LocalTransitionError::Terminal { .. }) => {
            Err(StepTransitionError::TerminalState { from, action })
        }
        Err(choregraph_core::LocalTransitionError::Invalid { .. }) => {
            Err(StepTransitionError::InvalidTransition { from, action })
        }
    }
}

/// Returns whether one compatibility state is terminal.
#[must_use]
pub const fn is_terminal_state(state: StepState) -> bool {
    matches!(
        state,
        StepState::Completed | StepState::Failed | StepState::Cancelled
    )
}

/// Expands a sparse legacy state map to the exact Choregraph aggregate state.
pub fn expand_state(
    plan: &ExecutionPlan,
    sparse_states: &BTreeMap<String, StepState>,
) -> Result<choregraph_core::MachineState, StateConversionError> {
    let mut entries = plan.machine.initial_state().entries().clone();
    for (raw_node_id, state) in sparse_states {
        let node_id =
            choregraph_core::NodeId::parse(raw_node_id, plan.machine.limits().max_identifier_bytes)
                .map_err(|_| StateConversionError::InvalidNodeId)?;
        let Some(entry) = entries.get_mut(&node_id) else {
            return Err(StateConversionError::UnknownNode);
        };
        *entry = core_state(*state);
    }
    let state = choregraph_core::MachineState::new(entries);
    let issues = choregraph_core::validate_invariant(&plan.machine, &state);
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
    let ready = choregraph_core::ready_nodes(&plan.machine, &state)
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
    debug_assert!(choregraph_core::well_formed(&plan.machine));
    debug_assert_eq!(plan.machine.node_ids().len(), plan.graph.nodes.len());
    let state = expand_state(plan, sparse_states).map_err(|error| match error {
        StateConversionError::InvalidState(issues) => ResumeDecisionError::InvalidState(issues),
        StateConversionError::InvalidNodeId | StateConversionError::UnknownNode => {
            ResumeDecisionError::UnknownNode
        }
    })?;
    let node_id =
        parse_node_id(&plan.machine, node_id).map_err(|_| ResumeDecisionError::UnknownNode)?;
    match choregraph_core::resume_decision(&plan.machine, &state, &node_id) {
        choregraph_core::ResumeDecision::Execute => Ok(ResumeDecision::Execute),
        choregraph_core::ResumeDecision::AwaitDependencies => Ok(ResumeDecision::AwaitDependencies),
        choregraph_core::ResumeDecision::SkipCompleted => Ok(ResumeDecision::SkipCompleted),
        choregraph_core::ResumeDecision::RejectTerminalMutation => {
            Ok(ResumeDecision::RejectTerminalMutation)
        }
        choregraph_core::ResumeDecision::BlockedByFailedDependency => {
            Ok(ResumeDecision::BlockedByFailedDependency)
        }
        choregraph_core::ResumeDecision::UnknownNode => Err(ResumeDecisionError::UnknownNode),
        choregraph_core::ResumeDecision::InvalidState => {
            Err(ResumeDecisionError::InvalidState(Vec::new()))
        }
    }
}

/// Applies one aggregate event through production `step` and exposes no effect intent.
#[must_use]
pub fn step_machine(
    plan: &ExecutionPlan,
    state: &choregraph_core::MachineState,
    node_id: &str,
    action: StepAction,
) -> MachineDecision {
    let event = choregraph_core::Event::new(node_id, core_action(action));
    match choregraph_core::step(&plan.machine, state, &event) {
        choregraph_core::Decision::Accept { next, intents } if intents.is_empty() => {
            MachineDecision::Accept { next }
        }
        choregraph_core::Decision::Accept { .. } => MachineDecision::AuthorityBoundaryViolation,
        choregraph_core::Decision::Deny { reason } => MachineDecision::Deny { reason },
    }
}

/// Checks one proposed edge through the production Choregraph cycle probe.
#[must_use]
#[expect(
    clippy::manual_unwrap_or,
    reason = "explicit compatibility control flow keeps the conservative rejection branch visible"
)]
#[allow(
    unknown_lints,
    ambiguous_params,
    reason = "the exact legacy cycle-probe signature has two adjacent identifier parameters"
)]
pub fn would_create_cycle(graph: &ExecutionGraph, source: &str, target: &str) -> bool {
    match would_create_cycle_checked(CycleProbe {
        graph,
        source,
        target,
    }) {
        Ok(decision) => decision,
        Err(_) => true,
    }
}

/// Checks one proposed edge with explicit graph conversion failures for new consumers.
pub fn would_create_cycle_checked(probe: CycleProbe<'_>) -> Result<bool, Vec<GraphDiagnostic>> {
    let plan = admit_graph(probe.graph)?;
    let Ok(source) = parse_node_id(&plan.machine, probe.source) else {
        return Ok(false);
    };
    let Ok(target) = parse_node_id(&plan.machine, probe.target) else {
        return Ok(false);
    };
    Ok(choregraph_core::would_create_cycle(
        &plan.machine,
        &source,
        &target,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepts_delegation_to_core() {
        assert!(super::accepts(
            super::CompatibilityMode::DelegateToChoregraphCore
        ));
    }

    #[test]
    fn rejects_an_independent_transition_core() {
        assert!(!super::accepts(
            super::CompatibilityMode::IndependentTransitionImplementation
        ));
    }
}
