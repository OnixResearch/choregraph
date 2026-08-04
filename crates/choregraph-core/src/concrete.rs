// Concrete-effect admission and binding resolution stay pure and deterministic.
#![allow(
    unknown_lints,
    allocation_in_loop,
    ambiguous_params,
    assertion_density,
    borrowed_argument_types,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    unbounded_collection_growth,
    reason = "the bounded concrete-effect core keeps explicit closed-schema control flow"
)]

// r[impl choregraph.effect_v2.schema]
// r[impl choregraph.effect_v2.declarations]
// r[impl choregraph.effect_v2.bindings]
// r[impl choregraph.effect_v2.references]
// r[impl choregraph.effect_v2.determinism]
// r[impl choregraph.effect_v2.diagnostics]
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::id::{
    AbilityId, BoundedId, DeclarationKey, IdentifierError, NodeId, OperationId, PayloadRef,
    PayloadRefError, ResourceId, SlotId, SourceRef,
};
use crate::invariant::well_formed;
use crate::model::{
    Action, AdmissionDiagnostic, AdmissionDiagnosticCode, AdmissionLimits, AdmittedMachine,
    EdgeSpec, EffectKey, LINEAR_LIFECYCLE_V1, LimitKind, MACHINE_SCHEMA_V2, MachineSchema,
    MachineSpec, MachineState, NodeSpec, ReplayBindings, ReplayLimits, TransitionSelector,
};
use crate::transition::lifecycle_target_for_profile;

pub const CANONICAL_REFERENCE_PROFILE_V1: &str = "choregraph.packed-preserves.v1";
const EVENT_ACTION_TAG_BYTES: usize = 1;

/// The closed canonical encoding profile accepted by the pure core.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalReferenceProfile {
    PackedPreservesV1,
}

impl CanonicalReferenceProfile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PackedPreservesV1 => CANONICAL_REFERENCE_PROFILE_V1,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            CANONICAL_REFERENCE_PROFILE_V1 => Some(Self::PackedPreservesV1),
            _ => None,
        }
    }
}

/// One untrusted canonical reference supplied by a schema or event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteReferenceSpec {
    pub profile: String,
    pub identity: String,
}

/// One admitted canonical packed Preserves reference.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConcreteReference {
    pub profile: CanonicalReferenceProfile,
    pub identity: PayloadRef,
}

/// One untrusted exact or runtime-slot binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceBindingSpec {
    Exact { reference: ConcreteReferenceSpec },
    Slot { slot_id: String, source_ref: String },
}

/// One admitted exact or runtime-slot binding.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReferenceBinding {
    Exact(ConcreteReference),
    Slot {
        slot_id: SlotId,
        source_ref: SourceRef,
    },
}

/// One untrusted v2 effect declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteEffectDeclarationSpec {
    pub declaration_key: String,
    pub node_id: String,
    pub transition: TransitionSelector,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request_binding: ReferenceBindingSpec,
    pub payload_binding: ReferenceBindingSpec,
}

/// One admitted immutable v2 effect declaration.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConcreteEffectDeclaration {
    pub declaration_key: DeclarationKey,
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request_binding: ReferenceBinding,
    pub payload_binding: ReferenceBinding,
}

/// One untrusted v2 machine specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineSpecV2 {
    pub schema: String,
    pub lifecycle_profile: String,
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<EdgeSpec>,
    pub effects: Vec<ConcreteEffectDeclarationSpec>,
    pub suspensions: Vec<crate::model::SuspensionDeclarationSpec>,
}

impl MachineSpecV2 {
    #[must_use]
    pub fn new(nodes: Vec<NodeSpec>, edges: Vec<EdgeSpec>) -> Self {
        Self {
            schema: String::from(MACHINE_SCHEMA_V2),
            lifecycle_profile: String::from(LINEAR_LIFECYCLE_V1),
            nodes,
            edges,
            effects: Vec::new(),
            suspensions: Vec::new(),
        }
    }
}

/// One concrete reference and its selected slot facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteBindingInput {
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
    pub reference: ConcreteReferenceSpec,
}

/// One v2 effect request carried by an event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteEffectRequest {
    pub declaration_key: String,
    pub request: ConcreteBindingInput,
    pub payload: ConcreteBindingInput,
}

