// r[verify choregraph.machine.fixtures]
// r[verify choregraph.machine.determinism]
// r[verify choregraph.machine.invariants]
// r[verify choregraph.machine.denial]
// r[verify choregraph.machine.effect_intents]
use std::collections::{BTreeMap, BTreeSet};

use choregraph_core::{
    Action, AdmissionDiagnosticCode, AdmissionLimits, AdmittedMachine, Decision, DenialReason,
    EdgeSpec, EffectDeclarationSpec, EffectRequest, Event, LocalState, MachineSpec, MachineState,
    NodeId, NodeSpec, PayloadRef, TransitionSelector, bottlenecks, critical_path, effects_allowed,
    invariant, parallel_groups, ready_nodes, resume_decision, step, validate_invariant,
    well_formed, would_create_cycle,
};

const NODE_SOURCE: &str = "source";
const NODE_LEFT: &str = "left";
const NODE_RIGHT: &str = "right";
const NODE_SINK: &str = "sink";
const NODE_MISSING: &str = "missing";
const DIAMOND_NODE_COUNT: usize = 4;
const DIAMOND_EDGE_COUNT: usize = 4;
const DIAMOND_GROUP_COUNT: usize = 3;
const DIAMOND_FANOUT_WIDTH: usize = 2;
const ROOT_DEPENDENT_COUNT: usize = 3;
const SMALL_IDENTIFIER_LIMIT: usize = 4;
const SMALL_EVENT_LIMIT: usize = 5;
const INTENT_IDENTIFIER_FIELD_COUNT: usize = 5;
const MINIMUM_TEST_INTENT_BYTES: usize =
    INTENT_IDENTIFIER_FIELD_COUNT + choregraph_core::BLAKE3_REFERENCE_BYTES;
const SMALL_INTENT_LIMIT: usize = SMALL_IDENTIFIER_LIMIT * INTENT_IDENTIFIER_FIELD_COUNT
    + choregraph_core::BLAKE3_REFERENCE_BYTES;
const SINGLE_NODE_LIMIT: usize = 1;
const NO_ITEMS: usize = 0;

fn node(id: &str) -> NodeSpec {
    NodeSpec::new(id, Vec::new())
}

fn diamond_nodes() -> Vec<NodeSpec> {
    vec![
        node(NODE_SINK),
        node(NODE_RIGHT),
        node(NODE_LEFT),
        node(NODE_SOURCE),
    ]
}

fn diamond_edges() -> Vec<EdgeSpec> {
    vec![
        EdgeSpec::new(NODE_SOURCE, NODE_RIGHT),
        EdgeSpec::new(NODE_RIGHT, NODE_SINK),
        EdgeSpec::new(NODE_SOURCE, NODE_LEFT),
        EdgeSpec::new(NODE_LEFT, NODE_SINK),
    ]
}

fn diamond_spec() -> MachineSpec {
    MachineSpec::new(diamond_nodes(), diamond_edges())
}

fn admit(spec: MachineSpec) -> AdmittedMachine {
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("machine admission failed: {diagnostics:?}"),
    }
}

fn node_id(machine: &AdmittedMachine, expected: &str) -> NodeId {
    for node_id in machine.node_ids() {
        if node_id.as_str() == expected {
            return node_id.clone();
        }
    }
    panic!("admitted node is missing: {expected}")
}

fn accept(decision: Decision) -> (MachineState, Vec<choregraph_core::EffectIntent>) {
    match decision {
        Decision::Accept { next, intents } => (next, intents),
        Decision::Deny { reason } => panic!("transition denied: {reason:?}"),
    }
}

fn admission_diagnostics(
    result: Result<AdmittedMachine, Vec<choregraph_core::AdmissionDiagnostic>>,
) -> Vec<choregraph_core::AdmissionDiagnostic> {
    match result {
        Ok(_) => panic!("invalid machine was admitted"),
        Err(diagnostics) => diagnostics,
    }
}

