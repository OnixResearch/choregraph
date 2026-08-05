// The canonical compatibility corpus retains explicit byte-level boundary cases.
#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    numeric_units,
    raw_arithmetic_overflow,
    reason = "the canonical compatibility corpus retains explicit byte-level boundary cases"
)]

// r[impl choregraph.interchange.kamacite.fixtures]
use crate::aggregate::{ArtifactSet, IdentityMaterials};
use crate::codec::{
    FSM_SPEC_ROOT_LABEL, FSM_SPEC_SCHEMA_VERSION, MACHINE_ROOT_LABEL, MACHINE_SCHEMA_VERSION,
    encode,
};
use crate::identity::{ArtifactDomain, KamaciteIdentityDomain};
use crate::receipt::{LinkReceiptRole, TraceObservation};
use crate::{
    AggregateError, AggregateMachineDescriptor, CodecBoundaryError, DescriptorDependencySet,
    DescriptorEdge, DescriptorEffect, DescriptorLimits, DescriptorNode, LinkReceiptBody,
    ReceiptError, build_artifact_set, build_profile_artifact, build_receipt_bundle,
    compute_identity, compute_kamacite_identity, encode_fsm_receipt_body, encode_link_receipt_body,
    encode_machine_descriptor, validate_fsm_receipt, validate_lifecycle_artifact,
    validate_link_receipt, validate_machine_artifact, validate_receipt_bundle,
};
use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, EdgeSpec, EffectDeclarationSpec, LocalState,
    MachineSpec, NodeSpec, SuspensionDeclarationSpec, TransitionSelector,
};
use serde::Serialize;

const PAYLOAD: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SUSPENSION_REFERENCE: &str =
    "blake3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const SUSPENSION_CLASS: &str = "manual-approval";
const DURABLE_MATRIX_CELLS: usize = 49;
const SOURCE_ENVELOPE: &[u8] = b"canonical-source-envelope-v1";
const PROOF_SET: &[u8] = b"proof-set-status-pending-v1";
const GENERATOR: &[u8] = b"choregraph-generator-source-v1";
const ADAPTER: &[u8] = b"choregraph-kamacite-adapter-source-v1";
const TOOLCHAIN: &[u8] = b"rust-nickel-verus-toolchain-v1";
const LOCAL_LIFECYCLE_MATRIX_CELLS: usize = 30;

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("expected success, got {error:?}"),
    }
}

fn must_some<T>(value: Option<T>) -> T {
    match value {
        Some(value) => value,
        None => panic!("expected a present fixture value"),
    }
}

