#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{
    ACTION_START, PROFILE_DURABLE_SUSPENSION_V1, STATE_SUSPENDED, lifecycle_target_spec,
};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. Real Verus must reject a suspended node
// becoming executable without a resume signal.
pub proof fn suspended_node_starts_without_signal()
    ensures lifecycle_target_spec(
        PROFILE_DURABLE_SUSPENSION_V1,
        STATE_SUSPENDED,
        ACTION_START,
    ).is_some()
{}

} // verus!
