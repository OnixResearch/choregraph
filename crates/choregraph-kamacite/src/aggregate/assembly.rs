use crate::aggregate::policy::EXECUTION_GRAPH_REVISION;
use crate::aggregate::policy::GLOBAL_STATE_CARDINALITY;
use crate::aggregate::policy::GLOBAL_STATE_MODEL;
use crate::aggregate::policy::KAMACITE_REVISION;
use crate::aggregate::policy::LATTICE_REVISION;
use crate::aggregate::policy::SupportedKamaciteProfile;
use crate::aggregate::policy::build_profile_artifact;
use crate::aggregate::record::AggregateMachineDescriptor;
use crate::aggregate::record::CompatibilityMetadata;
use crate::aggregate::record::DescriptorDependencySet;
use crate::aggregate::record::DescriptorEdge;
use crate::aggregate::record::DescriptorEffect;
use crate::aggregate::record::DescriptorLimits;
use crate::aggregate::record::DescriptorNode;
use crate::aggregate::record::DescriptorSuspension;
use crate::aggregate::record::DurableMachineDescriptor;
use crate::codec::graph::{
    ADMITTED_IR_ROOT_LABEL, ADMITTED_IR_SCHEMA_VERSION, MACHINE_ROOT_LABEL, MACHINE_SCHEMA_VERSION,
};
use crate::codec::{CanonicalArtifact, CodecBoundaryError, decode, encode};
use crate::identity::{
    ArtifactDomain, ArtifactIdentity, IdentityChain, IdentityError, KamaciteIdentityDomain,
    compute_identity, compute_kamacite_identity,
};
use crate::lifecycle::conversion::{
    LIFECYCLE_FSM_ID, LifecycleArtifact, LifecycleError, build_lifecycle_artifact,
};
use choregraph_core::{
    model::automaton::AdmittedMachine, model::automaton::MachineSpec,
    model::capacity::AdmissionLimits, model::dispatch::EffectDeclarationSpec,
    model::graph::EdgeSpec, model::graph::NodeSpec, model::semantics::Action,
    model::semantics::LocalState, model::semantics::TransitionSelector,
};
use serde::Serialize;
use std::fmt::{Display, Formatter};

const DESCRIPTOR_SCHEMA_VERSION: u32 = 1;
const DURABLE_DESCRIPTOR_SCHEMA_VERSION: u32 = 2;
const MACHINE_NON_CLAIM_COUNT: usize = 4;
const DURABLE_MACHINE_NON_CLAIM_COUNT: usize = 5;
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-admitted-ir-v1", deny_unknown_fields)]
struct AdmittedIrDescriptor<'a> {
    schema_version: u32,
    machine_schema: &'a str,
    lifecycle_profile_id: &'a str,
    limits: &'a DescriptorLimits,
    nodes: &'a [DescriptorNode],
    edges: &'a [DescriptorEdge],
    dependencies: &'a [DescriptorDependencySet],
    topological_order: &'a [String],
    effects: &'a [DescriptorEffect],
}

