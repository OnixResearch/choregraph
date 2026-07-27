// r[impl choregraph.nickel.fixtures]
use crate::{
    FrontendIssueCode, FrontendLimits, LoweredSourceFormat, MAX_SAFE_SUMMARY_BYTES, ObservedImport,
    TranslationMaterials, build_translation_receipts, lower_evaluated_json, redact_evaluator_error,
    summary, validate_translation_receipts,
};
use choregraph_core::AdmissionLimits;

const DIAMOND_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/lattice-diamond.json");
const DECLARED_IMPORT_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/declared-import.json");
const MINIMAL_JSON: &[u8] = include_bytes!("../../../fixtures/nickel/evaluated/minimal.json");
const SOURCE_ORDER_JSON: &[u8] =
    include_bytes!("../../../fixtures/nickel/evaluated/source-order.json");

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("expected success, got {error:?}"),
    }
}

fn must_array(value: &mut serde_json::Value) -> &mut Vec<serde_json::Value> {
    match value.as_array_mut() {
        Some(array) => array,
        None => panic!("expected a JSON fixture array"),
    }
}

fn materials<'a>(lowerer: &'a [u8]) -> TranslationMaterials<'a> {
    TranslationMaterials {
        source: b"source",
        import_closure: b"closure",
        evaluator: b"evaluator",
        evaluated_output: b"output",
        lowerer,
        canonical_descriptor: b"descriptor",
        admitted_ir: b"ir",
        proof_set: b"proof",
        toolchain: b"toolchain",
    }
}

#[test]
fn accepts_evaluated_data_and_rejects_ambient_stages() {
    assert!(crate::accepts(crate::LoweringInputStage::EvaluatedData));
    assert!(!crate::accepts(crate::LoweringInputStage::NickelSource));
    assert!(!crate::accepts(crate::LoweringInputStage::DynamicImport));
    assert!(!crate::accepts(crate::LoweringInputStage::EnvironmentValue));
}

#[test]
fn lowers_lattice_port_diamond_to_stable_node_graph() {
    let output = must(lower_evaluated_json(
        DIAMOND_JSON,
        &[],
        FrontendLimits::default(),
    ));
    assert_eq!(output.source_format, LoweredSourceFormat::LatticeWorkflowV1);
    assert_eq!(output.machine.node_ids().len(), 4);
    assert_eq!(output.machine.edges().len(), 4);
    assert_eq!(
        output
            .machine
            .topological_order()
            .iter()
            .map(|node| node.as_str())
            .collect::<Vec<_>>(),
        [
            "lattice-node/1",
            "lattice-node/2",
            "lattice-node/3",
            "lattice-node/4"
        ]
    );
}

#[test]
fn lowers_direct_graph_with_a_declared_content_addressed_import() {
    let output = must(lower_evaluated_json(
        DECLARED_IMPORT_JSON,
        &[ObservedImport {
            logical_id: "defaults",
            bytes: b"",
        }],
        FrontendLimits::default(),
    ));
    assert_eq!(output.import_closure.import_count, 1);
    assert_eq!(output.import_closure.total_bytes, 0);
}

#[test]
fn rejects_unknown_fields_cycles_bad_ports_events_and_excessive_bounds() {
    let unknown = br#"{"source_format":"choregraph-v1","unknown":true}"#;
    let failure = lower_evaluated_json(unknown, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::MalformedEvaluatedValue)
    );

    let mut wrong_direction: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    wrong_direction["nodes"][0]["ports"][0]["direction"] =
        serde_json::Value::String(String::from("output"));
    let wrong_direction = must(serde_json::to_vec(&wrong_direction));
    let failure = lower_evaluated_json(&wrong_direction, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::WrongLatticePortDirection)
    );

    let mut duplicate_port: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    duplicate_port["nodes"][1]["ports"][0]["id"] = serde_json::Value::from(40_u64);
    let duplicate_port = must(serde_json::to_vec(&duplicate_port));
    let failure = lower_evaluated_json(&duplicate_port, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::DuplicateLatticePort)
    );

    let mut duplicate_config: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    must_array(&mut duplicate_config["configs"]).push(serde_json::json!({
        "node_id": 1,
        "operation": "duplicate"
    }));
    let duplicate_config = must(serde_json::to_vec(&duplicate_config));
    let failure = lower_evaluated_json(&duplicate_config, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::DuplicateLatticeConfig)
    );

    let mut missing_port: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    missing_port["edges"][0]["target"] = serde_json::Value::from(999_u64);
    let missing_port = must(serde_json::to_vec(&missing_port));
    let failure = lower_evaluated_json(&missing_port, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::MissingLatticePort)
    );

    let mut cycle: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    must_array(&mut cycle["nodes"][0]["ports"]).push(serde_json::json!({
        "id": 41,
        "direction": "output"
    }));
    must_array(&mut cycle["nodes"][2]["ports"]).push(serde_json::json!({
        "id": 10,
        "direction": "input"
    }));
    must_array(&mut cycle["edges"]).push(serde_json::json!({
        "source": 41,
        "target": 10
    }));
    let cycle = must(serde_json::to_vec(&cycle));
    let failure = lower_evaluated_json(&cycle, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::MachineAdmissionRejected)
    );

    let mut bad_events: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    assert!(must_array(&mut bad_events["events"]).pop().is_some());
    let bad_events = must(serde_json::to_vec(&bad_events));
    let failure = lower_evaluated_json(&bad_events, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::UnsupportedEvents)
    );

    let small = FrontendLimits {
        machine_ceiling: AdmissionLimits {
            max_nodes: 1,
            ..AdmissionLimits::default()
        },
        ..FrontendLimits::default()
    };
    let failure = lower_evaluated_json(DIAMOND_JSON, &[], small);
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::MachineLimitExceedsProfile)
    );
}

