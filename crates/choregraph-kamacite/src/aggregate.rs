// r[impl choregraph.interchange.kamacite.aggregate]
// r[impl choregraph.interchange.kamacite.adapter]
// r[impl choregraph.interchange.kamacite.identity]
use crate::codec::{
    ADMITTED_IR_ROOT_LABEL, ADMITTED_IR_SCHEMA_VERSION, CanonicalArtifact, CodecBoundaryError,
    MACHINE_ROOT_LABEL, MACHINE_SCHEMA_VERSION, PROFILE_ROOT_LABEL, PROFILE_SCHEMA_VERSION, decode,
    encode,
};
use crate::identity::{
    ArtifactDomain, ArtifactIdentity, IdentityChain, IdentityError, KamaciteIdentityDomain,
    compute_identity, compute_kamacite_identity,
};
use crate::lifecycle::{
    LIFECYCLE_FSM_ID, LifecycleArtifact, LifecycleError, build_lifecycle_artifact,
};
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, EdgeSpec, EffectDeclarationSpec, LocalState,
    MachineSpec, NodeSpec, TransitionSelector,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub const KAMACITE_REVISION: &str = "6245a52498f7de511df660eabec3f9b350338cb6";
pub const EXECUTION_GRAPH_REVISION: &str = "c415ddf2c8252dd9ba6cb267a4e6346158069823";
pub const LATTICE_REVISION: &str = "c8d56ad83a57287064edb85f58808f53e4bf4536";
pub const GLOBAL_STATE_MODEL: &str = "parameterized-map-from-node-id-to-local-state";
pub const GLOBAL_STATE_CARDINALITY: &str = "not-enumerated";
const DESCRIPTOR_SCHEMA_VERSION: u32 = 1;
const PROFILE_SCHEMA_NUMBER: u32 = 1;
const KAMACITE_LICENSE: &str = "MIT OR Apache-2.0";
const MACHINE_SCHEMA_SOURCE_BLAKE3: &str =
    "4cb7079deb17e77fe2806bd24a80f47e5df1ca7c405725429a3ea47236ecb0f6";
const LINK_RECEIPT_SCHEMA_SOURCE_BLAKE3: &str =
    "35fe04687697fe9d8027f924aade0a2ddccb65f5b9843a17c8692b8fa93212c1";
