// r[verify choregraph.role_choreography.preference.profile]
// r[verify choregraph.role_choreography.preference.profile.baseline_default]
// r[verify choregraph.role_choreography.preference.profile.unchanged]
// r[verify choregraph.role_choreography.preference.source]
// r[verify choregraph.role_choreography.preference.source.rejected]
// r[verify choregraph.role_choreography.preference.projection]
// r[verify choregraph.role_choreography.preference.projection.skeleton]
// r[verify choregraph.role_choreography.preference.projection.unaddressable]
// r[verify choregraph.role_choreography.preference.identity]
// r[verify choregraph.role_choreography.preference.identity.stale]
// r[verify choregraph.role_choreography.preference.report]
// r[verify choregraph.role_choreography.preference.report.distinct]
// r[verify choregraph.role_choreography.preference.boundary]
// r[verify choregraph.role_choreography.preference.boundary.non_claims]

type Code = crate::issue::DiagnosticCode;

const ROLE_A: crate::model::participants::RoleId = crate::model::participants::RoleId(1);
const ROLE_B: crate::model::participants::RoleId = crate::model::participants::RoleId(2);
const ROLE_C: crate::model::participants::RoleId = crate::model::participants::RoleId(3);
const ROLE_D: crate::model::participants::RoleId = crate::model::participants::RoleId(4);
const VALUE_A: crate::model::participants::ValueId = crate::model::participants::ValueId(10);
const PAYLOAD_A: crate::model::participants::PayloadTag =
    crate::model::participants::PayloadTag(20);
const PAYLOAD_B: crate::model::participants::PayloadTag =
    crate::model::participants::PayloadTag(21);
const LABEL_A: crate::model::participants::LabelId = crate::model::participants::LabelId(30);
const LABEL_B: crate::model::participants::LabelId = crate::model::participants::LabelId(31);
const LABEL_C: crate::model::participants::LabelId = crate::model::participants::LabelId(32);
const FACTOR_COST: super::model::FactorId = super::model::FactorId(1);
const FACTOR_LATENCY: super::model::FactorId = super::model::FactorId(2);
const FACTOR_UNKNOWN: super::model::FactorId = super::model::FactorId(9);
const WORLD_MARKET: super::model::WorldId = super::model::WorldId(1);
const WORLD_UNKNOWN: super::model::WorldId = super::model::WorldId(9);
const SPAN_END: u32 = 8;
const ENVELOPE: &[u8] = b"source envelope";
const FIXTURE_JSON: &[u8] = include_bytes!(
    "../../../../fixtures/role-choreography/positive/annotated-two-branch-choice.json"
);

fn span() -> crate::model::participants::SourceSpan {
    crate::model::participants::SourceSpan {
        start: 0,
        end: SPAN_END,
    }
}

fn end() -> crate::model::tree::Node {
    crate::model::tree::Node::End { span: span() }
}

fn message(
    from: crate::model::participants::RoleId,
    to: crate::model::participants::RoleId,
    label: crate::model::participants::LabelId,
    next: crate::model::tree::Node,
) -> crate::model::tree::Node {
    crate::model::tree::Node::Communication {
        from,
        to,
        label,
        payload_tag: PAYLOAD_A,
        next: Box::new(next),
        span: span(),
    }
}

fn branch(
    label: crate::model::participants::LabelId,
    body: crate::model::tree::Node,
) -> crate::model::tree::ChoiceBranch {
    crate::model::tree::ChoiceBranch {
        label,
        body,
        span: span(),
    }
}

fn choice(
    decider: crate::model::participants::RoleId,
    branches: Vec<crate::model::tree::ChoiceBranch>,
) -> crate::model::tree::Node {
    crate::model::tree::Node::Choice {
        decider,
        branches,
        span: span(),
    }
}

