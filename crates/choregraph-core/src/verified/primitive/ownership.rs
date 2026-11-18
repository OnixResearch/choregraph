// Verus proof syntax and production-linked specifications require these local source shapes.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    clippy::assign_op_pattern,
    dead_code,
    nested_conditionals,
    non_trait_imports,
    raw_arithmetic_overflow,
    too_many_parameters,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus proof syntax and production-linked proof witnesses require these local source shapes"
)]

use vstd::prelude::*;

verus! {

pub const EPOCH_FENCE_ADMIT: u8 = 0u8;

pub const EPOCH_FENCE_UNKNOWN_ENTITY: u8 = 1u8;

pub const EPOCH_FENCE_ENTITY_MISMATCH: u8 = 2u8;

pub const EPOCH_FENCE_EPOCH_MISMATCH: u8 = 3u8;

pub const EPOCH_FENCE_TERMINAL_MUTATION: u8 = 4u8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EpochFenceFacts {
    pub entity_known: bool,
    pub entity_matches: bool,
    pub current_epoch: u64,
    pub offered_epoch: u64,
    pub terminal: bool,
}

// r[impl choregraph.epoch_fence.proof]
// r[verify choregraph.epoch_fence.proof]
pub open spec fn epoch_fence_decision_spec(
    facts: EpochFenceFacts,
) -> u8 {
    if !facts.entity_known {
        EPOCH_FENCE_UNKNOWN_ENTITY
    } else if !facts.entity_matches {
        EPOCH_FENCE_ENTITY_MISMATCH
    } else if facts.current_epoch != facts.offered_epoch {
        EPOCH_FENCE_EPOCH_MISMATCH
    } else if facts.terminal {
        EPOCH_FENCE_TERMINAL_MUTATION
    } else {
        EPOCH_FENCE_ADMIT
    }
}

pub const fn epoch_fence_decision(
    facts: EpochFenceFacts,
) -> (result: u8)
    ensures result == epoch_fence_decision_spec(facts)
{
    if !facts.entity_known {
        EPOCH_FENCE_UNKNOWN_ENTITY
    } else if !facts.entity_matches {
        EPOCH_FENCE_ENTITY_MISMATCH
    } else if facts.current_epoch != facts.offered_epoch {
        EPOCH_FENCE_EPOCH_MISMATCH
    } else if facts.terminal {
        EPOCH_FENCE_TERMINAL_MUTATION
    } else {
        EPOCH_FENCE_ADMIT
    }
}

} // verus!