#[derive(Clone, Copy, Debug)]
pub struct IdentityMaterials<'a> {
    pub source_envelope: &'a [u8],
    pub proof_set: &'a [u8],
    pub generator: &'a [u8],
    pub adapter: &'a [u8],
    pub toolchain: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactSet {
    pub profile: SupportedKamaciteProfile,
    pub profile_artifact: CanonicalArtifact,
    pub lifecycle: LifecycleArtifact,
    pub machine_descriptor: AggregateMachineDescriptor,
    pub machine_artifact: CanonicalArtifact,
    pub admitted_ir_artifact: CanonicalArtifact,
    pub identities: IdentityChain,
    pub kamacite_adapter_identity: crate::identity::KamaciteIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMachineArtifact {
    pub machine: AdmittedMachine,
    pub descriptor: AggregateMachineDescriptor,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AggregateError {
    Codec(CodecBoundaryError),
    Identity(IdentityError),
    Lifecycle(LifecycleError),
    NumericLimitOverflow,
    UnsupportedProfile,
    GlobalStateEnumeration,
    DescriptorMismatch,
    AdmissionRejected,
    UnknownLifecycleValue,
    MissingNormalizedEntry,
    EmptySourceEnvelope,
}

impl Display for AggregateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if let Self::Codec(error) = self {
            return Display::fmt(error, formatter);
        }
        if let Self::Identity(error) = self {
            return Display::fmt(error, formatter);
        }
        if let Self::Lifecycle(error) = self {
            return Display::fmt(error, formatter);
        }
        const DIAGNOSTICS: [(AggregateError, &str); 8] = [
            (
                AggregateError::NumericLimitOverflow,
                "descriptor numeric limit does not fit this target",
            ),
            (
                AggregateError::UnsupportedProfile,
                "unsupported Kamacite profile binding",
            ),
            (
                AggregateError::GlobalStateEnumeration,
                "aggregate product-state enumeration is not admitted",
            ),
            (
                AggregateError::DescriptorMismatch,
                "canonical machine descriptor does not match admitted IR",
            ),
            (
                AggregateError::AdmissionRejected,
                "decoded machine failed core admission",
            ),
            (
                AggregateError::UnknownLifecycleValue,
                "descriptor contains an unknown lifecycle value",
            ),
            (
                AggregateError::MissingNormalizedEntry,
                "admitted machine is missing a normalized index entry",
            ),
            (
                AggregateError::EmptySourceEnvelope,
                "source envelope identity material is empty",
            ),
        ];
        if let Some((_, message)) = DIAGNOSTICS.iter().find(|(category, _)| category == self) {
            formatter.write_str(message)
        } else {
            // New categories retain their exact variant and payload in diagnostics.
            std::fmt::Debug::fmt(self, formatter)
        }
    }
}

impl std::error::Error for AggregateError {}

impl From<CodecBoundaryError> for AggregateError {
    fn from(error: CodecBoundaryError) -> Self {
        Self::Codec(error)
    }
}

impl From<IdentityError> for AggregateError {
    fn from(error: IdentityError) -> Self {
        Self::Identity(error)
    }
}

impl From<LifecycleError> for AggregateError {
    fn from(error: LifecycleError) -> Self {
        Self::Lifecycle(error)
    }
}
fn usize_to_u64(value: usize) -> Result<u64, AggregateError> {
    u64::try_from(value).map_err(|_| AggregateError::NumericLimitOverflow)
}

fn u64_to_usize(value: u64) -> Result<usize, AggregateError> {
    usize::try_from(value).map_err(|_| AggregateError::NumericLimitOverflow)
}

pub(crate) fn descriptor_limits(
    limits: AdmissionLimits,
) -> Result<DescriptorLimits, AggregateError> {
    Ok(DescriptorLimits {
        max_nodes: usize_to_u64(limits.max_nodes)?,
        max_edges: usize_to_u64(limits.max_edges)?,
        max_operations: usize_to_u64(limits.max_operations)?,
        max_effect_declarations: usize_to_u64(limits.max_effect_declarations)?,
        max_identifier_bytes: usize_to_u64(limits.max_identifier_bytes)?,
        max_reference_bytes: usize_to_u64(limits.max_reference_bytes)?,
        max_event_bytes: usize_to_u64(limits.max_event_bytes)?,
        max_intent_bytes: usize_to_u64(limits.max_intent_bytes)?,
        max_intents_per_transition: usize_to_u64(limits.max_intents_per_transition)?,
    })
}

