use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::aggregate::ExecutionPlan;
use crate::topology::{ExecutionGraph, string_ids};

const LEGACY_EMPTY_NODE_MESSAGE: &str = "execution graph node id cannot be empty";
const LEGACY_DUPLICATE_NODE_MESSAGE: &str = "execution graph node id must be unique";
const LEGACY_DUPLICATE_EDGE_MESSAGE: &str = "execution graph edge must be unique";
const LEGACY_MISSING_SOURCE_MESSAGE: &str = "execution graph edge source node is missing";
const LEGACY_MISSING_TARGET_MESSAGE: &str = "execution graph edge target node is missing";
const LEGACY_SELF_LOOP_MESSAGE: &str = "execution graph edge cannot connect a node to itself";
const LEGACY_CYCLE_MESSAGE: &str = "execution graph contains a cycle";
const LEGACY_GRAPH_TOO_LARGE_MESSAGE: &str =
    "execution graph has more nodes than the supported level and analytics range";
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
const fn compatibility_limits() -> choregraph_core::model::capacity::AdmissionLimits {
    choregraph_core::model::capacity::AdmissionLimits {
        max_nodes: choregraph_core::model::capacity::DEFAULT_MAX_NODES,
        max_edges: choregraph_core::model::capacity::DEFAULT_MAX_EDGES,
        max_operations: choregraph_core::model::capacity::DEFAULT_MAX_OPERATIONS,
        max_effect_declarations: choregraph_core::model::capacity::DEFAULT_MAX_EFFECT_DECLARATIONS,
        max_identifier_bytes: choregraph_core::model::capacity::DEFAULT_MAX_IDENTIFIER_BYTES,
        max_reference_bytes: choregraph_core::model::capacity::DEFAULT_MAX_REFERENCE_BYTES,
        max_event_bytes: choregraph_core::model::capacity::DEFAULT_MAX_EVENT_BYTES,
        max_intent_bytes: choregraph_core::model::capacity::DEFAULT_MAX_INTENT_BYTES,
        max_intents_per_transition:
            choregraph_core::model::capacity::DEFAULT_MAX_INTENTS_PER_TRANSITION,
    }
}

fn machine_spec(graph: &ExecutionGraph) -> choregraph_core::model::automaton::MachineSpec {
    let nodes = graph
        .nodes
        .iter()
        .map(|node| choregraph_core::model::graph::NodeSpec::new(node.id.clone(), Vec::new()))
        .collect();
    let edges = graph
        .edges
        .iter()
        .map(|edge| {
            choregraph_core::model::graph::EdgeSpec::new(edge.source.clone(), edge.target.clone())
        })
        .collect();
    choregraph_core::model::automaton::MachineSpec::new(nodes, edges)
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
fn diagnostic_code(
    diagnostic: &choregraph_core::model::outcome::AdmissionDiagnostic,
) -> GraphDiagnosticCode {
    match diagnostic.code {
        choregraph_core::model::outcome::AdmissionDiagnosticCode::EmptyIdentifier => {
            GraphDiagnosticCode::EmptyNodeId
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::MalformedIdentifier
            if diagnostic
                .subject
                .as_deref()
                .is_some_and(|subject| subject.trim().is_empty()) =>
        {
            GraphDiagnosticCode::EmptyNodeId
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateNode => {
            GraphDiagnosticCode::DuplicateNodeId
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateEdge => {
            GraphDiagnosticCode::DuplicateEdge
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::MissingSource => {
            GraphDiagnosticCode::MissingSourceNode
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::MissingTarget => {
            GraphDiagnosticCode::MissingTargetNode
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::SelfLoop => GraphDiagnosticCode::SelfLoop,
        choregraph_core::model::outcome::AdmissionDiagnosticCode::CycleDetected => {
            GraphDiagnosticCode::CycleDetected
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::TooManyNodes
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::TooManyEdges => {
            GraphDiagnosticCode::GraphTooLarge
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::IdentifierTooLong
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::MalformedIdentifier => {
            GraphDiagnosticCode::GraphTooLarge
        }
        choregraph_core::model::outcome::AdmissionDiagnosticCode::InconsistentLimits
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::TooManyOperations
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::TooManyEffectDeclarations
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::UnsupportedSchema
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::UnsupportedLifecycleProfile
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateOperation
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::EffectNodeMissing
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::EffectOperationUndeclared
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::InvalidEffectTransition
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateEffectDeclaration
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateDeclarationKey
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateConcreteEffectDeclaration
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::UnknownReferenceProfile
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::MalformedBinding
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::MalformedReference
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::ReferenceTooLong
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::IntentTooLarge
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::TooManyIntentsForTransition
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::DuplicateSuspensionDeclaration
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::SuspensionDeclarationNodeMissing
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::SuspensionDeclarationWithoutProfile
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::MalformedSuspensionClass
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::MalformedSuspensionReference
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::SuspensionEffectTransition
        | choregraph_core::model::outcome::AdmissionDiagnosticCode::NormalizationInvariant => {
            GraphDiagnosticCode::GraphTooLarge
        }
    }
}

fn compatibility_diagnostics(
    diagnostics: Vec<choregraph_core::model::outcome::AdmissionDiagnostic>,
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
    let machine = choregraph_core::model::automaton::AdmittedMachine::admit(
        machine_spec(graph),
        compatibility_limits(),
    )
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
