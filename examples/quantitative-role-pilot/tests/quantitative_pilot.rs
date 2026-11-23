//! End-to-end integration tests for the opt-in quantitative role-protocol
//! model checking pilot over pinned PRISM 4.9.

use choregraph_choreography::endpoint::{
    LocalProtocol, LocalProtocolArtifactBody, LocalProtocolSetBody,
};
use choregraph_choreography::model::bounds::Limits;
use choregraph_choreography::model::participants::{
    LabelId, PayloadTag, RoleDeclaration, RoleId, RoleKind, SourceSpan, ValueDeclaration, ValueId,
};
use choregraph_choreography::model::schema::{
    REQUIRED_NON_CLAIMS as V1_NON_CLAIMS, ROLE_CHOREOGRAPHY_SCHEMA, RoleSource,
};
use choregraph_choreography::model::tree::{ChoiceBranch, Node};
use choregraph_quantitative_role_pilot::mdp::{
    enumerate_source_mdp, export_prism_mdp, export_prism_props, verify_prism_export_conformance,
};
use choregraph_quantitative_role_pilot::model::{
    AnalysisError, AnalysisManifest, BranchProbability, CLAIM_SCOPE, ChoiceProbabilityAnnotation,
    OwnershipAbstraction, ProbabilityProvenance, ProbabilitySidecar, PropertySpec,
    QUANTITATIVE_PILOT_PROFILE, QUANTITATIVE_PILOT_SCHEMA, REQUIRED_NON_CLAIMS, RationalProb,
    SchedulerAssumption, StateSpaceBounds, ToolConfiguration,
};
use choregraph_quantitative_role_pilot::prism::{
    parse_prism_results, replay_retained_subject, run_prism, verify_receipt,
};
use choregraph_quantitative_role_pilot::{analyze_projected_local_set, analyze_role_source};

const ROLE_A: RoleId = RoleId(1);
const ROLE_B: RoleId = RoleId(2);
const ROLE_C: RoleId = RoleId(3);
const LABEL_A: LabelId = LabelId(30);
const LABEL_B: LabelId = LabelId(31);
const LABEL_C: LabelId = LabelId(32);
const LABEL_D: LabelId = LabelId(33);
const PAYLOAD_A: PayloadTag = PayloadTag(20);
const PAYLOAD_B: PayloadTag = PayloadTag(21);
const VALUE_A: ValueId = ValueId(10);

fn span() -> SourceSpan {
    SourceSpan { start: 0, end: 8 }
}

fn native_role(id: RoleId) -> RoleDeclaration {
    RoleDeclaration {
        id,
        kind: RoleKind::Native,
        span: span(),
    }
}

fn end_node() -> Node {
    Node::End { span: span() }
}

fn comm(from: RoleId, to: RoleId, label: LabelId, payload_tag: PayloadTag, next: Node) -> Node {
    Node::Communication {
        from,
        to,
        label,
        payload_tag,
        next: Box::new(next),
        span: span(),
    }
}

fn two_branch_choice_source() -> RoleSource {
    let protocol = Node::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_A,
                body: comm(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, end_node()),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_B,
                body: comm(ROLE_A, ROLE_B, LABEL_B, PAYLOAD_B, end_node()),
                span: span(),
            },
        ],
        span: span(),
    };
    RoleSource {
        schema: String::from(ROLE_CHOREOGRAPHY_SCHEMA),
        limits: Limits::reviewed(),
        imports: Vec::new(),
        roles: vec![
            native_role(ROLE_A),
            native_role(ROLE_B),
            native_role(ROLE_C),
        ],
        values: Vec::new(),
        protocol,
        non_claims: V1_NON_CLAIMS.map(String::from).to_vec(),
    }
}

fn manifest(roles: Vec<RoleId>, properties: Vec<PropertySpec>) -> AnalysisManifest {
    AnalysisManifest {
        schema: String::from(QUANTITATIVE_PILOT_SCHEMA),
        profile: String::from(QUANTITATIVE_PILOT_PROFILE),
        semantics: String::from("synchronous-rendezvous-mdp-v1"),
        requested_roles: roles,
        ownership_abstraction: OwnershipAbstraction::RejectOwnershipTransfer,
        scheduler_assumption: SchedulerAssumption::UnrestrictedAdversarial,
        bounds: StateSpaceBounds::default(),
        properties,
        tool: ToolConfiguration::pinned_prism("prism"),
    }
}

