#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/topology.rs"]
mod verified_topology;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

#[path = "../proofs/liveness_lemmas.rs"]
mod verified_liveness;

verus! {

proof fn cyclic_pending_graph_falsely_has_ready_node(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    requires
        dependencies.len() == 2,
        states.len() == 2,
        dependencies[0].len() == 1 && dependencies[0][0] == 1usize,
        dependencies[1].len() == 1 && dependencies[1][0] == 0usize,
        states[0] == verified::codes::status::STATE_PENDING,
        states[1] == verified::codes::status::STATE_PENDING,
    ensures
        exists|node_index: usize|
            node_index < 2
                && verified::primitive::validity::dependencies_complete_spec(
                    (#[trigger] dependencies[node_index as int])@,
                    states,
                ),
{}

} // verus!
