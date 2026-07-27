// r[impl choregraph.interchange.kamacite.receipts]
use crate::aggregate::ArtifactSet;
use crate::codec::{
    CanonicalArtifact, CodecBoundaryError, FSM_RECEIPT_ROOT_LABEL, FSM_RECEIPT_SCHEMA_VERSION,
    LINK_RECEIPT_ROOT_LABEL, LINK_RECEIPT_SCHEMA_VERSION, decode, encode,
};
use crate::identity::{
    ArtifactDomain, ArtifactIdentity, IdentityError, KamaciteIdentity, KamaciteIdentityDomain,
    compute_identity, compute_kamacite_identity,
};
use crate::lifecycle::{FsmTransitionRow, LifecycleArtifact, LifecycleError, transition_identity};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

const SUPPORTED_SCHEMA_VERSION: u32 = 1;
const EXHAUSTIVENESS_RECEIPT_KIND: &str = "exhaustiveness";
const TRACE_RECEIPT_KIND: &str = "trace";
const REQUIRED_RECEIPT_NON_CLAIM_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkReceiptRole {
    Admission,
    Lifecycle,
    Source,
    ProofReference,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "choregraph-link-receipt-v1", deny_unknown_fields)]
pub struct LinkReceiptBody {
    pub schema_version: u32,
    pub role: LinkReceiptRole,
    pub subject: ArtifactIdentity,
    pub dependencies: Vec<ArtifactIdentity>,
    pub checked_laws: Vec<String>,
    pub non_claims: Vec<String>,
    pub semantic_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkReceiptArtifact {
    pub body: LinkReceiptBody,
    pub canonical: CanonicalArtifact,
    pub identity: ArtifactIdentity,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmGuardDecision {
    pub guard_id: String,
    pub allowed: bool,
    pub evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FsmTraceStep {
    pub from_state: String,
    pub event: String,
    pub to_state: String,
    pub transition_identity: Option<KamaciteIdentity>,
    pub guard_decisions: Vec<FsmGuardDecision>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "kamacite-fsm-receipt-v3", deny_unknown_fields)]
pub struct FsmReceiptBody {
    pub schema_version: u32,
    pub receipt_kind: String,
    pub spec_identity: KamaciteIdentity,
    pub state_universe_identity: KamaciteIdentity,
    pub event_universe_identity: KamaciteIdentity,
    pub transition_coverage_identity: KamaciteIdentity,
    pub denial_coverage_identity: KamaciteIdentity,
    pub adapter_identity: KamaciteIdentity,
    pub dependency_identities: Vec<KamaciteIdentity>,
    pub octet_receipt_identities: Vec<KamaciteIdentity>,
    pub valence_export_identity: Option<KamaciteIdentity>,
    pub checked_laws: Vec<String>,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
    pub semantic_claims: Vec<String>,
    pub trace_start_state: Option<String>,
    pub trace_events: Vec<String>,
    pub trace_steps: Vec<FsmTraceStep>,
    pub trace_final_state: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FsmReceiptArtifact {
    pub body: FsmReceiptBody,
    pub canonical: CanonicalArtifact,
    pub identity: KamaciteIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceObservation {
    pub from_state: String,
    pub event: String,
    pub to_state: String,
    pub guard_evidence_refs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptBundle {
    pub admission: LinkReceiptArtifact,
    pub lifecycle: LinkReceiptArtifact,
    pub source: LinkReceiptArtifact,
    pub proof_reference: LinkReceiptArtifact,
    pub exhaustiveness: FsmReceiptArtifact,
    pub trace: FsmReceiptArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReceiptError {
    Codec(CodecBoundaryError),
    Identity(IdentityError),
    Lifecycle(LifecycleError),
    LinkMismatch { role: LinkReceiptRole },
    StaleReceiptIdentity,
    ReceiptKind,
    FsmLinkage,
    Overclaim,
    TraceMismatch,
    GuardEvidenceMissing,
}

impl Display for ReceiptError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Codec(error) => Display::fmt(error, formatter),
            Self::Identity(error) => Display::fmt(error, formatter),
            Self::Lifecycle(error) => Display::fmt(error, formatter),
            Self::LinkMismatch { role } => {
                write!(
                    formatter,
                    "{role:?} receipt linkage does not match the artifact set"
                )
            }
            Self::StaleReceiptIdentity => formatter.write_str("receipt identity is stale"),
            Self::ReceiptKind => formatter.write_str("unsupported FSM receipt kind"),
            Self::FsmLinkage => {
                formatter.write_str("FSM receipt linkage does not match the lifecycle")
            }
            Self::Overclaim => formatter.write_str("receipt contains a prohibited semantic claim"),
            Self::TraceMismatch => {
                formatter.write_str("FSM trace does not match the lifecycle profile")
            }
            Self::GuardEvidenceMissing => {
                formatter.write_str("accepted guarded transition lacks positive evidence")
            }
        }
    }
}

impl std::error::Error for ReceiptError {}

impl From<CodecBoundaryError> for ReceiptError {
    fn from(error: CodecBoundaryError) -> Self {
        Self::Codec(error)
    }
}

impl From<IdentityError> for ReceiptError {
    fn from(error: IdentityError) -> Self {
        Self::Identity(error)
    }
}

impl From<LifecycleError> for ReceiptError {
    fn from(error: LifecycleError) -> Self {
        Self::Lifecycle(error)
    }
}

fn link_checked_laws(role: LinkReceiptRole) -> Vec<String> {
    match role {
        LinkReceiptRole::Admission => vec![
            String::from("canonical-machine-identity-matched"),
            String::from("admitted-ir-identity-matched"),
        ],
        LinkReceiptRole::Lifecycle => vec![
            String::from("closed-lifecycle-profile-matched"),
            String::from("lifecycle-identity-matched"),
        ],
        LinkReceiptRole::Source => vec![
            String::from("source-artifact-identity-matched"),
            String::from("profile-identity-matched"),
        ],
        LinkReceiptRole::ProofReference => vec![
            String::from("proof-artifact-identity-matched"),
            String::from("toolchain-identity-matched"),
        ],
    }
}

fn link_non_claims(role: LinkReceiptRole) -> Vec<String> {
    let role_non_claim = match role {
        LinkReceiptRole::Admission => "admission receipt does not prove transition correctness",
        LinkReceiptRole::Lifecycle => "lifecycle receipt does not prove implementation equivalence",
        LinkReceiptRole::Source => "source receipt does not prove source evaluation semantics",
        LinkReceiptRole::ProofReference => {
            "proof reference does not prove proof soundness or currency"
        }
    };
    vec![
        String::from(role_non_claim),
        String::from("receipt does not authorize runtime effects"),
        String::from("receipt does not prove external event truth"),
        String::from("receipt does not establish release eligibility"),
    ]
}

fn expected_link_body(set: &ArtifactSet, role: LinkReceiptRole) -> LinkReceiptBody {
    let (subject, dependencies) = match role {
        LinkReceiptRole::Admission => (
            set.identities.admitted_ir.clone(),
            vec![
                set.identities.machine.clone(),
                set.identities.lifecycle.clone(),
                set.identities.profile.clone(),
            ],
        ),
        LinkReceiptRole::Lifecycle => (
            set.identities.lifecycle.clone(),
            vec![
                set.identities.profile.clone(),
                set.identities.adapter.clone(),
            ],
        ),
        LinkReceiptRole::Source => (
            set.identities.generator.clone(),
            vec![set.identities.profile.clone()],
        ),
        LinkReceiptRole::ProofReference => (
            set.identities.proof_set.clone(),
            vec![
                set.identities.admitted_ir.clone(),
                set.identities.toolchain.clone(),
            ],
        ),
    };
    LinkReceiptBody {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        role,
        subject,
        dependencies,
        checked_laws: link_checked_laws(role),
        non_claims: link_non_claims(role),
        semantic_claims: Vec::new(),
    }
}

pub fn encode_link_receipt_body(body: &LinkReceiptBody) -> Result<CanonicalArtifact, ReceiptError> {
    encode(body, LINK_RECEIPT_SCHEMA_VERSION, LINK_RECEIPT_ROOT_LABEL).map_err(Into::into)
}

fn build_link_receipt(
    set: &ArtifactSet,
    role: LinkReceiptRole,
) -> Result<LinkReceiptArtifact, ReceiptError> {
    let body = expected_link_body(set, role);
    let canonical = encode_link_receipt_body(&body)?;
    let identity = compute_identity(ArtifactDomain::Receipt, &canonical.packed_bytes)?;
    Ok(LinkReceiptArtifact {
        body,
        canonical,
        identity,
    })
}

pub fn validate_link_receipt(
    input: &[u8],
    declared_identity: &ArtifactIdentity,
    set: &ArtifactSet,
    role: LinkReceiptRole,
) -> Result<LinkReceiptArtifact, ReceiptError> {
    declared_identity.validate(ArtifactDomain::Receipt)?;
    let (body, canonical): (LinkReceiptBody, CanonicalArtifact) =
        decode(input, LINK_RECEIPT_SCHEMA_VERSION, LINK_RECEIPT_ROOT_LABEL)?;
    if !body.semantic_claims.is_empty() {
        return Err(ReceiptError::Overclaim);
    }
    let expected = expected_link_body(set, role);
    if body != expected {
        return Err(ReceiptError::LinkMismatch { role });
    }
    let identity = compute_identity(ArtifactDomain::Receipt, &canonical.packed_bytes)?;
    if &identity != declared_identity {
        return Err(ReceiptError::StaleReceiptIdentity);
    }
    Ok(LinkReceiptArtifact {
        body,
        canonical,
        identity,
    })
}

fn fsm_non_claims() -> Vec<String> {
    vec![
        String::from("receipt does not prove transition implementation correctness"),
        String::from("receipt does not prove source equivalence"),
        String::from("receipt does not prove proof soundness"),
        String::from("receipt does not authorize or execute effects"),
        String::from("receipt does not establish release eligibility"),
    ]
}

fn base_fsm_receipt(
    lifecycle: &LifecycleArtifact,
    adapter_identity: KamaciteIdentity,
    receipt_kind: &str,
) -> FsmReceiptBody {
    FsmReceiptBody {
        schema_version: SUPPORTED_SCHEMA_VERSION,
        receipt_kind: String::from(receipt_kind),
        spec_identity: lifecycle.identity.clone(),
        state_universe_identity: lifecycle.coverage.state_universe_identity.clone(),
        event_universe_identity: lifecycle.coverage.event_universe_identity.clone(),
        transition_coverage_identity: lifecycle.coverage.transition_coverage_identity.clone(),
        denial_coverage_identity: lifecycle.coverage.denial_coverage_identity.clone(),
        adapter_identity,
        dependency_identities: lifecycle
            .descriptor
            .source_envelope_identity
            .iter()
            .cloned()
            .collect(),
        octet_receipt_identities: Vec::new(),
        valence_export_identity: None,
        checked_laws: Vec::new(),
        diagnostics: Vec::new(),
        non_claims: fsm_non_claims(),
        semantic_claims: Vec::new(),
        trace_start_state: None,
        trace_events: Vec::new(),
        trace_steps: Vec::new(),
        trace_final_state: None,
    }
}

pub fn encode_fsm_receipt_body(body: &FsmReceiptBody) -> Result<CanonicalArtifact, ReceiptError> {
    encode(body, FSM_RECEIPT_SCHEMA_VERSION, FSM_RECEIPT_ROOT_LABEL).map_err(Into::into)
}

fn finish_fsm_receipt(body: FsmReceiptBody) -> Result<FsmReceiptArtifact, ReceiptError> {
    let canonical = encode_fsm_receipt_body(&body)?;
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Receipt, &canonical.packed_bytes);
    Ok(FsmReceiptArtifact {
        body,
        canonical,
        identity,
    })
}

fn build_exhaustiveness_receipt(
    lifecycle: &LifecycleArtifact,
    adapter_identity: KamaciteIdentity,
) -> Result<FsmReceiptArtifact, ReceiptError> {
    let mut body = base_fsm_receipt(lifecycle, adapter_identity, EXHAUSTIVENESS_RECEIPT_KIND);
    body.checked_laws = vec![
        String::from("closed-state-event-matrix"),
        String::from("transition-and-denial-coverage-identities-matched"),
        String::from("no-missing-state-event-cells"),
    ];
    finish_fsm_receipt(body)
}

fn matching_transition<'a>(
    lifecycle: &'a LifecycleArtifact,
    observation: &TraceObservation,
) -> Option<&'a FsmTransitionRow> {
    lifecycle.descriptor.transitions.iter().find(|transition| {
        transition.from_state == observation.from_state
            && transition.event == observation.event
            && transition.to_state == observation.to_state
    })
}