fn diagnostic_codes(
    result: Result<AdmittedMachine, Vec<choregraph_core::AdmissionDiagnostic>>,
) -> BTreeSet<AdmissionDiagnosticCode> {
    admission_diagnostics(result)
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn blake3_reference() -> String {
    String::from("blake3:") + &"a".repeat(choregraph_core::BLAKE3_DIGEST_HEX_BYTES)
}

fn effect_machine() -> AdmittedMachine {
    let mut spec = MachineSpec::new(
        vec![NodeSpec::new(NODE_SOURCE, vec![String::from("fs/write")])],
        Vec::new(),
    );
    spec.effects.push(EffectDeclarationSpec {
        node_id: String::from(NODE_SOURCE),
        transition: TransitionSelector {
            from: LocalState::Running,
            action: Action::Complete,
        },
        operation: String::from("fs/write"),
        resource: String::from("root/work"),
        ability: String::from("file/write"),
        request: String::from("request-1"),
        payload: blake3_reference(),
    });
    admit(spec)
}

fn matching_effect_request() -> EffectRequest {
    EffectRequest {
        operation: String::from("fs/write"),
        resource: String::from("root/work"),
        ability: String::from("file/write"),
        request: String::from("request-1"),
        payload: blake3_reference(),
    }
}

fn effect_declaration(
    node_id: &str,
    operation: &str,
    request: &str,
    payload: String,
    transition: TransitionSelector,
) -> EffectDeclarationSpec {
    EffectDeclarationSpec {
        node_id: String::from(node_id),
        transition,
        operation: String::from(operation),
        resource: String::from("root/work"),
        ability: String::from("file/write"),
        request: String::from(request),
        payload,
    }
}

fn permutations<T: Clone>(values: &[T]) -> Vec<Vec<T>> {
    if values.is_empty() {
        return vec![Vec::new()];
    }
    let mut result = Vec::new();
    for selected in 0..values.len() {
        let mut remainder = values.to_vec();
        let value = remainder.remove(selected);
        for mut suffix in permutations(&remainder) {
            let mut permutation = vec![value.clone()];
            permutation.append(&mut suffix);
            result.push(permutation);
        }
    }
    result
}

#[test]
fn empty_machine_is_valid_and_has_no_work() {
    let machine = admit(MachineSpec::new(Vec::new(), Vec::new()));
    let state = machine.initial_state();
    assert!(well_formed(&machine));
    assert!(invariant(&machine, &state));
    assert!(ready_nodes(&machine, &state).is_ok_and(|nodes| nodes.is_empty()));
    assert!(parallel_groups(&machine).is_empty());
    assert!(critical_path(&machine).is_empty());
    assert!(bottlenecks(&machine).is_empty());
}

#[test]
fn lattice_diamond_has_stable_topology_readiness_and_analytics() {
    let machine = admit(diamond_spec());
    let state = machine.initial_state();
    assert_eq!(machine.node_ids().len(), DIAMOND_NODE_COUNT);
    assert_eq!(machine.edges().len(), DIAMOND_EDGE_COUNT);
    assert_eq!(
        machine
            .topological_order()
            .iter()
            .map(NodeId::as_str)
            .collect::<Vec<_>>(),
        vec![NODE_SOURCE, NODE_LEFT, NODE_RIGHT, NODE_SINK]
    );
    let source = node_id(&machine, NODE_SOURCE);
    let sink = node_id(&machine, NODE_SINK);
    assert!(would_create_cycle(&machine, &sink, &source));
    assert!(!would_create_cycle(&machine, &source, &sink));
    assert_eq!(
        machine
            .dependencies(&sink)
            .map(|values| values.iter().map(NodeId::as_str).collect::<Vec<_>>()),
        Some(vec![NODE_LEFT, NODE_RIGHT])
    );
    assert_eq!(
        ready_nodes(&machine, &state).map(|values| {
            values
                .into_iter()
                .map(|node_id| node_id.as_str().to_string())
                .collect::<Vec<_>>()
        }),
        Ok(vec![String::from(NODE_SOURCE)])
    );

    let groups = parallel_groups(&machine);
    assert_eq!(groups.len(), DIAMOND_GROUP_COUNT);
    assert_eq!(groups[1].node_ids.len(), DIAMOND_FANOUT_WIDTH);
    assert!(groups[1].can_parallelize);
    assert_eq!(
        critical_path(&machine)
            .into_iter()
            .map(|node_id| node_id.as_str().to_string())
            .collect::<Vec<_>>(),
        vec![
            String::from(NODE_SOURCE),
            String::from(NODE_LEFT),
            String::from(NODE_SINK),
        ]
    );
    let ranked = bottlenecks(&machine);
    assert_eq!(ranked[0].node_id.as_str(), NODE_SOURCE);
    assert_eq!(ranked[0].dependent_count, ROOT_DEPENDENT_COUNT);
}

#[test]
fn valid_lifecycle_emits_only_the_exact_declared_intent() {
    let machine = effect_machine();
    let initial = machine.initial_state();
    let (ready, no_ready_intents) = accept(step(
        &machine,
        &initial,
        &Event::new(NODE_SOURCE, Action::MarkReady),
    ));
    assert!(no_ready_intents.is_empty());
    let (running, no_start_intents) = accept(step(
        &machine,
        &ready,
        &Event::new(NODE_SOURCE, Action::Start),
    ));
    assert!(no_start_intents.is_empty());

    let mut complete = Event::new(NODE_SOURCE, Action::Complete);
    complete.effects.push(matching_effect_request());
    let decision = step(&machine, &running, &complete);
    let (completed, intents) = accept(decision);
    assert_eq!(
        completed.state(&node_id(&machine, NODE_SOURCE)),
        Some(LocalState::Completed)
    );
    assert_eq!(intents.len(), SINGLE_NODE_LIMIT);
    assert!(effects_allowed(&machine, &running, &complete, &intents));
}

#[test]
fn denied_events_do_not_replace_caller_state_or_emit_intents() {
    let machine = admit(diamond_spec());
    let initial = machine.initial_state();
    let before = initial.clone();
    let premature = step(
        &machine,
        &initial,
        &Event::new(NODE_LEFT, Action::MarkReady),
    );
    assert_eq!(
        premature,
        Decision::Deny {
            reason: DenialReason::DependenciesIncomplete
        }
    );
    assert_eq!(initial, before);

    let unknown = step(
        &machine,
        &initial,
        &Event::new(NODE_MISSING, Action::MarkReady),
    );
    assert_eq!(
        unknown,
        Decision::Deny {
            reason: DenialReason::UnknownNode
        }
    );
    assert_eq!(initial, before);
}

#[test]
fn terminal_state_mutation_is_denied_without_state_replacement() {
    let machine = effect_machine();
    let initial = machine.initial_state();
    let (ready, _) = accept(step(
        &machine,
        &initial,
        &Event::new(NODE_SOURCE, Action::MarkReady),
    ));
    let (running, _) = accept(step(
        &machine,
        &ready,
        &Event::new(NODE_SOURCE, Action::Start),
    ));
    let mut complete = Event::new(NODE_SOURCE, Action::Complete);
    complete.effects.push(matching_effect_request());
    let (completed, _) = accept(step(&machine, &running, &complete));
    let before = completed.clone();

    let denied = step(
        &machine,
        &completed,
        &Event::new(NODE_SOURCE, Action::Cancel),
    );
    assert_eq!(
        denied,
        Decision::Deny {
            reason: DenialReason::TerminalState {
                from: LocalState::Completed,
                action: Action::Cancel,
            }
        }
    );
    assert_eq!(completed, before);
}

#[test]
fn invariant_rejects_missing_extra_and_premature_state_entries() {
    let machine = admit(diamond_spec());
    let source = node_id(&machine, NODE_SOURCE);
    let left = node_id(&machine, NODE_LEFT);
    let extra = match NodeId::parse("extra", machine.limits().max_identifier_bytes) {
        Ok(identifier) => identifier,
        Err(error) => panic!("extra test identifier failed: {error:?}"),
    };
    let mut entries = BTreeMap::new();
    entries.insert(source, LocalState::Pending);
    entries.insert(left, LocalState::Ready);
    entries.insert(extra, LocalState::Pending);
    let invalid = MachineState::new(entries);
    let issues = validate_invariant(&machine, &invalid);
    assert!(!issues.is_empty());
    assert!(!invariant(&machine, &invalid));
}

#[test]
fn admission_rejects_malformed_graph_facts_and_cycles() {
    let malformed = MachineSpec::new(
        vec![
            NodeSpec::new("a", vec![String::from("op"), String::from("op")]),
            node("a"),
            node("bad id"),
        ],
        vec![
            EdgeSpec::new("a", "a"),
            EdgeSpec::new("a", NODE_MISSING),
            EdgeSpec::new(NODE_MISSING, "a"),
            EdgeSpec::new("a", NODE_MISSING),
        ],
    );
    let codes = diagnostic_codes(AdmittedMachine::admit(
        malformed,
        AdmissionLimits::default(),
    ));
    for expected in [
        AdmissionDiagnosticCode::DuplicateNode,
        AdmissionDiagnosticCode::DuplicateOperation,
        AdmissionDiagnosticCode::DuplicateEdge,
        AdmissionDiagnosticCode::MissingSource,
        AdmissionDiagnosticCode::MissingTarget,
        AdmissionDiagnosticCode::SelfLoop,
        AdmissionDiagnosticCode::MalformedIdentifier,
    ] {
        assert!(codes.contains(&expected));
    }

    let cycle = MachineSpec::new(
        vec![node("a"), node("b")],
        vec![EdgeSpec::new("a", "b"), EdgeSpec::new("b", "a")],
    );
    let cycle_codes = diagnostic_codes(AdmittedMachine::admit(cycle, AdmissionLimits::default()));
    assert_eq!(
        cycle_codes,
        BTreeSet::from([AdmissionDiagnosticCode::CycleDetected])
    );
}

#[test]
fn admission_rejects_unsupported_profiles_and_malformed_effects() {
    let mut unsupported = MachineSpec::new(Vec::new(), Vec::new());
    unsupported.lifecycle_profile = String::from("unknown");
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            unsupported,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::UnsupportedLifecycleProfile)
    );

    let mut undeclared = MachineSpec::new(vec![node(NODE_SOURCE)], Vec::new());
    undeclared.effects.push(EffectDeclarationSpec {
        node_id: String::from(NODE_SOURCE),
        transition: TransitionSelector {
            from: LocalState::Running,
            action: Action::Complete,
        },
        operation: String::from("fs/write"),
        resource: String::from("root/work"),
        ability: String::from("file/write"),
        request: String::from("request-1"),
        payload: blake3_reference(),
    });
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            undeclared,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::EffectOperationUndeclared)
    );

    let mut malformed_reference = MachineSpec::new(
        vec![NodeSpec::new(NODE_SOURCE, vec![String::from("fs/write")])],
        Vec::new(),
    );
    malformed_reference.effects.push(EffectDeclarationSpec {
        node_id: String::from(NODE_SOURCE),
        transition: TransitionSelector {
            from: LocalState::Running,
            action: Action::Complete,
        },
        operation: String::from("fs/write"),
        resource: String::from("root/work"),
        ability: String::from("file/write"),
        request: String::from("request-1"),
        payload: String::from("sha256:bad"),
    });
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            malformed_reference,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::MalformedReference)
    );
}

