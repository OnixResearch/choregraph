#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{EPOCH_FENCE_ADMIT, epoch_fence_decision_spec};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. A cross-entity mutation must not admit.
pub proof fn cross_entity_admits()
    ensures epoch_fence_decision_spec(true, false, 1u64, 1u64, false) == EPOCH_FENCE_ADMIT,
{}

} // verus!
