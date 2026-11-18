use super::*;

#[test]
fn nested_instances_expand_twice_with_unique_scopes_and_exact_ordered_routing() {
    let registry = fixture_registry();
    let output = admitted(&nested(), &registry);
    assert_eq!(
        output.projection.schema,
        crate::construction::model::profile::PROJECTION_SCHEMA
    );
    assert_eq!(output.projection.nodes.len(), 4);
    assert_eq!(output.projection.edges.len(), 4);
    assert_eq!(output.projection.control_edges.len(), 0);
    let pass_nodes: Vec<_> = output
        .projection
        .nodes
        .iter()
        .filter(|node| node.operation == "pass")
        .collect();
    assert_eq!(pass_nodes.len(), 2);
    assert_ne!(pass_nodes[0].id, pass_nodes[1].id);
    assert_ne!(pass_nodes[0].scoped_id, pass_nodes[1].scoped_id);
    assert!(
        output
            .origins
            .iter()
            .any(|origin| origin.instance_path.iter().any(|part| part == "a"))
    );
    assert!(
        output
            .origins
            .iter()
            .any(|origin| origin.instance_path.iter().any(|part| part == "b"))
    );
    assert_eq!(output.machine.edges().len(), 4);
}

#[test]
fn shared_handle_is_one_instance_with_two_occurrences_and_linear_fanout_is_denied() {
    let mut graph = diamond();
    graph["body"]["kind"]["connections"] = serde_json::json!([
        connection(("fork", "left"), ("a", "in")),
        connection(("fork", "left"), ("b", "in")),
        connection(("a", "out"), ("join", "left")),
        connection(("b", "out"), ("join", "right")),
    ]);
    let output = admitted(&graph, &fixture_registry());
    assert_eq!(output.projection.nodes.len(), 4);
    assert_eq!(output.projection.edges.len(), 4);
    let mut linear = fixture_registry();
    linear
        .operations
        .iter_mut()
        .find(|signature| signature.operation == "fork")
        .required("fork")
        .outputs[0]
        .value_type = "Linear".into();
    for signature in linear
        .operations
        .iter_mut()
        .filter(|signature| signature.operation == "pass")
    {
        signature.inputs[0].value_type = "Linear".into();
    }
    rejected(
        &graph,
        &linear,
        crate::construction::Failure::UsageViolation,
    );
}

#[test]
fn ordered_sequence_and_parallel_preserve_boundary_order_and_unit_control() {
    let sequence = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [
            operation(("first", "tick")),
            {"key": "identity", "kind": {"tag": "identity", "boundaries": []}},
            operation(("second", "tick")), operation(("third", "tick"))
        ]}}),
    );
    let output = admitted(&sequence, &fixture_registry());
    assert_eq!(output.projection.nodes.len(), 3);
    assert_eq!(output.projection.edges.len(), 0);
    assert_eq!(output.projection.control_edges.len(), 2);
    assert_eq!(output.machine.edges().len(), 2);
    let parallel = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "parallel", "parts": [
            operation(("a", "tick")), operation(("b", "tick"))
        ]}}),
    );
    assert_eq!(
        admitted(&parallel, &fixture_registry())
            .projection
            .control_edges
            .len(),
        0
    );
    let reversed = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [
            operation(("third", "tick")), operation(("second", "tick")), operation(("first", "tick"))
        ]}}),
    );
    assert_ne!(
        output.projection.control_edges,
        admitted(&reversed, &fixture_registry())
            .projection
            .control_edges
    );
}

