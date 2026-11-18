use super::*;

fn invocation_registry() -> crate::construction::model::host::TargetRegistry {
    let mut registry = fixture_registry();
    let tick = registry
        .operations
        .iter_mut()
        .find(|signature| signature.operation == "tick")
        .required("tick");
    tick.invocation = roles();
    tick.effects = vec![rule("shared/external-resource")];
    registry
}

fn roles() -> Vec<crate::construction::model::interface::Boundary> {
    vec![
        crate::construction::model::interface::Boundary {
            name: "dispatch-request".into(),
            value_type: "Text".into(),
        },
        crate::construction::model::interface::Boundary {
            name: "resolved-payload".into(),
            value_type: "Text".into(),
        },
    ]
}

fn invocation_graph() -> serde_json::Value {
    let mut operation = operation(("body", "tick"));
    operation["kind"]["effects"] = serde_json::json!([{"key":"invoke","from_state":"ready","action":"start","resource":"shared/external-resource","ability":"run",
        "request":{"mode":"slot","slot":"dispatch-request","source":{"direction":"invocation","name":"dispatch-request"}},
        "payload":{"mode":"slot","slot":"resolved-payload","source":{"direction":"invocation","name":"resolved-payload"}}}]);
    let mut graph = make_candidate(
        serde_json::json!({"key":"root","kind":{"tag":"sequence","parts":[
            {"key":"a","kind":{"tag":"instantiate","template":"invoke","arguments":{}}},
            {"key":"b","kind":{"tag":"instantiate","template":"invoke","arguments":{}}}
        ]}}),
    );
    graph["templates"] = serde_json::json!([{"name":"invoke","parameters":[],"body":operation}]);
    graph
}

fn field(name: &str, references: bool) -> crate::construction::model::host::ConfigurationField {
    crate::construction::model::host::ConfigurationField {
        name: name.into(),
        value_type: "string".into(),
        references,
        operators: if references {
            vec!["concat".into()]
        } else {
            vec![]
        },
    }
}

#[test]
fn configuration_uses_selected_field_types_on_compile_and_readmission() {
    let mut registry = fixture_registry();
    let tick = registry
        .operations
        .iter_mut()
        .find(|signature| signature.operation == "tick")
        .required("tick");
    let mut source = field("source-number", false);
    source.value_type = "number".into();
    let mut count = field("count", true);
    count.value_type = "number".into();
    tick.configuration = vec![field("label", false), source, count];

    let mut graph = make_candidate(operation(("node", "tick")));
    graph["body"]["kind"]["configuration"] = serde_json::json!({
        "label": {"tag": "literal", "value": "text"},
        "source-number": {"tag": "literal", "value": 3},
        "count": {"tag": "reference", "reference": {"direction": "config", "name": "source-number"}}
    });
    let admitted = admitted(&graph, &registry);
    let bytes = serde_json::to_vec(&admitted.projection).required("same-type projection");
    assert!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .is_ok()
    );

    graph["body"]["kind"]["configuration"]["count"]["reference"]["name"] =
        serde_json::json!("label");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::IncompatibleType,
    );

    let mut projection = admitted.projection.clone();
    let count = projection.nodes[0]
        .configuration
        .get_mut("count")
        .required("count configuration");
    let crate::construction::model::projection::ExpandedExpression::Reference { name, .. } = count
    else {
        panic!("count must remain a reference");
    };
    *name = "label".into();
    let bytes = serde_json::to_vec(&projection).required("mismatched projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::IncompatibleType)
    );
    let crate::construction::model::projection::ExpandedExpression::Reference { name, .. } =
        projection.nodes[0]
            .configuration
            .get_mut("count")
            .required("count configuration")
    else {
        panic!("count must remain a reference");
    };
    *name = "missing".into();
    let bytes = serde_json::to_vec(&projection).required("missing configuration projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidReference)
    );
    let mut projection = admitted.projection;
    let count = projection.nodes[0]
        .configuration
        .get_mut("count")
        .required("count configuration");
    let crate::construction::model::projection::ExpandedExpression::Reference { node, .. } = count
    else {
        panic!("count must remain a reference");
    };
    *node = u64::MAX;
    let bytes = serde_json::to_vec(&projection).required("foreign configuration projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidReference)
    );
}

fn scoped_effect_registry() -> crate::construction::model::host::TargetRegistry {
    let mut registry = fixture_registry();
    let tick = registry
        .operations
        .iter_mut()
        .find(|signature| signature.operation == "tick")
        .required("tick");
    tick.configuration = vec![field("label", false), field("request", true)];
    tick.effects = vec![rule("machine")];
    registry
}

fn scoped_effect_graph() -> serde_json::Value {
    let mut graph = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "parallel", "parts": []}}),
    );
    let fragment = scoped_fragment();
    graph["templates"] =
        serde_json::json!([{"name": "effectful", "parameters": [], "body": fragment}]);
    graph["body"]["kind"]["parts"] = serde_json::json!([
        {"key": "a", "kind": {"tag": "instantiate", "template": "effectful", "arguments": {}}},
        {"key": "b", "kind": {"tag": "instantiate", "template": "effectful", "arguments": {}}}
    ]);
    graph
}