fn materials(proof_set: &'static [u8]) -> IdentityMaterials<'static> {
    IdentityMaterials {
        source_envelope: SOURCE_ENVELOPE,
        proof_set,
        generator: GENERATOR,
        adapter: ADAPTER,
        toolchain: TOOLCHAIN,
    }
}

fn machine_spec(permuted: bool) -> MachineSpec {
    let nodes = if permuted {
        vec![
            NodeSpec::new("tail", vec![String::from("emit")]),
            NodeSpec::new("right", vec![String::from("run")]),
            NodeSpec::new("root", vec![String::from("prepare")]),
            NodeSpec::new("left", vec![String::from("run")]),
        ]
    } else {
        vec![
            NodeSpec::new("root", vec![String::from("prepare")]),
            NodeSpec::new("left", vec![String::from("run")]),
            NodeSpec::new("right", vec![String::from("run")]),
            NodeSpec::new("tail", vec![String::from("emit")]),
        ]
    };
    let edges = if permuted {
        vec![
            EdgeSpec::new("right", "tail"),
            EdgeSpec::new("root", "right"),
            EdgeSpec::new("left", "tail"),
            EdgeSpec::new("root", "left"),
        ]
    } else {
        vec![
            EdgeSpec::new("root", "left"),
            EdgeSpec::new("root", "right"),
            EdgeSpec::new("left", "tail"),
            EdgeSpec::new("right", "tail"),
        ]
    };
    let mut spec = MachineSpec::new(nodes, edges);
    spec.effects.push(EffectDeclarationSpec {
        node_id: String::from("root"),
        transition: TransitionSelector {
            from: LocalState::Ready,
            action: Action::Start,
        },
        operation: String::from("prepare"),
        resource: String::from("root/work"),
        ability: String::from("execute"),
        request: String::from("request/start"),
        payload: String::from(PAYLOAD),
    });
    spec
}

fn admitted(permuted: bool) -> AdmittedMachine {
    must(AdmittedMachine::admit(
        machine_spec(permuted),
        AdmissionLimits::default(),
    ))
}

fn artifact_set() -> ArtifactSet {
    must(build_artifact_set(&admitted(false), materials(PROOF_SET)))
}

fn successful_trace() -> Vec<TraceObservation> {
    vec![
        TraceObservation {
            from_state: String::from("pending"),
            event: String::from("mark-ready"),
            to_state: String::from("ready"),
            guard_evidence_refs: vec![String::from("dependency-snapshot:root:empty")],
        },
        TraceObservation {
            from_state: String::from("ready"),
            event: String::from("start"),
            to_state: String::from("running"),
            guard_evidence_refs: Vec::new(),
        },
        TraceObservation {
            from_state: String::from("running"),
            event: String::from("complete"),
            to_state: String::from("completed"),
            guard_evidence_refs: Vec::new(),
        },
    ]
}

#[test]
fn canonical_artifacts_round_trip_with_stable_normalized_identities() {
    let first = artifact_set();
    let second = must(build_artifact_set(&admitted(true), materials(PROOF_SET)));

    assert_eq!(first.machine_artifact, second.machine_artifact);
    assert_eq!(first.admitted_ir_artifact, second.admitted_ir_artifact);
    assert_eq!(first.identities, second.identities);
    assert!(first.lifecycle.coverage.missing_cells.is_empty());
    assert_eq!(
        first.lifecycle.descriptor.transitions.len() + first.lifecycle.descriptor.denials.len(),
        LOCAL_LIFECYCLE_MATRIX_CELLS
    );
    assert_eq!(
        first.lifecycle.descriptor.determinism_policy,
        "deterministic"
    );
    assert!(
        first
            .lifecycle
            .descriptor
            .side_effects
            .iter()
            .any(|effect| { effect.effect_id == "none" && effect.class == "no-side-effect" })
    );
    assert_eq!(
        first.machine_descriptor.global_state_cardinality,
        "not-enumerated"
    );
    assert!(
        !first
            .machine_artifact
            .text_projection
            .contains("global_states")
    );

    let lifecycle = must(validate_lifecycle_artifact(
        &first.lifecycle.canonical.packed_bytes,
    ));
    let validated = must(validate_machine_artifact(
        &first.machine_artifact.packed_bytes,
        &lifecycle,
        &first.identities.profile,
    ));
    assert_eq!(validated.machine, admitted(false));
    assert_eq!(validated.identity, first.identities.machine);
}

#[test]
fn durable_artifacts_round_trip_with_exact_suspension_facts() {
    let machine = durable_machine();
    let lifecycle = must(crate::build_durable_lifecycle_artifact(None));
    let (_profile, _profile_artifact, profile_identity) = must(build_profile_artifact());
    let descriptor = must(crate::durable_machine_descriptor(
        &machine,
        lifecycle.project_identity.clone(),
        profile_identity.clone(),
    ));
    let artifact = must(crate::encode_durable_machine_descriptor(&descriptor));
    let validated = must(crate::validate_durable_machine_artifact(
        &artifact.packed_bytes,
        &lifecycle,
        &profile_identity,
    ));
    assert_eq!(validated.machine, machine);
    assert_eq!(validated.descriptor.suspensions.len(), 1);
    let restarted = validated.machine.declared_suspension_state();
    let gate = must(
        choregraph_core::NodeId::parse("gate", AdmissionLimits::default().max_identifier_bytes)
            .map_err(|error| format!("gate node id: {error:?}")),
    );
    assert_eq!(
        restarted.state(&gate),
        Some(choregraph_core::LocalState::Suspended)
    );

    let decoded = must(crate::validate_durable_lifecycle_artifact(
        &lifecycle.canonical.packed_bytes,
    ));
    assert_eq!(decoded.descriptor.fsm_id, crate::DURABLE_FSM_ID);
    assert_eq!(
        decoded.descriptor.transitions.len() + decoded.descriptor.denials.len(),
        DURABLE_MATRIX_CELLS
    );
    assert!(decoded.descriptor.transitions.iter().any(|row| {
        row.from_state == "ready"
            && row.event == "suspend"
            && row.to_state == "suspended"
            && row.side_effect == "none"
    }));
    assert!(decoded.descriptor.transitions.iter().any(|row| {
        row.from_state == "suspended"
            && row.event == "resume-signal"
            && row.to_state == "ready"
            && row.side_effect == "none"
            && !row.guards.is_empty()
    }));
    assert!(
        decoded
            .descriptor
            .denials
            .iter()
            .any(|row| row.from_state == "suspended" && row.event == "start")
    );
    assert!(
        !decoded
            .descriptor
            .terminal_states
            .iter()
            .any(|state| state == "suspended")
    );
}

#[test]
fn durable_descriptor_tampering_and_profile_inference_fail_closed() {
    let machine = durable_machine();
    let lifecycle = must(crate::build_durable_lifecycle_artifact(None));
    let (_profile, _profile_artifact, profile_identity) = must(build_profile_artifact());
    let descriptor = must(crate::durable_machine_descriptor(
        &machine,
        lifecycle.project_identity.clone(),
        profile_identity.clone(),
    ));
    let artifact = must(crate::encode_durable_machine_descriptor(&descriptor));

    let as_v1 = validate_machine_artifact(&artifact.packed_bytes, &lifecycle, &profile_identity);
    assert!(as_v1.is_err());

    let mut malformed = descriptor.clone();
    malformed.suspensions[0].class = String::from("bad class!");
    let malformed = must(crate::encode_durable_machine_descriptor(&malformed));
    let failure = crate::validate_durable_machine_artifact(
        &malformed.packed_bytes,
        &lifecycle,
        &profile_identity,
    );
    assert!(matches!(failure, Err(AggregateError::AdmissionRejected)));

    let mut wrong_version = descriptor.clone();
    wrong_version.schema_version = 1;
    let wrong_version = must(crate::encode_durable_machine_descriptor(&wrong_version));
    let failure = crate::validate_durable_machine_artifact(
        &wrong_version.packed_bytes,
        &lifecycle,
        &profile_identity,
    );
    assert!(matches!(failure, Err(AggregateError::UnsupportedProfile)));

    let linear = admitted(false);
    let inference = crate::durable_machine_descriptor(
        &linear,
        lifecycle.project_identity.clone(),
        profile_identity.clone(),
    );
    assert!(matches!(inference, Err(AggregateError::UnsupportedProfile)));

    let widened = build_artifact_set(&machine, materials(PROOF_SET));
    assert!(matches!(widened, Err(AggregateError::UnsupportedProfile)));
}

fn durable_machine() -> AdmittedMachine {
    let mut spec = MachineSpec::new(
        vec![
            NodeSpec::new("source", Vec::new()),
            NodeSpec::new("gate", Vec::new()),
        ],
        vec![EdgeSpec::new("source", "gate")],
    );
    spec.lifecycle_profile = String::from(choregraph_core::DURABLE_SUSPENSION_V1);
    spec.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from("gate"),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(SUSPENSION_REFERENCE),
    });
    must(AdmittedMachine::admit(spec, AdmissionLimits::default()))
}

