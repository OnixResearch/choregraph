// r[verify choregraph.role_choreography.source.valid]
// r[verify choregraph.role_choreography.source.undeclared]
// r[verify choregraph.role_choreography.bounds.exceeded]
// r[verify choregraph.role_choreography.admission.stale_owner]
// r[verify choregraph.role_choreography.roles.external]
// r[verify choregraph.role_choreography.roles.mixed]
// r[verify choregraph.role_choreography.lowering.valid]
// r[verify choregraph.role_choreography.lowering.dropped]
// r[verify choregraph.role_choreography.trellis.unlinked]
// r[verify choregraph.role_choreography.projection.valid]
// r[verify choregraph.role_choreography.projection.conflict]
// r[verify choregraph.role_choreography.complete_set.missing]
// r[verify choregraph.role_choreography.identity.changed]
// r[verify choregraph.role_choreography.artifacts.valid]
// r[verify choregraph.role_choreography.diagnostics.redacted]
// r[verify choregraph.role_choreography.diagnostics.multispan]
// r[verify choregraph.role_choreography.reference.adopted]
// r[verify choregraph.role_choreography.validation.stale]
// r[verify choregraph.role_choreography.fixtures.coverage]
// r[verify choregraph.role_choreography.boundary.delivery]

const ROLE_A: crate::model::participants::RoleId = crate::model::participants::RoleId(1);
const ROLE_B: crate::model::participants::RoleId = crate::model::participants::RoleId(2);
const ROLE_C: crate::model::participants::RoleId = crate::model::participants::RoleId(3);
const ROLE_D: crate::model::participants::RoleId = crate::model::participants::RoleId(4);
const VALUE_A: crate::model::participants::ValueId = crate::model::participants::ValueId(10);
const VALUE_B: crate::model::participants::ValueId = crate::model::participants::ValueId(11);
const PAYLOAD_A: crate::model::participants::PayloadTag =
    crate::model::participants::PayloadTag(20);
const PAYLOAD_B: crate::model::participants::PayloadTag =
    crate::model::participants::PayloadTag(21);
const LABEL_A: crate::model::participants::LabelId = crate::model::participants::LabelId(30);
const LABEL_B: crate::model::participants::LabelId = crate::model::participants::LabelId(31);
const LABEL_C: crate::model::participants::LabelId = crate::model::participants::LabelId(32);
const SPAN_START: u32 = 0;
const SPAN_END: u32 = 8;
const ALTERNATE_SPAN_START: u32 = 9;
const ALTERNATE_SPAN_END: u32 = 17;
const OVERSIZED_ROLE_COUNT: usize = crate::model::bounds::MAX_ROLES.saturating_add(1);

fn span() -> crate::model::participants::SourceSpan {
    crate::model::participants::SourceSpan {
        start: SPAN_START,
        end: SPAN_END,
    }
}

fn alternate_span() -> crate::model::participants::SourceSpan {
    crate::model::participants::SourceSpan {
        start: ALTERNATE_SPAN_START,
        end: ALTERNATE_SPAN_END,
    }
}

fn exact_identity(seed: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(seed).to_hex())
}

fn native(id: crate::model::participants::RoleId) -> crate::model::participants::RoleDeclaration {
    crate::model::participants::RoleDeclaration {
        id,
        kind: crate::model::participants::RoleKind::Native,
        span: span(),
    }
}

fn external(id: crate::model::participants::RoleId) -> crate::model::participants::RoleDeclaration {
    crate::model::participants::RoleDeclaration {
        id,
        kind: crate::model::participants::RoleKind::External {
            contract_identity: exact_identity(b"contract"),
            input_schema_identity: exact_identity(b"input"),
            output_schema_identity: exact_identity(b"output"),
        },
        span: span(),
    }
}

fn value(
    id: crate::model::participants::ValueId,
    payload_tag: crate::model::participants::PayloadTag,
    owner: crate::model::participants::RoleId,
) -> crate::model::participants::ValueDeclaration {
    crate::model::participants::ValueDeclaration {
        id,
        payload_tag,
        initial_owner: owner,
        span: span(),
    }
}

fn end() -> crate::model::tree::Node {
    crate::model::tree::Node::End { span: span() }
}

