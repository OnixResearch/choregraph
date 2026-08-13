use super::*;

const ROLE_A: RoleId = RoleId(1);
const ROLE_B: RoleId = RoleId(2);
const ROLE_C: RoleId = RoleId(3);
const ROLE_D: RoleId = RoleId(4);
const VALUE_A: ValueId = ValueId(10);
const VALUE_B: ValueId = ValueId(11);
const PAYLOAD_A: PayloadTag = PayloadTag(20);
const PAYLOAD_B: PayloadTag = PayloadTag(21);
const LABEL_A: LabelId = LabelId(30);
const LABEL_B: LabelId = LabelId(31);
const LABEL_C: LabelId = LabelId(32);
const SPAN_START: u32 = 0;
const SPAN_END: u32 = 8;
const ALTERNATE_SPAN_START: u32 = 9;
const ALTERNATE_SPAN_END: u32 = 17;
const OVERSIZED_ROLE_COUNT: usize = MAX_ROLES.saturating_add(1);

fn span() -> SourceSpan {
    SourceSpan {
        start: SPAN_START,
        end: SPAN_END,
    }
}

fn alternate_span() -> SourceSpan {
    SourceSpan {
        start: ALTERNATE_SPAN_START,
        end: ALTERNATE_SPAN_END,
    }
}

fn exact_identity(seed: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(seed).to_hex())
}

fn native(id: RoleId) -> RoleDeclaration {
    RoleDeclaration {
        id,
        kind: RoleKind::Native,
        span: span(),
    }
}

fn external(id: RoleId) -> RoleDeclaration {
    RoleDeclaration {
        id,
        kind: RoleKind::External {
            contract_identity: exact_identity(b"contract"),
            input_schema_identity: exact_identity(b"input"),
            output_schema_identity: exact_identity(b"output"),
        },
        span: span(),
    }
}

fn value(id: ValueId, payload_tag: PayloadTag, owner: RoleId) -> ValueDeclaration {
    ValueDeclaration {
        id,
        payload_tag,
        initial_owner: owner,
        span: span(),
    }
}

fn end() -> ChoreographyNode {
    ChoreographyNode::End { span: span() }
}

fn communication(
    from: RoleId,
    to: RoleId,
    label: LabelId,
    payload_tag: PayloadTag,
    next: ChoreographyNode,
) -> ChoreographyNode {
    ChoreographyNode::Communication {
        from,
        to,
        label,
        payload_tag,
        next: Box::new(next),
        span: span(),
    }
}

fn transfer(
    from: RoleId,
    to: RoleId,
    label: LabelId,
    payload_tag: PayloadTag,
    value: ValueId,
    next: ChoreographyNode,
) -> ChoreographyNode {
    ChoreographyNode::Transfer {
        from,
        to,
        label,
        payload_tag,
        value,
        next: Box::new(next),
        span: span(),
    }
}

fn source(protocol: ChoreographyNode) -> RoleChoreographySource {
    RoleChoreographySource {
        schema: String::from(ROLE_CHOREOGRAPHY_SCHEMA),
        limits: ChoreographyLimits::reviewed(),
        imports: Vec::new(),
        roles: vec![native(ROLE_A), native(ROLE_B), native(ROLE_C)],
        values: vec![value(VALUE_A, PAYLOAD_A, ROLE_A)],
        protocol,
        non_claims: REQUIRED_NON_CLAIMS.map(String::from).to_vec(),
    }
}