fn sidecar_70_30() -> Result<ProbabilitySidecar, AnalysisError> {
    Ok(ProbabilitySidecar {
        schema: String::from(QUANTITATIVE_PILOT_SCHEMA),
        numeric_representation: String::from("exact-rational-u64"),
        choices: vec![ChoiceProbabilityAnnotation {
            decider: ROLE_A,
            choice_ordinal: 0,
            provenance: ProbabilityProvenance::Hypothesis,
            branches: vec![
                BranchProbability {
                    label: LABEL_A,
                    probability: RationalProb::new(7, 10)?,
                },
                BranchProbability {
                    label: LABEL_B,
                    probability: RationalProb::new(3, 10)?,
                },
            ],
        }],
    })
}

fn approx_eq(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < 1e-6
}

#[test]
fn trellis_projected_two_branch_choice_checks_with_prism_and_replays_receipt() -> Result<(), String>
{
    let source = two_branch_choice_source();
    let sidecar = sidecar_70_30().map_err(|err| format!("sidecar: {err}"))?;
    let properties = vec![
        PropertySpec::BoundedCompletionMin { max_steps: 2 },
        PropertySpec::BoundedCompletionMax { max_steps: 1 },
        PropertySpec::UnboundedCompletionMin,
        PropertySpec::UnboundedCompletionMax,
        PropertySpec::DeadlockReachabilityMax,
        PropertySpec::BranchLabelReachabilityMin {
            decider: ROLE_A,
            label: LABEL_A,
        },
        PropertySpec::BranchLabelReachabilityMax {
            decider: ROLE_A,
            label: LABEL_B,
        },
    ];
    let analysis_manifest = manifest(vec![ROLE_A, ROLE_B, ROLE_C], properties.clone());

    let (report, retained, _mdp) =
        analyze_role_source(source.clone(), &sidecar, &analysis_manifest)
            .map_err(|err| format!("analyze_role_source: {err}"))?;

    assert!(report.conformance_verified);
    assert_eq!(report.claim_scope, CLAIM_SCOPE);
    assert!(report.has_probabilistic_choices);
    assert!(!report.has_nondeterministic_choices);
    assert_eq!(
        report.provenance_classes,
        vec![ProbabilityProvenance::Hypothesis]
    );
    assert_eq!(report.inventory.deadlock_states, 0);
    assert!(report.inventory.terminal_end_states >= 1);
    assert_eq!(report.non_claims.len(), REQUIRED_NON_CLAIMS.len());

    // Bounded completion: takes 2 steps (probabilistic choice -> comm -> End).
    assert!(approx_eq(report.results[0].probability, 1.0));
    assert!(report.results[0].is_bounded);
    // In <= 1 step, End is not yet reached.
    assert!(approx_eq(report.results[1].probability, 0.0));
    // Unbounded completion min & max both 1.0; deadlock reachability is 0.0.
    assert!(approx_eq(report.results[2].probability, 1.0));
    assert!(!report.results[2].is_bounded);
    assert!(approx_eq(report.results[3].probability, 1.0));
    assert!(approx_eq(report.results[4].probability, 0.0));
    // Branch probabilities match 7/10 and 3/10.
    assert!(approx_eq(report.results[5].probability, 0.7));
    assert!(approx_eq(report.results[6].probability, 0.3));

    // Probability-change control: 25/100 and 75/100 changes results & invalidates receipt.
    let changed_sidecar = ProbabilitySidecar {
        schema: String::from(QUANTITATIVE_PILOT_SCHEMA),
        numeric_representation: String::from("exact-rational-u64"),
        choices: vec![ChoiceProbabilityAnnotation {
            decider: ROLE_A,
            choice_ordinal: 0,
            provenance: ProbabilityProvenance::ExternallyEstimated,
            branches: vec![
                BranchProbability {
                    label: LABEL_A,
                    probability: RationalProb::new(1, 4).map_err(|e| format!("{e}"))?,
                },
                BranchProbability {
                    label: LABEL_B,
                    probability: RationalProb::new(3, 4).map_err(|e| format!("{e}"))?,
                },
            ],
        }],
    };
    let (changed_report, _, _) =
        analyze_role_source(source.clone(), &changed_sidecar, &analysis_manifest)
            .map_err(|err| format!("changed probability run: {err}"))?;
    assert!(approx_eq(changed_report.results[5].probability, 0.25));
    assert!(approx_eq(changed_report.results[6].probability, 0.75));
    assert_ne!(
        report.receipt.receipt_blake3,
        changed_report.receipt.receipt_blake3
    );

    // Replay consumes retained subject & properties and reproduces exact results.
    let replayed = replay_retained_subject(&retained, &properties, &analysis_manifest)
        .map_err(|err| format!("replay retained: {err}"))?;
    assert_eq!(replayed, report.results);

    // Receipt verification catches drift in source/endpoints, annotations, properties,
    // scheduler assumptions, tool config, and exported model bytes.
    let source_bytes = serde_json::to_vec(&source).map_err(|e| format!("{e}"))?;
    let admitted = choregraph_choreography::admission::admit_source(source)
        .map_err(|e| format!("admit: {e:?}"))?;
    let projected =
        choregraph_choreography::endpoint::project_all(&admitted).map_err(|e| format!("{e:?}"))?;

    verify_receipt(
        &report.receipt,
        &source_bytes,
        &projected.locals,
        &sidecar,
        &analysis_manifest,
        &retained.prism_model,
    )
    .map_err(|e| format!("valid receipt rejected: {e}"))?;

    assert!(matches!(
        verify_receipt(
            &report.receipt,
            b"mutated-source",
            &projected.locals,
            &sidecar,
            &analysis_manifest,
            &retained.prism_model,
        ),
        Err(AnalysisError::ReceiptMismatch(_))
    ));
    assert!(matches!(
        verify_receipt(
            &report.receipt,
            &source_bytes,
            &projected.locals,
            &changed_sidecar,
            &analysis_manifest,
            &retained.prism_model,
        ),
        Err(AnalysisError::ReceiptMismatch(_))
    ));
    let mut changed_tool_manifest = analysis_manifest.clone();
    changed_tool_manifest
        .tool
        .extra_args
        .push(String::from("-javamaxmem 512m"));
    assert!(matches!(
        verify_receipt(
            &report.receipt,
            &source_bytes,
            &projected.locals,
            &sidecar,
            &changed_tool_manifest,
            &retained.prism_model,
        ),
        Err(AnalysisError::ReceiptMismatch(_))
    ));
    let mut tampered_retained = retained.clone();
    tampered_retained.prism_model.push_str("\n// tampered\n");
    assert!(matches!(
        replay_retained_subject(&tampered_retained, &properties, &analysis_manifest),
        Err(AnalysisError::ReceiptMismatch(_))
    ));

    Ok(())
}

