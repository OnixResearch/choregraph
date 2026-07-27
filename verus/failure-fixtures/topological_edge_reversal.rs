#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified/topology.rs"]
mod production_topology;

use production_topology::topological_order_spec;
use vstd::prelude::*;

verus! {

pub const SOURCE_NODE: usize = 0usize;
pub const TARGET_NODE: usize = 1usize;
pub const NODE_COUNT: usize = 2usize;

// Failure-path evidence only. Real Verus must reject a reversed edge order.
pub proof fn topological_edge_reversal_must_fail()
    ensures topological_order_spec(
        seq![(SOURCE_NODE, TARGET_NODE)],
        NODE_COUNT,
        seq![TARGET_NODE, SOURCE_NODE],
    ),
{}

} // verus!