/// The two-branch protocol of `projectable_choice_includes_uninvolved_role`.
fn two_branch_protocol() -> crate::model::tree::Node {
    let mut second = message(ROLE_A, ROLE_B, LABEL_B, end());
    if let crate::model::tree::Node::Communication { payload_tag, .. } = &mut second {
        *payload_tag = PAYLOAD_B;
    }
    choice(
        ROLE_A,
        vec![
            branch(LABEL_A, message(ROLE_A, ROLE_B, LABEL_A, end())),
            branch(LABEL_B, second),
        ],
    )
}

fn base(protocol: crate::model::tree::Node) -> crate::model::schema::RoleSource {
    let role = |id| crate::model::participants::RoleDeclaration {
        id,
        kind: crate::model::participants::RoleKind::Native,
        span: span(),
    };
    let source = crate::model::schema::RoleSource {
        schema: String::from(crate::model::schema::ROLE_CHOREOGRAPHY_SCHEMA),
        limits: crate::model::bounds::Limits::reviewed(),
        imports: Vec::new(),
        roles: vec![role(ROLE_A), role(ROLE_B), role(ROLE_C)],
        values: vec![crate::model::participants::ValueDeclaration {
            id: VALUE_A,
            payload_tag: PAYLOAD_A,
            initial_owner: ROLE_A,
            span: span(),
        }],
        protocol,
        non_claims: crate::model::schema::REQUIRED_NON_CLAIMS
            .map(String::from)
            .to_vec(),
    };
    assert_eq!(source.roles.len(), 3);
    assert!(source.protocol.span().is_some());
    source
}

fn factor(
    id: super::model::FactorId,
    role: crate::model::participants::RoleId,
    name: &str,
) -> super::model::FactorDeclaration {
    super::model::FactorDeclaration {
        id,
        role,
        name: String::from(name),
        span: span(),
    }
}

fn annotation(
    path: Vec<super::model::PathStep>,
    decider: crate::model::participants::RoleId,
    factors: Vec<super::model::FactorId>,
) -> super::model::ChoiceAnnotation {
    super::model::ChoiceAnnotation {
        path,
        decider,
        factors,
        world: None,
        span: span(),
    }
}

fn annotated_two_branch() -> super::model::AnnotatedSource {
    let mut root = annotation(Vec::new(), ROLE_A, vec![FACTOR_COST, FACTOR_LATENCY]);
    root.world = Some(WORLD_MARKET);
    let source = super::model::AnnotatedSource {
        schema: String::from(super::model::ANNOTATED_SCHEMA),
        base: base(two_branch_protocol()),
        annotations: super::model::AnnotationSet {
            factors: vec![
                factor(FACTOR_COST, ROLE_A, "cost"),
                factor(FACTOR_LATENCY, ROLE_A, "latency"),
            ],
            worlds: vec![super::model::WorldDeclaration {
                id: WORLD_MARKET,
                name: String::from("market"),
                span: span(),
            }],
            choices: vec![root],
        },
        non_claims: super::model::REQUIRED_NON_CLAIMS.map(String::from).to_vec(),
    };
    assert_eq!(source.annotations.choices.len(), 1);
    assert_eq!(source.annotations.factors.len(), 2);
    source
}

fn json<T: serde::Serialize>(value: &T) -> Vec<u8> {
    match serde_json::to_vec(value) {
        Ok(bytes) => bytes,
        Err(error) => panic!("fixture serialization failed: {error}"),
    }
}

fn selected() -> super::ProfileSelection {
    super::ProfileSelection::PreferenceAnnotations
}

fn must_compile(source: &super::model::AnnotatedSource) -> super::Report {
    let report = super::compile_evaluated(&json(source), ENVELOPE, selected());
    assert_eq!(
        report.status(),
        super::ProjectionStatus::Projected,
        "refusal: {:?}",
        report.refusal()
    );
    report
}

fn must_bundle(report: &super::Report) -> &super::linkage::AnnotatedBundle {
    match report.bundle() {
        Some(bundle) => bundle,
        None => panic!("projected report has no bundle"),
    }
}

fn diagnostics(error: Option<&crate::admission::Error>) -> &[crate::issue::Diagnostic] {
    match error {
        Some(crate::admission::Error::Diagnostics(items)) => items,
        Some(crate::admission::Error::UnsupportedNode) => panic!("known node was unsupported"),
        None => panic!("invalid source unexpectedly projected"),
    }
}