fn two_role_source() -> RoleChoreographySource {
    let mut result = source(transfer(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    result.roles.pop();
    result
}

fn must_admit(source: RoleChoreographySource) -> AdmittedChoreography {
    match admit_source(source) {
        Ok(admitted) => admitted,
        Err(diagnostics) => panic!("admission failed: {diagnostics:?}"),
    }
}

fn must_project(source: RoleChoreographySource) -> ProjectedChoreography {
    match project_all(&must_admit(source)) {
        Ok(projected) => projected,
        Err(diagnostics) => panic!("projection failed: {diagnostics:?}"),
    }
}

fn must_bundle(source: RoleChoreographySource) -> ProjectionBundle {
    let admitted = must_admit(source);
    let materials = ProjectionMaterials::reviewed(b"source envelope", b"evaluator output");
    match build_projection_bundle(&admitted, &materials) {
        Ok(bundle) => bundle,
        Err(diagnostics) => panic!("artifact build failed: {diagnostics:?}"),
    }
}

fn first_code(result: Result<AdmittedChoreography, Vec<ChoreographyDiagnostic>>) -> DiagnosticCode {
    match result {
        Ok(_) => panic!("invalid source unexpectedly passed"),
        Err(diagnostics) => match diagnostics.first() {
            Some(diagnostic) => diagnostic.code,
            None => panic!("failed admission returned no diagnostic"),
        },
    }
}

#[test]
fn two_role_transfer_matches_trellis_sender_and_receiver() {
    let projected = must_project(two_role_source());
    assert_eq!(projected.locals.locals.len(), 2);
    assert!(matches!(
        projected.locals.locals[0].protocol,
        LocalProtocol::SendOwned {
            peer: ROLE_B,
            label: LABEL_A,
            payload_tag: PAYLOAD_A,
            value: VALUE_A,
            ref next,
        } if **next == LocalProtocol::End
    ));
    assert!(matches!(
        projected.locals.locals[1].protocol,
        LocalProtocol::ReceiveOwned {
            peer: ROLE_A,
            label: LABEL_A,
            payload_tag: PAYLOAD_A,
            value: VALUE_A,
            ref next,
        } if **next == LocalProtocol::End
    ));
}

#[test]
fn chained_and_return_transfer_preserve_exact_value() {
    let protocol = transfer(
        ROLE_A,
        ROLE_B,
        LABEL_A,
        PAYLOAD_A,
        VALUE_A,
        transfer(
            ROLE_B,
            ROLE_C,
            LABEL_B,
            PAYLOAD_A,
            VALUE_A,
            transfer(ROLE_C, ROLE_A, LABEL_C, PAYLOAD_A, VALUE_A, end()),
        ),
    );
    let projected = must_project(source(protocol));
    assert_eq!(projected.locals.locals.len(), 3);
    assert_eq!(projected.locals.locals[0].role, ROLE_A);
    assert_eq!(projected.locals.locals[1].role, ROLE_B);
    assert_eq!(projected.locals.locals[2].role, ROLE_C);
}

#[test]
fn projectable_choice_includes_uninvolved_role() {
    let protocol = ChoreographyNode::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_A,
                body: communication(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, end()),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_B,
                body: communication(ROLE_A, ROLE_B, LABEL_B, PAYLOAD_B, end()),
                span: span(),
            },
        ],
        span: span(),
    };
    let projected = must_project(source(protocol));
    assert!(matches!(
        projected.locals.locals[0].protocol,
        LocalProtocol::InternalChoice { .. }
    ));
    assert!(matches!(
        projected.locals.locals[1].protocol,
        LocalProtocol::Offer { from: ROLE_A, .. }
    ));
    assert_eq!(projected.locals.locals[2].protocol, LocalProtocol::End);
}

#[test]
fn external_role_is_identity_only_protocol_data() {
    let mut candidate = two_role_source();
    candidate.roles[1] = external(ROLE_B);
    let projected = must_project(candidate);
    assert!(matches!(
        projected.locals.locals[1].role_kind,
        RoleKind::External { .. }
    ));
}

#[test]
fn canonical_bundle_binds_every_local_and_rejects_receipt_mutation() {
    let source = two_role_source();
    if let Err(error) = choregraph_kamacite::encode_role_choreography_source(&source) {
        panic!("source encoding failed: {error}");
    }
    let bundle = must_bundle(source);
    assert_eq!(bundle.locals.len(), 2);
    assert_eq!(bundle.receipt_body.local_artifacts.len(), 2);
    assert_eq!(bundle.receipt_body.trellis_revision, TRELLIS_REVISION);
    assert!(identity_is_valid(
        &bundle.receipt.identity,
        ChoreographyIdentityDomain::ProjectionReceipt
    ));
    assert!(
        verify_projection_receipt(&bundle, &bundle.receipt_body, &bundle.receipt.packed_bytes)
            .is_ok()
    );

    let mut stale = bundle.receipt_body.clone();
    stale.trellis_revision = String::from(TRELLIS_SEMANTIC_COMMIT);
    assert!(verify_projection_receipt(&bundle, &stale, &bundle.receipt.packed_bytes).is_err());

    let mut stale_schema = bundle.receipt_body.clone();
    stale_schema.schema_set = bundle.receipt_body.source_envelope.clone();
    assert!(
        verify_projection_receipt(&bundle, &stale_schema, &bundle.receipt.packed_bytes).is_err()
    );

    let mut stale_lowerer = bundle.receipt_body.clone();
    stale_lowerer.lowerer = bundle.receipt_body.source_envelope.clone();
    assert!(
        verify_projection_receipt(&bundle, &stale_lowerer, &bundle.receipt.packed_bytes).is_err()
    );

    let mut stale_proof_set = bundle.receipt_body.clone();
    stale_proof_set.trellis_proof_set = bundle.receipt_body.source_envelope.clone();
    assert!(
        verify_projection_receipt(&bundle, &stale_proof_set, &bundle.receipt.packed_bytes).is_err()
    );

    let mut wrong_local = bundle.receipt_body.clone();
    wrong_local.local_artifacts[0] = bundle.receipt_body.global_artifact.clone();
    assert!(
        verify_projection_receipt(&bundle, &wrong_local, &bundle.receipt.packed_bytes).is_err()
    );

    let mut wrong_order = bundle.receipt_body.clone();
    wrong_order.local_artifacts.swap(0, 1);
    assert!(
        verify_projection_receipt(&bundle, &wrong_order, &bundle.receipt.packed_bytes).is_err()
    );

    let mut wrong_set = bundle.receipt_body.clone();
    wrong_set.complete_local_set = bundle.receipt_body.source_envelope.clone();
    assert!(verify_projection_receipt(&bundle, &wrong_set, &bundle.receipt.packed_bytes).is_err());

    let forbidden_claims = [
        "trellis-proof-transferred",
        "deadlock-freedom-under-faults-proven",
        "network-delivery-proven",
        "release-eligibility-proven",
    ];
    for forbidden_claim in forbidden_claims {
        let mut overclaim = bundle.receipt_body.clone();
        overclaim.evidence_roles[0].claim = String::from(forbidden_claim);
        assert!(
            verify_projection_receipt(&bundle, &overclaim, &bundle.receipt.packed_bytes).is_err()
        );
    }
}