#[test]
fn admission_diagnostics_are_independent_of_input_order() {
    let first = MachineSpec::new(
        vec![node("a"), node("a"), node("bad id")],
        vec![
            EdgeSpec::new("a", NODE_MISSING),
            EdgeSpec::new(NODE_MISSING, "a"),
            EdgeSpec::new("a", NODE_MISSING),
        ],
    );
    let mut second = first.clone();
    second.nodes.reverse();
    second.edges.reverse();
    assert_eq!(
        admission_diagnostics(AdmittedMachine::admit(first, AdmissionLimits::default())),
        admission_diagnostics(AdmittedMachine::admit(second, AdmissionLimits::default()))
    );
}

#[test]
fn admission_failure_corpus_covers_limits_and_effect_closure() {
    let inconsistent_limits = AdmissionLimits {
        max_identifier_bytes: NO_ITEMS,
        ..AdmissionLimits::default()
    };
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            MachineSpec::new(Vec::new(), Vec::new()),
            inconsistent_limits
        ))
        .contains(&AdmissionDiagnosticCode::InconsistentLimits)
    );

    let identifier_limits = AdmissionLimits {
        max_identifier_bytes: SMALL_IDENTIFIER_LIMIT,
        ..AdmissionLimits::default()
    };
    let identifiers = MachineSpec::new(vec![node(""), node("longer")], Vec::new());
    let identifier_codes = diagnostic_codes(AdmittedMachine::admit(identifiers, identifier_limits));
    assert!(identifier_codes.contains(&AdmissionDiagnosticCode::EmptyIdentifier));
    assert!(identifier_codes.contains(&AdmissionDiagnosticCode::IdentifierTooLong));

    let mut schema = MachineSpec::new(Vec::new(), Vec::new());
    schema.schema = String::from("unknown");
    assert!(
        diagnostic_codes(AdmittedMachine::admit(schema, AdmissionLimits::default()))
            .contains(&AdmissionDiagnosticCode::UnsupportedSchema)
    );

    let valid_transition = TransitionSelector {
        from: LocalState::Running,
        action: Action::Complete,
    };
    let mut missing_node = MachineSpec::new(
        vec![NodeSpec::new("a", vec![String::from("op")])],
        Vec::new(),
    );
    missing_node.effects.push(effect_declaration(
        NODE_MISSING,
        "op",
        "request",
        blake3_reference(),
        valid_transition,
    ));
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            missing_node,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::EffectNodeMissing)
    );

    let mut invalid_transition = MachineSpec::new(
        vec![NodeSpec::new("a", vec![String::from("op")])],
        Vec::new(),
    );
    invalid_transition.effects.push(effect_declaration(
        "a",
        "op",
        "request",
        blake3_reference(),
        TransitionSelector {
            from: LocalState::Completed,
            action: Action::Complete,
        },
    ));
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            invalid_transition,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::InvalidEffectTransition)
    );

    let declaration =
        effect_declaration("a", "op", "request", blake3_reference(), valid_transition);
    let mut duplicate = MachineSpec::new(
        vec![NodeSpec::new("a", vec![String::from("op")])],
        Vec::new(),
    );
    duplicate.effects = vec![declaration.clone(), declaration];
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            duplicate,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::DuplicateEffectDeclaration)
    );

    let oversized_reference = String::from("blake3:")
        + &"a".repeat(choregraph_core::BLAKE3_DIGEST_HEX_BYTES + SINGLE_NODE_LIMIT);
    let mut reference_spec = MachineSpec::new(
        vec![NodeSpec::new("a", vec![String::from("op")])],
        Vec::new(),
    );
    reference_spec.effects.push(effect_declaration(
        "a",
        "op",
        "request",
        oversized_reference,
        valid_transition,
    ));
    assert!(
        diagnostic_codes(AdmittedMachine::admit(
            reference_spec,
            AdmissionLimits::default()
        ))
        .contains(&AdmissionDiagnosticCode::ReferenceTooLong)
    );

    let intent_limits = AdmissionLimits {
        max_intent_bytes: MINIMUM_TEST_INTENT_BYTES,
        ..AdmissionLimits::default()
    };
    let mut large_intent = MachineSpec::new(
        vec![NodeSpec::new(NODE_SOURCE, vec![String::from("fs/write")])],
        Vec::new(),
    );
    large_intent.effects.push(effect_declaration(
        NODE_SOURCE,
        "fs/write",
        "request-1",
        blake3_reference(),
        valid_transition,
    ));
    assert!(
        diagnostic_codes(AdmittedMachine::admit(large_intent, intent_limits))
            .contains(&AdmissionDiagnosticCode::IntentTooLarge)
    );

    let transition_limits = AdmissionLimits {
        max_intents_per_transition: SINGLE_NODE_LIMIT,
        ..AdmissionLimits::default()
    };
    let mut too_many_intents = MachineSpec::new(
        vec![NodeSpec::new(
            "a",
            vec![String::from("op/a"), String::from("op/b")],
        )],
        Vec::new(),
    );
    for operation in ["op/a", "op/b"] {
        too_many_intents.effects.push(effect_declaration(
            "a",
            operation,
            operation,
            blake3_reference(),
            valid_transition,
        ));
    }
    assert!(
        diagnostic_codes(AdmittedMachine::admit(too_many_intents, transition_limits))
            .contains(&AdmissionDiagnosticCode::TooManyIntentsForTransition)
    );
}