#[test]
fn empty_identity_material_is_rejected_before_artifact_issuance() {
    let machine = admitted(false);
    let empty_source = IdentityMaterials {
        source_envelope: b"",
        proof_set: PROOF_SET,
        generator: GENERATOR,
        adapter: ADAPTER,
        toolchain: TOOLCHAIN,
    };
    assert!(matches!(
        build_artifact_set(&machine, empty_source),
        Err(AggregateError::EmptySourceEnvelope)
    ));

    let empty_proof = IdentityMaterials {
        source_envelope: SOURCE_ENVELOPE,
        proof_set: b"",
        generator: GENERATOR,
        adapter: ADAPTER,
        toolchain: TOOLCHAIN,
    };
    assert!(matches!(
        build_artifact_set(&machine, empty_proof),
        Err(AggregateError::Identity(_))
    ));
}

#[test]
fn every_identity_binding_changes_only_with_its_selected_material() {
    let first = artifact_set();
    let changed = must(build_artifact_set(
        &admitted(false),
        materials(b"proof-set-status-discharged-v1"),
    ));

    assert_ne!(first.identities.proof_set, changed.identities.proof_set);
    assert_eq!(first.identities.machine, changed.identities.machine);
    assert_eq!(first.identities.lifecycle, changed.identities.lifecycle);
    assert_eq!(first.identities.admitted_ir, changed.identities.admitted_ir);
    assert_eq!(first.identities.generator, changed.identities.generator);
    assert_eq!(first.identities.adapter, changed.identities.adapter);
    assert_eq!(first.identities.profile, changed.identities.profile);
    assert_eq!(first.identities.toolchain, changed.identities.toolchain);
}

