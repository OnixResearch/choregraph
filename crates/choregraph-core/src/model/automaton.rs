use crate::concrete::effect::ConcreteEffectDeclaration;
use crate::id::bounded::{DeclarationKey, NodeId, OperationId};
use crate::model::capacity::AdmissionLimits;
use crate::model::dispatch::{EffectDeclaration, EffectDeclarationSpec};
use crate::model::graph::{Edge, EdgeSpec, NodeSpec};
use crate::model::pause::{SuspensionDeclaration, SuspensionDeclarationSpec, SuspensionRecord};
use crate::model::semantics::{
    LINEAR_LIFECYCLE_V1, LifecycleProfile, MACHINE_SCHEMA_V1, MACHINE_SCHEMA_V2, MachineSchema,
};
use crate::model::semantics::{LocalState, TransitionSelector};
use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

/// One untrusted canonical machine specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineSpec {
    pub schema: String,
    pub lifecycle_profile: String,
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<EdgeSpec>,
    pub effects: Vec<EffectDeclarationSpec>,
    pub suspensions: Vec<SuspensionDeclarationSpec>,
}

impl MachineSpec {
    #[must_use]
    pub fn new(nodes: Vec<NodeSpec>, edges: Vec<EdgeSpec>) -> Self {
        Self {
            schema: String::from(MACHINE_SCHEMA_V1),
            lifecycle_profile: String::from(LINEAR_LIFECYCLE_V1),
            nodes,
            edges,
            effects: Vec::new(),
            suspensions: Vec::new(),
        }
    }
}

/// One admitted immutable node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AdmittedNode {
    pub(crate) operations: BTreeSet<OperationId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct EffectKey {
    pub(crate) node_id: NodeId,
    pub(crate) transition: TransitionSelector,
}

/// An admitted machine with normalized immutable indexes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedMachine {
    pub(crate) limits: AdmissionLimits,
    pub(crate) schema: MachineSchema,
    pub(crate) lifecycle_profile: LifecycleProfile,
    pub(crate) nodes: BTreeMap<NodeId, AdmittedNode>,
    pub(crate) node_order: Vec<NodeId>,
    pub(crate) node_indices: BTreeMap<NodeId, usize>,
    pub(crate) edges: Vec<Edge>,
    pub(crate) edge_indices: Vec<(usize, usize)>,
    pub(crate) dependencies: BTreeMap<NodeId, Vec<NodeId>>,
    pub(crate) dependency_indices: Vec<Vec<usize>>,
    pub(crate) dependents: BTreeMap<NodeId, Vec<NodeId>>,
    pub(crate) topological_order: Vec<NodeId>,
    pub(crate) topological_indices: Vec<usize>,
    pub(crate) effects: Vec<EffectDeclaration>,
    pub(crate) effect_index: BTreeMap<EffectKey, Vec<EffectDeclaration>>,
    pub(crate) effect_route_indices: BTreeMap<EffectKey, Vec<usize>>,
    pub(crate) concrete_effects: Vec<ConcreteEffectDeclaration>,
    pub(crate) concrete_effect_key_indices: BTreeMap<DeclarationKey, usize>,
    pub(crate) concrete_effect_route_indices: BTreeMap<EffectKey, Vec<usize>>,
    pub(crate) suspension_declarations: Vec<SuspensionDeclaration>,
    pub(crate) suspension_declaration_indices: BTreeMap<NodeId, usize>,
}