fn trace_steps(
    lifecycle: &LifecycleArtifact,
    observations: &[TraceObservation],
) -> Result<Vec<FsmTraceStep>, ReceiptError> {
    let mut steps = Vec::with_capacity(observations.len());
    let mut expected_from = lifecycle.descriptor.initial_state.as_str();
    for observation in observations {
        if observation.from_state != expected_from {
            return Err(ReceiptError::TraceMismatch);
        }
        let transition =
            matching_transition(lifecycle, observation).ok_or(ReceiptError::TraceMismatch)?;
        let guard_decisions = transition
            .guards
            .iter()
            .map(|guard_id| {
                if observation.guard_evidence_refs.is_empty() {
                    return Err(ReceiptError::GuardEvidenceMissing);
                }
                Ok(FsmGuardDecision {
                    guard_id: guard_id.clone(),
                    allowed: true,
                    evidence_refs: observation.guard_evidence_refs.clone(),
                })
            })
            .collect::<Result<Vec<_>, ReceiptError>>()?;
        steps.push(FsmTraceStep {
            from_state: observation.from_state.clone(),
            event: observation.event.clone(),
            to_state: observation.to_state.clone(),
            transition_identity: Some(transition_identity(transition)?),
            guard_decisions,
        });
        expected_from = observation.to_state.as_str();
    }
    Ok(steps)
}

