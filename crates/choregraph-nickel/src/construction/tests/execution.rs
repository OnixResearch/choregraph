use super::*;

fn complete_native(
    machine: &choregraph_core::model::automaton::AdmittedMachine,
    mut state: choregraph_core::model::automaton::MachineState,
    node: &str,
) -> choregraph_core::model::automaton::MachineState {
    let node_id = machine
        .node_ids()
        .iter()
        .find(|id| id.as_str() == node)
        .required("native fixture node");
    for (action, expected) in [
        (
            choregraph_core::model::semantics::Action::MarkReady,
            choregraph_core::model::semantics::LocalState::Ready,
        ),
        (
            choregraph_core::model::semantics::Action::Start,
            choregraph_core::model::semantics::LocalState::Running,
        ),
        (
            choregraph_core::model::semantics::Action::Complete,
            choregraph_core::model::semantics::LocalState::Completed,
        ),
    ] {
        match choregraph_core::transition::step_concrete(
            machine,
            &state,
            &choregraph_core::concrete::event::ConcreteEvent::new(node, action),
        ) {
            choregraph_core::concrete::event::ConcreteDecision::Accept { next, intents } => {
                assert_eq!(next.state(node_id), Some(expected));
                assert!(intents.is_empty());
                state = next;
            }
            decision => panic!("native construction lifecycle denied: {decision:?}"),
        }
    }
    state
}

#[test]
fn unit_sequence_and_generated_fork_join_use_production_readiness() {
    let graph = make_candidate(
        serde_json::json!({"key":"root","kind":{"tag":"sequence","parts":[operation(("a","tick")),operation(("b","tick"))]}}),
    );
    let output = admitted(&graph, &fixture_registry());
    let edge = &output.projection.control_edges[0];
    let first = format!("lattice-node/{}", edge.source);
    let second = format!("lattice-node/{}", edge.target);
    let initial = output.machine.initial_state();
    let ready = choregraph_core::analytics::ready_nodes(&output.machine, &initial)
        .required("Unit initial readiness");
    assert_eq!(
        ready.iter().map(|node| node.as_str()).collect::<Vec<_>>(),
        [first.as_str()]
    );
    assert!(matches!(
        choregraph_core::transition::step_concrete(
            &output.machine,
            &initial,
            &choregraph_core::concrete::event::ConcreteEvent::new(
                &second,
                choregraph_core::model::semantics::Action::MarkReady
            )
        ),
        choregraph_core::concrete::event::ConcreteDecision::Deny { .. }
    ));
    let completed = complete_native(&output.machine, initial, &first);
    let ready = choregraph_core::analytics::ready_nodes(&output.machine, &completed)
        .required("Unit successor readiness");
    assert_eq!(
        ready.iter().map(|node| node.as_str()).collect::<Vec<_>>(),
        [second.as_str()]
    );

    let output = admitted(&diamond(), &fixture_registry());
    let initial = output.machine.initial_state();
    let fork = output
        .projection
        .nodes
        .iter()
        .find(|node| node.operation == "fork")
        .required("fork");
    let join = output
        .projection
        .nodes
        .iter()
        .find(|node| node.operation == "join")
        .required("join");
    assert!(matches!(
        choregraph_core::transition::step_concrete(
            &output.machine,
            &initial,
            &choregraph_core::concrete::event::ConcreteEvent::new(
                format!("lattice-node/{}", join.id),
                choregraph_core::model::semantics::Action::MarkReady
            )
        ),
        choregraph_core::concrete::event::ConcreteDecision::Deny { .. }
    ));
    let mut state = complete_native(
        &output.machine,
        initial,
        &format!("lattice-node/{}", fork.id),
    );
    let branches = choregraph_core::analytics::ready_nodes(&output.machine, &state)
        .required("generated branches");
    assert_eq!(branches.len(), 2);
    for branch in branches {
        state = complete_native(&output.machine, state, branch.as_str());
    }
    let ready =
        choregraph_core::analytics::ready_nodes(&output.machine, &state).required("generated join");
    assert_eq!(
        ready.iter().map(|node| node.as_str()).collect::<Vec<_>>(),
        [format!("lattice-node/{}", join.id)]
    );
}