/// One bounded v2 lifecycle event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteEvent {
    pub node_id: String,
    pub action: Action,
    pub effects: Vec<ConcreteEffectRequest>,
    pub suspension: Option<crate::model::SuspensionFactSpec>,
}

impl ConcreteEvent {
    #[must_use]
    pub fn new(node_id: impl Into<String>, action: Action) -> Self {
        Self {
            node_id: node_id.into(),
            action,
            effects: Vec::new(),
            suspension: None,
        }
    }

    #[must_use]
    pub fn suspension(
        node_id: impl Into<String>,
        action: Action,
        class: impl Into<String>,
        reference: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            action,
            effects: Vec::new(),
            suspension: Some(crate::model::SuspensionFactSpec {
                class: class.into(),
                reference: reference.into(),
            }),
        }
    }
}

/// One concrete intent emitted by an accepted v2 transition.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConcreteEffectIntent {
    pub declaration_key: DeclarationKey,
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request_slot: Option<SlotId>,
    pub request_source_ref: Option<SourceRef>,
    pub request_reference: ConcreteReference,
    pub payload_slot: Option<SlotId>,
    pub payload_source_ref: Option<SourceRef>,
    pub payload_reference: ConcreteReference,
}

/// The result of one pure v2 transition request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConcreteDecision {
    Accept {
        next: MachineState,
        intents: Vec<ConcreteEffectIntent>,
    },
    Deny {
        reason: crate::DenialReason,
    },
}

/// An ordered v2 event slice with exact canonical identity bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcreteBoundedTrace<'a> {
    bindings: ReplayBindings,
    events: &'a [ConcreteEvent],
}

impl<'a> ConcreteBoundedTrace<'a> {
    #[must_use]
    pub const fn new(bindings: ReplayBindings, events: &'a [ConcreteEvent]) -> Self {
        Self { bindings, events }
    }

    #[must_use]
    pub const fn bindings(&self) -> ReplayBindings {
        self.bindings
    }

    #[must_use]
    pub const fn events(&self) -> &'a [ConcreteEvent] {
        self.events
    }
}

/// All explicit inputs to one deterministic v2 replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcreteReplayInput<'a> {
    pub(crate) machine: &'a AdmittedMachine,
    pub(crate) initial_state: &'a MachineState,
    pub(crate) expected_bindings: ReplayBindings,
    pub(crate) trace: ConcreteBoundedTrace<'a>,
    pub(crate) limits: ReplayLimits,
}

impl<'a> ConcreteReplayInput<'a> {
    #[must_use]
    pub const fn new(
        machine: &'a AdmittedMachine,
        initial_state: &'a MachineState,
        expected_bindings: ReplayBindings,
        trace: ConcreteBoundedTrace<'a>,
        limits: ReplayLimits,
    ) -> Self {
        Self {
            machine,
            initial_state,
            expected_bindings,
            trace,
            limits,
        }
    }
}

fn diagnostic(code: AdmissionDiagnosticCode, subject: &'static str) -> AdmissionDiagnostic {
    AdmissionDiagnostic {
        code,
        subject: Some(String::from(subject)),
        related: None,
        limit: None,
        observed: None,
        maximum: None,
    }
}

fn limit_diagnostic(
    code: AdmissionDiagnosticCode,
    kind: LimitKind,
    observed: usize,
    maximum: usize,
) -> AdmissionDiagnostic {
    AdmissionDiagnostic {
        code,
        subject: None,
        related: None,
        limit: Some(kind),
        observed: Some(observed),
        maximum: Some(maximum),
    }
}

fn identifier_diagnostic(error: IdentifierError, subject: &'static str) -> AdmissionDiagnostic {
    match error {
        IdentifierError::Empty => diagnostic(AdmissionDiagnosticCode::EmptyIdentifier, subject),
        IdentifierError::TooLong { observed, maximum } => AdmissionDiagnostic {
            code: AdmissionDiagnosticCode::IdentifierTooLong,
            subject: Some(String::from(subject)),
            related: None,
            limit: Some(LimitKind::IdentifierBytes),
            observed: Some(observed),
            maximum: Some(maximum),
        },
        IdentifierError::InvalidByte { index } => AdmissionDiagnostic {
            code: AdmissionDiagnosticCode::MalformedIdentifier,
            subject: Some(String::from(subject)),
            related: None,
            limit: None,
            observed: Some(index),
            maximum: None,
        },
    }
}