#[test]
fn top_level_count_limits_fail_before_graph_normalization() {
    let edge_limits = AdmissionLimits {
        max_edges: NO_ITEMS,
        ..AdmissionLimits::default()
    };
    let edges = MachineSpec::new(vec![node("a"), node("b")], vec![EdgeSpec::new("a", "b")]);
    assert!(
        diagnostic_codes(AdmittedMachine::admit(edges, edge_limits))
            .contains(&AdmissionDiagnosticCode::TooManyEdges)
    );

    let operation_limits = AdmissionLimits {
        max_operations: NO_ITEMS,
        ..AdmissionLimits::default()
    };
    let operations = MachineSpec::new(
        vec![NodeSpec::new("a", vec![String::from("op")])],
        Vec::new(),
    );
    assert!(
        diagnostic_codes(AdmittedMachine::admit(operations, operation_limits))
            .contains(&AdmissionDiagnosticCode::TooManyOperations)
    );

    let effect_limits = AdmissionLimits {
        max_effect_declarations: NO_ITEMS,
        ..AdmissionLimits::default()
    };
    let mut effects = MachineSpec::new(vec![node("a")], Vec::new());
    effects.effects.push(effect_declaration(
        "a",
        "op",
        "request",
        blake3_reference(),
        TransitionSelector {
            from: LocalState::Running,
            action: Action::Complete,
        },
    ));
    assert!(
        diagnostic_codes(AdmittedMachine::admit(effects, effect_limits))
            .contains(&AdmissionDiagnosticCode::TooManyEffectDeclarations)
    );
}

