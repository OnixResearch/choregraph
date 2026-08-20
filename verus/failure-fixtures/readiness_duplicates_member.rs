#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{pairwise_distinct_spec, ready_frontier_spec};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. Duplicating a production frontier member is not unique.
pub proof fn readiness_duplicates_member_must_fail(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    requires ready_frontier_spec(order, dependencies, states).len() > 0,
    ensures pairwise_distinct_spec(
        ready_frontier_spec(order, dependencies, states).push(
            ready_frontier_spec(order, dependencies, states)[0],
        ),
    ),
{}

} // verus!