fn communication(
    from: crate::model::participants::RoleId,
    to: crate::model::participants::RoleId,
    label: crate::model::participants::LabelId,
    payload_tag: crate::model::participants::PayloadTag,
    next: crate::model::tree::Node,
) -> crate::model::tree::Node {
    crate::model::tree::Node::Communication {
        from,
        to,
        label,
        payload_tag,
        next: Box::new(next),
        span: span(),
    }
}

fn transfer(
    from: crate::model::participants::RoleId,
    to: crate::model::participants::RoleId,
    label: crate::model::participants::LabelId,
    payload_tag: crate::model::participants::PayloadTag,
    value: crate::model::participants::ValueId,
    next: crate::model::tree::Node,
) -> crate::model::tree::Node {
    crate::model::tree::Node::Transfer {
        from,
        to,
        label,
        payload_tag,
        value,
        next: Box::new(next),
        span: span(),
    }
}

fn source(protocol: crate::model::tree::Node) -> crate::model::schema::RoleSource {
    crate::model::schema::RoleSource {
        schema: String::from(crate::model::schema::ROLE_CHOREOGRAPHY_SCHEMA),
        limits: crate::model::bounds::Limits::reviewed(),
        imports: Vec::new(),
        roles: vec![native(ROLE_A), native(ROLE_B), native(ROLE_C)],
        values: vec![value(VALUE_A, PAYLOAD_A, ROLE_A)],
        protocol,
        non_claims: crate::model::schema::REQUIRED_NON_CLAIMS
            .map(String::from)
            .to_vec(),
    }
}

