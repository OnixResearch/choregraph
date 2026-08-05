#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{
    ACTION_SUSPEND, IndexedSuspensionFact, PROFILE_DURABLE_SUSPENSION_V1, STATE_READY,
    indexed_step_accepts_spec,
};

verus! {

// Failure-path evidence only. Real Verus must reject a suspend transition
// that carries a product effect intent.
pub proof fn suspend_with_effect_intent(
    dependencies: Seq<Vec<usize>>,
    fact: IndexedSuspensionFact,
)
    requires
        dependencies.len() == 1,
        dependencies[0]@.len() == 0,
    ensures indexed_step_accepts_spec(
        dependencies,
        seq![STATE_READY],
        seq![None],
        PROFILE_DURABLE_SUSPENSION_V1,
        0usize,
        ACTION_SUSPEND,
        Some(fact),
        seq![0usize],
        seq![0usize],
    )
{}

} // verus!
