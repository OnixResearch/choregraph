use alloc::string::String;
use alloc::vec::Vec;

use crate::adapter;
use crate::structure::Topology;

pub fn direct_dependencies(graph: &Topology, node_id: &str) -> Vec<String> {
    match adapter::admit_machine(graph) {
        Ok(machine) => adapter::dependencies(&machine, node_id),
        Err(_) => Vec::new(),
    }
}

pub fn direct_dependents(graph: &Topology, node_id: &str) -> Vec<String> {
    match adapter::admit_machine(graph) {
        Ok(machine) => adapter::dependents(&machine, node_id),
        Err(_) => Vec::new(),
    }
}
pub fn would_create_cycle(graph: &Topology, source: &str, target: &str) -> bool {
    match adapter::admit_machine(graph) {
        Ok(machine) => adapter::would_create_cycle(&machine, source, target),
        Err(_) => false,
    }
}