fn refusal_code(source: &super::model::AnnotatedSource) -> crate::issue::DiagnosticCode {
    let report = super::compile_evaluated(&json(source), ENVELOPE, selected());
    assert_eq!(report.status(), super::ProjectionStatus::Refused);
    assert_eq!(
        report.game_mapping(),
        super::linkage::GameMapping::NotFormalized
    );
    let items = diagnostics(report.refusal());
    assert!(crate::issue::diagnostics_are_safe(items));
    match items.first() {
        Some(item) => item.code,
        None => panic!("refusal carried no diagnostic"),
    }
}

fn mutated(
    change: impl FnOnce(&mut super::model::AnnotatedSource),
) -> crate::issue::DiagnosticCode {
    let mut source = annotated_two_branch();
    change(&mut source);
    refusal_code(&source)
}

#[test]
fn annotated_two_branch_choice_keeps_the_unannotated_skeleton() {
    let source = annotated_two_branch();
    let baseline = match crate::compile_evaluated(&json(&source.base), ENVELOPE) {
        Ok(bundle) => bundle,
        Err(error) => panic!("baseline failed: {error:?}"),
    };
    let report = must_compile(&source);
    let bundle = must_bundle(&report);
    assert_eq!(bundle.base().global().identity, baseline.global().identity);
    assert_eq!(
        bundle.base().local_set().identity,
        baseline.local_set().identity
    );
    assert_eq!(
        bundle.base().receipt_body().local_artifacts,
        baseline.receipt_body().local_artifacts
    );
    let locals = &bundle.projected().locals.locals;
    assert!(matches!(
        locals[0].protocol,
        crate::endpoint::LocalProtocol::InternalChoice { .. }
    ));
    assert!(matches!(
        locals[1].protocol,
        crate::endpoint::LocalProtocol::Offer { from: ROLE_A, .. }
    ));
    assert_eq!(locals[2].protocol, crate::endpoint::LocalProtocol::End);

    let set = bundle.local_set_body();
    assert_eq!(set.complete_local_set, baseline.local_set().identity);
    assert_eq!(set.locals.len(), 3);
    let decider = &set.locals[0].decisions;
    assert_eq!(decider.len(), 1);
    assert!(decider[0].local_path.is_empty());
    assert_eq!(decider[0].labels, vec![LABEL_A, LABEL_B]);
    assert_eq!(decider[0].factors.len(), 2);
    assert_eq!(
        decider[0].world.as_ref().map(|world| world.id),
        Some(WORLD_MARKET)
    );
    assert!(set.locals[1].decisions.is_empty());
    assert!(set.locals[2].decisions.is_empty());
    for (entry, local) in set.locals.iter().zip(baseline.locals()) {
        assert_eq!(entry.local_artifact, local.artifact.identity);
    }
}

