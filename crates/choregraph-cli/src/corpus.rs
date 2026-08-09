// Corpus generation binds reviewed source bytes and preserves established canonical fixture shapes.
#![allow(
    unknown_lints,
    assertion_density,
    explicit_defaults,
    function_length,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    reason = "corpus generation binds reviewed source bytes and preserves established canonical fixture shapes"
)]

use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, CANONICAL_REFERENCE_PROFILE_V1, ConcreteBindingInput,
    ConcreteDecision, ConcreteEffectDeclarationSpec, ConcreteEffectIntent, ConcreteEffectRequest,
    ConcreteEvent, ConcreteReferenceSpec, DURABLE_SUSPENSION_V1, EdgeSpec, EffectDeclarationSpec,
    LocalState, MachineSpec, MachineSpecV2, NodeSpec, ReferenceBindingSpec,
    SuspensionDeclarationSpec, TransitionSelector, step_concrete,
};
use choregraph_kamacite::{
    AggregateMachineDescriptor, ArtifactDomain, IdentityMaterials, KamaciteIdentityDomain,
    TraceObservation, build_artifact_set, build_concrete_intent_artifact,
    build_concrete_intent_receipt, build_concrete_machine_artifact,
    build_durable_lifecycle_artifact, build_receipt_bundle, compute_identity,
    compute_kamacite_identity, durable_machine_descriptor, encode_concrete_intent_receipt_body,
    encode_durable_machine_descriptor, encode_fsm_receipt_body, encode_link_receipt_body,
    encode_machine_descriptor,
};
use std::collections::BTreeSet;

const PAYLOAD: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SUSPENSION_CLASS: &str = "manual-approval";
const SUSPENSION_REFERENCE: &str =
    "blake3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const CONCRETE_REQUEST: &str =
    "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const CONCRETE_SLOT: &str = "request/input";
const CONCRETE_SOURCE: &str = "node/input/request";
const CONCRETE_DECLARATION: &str = "request/main";
const SOURCE_ENVELOPE: &[u8] = b"canonical-source-envelope-v1";
const PROOF_SET: &[u8] = b"proof-set-status-pending-v1";
const MATERIAL_LENGTH_BYTES: usize = size_of::<u64>();
pub(crate) const KAMACITE_AGGREGATE_SOURCE: &[u8] =
    include_bytes!("../../choregraph-kamacite/src/aggregate.rs");
const ADAPTER_SOURCES: &[&[u8]] = &[
    include_bytes!("../../choregraph-kamacite/src/lib.rs"),
    KAMACITE_AGGREGATE_SOURCE,
    include_bytes!("../../choregraph-kamacite/src/codec.rs"),
    include_bytes!("../../choregraph-kamacite/src/concrete.rs"),
    include_bytes!("../../choregraph-kamacite/src/identity.rs"),
    include_bytes!("../../choregraph-kamacite/src/lifecycle.rs"),
    include_bytes!("../../choregraph-kamacite/src/receipt.rs"),
    include_bytes!("../../../vendor/kamacite-core-selected/preserves/codec.rs"),
];
const GENERATOR_SOURCES: &[&[u8]] = &[include_bytes!("corpus.rs")];
const TOOLCHAIN_SOURCES: &[&[u8]] = &[
    include_bytes!("../../../rust-toolchain.toml"),
    include_bytes!("../../../Cargo.lock"),
];
const CANONICAL_INTEGER_BYTES: [u8; 3] = [0xb0, 0x01, 0x01];
const NON_CANONICAL_INTEGER_BYTES: [u8; 4] = [0xb0, 0x02, 0x00, 0x01];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KamaciteCorpusEntry {
    pub relative_path: &'static str,
    pub bytes: Vec<u8>,
}

fn diamond_machine() -> Result<AdmittedMachine, String> {
    let nodes = vec![
        NodeSpec::new("tail", vec![String::from("emit")]),
        NodeSpec::new("right", vec![String::from("run")]),
        NodeSpec::new("root", vec![String::from("prepare")]),
        NodeSpec::new("left", vec![String::from("run")]),
    ];
    let edges = vec![
        EdgeSpec::new("right", "tail"),
        EdgeSpec::new("root", "right"),
        EdgeSpec::new("left", "tail"),
        EdgeSpec::new("root", "left"),
    ];
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
    AdmittedMachine::admit(spec, AdmissionLimits::default())
        .map_err(|diagnostics| format!("diamond admission failed: {diagnostics:?}"))
}

