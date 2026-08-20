#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{dependency_ready_spec, ready_frontier_spec};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. The production frontier cannot omit an eligible node.
pub proof fn readiness_omits_eligible_must_fail(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    position: int,
)
    requires
        0 <= position < order.len(),
        dependency_ready_spec(dependencies, states, order[position]),
    ensures !ready_frontier_spec(order, dependencies, states).contains(order[position]),
{}

} // verus!