#[test]
fn all_receipt_roles_validate_exact_links_without_behavioral_claims() {
    let set = artifact_set();
    let bundle = must(build_receipt_bundle(&set, &successful_trace()));

    must(validate_receipt_bundle(&bundle, &set));
    assert_eq!(bundle.exhaustiveness.body.receipt_kind, "exhaustiveness");
    assert_eq!(bundle.trace.body.receipt_kind, "trace");
    assert!(bundle.exhaustiveness.body.semantic_claims.is_empty());
    assert!(bundle.trace.body.semantic_claims.is_empty());
    assert!(
        bundle
            .proof_reference
            .body
            .non_claims
            .iter()
            .any(|claim| claim.contains("does not prove proof soundness"))
    );
}

#[test]
fn malformed_noncanonical_and_json_substitution_fail_closed() {
    let set = artifact_set();
    let malformed = validate_machine_artifact(&[0xff], &set.lifecycle, &set.identities.profile);
    assert!(matches!(malformed, Err(AggregateError::Codec(_))));

    let json = validate_machine_artifact(
        br#"{"schema_version":1}"#,
        &set.lifecycle,
        &set.identities.profile,
    );
    assert!(matches!(
        json,
        Err(AggregateError::Codec(
            CodecBoundaryError::JsonProjectionIsNotAuthority
        ))
    ));

    let mut noncanonical = set.machine_artifact.packed_bytes.clone();
    let integer = must_some(
        noncanonical
            .windows(3)
            .position(|window| window == [0xb0, 0x01, 0x01]),
    );
    noncanonical.splice(integer..integer + 3, [0xb0, 0x02, 0x00, 0x01]);
    let result = validate_machine_artifact(&noncanonical, &set.lifecycle, &set.identities.profile);
    assert!(matches!(result, Err(AggregateError::Codec(_))));
}

#[test]
fn incomplete_local_lifecycle_matrix_is_rejected() {
    let set = artifact_set();
    let mut incomplete = set.lifecycle.descriptor.clone();
    assert!(incomplete.denials.pop().is_some());
    let encoded = must(encode(
        &incomplete,
        FSM_SPEC_SCHEMA_VERSION,
        FSM_SPEC_ROOT_LABEL,
    ));
    assert!(matches!(
        validate_lifecycle_artifact(&encoded.packed_bytes),
        Err(crate::LifecycleError::ProfileMismatch)
    ));
}

#[test]
fn unsupported_lifecycle_and_global_enumeration_are_rejected() {
    let set = artifact_set();
    let mut unsupported = set.machine_descriptor.clone();
    unsupported.lifecycle_profile_id = String::from("unsupported.lifecycle.v9");
    let unsupported_artifact = must(encode_machine_descriptor(&unsupported));
    assert!(matches!(
        validate_machine_artifact(
            &unsupported_artifact.packed_bytes,
            &set.lifecycle,
            &set.identities.profile,
        ),
        Err(AggregateError::UnsupportedProfile)
    ));

    let mut enumerated = set.machine_descriptor.clone();
    enumerated.global_state_model = String::from("enumerated-product-table");
    enumerated.global_state_cardinality = String::from("six-to-the-node-count");
    let enumerated_artifact = must(encode_machine_descriptor(&enumerated));
    assert!(matches!(
        validate_machine_artifact(
            &enumerated_artifact.packed_bytes,
            &set.lifecycle,
            &set.identities.profile,
        ),
        Err(AggregateError::GlobalStateEnumeration)
    ));
}