#[test]
fn conformance_check_catches_swapped_labels_omitted_branch_altered_prob_and_end_as_deadlock()
-> Result<(), String> {
    let source = two_branch_choice_source();
    let sidecar = sidecar_70_30().map_err(|e| format!("{e}"))?;
    let admitted = choregraph_choreography::admission::admit_source(source)
        .map_err(|e| format!("admit: {e:?}"))?;
    let projected =
        choregraph_choreography::endpoint::project_all(&admitted).map_err(|e| format!("{e:?}"))?;
    let mdp = enumerate_source_mdp(
        &[ROLE_A, ROLE_B, ROLE_C],
        &projected.locals,
        &sidecar,
        OwnershipAbstraction::RejectOwnershipTransfer,
        StateSpaceBounds::default(),
    )
    .map_err(|e| format!("enumerate: {e}"))?;

    let valid_prism = export_prism_mdp(&mdp);
    let props = export_prism_props(&[PropertySpec::UnboundedCompletionMin]);
    let tool = ToolConfiguration::pinned_prism("prism");

    // 1. Swapped participant/label mutation in exported PRISM model.
    let swapped = valid_prism.replace("comm_1_2_30_20", "comm_2_1_30_20");
    assert_ne!(swapped, valid_prism);
    let run_swapped = run_prism(&tool, &swapped, &props, 1, 15_000).map_err(|e| format!("{e}"))?;
    assert!(matches!(
        verify_prism_export_conformance(
            &mdp,
            &run_swapped.states_txt,
            &run_swapped.trans_txt,
            &run_swapped.labels_txt,
        ),
        Err(AnalysisError::ConformanceMismatch(_))
    ));

    // 2. Altered probability mutation (7/10 + 3/10 -> 1/2 + 1/2).
    let altered_prob = valid_prism
        .replace("7/10 :", "1/2 :")
        .replace("3/10 :", "1/2 :");
    assert_ne!(altered_prob, valid_prism);
    let run_prob =
        run_prism(&tool, &altered_prob, &props, 1, 15_000).map_err(|e| format!("{e}"))?;
    assert!(matches!(
        verify_prism_export_conformance(
            &mdp,
            &run_prob.states_txt,
            &run_prob.trans_txt,
            &run_prob.labels_txt,
        ),
        Err(AnalysisError::ConformanceMismatch(_))
    ));

    // 3. Omitted branch mutation (drop second branch outcome -> 1 : first target).
    let mut omitted = valid_prism.clone();
    let line_to_replace = valid_prism
        .lines()
        .find(|line| line.contains("[pchoice_1_0]"))
        .ok_or_else(|| String::from("missing pchoice_1_0"))?
        .to_string();
    omitted = omitted.replace(&line_to_replace, "  [pchoice_1_0] s=0 -> 1 : (s'=1);");
    let run_omitted = run_prism(&tool, &omitted, &props, 1, 15_000).map_err(|e| format!("{e}"))?;
    assert!(matches!(
        verify_prism_export_conformance(
            &mdp,
            &run_omitted.states_txt,
            &run_omitted.trans_txt,
            &run_omitted.labels_txt,
        ),
        Err(AnalysisError::ConformanceMismatch(_))
    ));

    // 4. Buggy exporter that omits the `terminated` self-loop, causing PRISM to
    //    label terminal End state as a deadlock.
    let no_end_loop = valid_prism
        .lines()
        .filter(|line| !line.contains("[terminated]"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let run_deadlock_bug =
        run_prism(&tool, &no_end_loop, &props, 1, 15_000).map_err(|e| format!("{e}"))?;
    assert!(matches!(
        verify_prism_export_conformance(
            &mdp,
            &run_deadlock_bug.states_txt,
            &run_deadlock_bug.trans_txt,
            &run_deadlock_bug.labels_txt,
        ),
        Err(AnalysisError::ConformanceMismatch(msg)) if msg.contains("deadlock")
    ));

    Ok(())
}

#[test]
fn nondeterministic_scheduling_and_mismatched_deadlock_controls_distinguish_properties()
-> Result<(), String> {
    // 1. Mixed nondeterministic (outer unannotated choice) + probabilistic (inner choice):
    //    Outer choice by ROLE_A is unannotated -> nondeterministic!
    //      Branch A -> End immediately (prob 0 of reaching LABEL_C)
    //      Branch B -> nested probabilistic choice (4/10 LABEL_C, 6/10 LABEL_D)
    //    Therefore Pmin[F branch_1_32] == 0.0 while Pmax[F branch_1_32] == 0.4!
    let inner_choice = Node::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_C,
                body: comm(ROLE_A, ROLE_B, LABEL_C, PAYLOAD_A, end_node()),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_D,
                body: comm(ROLE_A, ROLE_B, LABEL_D, PAYLOAD_B, end_node()),
                span: span(),
            },
        ],
        span: span(),
    };
    let protocol = Node::Choice {
        decider: ROLE_A,
        branches: vec![
            ChoiceBranch {
                label: LABEL_A,
                body: comm(ROLE_A, ROLE_B, LABEL_A, PAYLOAD_A, end_node()),
                span: span(),
            },
            ChoiceBranch {
                label: LABEL_B,
                body: comm(ROLE_A, ROLE_B, LABEL_B, PAYLOAD_B, inner_choice),
                span: span(),
            },
        ],
        span: span(),
    };
    let source = RoleSource {
        schema: String::from(ROLE_CHOREOGRAPHY_SCHEMA),
        limits: Limits::reviewed(),
        imports: Vec::new(),
        roles: vec![native_role(ROLE_A), native_role(ROLE_B)],
        values: Vec::new(),
        protocol,
        non_claims: V1_NON_CLAIMS.map(String::from).to_vec(),
    };
    // Annotate ONLY choice_ordinal 1 (the inner choice); choice_ordinal 0 stays nondeterministic.
    let sidecar = ProbabilitySidecar {
        schema: String::from(QUANTITATIVE_PILOT_SCHEMA),
        numeric_representation: String::from("exact-rational-u64"),
        choices: vec![ChoiceProbabilityAnnotation {
            decider: ROLE_A,
            choice_ordinal: 1,
            provenance: ProbabilityProvenance::Hypothesis,
            branches: vec![
                BranchProbability {
                    label: LABEL_C,
                    probability: RationalProb::new(2, 5).map_err(|e| format!("{e}"))?,
                },
                BranchProbability {
                    label: LABEL_D,
                    probability: RationalProb::new(3, 5).map_err(|e| format!("{e}"))?,
                },
            ],
        }],
    };
    let analysis_manifest = manifest(
        vec![ROLE_A, ROLE_B],
        vec![
            PropertySpec::BranchLabelReachabilityMin {
                decider: ROLE_A,
                label: LABEL_C,
            },
            PropertySpec::BranchLabelReachabilityMax {
                decider: ROLE_A,
                label: LABEL_C,
            },
            PropertySpec::UnboundedCompletionMin,
            PropertySpec::DeadlockReachabilityMax,
        ],
    );
    let (report, _, _) = analyze_role_source(source, &sidecar, &analysis_manifest)
        .map_err(|e| format!("nondet control failed: {e}"))?;
    assert!(report.has_nondeterministic_choices);
    assert!(report.has_probabilistic_choices);
    assert!(approx_eq(report.results[0].probability, 0.0));
    assert!(approx_eq(report.results[1].probability, 0.4));
    assert!(approx_eq(report.results[2].probability, 1.0));
    assert!(approx_eq(report.results[3].probability, 0.0));

    // 2. Deliberate mismatched communication / deadlock control (A sends label 30,
    //    B expects label 31 -> immediate deadlock, never reaches all_ended).
    let mismatched_locals = LocalProtocolSetBody {
        locals: vec![
            LocalProtocolArtifactBody {
                role: ROLE_A,
                role_kind: RoleKind::Native,
                protocol: LocalProtocol::Send {
                    peer: ROLE_B,
                    label: LABEL_A,
                    payload_tag: PAYLOAD_A,
                    next: Box::new(LocalProtocol::End),
                },
            },
            LocalProtocolArtifactBody {
                role: ROLE_B,
                role_kind: RoleKind::Native,
                protocol: LocalProtocol::Receive {
                    peer: ROLE_A,
                    label: LABEL_B,
                    payload_tag: PAYLOAD_A,
                    next: Box::new(LocalProtocol::End),
                },
            },
        ],
    };
    let deadlock_manifest = manifest(
        vec![ROLE_A, ROLE_B],
        vec![
            PropertySpec::DeadlockReachabilityMax,
            PropertySpec::UnboundedCompletionMax,
        ],
    );
    let (deadlock_report, _, _) = analyze_projected_local_set(
        b"synthetic-mismatched-local-set",
        &mismatched_locals,
        &ProbabilitySidecar::empty(),
        &deadlock_manifest,
    )
    .map_err(|e| format!("deadlock control failed: {e}"))?;
    assert_eq!(deadlock_report.inventory.deadlock_states, 1);
    assert!(approx_eq(deadlock_report.results[0].probability, 1.0));
    assert!(approx_eq(deadlock_report.results[1].probability, 0.0));

    Ok(())
}