#[test]
fn selected_limits_accept_the_boundary_and_reject_the_next_value() {
    let limits = AdmissionLimits {
        max_nodes: SINGLE_NODE_LIMIT,
        max_edges: NO_ITEMS,
        max_operations: NO_ITEMS,
        max_effect_declarations: NO_ITEMS,
        max_identifier_bytes: SMALL_IDENTIFIER_LIMIT,
        max_reference_bytes: choregraph_core::BLAKE3_REFERENCE_BYTES,
        max_event_bytes: SMALL_EVENT_LIMIT,
        max_intent_bytes: SMALL_INTENT_LIMIT,
        max_intents_per_transition: NO_ITEMS,
    };
    let admitted =
        match AdmittedMachine::admit(MachineSpec::new(vec![node("root")], Vec::new()), limits) {
            Ok(machine) => machine,
            Err(diagnostics) => panic!("boundary machine failed: {diagnostics:?}"),
        };
    let denied = step(
        &admitted,
        &admitted.initial_state(),
        &Event::new("roots", Action::MarkReady),
    );
    assert!(matches!(
        denied,
        Decision::Deny {
            reason: DenialReason::EventTooLarge { .. }
        }
    ));

    let over = MachineSpec::new(vec![node("root"), node("next")], Vec::new());
    let codes = diagnostic_codes(AdmittedMachine::admit(over, limits));
    assert!(codes.contains(&AdmissionDiagnosticCode::TooManyNodes));
}

