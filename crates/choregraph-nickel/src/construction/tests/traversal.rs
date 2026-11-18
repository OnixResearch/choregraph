use super::*;

#[test]
fn fragment_depth_is_inclusive_and_keeps_full_origin_scope() {
    let graph = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "parallel", "parts": [
            {"key": "middle", "kind": {"tag": "parallel", "parts": [operation(("leaf", "tick"))]}}
        ]}}),
    );
    let registry = fixture_registry();
    let limits = crate::construction::model::profile::Limits {
        max_depth: 2,
        ..crate::construction::model::profile::Limits::default()
    };
    let output = compile(&graph, &registry, limits).required("inclusive fragment depth");
    assert_eq!(output.projection.nodes.len(), 1);
    assert!(
        output
            .origins
            .iter()
            .any(|origin| origin.instance_path == ["workflow", "root", "middle", "leaf"])
    );
    assert_eq!(
        compile(
            &graph,
            &registry,
            crate::construction::model::profile::Limits {
                max_depth: 1,
                ..limits
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
}

#[test]
fn thousand_identity_junctions_between_operations_admit() {
    const JUNCTIONS: usize = 1_000;
    let mut parts = vec![operation(("source", "fork")), operation(("sink", "join"))];
    let mut connections = Vec::with_capacity(JUNCTIONS + 2);
    let mut previous = String::from("source");
    let mut previous_boundary = "left";
    for index in 0..JUNCTIONS {
        let key = format!("junction-{index:04}");
        connections.push(connection((&previous, previous_boundary), (&key, "value")));
        parts.push(
            serde_json::json!({"key": key, "kind": {"tag": "identity", "boundaries": [
                {"name": "value", "value_type": "Text"}
            ]}}),
        );
        previous = key;
        previous_boundary = "value";
    }
    connections.push(connection((&previous, previous_boundary), ("sink", "left")));
    connections.push(connection(("source", "right"), ("sink", "right")));
    let graph = make_candidate(wire(parts, connections));
    let output = admitted(&graph, &fixture_registry());
    assert_eq!(output.projection.nodes.len(), 2);
    assert_eq!(output.projection.edges.len(), 2);
}

#[test]
fn disconnected_identity_only_cycle_is_rejected() {
    let identity = |key| {
        serde_json::json!({"key": key, "kind": {"tag": "identity", "boundaries": [
            {"name": "value", "value_type": "Text"}
        ]}})
    };
    let graph = make_candidate(wire(
        vec![operation(("node", "tick")), identity("a"), identity("b")],
        vec![
            connection(("a", "value"), ("b", "value")),
            connection(("b", "value"), ("a", "value")),
        ],
    ));
    rejected(
        &graph,
        &fixture_registry(),
        crate::construction::Failure::AdmissionRejected,
    );
}

fn expression_registry() -> crate::construction::model::host::TargetRegistry {
    let mut registry = fixture_registry();
    let tick = registry
        .operations
        .iter_mut()
        .find(|signature| signature.operation == "tick")
        .required("tick");
    tick.configuration = vec![crate::construction::model::host::ConfigurationField {
        name: "label".into(),
        value_type: "string".into(),
        references: true,
        operators: vec!["concat".into()],
    }];
    registry
}

#[test]
fn expression_depth_is_inclusive_in_substitution_and_projection() {
    let mut graph = make_candidate(operation(("tick", "tick")));
    graph["parameters"] = serde_json::json!({"label": "value"});
    graph["body"]["kind"]["configuration"] = serde_json::json!({"label": {"tag": "apply", "operator": "concat", "arguments": [
        {"tag": "apply", "operator": "concat", "arguments": [{"tag": "parameter", "name": "label"}]}
    ]}});
    let registry = expression_registry();
    let limits = crate::construction::model::profile::Limits {
        max_depth: 2,
        ..crate::construction::model::profile::Limits::default()
    };
    let output = compile(&graph, &registry, limits).required("inclusive expression depth");
    let bytes = serde_json::to_vec(&output.projection).required("expression projection");
    let readmitted =
        crate::construction::admission::lower_projection_json(&bytes, &registry, limits)
            .required("iterative native readmission");
    assert_eq!(readmitted.projection, output.projection);
    assert_eq!(
        compile(
            &graph,
            &registry,
            crate::construction::model::profile::Limits {
                max_depth: 1,
                ..limits
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
    graph["body"]["kind"]["configuration"]["label"]["arguments"][0]["arguments"][0] = serde_json::json!({"tag": "reference", "reference": {"direction": "config", "name": "label"}});
    assert_eq!(
        compile(&graph, &registry, limits).err(),
        Some(crate::construction::Failure::InvalidReference)
    );
}

#[test]
fn recursive_templates_stop_at_both_independent_budgets() {
    let mut graph = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "instantiate", "template": "loop", "arguments": {}}}),
    );
    graph["templates"] = serde_json::json!([{"name": "loop", "parameters": [], "body": {
        "key": "again", "kind": {"tag": "instantiate", "template": "loop", "arguments": {}}
    }}]);
    let registry = fixture_registry();
    assert_eq!(
        compile(
            &graph,
            &registry,
            crate::construction::model::profile::Limits {
                max_depth: 3,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
    assert_eq!(
        compile(
            &graph,
            &registry,
            crate::construction::model::profile::Limits {
                max_instances: 3,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
}

#[test]
fn earlier_child_denial_precedes_later_duplicate_detection() {
    let mut graph = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "parallel", "parts": [
            operation(("same", "absent")), operation(("same", "tick"))
        ]}}),
    );
    let registry = fixture_registry();
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::UnknownOperation,
    );
    graph["body"]["kind"]["parts"][0]["kind"]["operation"] = serde_json::json!("tick");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::DuplicateKey,
    );
}