#[derive(Serialize)]
#[serde(rename = "choregraph-machine-v1")]
struct DescriptorWithGlobalStates<'a> {
    schema_version: u32,
    machine_schema: &'a str,
    lifecycle_profile_id: &'a str,
    lifecycle_profile_identity: &'a crate::ArtifactIdentity,
    kamacite_profile_identity: &'a crate::ArtifactIdentity,
    global_state_model: &'a str,
    global_state_cardinality: &'a str,
    limits: &'a DescriptorLimits,
    nodes: &'a [DescriptorNode],
    edges: &'a [DescriptorEdge],
    dependencies: &'a [DescriptorDependencySet],
    topological_order: &'a [String],
    effects: &'a [DescriptorEffect],
    compatibility: &'a crate::CompatibilityMetadata,
    checked_laws: &'a [String],
    non_claims: &'a [String],
    global_states: Vec<Vec<String>>,
}

fn descriptor_with_global_states(
    descriptor: &AggregateMachineDescriptor,
) -> DescriptorWithGlobalStates<'_> {
    DescriptorWithGlobalStates {
        schema_version: descriptor.schema_version,
        machine_schema: &descriptor.machine_schema,
        lifecycle_profile_id: &descriptor.lifecycle_profile_id,
        lifecycle_profile_identity: &descriptor.lifecycle_profile_identity,
        kamacite_profile_identity: &descriptor.kamacite_profile_identity,
        global_state_model: &descriptor.global_state_model,
        global_state_cardinality: &descriptor.global_state_cardinality,
        limits: &descriptor.limits,
        nodes: &descriptor.nodes,
        edges: &descriptor.edges,
        dependencies: &descriptor.dependencies,
        topological_order: &descriptor.topological_order,
        effects: &descriptor.effects,
        compatibility: &descriptor.compatibility,
        checked_laws: &descriptor.checked_laws,
        non_claims: &descriptor.non_claims,
        global_states: vec![vec![String::from("pending"), String::from("ready")]],
    }
}

#[test]
fn unknown_global_product_table_field_is_rejected() {
    let set = artifact_set();
    let malicious = descriptor_with_global_states(&set.machine_descriptor);
    let encoded = must(encode(
        &malicious,
        MACHINE_SCHEMA_VERSION,
        MACHINE_ROOT_LABEL,
    ));
    assert!(matches!(
        validate_machine_artifact(
            &encoded.packed_bytes,
            &set.lifecycle,
            &set.identities.profile,
        ),
        Err(AggregateError::Codec(_))
    ));
}

#[derive(Serialize)]
#[serde(rename = "choregraph-machine-v1")]
struct DescriptorWithDuplicateField<'a> {
    schema_version: u32,
    machine_schema: &'a str,
    #[serde(rename = "machine_schema")]
    duplicate_machine_schema: &'a str,
    lifecycle_profile_id: &'a str,
    lifecycle_profile_identity: &'a crate::ArtifactIdentity,
    kamacite_profile_identity: &'a crate::ArtifactIdentity,
    global_state_model: &'a str,
    global_state_cardinality: &'a str,
    limits: &'a DescriptorLimits,
    nodes: &'a [DescriptorNode],
    edges: &'a [DescriptorEdge],
    dependencies: &'a [DescriptorDependencySet],
    topological_order: &'a [String],
    effects: &'a [DescriptorEffect],
    compatibility: &'a crate::CompatibilityMetadata,
    checked_laws: &'a [String],
    non_claims: &'a [String],
}

#[test]
fn duplicate_descriptor_fields_are_rejected() {
    let set = artifact_set();
    let descriptor = &set.machine_descriptor;
    let duplicate = DescriptorWithDuplicateField {
        schema_version: descriptor.schema_version,
        machine_schema: &descriptor.machine_schema,
        duplicate_machine_schema: &descriptor.machine_schema,
        lifecycle_profile_id: &descriptor.lifecycle_profile_id,
        lifecycle_profile_identity: &descriptor.lifecycle_profile_identity,
        kamacite_profile_identity: &descriptor.kamacite_profile_identity,
        global_state_model: &descriptor.global_state_model,
        global_state_cardinality: &descriptor.global_state_cardinality,
        limits: &descriptor.limits,
        nodes: &descriptor.nodes,
        edges: &descriptor.edges,
        dependencies: &descriptor.dependencies,
        topological_order: &descriptor.topological_order,
        effects: &descriptor.effects,
        compatibility: &descriptor.compatibility,
        checked_laws: &descriptor.checked_laws,
        non_claims: &descriptor.non_claims,
    };
    match encode(&duplicate, MACHINE_SCHEMA_VERSION, MACHINE_ROOT_LABEL) {
        Ok(encoded) => assert!(matches!(
            validate_machine_artifact(
                &encoded.packed_bytes,
                &set.lifecycle,
                &set.identities.profile,
            ),
            Err(AggregateError::Codec(_))
        )),
        Err(CodecBoundaryError::Preserves(_)) => {}
        Err(other) => panic!("unexpected duplicate-field error: {other}"),
    }
}