fn build_trace_receipt(
    lifecycle: &LifecycleArtifact,
    adapter_identity: KamaciteIdentity,
    observations: &[TraceObservation],
) -> Result<FsmReceiptArtifact, ReceiptError> {
    let steps = trace_steps(lifecycle, observations)?;
    let mut body = base_fsm_receipt(lifecycle, adapter_identity, TRACE_RECEIPT_KIND);
    body.checked_laws = vec![
        String::from("trace-steps-match-closed-profile"),
        String::from("trace-transition-identities-recomputed"),
        String::from("accepted-guards-have-positive-evidence"),
    ];
    body.trace_start_state = Some(lifecycle.descriptor.initial_state.clone());
    body.trace_events = steps.iter().map(|step| step.event.clone()).collect();
    body.trace_final_state = steps
        .last()
        .map(|step| step.to_state.clone())
        .or_else(|| body.trace_start_state.clone());
    body.trace_steps = steps;
    finish_fsm_receipt(body)
}

fn validate_trace_body(
    body: &FsmReceiptBody,
    lifecycle: &LifecycleArtifact,
) -> Result<(), ReceiptError> {
    let start = body
        .trace_start_state
        .as_deref()
        .ok_or(ReceiptError::TraceMismatch)?;
    if start != lifecycle.descriptor.initial_state {
        return Err(ReceiptError::TraceMismatch);
    }
    if body.trace_events.len() != body.trace_steps.len() {
        return Err(ReceiptError::TraceMismatch);
    }
    let mut expected_from = start;
    for (event, step) in body.trace_events.iter().zip(&body.trace_steps) {
        if event != &step.event || step.from_state != expected_from {
            return Err(ReceiptError::TraceMismatch);
        }
        let observation = TraceObservation {
            from_state: step.from_state.clone(),
            event: step.event.clone(),
            to_state: step.to_state.clone(),
            guard_evidence_refs: step
                .guard_decisions
                .iter()
                .flat_map(|decision| decision.evidence_refs.iter().cloned())
                .collect(),
        };
        let transition =
            matching_transition(lifecycle, &observation).ok_or(ReceiptError::TraceMismatch)?;
        let expected_identity = transition_identity(transition)?;
        if step.transition_identity.as_ref() != Some(&expected_identity) {
            return Err(ReceiptError::TraceMismatch);
        }
        if transition.guards.len() != step.guard_decisions.len() {
            return Err(ReceiptError::TraceMismatch);
        }
        for (guard_id, decision) in transition.guards.iter().zip(&step.guard_decisions) {
            if decision.guard_id != *guard_id
                || !decision.allowed
                || decision.evidence_refs.is_empty()
            {
                return Err(ReceiptError::GuardEvidenceMissing);
            }
        }
        expected_from = step.to_state.as_str();
    }
    if body.trace_final_state.as_deref() != Some(expected_from) {
        return Err(ReceiptError::TraceMismatch);
    }
    Ok(())
}

