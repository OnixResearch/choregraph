#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use verified::{primitive::ownership::EPOCH_FENCE_ADMIT, primitive::ownership::epoch_fence_decision_spec, primitive::ownership::EpochFenceFacts};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. A stale epoch must not admit.
pub proof fn stale_epoch_admits()
    ensures epoch_fence_decision_spec(EpochFenceFacts { entity_known: true, entity_matches: true, current_epoch: 2u64, offered_epoch: 1u64, terminal: false }) == EPOCH_FENCE_ADMIT,
{}

} // verus!
