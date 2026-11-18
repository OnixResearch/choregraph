#![no_std]

// r[verify choregraph.epoch_fence.proof]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use verified::{primitive::ownership::EPOCH_FENCE_ADMIT, primitive::ownership::EPOCH_FENCE_ENTITY_MISMATCH, primitive::ownership::EPOCH_FENCE_EPOCH_MISMATCH, primitive::ownership::EPOCH_FENCE_TERMINAL_MUTATION, primitive::ownership::EPOCH_FENCE_UNKNOWN_ENTITY, primitive::ownership::epoch_fence_decision_spec, primitive::ownership::EpochFenceFacts};
use vstd::prelude::*;

verus! {

pub proof fn matching_epoch_for_known_nonterminal_entity_admits(epoch: u64)
    ensures epoch_fence_decision_spec(EpochFenceFacts { entity_known: true, entity_matches: true, current_epoch: epoch, offered_epoch: epoch, terminal: false }) == EPOCH_FENCE_ADMIT,
{}

pub proof fn stale_or_future_epoch_is_rejected(current: u64, offered: u64)
    requires current != offered,
    ensures epoch_fence_decision_spec(EpochFenceFacts { entity_known: true, entity_matches: true, current_epoch: current, offered_epoch: offered, terminal: false })
        == EPOCH_FENCE_EPOCH_MISMATCH,
{}

pub proof fn cross_entity_mutation_is_rejected(epoch: u64)
    ensures epoch_fence_decision_spec(EpochFenceFacts { entity_known: true, entity_matches: false, current_epoch: epoch, offered_epoch: epoch, terminal: false })
        == EPOCH_FENCE_ENTITY_MISMATCH,
{}

pub proof fn terminal_mutation_is_rejected(epoch: u64)
    ensures epoch_fence_decision_spec(EpochFenceFacts { entity_known: true, entity_matches: true, current_epoch: epoch, offered_epoch: epoch, terminal: true })
        == EPOCH_FENCE_TERMINAL_MUTATION,
{}

pub proof fn unknown_entity_is_rejected(epoch: u64)
    ensures epoch_fence_decision_spec(EpochFenceFacts { entity_known: false, entity_matches: true, current_epoch: epoch, offered_epoch: epoch, terminal: false })
        == EPOCH_FENCE_UNKNOWN_ENTITY,
{}

} // verus!
