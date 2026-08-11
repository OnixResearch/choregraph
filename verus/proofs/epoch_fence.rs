#![no_std]

// r[verify choregraph.epoch_fence.proof]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{
    EPOCH_FENCE_ADMIT, EPOCH_FENCE_ENTITY_MISMATCH, EPOCH_FENCE_EPOCH_MISMATCH,
    EPOCH_FENCE_TERMINAL_MUTATION, EPOCH_FENCE_UNKNOWN_ENTITY, epoch_fence_decision_spec,
};
use vstd::prelude::*;

verus! {

pub proof fn matching_epoch_for_known_nonterminal_entity_admits(epoch: u64)
    ensures epoch_fence_decision_spec(true, true, epoch, epoch, false) == EPOCH_FENCE_ADMIT,
{}

pub proof fn stale_or_future_epoch_is_rejected(current: u64, offered: u64)
    requires current != offered,
    ensures epoch_fence_decision_spec(true, true, current, offered, false)
        == EPOCH_FENCE_EPOCH_MISMATCH,
{}

pub proof fn cross_entity_mutation_is_rejected(epoch: u64)
    ensures epoch_fence_decision_spec(true, false, epoch, epoch, false)
        == EPOCH_FENCE_ENTITY_MISMATCH,
{}

pub proof fn terminal_mutation_is_rejected(epoch: u64)
    ensures epoch_fence_decision_spec(true, true, epoch, epoch, true)
        == EPOCH_FENCE_TERMINAL_MUTATION,
{}

pub proof fn unknown_entity_is_rejected(epoch: u64)
    ensures epoch_fence_decision_spec(false, true, epoch, epoch, false)
        == EPOCH_FENCE_UNKNOWN_ENTITY,
{}

} // verus!