#[test]
fn undeclared_and_duplicate_effect_requests_are_denied() {
    let machine = effect_machine();
    let initial = machine.initial_state();
    let (ready, _) = accept(step(
        &machine,
        &initial,
        &Event::new(NODE_SOURCE, Action::MarkReady),
    ));
    let (running, _) = accept(step(
        &machine,
        &ready,
        &Event::new(NODE_SOURCE, Action::Start),
    ));

    let mut undeclared = Event::new(NODE_SOURCE, Action::Complete);
    let mut wrong = matching_effect_request();
    wrong.ability = String::from("file/delete");
    undeclared.effects.push(wrong);
    assert_eq!(
        step(&machine, &running, &undeclared),
        Decision::Deny {
            reason: DenialReason::UndeclaredEffect
        }
    );

    let mut duplicate = Event::new(NODE_SOURCE, Action::Complete);
    duplicate.effects.push(matching_effect_request());
    duplicate.effects.push(matching_effect_request());
    assert_eq!(
        step(&machine, &running, &duplicate),
        Decision::Deny {
            reason: DenialReason::DuplicateEffectRequest
        }
    );
}

#[test]
fn property_all_diamond_input_orders_normalize_equally() {
    let baseline = admit(diamond_spec());
    for nodes in permutations(&diamond_nodes()) {
        for edges in permutations(&diamond_edges()) {
            let candidate = admit(MachineSpec::new(nodes.clone(), edges));
            assert_eq!(candidate, baseline);
        }
    }
}

#[test]
fn property_every_accepted_local_transition_preserves_the_invariant() {
    let machine = admit(MachineSpec::new(vec![node(NODE_SOURCE)], Vec::new()));
    let node_id = node_id(&machine, NODE_SOURCE);
    let states = [
        LocalState::Pending,
        LocalState::Ready,
        LocalState::Running,
        LocalState::Completed,
        LocalState::Failed,
        LocalState::Cancelled,
    ];
    let actions = [
        Action::MarkReady,
        Action::Start,
        Action::Complete,
        Action::Fail,
        Action::Cancel,
    ];

    for local_state in states {
        let state = MachineState::new(BTreeMap::from([(node_id.clone(), local_state)]));
        assert!(invariant(&machine, &state));
        for action in actions {
            if let Decision::Accept { next, intents } =
                step(&machine, &state, &Event::new(NODE_SOURCE, action))
            {
                assert!(invariant(&machine, &next));
                assert!(intents.is_empty());
            }
        }
    }
}