#[test]
fn receipt_binds_annotations_to_the_projection_cohort() {
    let report = must_compile(&annotated_two_branch());
    let bundle = must_bundle(&report);
    let body = bundle.receipt_body();
    assert_eq!(body.base_receipt, bundle.base().receipt().identity);
    assert_eq!(body.complete_local_set, bundle.base().local_set().identity);
    assert_eq!(body.global_artifact, bundle.base().global().identity);
    assert_eq!(body.annotated_local_set, bundle.local_set().identity);
    assert_eq!(body.annotations, bundle.annotations().identity);
    assert_eq!(body.strict_dto, bundle.source().identity);
    assert_eq!(body.profile, super::ANNOTATED_PROFILE);
    let schema_versions = [
        (
            bundle.source(),
            "choregraph.role-choreography-preference-source-preserves.v1",
        ),
        (
            bundle.annotations(),
            "choregraph.role-choreography-preference-annotations-preserves.v1",
        ),
        (
            bundle.local_set(),
            "choregraph.role-choreography-preference-local-set-preserves.v1",
        ),
        (
            bundle.receipt(),
            "choregraph.role-choreography-preference-receipt-preserves.v1",
        ),
    ];
    for (artifact, schema_version) in schema_versions {
        assert_eq!(artifact.schema_version, schema_version);
    }
    assert_eq!(
        body.game_mapping,
        super::linkage::GameMapping::NotFormalized
    );
    assert!(crate::linkage::digest::identity_is_valid(
        &bundle.receipt().identity,
        crate::linkage::digest::IdentityDomain::PreferenceReceipt
    ));
    let packed = &bundle.receipt().packed_bytes;
    assert!(super::linkage::verify_receipt(bundle, body, packed).is_ok());
    assert!(super::linkage::verify_receipt(bundle, body, &packed[1..]).is_err());

    let stale_cases: [fn(&mut super::linkage::ReceiptBody); 7] = [
        |stale| stale.base_receipt = stale.strict_dto.clone(),
        |stale| stale.complete_local_set = stale.annotations.clone(),
        |stale| stale.local_artifacts.swap(0, 1),
        |stale| stale.annotated_local_set = stale.global_artifact.clone(),
        |stale| stale.lowerer = stale.schema_set.clone(),
        |stale| stale.profile = String::from(super::BASELINE_PROFILE),
        |stale| stale.claim = String::from("solver-equilibrium-proven"),
    ];
    for change in stale_cases {
        let mut stale = body.clone();
        change(&mut stale);
        assert!(super::linkage::verify_receipt(bundle, &stale, packed).is_err());
    }
}

#[test]
fn factor_metadata_changes_annotation_identities_but_not_the_skeleton() {
    let original = must_compile(&annotated_two_branch());
    let mut renamed = annotated_two_branch();
    renamed.annotations.factors[0].name = String::from("price");
    let changed = must_compile(&renamed);
    let (left, right) = (must_bundle(&original), must_bundle(&changed));
    assert_eq!(
        left.base().local_set().identity,
        right.base().local_set().identity
    );
    assert_ne!(left.annotations().identity, right.annotations().identity);
    assert_ne!(left.local_set().identity, right.local_set().identity);
    assert_ne!(left.receipt().identity, right.receipt().identity);
    assert!(
        super::linkage::verify_receipt(right, left.receipt_body(), &left.receipt().packed_bytes)
            .is_err()
    );
}

#[test]
fn cooperative_baseline_refuses_annotated_sources() {
    let annotated = json(&annotated_two_branch());
    let default_selection = super::ProfileSelection::default();
    assert_eq!(
        default_selection,
        super::ProfileSelection::CooperativeBaseline
    );

    let baseline = crate::compile_evaluated(&annotated, ENVELOPE);
    let Err(error) = baseline else {
        panic!("baseline admitted an annotated source");
    };
    assert_eq!(
        diagnostics(Some(&error))[0].code,
        crate::issue::DiagnosticCode::PreferenceProfileNotSelected
    );

    let report = super::compile_evaluated(&annotated, ENVELOPE, default_selection);
    let summary = report.summary();
    assert_eq!(summary.status, super::ProjectionStatus::Refused);
    assert_eq!(summary.profile, super::BASELINE_PROFILE);
    assert_eq!(
        summary.diagnostics,
        vec![crate::issue::DiagnosticCode::PreferenceProfileNotSelected]
    );
    assert!(summary.receipt.is_none());

    let direct = super::admission::admit_source(annotated_two_branch(), default_selection);
    assert!(matches!(
        &direct,
        Err(crate::admission::Error::Diagnostics(items))
            if items[0].code == crate::issue::DiagnosticCode::PreferenceProfileNotSelected
    ));
}