fn durable_machine() -> Result<AdmittedMachine, String> {
    let mut spec = MachineSpec::new(
        vec![
            NodeSpec::new("source", Vec::new()),
            NodeSpec::new("gate", Vec::new()),
        ],
        vec![EdgeSpec::new("source", "gate")],
    );
    spec.lifecycle_profile = String::from(DURABLE_SUSPENSION_V1);
    spec.suspensions.push(SuspensionDeclarationSpec {
        node_id: String::from("gate"),
        class: String::from(SUSPENSION_CLASS),
        reference: String::from(SUSPENSION_REFERENCE),
    });
    AdmittedMachine::admit(spec, AdmissionLimits::default())
        .map_err(|diagnostics| format!("durable machine admission failed: {diagnostics:?}"))
}

fn concrete_reference(identity: &str) -> ConcreteReferenceSpec {
    ConcreteReferenceSpec {
        profile: String::from(CANONICAL_REFERENCE_PROFILE_V1),
        identity: String::from(identity),
    }
}

fn concrete_machine_and_intent() -> Result<(AdmittedMachine, ConcreteEffectIntent), String> {
    let mut spec = MachineSpecV2::new(
        vec![NodeSpec::new("effect", vec![String::from("http/request")])],
        Vec::new(),
    );
    spec.effects.push(ConcreteEffectDeclarationSpec {
        declaration_key: String::from(CONCRETE_DECLARATION),
        node_id: String::from("effect"),
        transition: TransitionSelector {
            from: LocalState::Ready,
            action: Action::Start,
        },
        operation: String::from("http/request"),
        resource: String::from("service/example"),
        ability: String::from("http/request"),
        request_binding: ReferenceBindingSpec::Slot {
            slot_id: String::from(CONCRETE_SLOT),
            source_ref: String::from(CONCRETE_SOURCE),
        },
        payload_binding: ReferenceBindingSpec::Exact {
            reference: concrete_reference(PAYLOAD),
        },
    });
    let machine = AdmittedMachine::admit_v2(spec, AdmissionLimits::default())
        .map_err(|diagnostics| format!("concrete machine admission failed: {diagnostics:?}"))?;
    let ready = match step_concrete(
        &machine,
        &machine.initial_state(),
        &ConcreteEvent::new("effect", Action::MarkReady),
    ) {
        ConcreteDecision::Accept { next, intents } if intents.is_empty() => next,
        decision => return Err(format!("concrete mark-ready failed: {decision:?}")),
    };
    let mut start = ConcreteEvent::new("effect", Action::Start);
    start.effects.push(ConcreteEffectRequest {
        declaration_key: String::from(CONCRETE_DECLARATION),
        request: ConcreteBindingInput {
            slot_id: Some(String::from(CONCRETE_SLOT)),
            source_ref: Some(String::from(CONCRETE_SOURCE)),
            reference: concrete_reference(CONCRETE_REQUEST),
        },
        payload: ConcreteBindingInput {
            slot_id: None,
            source_ref: None,
            reference: concrete_reference(PAYLOAD),
        },
    });
    let intent = match step_concrete(&machine, &ready, &start) {
        ConcreteDecision::Accept { mut intents, .. } => intents
            .pop()
            .ok_or_else(|| String::from("concrete start emitted no intent"))?,
        ConcreteDecision::Deny { reason } => {
            return Err(format!("concrete start denied: {reason:?}"));
        }
    };
    Ok((machine, intent))
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

fn framed_material(sources: &[&[u8]]) -> Result<Vec<u8>, String> {
    let mut material = Vec::new();
    for source in sources {
        let source_length = u64::try_from(source.len())
            .map_err(|_| String::from("identity source length exceeds u64"))?;
        material.extend_from_slice(&source_length.to_le_bytes()[..MATERIAL_LENGTH_BYTES]);
        material.extend_from_slice(source);
    }
    Ok(material)
}

fn noncanonical_equivalent(canonical: &[u8]) -> Result<Vec<u8>, String> {
    let mut bytes = canonical.to_vec();
    let offset = bytes
        .windows(CANONICAL_INTEGER_BYTES.len())
        .position(|window| window == CANONICAL_INTEGER_BYTES)
        .ok_or_else(|| String::from("canonical machine lacks the schema-version integer"))?;
    bytes.splice(
        offset..offset + CANONICAL_INTEGER_BYTES.len(),
        NON_CANONICAL_INTEGER_BYTES,
    );
    Ok(bytes)
}

fn machine_variant(
    descriptor: &AggregateMachineDescriptor,
    mutate: impl FnOnce(&mut AggregateMachineDescriptor),
) -> Result<Vec<u8>, String> {
    let mut variant = descriptor.clone();
    mutate(&mut variant);
    encode_machine_descriptor(&variant)
        .map(|artifact| artifact.packed_bytes)
        .map_err(|error| error.to_string())
}

pub fn build_kamacite_corpus() -> Result<Vec<KamaciteCorpusEntry>, String> {
    let machine = diamond_machine()?;
    let generator_material = framed_material(GENERATOR_SOURCES)?;
    let adapter_material = framed_material(ADAPTER_SOURCES)?;
    let toolchain_material = framed_material(TOOLCHAIN_SOURCES)?;
    let set = build_artifact_set(
        &machine,
        IdentityMaterials {
            source_envelope: SOURCE_ENVELOPE,
            proof_set: PROOF_SET,
            generator: &generator_material,
            adapter: &adapter_material,
            toolchain: &toolchain_material,
        },
    )
    .map_err(|error| error.to_string())?;
    let bundle =
        build_receipt_bundle(&set, &successful_trace()).map_err(|error| error.to_string())?;
    let (concrete_machine, concrete_intent) = concrete_machine_and_intent()?;
    let concrete_machine_artifact =
        build_concrete_machine_artifact(&concrete_machine).map_err(|error| error.to_string())?;
    let concrete_intent_artifact =
        build_concrete_intent_artifact(&concrete_machine, &concrete_intent)
            .map_err(|error| error.to_string())?;
    let concrete_receipt =
        build_concrete_intent_receipt(&concrete_machine_artifact, &concrete_intent_artifact)
            .map_err(|error| error.to_string())?;

    let unsupported = machine_variant(&set.machine_descriptor, |descriptor| {
        descriptor.lifecycle_profile_id = String::from("unsupported.lifecycle.v9");
    })?;
    let enumerated = machine_variant(&set.machine_descriptor, |descriptor| {
        descriptor.global_state_model = String::from("enumerated-product-table");
        descriptor.global_state_cardinality = String::from("six-to-the-node-count");
    })?;
    let mismatched_lifecycle_identity =
        compute_identity(ArtifactDomain::Lifecycle, b"different-lifecycle")
            .map_err(|error| error.to_string())?;
    let lifecycle_mismatch = machine_variant(&set.machine_descriptor, |descriptor| {
        descriptor.lifecycle_profile_identity = mismatched_lifecycle_identity;
    })?;

    let mut proof_mismatch = bundle.proof_reference.body.clone();
    proof_mismatch.subject = set.identities.machine.clone();
    let proof_mismatch = encode_link_receipt_body(&proof_mismatch)
        .map_err(|error| error.to_string())?
        .packed_bytes;

    let mut trace_mismatch = bundle.trace.body.clone();
    trace_mismatch.trace_steps[1].to_state = String::from("completed");
    let trace_mismatch = encode_fsm_receipt_body(&trace_mismatch)
        .map_err(|error| error.to_string())?
        .packed_bytes;

    let mut overclaim = bundle.exhaustiveness.body.clone();
    overclaim
        .semantic_claims
        .push(String::from("transition implementation is formally proven"));
    let overclaim = encode_fsm_receipt_body(&overclaim)
        .map_err(|error| error.to_string())?
        .packed_bytes;

    let mut concrete_overclaim = concrete_receipt.body.clone();
    concrete_overclaim
        .semantic_claims
        .push(String::from("runtime effect is authorized"));
    let concrete_overclaim = encode_concrete_intent_receipt_body(&concrete_overclaim)
        .map_err(|error| error.to_string())?
        .packed_bytes;

    let stale_identity = compute_identity(ArtifactDomain::Receipt, b"stale-receipt")
        .map_err(|error| error.to_string())?;
    let stale_identity = format!("{}\n", stale_identity.hex).into_bytes();
    let adapter_identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Adapter, &adapter_material);
    let adapter_identity = format!("{}\n", adapter_identity.hex).into_bytes();

    let source_identity =
        compute_kamacite_identity(KamaciteIdentityDomain::Source, SOURCE_ENVELOPE);
    let durable_lifecycle = build_durable_lifecycle_artifact(Some(source_identity))
        .map_err(|error| error.to_string())?;
    let durable_descriptor = durable_machine_descriptor(
        &durable_machine()?,
        durable_lifecycle.project_identity.clone(),
        set.identities.profile.clone(),
    )
    .map_err(|error| error.to_string())?;
    let durable_artifact = encode_durable_machine_descriptor(&durable_descriptor)
        .map_err(|error| error.to_string())?;
    let mut durable_malformed = durable_descriptor.clone();
    if let Some(suspension) = durable_malformed.suspensions.first_mut() {
        suspension.class = String::from("bad class!");
    }
    let durable_malformed =
        encode_durable_machine_descriptor(&durable_malformed).map_err(|error| error.to_string())?;

    let entries = vec![
        KamaciteCorpusEntry {
            relative_path: "positive/profile.preserves",
            bytes: set.profile_artifact.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/lifecycle.preserves",
            bytes: set.lifecycle.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/machine.preserves",
            bytes: set.machine_artifact.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/admitted-ir.preserves",
            bytes: set.admitted_ir_artifact.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/admission.receipt.preserves",
            bytes: bundle.admission.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/lifecycle.receipt.preserves",
            bytes: bundle.lifecycle.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/source.receipt.preserves",
            bytes: bundle.source.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/proof-reference.receipt.preserves",
            bytes: bundle.proof_reference.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/exhaustiveness.receipt.preserves",
            bytes: bundle.exhaustiveness.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/trace.receipt.preserves",
            bytes: bundle.trace.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/adapter-identity.txt",
            bytes: adapter_identity,
        },
        KamaciteCorpusEntry {
            relative_path: "positive/machine-v2.preserves",
            bytes: concrete_machine_artifact.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/concrete-intent-v2.preserves",
            bytes: concrete_intent_artifact.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/concrete-intent.receipt.preserves",
            bytes: concrete_receipt.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/durable-lifecycle.preserves",
            bytes: durable_lifecycle.canonical.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "positive/durable-machine.preserves",
            bytes: durable_artifact.packed_bytes.clone(),
        },
        KamaciteCorpusEntry {
            relative_path: "negative/malformed.preserves",
            bytes: vec![0xff],
        },
        KamaciteCorpusEntry {
            relative_path: "negative/json-substitution.json",
            bytes: br#"{"schema_version":1}"#.to_vec(),
        },
        KamaciteCorpusEntry {
            relative_path: "negative/noncanonical.preserves",
            bytes: noncanonical_equivalent(&set.machine_artifact.packed_bytes)?,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/unsupported-profile.preserves",
            bytes: unsupported,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/global-enumeration.preserves",
            bytes: enumerated,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/lifecycle-mismatch.preserves",
            bytes: lifecycle_mismatch,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/proof-mismatch.receipt.preserves",
            bytes: proof_mismatch,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/trace-mismatch.receipt.preserves",
            bytes: trace_mismatch,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/overclaim.receipt.preserves",
            bytes: overclaim,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/stale-receipt-identity.txt",
            bytes: stale_identity,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/concrete-intent-overclaim.receipt.preserves",
            bytes: concrete_overclaim,
        },
        KamaciteCorpusEntry {
            relative_path: "negative/durable-malformed-suspension.preserves",
            bytes: durable_malformed.packed_bytes.clone(),
        },
    ];
    let unique_paths = entries
        .iter()
        .map(|entry| entry.relative_path)
        .collect::<BTreeSet<_>>();
    if unique_paths.len() != entries.len() {
        return Err(String::from("Kamacite corpus contains duplicate paths"));
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::build_kamacite_corpus;

    const MINIMUM_POSITIVE_CASES: usize = 10;
    const MINIMUM_NEGATIVE_CASES: usize = 10;

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected success, got {error:?}"),
        }
    }

    fn must_some<T>(value: Option<T>) -> T {
        match value {
            Some(value) => value,
            None => panic!("expected a checked-in corpus fixture"),
        }
    }

    fn checked_in_fixture(relative_path: &str) -> Option<&'static [u8]> {
        let bytes: &[u8] = match relative_path {
            "positive/profile.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/profile.preserves")
            }
            "positive/lifecycle.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/lifecycle.preserves")
            }
            "positive/machine.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/machine.preserves")
            }
            "positive/admitted-ir.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/admitted-ir.preserves")
            }
            "positive/admission.receipt.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/admission.receipt.preserves")
            }
            "positive/lifecycle.receipt.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/lifecycle.receipt.preserves")
            }
            "positive/source.receipt.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/source.receipt.preserves")
            }
            "positive/proof-reference.receipt.preserves" => include_bytes!(
                "../../../fixtures/kamacite/positive/proof-reference.receipt.preserves"
            ),
            "positive/exhaustiveness.receipt.preserves" => include_bytes!(
                "../../../fixtures/kamacite/positive/exhaustiveness.receipt.preserves"
            ),
            "positive/trace.receipt.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/trace.receipt.preserves")
            }
            "positive/adapter-identity.txt" => {
                include_bytes!("../../../fixtures/kamacite/positive/adapter-identity.txt")
            }
            "positive/machine-v2.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/machine-v2.preserves")
            }
            "positive/concrete-intent-v2.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/concrete-intent-v2.preserves")
            }
            "positive/concrete-intent.receipt.preserves" => include_bytes!(
                "../../../fixtures/kamacite/positive/concrete-intent.receipt.preserves"
            ),
            "negative/malformed.preserves" => {
                include_bytes!("../../../fixtures/kamacite/negative/malformed.preserves")
            }
            "negative/json-substitution.json" => {
                include_bytes!("../../../fixtures/kamacite/negative/json-substitution.json")
            }
            "negative/noncanonical.preserves" => {
                include_bytes!("../../../fixtures/kamacite/negative/noncanonical.preserves")
            }
            "negative/unsupported-profile.preserves" => {
                include_bytes!("../../../fixtures/kamacite/negative/unsupported-profile.preserves")
            }
            "negative/global-enumeration.preserves" => {
                include_bytes!("../../../fixtures/kamacite/negative/global-enumeration.preserves")
            }
            "negative/lifecycle-mismatch.preserves" => {
                include_bytes!("../../../fixtures/kamacite/negative/lifecycle-mismatch.preserves")
            }
            "negative/proof-mismatch.receipt.preserves" => include_bytes!(
                "../../../fixtures/kamacite/negative/proof-mismatch.receipt.preserves"
            ),
            "negative/trace-mismatch.receipt.preserves" => include_bytes!(
                "../../../fixtures/kamacite/negative/trace-mismatch.receipt.preserves"
            ),
            "negative/overclaim.receipt.preserves" => {
                include_bytes!("../../../fixtures/kamacite/negative/overclaim.receipt.preserves")
            }
            "negative/stale-receipt-identity.txt" => {
                include_bytes!("../../../fixtures/kamacite/negative/stale-receipt-identity.txt")
            }
            "negative/concrete-intent-overclaim.receipt.preserves" => include_bytes!(
                "../../../fixtures/kamacite/negative/concrete-intent-overclaim.receipt.preserves"
            ),
            "positive/durable-lifecycle.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/durable-lifecycle.preserves")
            }
            "positive/durable-machine.preserves" => {
                include_bytes!("../../../fixtures/kamacite/positive/durable-machine.preserves")
            }
            "negative/durable-malformed-suspension.preserves" => include_bytes!(
                "../../../fixtures/kamacite/negative/durable-malformed-suspension.preserves"
            ),
            _ => return None,
        };
        Some(bytes)
    }

    #[test]
    fn corpus_has_positive_and_negative_cases() {
        let corpus = must(build_kamacite_corpus());
        let positives = corpus
            .iter()
            .filter(|entry| entry.relative_path.starts_with("positive/"))
            .count();
        let negatives = corpus
            .iter()
            .filter(|entry| entry.relative_path.starts_with("negative/"))
            .count();
        assert!(positives >= MINIMUM_POSITIVE_CASES);
        assert!(negatives >= MINIMUM_NEGATIVE_CASES);
    }

    #[test]
    fn checked_in_corpus_matches_the_current_profile_and_adapter() {
        let corpus = must(build_kamacite_corpus());
        for entry in corpus {
            let checked_in = must_some(checked_in_fixture(entry.relative_path));
            assert_eq!(checked_in, entry.bytes, "{} is stale", entry.relative_path);
        }
    }
}