fn two_role_source() -> crate::model::schema::RoleSource {
    let mut result = source(transfer(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    result.roles.pop();
    result
}

fn must_admit(source: crate::model::schema::RoleSource) -> crate::admission::Admitted {
    match crate::admission::admit_source(source) {
        Ok(admitted) => admitted,
        Err(diagnostics) => panic!("admission failed: {diagnostics:?}"),
    }
}

fn must_project(source: crate::model::schema::RoleSource) -> crate::endpoint::Projected {
    match crate::endpoint::project_all(&must_admit(source)) {
        Ok(projected) => projected,
        Err(diagnostics) => panic!("projection failed: {diagnostics:?}"),
    }
}

fn must_bundle(source: crate::model::schema::RoleSource) -> crate::linkage::ProjectionBundle {
    let admitted = must_admit(source);
    let materials =
        crate::linkage::ProjectionMaterials::reviewed(b"source envelope", b"evaluator output");
    match crate::linkage::build_projection_bundle(&admitted, &materials) {
        Ok(bundle) => bundle,
        Err(diagnostics) => panic!("artifact build failed: {diagnostics:?}"),
    }
}

fn first_code(
    result: Result<crate::admission::Admitted, crate::admission::Error>,
) -> crate::issue::DiagnosticCode {
    match result {
        Ok(_) => panic!("invalid source unexpectedly passed"),
        Err(crate::admission::Error::Diagnostics(diagnostics)) => match diagnostics.first() {
            Some(diagnostic) => diagnostic.code,
            None => panic!("failed admission returned no diagnostic"),
        },
        Err(crate::admission::Error::UnsupportedNode) => panic!("known test node was unsupported"),
    }
}

#[test]
fn two_role_transfer_matches_trellis_sender_and_receiver() {
    let projected = must_project(two_role_source());
    assert_eq!(projected.locals.locals.len(), 2);
    assert!(matches!(
        projected.locals.locals[0].protocol,
        crate::endpoint::LocalProtocol::SendOwned {
            peer: ROLE_B,
            label: LABEL_A,
            payload_tag: PAYLOAD_A,
            value: VALUE_A,
            ref next,
        } if **next == crate::endpoint::LocalProtocol::End
    ));
    assert!(matches!(
        projected.locals.locals[1].protocol,
        crate::endpoint::LocalProtocol::ReceiveOwned {
            peer: ROLE_A,
            label: LABEL_A,
            payload_tag: PAYLOAD_A,
            value: VALUE_A,
            ref next,
        } if **next == crate::endpoint::LocalProtocol::End
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
    let protocol = crate::model::tree::Node::Choice {
        decider: ROLE_A,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: communication(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, end()),
                span: span(),
            },
            crate::model::tree::ChoiceBranch {
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
        crate::endpoint::LocalProtocol::InternalChoice { .. }
    ));
    assert!(matches!(
        projected.locals.locals[1].protocol,
        crate::endpoint::LocalProtocol::Offer { from: ROLE_A, .. }
    ));
    assert_eq!(
        projected.locals.locals[2].protocol,
        crate::endpoint::LocalProtocol::End
    );
}

#[test]
fn external_role_is_identity_only_protocol_data() {
    let mut candidate = two_role_source();
    candidate.roles[1] = external(ROLE_B);
    let projected = must_project(candidate);
    assert!(matches!(
        projected.locals.locals[1].role_kind,
        crate::model::participants::RoleKind::External { .. }
    ));
}

#[test]
fn canonical_bundle_binds_every_local_and_rejects_receipt_mutation() {
    let source = two_role_source();
    if let Err(error) = choregraph_kamacite::profile::source::encode(&source) {
        panic!("source encoding failed: {error}");
    }
    let bundle = must_bundle(source);
    assert_eq!(bundle.locals.len(), 2);
    assert_eq!(bundle.receipt_body.local_artifacts.len(), 2);
    assert_eq!(
        bundle.receipt_body.trellis_revision,
        crate::linkage::provenance::TRELLIS_REVISION
    );
    assert!(crate::linkage::digest::identity_is_valid(
        &bundle.receipt.identity,
        crate::linkage::digest::IdentityDomain::ProjectionReceipt
    ));
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &bundle.receipt_body,
            &bundle.receipt.packed_bytes
        )
        .is_ok()
    );

    let mut stale = bundle.receipt_body.clone();
    stale.trellis_revision = String::from(crate::linkage::provenance::TRELLIS_SEMANTIC_COMMIT);
    assert!(
        crate::linkage::verify_projection_receipt(&bundle, &stale, &bundle.receipt.packed_bytes)
            .is_err()
    );

    let mut stale_schema = bundle.receipt_body.clone();
    stale_schema.schema_set = bundle.receipt_body.source_envelope.clone();
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &stale_schema,
            &bundle.receipt.packed_bytes
        )
        .is_err()
    );

    let mut stale_lowerer = bundle.receipt_body.clone();
    stale_lowerer.lowerer = bundle.receipt_body.source_envelope.clone();
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &stale_lowerer,
            &bundle.receipt.packed_bytes
        )
        .is_err()
    );

    let mut stale_proof_set = bundle.receipt_body.clone();
    stale_proof_set.trellis_proof_set = bundle.receipt_body.source_envelope.clone();
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &stale_proof_set,
            &bundle.receipt.packed_bytes
        )
        .is_err()
    );

    let mut wrong_local = bundle.receipt_body.clone();
    wrong_local.local_artifacts[0] = bundle.receipt_body.global_artifact.clone();
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &wrong_local,
            &bundle.receipt.packed_bytes
        )
        .is_err()
    );

    let mut wrong_order = bundle.receipt_body.clone();
    wrong_order.local_artifacts.swap(0, 1);
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &wrong_order,
            &bundle.receipt.packed_bytes
        )
        .is_err()
    );

    let mut wrong_set = bundle.receipt_body.clone();
    wrong_set.complete_local_set = bundle.receipt_body.source_envelope.clone();
    assert!(
        crate::linkage::verify_projection_receipt(
            &bundle,
            &wrong_set,
            &bundle.receipt.packed_bytes
        )
        .is_err()
    );

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
            crate::linkage::verify_projection_receipt(
                &bundle,
                &overclaim,
                &bundle.receipt.packed_bytes
            )
            .is_err()
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
        crate::linkage::verify_projection_receipt(
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
        first_code(crate::admission::admit_source(duplicate)),
        crate::issue::DiagnosticCode::DuplicateRole
    );

    let undeclared = source(transfer(ROLE_A, ROLE_D, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    assert_eq!(
        first_code(crate::admission::admit_source(undeclared)),
        crate::issue::DiagnosticCode::UndeclaredRole
    );
}

#[test]
fn self_transfer_and_stale_owner_fail() {
    let self_transfer = source(transfer(ROLE_A, ROLE_A, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    assert_eq!(
        first_code(crate::admission::admit_source(self_transfer)),
        crate::issue::DiagnosticCode::SelfTransfer
    );

    let stale = source(transfer(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, VALUE_A, end()));
    assert_eq!(
        first_code(crate::admission::admit_source(stale)),
        crate::issue::DiagnosticCode::StaleOwner
    );
}

#[test]
fn wrong_value_and_payload_fail() {
    let unknown = source(transfer(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, VALUE_B, end()));
    assert_eq!(
        first_code(crate::admission::admit_source(unknown)),
        crate::issue::DiagnosticCode::UnknownValue
    );

    let mismatch = source(transfer(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_B, VALUE_A, end()));
    assert_eq!(
        first_code(crate::admission::admit_source(mismatch)),
        crate::issue::DiagnosticCode::PayloadMismatch
    );
}

#[test]
fn duplicate_and_unsorted_branch_labels_fail() {
    let duplicate = source(crate::model::tree::Node::Choice {
        decider: ROLE_A,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: end(),
                span: span(),
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: end(),
                span: span(),
            },
        ],
        span: span(),
    });
    assert_eq!(
        first_code(crate::admission::admit_source(duplicate)),
        crate::issue::DiagnosticCode::DuplicateBranchLabel
    );

    let unsorted = source(crate::model::tree::Node::Choice {
        decider: ROLE_A,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_B,
                body: end(),
                span: span(),
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: end(),
                span: span(),
            },
        ],
        span: span(),
    });
    assert_eq!(
        first_code(crate::admission::admit_source(unsorted)),
        crate::issue::DiagnosticCode::UnsortedBranchLabel
    );
}

