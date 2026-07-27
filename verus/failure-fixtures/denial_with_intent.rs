#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{deny, IndexedDecision, DENIAL_INVALID_STATE};
use vstd::prelude::*;

verus! {

pub const FORBIDDEN_DENIAL_INTENT_COUNT: usize = 1usize;

// Failure-path evidence only. Real Verus must reject denial output data.
pub fn denial_with_intent_must_fail() -> (result: IndexedDecision)
    ensures
        !result.accepted,
        result.intent_indices@.len() == FORBIDDEN_DENIAL_INTENT_COUNT,
{
    deny(DENIAL_INVALID_STATE)
}

} // verus!