const MACHINE_NON_CLAIM_COUNT: usize = 4;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorLimits {
    pub max_nodes: u64,
    pub max_edges: u64,
    pub max_operations: u64,
    pub max_effect_declarations: u64,
    pub max_identifier_bytes: u64,
    pub max_reference_bytes: u64,
    pub max_event_bytes: u64,
    pub max_intent_bytes: u64,
    pub max_intents_per_transition: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorNode {
    pub node_id: String,
    pub operations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorEdge {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorDependencySet {
    pub node_id: String,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorEffect {
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompatibilityMetadata {
    pub execution_graph_revision: String,
    pub lattice_revision: String,
    pub kamacite_revision: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-machine-v1", deny_unknown_fields)]
pub struct AggregateMachineDescriptor {
    pub schema_version: u32,
    pub machine_schema: String,
    pub lifecycle_profile_id: String,
    pub lifecycle_profile_identity: ArtifactIdentity,
    pub kamacite_profile_identity: ArtifactIdentity,
    pub global_state_model: String,
    pub global_state_cardinality: String,
    pub limits: DescriptorLimits,
    pub nodes: Vec<DescriptorNode>,
    pub edges: Vec<DescriptorEdge>,
    pub dependencies: Vec<DescriptorDependencySet>,
    pub topological_order: Vec<String>,
    pub effects: Vec<DescriptorEffect>,
    pub compatibility: CompatibilityMetadata,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-admitted-ir-v1", deny_unknown_fields)]
struct AdmittedIrDescriptor {
    schema_version: u32,
    machine_schema: String,
    lifecycle_profile_id: String,
    limits: DescriptorLimits,
    nodes: Vec<DescriptorNode>,
    edges: Vec<DescriptorEdge>,
    dependencies: Vec<DescriptorDependencySet>,
    topological_order: Vec<String>,
    effects: Vec<DescriptorEffect>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-kamacite-profile-v1", deny_unknown_fields)]
pub struct SupportedKamaciteProfile {
    pub schema_version: u32,
    pub kamacite_revision: String,
    pub kamacite_license: String,
    pub canonical_codec: String,
    pub preserves_package_version: String,
    pub preserves_spec_version: String,
    pub machine_schema: String,
    pub machine_schema_source_blake3: String,
    pub link_receipt_schema_source_blake3: String,
    pub fsm_spec_schema: String,
    pub fsm_receipt_schema: String,
    pub fsm_determinism_policy: String,
    pub fsm_receipt_kinds: Vec<String>,
    pub fsm_denial_no_effect_class: String,
    pub lifecycle_fsm_id: String,
    pub authority: String,
    pub non_claims: Vec<String>,
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
        match self {
            Self::Codec(error) => Display::fmt(error, formatter),
            Self::Identity(error) => Display::fmt(error, formatter),
            Self::Lifecycle(error) => Display::fmt(error, formatter),
            Self::NumericLimitOverflow => {
                formatter.write_str("descriptor numeric limit does not fit this target")
            }
            Self::UnsupportedProfile => formatter.write_str("unsupported Kamacite profile binding"),
            Self::GlobalStateEnumeration => {
                formatter.write_str("aggregate product-state enumeration is not admitted")
            }
            Self::DescriptorMismatch => {
                formatter.write_str("canonical machine descriptor does not match admitted IR")
            }
            Self::AdmissionRejected => formatter.write_str("decoded machine failed core admission"),
            Self::UnknownLifecycleValue => {
                formatter.write_str("descriptor contains an unknown lifecycle value")
            }
            Self::MissingNormalizedEntry => {
                formatter.write_str("admitted machine is missing a normalized index entry")
            }
            Self::EmptySourceEnvelope => {
                formatter.write_str("source envelope identity material is empty")
            }
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

fn profile() -> SupportedKamaciteProfile {
    SupportedKamaciteProfile {
        schema_version: PROFILE_SCHEMA_NUMBER,
        kamacite_revision: String::from(KAMACITE_REVISION),
        kamacite_license: String::from(KAMACITE_LICENSE),
        canonical_codec: String::from(crate::selected_kamacite_codec::PRESERVES_CANONICAL_CODEC),
        preserves_package_version: String::from(
            crate::selected_kamacite_codec::PRESERVES_PACKAGE_VERSION,
        ),
        preserves_spec_version: String::from(
            crate::selected_kamacite_codec::PRESERVES_SPEC_VERSION,
        ),
        machine_schema: String::from(MACHINE_SCHEMA_VERSION),
        machine_schema_source_blake3: String::from(MACHINE_SCHEMA_SOURCE_BLAKE3),
        link_receipt_schema_source_blake3: String::from(LINK_RECEIPT_SCHEMA_SOURCE_BLAKE3),
        fsm_spec_schema: String::from(crate::codec::FSM_SPEC_SCHEMA_VERSION),
        fsm_receipt_schema: String::from(crate::codec::FSM_RECEIPT_SCHEMA_VERSION),
        fsm_determinism_policy: String::from("deterministic"),
        fsm_receipt_kinds: vec![String::from("exhaustiveness"), String::from("trace")],
        fsm_denial_no_effect_class: String::from("no-side-effect"),
        lifecycle_fsm_id: String::from(LIFECYCLE_FSM_ID),
        authority: String::from("canonical-packed-preserves"),
        non_claims: vec![
            String::from("profile identity does not prove transition correctness"),
            String::from("profile identity does not prove source equivalence"),
            String::from("profile identity does not authorize runtime effects"),
        ],
    }
}

pub fn build_profile_artifact() -> Result<
    (
        SupportedKamaciteProfile,
        CanonicalArtifact,
        ArtifactIdentity,
    ),
    AggregateError,
> {
    let profile = profile();
    let artifact = encode(&profile, PROFILE_SCHEMA_VERSION, PROFILE_ROOT_LABEL)?;
    let identity = compute_identity(ArtifactDomain::Profile, &artifact.packed_bytes)?;
    Ok((profile, artifact, identity))
}

fn usize_to_u64(value: usize) -> Result<u64, AggregateError> {
    u64::try_from(value).map_err(|_| AggregateError::NumericLimitOverflow)
}

fn u64_to_usize(value: u64) -> Result<usize, AggregateError> {
    usize::try_from(value).map_err(|_| AggregateError::NumericLimitOverflow)
}

fn descriptor_limits(limits: AdmissionLimits) -> Result<DescriptorLimits, AggregateError> {
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

fn admission_limits(limits: &DescriptorLimits) -> Result<AdmissionLimits, AggregateError> {
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

fn state_id(state: LocalState) -> &'static str {
    match state {
        LocalState::Pending => "pending",
        LocalState::Ready => "ready",
        LocalState::Running => "running",
        LocalState::Completed => "completed",
        LocalState::Failed => "failed",
        LocalState::Cancelled => "cancelled",
    }
}

fn action_id(action: Action) -> &'static str {
    match action {
        Action::MarkReady => "mark-ready",
        Action::Start => "start",
        Action::Complete => "complete",
        Action::Fail => "fail",
        Action::Cancel => "cancel",
    }
}

fn action_from_id(value: &str) -> Result<Action, AggregateError> {
    match value {
        "mark-ready" => Ok(Action::MarkReady),
        "start" => Ok(Action::Start),
        "complete" => Ok(Action::Complete),
        "fail" => Ok(Action::Fail),
        "cancel" => Ok(Action::Cancel),
        _ => Err(AggregateError::UnknownLifecycleValue),
    }
}

fn state_from_id(value: &str) -> Result<LocalState, AggregateError> {
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
            .ok_or(AggregateError::MissingNormalizedEntry)?
            .iter()
            .map(|operation| String::from(operation.as_str()))
            .collect();
        nodes.push(DescriptorNode {
            node_id: String::from(node_id.as_str()),
            operations,
        });
        let dependency_ids = machine
            .dependencies(node_id)
            .ok_or(AggregateError::MissingNormalizedEntry)?
            .iter()
            .map(|dependency| String::from(dependency.as_str()))
            .collect();
        dependencies.push(DescriptorDependencySet {
            node_id: String::from(node_id.as_str()),
            dependencies: dependency_ids,
        });
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

fn admitted_ir_descriptor(
    machine: &AdmittedMachine,
) -> Result<AdmittedIrDescriptor, AggregateError> {
    let parts = normalized_parts(machine)?;
    Ok(AdmittedIrDescriptor {
        schema_version: DESCRIPTOR_SCHEMA_VERSION,
        machine_schema: String::from(machine.schema()),
        lifecycle_profile_id: String::from(LIFECYCLE_FSM_ID),
        limits: descriptor_limits(machine.limits())?,
        nodes: parts.nodes,
        edges: parts.edges,
        dependencies: parts.dependencies,
        topological_order: parts.topological_order,
        effects: parts.effects,
    })
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
    let admitted_ir = admitted_ir_descriptor(machine)?;
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