#[test]
fn incompatible_offer_heads_are_not_projectable() {
    let protocol = crate::model::tree::Node::Choice {
        decider: ROLE_A,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: transfer(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, VALUE_A, end()),
                span: span(),
            },
            crate::model::tree::ChoiceBranch {
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
    let diagnostics = match crate::admission::admit_source(candidate) {
        Ok(_) => panic!("incompatible offer unexpectedly admitted"),
        Err(crate::admission::Error::Diagnostics(diagnostics)) => diagnostics,
        Err(crate::admission::Error::UnsupportedNode) => panic!("known test node was unsupported"),
    };
    assert_eq!(
        diagnostics[0].code,
        crate::issue::DiagnosticCode::TrellisProjectionRejected
    );
    assert_eq!(diagnostics[0].primary, span());
    assert_eq!(diagnostics[0].secondary, vec![span(), alternate_span()]);
    assert!(
        diagnostics[0]
            .notes
            .iter()
            .any(|note| note.contains("role 2") && note.contains("incompatible offer heads"))
    );
    assert!(
        diagnostics[0]
            .hints
            .iter()
            .any(|hint| hint.contains("may not suffice"))
    );
    assert!(crate::issue::diagnostics_are_safe(&diagnostics));
    let rendered = crate::issue::render_safe_diagnostics(&diagnostics);
    assert!(rendered.contains("TrellisProjectionRejected:0..8"));
    assert!(rendered.contains("secondary: 0..8, 9..17"));
    assert!(rendered.contains("note: "));
    assert!(rendered.contains("hint: "));
    assert!(rendered.len() <= crate::model::bounds::MAX_DIAGNOSTIC_TEXT_BYTES);
}

const fn span_range(range: core::ops::Range<u32>) -> crate::model::participants::SourceSpan {
    crate::model::participants::SourceSpan {
        start: range.start,
        end: range.end,
    }
}

#[test]
fn rejected_choice_beneath_leading_communication_reports_choice_span_and_role() {
    let root_span = span_range(0..100);
    let choice_span = span_range(20..90);
    let branch_a_span = span_range(25..50);
    let branch_b_span = span_range(55..85);
    let protocol = crate::model::tree::Node::Communication {
        from: ROLE_A,
        to: ROLE_B,
        label: LABEL_A,
        payload_tag: PAYLOAD_A,
        next: Box::new(crate::model::tree::Node::Choice {
            decider: ROLE_A,
            branches: vec![
                crate::model::tree::ChoiceBranch {
                    label: LABEL_A,
                    body: communication(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, end()),
                    span: branch_a_span,
                },
                crate::model::tree::ChoiceBranch {
                    label: LABEL_B,
                    body: communication(ROLE_C, ROLE_B, LABEL_B, PAYLOAD_B, end()),
                    span: branch_b_span,
                },
            ],
            span: choice_span,
        }),
        span: root_span,
    };
    let diagnostics = match crate::admission::admit_source(source(protocol)) {
        Ok(_) => panic!("unprojectable choice beneath communication unexpectedly admitted"),
        Err(crate::admission::Error::Diagnostics(diagnostics)) => diagnostics,
        Err(crate::admission::Error::UnsupportedNode) => panic!("known test node was unsupported"),
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        crate::issue::DiagnosticCode::TrellisProjectionRejected
    );
    assert_eq!(diagnostics[0].primary, choice_span);
    assert_ne!(diagnostics[0].primary, root_span);
    assert_eq!(diagnostics[0].secondary, vec![branch_a_span, branch_b_span]);
    assert!(diagnostics[0].notes.iter().any(|note| {
        note.contains("role 2")
            && note.contains("decider 1")
            && note.contains("branch 30 (send)")
            && note.contains("branch 31 (recv from other)")
    }));
    assert!(crate::issue::diagnostics_are_safe(&diagnostics));
}

#[test]
fn nested_choice_incompatibility_locates_inner_choice_and_conflicting_branches() {
    let outer_span = span_range(0..200);
    let outer_ok_span = span_range(10..40);
    let outer_bad_span = span_range(50..190);
    let inner_choice_span = span_range(60..180);
    let inner_ok_branch_span = span_range(70..95);
    let inner_conflict_a_span = span_range(100..135);
    let inner_conflict_b_span = span_range(140..175);

    let inner_choice = crate::model::tree::Node::Choice {
        decider: ROLE_B,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: communication(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, end()),
                span: inner_ok_branch_span,
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_B,
                body: communication(ROLE_B, ROLE_C, LABEL_B, PAYLOAD_A, end()),
                span: inner_conflict_a_span,
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_C,
                body: end(),
                span: inner_conflict_b_span,
            },
        ],
        span: inner_choice_span,
    };
    let protocol = crate::model::tree::Node::Choice {
        decider: ROLE_A,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: communication(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, end()),
                span: outer_ok_span,
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_B,
                body: communication(ROLE_A, ROLE_B, LABEL_B, PAYLOAD_B, inner_choice),
                span: outer_bad_span,
            },
        ],
        span: outer_span,
    };

    let diagnostics = match crate::admission::admit_source(source(protocol)) {
        Ok(_) => panic!("nested unprojectable choice unexpectedly admitted"),
        Err(crate::admission::Error::Diagnostics(diagnostics)) => diagnostics,
        Err(crate::admission::Error::UnsupportedNode) => panic!("known test node was unsupported"),
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].primary, inner_choice_span);
    assert_eq!(
        diagnostics[0].secondary,
        vec![inner_ok_branch_span, inner_conflict_b_span]
    );
    assert!(diagnostics[0].notes.iter().any(|note| {
        note.contains("role 3")
            && note.contains("decider 2")
            && note.contains("recv from decider")
            && note.contains("(end)")
    }));
    assert!(
        !diagnostics[0]
            .notes
            .iter()
            .any(|note| note.contains("role 1"))
    );
    assert!(crate::issue::diagnostics_are_safe(&diagnostics));
}