#[test]
fn ordered_parallel_boundaries_change_positional_sequence_routing() {
    let middle = serde_json::json!({"key": "branch", "kind": {"tag": "parallel", "parts": [
        operation(("a", "pass")), operation(("b", "pass"))
    ]}});
    let graph = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [
            operation(("fork", "fork")), middle, operation(("join", "join"))
        ]}}),
    );
    let first = admitted(&graph, &fixture_registry());
    assert_eq!(first.projection.edges.len(), 4);
    let first_branch = first
        .projection
        .interfaces
        .iter()
        .find(|interface| interface.inputs.len() == 2 && interface.outputs.len() == 2)
        .required("parallel branch interface");
    assert_eq!(
        first_branch
            .inputs
            .iter()
            .map(|port| port.name.as_str())
            .collect::<Vec<_>>(),
        ["a/in", "b/in"]
    );
    assert_eq!(
        first_branch
            .outputs
            .iter()
            .map(|port| port.name.as_str())
            .collect::<Vec<_>>(),
        ["a/out", "b/out"]
    );
    let mut swapped = graph;
    swapped["body"]["kind"]["parts"][1]["kind"]["parts"]
        .as_array_mut()
        .required("parallel parts")
        .swap(0, 1);
    let second = admitted(&swapped, &fixture_registry());
    assert_eq!(second.projection.edges.len(), 4);
    let second_branch = second
        .projection
        .interfaces
        .iter()
        .find(|interface| interface.inputs.len() == 2 && interface.outputs.len() == 2)
        .required("swapped branch interface");
    assert_eq!(
        second_branch
            .inputs
            .iter()
            .map(|port| port.name.as_str())
            .collect::<Vec<_>>(),
        ["b/in", "a/in"]
    );
    assert_ne!(
        first.projection.edges, second.projection.edges,
        "ordered ports change dependencies"
    );
}
#[test]
fn structural_identity_keeps_exact_routing_and_required_dependencies() {
    let branch = serde_json::json!({"key": "branch", "kind": {"tag": "parallel", "parts": [
        operation(("a", "pass")), operation(("b", "pass"))
    ]}});
    let base = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [
            operation(("fork", "fork")), branch, operation(("join", "join"))
        ]}}),
    );
    let mut with_identity = base.clone();
    with_identity["body"]["kind"]["parts"]
        .as_array_mut()
        .required("sequence parts")
        .insert(
            1,
            serde_json::json!({"key": "identity", "kind": {"tag": "identity", "boundaries": [
                {"name": "first", "value_type": "Text"}, {"name": "second", "value_type": "Text"}
            ]}}),
        );
    let direct = admitted(&base, &fixture_registry());
    let neutral = admitted(&with_identity, &fixture_registry());
    assert_eq!(direct.projection.nodes, neutral.projection.nodes);
    assert_eq!(direct.projection.edges, neutral.projection.edges);
    assert_eq!(
        direct.projection.control_edges,
        neutral.projection.control_edges
    );
    assert_eq!(direct.projection.effects, neutral.projection.effects);
    assert_eq!(direct.machine, neutral.machine);
    assert_ne!(
        direct.binding, neutral.binding,
        "source and origins remain exactly bound"
    );
}
#[test]
fn inverse_public_boundary_permutation_is_not_identity() {
    let branch = serde_json::json!({"key": "branch", "kind": {"tag": "parallel", "parts": [
        operation(("a", "pass")), operation(("b", "pass"))
    ]}});
    let straight = make_candidate(
        serde_json::json!({"key": "root", "kind": {"tag": "sequence", "parts": [
            operation(("fork", "fork")), branch, operation(("join", "join"))
        ]}}),
    );
    let mut inverted = straight.clone();
    let swap = serde_json::json!({"key": "swap", "kind": {"tag": "wire", "parts": [
        {"key": "left", "kind": {"tag": "identity", "boundaries": [{"name": "v", "value_type": "Text"}]}},
        {"key": "right", "kind": {"tag": "identity", "boundaries": [{"name": "v", "value_type": "Text"}]}}
    ], "connections": [],
    "inputs": [
        {"name": "left", "handle": handle(("left", "input", "v"))},
        {"name": "right", "handle": handle(("right", "input", "v"))}
    ], "outputs": [
        {"name": "right", "handle": handle(("right", "output", "v"))},
        {"name": "left", "handle": handle(("left", "output", "v"))}
    ]}});
    inverted["body"]["kind"]["parts"]
        .as_array_mut()
        .required("sequence parts")
        .insert(1, swap);
    let direct = admitted(&straight, &fixture_registry());
    let reversed = admitted(&inverted, &fixture_registry());
    assert_ne!(direct.projection.edges, reversed.projection.edges);
    let swap_interface = reversed
        .projection
        .interfaces
        .iter()
        .find(|interface| {
            interface
                .inputs
                .iter()
                .map(|boundary| boundary.name.as_str())
                .eq(["left", "right"])
                && interface.outputs.len() == 2
        })
        .required("swap public interface");
    assert_eq!(
        swap_interface
            .outputs
            .iter()
            .map(|boundary| boundary.name.as_str())
            .collect::<Vec<_>>(),
        ["right", "left"]
    );
    let mut restored = inverted.clone();
    let mut inverse = restored["body"]["kind"]["parts"][1].clone();
    inverse["key"] = serde_json::json!("unswap");
    restored["body"]["kind"]["parts"]
        .as_array_mut()
        .required("sequence parts")
        .insert(2, inverse);
    let restored = admitted(&restored, &fixture_registry());
    assert_eq!(
        direct.projection.edges, restored.projection.edges,
        "a permutation followed by its inverse restores exact selected routing"
    );
    assert_eq!(direct.machine, restored.machine);
}