impl AdmittedMachine {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        match self.schema {
            MachineSchema::V1 => MACHINE_SCHEMA_V1,
            MachineSchema::V2 => MACHINE_SCHEMA_V2,
        }
    }

    #[must_use]
    pub const fn schema_version(&self) -> MachineSchema {
        self.schema
    }

    #[must_use]
    pub const fn lifecycle_profile(&self) -> LifecycleProfile {
        self.lifecycle_profile
    }

    #[must_use]
    pub const fn limits(&self) -> AdmissionLimits {
        self.limits
    }

    #[must_use]
    pub fn node_ids(&self) -> &[NodeId] {
        &self.node_order
    }

    #[must_use]
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    #[must_use]
    pub fn topological_order(&self) -> &[NodeId] {
        &self.topological_order
    }

    #[must_use]
    pub fn dependencies(&self, node_id: &NodeId) -> Option<&[NodeId]> {
        self.dependencies.get(node_id).map(Vec::as_slice)
    }

    #[must_use]
    pub fn operations(&self, node_id: &NodeId) -> Option<&BTreeSet<OperationId>> {
        self.nodes.get(node_id).map(|node| &node.operations)
    }

    #[must_use]
    pub fn dependents(&self, node_id: &NodeId) -> Option<&[NodeId]> {
        self.dependents.get(node_id).map(Vec::as_slice)
    }

    #[must_use]
    pub fn effect_declarations(&self) -> &[EffectDeclaration] {
        &self.effects
    }

    #[must_use]
    pub fn concrete_effect_declarations(&self) -> &[ConcreteEffectDeclaration] {
        &self.concrete_effects
    }

    #[must_use]
    pub fn suspension_declarations(&self) -> &[SuspensionDeclaration] {
        &self.suspension_declarations
    }

    #[must_use]
    pub fn suspension_declaration(&self, node_id: &NodeId) -> Option<&SuspensionDeclaration> {
        self.suspension_declaration_indices
            .get(node_id)
            .and_then(|index| self.suspension_declarations.get(*index))
    }

    #[must_use]
    pub fn initial_state(&self) -> MachineState {
        let entries = self
            .node_order
            .iter()
            .cloned()
            .map(|node_id| (node_id, LocalState::Pending))
            .collect();
        MachineState::new(entries)
    }

    /// Builds the restart state with every admitted suspension declaration
    /// applied as a suspended node with its exact recorded facts.
    ///
    /// This is the durable-resume entrypoint: a consumer re-admits the machine
    /// after a process exit and reconstructs the recorded wait without
    /// replaying product history. Choregraph still does not persist anything.
    #[must_use]
    pub fn declared_suspension_state(&self) -> MachineState {
        let mut state = self.initial_state();
        for declaration in &self.suspension_declarations {
            state
                .entries
                .insert(declaration.node_id.clone(), LocalState::Suspended);
            state.suspensions.insert(
                declaration.node_id.clone(),
                SuspensionRecord {
                    class: declaration.class.clone(),
                    reference: declaration.reference.clone(),
                },
            );
        }
        state
    }
}

/// Parameterized aggregate state with one entry per admitted node.
///
/// Suspension records exist exactly for nodes in `LocalState::Suspended`.
/// Machines admitted under `LINEAR_LIFECYCLE_V1` always have an empty
/// suspension map, which keeps v1 canonical bytes and equality unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineState {
    pub(crate) entries: BTreeMap<NodeId, LocalState>,
    pub(crate) suspensions: BTreeMap<NodeId, SuspensionRecord>,
}

impl MachineState {
    #[must_use]
    pub fn new(entries: BTreeMap<NodeId, LocalState>) -> Self {
        Self {
            entries,
            suspensions: BTreeMap::new(),
        }
    }

    /// Builds one aggregate state with explicit suspension records.
    ///
    /// Consumers use this to reconstruct a recorded durable wait from their own
    /// persisted facts. The executable invariant still validates that records
    /// exist exactly for suspended nodes before any transition accepts it.
    #[must_use]
    pub fn with_suspensions(
        entries: BTreeMap<NodeId, LocalState>,
        suspensions: BTreeMap<NodeId, SuspensionRecord>,
    ) -> Self {
        Self {
            entries,
            suspensions,
        }
    }

    #[must_use]
    pub fn entries(&self) -> &BTreeMap<NodeId, LocalState> {
        &self.entries
    }

    #[must_use]
    pub fn state(&self, node_id: &NodeId) -> Option<LocalState> {
        self.entries.get(node_id).copied()
    }

    #[must_use]
    pub fn suspensions(&self) -> &BTreeMap<NodeId, SuspensionRecord> {
        &self.suspensions
    }

    #[must_use]
    pub fn suspension(&self, node_id: &NodeId) -> Option<&SuspensionRecord> {
        self.suspensions.get(node_id)
    }
}