#[test]
fn outer_choice_conflict_with_valid_nested_choice_reports_outer_choice() {
    let outer_span = span_range(0..200);
    let outer_branch_a_span = span_range(10..90);
    let outer_branch_b_span = span_range(100..190);
    let valid_inner_span = span_range(20..80);

    let valid_inner = crate::model::tree::Node::Choice {
        decider: ROLE_B,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: communication(ROLE_B, ROLE_C, LABEL_A, PAYLOAD_A, end()),
                span: span_range(25..45),
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_B,
                body: communication(ROLE_B, ROLE_C, LABEL_B, PAYLOAD_B, end()),
                span: span_range(50..75),
            },
        ],
        span: valid_inner_span,
    };
    let protocol = crate::model::tree::Node::Choice {
        decider: ROLE_A,
        branches: vec![
            crate::model::tree::ChoiceBranch {
                label: LABEL_A,
                body: valid_inner,
                span: outer_branch_a_span,
            },
            crate::model::tree::ChoiceBranch {
                label: LABEL_B,
                body: communication(ROLE_C, ROLE_B, LABEL_A, PAYLOAD_A, end()),
                span: outer_branch_b_span,
            },
        ],
        span: outer_span,
    };

    let diagnostics = match crate::admission::admit_source(source(protocol)) {
        Ok(_) => panic!("outer choice conflict unexpectedly admitted"),
        Err(crate::admission::Error::Diagnostics(diagnostics)) => diagnostics,
        Err(crate::admission::Error::UnsupportedNode) => panic!("known test node was unsupported"),
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].primary, outer_span);
    assert_eq!(
        diagnostics[0].secondary,
        vec![outer_branch_a_span, outer_branch_b_span]
    );
    assert!(
        diagnostics[0]
            .notes
            .iter()
            .any(|note| note.contains("decider 1"))
    );
    assert!(crate::issue::diagnostics_are_safe(&diagnostics));
}

