use super::*;

#[test]
fn unknown_operations_templates_parameters_and_open_root_are_denied() {
    let registry = fixture_registry();
    let mut graph = make_candidate(operation(("only", "not-registered")));
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::UnknownOperation,
    );
    graph = make_candidate(operation(("only", "pass")));
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::OpenBoundary,
    );
    graph = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "instantiate", "template": "missing", "arguments": {}}}),
    );
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::UnknownTemplate,
    );
    graph = nested();
    graph["body"]["kind"]["parts"][1]["kind"]["arguments"] = serde_json::json!({});
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidParameters,
    );
}

#[test]
fn invalid_configuration_schema_and_key_are_denied() {
    let registry = fixture_registry();
    let mut graph = make_candidate(operation(("tick", "tick")));
    graph["body"]["kind"]["configuration"] =
        serde_json::json!({"invented": {"tag": "literal", "value": "text"}});
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidConfiguration,
    );
    graph = make_candidate(operation(("tick", "tick")));
    graph["schema"] = serde_json::json!("choregraph.graph-construction.v0");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidSchema,
    );
    graph = make_candidate(operation(("tick", "tick")));
    graph["body"]["key"] = serde_json::json!("bad/key");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidIdentifier,
    );
}

#[test]
fn wire_denies_wrong_direction_port_type_root_key_and_cycle() {
    let registry = fixture_registry();
    let mut graph = diamond();
    graph["body"]["kind"]["connections"][0]["source"]["direction"] = serde_json::json!("input");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::WrongDirection,
    );
    graph = diamond();
    graph["body"]["kind"]["connections"][0]["target"]["boundary"] = serde_json::json!("absent");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::UnknownBoundary,
    );
    graph = diamond();
    graph["body"]["kind"]["connections"][0]["source"]["root"] = serde_json::json!("other");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidHandle,
    );
    graph = diamond();
    graph["body"]["kind"]["connections"][0]["source"]["instance"] = serde_json::json!("missing");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::InvalidHandle,
    );
}

#[test]
fn duplicate_keys_cycles_and_type_mismatches_are_denied() {
    let registry = fixture_registry();
    let mut graph = diamond();
    graph["body"]["kind"]["parts"][2]["key"] = serde_json::json!("a");
    rejected(
        &graph,
        &registry,
        crate::construction::Failure::DuplicateKey,
    );
    graph = diamond();
    graph["body"]["kind"]["connections"][0]["target"]["instance"] = serde_json::json!("join");
    graph["body"]["kind"]["connections"][0]["target"]["boundary"] = serde_json::json!("left");
    graph["body"]["kind"]["connections"][2]["target"]["instance"] = serde_json::json!("a");
    graph["body"]["kind"]["connections"][2]["target"]["boundary"] = serde_json::json!("in");
    assert!(
        compile(
            &graph,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .is_err(),
        "cycle/multiple-input must reject"
    );
}

#[test]
fn incompatible_wire_type_is_denied() {
    let registry = fixture_registry();
    let mut mismatch = registry.clone();
    mismatch
        .operations
        .iter_mut()
        .find(|sig| sig.operation == "pass")
        .required("pass")
        .inputs[0]
        .value_type = "Linear".into();
    rejected(
        &diamond(),
        &mismatch,
        crate::construction::Failure::IncompatibleType,
    );
}

#[test]
fn registry_traits_and_effect_rule_are_host_selected() {
    let mut registry = fixture_registry();
    registry.types.retain(|ty| ty.name != "Unit");
    rejected(
        &diamond(),
        &registry,
        crate::construction::Failure::InvalidRegistry,
    );
    let baseline = admitted(&diamond(), &fixture_registry());
    let mut registry = fixture_registry();
    registry.implementation = "other-implementation".into();
    let changed = admitted(&diamond(), &registry);
    assert_ne!(baseline.binding, changed.binding);
}

#[test]
fn copy_traits_and_linear_drop_are_host_selected() {
    let mut registry = fixture_registry();
    registry
        .types
        .iter_mut()
        .find(|ty| ty.name == "Text")
        .required("Text")
        .copyable = false;
    let mut duplicated = diamond();
    duplicated["body"]["kind"]["connections"][1]["source"] =
        duplicated["body"]["kind"]["connections"][0]["source"].clone();
    assert_eq!(
        compile(
            &duplicated,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::UsageViolation)
    );
    let mut dropped = make_candidate(wire(vec![operation(("source", "linear-source"))], vec![]));
    assert_eq!(
        compile(
            &dropped,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::UsageViolation)
    );
    dropped["body"]["kind"]["parts"]
        .as_array_mut()
        .required("parts")
        .push(operation(("sink", "linear-sink")));
    dropped["body"]["kind"]["connections"] =
        serde_json::json!([connection(("source", "out"), ("sink", "in"))]);
    admitted(&dropped, &registry);
}

#[test]
fn effect_rules_are_host_selected() {
    let mut registry = fixture_registry();
    registry
        .operations
        .iter_mut()
        .find(|sig| sig.operation == "tick")
        .required("tick")
        .effects = vec![rule("machine")];
    rejected(
        &make_candidate(operation(("tick", "tick"))),
        &registry,
        crate::construction::Failure::InvalidEffect,
    );
}
#[test]
fn author_copy_and_purity_labels_cannot_widen_registry_rules() {
    let mut graph = diamond();
    graph["body"]["kind"]["parts"][0]["kind"]["copyable"] = serde_json::json!(true);
    rejected(
        &graph,
        &fixture_registry(),
        crate::construction::Failure::InvalidJson,
    );
    let mut graph = make_candidate(operation(("tick", "tick")));
    graph["body"]["kind"]["pure"] = serde_json::json!(true);
    rejected(
        &graph,
        &fixture_registry(),
        crate::construction::Failure::InvalidJson,
    );
}
