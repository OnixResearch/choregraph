// Canonical v2 machine, concrete-intent, and linkage receipt adapters.
#![allow(
    unknown_lints,
    assertion_density,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "the canonical v2 adapter keeps explicit closed-schema conversion and linkage checks"
)]

// r[impl choregraph.effect_v2.kamacite]
// r[impl choregraph.effect_v2.references]
// r[impl choregraph.effect_v2.occurrence_identity]
use crate::aggregate::{
    AggregateError, DescriptorEdge, DescriptorLimits, DescriptorNode, action_from_id, action_id,
    admission_limits, descriptor_limits, state_from_id, state_id,
};
use crate::codec::{
    CONCRETE_INTENT_RECEIPT_ROOT_LABEL, CONCRETE_INTENT_RECEIPT_SCHEMA_VERSION,
    CONCRETE_INTENT_ROOT_LABEL, CONCRETE_INTENT_SCHEMA_VERSION, CONCRETE_MACHINE_ROOT_LABEL,
    CONCRETE_MACHINE_SCHEMA_VERSION, CanonicalArtifact, CodecBoundaryError, decode, encode,
};
use crate::identity::{ArtifactDomain, ArtifactIdentity, IdentityError, compute_identity};
use choregraph_core::{
    AbilityId, AdmittedMachine, CanonicalReferenceProfile, ConcreteEffectDeclarationSpec,
    ConcreteEffectIntent, ConcreteReference, ConcreteReferenceSpec, DeclarationKey, EdgeSpec,
    LINEAR_LIFECYCLE_V1, MACHINE_SCHEMA_V2, MachineSpecV2, NodeId, NodeSpec, OperationId,
    PayloadRef, ReferenceBinding, ReferenceBindingSpec, ResourceId, SlotId, SourceRef,
    TransitionSelector,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

const SUPPORTED_SCHEMA_VERSION: u32 = 1;
const MACHINE_NON_CLAIM_COUNT: usize = 4;
const INTENT_NON_CLAIM_COUNT: usize = 4;
const RECEIPT_NON_CLAIM_COUNT: usize = 4;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DescriptorReferenceBindingV2 {
    pub mode: String,
    pub profile: Option<String>,
    pub identity: Option<String>,
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorConcreteEffectV2 {
    pub declaration_key: String,
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request_binding: DescriptorReferenceBindingV2,
    pub payload_binding: DescriptorReferenceBindingV2,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-machine-v2", deny_unknown_fields)]
pub struct ConcreteMachineDescriptorV2 {
    pub schema_version: u32,
    pub machine_schema: String,
    pub lifecycle_profile_id: String,
    pub limits: DescriptorLimits,
    pub nodes: Vec<DescriptorNode>,
    pub edges: Vec<DescriptorEdge>,
    pub effects: Vec<DescriptorConcreteEffectV2>,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DescriptorSelectedReferenceV2 {
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
    pub profile: String,
    pub identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-concrete-intent-v2", deny_unknown_fields)]
pub struct ConcreteIntentDescriptorV2 {
    pub schema_version: u32,
    pub declaration_key: String,
    pub node_id: String,
    pub from_state: String,
    pub action: String,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: DescriptorSelectedReferenceV2,
    pub payload: DescriptorSelectedReferenceV2,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-concrete-intent-receipt-v1", deny_unknown_fields)]
pub struct ConcreteIntentReceiptBody {
    pub schema_version: u32,
    pub machine_identity: ArtifactIdentity,
    pub intent_identity: ArtifactIdentity,
    pub declaration_key: String,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
    pub semantic_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteMachineArtifact {
    pub machine: AdmittedMachine,
    pub descriptor: ConcreteMachineDescriptorV2,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteIntentArtifact {
    pub intent: ConcreteEffectIntent,
    pub descriptor: ConcreteIntentDescriptorV2,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteIntentReceiptArtifact {
    pub body: ConcreteIntentReceiptBody,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConcreteArtifactError {
    Aggregate(AggregateError),
    Codec(CodecBoundaryError),
    Identity(IdentityError),
    UnsupportedSchema,
    DescriptorMismatch,
    MachineAdmissionRejected,
    IntentMismatch,
    StaleIdentity,
    Overclaim,
}

impl Display for ConcreteArtifactError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Aggregate(error) => Display::fmt(error, formatter),
            Self::Codec(error) => Display::fmt(error, formatter),
            Self::Identity(error) => Display::fmt(error, formatter),
            Self::UnsupportedSchema => formatter.write_str("unsupported concrete artifact schema"),
            Self::DescriptorMismatch => {
                formatter.write_str("concrete artifact does not match its normalized value")
            }
            Self::MachineAdmissionRejected => {
                formatter.write_str("decoded v2 machine failed core admission")
            }
            Self::IntentMismatch => {
                formatter.write_str("concrete intent does not match its admitted declaration")
            }
            Self::StaleIdentity => formatter.write_str("concrete artifact identity is stale"),
            Self::Overclaim => formatter.write_str("concrete intent receipt contains an overclaim"),
        }
    }
}

impl std::error::Error for ConcreteArtifactError {}

impl From<AggregateError> for ConcreteArtifactError {
    fn from(error: AggregateError) -> Self {
        Self::Aggregate(error)
    }
}

impl From<CodecBoundaryError> for ConcreteArtifactError {
    fn from(error: CodecBoundaryError) -> Self {
        Self::Codec(error)
    }
}

impl From<IdentityError> for ConcreteArtifactError {
    fn from(error: IdentityError) -> Self {
        Self::Identity(error)
    }
}

fn machine_non_claims() -> Vec<String> {
    vec![
        String::from("machine artifact does not prove request truth"),
        String::from("machine artifact does not prove payload availability"),
        String::from("machine artifact does not authorize or execute effects"),
        String::from("machine artifact does not establish release eligibility"),
    ]
}

fn intent_non_claims() -> Vec<String> {
    vec![
        String::from("intent identity identifies content and not a run occurrence"),
        String::from("intent artifact does not prove request or payload truth"),
        String::from("intent artifact does not grant authorization or dispatch authority"),
        String::from("intent artifact does not prove external effect success"),
    ]
}

fn receipt_non_claims() -> Vec<String> {
    vec![
        String::from("receipt linkage does not prove runtime authorization"),
        String::from("receipt linkage does not prove request or payload truth"),
        String::from("receipt linkage does not create an occurrence identity"),
        String::from("receipt linkage does not prove dispatch or external success"),
    ]
}

fn descriptor_binding(binding: &ReferenceBinding) -> DescriptorReferenceBindingV2 {
    match binding {
        ReferenceBinding::Exact(reference) => DescriptorReferenceBindingV2 {
            mode: String::from("exact"),
            profile: Some(String::from(reference.profile.as_str())),
            identity: Some(String::from(reference.identity.as_str())),
            slot_id: None,
            source_ref: None,
        },
        ReferenceBinding::Slot {
            slot_id,
            source_ref,
        } => DescriptorReferenceBindingV2 {
            mode: String::from("slot"),
            profile: None,
            identity: None,
            slot_id: Some(String::from(slot_id.as_str())),
            source_ref: Some(String::from(source_ref.as_str())),
        },
    }
}

fn binding_spec(
    binding: &DescriptorReferenceBindingV2,
) -> Result<ReferenceBindingSpec, ConcreteArtifactError> {
    match binding.mode.as_str() {
        "exact"
            if binding.slot_id.is_none()
                && binding.source_ref.is_none()
                && binding.profile.is_some()
                && binding.identity.is_some() =>
        {
            Ok(ReferenceBindingSpec::Exact {
                reference: ConcreteReferenceSpec {
                    profile: binding.profile.clone().unwrap_or_default(),
                    identity: binding.identity.clone().unwrap_or_default(),
                },
            })
        }
        "slot"
            if binding.profile.is_none()
                && binding.identity.is_none()
                && binding.slot_id.is_some()
                && binding.source_ref.is_some() =>
        {
            Ok(ReferenceBindingSpec::Slot {
                slot_id: binding.slot_id.clone().unwrap_or_default(),
                source_ref: binding.source_ref.clone().unwrap_or_default(),
            })
        }
        _ => Err(ConcreteArtifactError::DescriptorMismatch),
    }
}

fn machine_descriptor(
    machine: &AdmittedMachine,
) -> Result<ConcreteMachineDescriptorV2, ConcreteArtifactError> {
    if machine.schema() != MACHINE_SCHEMA_V2 {
        return Err(ConcreteArtifactError::UnsupportedSchema);
    }
    let mut nodes = Vec::with_capacity(machine.node_ids().len());
    for node_id in machine.node_ids() {
        let operations = machine
            .operations(node_id)
            .ok_or(ConcreteArtifactError::DescriptorMismatch)?
            .iter()
            .map(|operation| String::from(operation.as_str()))
            .collect();
        nodes.push(DescriptorNode {
            node_id: String::from(node_id.as_str()),
            operations,
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
    let effects = machine
        .concrete_effect_declarations()
        .iter()
        .map(|declaration| DescriptorConcreteEffectV2 {
            declaration_key: String::from(declaration.declaration_key.as_str()),
            node_id: String::from(declaration.node_id.as_str()),
            from_state: String::from(state_id(declaration.transition.from)),
            action: String::from(action_id(declaration.transition.action)),
            operation: String::from(declaration.operation.as_str()),
            resource: String::from(declaration.resource.as_str()),
            ability: String::from(declaration.ability.as_str()),
            request_binding: descriptor_binding(&declaration.request_binding),
            payload_binding: descriptor_binding(&declaration.payload_binding),
        })
        .collect();
    Ok(ConcreteMachineDescriptorV2 {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        machine_schema: String::from(MACHINE_SCHEMA_V2),
        lifecycle_profile_id: String::from(LINEAR_LIFECYCLE_V1),
        limits: descriptor_limits(machine.limits())?,
        nodes,
        edges,
        effects,
        checked_laws: vec![
            String::from("bounded-normalized-v2-machine"),
            String::from("unique-declaration-keys"),
            String::from("exact-or-slot-reference-bindings"),
        ],
        non_claims: machine_non_claims(),
    })
}

fn machine_spec(
    descriptor: &ConcreteMachineDescriptorV2,
) -> Result<MachineSpecV2, ConcreteArtifactError> {
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
            Ok(ConcreteEffectDeclarationSpec {
                declaration_key: effect.declaration_key.clone(),
                node_id: effect.node_id.clone(),
                transition: TransitionSelector {
                    from: state_from_id(&effect.from_state)?,
                    action: action_from_id(&effect.action)?,
                },
                operation: effect.operation.clone(),
                resource: effect.resource.clone(),
                ability: effect.ability.clone(),
                request_binding: binding_spec(&effect.request_binding)?,
                payload_binding: binding_spec(&effect.payload_binding)?,
            })
        })
        .collect::<Result<Vec<_>, ConcreteArtifactError>>()?;
    Ok(MachineSpecV2 {
        schema: descriptor.machine_schema.clone(),
        lifecycle_profile: descriptor.lifecycle_profile_id.clone(),
        nodes,
        edges,
        effects,
    })
}

pub fn encode_concrete_machine_descriptor(
    descriptor: &ConcreteMachineDescriptorV2,
) -> Result<CanonicalArtifact, ConcreteArtifactError> {
    encode(
        descriptor,
        CONCRETE_MACHINE_SCHEMA_VERSION,
        CONCRETE_MACHINE_ROOT_LABEL,
    )
    .map_err(Into::into)
}

pub fn build_concrete_machine_artifact(
    machine: &AdmittedMachine,
) -> Result<ConcreteMachineArtifact, ConcreteArtifactError> {
    let descriptor = machine_descriptor(machine)?;
    let canonical = encode_concrete_machine_descriptor(&descriptor)?;
    let identity = compute_identity(ArtifactDomain::Machine, &canonical.packed_bytes)?;
    Ok(ConcreteMachineArtifact {
        machine: machine.clone(),
        descriptor,
        canonical,
        identity,
    })
}

pub fn validate_concrete_machine_artifact(
    input: &[u8],
    declared_identity: &ArtifactIdentity,
) -> Result<ConcreteMachineArtifact, ConcreteArtifactError> {
    declared_identity.validate(ArtifactDomain::Machine)?;
    let (descriptor, canonical): (ConcreteMachineDescriptorV2, CanonicalArtifact) = decode(
        input,
        CONCRETE_MACHINE_SCHEMA_VERSION,
        CONCRETE_MACHINE_ROOT_LABEL,
    )?;
    if descriptor.schema_version != SUPPORTED_SCHEMA_VERSION
        || descriptor.machine_schema != MACHINE_SCHEMA_V2
        || descriptor.lifecycle_profile_id != LINEAR_LIFECYCLE_V1
        || descriptor.non_claims != machine_non_claims()
        || descriptor.non_claims.len() != MACHINE_NON_CLAIM_COUNT
    {
        return Err(ConcreteArtifactError::UnsupportedSchema);
    }
    let limits = admission_limits(&descriptor.limits)?;
    let machine = AdmittedMachine::admit_v2(machine_spec(&descriptor)?, limits)
        .map_err(|_| ConcreteArtifactError::MachineAdmissionRejected)?;
    if descriptor != machine_descriptor(&machine)? {
        return Err(ConcreteArtifactError::DescriptorMismatch);
    }
    let identity = compute_identity(ArtifactDomain::Machine, &canonical.packed_bytes)?;
    if &identity != declared_identity {
        return Err(ConcreteArtifactError::StaleIdentity);
    }
    Ok(ConcreteMachineArtifact {
        machine,
        descriptor,
        canonical,
        identity,
    })
}

fn selected_reference(
    slot_id: Option<&SlotId>,
    source_ref: Option<&SourceRef>,
    reference: &ConcreteReference,
) -> DescriptorSelectedReferenceV2 {
    DescriptorSelectedReferenceV2 {
        slot_id: slot_id.map(|value| String::from(value.as_str())),
        source_ref: source_ref.map(|value| String::from(value.as_str())),
        profile: String::from(reference.profile.as_str()),
        identity: String::from(reference.identity.as_str()),
    }
}

fn intent_descriptor(intent: &ConcreteEffectIntent) -> ConcreteIntentDescriptorV2 {
    ConcreteIntentDescriptorV2 {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        declaration_key: String::from(intent.declaration_key.as_str()),
        node_id: String::from(intent.node_id.as_str()),
        from_state: String::from(state_id(intent.transition.from)),
        action: String::from(action_id(intent.transition.action)),
        operation: String::from(intent.operation.as_str()),
        resource: String::from(intent.resource.as_str()),
        ability: String::from(intent.ability.as_str()),
        request: selected_reference(
            intent.request_slot.as_ref(),
            intent.request_source_ref.as_ref(),
            &intent.request_reference,
        ),
        payload: selected_reference(
            intent.payload_slot.as_ref(),
            intent.payload_source_ref.as_ref(),
            &intent.payload_reference,
        ),
        non_claims: intent_non_claims(),
    }
}

fn parse_selected_reference(
    descriptor: &DescriptorSelectedReferenceV2,
    limits: choregraph_core::AdmissionLimits,
) -> Result<(Option<SlotId>, Option<SourceRef>, ConcreteReference), ConcreteArtifactError> {
    let profile = match descriptor.profile.as_str() {
        choregraph_core::CANONICAL_REFERENCE_PROFILE_V1 => {
            CanonicalReferenceProfile::PackedPreservesV1
        }
        _ => return Err(ConcreteArtifactError::IntentMismatch),
    };
    let identity = PayloadRef::parse(&descriptor.identity, limits.max_reference_bytes)
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let slot_id = descriptor
        .slot_id
        .as_deref()
        .map(|value| SlotId::parse(value, limits.max_identifier_bytes))
        .transpose()
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let source_ref = descriptor
        .source_ref
        .as_deref()
        .map(|value| SourceRef::parse(value, limits.max_identifier_bytes))
        .transpose()
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    Ok((slot_id, source_ref, ConcreteReference { profile, identity }))
}

fn intent_from_descriptor(
    descriptor: &ConcreteIntentDescriptorV2,
    machine: &AdmittedMachine,
) -> Result<ConcreteEffectIntent, ConcreteArtifactError> {
    let limits = machine.limits();
    let declaration_key =
        DeclarationKey::parse(&descriptor.declaration_key, limits.max_identifier_bytes)
            .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let node_id = NodeId::parse(&descriptor.node_id, limits.max_identifier_bytes)
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let operation = OperationId::parse(&descriptor.operation, limits.max_identifier_bytes)
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let resource = ResourceId::parse(&descriptor.resource, limits.max_identifier_bytes)
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let ability = AbilityId::parse(&descriptor.ability, limits.max_identifier_bytes)
        .map_err(|_| ConcreteArtifactError::IntentMismatch)?;
    let request = parse_selected_reference(&descriptor.request, limits)?;
    let payload = parse_selected_reference(&descriptor.payload, limits)?;
    Ok(ConcreteEffectIntent {
        declaration_key,
        node_id,
        transition: TransitionSelector {
            from: state_from_id(&descriptor.from_state)?,
            action: action_from_id(&descriptor.action)?,
        },
        operation,
        resource,
        ability,
        request_slot: request.0,
        request_source_ref: request.1,
        request_reference: request.2,
        payload_slot: payload.0,
        payload_source_ref: payload.1,
        payload_reference: payload.2,
    })
}

fn selected_binding_matches(
    binding: &ReferenceBinding,
    slot_id: Option<&SlotId>,
    source_ref: Option<&SourceRef>,
    reference: &ConcreteReference,
) -> bool {
    match binding {
        ReferenceBinding::Exact(expected) => {
            slot_id.is_none() && source_ref.is_none() && expected == reference
        }
        ReferenceBinding::Slot {
            slot_id: expected_slot,
            source_ref: expected_source,
        } => slot_id == Some(expected_slot) && source_ref == Some(expected_source),
    }
}

fn intent_matches_machine(machine: &AdmittedMachine, intent: &ConcreteEffectIntent) -> bool {
    machine
        .concrete_effect_declarations()
        .iter()
        .find(|declaration| declaration.declaration_key == intent.declaration_key)
        .is_some_and(|declaration| {
            declaration.node_id == intent.node_id
                && declaration.transition == intent.transition
                && declaration.operation == intent.operation
                && declaration.resource == intent.resource
                && declaration.ability == intent.ability
                && selected_binding_matches(
                    &declaration.request_binding,
                    intent.request_slot.as_ref(),
                    intent.request_source_ref.as_ref(),
                    &intent.request_reference,
                )
                && selected_binding_matches(
                    &declaration.payload_binding,
                    intent.payload_slot.as_ref(),
                    intent.payload_source_ref.as_ref(),
                    &intent.payload_reference,
                )
        })
}

pub fn encode_concrete_intent_descriptor(
    descriptor: &ConcreteIntentDescriptorV2,
) -> Result<CanonicalArtifact, ConcreteArtifactError> {
    encode(
        descriptor,
        CONCRETE_INTENT_SCHEMA_VERSION,
        CONCRETE_INTENT_ROOT_LABEL,
    )
    .map_err(Into::into)
}

pub fn build_concrete_intent_artifact(
    machine: &AdmittedMachine,
    intent: &ConcreteEffectIntent,
) -> Result<ConcreteIntentArtifact, ConcreteArtifactError> {
    if !intent_matches_machine(machine, intent) {
        return Err(ConcreteArtifactError::IntentMismatch);
    }
    let descriptor = intent_descriptor(intent);
    let canonical = encode_concrete_intent_descriptor(&descriptor)?;
    let identity = compute_identity(ArtifactDomain::ConcreteIntent, &canonical.packed_bytes)?;
    Ok(ConcreteIntentArtifact {
        intent: intent.clone(),
        descriptor,
        canonical,
        identity,
    })
}

pub fn validate_concrete_intent_artifact(
    input: &[u8],
    declared_identity: &ArtifactIdentity,
    machine: &AdmittedMachine,
) -> Result<ConcreteIntentArtifact, ConcreteArtifactError> {
    declared_identity.validate(ArtifactDomain::ConcreteIntent)?;
    let (descriptor, canonical): (ConcreteIntentDescriptorV2, CanonicalArtifact) = decode(
        input,
        CONCRETE_INTENT_SCHEMA_VERSION,
        CONCRETE_INTENT_ROOT_LABEL,
    )?;
    if descriptor.schema_version != SUPPORTED_SCHEMA_VERSION
        || descriptor.non_claims != intent_non_claims()
        || descriptor.non_claims.len() != INTENT_NON_CLAIM_COUNT
    {
        return Err(ConcreteArtifactError::UnsupportedSchema);
    }
    let intent = intent_from_descriptor(&descriptor, machine)?;
    if !intent_matches_machine(machine, &intent) || descriptor != intent_descriptor(&intent) {
        return Err(ConcreteArtifactError::IntentMismatch);
    }
    let identity = compute_identity(ArtifactDomain::ConcreteIntent, &canonical.packed_bytes)?;
    if &identity != declared_identity {
        return Err(ConcreteArtifactError::StaleIdentity);
    }
    Ok(ConcreteIntentArtifact {
        intent,
        descriptor,
        canonical,
        identity,
    })
}

fn expected_receipt_body(
    machine: &ConcreteMachineArtifact,
    intent: &ConcreteIntentArtifact,
) -> ConcreteIntentReceiptBody {
    ConcreteIntentReceiptBody {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        machine_identity: machine.identity.clone(),
        intent_identity: intent.identity.clone(),
        declaration_key: String::from(intent.intent.declaration_key.as_str()),
        checked_laws: vec![
            String::from("machine-identity-matched"),
            String::from("declared-intent-content-identity-matched"),
            String::from("declaration-key-matched"),
        ],
        non_claims: receipt_non_claims(),
        semantic_claims: Vec::new(),
    }
}

pub fn encode_concrete_intent_receipt_body(
    body: &ConcreteIntentReceiptBody,
) -> Result<CanonicalArtifact, ConcreteArtifactError> {
    encode(
        body,
        CONCRETE_INTENT_RECEIPT_SCHEMA_VERSION,
        CONCRETE_INTENT_RECEIPT_ROOT_LABEL,
    )
    .map_err(Into::into)
}

pub fn build_concrete_intent_receipt(
    machine: &ConcreteMachineArtifact,
    intent: &ConcreteIntentArtifact,
) -> Result<ConcreteIntentReceiptArtifact, ConcreteArtifactError> {
    machine.identity.validate(ArtifactDomain::Machine)?;
    intent.identity.validate(ArtifactDomain::ConcreteIntent)?;
    if !intent_matches_machine(&machine.machine, &intent.intent) {
        return Err(ConcreteArtifactError::IntentMismatch);
    }
    let body = expected_receipt_body(machine, intent);
    let canonical = encode_concrete_intent_receipt_body(&body)?;
    let identity = compute_identity(ArtifactDomain::Receipt, &canonical.packed_bytes)?;
    Ok(ConcreteIntentReceiptArtifact {
        body,
        canonical,
        identity,
    })
}

pub fn validate_concrete_intent_receipt(
    input: &[u8],
    declared_identity: &ArtifactIdentity,
    machine: &ConcreteMachineArtifact,
    intent: &ConcreteIntentArtifact,
) -> Result<ConcreteIntentReceiptArtifact, ConcreteArtifactError> {
    declared_identity.validate(ArtifactDomain::Receipt)?;
    let (body, canonical): (ConcreteIntentReceiptBody, CanonicalArtifact) = decode(
        input,
        CONCRETE_INTENT_RECEIPT_SCHEMA_VERSION,
        CONCRETE_INTENT_RECEIPT_ROOT_LABEL,
    )?;
    if !body.semantic_claims.is_empty()
        || body.non_claims != receipt_non_claims()
        || body.non_claims.len() != RECEIPT_NON_CLAIM_COUNT
    {
        return Err(ConcreteArtifactError::Overclaim);
    }
    if body != expected_receipt_body(machine, intent) {
        return Err(ConcreteArtifactError::DescriptorMismatch);
    }
    let identity = compute_identity(ArtifactDomain::Receipt, &canonical.packed_bytes)?;
    if &identity != declared_identity {
        return Err(ConcreteArtifactError::StaleIdentity);
    }
    Ok(ConcreteIntentReceiptArtifact {
        body,
        canonical,
        identity,
    })
}