#[test]
fn local_continuation_change_invalidates_local_set_and_receipt() {
    let original = must_bundle(two_role_source());
    let mut changed_source = two_role_source();
    changed_source.protocol = transfer(
        ROLE_A,
        ROLE_B,
        LABEL_A,
        PAYLOAD_A,
        VALUE_A,
        communication(ROLE_B, ROLE_A, LABEL_B, PAYLOAD_B, end()),
    );
    let changed = must_bundle(changed_source);

    assert_ne!(
        original.receipt_body.local_artifacts,
        changed.receipt_body.local_artifacts
    );
    assert_ne!(
        original.receipt_body.complete_local_set,
        changed.receipt_body.complete_local_set
    );
    assert!(
        verify_projection_receipt(
            &changed,
            &original.receipt_body,
            &original.receipt.packed_bytes,
        )
        .is_err()
    );
}

#[test]
fn duplicate_and_undeclared_roles_fail() {
    let mut duplicate = two_role_source();
    duplicate.roles.push(native(ROLE_A));
    assert_eq!(
        first_code(admit_source(duplicate)),
        DiagnosticCode::DuplicateRole
    );

    let undeclared = source(transfer(ROLE_A, ROLE_D, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    assert_eq!(
        first_code(admit_source(undeclared)),
        DiagnosticCode::UndeclaredRole
    );
}

#[test]
fn self_transfer_and_stale_owner_fail() {
    let self_transfer = source(transfer(ROLE_A, ROLE_A, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    assert_eq!(
        first_code(admit_source(self_transfer)),
        DiagnosticCode::SelfTransfer
    );

    let stale = source(transfer(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    assert_eq!(first_code(admit_source(stale)), DiagnosticCode::StaleOwner);
}

#[test]
fn wrong_value_and_payload_fail() {
    let unknown = source(transfer(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, VALUE_B, end()));
    assert_eq!(
        first_code(admit_source(unknown)),
        DiagnosticCode::UnknownValue
    );

    let mismatch = source(transfer(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_B, VALUE_A, end()));
    assert_eq!(
        first_code(admit_source(mismatch)),
        DiagnosticCode::PayloadMismatch
    );
}

#[test]
fn duplicate_and_unsorted_branch_labels_fail() {
    let duplicate = source(ChoreographyNode::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_A,
                body: end(),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_A,
                body: end(),
                span: span(),
            },
        ],
        span: span(),
    });
    assert_eq!(
        first_code(admit_source(duplicate)),
        DiagnosticCode::DuplicateBranchLabel
    );

    let unsorted = source(ChoreographyNode::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_B,
                body: end(),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_A,
                body: end(),
                span: span(),
            },
        ],
        span: span(),
    });
    assert_eq!(
        first_code(admit_source(unsorted)),
        DiagnosticCode::UnsortedBranchLabel
    );
}

#[test]
fn incompatible_offer_heads_are_not_projectable() {
    let protocol = ChoreographyNode::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_A,
                body: transfer(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, VALUE_A, end()),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_B,
                body: transfer(ROLE_C, ROLE_B, LABEL_B, PAYLOAD_B, VALUE_B, end()),
                span: alternate_span(),
            },
        ],
        span: span(),
    };
    let mut candidate = source(protocol);
    candidate.values[0].initial_owner = ROLE_B;
    candidate.values.push(value(VALUE_B, PAYLOAD_B, ROLE_C));
    let diagnostics = match admit_source(candidate) {
        Ok(_) => panic!("incompatible offer unexpectedly admitted"),
        Err(diagnostics) => diagnostics,
    };
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::TrellisProjectionRejected
    );
    assert_eq!(diagnostics[0].secondary, vec![span(), alternate_span()]);
    assert!(diagnostics_are_safe(&diagnostics));
}