#[test]
fn unannotated_v1_sources_keep_strict_baseline_behavior() {
    let source = annotated_two_branch().base;
    assert!(crate::compile_evaluated(&json(&source), ENVELOPE).is_ok());

    let mut value = match serde_json::to_value(&source) {
        Ok(value) => value,
        Err(error) => panic!("serialize fixture: {error}"),
    };
    if let Some(object) = value.as_object_mut() {
        object.insert(String::from("preferences"), serde_json::Value::Null);
    }
    let Err(error) = crate::compile_evaluated(&json(&value), ENVELOPE) else {
        panic!("unknown v1 field unexpectedly admitted");
    };
    assert_eq!(
        diagnostics(Some(&error))[0].code,
        crate::issue::DiagnosticCode::MalformedEvaluatedData
    );

    let report = super::compile_evaluated(&json(&source), ENVELOPE, selected());
    assert_eq!(
        diagnostics(report.refusal())[0].code,
        crate::issue::DiagnosticCode::MalformedEvaluatedData
    );
}

#[test]
fn report_summary_distinguishes_success_refusal_and_game_mapping() {
    let projected = must_compile(&annotated_two_branch()).summary();
    assert_eq!(projected.status, super::ProjectionStatus::Projected);
    assert_eq!(
        projected.game_mapping,
        super::linkage::GameMapping::NotFormalized
    );
    assert!(projected.receipt.is_some());
    assert!(projected.diagnostics.is_empty());
    assert_eq!(projected.profile, super::ANNOTATED_PROFILE);
    assert_eq!(projected.schema, super::REPORT_SCHEMA);

    let rendered = String::from_utf8_lossy(&json(&projected)).into_owned();
    assert!(rendered.contains("\"status\":\"projected\""));
    assert!(rendered.contains("\"game_mapping\":\"not-formalized\""));

    let mut invalid = annotated_two_branch();
    invalid.annotations.choices[0].decider = ROLE_B;
    let refused = super::compile_evaluated(&json(&invalid), ENVELOPE, selected()).summary();
    assert_eq!(refused.status, super::ProjectionStatus::Refused);
    assert_eq!(
        refused.game_mapping,
        super::linkage::GameMapping::NotFormalized
    );
    assert!(refused.receipt.is_none());
    let rendered = String::from_utf8_lossy(&json(&refused)).into_owned();
    assert!(rendered.contains("\"status\":\"refused\""));
    assert!(rendered.contains("stale-choice-decider"));
}

#[test]
fn profile_selectors_are_exact() {
    for selection in [
        super::ProfileSelection::CooperativeBaseline,
        super::ProfileSelection::PreferenceAnnotations,
    ] {
        assert_eq!(
            super::ProfileSelection::from_selector(selection.selector()),
            Some(selection)
        );
    }
    assert_eq!(
        super::ProfileSelection::from_selector(crate::model::schema::ROLE_CHOREOGRAPHY_SCHEMA),
        None
    );
    assert_eq!(super::ProfileSelection::from_selector(""), None);
}

#[test]
fn nickel_exported_fixture_projects_under_the_selected_profile() {
    let report = super::compile_evaluated(FIXTURE_JSON, ENVELOPE, selected());
    assert_eq!(report.status(), super::ProjectionStatus::Projected);
    let bundle = must_bundle(&report);
    let decisions = &bundle.local_set_body().locals[0].decisions;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].factors.len(), 2);

    let baseline = crate::compile_evaluated(FIXTURE_JSON, ENVELOPE);
    assert!(baseline.is_err());
}

#[test]
fn source_level_annotation_errors_are_refused() {
    assert_eq!(
        mutated(|s| s.schema = String::from("choregraph.role-choreography.v1")),
        Code::UnsupportedSchema
    );
    assert_eq!(
        mutated(|s| {
            s.non_claims.pop();
        }),
        Code::PreferenceNonClaimMismatch
    );
    assert_eq!(
        mutated(|s| s.non_claims.push(String::from("solver-proven"))),
        Code::PreferenceNonClaimMismatch
    );
    assert_eq!(
        mutated(|s| s.annotations.choices.clear()),
        Code::EmptyPreferenceAnnotations
    );
    assert_eq!(
        mutated(|s| s.annotations.factors[0].name = String::from("Cost")),
        Code::InvalidPreferenceName
    );
    assert_eq!(
        mutated(|s| s.annotations.factors[0].name = String::from("home-path/x")),
        Code::InvalidPreferenceName
    );
    assert_eq!(
        mutated(|s| s.annotations.worlds[0].name = String::new()),
        Code::InvalidPreferenceName
    );
    assert_eq!(
        mutated(|s| s.annotations.factors[0].role = ROLE_D),
        Code::UndeclaredRole
    );
    assert_eq!(
        mutated(|s| s.annotations.factors[1].id = FACTOR_COST),
        Code::DuplicatePreferenceFactor
    );
    assert_eq!(
        mutated(|s| s.annotations.factors[1].name = String::from("cost")),
        Code::DuplicatePreferenceFactor
    );
    assert_eq!(
        mutated(|s| s.annotations.factors.swap(0, 1)),
        Code::UnsortedPreferenceFactor
    );
    assert_eq!(mutated(|s| s.base.roles.clear()), Code::EmptyRoles);
}

