#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{
    dependency_ready_spec, frontier_is_admitted_order_filter_spec,
};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. A reversed two-node frontier is not the admitted-order filter.
pub proof fn readiness_reverses_order_must_fail(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    requires
        order.len() == 2,
        order[0] != order[1],
        dependency_ready_spec(dependencies, states, order[0]),
        dependency_ready_spec(dependencies, states, order[1]),
    ensures frontier_is_admitted_order_filter_spec(
        order,
        dependencies,
        states,
        seq![order[1], order[0]],
    ),
{}

} // verus!
