use super::*;

#[test]
fn persisted_projection_requires_current_registry_and_exact_typed_ports() {
    let registry = fixture_registry();
    let compiled = admitted(&nested(), &registry);
    let serialized = serde_json::to_vec(&compiled.projection).required("projection JSON");
    let admitted_again = crate::construction::admission::lower_projection_json(
        &serialized,
        &registry,
        crate::construction::model::profile::Limits::default(),
    )
    .required("native readmission");
    assert_eq!(admitted_again.projection, compiled.projection);
    assert_eq!(admitted_again.machine, compiled.machine);
    assert_eq!(admitted_again.artifact, compiled.artifact);
    let mut changed_registry = registry.clone();
    changed_registry
        .operations
        .iter_mut()
        .find(|signature| signature.operation == "pass")
        .required("pass")
        .implementation = "changed-pass".into();
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &serialized,
            &changed_registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidRegistry)
    );
    let mut changed: serde_json::Value =
        serde_json::from_slice(&serialized).required("projection JSON");
    changed["nodes"][0]["ports"][0]["value_type"] = serde_json::json!("Linear");
    let changed = serde_json::to_vec(&changed).required("changed projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &changed,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::IncompatibleType)
    );
    let mut changed_interface = compiled.projection.clone();
    let output_boundary = changed_interface
        .interfaces
        .iter_mut()
        .flat_map(|interface| &mut interface.outputs)
        .find(|boundary| !boundary.ports.is_empty())
        .required("public output mapping");
    output_boundary.ports[0] = u64::MAX;
    let changed_interface = serde_json::to_vec(&changed_interface).required("changed interface");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &changed_interface,
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::UnknownBoundary)
    );
    let unit = admitted(
        &make_candidate(
            serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [operation(("a", "tick")), operation(("b", "tick"))]}}),
        ),
        &registry,
    );
    let serialized_unit = serde_json::to_vec(&unit.projection).required("Unit projection");
    let readmitted = crate::construction::admission::lower_projection_json(
        &serialized_unit,
        &registry,
        crate::construction::model::profile::Limits::default(),
    )
    .required("Unit readmission");
    assert_eq!(readmitted.machine.edges().len(), 1);
}

#[test]
fn projection_preflight_admits_exact_collection_boundaries_and_rejects_excess() {
    let mut registry = fixture_registry();
    registry
        .operations
        .retain(|signature| matches!(signature.operation.as_str(), "fork" | "pass" | "join"));
    let projection = admitted(&nested(), &registry).projection;
    let ports: usize = projection.nodes.iter().map(|node| node.ports.len()).sum();
    let invocation: usize = projection
        .nodes
        .iter()
        .map(|node| node.invocation.len())
        .sum();
    let boundaries: usize = projection
        .interfaces
        .iter()
        .map(|interface| interface.inputs.len() + interface.outputs.len())
        .sum();
    let mapped: usize = projection
        .interfaces
        .iter()
        .flat_map(|interface| interface.inputs.iter().chain(&interface.outputs))
        .map(|boundary| boundary.ports.len())
        .sum();
    let limits = crate::construction::model::profile::Limits {
        max_nodes: projection.nodes.len().max(1),
        max_ports: ports.max(1),
        max_edges: (projection.edges.len() + projection.control_edges.len()).max(1),
        max_effects: projection.effects.len().max(1),
        max_instances: projection.interfaces.len().max(1),
        max_boundaries: invocation.max(boundaries).max(mapped).max(1),
        ..crate::construction::model::profile::Limits::default()
    };
    let valid = serde_json::to_vec(&projection).required("at-boundary projection");
    assert_eq!(
        crate::construction::admission::lower_projection_json(&valid, &registry, limits)
            .required("at-boundary readmission")
            .projection,
        projection
    );

    let original = serde_json::to_value(&projection).required("projection value");
    for field in [
        "nodes",
        "ports",
        "invocation",
        "edges",
        "control_edges",
        "effects",
        "interfaces",
        "boundaries",
        "mapped",
    ] {
        let mut changed = original.clone();
        let array = match field {
            "nodes" | "edges" | "control_edges" | "effects" | "interfaces" => changed[field]
                .as_array_mut()
                .required("top-level collection"),
            "ports" | "invocation" => changed["nodes"][0][field]
                .as_array_mut()
                .required("node collection"),
            "boundaries" => changed["interfaces"][0]["inputs"]
                .as_array_mut()
                .required("interface boundaries"),
            "mapped" => {
                let interface = &mut changed["interfaces"][0];
                let group = if !interface["inputs"].as_array().required("inputs").is_empty() {
                    "inputs"
                } else {
                    "outputs"
                };
                interface[group][0]["ports"]
                    .as_array_mut()
                    .required("mapped ports")
            }
            _ => unreachable!(),
        };
        // The invalid element proves the resource ceiling is seen before typed DTO parsing.
        let ceiling = match field {
            "nodes" => limits.max_nodes,
            "ports" => limits.max_ports,
            "invocation" | "boundaries" | "mapped" => limits.max_boundaries,
            "edges" | "control_edges" => limits.max_edges,
            "effects" => limits.max_effects,
            "interfaces" => limits.max_instances,
            _ => unreachable!(),
        };
        for _ in 0..=ceiling {
            array.push(serde_json::json!({}));
        }
        let bytes = serde_json::to_vec(&changed).required("over-budget projection");
        assert_eq!(
            crate::construction::admission::lower_projection_json(&bytes, &registry, limits).err(),
            Some(crate::construction::Failure::LimitExceeded),
            "{field}"
        );
    }
}