fn reference_diagnostic(error: PayloadRefError, subject: &'static str) -> AdmissionDiagnostic {
    match error {
        PayloadRefError::TooLong { observed, maximum } => AdmissionDiagnostic {
            code: AdmissionDiagnosticCode::ReferenceTooLong,
            subject: Some(String::from(subject)),
            related: None,
            limit: Some(LimitKind::ReferenceBytes),
            observed: Some(observed),
            maximum: Some(maximum),
        },
        PayloadRefError::Malformed => {
            diagnostic(AdmissionDiagnosticCode::MalformedReference, subject)
        }
    }
}

fn parse_id<Domain>(
    value: &str,
    limits: AdmissionLimits,
    subject: &'static str,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> Option<BoundedId<Domain>> {
    match BoundedId::parse(value, limits.max_identifier_bytes) {
        Ok(identifier) => Some(identifier),
        Err(error) => {
            diagnostics.push(identifier_diagnostic(error, subject));
            None
        }
    }
}

fn parse_reference(
    value: &ConcreteReferenceSpec,
    limits: AdmissionLimits,
    subject: &'static str,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> Option<ConcreteReference> {
    let Some(profile) = CanonicalReferenceProfile::parse(&value.profile) else {
        diagnostics.push(diagnostic(
            AdmissionDiagnosticCode::UnknownReferenceProfile,
            subject,
        ));
        return None;
    };
    let identity = match PayloadRef::parse(&value.identity, limits.max_reference_bytes) {
        Ok(identity) => identity,
        Err(error) => {
            diagnostics.push(reference_diagnostic(error, subject));
            return None;
        }
    };
    Some(ConcreteReference { profile, identity })
}

fn parse_binding(
    value: &ReferenceBindingSpec,
    limits: AdmissionLimits,
    subject: &'static str,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> Option<ReferenceBinding> {
    match value {
        ReferenceBindingSpec::Exact { reference } => {
            parse_reference(reference, limits, subject, diagnostics).map(ReferenceBinding::Exact)
        }
        ReferenceBindingSpec::Slot {
            slot_id,
            source_ref,
        } => {
            let slot_id =
                parse_id::<crate::id::SlotDomain>(slot_id, limits, "slot-id", diagnostics);
            let source_ref =
                parse_id::<crate::id::SourceDomain>(source_ref, limits, "source-ref", diagnostics);
            match (slot_id, source_ref) {
                (Some(slot_id), Some(source_ref)) => Some(ReferenceBinding::Slot {
                    slot_id,
                    source_ref,
                }),
                _ => None,
            }
        }
    }
}

fn binding_size(binding: &ReferenceBinding) -> usize {
    match binding {
        ReferenceBinding::Exact(reference) => reference
            .profile
            .as_str()
            .len()
            .saturating_add(reference.identity.as_str().len()),
        ReferenceBinding::Slot {
            slot_id,
            source_ref,
        } => slot_id
            .as_str()
            .len()
            .saturating_add(source_ref.as_str().len()),
    }
}

fn declaration_size(declaration: &ConcreteEffectDeclaration) -> usize {
    let fields = [
        declaration.declaration_key.as_str().len(),
        declaration.node_id.as_str().len(),
        declaration.operation.as_str().len(),
        declaration.resource.as_str().len(),
        declaration.ability.as_str().len(),
        binding_size(&declaration.request_binding),
        binding_size(&declaration.payload_binding),
    ];
    let mut size = 0usize;
    for field in fields {
        size = size.saturating_add(field);
    }
    size
}