pub fn validate_fsm_receipt(
    input: &[u8],
    declared_identity: &KamaciteIdentity,
    lifecycle: &LifecycleArtifact,
    expected_adapter_identity: &KamaciteIdentity,
) -> Result<FsmReceiptArtifact, ReceiptError> {
    declared_identity
        .validate(KamaciteIdentityDomain::Receipt)
        .map_err(ReceiptError::Identity)?;
    expected_adapter_identity
        .validate(KamaciteIdentityDomain::Adapter)
        .map_err(ReceiptError::Identity)?;
    let (body, canonical): (FsmReceiptBody, CanonicalArtifact) =
        decode(input, FSM_RECEIPT_SCHEMA_VERSION, FSM_RECEIPT_ROOT_LABEL)?;
    if !body.semantic_claims.is_empty()
        || body.non_claims != fsm_non_claims()
        || body.non_claims.len() != REQUIRED_RECEIPT_NON_CLAIM_COUNT
    {
        return Err(ReceiptError::Overclaim);
    }
    let expected_dependencies = lifecycle
        .descriptor
        .source_envelope_identity
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    if body.schema_version != SUPPORTED_SCHEMA_VERSION
        || body.spec_identity != lifecycle.identity
        || body.state_universe_identity != lifecycle.coverage.state_universe_identity
        || body.event_universe_identity != lifecycle.coverage.event_universe_identity
        || body.transition_coverage_identity != lifecycle.coverage.transition_coverage_identity
        || body.denial_coverage_identity != lifecycle.coverage.denial_coverage_identity
        || body.adapter_identity != *expected_adapter_identity
        || body.dependency_identities != expected_dependencies
        || !body.octet_receipt_identities.is_empty()
        || body.valence_export_identity.is_some()
        || !body.diagnostics.is_empty()
    {
        return Err(ReceiptError::FsmLinkage);
    }
    match body.receipt_kind.as_str() {
        EXHAUSTIVENESS_RECEIPT_KIND => {
            if body.checked_laws
                != [
                    "closed-state-event-matrix",
                    "transition-and-denial-coverage-identities-matched",
                    "no-missing-state-event-cells",
                ]
                || body.trace_start_state.is_some()
                || !body.trace_events.is_empty()
                || !body.trace_steps.is_empty()
                || body.trace_final_state.is_some()
            {
                return Err(ReceiptError::FsmLinkage);
            }
        }
        TRACE_RECEIPT_KIND => {
            if body.checked_laws
                != [
                    "trace-steps-match-closed-profile",
                    "trace-transition-identities-recomputed",
                    "accepted-guards-have-positive-evidence",
                ]
            {
                return Err(ReceiptError::FsmLinkage);
            }
            validate_trace_body(&body, lifecycle)?;
        }
        _ => return Err(ReceiptError::ReceiptKind),
    }
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Receipt, &canonical.packed_bytes);
    if &identity != declared_identity {
        return Err(ReceiptError::StaleReceiptIdentity);
    }
    Ok(FsmReceiptArtifact {
        body,
        canonical,
        identity,
    })
}