pub(crate) fn admission_limits(
    limits: &DescriptorLimits,
) -> Result<AdmissionLimits, AggregateError> {
    Ok(AdmissionLimits {
        max_nodes: u64_to_usize(limits.max_nodes)?,
        max_edges: u64_to_usize(limits.max_edges)?,
        max_operations: u64_to_usize(limits.max_operations)?,
        max_effect_declarations: u64_to_usize(limits.max_effect_declarations)?,
        max_identifier_bytes: u64_to_usize(limits.max_identifier_bytes)?,
        max_reference_bytes: u64_to_usize(limits.max_reference_bytes)?,
        max_event_bytes: u64_to_usize(limits.max_event_bytes)?,
        max_intent_bytes: u64_to_usize(limits.max_intent_bytes)?,
        max_intents_per_transition: u64_to_usize(limits.max_intents_per_transition)?,
    })
}

pub(crate) fn state_id(state: LocalState) -> &'static str {
    match state {
        LocalState::Pending => "pending",
        LocalState::Ready => "ready",
        LocalState::Running => "running",
        LocalState::Completed => "completed",
        LocalState::Failed => "failed",
        LocalState::Cancelled => "cancelled",
        LocalState::Suspended => "suspended",
    }
}

pub(crate) fn action_id(action: Action) -> &'static str {
    match action {
        Action::MarkReady => "mark-ready",
        Action::Start => "start",
        Action::Complete => "complete",
        Action::Fail => "fail",
        Action::Cancel => "cancel",
        Action::Suspend => "suspend",
        Action::ResumeSignal => "resume-signal",
    }
}

pub(crate) fn action_from_id(value: &str) -> Result<Action, AggregateError> {
    match value {
        "mark-ready" => Ok(Action::MarkReady),
        "start" => Ok(Action::Start),
        "complete" => Ok(Action::Complete),
        "fail" => Ok(Action::Fail),
        "cancel" => Ok(Action::Cancel),
        _ => Err(AggregateError::UnknownLifecycleValue),
    }
}

pub(crate) fn state_from_id(value: &str) -> Result<LocalState, AggregateError> {
    match value {
        "pending" => Ok(LocalState::Pending),
        "ready" => Ok(LocalState::Ready),
        "running" => Ok(LocalState::Running),
        "completed" => Ok(LocalState::Completed),
        "failed" => Ok(LocalState::Failed),
        "cancelled" => Ok(LocalState::Cancelled),
        _ => Err(AggregateError::UnknownLifecycleValue),
    }
}

struct NormalizedParts {
    nodes: Vec<DescriptorNode>,
    edges: Vec<DescriptorEdge>,
    dependencies: Vec<DescriptorDependencySet>,
    topological_order: Vec<String>,
    effects: Vec<DescriptorEffect>,
}

fn normalized_parts(machine: &AdmittedMachine) -> Result<NormalizedParts, AggregateError> {
    let mut nodes = Vec::with_capacity(machine.node_ids().len());
    let mut dependencies = Vec::with_capacity(machine.node_ids().len());
    for node_id in machine.node_ids() {
        let operations = machine
            .operations(node_id)
            .ok_or(AggregateError::MissingNormalizedEntry)?;
        nodes.push(DescriptorNode::from((node_id, operations)));
        let dependency_ids = machine
            .dependencies(node_id)
            .ok_or(AggregateError::MissingNormalizedEntry)?;
        dependencies.push(DescriptorDependencySet::from((node_id, dependency_ids)));
    }
    let edges = machine
        .edges()
        .iter()
        .map(|edge| DescriptorEdge {
            source: String::from(edge.source.as_str()),
            target: String::from(edge.target.as_str()),
        })
        .collect();
    let topological_order = machine
        .topological_order()
        .iter()
        .map(|node_id| String::from(node_id.as_str()))
        .collect();
    let effects = machine
        .effect_declarations()
        .iter()
        .map(|effect| DescriptorEffect {
            node_id: String::from(effect.node_id.as_str()),
            from_state: String::from(state_id(effect.transition.from)),
            action: String::from(action_id(effect.transition.action)),
            operation: String::from(effect.operation.as_str()),
            resource: String::from(effect.resource.as_str()),
            ability: String::from(effect.ability.as_str()),
            request: String::from(effect.request.as_str()),
            payload: String::from(effect.payload.as_str()),
        })
        .collect();
    Ok(NormalizedParts {
        nodes,
        edges,
        dependencies,
        topological_order,
        effects,
    })
}