fn parse_declaration(
    raw: &ConcreteEffectDeclarationSpec,
    limits: AdmissionLimits,
    profile: crate::LifecycleProfile,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> Option<ConcreteEffectDeclaration> {
    let declaration_key = parse_id::<crate::id::DeclarationDomain>(
        &raw.declaration_key,
        limits,
        "declaration-key",
        diagnostics,
    )?;
    let node_id =
        parse_id::<crate::id::NodeDomain>(&raw.node_id, limits, "effect-node", diagnostics)?;
    let operation = parse_id::<crate::id::OperationDomain>(
        &raw.operation,
        limits,
        "effect-operation",
        diagnostics,
    )?;
    let resource = parse_id::<crate::id::ResourceDomain>(
        &raw.resource,
        limits,
        "effect-resource",
        diagnostics,
    )?;
    let ability =
        parse_id::<crate::id::AbilityDomain>(&raw.ability, limits, "effect-ability", diagnostics)?;
    let request_binding =
        parse_binding(&raw.request_binding, limits, "request-binding", diagnostics)?;
    let payload_binding =
        parse_binding(&raw.payload_binding, limits, "payload-binding", diagnostics)?;
    if matches!(
        raw.transition.action,
        crate::Action::Suspend | crate::Action::ResumeSignal
    ) {
        diagnostics.push(diagnostic(
            AdmissionDiagnosticCode::SuspensionEffectTransition,
            "effect-transition",
        ));
        return None;
    }
    if lifecycle_target_for_profile(profile, raw.transition.from, raw.transition.action).is_err()
    {
        diagnostics.push(diagnostic(
            AdmissionDiagnosticCode::InvalidEffectTransition,
            "effect-transition",
        ));
        return None;
    }
    Some(ConcreteEffectDeclaration {
        declaration_key,
        node_id,
        transition: raw.transition,
        operation,
        resource,
        ability,
        request_binding,
        payload_binding,
    })
}

struct AdmittedConcreteEffects {
    declarations: Vec<ConcreteEffectDeclaration>,
    key_indices: BTreeMap<DeclarationKey, usize>,
    route_indices: BTreeMap<EffectKey, Vec<usize>>,
}

fn admit_concrete_effects(
    raw_effects: &[ConcreteEffectDeclarationSpec],
    limits: AdmissionLimits,
    machine: &AdmittedMachine,
    diagnostics: &mut Vec<AdmissionDiagnostic>,
) -> AdmittedConcreteEffects {
    let profile = machine.lifecycle_profile();
    let mut declarations = Vec::new();
    let mut keys = BTreeSet::new();
    for raw in raw_effects {
        let Some(declaration) = parse_declaration(raw, limits, profile, diagnostics) else {
            continue;
        };
        if !keys.insert(declaration.declaration_key.clone()) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::DuplicateDeclarationKey,
                "declaration-key",
            ));
            continue;
        }
        let Some(node) = machine.nodes.get(&declaration.node_id) else {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::EffectNodeMissing,
                "effect-node",
            ));
            continue;
        };
        if !node.operations.contains(&declaration.operation) {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::EffectOperationUndeclared,
                "effect-operation",
            ));
            continue;
        }
        let observed = declaration_size(&declaration);
        if observed > limits.max_intent_bytes {
            diagnostics.push(limit_diagnostic(
                AdmissionDiagnosticCode::IntentTooLarge,
                LimitKind::IntentBytes,
                observed,
                limits.max_intent_bytes,
            ));
            continue;
        }
        declarations.push(declaration);
    }
    declarations.sort();
    let mut key_indices = BTreeMap::new();
    let mut route_indices = BTreeMap::<EffectKey, Vec<usize>>::new();
    for (index, declaration) in declarations.iter().enumerate() {
        if key_indices
            .insert(declaration.declaration_key.clone(), index)
            .is_some()
        {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::DuplicateConcreteEffectDeclaration,
                "declaration-key",
            ));
        }
        let route = EffectKey {
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
        };
        route_indices.entry(route).or_default().push(index);
    }
    for indices in route_indices.values() {
        if indices.len() > limits.max_intents_per_transition {
            diagnostics.push(limit_diagnostic(
                AdmissionDiagnosticCode::TooManyIntentsForTransition,
                LimitKind::IntentsPerTransition,
                indices.len(),
                limits.max_intents_per_transition,
            ));
        }
    }
    AdmittedConcreteEffects {
        declarations,
        key_indices,
        route_indices,
    }
}