#[test]
fn stale_and_mismatched_receipt_links_are_rejected() {
    let set = artifact_set();
    let bundle = must(build_receipt_bundle(&set, &successful_trace()));
    let stale = must(compute_identity(ArtifactDomain::Receipt, b"stale-receipt"));
    assert!(matches!(
        validate_link_receipt(
            &bundle.admission.canonical.packed_bytes,
            &stale,
            &set,
            LinkReceiptRole::Admission,
        ),
        Err(ReceiptError::StaleReceiptIdentity)
    ));

    let mut mismatched: LinkReceiptBody = bundle.proof_reference.body.clone();
    mismatched.subject = set.identities.machine.clone();
    let canonical = must(encode_link_receipt_body(&mismatched));
    let declared = must(compute_identity(
        ArtifactDomain::Receipt,
        &canonical.packed_bytes,
    ));
    assert!(matches!(
        validate_link_receipt(
            &canonical.packed_bytes,
            &declared,
            &set,
            LinkReceiptRole::ProofReference,
        ),
        Err(ReceiptError::LinkMismatch {
            role: LinkReceiptRole::ProofReference
        })
    ));
}

#[test]
fn stale_fsm_dependency_links_are_rejected() {
    let set = artifact_set();
    let bundle = must(build_receipt_bundle(&set, &successful_trace()));
    let mut stale = bundle.exhaustiveness.body.clone();
    stale.dependency_identities.clear();
    let artifact = must(encode_fsm_receipt_body(&stale));
    let identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Receipt, &artifact.packed_bytes);
    assert!(matches!(
        validate_fsm_receipt(
            &artifact.packed_bytes,
            &identity,
            &set.lifecycle,
            &set.kamacite_adapter_identity,
        ),
        Err(ReceiptError::FsmLinkage)
    ));
}

#[test]
fn trace_mismatch_and_overclaim_receipts_are_rejected() {
    let set = artifact_set();
    let bundle = must(build_receipt_bundle(&set, &successful_trace()));

    let mut trace = bundle.trace.body.clone();
    trace.trace_steps[1].to_state = String::from("completed");
    let trace_artifact = must(encode_fsm_receipt_body(&trace));
    let trace_identity = compute_kamacite_identity(
        KamaciteIdentityDomain::Receipt,
        &trace_artifact.packed_bytes,
    );
    assert!(matches!(
        validate_fsm_receipt(
            &trace_artifact.packed_bytes,
            &trace_identity,
            &set.lifecycle,
            &set.kamacite_adapter_identity,
        ),
        Err(ReceiptError::TraceMismatch)
    ));

    let mut overclaim = bundle.exhaustiveness.body.clone();
    overclaim
        .semantic_claims
        .push(String::from("transition implementation is formally proven"));
    let overclaim_artifact = must(encode_fsm_receipt_body(&overclaim));
    let overclaim_identity = compute_kamacite_identity(
        KamaciteIdentityDomain::Receipt,
        &overclaim_artifact.packed_bytes,
    );
    assert!(matches!(
        validate_fsm_receipt(
            &overclaim_artifact.packed_bytes,
            &overclaim_identity,
            &set.lifecycle,
            &set.kamacite_adapter_identity,
        ),
        Err(ReceiptError::Overclaim)
    ));
}

#[test]
fn profile_and_adapter_identities_are_exact_and_bounded() {
    let set = artifact_set();
    let (profile, artifact, identity) = must(build_profile_artifact());
    assert_eq!(profile.kamacite_revision, crate::KAMACITE_REVISION);
    assert_eq!(identity, set.identities.profile);
    assert_eq!(artifact, set.profile_artifact);
    assert_eq!(
        set.kamacite_adapter_identity,
        compute_kamacite_identity(KamaciteIdentityDomain::Adapter, ADAPTER)
    );
}