fn machine_descriptor(
    machine: &AdmittedMachine,
    lifecycle_identity: ArtifactIdentity,
    profile_identity: ArtifactIdentity,
) -> Result<AggregateMachineDescriptor, AggregateError> {
    if machine.lifecycle_profile() != choregraph_core::model::semantics::LifecycleProfile::LinearV1
    {
        return Err(AggregateError::UnsupportedProfile);
    }
    let parts = normalized_parts(machine)?;
    Ok(AggregateMachineDescriptor {
        schema_version: DESCRIPTOR_SCHEMA_VERSION,
        machine_schema: String::from(machine.schema()),
        lifecycle_profile_id: String::from(LIFECYCLE_FSM_ID),
        lifecycle_profile_identity: lifecycle_identity,
        kamacite_profile_identity: profile_identity,
        global_state_model: String::from(GLOBAL_STATE_MODEL),
        global_state_cardinality: String::from(GLOBAL_STATE_CARDINALITY),
        limits: descriptor_limits(machine.limits())?,
        nodes: parts.nodes,
        edges: parts.edges,
        dependencies: parts.dependencies,
        topological_order: parts.topological_order,
        effects: parts.effects,
        compatibility: CompatibilityMetadata {
            execution_graph_revision: String::from(EXECUTION_GRAPH_REVISION),
            lattice_revision: String::from(LATTICE_REVISION),
            kamacite_revision: String::from(KAMACITE_REVISION),
        },
        checked_laws: vec![
            String::from("bounded-normalized-admitted-machine"),
            String::from("parameterized-aggregate-state"),
            String::from("exact-lifecycle-profile-binding"),
        ],
        non_claims: vec![
            String::from("does not enumerate aggregate product states"),
            String::from("does not prove Nickel evaluation semantics"),
            String::from("does not authorize or execute runtime effects"),
            String::from("does not establish release eligibility"),
        ],
    })
}

fn admitted_ir_descriptor(descriptor: &AggregateMachineDescriptor) -> AdmittedIrDescriptor<'_> {
    AdmittedIrDescriptor {
        schema_version: DESCRIPTOR_SCHEMA_VERSION,
        machine_schema: &descriptor.machine_schema,
        lifecycle_profile_id: &descriptor.lifecycle_profile_id,
        limits: &descriptor.limits,
        nodes: &descriptor.nodes,
        edges: &descriptor.edges,
        dependencies: &descriptor.dependencies,
        topological_order: &descriptor.topological_order,
        effects: &descriptor.effects,
    }
}

pub fn encode_machine_descriptor(
    descriptor: &AggregateMachineDescriptor,
) -> Result<CanonicalArtifact, AggregateError> {
    encode(descriptor, MACHINE_SCHEMA_VERSION, MACHINE_ROOT_LABEL).map_err(Into::into)
}