impl AdmittedMachine {
    /// Admits and normalizes one untrusted v2 machine specification.
    pub fn admit_v2(
        spec: MachineSpecV2,
        limits: AdmissionLimits,
    ) -> Result<Self, Vec<AdmissionDiagnostic>> {
        let mut diagnostics = Vec::new();
        if spec.schema != MACHINE_SCHEMA_V2 {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::UnsupportedSchema,
                "machine-schema",
            ));
        }
        if spec.lifecycle_profile != LINEAR_LIFECYCLE_V1
            && spec.lifecycle_profile != crate::DURABLE_SUSPENSION_V1
        {
            diagnostics.push(diagnostic(
                AdmissionDiagnosticCode::UnsupportedLifecycleProfile,
                "lifecycle-profile",
            ));
        }
        if spec.effects.len() > limits.max_effect_declarations {
            diagnostics.push(limit_diagnostic(
                AdmissionDiagnosticCode::TooManyEffectDeclarations,
                LimitKind::EffectDeclarations,
                spec.effects.len(),
                limits.max_effect_declarations,
            ));
        }
        if !diagnostics.is_empty() {
            diagnostics.sort();
            diagnostics.dedup();
            return Err(diagnostics);
        }

        let mut v1_spec = MachineSpec::new(spec.nodes, spec.edges);
        v1_spec.lifecycle_profile = spec.lifecycle_profile.clone();
        v1_spec.suspensions = spec.suspensions;
        let mut machine = Self::admit(v1_spec, limits)?;
        let admitted_effects =
            admit_concrete_effects(&spec.effects, limits, &machine, &mut diagnostics);
        diagnostics.sort();
        diagnostics.dedup();
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        machine.schema = MachineSchema::V2;
        machine.concrete_effects = admitted_effects.declarations;
        machine.concrete_effect_key_indices = admitted_effects.key_indices;
        machine.concrete_effect_route_indices = admitted_effects.route_indices;
        if !well_formed(&machine) {
            return Err(alloc::vec![diagnostic(
                AdmissionDiagnosticCode::NormalizationInvariant,
                "concrete-effect-index",
            )]);
        }
        Ok(machine)
    }
}

fn reference_is_well_formed(reference: &ConcreteReference, limits: AdmissionLimits) -> bool {
    reference.profile == CanonicalReferenceProfile::PackedPreservesV1
        && reference.identity.as_str().len() <= limits.max_reference_bytes
}

fn binding_is_well_formed(binding: &ReferenceBinding, limits: AdmissionLimits) -> bool {
    match binding {
        ReferenceBinding::Exact(reference) => reference_is_well_formed(reference, limits),
        ReferenceBinding::Slot {
            slot_id,
            source_ref,
        } => {
            slot_id.as_str().len() <= limits.max_identifier_bytes
                && source_ref.as_str().len() <= limits.max_identifier_bytes
        }
    }
}

pub(crate) fn concrete_effects_match(machine: &AdmittedMachine) -> bool {
    let mut normalized = machine.concrete_effects.clone();
    normalized.sort();
    normalized.dedup();
    if normalized != machine.concrete_effects {
        return false;
    }
    let mut key_indices = BTreeMap::new();
    let mut route_indices = BTreeMap::<EffectKey, Vec<usize>>::new();
    for (index, declaration) in machine.concrete_effects.iter().enumerate() {
        let Some(node) = machine.nodes.get(&declaration.node_id) else {
            return false;
        };
        if !node.operations.contains(&declaration.operation) {
            return false;
        }
        if !matches!(
            declaration.transition.action,
            crate::Action::Suspend | crate::Action::ResumeSignal
        ) && lifecycle_target_for_profile(
            machine.lifecycle_profile,
            declaration.transition.from,
            declaration.transition.action,
        )
        .is_err()
        {
            return false;
        }
        if matches!(
            declaration.transition.action,
            crate::Action::Suspend | crate::Action::ResumeSignal
        ) {
            return false;
        }
        if !binding_is_well_formed(&declaration.request_binding, machine.limits) {
            return false;
        }
        if !binding_is_well_formed(&declaration.payload_binding, machine.limits) {
            return false;
        }
        if declaration_size(declaration) > machine.limits.max_intent_bytes {
            return false;
        }
        if key_indices
            .insert(declaration.declaration_key.clone(), index)
            .is_some()
        {
            return false;
        }
        let route = EffectKey {
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
        };
        route_indices.entry(route).or_default().push(index);
    }
    if route_indices
        .values()
        .any(|indices| indices.len() > machine.limits.max_intents_per_transition)
    {
        return false;
    }
    key_indices == machine.concrete_effect_key_indices
        && route_indices == machine.concrete_effect_route_indices
}