#[test]
fn world_and_limit_errors_are_refused() {
    let duplicate_world = |s: &mut super::model::AnnotatedSource| {
        let copy = s.annotations.worlds[0].clone();
        s.annotations.worlds.push(copy);
    };
    assert_eq!(mutated(duplicate_world), Code::DuplicateWorldVariable);
    let unsorted_world = |s: &mut super::model::AnnotatedSource| {
        let mut later = s.annotations.worlds[0].clone();
        later.id = super::model::WorldId(0);
        later.name = String::from("weather");
        s.annotations.worlds.push(later);
    };
    assert_eq!(mutated(unsorted_world), Code::UnsortedWorldVariable);
    let too_many = |s: &mut super::model::AnnotatedSource| {
        let template = s.annotations.factors[0].clone();
        s.annotations.factors = (0..=super::admission::MAX_FACTORS)
            .map(|index| super::model::FactorDeclaration {
                id: super::model::FactorId(match u32::try_from(index) {
                    Ok(raw) => raw,
                    Err(error) => panic!("factor fixture identifier does not fit: {error}"),
                }),
                ..template.clone()
            })
            .collect();
    };
    assert_eq!(mutated(too_many), Code::PreferenceLimitExceeded);
    let long_path = |s: &mut super::model::AnnotatedSource| {
        s.annotations.choices[0].path =
            vec![super::model::PathStep::Next; super::admission::MAX_PATH_STEPS + 1];
    };
    assert_eq!(mutated(long_path), Code::PreferenceLimitExceeded);
    let invalid_span = |s: &mut super::model::AnnotatedSource| {
        s.annotations.choices[0].span = crate::model::participants::SourceSpan { start: 2, end: 1 };
    };
    assert_eq!(mutated(invalid_span), Code::InvalidSpan);
}

#[test]
fn choice_annotation_errors_are_refused() {
    let to_end = vec![
        super::model::PathStep::Branch { label: LABEL_A },
        super::model::PathStep::Next,
    ];
    assert_eq!(
        mutated(|s| s.annotations.choices[0].path = vec![super::model::PathStep::Next]),
        Code::InvalidPreferencePath
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].path =
            vec![super::model::PathStep::Branch { label: LABEL_C }]),
        Code::InvalidPreferencePath
    );
    assert_eq!(
        mutated(move |s| s.annotations.choices[0].path = to_end),
        Code::PreferenceTargetNotChoice
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].decider = ROLE_B),
        Code::StaleChoiceDecider
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].factors.clear()),
        Code::EmptyChoiceFactors
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].factors.reverse()),
        Code::UnsortedChoiceFactors
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].factors.push(FACTOR_LATENCY)),
        Code::UnsortedChoiceFactors
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].factors.push(FACTOR_UNKNOWN)),
        Code::UnknownPreferenceFactor
    );
    assert_eq!(
        mutated(|s| s.annotations.factors[1].role = ROLE_B),
        Code::PreferenceFactorRoleMismatch
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].world = Some(WORLD_UNKNOWN)),
        Code::UnknownWorldVariable
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].world = None),
        Code::UnusedPreferenceDeclaration
    );
    assert_eq!(
        mutated(|s| s.annotations.choices[0].factors.truncate(1)),
        Code::UnusedPreferenceDeclaration
    );
    let duplicate = |s: &mut super::model::AnnotatedSource| {
        let copy = s.annotations.choices[0].clone();
        s.annotations.choices.push(copy);
    };
    assert_eq!(mutated(duplicate), Code::DuplicateChoiceAnnotation);
}