#[test]
fn render_safe_diagnostics_truncates_deterministically_and_rejects_unsafe_or_excessive() {
    let mut many = Vec::with_capacity(10);
    for index in 0..10u32 {
        let start = index.saturating_mul(10);
        let end_bytes = start.saturating_add(5);
        many.push(
            crate::issue::Diagnostic::error(
                crate::issue::DiagnosticCode::TrellisProjectionRejected,
                span_range(start..end_bytes),
                "role 2 cannot distinguish choice by decider 1: incompatible offer heads",
            )
            .with_secondary(span_range(start.saturating_add(1)..start.saturating_add(3)))
            .with_hint("review choice communication and branch structure for affected roles"),
        );
    }
    let rendered_first = crate::issue::render_safe_diagnostics(&many);
    let rendered_second = crate::issue::render_safe_diagnostics(&many);
    assert_eq!(rendered_first, rendered_second);
    assert!(rendered_first.len() <= crate::model::bounds::MAX_DIAGNOSTIC_TEXT_BYTES);
    assert!(rendered_first.ends_with("role-choreography:truncated\n"));

    let malformed_span = vec![crate::issue::Diagnostic::error(
        crate::issue::DiagnosticCode::TrellisProjectionRejected,
        crate::model::participants::SourceSpan { start: 10, end: 5 },
        "bad span",
    )];
    assert_eq!(
        crate::issue::render_safe_diagnostics(&malformed_span),
        "role-choreography: unsafe diagnostic rejected"
    );

    let overflow_count = crate::model::bounds::MAX_DIAGNOSTICS.saturating_add(1);
    let mut excessive = Vec::with_capacity(overflow_count);
    for _ in 0..overflow_count {
        excessive.push(crate::issue::Diagnostic::error(
            crate::issue::DiagnosticCode::TrellisProjectionRejected,
            span(),
            "overflow",
        ));
    }
    assert_eq!(
        crate::issue::render_safe_diagnostics(&excessive),
        "role-choreography: unsafe diagnostic rejected"
    );
}

#[test]
fn diagnostic_changes_preserve_canonical_protocol_and_local_set_identities() {
    let annotated_json = include_bytes!(
        "../../../fixtures/role-choreography/positive/annotated-two-branch-choice.json"
    );
    let annotated: crate::preference::model::AnnotatedSource =
        match serde_json::from_slice(annotated_json) {
            Ok(source) => source,
            Err(error) => panic!("parse checked-in annotated fixture: {error}"),
        };
    let base_bytes = match serde_json::to_vec(&annotated.base) {
        Ok(bytes) => bytes,
        Err(error) => panic!("serialize base fixture: {error}"),
    };
    let bundle = match crate::compile_evaluated(&base_bytes, b"envelope") {
        Ok(bundle) => bundle,
        Err(error) => panic!("compile unchanged two-branch base fixture: {error:?}"),
    };
    assert_eq!(
        bundle.local_set().identity.hex,
        "aae9fbd4fefa129f8772f99225d6e0b2a67a311a2d61dd7bb66ee6697ce12194"
    );
}