fn parse_event_reference(
    value: &ConcreteReferenceSpec,
    limits: AdmissionLimits,
) -> Result<ConcreteReference, crate::DenialReason> {
    let profile = CanonicalReferenceProfile::parse(&value.profile)
        .ok_or(crate::DenialReason::MalformedEffectRequest)?;
    let identity = PayloadRef::parse(&value.identity, limits.max_reference_bytes)
        .map_err(|_| crate::DenialReason::MalformedEffectRequest)?;
    Ok(ConcreteReference { profile, identity })
}

struct ResolvedBinding {
    slot_id: Option<SlotId>,
    source_ref: Option<SourceRef>,
    reference: ConcreteReference,
}

fn resolve_binding(
    declaration: &ReferenceBinding,
    input: &ConcreteBindingInput,
    limits: AdmissionLimits,
) -> Result<ResolvedBinding, crate::DenialReason> {
    let reference = parse_event_reference(&input.reference, limits)?;
    match declaration {
        ReferenceBinding::Exact(expected) => {
            let is_accepted = crate::verified::concrete_binding_accepts(
                crate::verified::BINDING_EXACT,
                false,
                false,
                reference == *expected,
                input.slot_id.is_none(),
                input.source_ref.is_none(),
            );
            if !is_accepted {
                return Err(crate::DenialReason::ConcreteBindingMismatch);
            }
            Ok(ResolvedBinding {
                slot_id: None,
                source_ref: None,
                reference,
            })
        }
        ReferenceBinding::Slot {
            slot_id,
            source_ref,
        } => {
            let selected_slot = input
                .slot_id
                .as_deref()
                .ok_or(crate::DenialReason::ConcreteBindingMismatch)
                .and_then(|value| {
                    SlotId::parse(value, limits.max_identifier_bytes)
                        .map_err(|_| crate::DenialReason::MalformedEffectRequest)
                })?;
            let selected_source = input
                .source_ref
                .as_deref()
                .ok_or(crate::DenialReason::ConcreteBindingMismatch)
                .and_then(|value| {
                    SourceRef::parse(value, limits.max_identifier_bytes)
                        .map_err(|_| crate::DenialReason::MalformedEffectRequest)
                })?;
            let is_accepted = crate::verified::concrete_binding_accepts(
                crate::verified::BINDING_SLOT,
                selected_slot == *slot_id,
                selected_source == *source_ref,
                false,
                false,
                false,
            );
            if !is_accepted {
                return Err(crate::DenialReason::ConcreteBindingMismatch);
            }
            Ok(ResolvedBinding {
                slot_id: Some(selected_slot),
                source_ref: Some(selected_source),
                reference,
            })
        }
    }
}

fn intent_size(intent: &ConcreteEffectIntent) -> usize {
    let optional_size = |value: &Option<BoundedId<crate::id::SlotDomain>>| {
        value
            .as_ref()
            .map_or(0, |identifier| identifier.as_str().len())
    };
    let source_size = |value: &Option<BoundedId<crate::id::SourceDomain>>| {
        value
            .as_ref()
            .map_or(0, |identifier| identifier.as_str().len())
    };
    let fields = [
        intent.declaration_key.as_str().len(),
        intent.node_id.as_str().len(),
        intent.operation.as_str().len(),
        intent.resource.as_str().len(),
        intent.ability.as_str().len(),
        optional_size(&intent.request_slot),
        source_size(&intent.request_source_ref),
        intent.request_reference.profile.as_str().len(),
        intent.request_reference.identity.as_str().len(),
        optional_size(&intent.payload_slot),
        source_size(&intent.payload_source_ref),
        intent.payload_reference.profile.as_str().len(),
        intent.payload_reference.identity.as_str().len(),
    ];
    let mut size = 0usize;
    for field in fields {
        size = size.saturating_add(field);
    }
    size
}

fn binding_input_size(input: &ConcreteBindingInput) -> Option<usize> {
    let slot_size = input.slot_id.as_ref().map_or(0, String::len);
    let source_size = input.source_ref.as_ref().map_or(0, String::len);
    slot_size
        .checked_add(source_size)?
        .checked_add(input.reference.profile.len())?
        .checked_add(input.reference.identity.len())
}