pub fn build_artifact_set(
    machine: &AdmittedMachine,
    materials: IdentityMaterials<'_>,
) -> Result<ArtifactSet, AggregateError> {
    if materials.source_envelope.is_empty() {
        return Err(AggregateError::EmptySourceEnvelope);
    }
    let (profile, profile_artifact, profile_identity) = build_profile_artifact()?;
    let source_identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Source, materials.source_envelope);
    let lifecycle = build_lifecycle_artifact(Some(source_identity))?;
    let machine_descriptor = machine_descriptor(
        machine,
        lifecycle.project_identity.clone(),
        profile_identity.clone(),
    )?;
    let machine_artifact = encode_machine_descriptor(&machine_descriptor)?;
    let admitted_ir = admitted_ir_descriptor(&machine_descriptor);
    let admitted_ir_artifact = encode(
        &admitted_ir,
        ADMITTED_IR_SCHEMA_VERSION,
        ADMITTED_IR_ROOT_LABEL,
    )?;
    let kamacite_adapter_identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Adapter, materials.adapter);
    let identities = IdentityChain {
        machine: compute_identity(ArtifactDomain::Machine, &machine_artifact.packed_bytes)?,
        lifecycle: lifecycle.project_identity.clone(),
        admitted_ir: compute_identity(
            ArtifactDomain::AdmittedIr,
            &admitted_ir_artifact.packed_bytes,
        )?,
        proof_set: compute_identity(ArtifactDomain::ProofSet, materials.proof_set)?,
        generator: compute_identity(ArtifactDomain::Generator, materials.generator)?,
        adapter: compute_identity(ArtifactDomain::Adapter, materials.adapter)?,
        profile: profile_identity,
        toolchain: compute_identity(ArtifactDomain::Toolchain, materials.toolchain)?,
    };
    Ok(ArtifactSet {
        profile,
        profile_artifact,
        lifecycle,
        machine_descriptor,
        machine_artifact,
        admitted_ir_artifact,
        identities,
        kamacite_adapter_identity,
    })
}

fn machine_spec(descriptor: &AggregateMachineDescriptor) -> Result<MachineSpec, AggregateError> {
    let nodes = descriptor
        .nodes
        .iter()
        .map(|node| NodeSpec::new(node.node_id.clone(), node.operations.clone()))
        .collect();
    let edges = descriptor
        .edges
        .iter()
        .map(|edge| EdgeSpec::new(edge.source.clone(), edge.target.clone()))
        .collect();
    let effects = descriptor
        .effects
        .iter()
        .map(|effect| {
            Ok(EffectDeclarationSpec {
                node_id: effect.node_id.clone(),
                transition: TransitionSelector {
                    from: state_from_id(&effect.from_state)?,
                    action: action_from_id(&effect.action)?,
                },
                operation: effect.operation.clone(),
                resource: effect.resource.clone(),
                ability: effect.ability.clone(),
                request: effect.request.clone(),
                payload: effect.payload.clone(),
            })
        })
        .collect::<Result<Vec<_>, AggregateError>>()?;
    Ok(MachineSpec {
        schema: descriptor.machine_schema.clone(),
        lifecycle_profile: descriptor.lifecycle_profile_id.clone(),
        nodes,
        edges,
        effects,
        suspensions: Vec::new(),
    })
}

/// Builds the version-2 descriptor for an admitted durable-suspension machine.
pub fn durable_machine_descriptor(
    machine: &AdmittedMachine,
    lifecycle_identity: ArtifactIdentity,
    profile_identity: ArtifactIdentity,
) -> Result<DurableMachineDescriptor, AggregateError> {
    if machine.lifecycle_profile()
        != choregraph_core::model::semantics::LifecycleProfile::DurableSuspensionV1
    {
        return Err(AggregateError::UnsupportedProfile);
    }
    let parts = normalized_parts(machine)?;
    let suspensions = machine
        .suspension_declarations()
        .iter()
        .map(|declaration| DescriptorSuspension {
            node_id: String::from(declaration.node_id.as_str()),
            class: String::from(declaration.class.as_str()),
            reference: String::from(declaration.reference.as_str()),
        })
        .collect();
    Ok(DurableMachineDescriptor {
        schema_version: DURABLE_DESCRIPTOR_SCHEMA_VERSION,
        machine_schema: String::from(machine.schema()),
        lifecycle_profile_id: String::from(crate::lifecycle::conversion::DURABLE_FSM_ID),
        lifecycle_profile_identity: lifecycle_identity,
        kamacite_profile_identity: profile_identity,
        global_state_model: String::from(GLOBAL_STATE_MODEL),
        global_state_cardinality: String::from(GLOBAL_STATE_CARDINALITY),
        limits: descriptor_limits(machine.limits())?,
        nodes: parts.nodes,
        edges: parts.edges,
        dependencies: parts.dependencies,
        topological_order: parts.topological_order,
        effects: parts.effects,
        suspensions,
        compatibility: CompatibilityMetadata {
            execution_graph_revision: String::from(EXECUTION_GRAPH_REVISION),
            lattice_revision: String::from(LATTICE_REVISION),
            kamacite_revision: String::from(KAMACITE_REVISION),
        },
        checked_laws: vec![
            String::from("bounded-normalized-admitted-machine"),
            String::from("parameterized-aggregate-state"),
            String::from("exact-lifecycle-profile-binding"),
            String::from("durable-suspension-exact-reference-binding"),
        ],
        non_claims: vec![
            String::from("does not enumerate aggregate product states"),
            String::from("does not prove Nickel evaluation semantics"),
            String::from("does not authorize or execute runtime effects"),
            String::from("does not establish release eligibility"),
            String::from(
                "does not prove approval authority, signal truth, or persistence durability",
            ),
        ],
    })
}