fn scoped_fragment() -> serde_json::Value {
    serde_json::json!({"key": "worker", "kind": {"tag": "operation", "operation": "tick", "configuration": {
        "label": {"tag": "literal", "value": "prefix"},
        "request": {"tag": "apply", "operator": "concat", "arguments": [
            {"tag": "literal", "value": "prefix"},
            {"tag": "reference", "reference": {"direction": "config", "name": "label"}}
        ]}
    }, "effects": [{"key": "effect", "from_state": "ready", "action": "start", "resource": "machine", "ability": "run",
        "request": {"mode": "exact", "reference": {"profile": "choregraph.packed-preserves.v1", "identity": "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},
        "payload": {"mode": "slot", "slot": "payload", "source": {"direction": "config", "name": "request"}}
    }]}})
}

#[test]
fn scoped_effect_and_config_refs_rewrite_but_exact_refs_stay_exact() {
    let registry = scoped_effect_registry();
    let graph = scoped_effect_graph();
    let output = admitted(&graph, &registry);
    assert_eq!(output.projection.effects.len(), 2);
    let mut one_rule_registry = registry.clone();
    one_rule_registry
        .operations
        .retain(|signature| signature.operation != "effect-tick");
    assert_eq!(
        compile(
            &graph,
            &one_rule_registry,
            crate::construction::model::profile::Limits {
                max_effects: 1,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
    assert_ne!(
        output.projection.effects[0].declaration_key,
        output.projection.effects[1].declaration_key
    );
    match (
        &output.projection.effects[0].request_binding,
        &output.projection.effects[1].request_binding,
    ) {
        (
            crate::construction::model::projection::ExpandedBinding::Exact { reference: a },
            crate::construction::model::projection::ExpandedBinding::Exact { reference: b },
        ) => {
            assert_eq!(a, b)
        }
        other => panic!("exact binding lost: {other:?}"),
    }
    assert_ne!(
        output.projection.effects[0].payload_binding,
        output.projection.effects[1].payload_binding
    );
    let mut sequenced = graph.clone();
    sequenced["body"]["kind"]["tag"] = serde_json::json!("sequence");
    sequenced["body"]["kind"]
        .as_object_mut()
        .required("sequence kind")
        .retain(|name, _| name == "tag" || name == "parts");
    let sequenced = admitted(&sequenced, &registry);
    assert_eq!(sequenced.projection.effects.len(), 2);
    assert_eq!(sequenced.projection.control_edges.len(), 1);
    let mut invalid = graph.clone();
    invalid["templates"][0]["body"]["kind"]["effects"][0]["payload"]["source"]["name"] =
        serde_json::json!("missing");
    rejected(
        &invalid,
        &registry,
        crate::construction::Failure::InvalidReference,
    );
    invalid = graph;
    invalid["templates"][0]["body"]["kind"]["configuration"]["request"]["operator"] =
        serde_json::json!("forbidden");
    rejected(
        &invalid,
        &registry,
        crate::construction::Failure::InvalidConfiguration,
    );
}
#[test]
fn invocation_sources_are_registry_selected_not_data_ports_or_author_fields() {
    let registry = invocation_registry();
    let graph = invocation_graph();
    let output = admitted(&graph, &registry);
    assert_eq!(output.projection.control_edges.len(), 1);
    for node in &output.projection.nodes {
        assert!(
            node.ports.is_empty(),
            "invocation roles do not invent ordinary data ports"
        );
        assert_eq!(node.invocation.len(), 2);
    }
    for effect in &output.projection.effects {
        assert_eq!(
            effect.resource, "shared/external-resource",
            "exact shared resources are not occurrence identities"
        );
        match &effect.request_binding {
            crate::construction::model::projection::ExpandedBinding::Slot {
                source_ref, ..
            } => assert_eq!(
                source_ref,
                &format!("node/{}/invocation/dispatch-request", effect.node_id)
            ),
            _ => panic!("invocation slot became exact"),
        }
    }
    let mut invalid = graph.clone();
    invalid["templates"][0]["body"]["kind"]["effects"][0]["request"]["source"]["name"] =
        serde_json::json!("author-invented");
    rejected(
        &invalid,
        &registry,
        crate::construction::Failure::InvalidReference,
    );
    let mut projection = output.projection.clone();
    projection.nodes[0]
        .invocation
        .push(crate::construction::model::interface::Boundary {
            name: "invented".into(),
            value_type: "Text".into(),
        });
    let bytes = serde_json::to_vec(&projection).required("invented role projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidRegistry)
    );
    let mut projection = output.projection.clone();
    let other = projection.effects[1].node_id;
    if let crate::construction::model::projection::ExpandedBinding::Slot { source_ref, .. } =
        &mut projection.effects[0].request_binding
    {
        *source_ref = format!("node/{other}/invocation/dispatch-request");
    }
    let bytes = serde_json::to_vec(&projection).required("foreign invocation role projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidReference)
    );
}