#[test]
fn separate_recursive_conformance_fixture_distinguishes_deadlock_freedom_from_progress()
-> Result<(), String> {
    // Explicitly separate checker-conformance fixture (NOT part of Choregraph's
    // production finite RoleSource grammar): a 2-state deadlock-free ping-pong
    // loop that never reaches `all_ended`.
    let recursive_ping_pong = r#"
mdp
module recursive_ping_pong
  s : [0..2] init 0;
  [ping] s=0 -> 1 : (s'=1);
  [pong] s=1 -> 1 : (s'=0);
  [terminated] s=2 -> 1 : (s'=2);
endmodule
label "all_ended" = s=2;
label "protocol_deadlock" = false;
"#;
    let props = "Pmax=? [ F \"protocol_deadlock\" ]\nPmax=? [ F \"all_ended\" ]\nPmin=? [ F<=10 \"all_ended\" ]\n";
    let tool = ToolConfiguration::pinned_prism("prism");
    let run = run_prism(&tool, recursive_ping_pong, props, 3, 15_000)
        .map_err(|e| format!("recursive conformance fixture failed: {e}"))?;
    // Deadlock probability is 0.0 (deadlock-free), yet completion probability is 0.0 (non-progressing).
    assert!(approx_eq(run.probabilities[0], 0.0));
    assert!(approx_eq(run.probabilities[1], 0.0));
    assert!(approx_eq(run.probabilities[2], 0.0));
    Ok(())
}

#[test]
fn fail_closed_on_invalid_probabilities_ownership_bounds_timeouts_and_tool_errors()
-> Result<(), String> {
    let source = two_branch_choice_source();
    let valid_sidecar = sidecar_70_30().map_err(|e| format!("{e}"))?;

    // 1. Default-off refusal without explicit profile selection.
    let mut unselected = manifest(
        vec![ROLE_A, ROLE_B, ROLE_C],
        vec![PropertySpec::UnboundedCompletionMin],
    );
    unselected.profile =
        String::from("choregraph.role-choreography.profile.cooperative-baseline.v1");
    assert!(matches!(
        analyze_role_source(source.clone(), &valid_sidecar, &unselected),
        Err(AnalysisError::ProfileNotSelected)
    ));

    // 2. Invalid probabilities (zero denom, >1, sum != 1).
    assert!(matches!(
        RationalProb::new(1, 0),
        Err(AnalysisError::InvalidProbability(_))
    ));
    assert!(matches!(
        RationalProb::new(3, 2),
        Err(AnalysisError::InvalidProbability(_))
    ));
    let unnormalized = ProbabilitySidecar {
        schema: String::from(QUANTITATIVE_PILOT_SCHEMA),
        numeric_representation: String::from("exact-rational-u64"),
        choices: vec![ChoiceProbabilityAnnotation {
            decider: ROLE_A,
            choice_ordinal: 0,
            provenance: ProbabilityProvenance::Hypothesis,
            branches: vec![
                BranchProbability {
                    label: LABEL_A,
                    probability: RationalProb::new(4, 10).map_err(|e| format!("{e}"))?,
                },
                BranchProbability {
                    label: LABEL_B,
                    probability: RationalProb::new(4, 10).map_err(|e| format!("{e}"))?,
                },
            ],
        }],
    };
    let valid_manifest = manifest(
        vec![ROLE_A, ROLE_B, ROLE_C],
        vec![PropertySpec::UnboundedCompletionMin],
    );
    assert!(matches!(
        analyze_role_source(source.clone(), &unnormalized, &valid_manifest),
        Err(AnalysisError::InvalidProbability(_))
    ));

    // 3. Incomplete endpoint set rejected.
    let incomplete_manifest = manifest(
        vec![ROLE_A, ROLE_B],
        vec![PropertySpec::UnboundedCompletionMin],
    );
    assert!(matches!(
        analyze_role_source(source.clone(), &valid_sidecar, &incomplete_manifest),
        Err(AnalysisError::IncompleteEndpointSet(_))
    ));

    // 4. Unsupported construct (ownership transfer) rejected rather than dropped.
    let transfer_source = RoleSource {
        schema: String::from(ROLE_CHOREOGRAPHY_SCHEMA),
        limits: Limits::reviewed(),
        imports: Vec::new(),
        roles: vec![native_role(ROLE_A), native_role(ROLE_B)],
        values: vec![ValueDeclaration {
            id: VALUE_A,
            payload_tag: PAYLOAD_A,
            initial_owner: ROLE_A,
            span: span(),
        }],
        protocol: Node::Transfer {
            from: ROLE_A,
            to: ROLE_B,
            label: LABEL_A,
            payload_tag: PAYLOAD_A,
            value: VALUE_A,
            next: Box::new(end_node()),
            span: span(),
        },
        non_claims: V1_NON_CLAIMS.map(String::from).to_vec(),
    };
    let transfer_manifest = manifest(
        vec![ROLE_A, ROLE_B],
        vec![PropertySpec::UnboundedCompletionMin],
    );
    assert!(matches!(
        analyze_role_source(
            transfer_source,
            &ProbabilitySidecar::empty(),
            &transfer_manifest
        ),
        Err(AnalysisError::UnsupportedConstruct(_))
    ));

    // 5. State-space exhaustion rejected cleanly.
    let mut tiny_bounds = valid_manifest.clone();
    tiny_bounds.bounds.max_states = 2;
    assert!(matches!(
        analyze_role_source(source.clone(), &valid_sidecar, &tiny_bounds),
        Err(AnalysisError::StateSpaceExhausted { .. })
    ));

    // 6. Missing/unavailable tool binary rejected cleanly.
    let mut bad_tool = valid_manifest.clone();
    bad_tool.tool.prism_binary = String::from("/nonexistent/prism-missing");
    assert!(matches!(
        analyze_role_source(source.clone(), &valid_sidecar, &bad_tool),
        Err(AnalysisError::ToolUnavailable(_))
    ));

    // 7. Tool timeout rejected cleanly (using a tiny stalled script as the checker binary).
    let stall_script =
        std::env::temp_dir().join(format!("choregraph-prism-stall-{}.sh", std::process::id()));
    std::fs::write(&stall_script, "#!/bin/sh\nsleep 5\n")
        .map_err(|e| format!("write stall script: {e}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stall_script, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod stall script: {e}"))?;
    }
    let mut timeout_manifest = valid_manifest.clone();
    timeout_manifest.tool.prism_binary = stall_script.display().to_string();
    timeout_manifest.tool.extra_args = Vec::new();
    timeout_manifest.bounds.timeout_ms = 50;
    let timeout_result = analyze_role_source(source, &valid_sidecar, &timeout_manifest);
    let _ = std::fs::remove_file(&stall_script);
    assert!(matches!(
        timeout_result,
        Err(AnalysisError::ToolTimeout { timeout_ms: 50 })
    ));

    // 8. Malformed checker output rejected cleanly.
    assert!(matches!(
        parse_prism_results("PRISM\nResult: NaN\n", 1),
        Err(AnalysisError::MalformedCheckerOutput(_))
    ));
    assert!(matches!(
        parse_prism_results("PRISM\nno result lines here\n", 1),
        Err(AnalysisError::MalformedCheckerOutput(_))
    ));

    // 9. Existing unannotated v1 fixture retains its exact canonical local-set identity.
    let annotated_json = include_bytes!(
        "../../../fixtures/role-choreography/positive/annotated-two-branch-choice.json"
    );
    let annotated: choregraph_choreography::preference::model::AnnotatedSource =
        serde_json::from_slice(annotated_json).map_err(|e| format!("parse v1 fixture: {e}"))?;
    let base_bytes = serde_json::to_vec(&annotated.base).map_err(|e| format!("{e}"))?;
    let bundle = choregraph_choreography::compile_evaluated(&base_bytes, b"envelope")
        .map_err(|e| format!("compile v1 baseline: {e:?}"))?;
    assert_eq!(
        bundle.local_set().identity.hex,
        "aae9fbd4fefa129f8772f99225d6e0b2a67a311a2d61dd7bb66ee6697ce12194"
    );

    Ok(())
}