pub fn encode_durable_machine_descriptor(
    descriptor: &DurableMachineDescriptor,
) -> Result<CanonicalArtifact, AggregateError> {
    encode(
        descriptor,
        MACHINE_SCHEMA_VERSION,
        crate::codec::graph::DURABLE_MACHINE_ROOT_LABEL,
    )
    .map_err(Into::into)
}

fn durable_machine_spec(
    descriptor: &DurableMachineDescriptor,
) -> Result<MachineSpec, AggregateError> {
    let mut spec = MachineSpec::new(
        descriptor
            .nodes
            .iter()
            .map(|node| NodeSpec::new(node.node_id.clone(), node.operations.clone()))
            .collect(),
        descriptor
            .edges
            .iter()
            .map(|edge| EdgeSpec::new(edge.source.clone(), edge.target.clone()))
            .collect(),
    );
    spec.schema = descriptor.machine_schema.clone();
    spec.lifecycle_profile = descriptor.lifecycle_profile_id.clone();
    spec.effects = descriptor
        .effects
        .iter()
        .map(|effect| {
            Ok(EffectDeclarationSpec {
                node_id: effect.node_id.clone(),
                transition: TransitionSelector {
                    from: state_from_id(&effect.from_state)?,
                    action: action_from_id(&effect.action)?,
                },
                operation: effect.operation.clone(),
                resource: effect.resource.clone(),
                ability: effect.ability.clone(),
                request: effect.request.clone(),
                payload: effect.payload.clone(),
            })
        })
        .collect::<Result<Vec<_>, AggregateError>>()?;
    spec.suspensions = descriptor
        .suspensions
        .iter()
        .map(
            |suspension| choregraph_core::model::pause::SuspensionDeclarationSpec {
                node_id: suspension.node_id.clone(),
                class: suspension.class.clone(),
                reference: suspension.reference.clone(),
            },
        )
        .collect();
    Ok(spec)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedDurableMachineArtifact {
    pub machine: AdmittedMachine,
    pub descriptor: DurableMachineDescriptor,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

pub fn validate_durable_machine_artifact(
    input: &[u8],
    lifecycle: &LifecycleArtifact,
    expected_profile_identity: &ArtifactIdentity,
) -> Result<ValidatedDurableMachineArtifact, AggregateError> {
    expected_profile_identity.validate(ArtifactDomain::Profile)?;
    lifecycle
        .project_identity
        .validate(ArtifactDomain::Lifecycle)?;
    let (descriptor, canonical): (DurableMachineDescriptor, CanonicalArtifact) = decode(
        input,
        MACHINE_SCHEMA_VERSION,
        crate::codec::graph::DURABLE_MACHINE_ROOT_LABEL,
    )?;
    if descriptor.schema_version != DURABLE_DESCRIPTOR_SCHEMA_VERSION
        || descriptor.lifecycle_profile_id != crate::lifecycle::conversion::DURABLE_FSM_ID
        || descriptor.compatibility.kamacite_revision != KAMACITE_REVISION
    {
        return Err(AggregateError::UnsupportedProfile);
    }
    if descriptor.global_state_model != GLOBAL_STATE_MODEL
        || descriptor.global_state_cardinality != GLOBAL_STATE_CARDINALITY
    {
        return Err(AggregateError::GlobalStateEnumeration);
    }
    if descriptor.lifecycle_profile_identity != lifecycle.project_identity
        || descriptor.kamacite_profile_identity != *expected_profile_identity
    {
        return Err(AggregateError::UnsupportedProfile);
    }
    if descriptor.non_claims.len() != DURABLE_MACHINE_NON_CLAIM_COUNT {
        return Err(AggregateError::DescriptorMismatch);
    }
    let limits = admission_limits(&descriptor.limits)?;
    let spec = durable_machine_spec(&descriptor)?;
    let machine = AdmittedMachine::admit(spec, limits)
        .map_err(|_diagnostics| AggregateError::AdmissionRejected)?;
    let expected = durable_machine_descriptor(
        &machine,
        lifecycle.project_identity.clone(),
        expected_profile_identity.clone(),
    )?;
    if descriptor != expected {
        return Err(AggregateError::DescriptorMismatch);
    }
    let identity = compute_identity(ArtifactDomain::Machine, &canonical.packed_bytes)?;
    Ok(ValidatedDurableMachineArtifact {
        machine,
        descriptor,
        canonical,
        identity,
    })
}

pub fn validate_machine_artifact(
    input: &[u8],
    lifecycle: &LifecycleArtifact,
    expected_profile_identity: &ArtifactIdentity,
) -> Result<ValidatedMachineArtifact, AggregateError> {
    expected_profile_identity.validate(ArtifactDomain::Profile)?;
    lifecycle
        .project_identity
        .validate(ArtifactDomain::Lifecycle)?;
    let (descriptor, canonical): (AggregateMachineDescriptor, CanonicalArtifact) =
        decode(input, MACHINE_SCHEMA_VERSION, MACHINE_ROOT_LABEL)?;
    if descriptor.schema_version != DESCRIPTOR_SCHEMA_VERSION
        || descriptor.lifecycle_profile_id != LIFECYCLE_FSM_ID
        || descriptor.compatibility.kamacite_revision != KAMACITE_REVISION
    {
        return Err(AggregateError::UnsupportedProfile);
    }
    if descriptor.global_state_model != GLOBAL_STATE_MODEL
        || descriptor.global_state_cardinality != GLOBAL_STATE_CARDINALITY
    {
        return Err(AggregateError::GlobalStateEnumeration);
    }
    if descriptor.lifecycle_profile_identity != lifecycle.project_identity
        || descriptor.kamacite_profile_identity != *expected_profile_identity
    {
        return Err(AggregateError::UnsupportedProfile);
    }
    if descriptor.non_claims.len() != MACHINE_NON_CLAIM_COUNT {
        return Err(AggregateError::DescriptorMismatch);
    }
    let limits = admission_limits(&descriptor.limits)?;
    let spec = machine_spec(&descriptor)?;
    let machine = AdmittedMachine::admit(spec, limits)
        .map_err(|_diagnostics| AggregateError::AdmissionRejected)?;
    let expected = machine_descriptor(
        &machine,
        lifecycle.project_identity.clone(),
        expected_profile_identity.clone(),
    )?;
    if descriptor != expected {
        return Err(AggregateError::DescriptorMismatch);
    }
    let identity = compute_identity(ArtifactDomain::Machine, &canonical.packed_bytes)?;
    Ok(ValidatedMachineArtifact {
        machine,
        descriptor,
        canonical,
        identity,
    })
}