#[test]
fn unknown_annotation_fields_are_malformed() {
    let mut value = match serde_json::to_value(annotated_two_branch()) {
        Ok(value) => value,
        Err(error) => panic!("serialize fixture: {error}"),
    };
    if let Some(choice) = value
        .pointer_mut("/annotations/choices/0")
        .and_then(serde_json::Value::as_object_mut)
    {
        choice.insert(String::from("utility"), serde_json::Value::from(1));
    }
    let report = super::compile_evaluated(&json(&value), ENVELOPE, selected());
    assert_eq!(
        diagnostics(report.refusal())[0].code,
        crate::issue::DiagnosticCode::MalformedEvaluatedData
    );
}

/// Outer choice by A whose first branch sends to B and nests a choice by
/// `inner_decider` between A and B.
fn nested_protocol(inner_decider: crate::model::participants::RoleId) -> crate::model::tree::Node {
    let peer = if inner_decider == ROLE_A {
        ROLE_B
    } else {
        ROLE_A
    };
    assert_ne!(peer, inner_decider);
    assert_ne!(peer, ROLE_C);
    let inner = choice(
        inner_decider,
        vec![
            branch(LABEL_B, message(inner_decider, peer, LABEL_B, end())),
            branch(LABEL_C, message(inner_decider, peer, LABEL_C, end())),
        ],
    );
    choice(
        ROLE_A,
        vec![
            branch(LABEL_A, message(ROLE_A, ROLE_B, LABEL_A, inner)),
            branch(LABEL_B, message(ROLE_A, ROLE_B, LABEL_B, end())),
        ],
    )
}

fn nested_source(
    protocol: crate::model::tree::Node,
    choices: Vec<super::model::ChoiceAnnotation>,
    role: crate::model::participants::RoleId,
) -> super::model::AnnotatedSource {
    let mut source = annotated_two_branch();
    source.base = base(protocol);
    source.annotations.factors = vec![factor(FACTOR_COST, role, "cost")];
    source.annotations.worlds.clear();
    source.annotations.choices = choices;
    source
}

fn inner_path() -> Vec<super::model::PathStep> {
    vec![
        super::model::PathStep::Branch { label: LABEL_A },
        super::model::PathStep::Next,
    ]
}

#[test]
fn nested_decider_choices_have_exact_local_addresses() {
    let choices = vec![
        annotation(Vec::new(), ROLE_A, vec![FACTOR_COST]),
        annotation(inner_path(), ROLE_A, vec![FACTOR_COST]),
    ];
    let report = must_compile(&nested_source(nested_protocol(ROLE_A), choices, ROLE_A));
    let decisions = &must_bundle(&report).local_set_body().locals[0].decisions;
    assert_eq!(decisions.len(), 2);
    assert!(decisions[0].local_path.is_empty());
    assert_eq!(decisions[1].local_path, inner_path());
    assert_eq!(decisions[1].labels, vec![LABEL_B, LABEL_C]);

    let mut unsorted = vec![
        annotation(inner_path(), ROLE_A, vec![FACTOR_COST]),
        annotation(Vec::new(), ROLE_A, vec![FACTOR_COST]),
    ];
    let code = refusal_code(&nested_source(
        nested_protocol(ROLE_A),
        unsorted.clone(),
        ROLE_A,
    ));
    assert_eq!(code, crate::issue::DiagnosticCode::UnsortedChoiceAnnotation);
    unsorted.truncate(1);
    assert!(
        must_bundle(&must_compile(&nested_source(
            nested_protocol(ROLE_A),
            unsorted,
            ROLE_A
        )))
        .local_set_body()
        .locals[1]
            .decisions
            .is_empty()
    );
}