#[test]
fn projection_preflight_bounds_expression_width_and_depth_without_changing_json_keys() {
    let mut registry = fixture_registry();
    let configuration = std::mem::take(
        &mut registry
            .operations
            .iter_mut()
            .find(|signature| signature.operation == "pass")
            .required("pass signature")
            .configuration,
    );
    registry
        .operations
        .retain(|signature| signature.operation == "tick");
    registry
        .types
        .retain(|value_type| value_type.name == "Unit");
    registry.operations[0].configuration = configuration;
    let mut body = operation(("a", "tick"));
    body["kind"]["configuration"] =
        serde_json::json!({"label": {"tag": "literal", "value": "example"}});
    let projection = admitted(&make_candidate(body), &registry).projection;
    let mut changed = serde_json::to_value(&projection).required("projection value");
    changed["nodes"][0]["configuration"]["label"] = serde_json::json!({"tag": "apply", "operator": "concat", "arguments": [{"tag": "literal", "value": "first"}, null]});
    let bytes = serde_json::to_vec(&changed).required("wide configuration");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits {
                max_boundaries: 1,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
    changed["nodes"][0]["configuration"]["label"] = serde_json::json!({
        "tag": "apply", "operator": "concat", "arguments": [{
            "tag": "apply", "operator": "concat", "arguments": [null]
        }]
    });
    let bytes = serde_json::to_vec(&changed).required("deep configuration");
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits {
                max_depth: 1,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );

    let serialized = String::from_utf8(serde_json::to_vec(&projection).required("projection JSON"))
        .required("UTF-8 projection");
    let escaped = serialized.replacen("\"nodes\"", "\"\\u006eodes\"", 1);
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            escaped.as_bytes(),
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .required("escaped known key")
        .projection,
        projection
    );
    let duplicate = serialized.replacen("\"nodes\":", "\"nodes\":[],\"nodes\":", 1);
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            duplicate.as_bytes(),
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidJson)
    );
    let unknown = serialized.replacen("\"nodes\":", "\"unexpected\":[],\"nodes\":", 1);
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            unknown.as_bytes(),
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidJson)
    );
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            b"[]",
            &registry,
            crate::construction::model::profile::Limits::default()
        )
        .err(),
        Some(crate::construction::Failure::InvalidJson)
    );
    // A configuration is a map: repeated (including escaped) names replace an
    // entry, unlike duplicate fields of the closed projection DTO.
    let extra = r#""\u006cabel":{"tag":"literal","value":"example"},"#;
    let duplicated_configuration = serialized.replacen(
        "\"configuration\":{",
        &format!("\"configuration\":{{{}", extra.repeat(3)),
        1,
    );
    let map_limits = crate::construction::model::profile::Limits {
        max_boundaries: 1,
        ..crate::construction::model::profile::Limits::default()
    };
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            duplicated_configuration.as_bytes(),
            &registry,
            map_limits
        )
        .required("repeated configuration name")
        .projection,
        projection
    );
}

#[test]
fn large_projection_collections_stop_at_selected_node_and_port_ceilings() {
    const EXCESS_ENTRIES: usize = 20_000;
    let registry = fixture_registry();
    let projection = admitted(&diamond(), &registry).projection;
    let mut changed = serde_json::to_value(&projection).required("projection value");
    changed["nodes"]
        .as_array_mut()
        .required("nodes")
        .extend(std::iter::repeat_n(serde_json::Value::Null, EXCESS_ENTRIES));
    let bytes = serde_json::to_vec(&changed).required("many-node projection");
    assert!(bytes.len() < crate::construction::model::profile::Limits::default().max_output_bytes);
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits {
                max_nodes: 1,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );

    let mut changed = serde_json::to_value(&projection).required("projection value");
    let port_count = projection.nodes[0].ports.len();
    changed["nodes"][0]["ports"]
        .as_array_mut()
        .required("ports")
        .extend(std::iter::repeat_n(serde_json::Value::Null, EXCESS_ENTRIES));
    let bytes = serde_json::to_vec(&changed).required("many-port projection");
    assert!(bytes.len() < crate::construction::model::profile::Limits::default().max_output_bytes);
    assert_eq!(
        crate::construction::admission::lower_projection_json(
            &bytes,
            &registry,
            crate::construction::model::profile::Limits {
                max_ports: port_count,
                ..crate::construction::model::profile::Limits::default()
            }
        )
        .err(),
        Some(crate::construction::Failure::LimitExceeded)
    );
}