#[test]
fn rejects_import_escape_hash_mismatch_and_undeclared_observation() {
    let mut escaped: serde_json::Value = must(serde_json::from_slice(DECLARED_IMPORT_JSON));
    escaped["imports"][0]["relative_path"] = serde_json::Value::String(String::from("../secret"));
    let escaped = must(serde_json::to_vec(&escaped));
    let failure = lower_evaluated_json(&escaped, &[], FrontendLimits::default());
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::InvalidImportPath)
    );

    let failure = lower_evaluated_json(
        DECLARED_IMPORT_JSON,
        &[ObservedImport {
            logical_id: "defaults",
            bytes: b"changed",
        }],
        FrontendLimits::default(),
    );
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::ImportHashMismatch)
    );

    let failure = lower_evaluated_json(
        DIAMOND_JSON,
        &[ObservedImport {
            logical_id: "secret",
            bytes: b"value",
        }],
        FrontendLimits::default(),
    );
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::UndeclaredImportObservation)
    );
}

#[test]
fn source_and_lattice_array_order_variation_lower_to_the_same_machine() {
    let minimal = must(lower_evaluated_json(
        MINIMAL_JSON,
        &[],
        FrontendLimits::default(),
    ));
    let source_order = must(lower_evaluated_json(
        SOURCE_ORDER_JSON,
        &[],
        FrontendLimits::default(),
    ));
    assert_eq!(minimal.machine, source_order.machine);

    let baseline = must(lower_evaluated_json(
        DIAMOND_JSON,
        &[],
        FrontendLimits::default(),
    ));
    let mut reordered: serde_json::Value = must(serde_json::from_slice(DIAMOND_JSON));
    must_array(&mut reordered["nodes"]).reverse();
    must_array(&mut reordered["edges"]).reverse();
    must_array(&mut reordered["configs"]).reverse();
    let reordered = must(serde_json::to_vec(&reordered));
    let candidate = must(lower_evaluated_json(
        &reordered,
        &[],
        FrontendLimits::default(),
    ));
    assert_eq!(baseline.machine, candidate.machine);
    assert_eq!(baseline.import_closure, candidate.import_closure);
}

#[test]
fn translation_receipts_fail_when_lowerer_or_evaluator_identity_is_stale() {
    let baseline_materials = materials(b"lowerer-v1");
    let receipts = must(build_translation_receipts(baseline_materials));
    must(validate_translation_receipts(&receipts, baseline_materials));
    let stale = validate_translation_receipts(&receipts, materials(b"lowerer-v2"));
    assert_eq!(
        stale.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::StaleTranslationReceipt)
    );
    let mut stale_evaluator = baseline_materials;
    stale_evaluator.evaluator = b"evaluator-v2";
    let stale = validate_translation_receipts(&receipts, stale_evaluator);
    assert_eq!(
        stale.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::StaleTranslationReceipt)
    );
    assert!(
        receipts
            .evaluator
            .non_claims
            .iter()
            .any(|claim| claim.contains("does not prove arbitrary Nickel semantic equivalence"))
    );
}

#[test]
fn translation_identity_chain_changes_with_selected_material_only() {
    let first_materials = materials(b"lowerer-v1");
    let first = must(build_translation_receipts(first_materials));
    let mut changed_materials = first_materials;
    changed_materials.evaluated_output = b"output-v2";
    let changed = must(build_translation_receipts(changed_materials));
    assert_ne!(
        first.evaluator.identities.evaluated_output,
        changed.evaluator.identities.evaluated_output
    );
    assert_eq!(
        first.evaluator.identities.source,
        changed.evaluator.identities.source
    );
    assert_eq!(
        first.evaluator.identities.import_closure,
        changed.evaluator.identities.import_closure
    );
    assert_eq!(
        first.evaluator.identities.evaluator,
        changed.evaluator.identities.evaluator
    );
    assert_eq!(
        first.evaluator.identities.lowerer,
        changed.evaluator.identities.lowerer
    );
    assert_eq!(
        first.evaluator.identities.canonical_descriptor,
        changed.evaluator.identities.canonical_descriptor
    );
    assert_eq!(
        first.evaluator.identities.admitted_ir,
        changed.evaluator.identities.admitted_ir
    );
    assert_eq!(
        first.evaluator.identities.proof_set,
        changed.evaluator.identities.proof_set
    );
    assert_eq!(
        first.evaluator.identities.toolchain,
        changed.evaluator.identities.toolchain
    );

    let mut empty = first_materials;
    empty.source = b"";
    let failure = build_translation_receipts(empty);
    assert_eq!(
        failure.err().map(|error| error.issues[0].code),
        Some(FrontendIssueCode::EmptyIdentityMaterial)
    );
}

#[test]
fn safe_diagnostics_are_constant_bounded_and_contain_no_raw_context() {
    let raw = "evaluation failed at /home/operator/private.ncl with credential=secret";
    let redacted = redact_evaluator_error(raw);
    assert_eq!(redacted.code, FrontendIssueCode::EvaluatorRejected);
    assert!(!redacted.summary.contains("/home/"));
    assert!(!redacted.summary.contains("credential=secret"));

    for code in [
        FrontendIssueCode::MalformedEvaluatedValue,
        FrontendIssueCode::ImportHashMismatch,
        FrontendIssueCode::StaleTranslationReceipt,
    ] {
        let text = summary(code);
        assert!(text.len() <= MAX_SAFE_SUMMARY_BYTES);
        assert!(!text.contains("/home/"));
        assert!(!text.contains("credential"));
        assert!(!text.contains("environment value"));
    }
}