#[test]
fn peer_decider_under_an_offer_is_addressed_through_the_offer() {
    let choices = vec![annotation(inner_path(), ROLE_B, vec![FACTOR_COST])];
    let report = must_compile(&nested_source(nested_protocol(ROLE_B), choices, ROLE_B));
    let set = must_bundle(&report).local_set_body();
    assert!(set.locals[0].decisions.is_empty());
    let decisions = &set.locals[1].decisions;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].local_path, inner_path());
    assert_eq!(decisions[0].factors[0].role, ROLE_B);
}

#[test]
fn merged_foreign_choice_is_not_locally_addressable() {
    let inner = || {
        choice(
            ROLE_B,
            vec![
                branch(LABEL_B, message(ROLE_B, ROLE_C, LABEL_B, end())),
                branch(LABEL_C, message(ROLE_B, ROLE_C, LABEL_C, end())),
            ],
        )
    };
    let protocol = choice(
        ROLE_A,
        vec![
            branch(LABEL_A, message(ROLE_A, ROLE_C, LABEL_A, inner())),
            branch(LABEL_B, message(ROLE_A, ROLE_C, LABEL_B, inner())),
        ],
    );
    let choices = vec![annotation(inner_path(), ROLE_B, vec![FACTOR_COST])];
    let code = refusal_code(&nested_source(protocol, choices, ROLE_B));
    assert_eq!(
        code,
        crate::issue::DiagnosticCode::PreferencePathNotLocallyAddressable
    );
}

#[test]
fn annotated_local_set_rejects_incomplete_reordered_and_stale_entries() {
    let report = must_compile(&annotated_two_branch());
    let bundle = must_bundle(&report);
    let check = |change: fn(&mut super::projection::AnnotatedLocalSet)| {
        let mut set = bundle.local_set_body().clone();
        change(&mut set);
        match super::projection::validate_set(bundle.base(), &set) {
            Ok(()) => panic!("invalid annotated set unexpectedly passed"),
            Err(items) => items[0].code,
        }
    };
    assert!(super::projection::validate_set(bundle.base(), bundle.local_set_body()).is_ok());
    assert!(super::linkage::verify_local_set(bundle, bundle.local_set_body()).is_ok());
    let mut renamed = bundle.local_set_body().clone();
    renamed.locals[0].decisions[0].factors[0].name = String::from("price");
    assert!(super::projection::validate_set(bundle.base(), &renamed).is_ok());
    let Err(items) = super::linkage::verify_local_set(bundle, &renamed) else {
        panic!("changed annotated local set unexpectedly verified");
    };
    assert_eq!(items[0].code, Code::IdentityMismatch);
    let mut dropped = bundle.local_set_body().clone();
    dropped.locals.pop();
    assert!(super::linkage::verify_local_set(bundle, &dropped).is_err());
    assert_eq!(
        check(|set| {
            set.locals.pop();
        }),
        Code::IncompleteLocalSet
    );
    assert_eq!(check(|set| set.locals.swap(0, 1)), Code::LocalOrderMismatch);
    assert_eq!(
        check(|set| set.locals[1].role = set.locals[0].role),
        Code::DuplicateLocalRole
    );
    assert_eq!(
        check(|set| set.locals[0].local_artifact = set.complete_local_set.clone()),
        Code::IdentityMismatch
    );
    assert_eq!(
        check(|set| set.complete_local_set = set.locals[0].local_artifact.clone()),
        Code::IdentityMismatch
    );
    let moved = |set: &mut super::projection::AnnotatedLocalSet| {
        let decisions = std::mem::take(&mut set.locals[0].decisions);
        set.locals[1].decisions = decisions;
    };
    assert_eq!(check(moved), Code::PreferenceFactorRoleMismatch);
    let duplicated = |set: &mut super::projection::AnnotatedLocalSet| {
        let copy = set.locals[0].decisions[0].clone();
        set.locals[0].decisions.push(copy);
    };
    assert_eq!(check(duplicated), Code::UnsortedChoiceAnnotation);
}