#[test]
fn resume_decisions_match_completed_failed_and_waiting_dependencies() {
    let machine = admit(diamond_spec());
    let source = node_id(&machine, NODE_SOURCE);
    let left = node_id(&machine, NODE_LEFT);
    let initial = machine.initial_state();
    assert_eq!(
        resume_decision(&machine, &initial, &left),
        choregraph_core::ResumeDecision::AwaitDependencies
    );

    let mut completed_entries = initial.entries().clone();
    completed_entries.insert(source.clone(), LocalState::Completed);
    let completed = MachineState::new(completed_entries);
    assert_eq!(
        resume_decision(&machine, &completed, &left),
        choregraph_core::ResumeDecision::Execute
    );
    assert_eq!(
        resume_decision(&machine, &completed, &source),
        choregraph_core::ResumeDecision::SkipCompleted
    );

    let mut failed_entries = initial.entries().clone();
    failed_entries.insert(source, LocalState::Failed);
    let failed = MachineState::new(failed_entries);
    assert_eq!(
        resume_decision(&machine, &failed, &left),
        choregraph_core::ResumeDecision::BlockedByFailedDependency
    );
}

#[test]
fn dependency_invariant_checks_the_complete_ancestor_chain() {
    let machine = admit(MachineSpec::new(
        vec![node("a"), node("b"), node("c")],
        vec![EdgeSpec::new("a", "b"), EdgeSpec::new("b", "c")],
    ));
    let a = node_id(&machine, "a");
    let b = node_id(&machine, "b");
    let c = node_id(&machine, "c");
    let invalid = MachineState::new(BTreeMap::from([
        (a, LocalState::Pending),
        (b, LocalState::Completed),
        (c, LocalState::Pending),
    ]));
    assert!(!invariant(&machine, &invalid));
}

#[test]
fn effect_intent_order_is_independent_of_declaration_and_request_order() {
    let payload = blake3_reference();
    let mut spec = MachineSpec::new(
        vec![NodeSpec::new(
            NODE_SOURCE,
            vec![String::from("op/z"), String::from("op/a")],
        )],
        Vec::new(),
    );
    for operation in ["op/z", "op/a"] {
        spec.effects.push(EffectDeclarationSpec {
            node_id: String::from(NODE_SOURCE),
            transition: TransitionSelector {
                from: LocalState::Running,
                action: Action::Complete,
            },
            operation: String::from(operation),
            resource: String::from("root/work"),
            ability: String::from("file/write"),
            request: String::from(operation),
            payload: payload.clone(),
        });
    }
    let machine = admit(spec);
    let initial = machine.initial_state();
    let (ready, _) = accept(step(
        &machine,
        &initial,
        &Event::new(NODE_SOURCE, Action::MarkReady),
    ));
    let (running, _) = accept(step(
        &machine,
        &ready,
        &Event::new(NODE_SOURCE, Action::Start),
    ));
    let mut complete = Event::new(NODE_SOURCE, Action::Complete);
    for operation in ["op/z", "op/a"] {
        complete.effects.push(EffectRequest {
            operation: String::from(operation),
            resource: String::from("root/work"),
            ability: String::from("file/write"),
            request: String::from(operation),
            payload: payload.clone(),
        });
    }
    let reverse_decision = step(&machine, &running, &complete);
    complete.effects.reverse();
    let forward_decision = step(&machine, &running, &complete);
    assert_eq!(reverse_decision, forward_decision);
    let (_, intents) = accept(reverse_decision);
    assert_eq!(
        intents
            .iter()
            .map(|intent| intent.operation.as_str())
            .collect::<Vec<_>>(),
        vec!["op/a", "op/z"]
    );
}

#[test]
fn payload_reference_parser_accepts_only_canonical_blake3_identity() {
    assert!(PayloadRef::parse(blake3_reference(), choregraph_core::BLAKE3_REFERENCE_BYTES).is_ok());
    assert!(PayloadRef::parse("blake3:ABC", choregraph_core::BLAKE3_REFERENCE_BYTES).is_err());
}