#[test]
fn malformed_external_role_and_limit_widening_fail() {
    let mut external_source = two_role_source();
    external_source.roles[1] = RoleDeclaration {
        id: ROLE_B,
        kind: RoleKind::External {
            contract_identity: String::from("ambient-contract"),
            input_schema_identity: exact_identity(b"input"),
            output_schema_identity: exact_identity(b"output"),
        },
        span: span(),
    };
    assert_eq!(
        first_code(admit_source(external_source)),
        DiagnosticCode::InvalidRoleKind
    );

    let mut widened = two_role_source();
    widened.limits.max_roles = OVERSIZED_ROLE_COUNT;
    assert_eq!(
        first_code(admit_source(widened)),
        DiagnosticCode::LimitsMismatch
    );
}

#[test]
fn oversized_role_and_identifier_inputs_fail() {
    let mut invalid_roles_fixture = source(end());
    invalid_roles_fixture.roles.clear();
    for raw in 0..OVERSIZED_ROLE_COUNT {
        let id = match u32::try_from(raw) {
            Ok(id) => RoleId(id),
            Err(error) => panic!("role fixture identifier does not fit: {error}"),
        };
        invalid_roles_fixture.roles.push(native(id));
    }
    assert_eq!(
        first_code(admit_source(invalid_roles_fixture)),
        DiagnosticCode::RoleLimitExceeded
    );

    let mut invalid_identifier_fixture = two_role_source();
    invalid_identifier_fixture.imports.push(ImportIdentity {
        logical_id: String::from("x").repeat(MAX_IDENTIFIER_BYTES.saturating_add(1)),
        blake3: blake3::hash(b"oversized-import").to_hex().to_string(),
    });
    assert_eq!(
        first_code(admit_source(invalid_identifier_fixture)),
        DiagnosticCode::InvalidImportIdentity
    );
}

#[test]
fn unknown_fields_and_missing_non_claims_fail() {
    let source = two_role_source();
    let mut value = match serde_json::to_value(source) {
        Ok(value) => value,
        Err(error) => panic!("serialize fixture: {error}"),
    };
    let Some(object) = value.as_object_mut() else {
        panic!("source fixture is not an object");
    };
    object.insert(
        String::from("ambient_authority"),
        serde_json::Value::Bool(true),
    );
    let bytes = match serde_json::to_vec(&value) {
        Ok(bytes) => bytes,
        Err(error) => panic!("serialize changed fixture: {error}"),
    };
    let diagnostics = match parse_evaluated_json(&bytes) {
        Ok(_) => panic!("unknown field unexpectedly passed"),
        Err(diagnostics) => diagnostics,
    };
    assert_eq!(diagnostics[0].code, DiagnosticCode::MalformedEvaluatedData);

    let mut missing = two_role_source();
    missing.non_claims.pop();
    assert_eq!(
        first_code(admit_source(missing)),
        DiagnosticCode::MissingNonClaim
    );
}

#[test]
fn complete_set_rejects_missing_duplicate_and_reordered_roles() {
    let projected = must_project(two_role_source());
    let expected = [ROLE_A, ROLE_B];

    let mut missing = projected.locals.clone();
    missing.locals.pop();
    assert!(validate_complete_set(&expected, &missing).is_err());

    let mut duplicate = projected.locals.clone();
    duplicate.locals[1].role = ROLE_A;
    assert!(validate_complete_set(&expected, &duplicate).is_err());

    let mut reordered = projected.locals;
    reordered.locals.swap(0, 1);
    assert!(validate_complete_set(&expected, &reordered).is_err());
}

#[test]
fn diagnostics_reject_host_and_secret_leakage() {
    assert!(!safe_diagnostic_text("/home/operator/private"));
    assert!(!safe_diagnostic_text("/etc/operator-private"));
    assert!(!safe_diagnostic_text("C:\\operator-private"));
    assert!(!safe_diagnostic_text("SECRET value"));
    assert!(!safe_diagnostic_text("token value"));
    assert!(!safe_diagnostic_text("PATH=value"));

    let unsafe_diagnostic = ChoreographyDiagnostic {
        code: DiagnosticCode::UnsafeDiagnostic,
        severity: DiagnosticSeverity::Error,
        primary: span(),
        secondary: Vec::new(),
        notes: vec![String::from("/home/operator/private")],
        hints: Vec::new(),
        safe_replacement: None,
    };
    assert!(!diagnostics_are_safe(&[unsafe_diagnostic]));
}