fn request_size(request: &ConcreteEffectRequest) -> Option<usize> {
    request
        .declaration_key
        .len()
        .checked_add(binding_input_size(&request.request)?)?
        .checked_add(binding_input_size(&request.payload)?)
}

pub(crate) fn concrete_event_size(event: &ConcreteEvent) -> Option<usize> {
    let mut size = event.node_id.len().checked_add(EVENT_ACTION_TAG_BYTES)?;
    if let Some(fact) = &event.suspension {
        size = size.checked_add(fact.class.len())?;
        size = size.checked_add(fact.reference.len())?;
    }
    for request in &event.effects {
        size = size.checked_add(request_size(request)?)?;
    }
    Some(size)
}

pub(crate) fn resolve_concrete_intents(
    machine: &AdmittedMachine,
    allowed_indices: &[usize],
    event: &ConcreteEvent,
) -> Result<(Vec<usize>, Vec<ConcreteEffectIntent>), crate::DenialReason> {
    let mut selected = BTreeSet::new();
    let allowed = allowed_indices.iter().copied().collect::<BTreeSet<_>>();
    let mut resolved = Vec::with_capacity(event.effects.len());
    for request in &event.effects {
        let declaration_key = DeclarationKey::parse(
            &request.declaration_key,
            machine.limits.max_identifier_bytes,
        )
        .map_err(|_| crate::DenialReason::MalformedEffectRequest)?;
        let Some(index) = machine
            .concrete_effect_key_indices
            .get(&declaration_key)
            .copied()
        else {
            return Err(crate::DenialReason::UnknownDeclaration);
        };
        if !allowed.contains(&index) {
            return Err(crate::DenialReason::UndeclaredEffect);
        }
        if !selected.insert(index) {
            return Err(crate::DenialReason::DuplicateEffectRequest);
        }
        let Some(declaration) = machine.concrete_effects.get(index) else {
            return Err(crate::DenialReason::InvalidState);
        };
        let request_binding = resolve_binding(
            &declaration.request_binding,
            &request.request,
            machine.limits,
        )?;
        let payload_binding = resolve_binding(
            &declaration.payload_binding,
            &request.payload,
            machine.limits,
        )?;
        let intent = ConcreteEffectIntent {
            declaration_key: declaration.declaration_key.clone(),
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
            operation: declaration.operation.clone(),
            resource: declaration.resource.clone(),
            ability: declaration.ability.clone(),
            request_slot: request_binding.slot_id,
            request_source_ref: request_binding.source_ref,
            request_reference: request_binding.reference,
            payload_slot: payload_binding.slot_id,
            payload_source_ref: payload_binding.source_ref,
            payload_reference: payload_binding.reference,
        };
        let observed = intent_size(&intent);
        if observed > machine.limits.max_intent_bytes {
            return Err(crate::DenialReason::ConcreteIntentTooLarge {
                observed,
                maximum: machine.limits.max_intent_bytes,
            });
        }
        resolved.push((index, intent));
    }
    resolved.sort_by(|left, right| left.1.cmp(&right.1));
    let indices = resolved.iter().map(|(index, _)| *index).collect();
    let intents = resolved.into_iter().map(|(_, intent)| intent).collect();
    Ok((indices, intents))
}

#[must_use]
pub fn concrete_effects_allowed(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &ConcreteEvent,
    intents: &[ConcreteEffectIntent],
) -> bool {
    if machine.schema != MachineSchema::V2
        || !crate::invariant(machine, state)
        || intents.len() != event.effects.len()
    {
        return false;
    }
    let Ok(node_id) = NodeId::parse(&event.node_id, machine.limits.max_identifier_bytes) else {
        return false;
    };
    let Some(from) = state.state(&node_id) else {
        return false;
    };
    let route = EffectKey {
        node_id,
        transition: TransitionSelector {
            from,
            action: event.action,
        },
    };
    let allowed = machine
        .concrete_effect_route_indices
        .get(&route)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    resolve_concrete_intents(machine, allowed, event).is_ok_and(|(_, expected)| expected == intents)
}