pub fn build_receipt_bundle(
    set: &ArtifactSet,
    observations: &[TraceObservation],
) -> Result<ReceiptBundle, ReceiptError> {
    Ok(ReceiptBundle {
        admission: build_link_receipt(set, LinkReceiptRole::Admission)?,
        lifecycle: build_link_receipt(set, LinkReceiptRole::Lifecycle)?,
        source: build_link_receipt(set, LinkReceiptRole::Source)?,
        proof_reference: build_link_receipt(set, LinkReceiptRole::ProofReference)?,
        exhaustiveness: build_exhaustiveness_receipt(
            &set.lifecycle,
            set.kamacite_adapter_identity.clone(),
        )?,
        trace: build_trace_receipt(
            &set.lifecycle,
            set.kamacite_adapter_identity.clone(),
            observations,
        )?,
    })
}

pub fn validate_receipt_bundle(
    bundle: &ReceiptBundle,
    set: &ArtifactSet,
) -> Result<(), ReceiptError> {
    for (artifact, role) in [
        (&bundle.admission, LinkReceiptRole::Admission),
        (&bundle.lifecycle, LinkReceiptRole::Lifecycle),
        (&bundle.source, LinkReceiptRole::Source),
        (&bundle.proof_reference, LinkReceiptRole::ProofReference),
    ] {
        validate_link_receipt(
            &artifact.canonical.packed_bytes,
            &artifact.identity,
            set,
            role,
        )?;
    }
    for artifact in [&bundle.exhaustiveness, &bundle.trace] {
        validate_fsm_receipt(
            &artifact.canonical.packed_bytes,
            &artifact.identity,
            &set.lifecycle,
            &set.kamacite_adapter_identity,
        )?;
    }
    Ok(())
}
