use super::*;

#[test]
fn pure_laws_hold_only_when_dependency_and_effect_premises_hold() {
    let registry = fixture_registry();
    let left = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "parallel", "parts": [operation(("a", "tick")), operation(("b", "tick"))]}}),
    );
    let right = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "parallel", "parts": [operation(("b", "tick")), operation(("a", "tick"))]}}),
    );
    let a = admitted(&left, &registry);
    let b = admitted(&right, &registry);
    assert!(a.projection.edges.is_empty());
    assert!(b.projection.edges.is_empty());
    assert!(a.projection.control_edges.is_empty());
    assert!(b.projection.control_edges.is_empty());
    assert_eq!(
        a.projection, b.projection,
        "independent pure named nodes commute after scoped sorting"
    );
    assert_ne!(
        a.binding, b.binding,
        "authored candidate order remains exactly bound"
    );
    let sequence = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [operation(("a", "tick")), operation(("b", "tick"))]}}),
    );
    assert_ne!(
        a.projection.control_edges,
        admitted(&sequence, &registry).projection.control_edges
    );
    let mut changed_op = right.clone();
    changed_op["body"]["kind"]["parts"][0]["kind"]["operation"] =
        serde_json::json!("linear-source");
    assert!(
        compile(
            &changed_op,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .is_err(),
        "changing operation loses purity/closedness"
    );
    let mut dependent = diamond();
    let _ = dependent["body"]["kind"]["connections"]
        .as_array_mut()
        .required("connections")
        .pop();
    assert!(
        compile(
            &dependent,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .is_err(),
        "omitting dependency is not a law"
    );
    let mut incorrect = diamond();
    incorrect["body"]["kind"]["connections"][0]["source"]["boundary"] =
        serde_json::json!("missing");
    rejected(
        &incorrect,
        &registry,
        crate::construction::Failure::UnknownBoundary,
    );
}
type ObserverRelation = std::collections::BTreeSet<(String, String)>;

// This bounded observer is deliberately restricted to the named, zero-data pure
// fixtures below; it is not a result-semantics interpreter or a general theorem.
fn unit_observer(output: &crate::construction::Output) -> (ObserverRelation, ObserverRelation) {
    let local = |scoped: &str| {
        let mut rest = scoped;
        let mut previous = None;
        let mut last = None;
        while !rest.is_empty() {
            let (length, suffix) = rest.split_once(':').required("framed scope");
            let length_bytes: usize = length.parse().required("scope length");
            let (segment, suffix) = suffix.split_at(length_bytes);
            previous = last;
            last = Some(segment);
            rest = suffix;
        }
        previous.required("node owner scope").to_owned()
    };
    let names = output
        .projection
        .nodes
        .iter()
        .map(|node| (node.id, local(&node.scoped_id)))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        names
            .values()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        names.len()
    );
    assert!(output.projection.edges.is_empty());
    assert!(output.projection.effects.is_empty());
    let nodes = output
        .projection
        .nodes
        .iter()
        .map(|node| {
            assert!(node.configuration.is_empty());
            (names[&node.id].clone(), node.operation.clone())
        })
        .collect();
    let edges = output
        .projection
        .control_edges
        .iter()
        .map(|edge| (names[&edge.source].clone(), names[&edge.target].clone()))
        .collect();
    (nodes, edges)
}

#[test]
fn bounded_associativity_and_dependency_mutation_controls() {
    let sequence = |key: &str, parts: Vec<serde_json::Value>| serde_json::json!({"key":key,"kind":{"tag":"sequence","parts":parts}});
    let left = make_candidate(sequence(
        "root",
        vec![
            sequence(
                "left",
                vec![operation(("a", "tick")), operation(("b", "tick"))],
            ),
            operation(("c", "tick")),
        ],
    ));
    let right = make_candidate(sequence(
        "root",
        vec![
            operation(("a", "tick")),
            sequence(
                "right",
                vec![operation(("b", "tick")), operation(("c", "tick"))],
            ),
        ],
    ));
    let registry = fixture_registry();
    let left = admitted(&left, &registry);
    let right = admitted(&right, &registry);
    assert_eq!(unit_observer(&left), unit_observer(&right));
    let mut changed_registry = registry.clone();
    let mut alternative = changed_registry
        .operations
        .iter()
        .find(|signature| signature.operation == "tick")
        .required("tick")
        .clone();
    alternative.operation = "alternative".into();
    alternative.implementation = "fixture/alternative".into();
    changed_registry.operations.push(alternative);
    let changed = make_candidate(sequence(
        "root",
        vec![
            operation(("a", "alternative")),
            operation(("b", "tick")),
            operation(("c", "tick")),
        ],
    ));
    assert_ne!(
        unit_observer(&left),
        unit_observer(&admitted(&changed, &changed_registry)),
        "a changed admitted operation is observable"
    );
    let mut missing = left.projection.clone();
    missing.control_edges.clear();
    let serialized = serde_json::to_vec(&missing).required("missing dependency projection");
    let missing = crate::construction::admission::lower_projection_json(
        &serialized,
        &registry,
        crate::construction::model::profile::Limits::default(),
    )
    .required("independent graph is separately admissible");
    assert_ne!(
        left.machine, missing.machine,
        "omitting Unit dependency changes executable semantics"
    );
}

// r[verify choregraph.construction.laws]
#[test]
fn sequence_parallel_interchange_changes_barriers_and_is_not_equivalent() {
    let group = |tag: &str, key: &str, parts: Vec<serde_json::Value>| serde_json::json!({"key":key,"kind":{"tag":tag,"parts":parts}});
    let registry = fixture_registry();
    let lanes = admitted(
        &make_candidate(group(
            "parallel",
            "root",
            vec![
                group(
                    "sequence",
                    "upper",
                    vec![operation(("a", "tick")), operation(("b", "tick"))],
                ),
                group(
                    "sequence",
                    "lower",
                    vec![operation(("c", "tick")), operation(("d", "tick"))],
                ),
            ],
        )),
        &registry,
    );
    let stages = admitted(
        &make_candidate(group(
            "sequence",
            "root",
            vec![
                group(
                    "parallel",
                    "first",
                    vec![operation(("a", "tick")), operation(("c", "tick"))],
                ),
                group(
                    "parallel",
                    "second",
                    vec![operation(("b", "tick")), operation(("d", "tick"))],
                ),
            ],
        )),
        &registry,
    );
    let (lane_nodes, lane_edges) = unit_observer(&lanes);
    let (stage_nodes, stage_edges) = unit_observer(&stages);
    assert_eq!(
        lane_nodes, stage_nodes,
        "both groupings admit the same operations"
    );
    let relation = |pairs: &[(&str, &str)]| -> ObserverRelation {
        pairs
            .iter()
            .map(|(source, target)| ((*source).to_owned(), (*target).to_owned()))
            .collect()
    };
    assert_eq!(lane_edges, relation(&[("a", "b"), ("c", "d")]));
    assert_eq!(
        stage_edges,
        relation(&[("a", "b"), ("a", "d"), ("c", "b"), ("c", "d")]),
        "all-to-all exit/entry barriers join the two stages"
    );
    assert_ne!(
        lanes.machine, stages.machine,
        "the interchange barrier is executable dependency structure, not metadata"
    );
}