#[test]
fn malformed_external_role_and_limit_widening_fail() {
    let mut external_source = two_role_source();
    external_source.roles[1] = crate::model::participants::RoleDeclaration {
        id: ROLE_B,
        kind: crate::model::participants::RoleKind::External {
            contract_identity: String::from("ambient-contract"),
            input_schema_identity: exact_identity(b"input"),
            output_schema_identity: exact_identity(b"output"),
        },
        span: span(),
    };
    assert_eq!(
        first_code(crate::admission::admit_source(external_source)),
        crate::issue::DiagnosticCode::InvalidRoleKind
    );

    let mut widened = two_role_source();
    widened.limits.max_roles = OVERSIZED_ROLE_COUNT;
    assert_eq!(
        first_code(crate::admission::admit_source(widened)),
        crate::issue::DiagnosticCode::LimitsMismatch
    );
}

#[test]
fn oversized_role_and_identifier_inputs_fail() {
    let mut invalid_roles_fixture = source(end());
    invalid_roles_fixture.roles.clear();
    for raw in 0..OVERSIZED_ROLE_COUNT {
        let id = match u32::try_from(raw) {
            Ok(id) => crate::model::participants::RoleId(id),
            Err(error) => panic!("role fixture identifier does not fit: {error}"),
        };
        invalid_roles_fixture.roles.push(native(id));
    }
    assert_eq!(
        first_code(crate::admission::admit_source(invalid_roles_fixture)),
        crate::issue::DiagnosticCode::RoleLimitExceeded
    );

    let mut invalid_identifier_fixture = two_role_source();
    invalid_identifier_fixture
        .imports
        .push(crate::model::participants::ImportIdentity {
            logical_id: String::from("x")
                .repeat(crate::model::bounds::MAX_IDENTIFIER_BYTES.saturating_add(1)),
            blake3: blake3::hash(b"oversized-import").to_hex().to_string(),
        });
    assert_eq!(
        first_code(crate::admission::admit_source(invalid_identifier_fixture)),
        crate::issue::DiagnosticCode::InvalidImportIdentity
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
    let diagnostics = match crate::admission::parse_evaluated_json(&bytes) {
        Ok(_) => panic!("unknown field unexpectedly passed"),
        Err(diagnostics) => diagnostics,
    };
    assert_eq!(
        diagnostics[0].code,
        crate::issue::DiagnosticCode::MalformedEvaluatedData
    );

    let mut missing = two_role_source();
    missing.non_claims.pop();
    assert_eq!(
        first_code(crate::admission::admit_source(missing)),
        crate::issue::DiagnosticCode::MissingNonClaim
    );
}

#[test]
fn complete_set_rejects_missing_duplicate_and_reordered_roles() {
    let projected = must_project(two_role_source());
    let expected = [ROLE_A, ROLE_B];

    let mut missing = projected.locals.clone();
    missing.locals.pop();
    assert!(crate::endpoint::validate_complete_set(&expected, &missing).is_err());

    let mut duplicate = projected.locals.clone();
    duplicate.locals[1].role = ROLE_A;
    assert!(crate::endpoint::validate_complete_set(&expected, &duplicate).is_err());

    let mut reordered = projected.locals;
    reordered.locals.swap(0, 1);
    assert!(crate::endpoint::validate_complete_set(&expected, &reordered).is_err());
}

#[test]
fn diagnostics_reject_host_and_secret_leakage() {
    assert!(!crate::issue::safe_diagnostic_text(
        "/home/operator/private"
    ));
    assert!(!crate::issue::safe_diagnostic_text("/etc/operator-private"));
    assert!(!crate::issue::safe_diagnostic_text("C:\\operator-private"));
    assert!(!crate::issue::safe_diagnostic_text("SECRET value"));
    assert!(!crate::issue::safe_diagnostic_text("token value"));
    assert!(!crate::issue::safe_diagnostic_text("PATH=value"));

    let unsafe_diagnostic = crate::issue::Diagnostic {
        code: crate::issue::DiagnosticCode::UnsafeDiagnostic,
        severity: crate::issue::DiagnosticSeverity::Error,
        primary: span(),
        secondary: Vec::new(),
        notes: vec![String::from("/home/operator/private")],
        hints: Vec::new(),
        safe_replacement: None,
    };
    assert!(!crate::issue::diagnostics_are_safe(&[unsafe_diagnostic]));
}
