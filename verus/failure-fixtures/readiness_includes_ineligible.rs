#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{dependency_ready_spec, ready_frontier_spec};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. The production frontier cannot add an ineligible node.
pub proof fn readiness_includes_ineligible_must_fail(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
)
    requires
        order.contains(node_index),
        !dependency_ready_spec(dependencies, states, node_index),
    ensures ready_frontier_spec(order, dependencies, states).contains(node_index),
{}

} // verus!
