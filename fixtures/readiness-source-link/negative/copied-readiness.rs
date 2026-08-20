pub fn ready_nodes(machine: &AdmittedMachine, state: &MachineState) -> Vec<NodeId> {
    let mut ready = Vec::new();
    for node_id in &machine.topological_order {
        if state.state(node_id) == Some(LocalState::Pending)
            && dependencies_complete(machine, state, node_id)
        {
            ready.push(node_id.clone());
        }
    }
    ready
}

/// Returns one deterministic resume decision for an admitted node.
