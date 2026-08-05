#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{
    ACTION_RESUME_SIGNAL, IndexedSuspensionFact, PROFILE_DURABLE_SUSPENSION_V1, STATE_SUSPENDED,
    indexed_step_accepts_spec,
};

verus! {

// Failure-path evidence only. Real Verus must reject a resume signal whose
// class or reference differs from the recorded suspension fact.
pub proof fn mismatched_resume_accepted(
    dependencies: Seq<Vec<usize>>,
    stored: IndexedSuspensionFact,
    event: IndexedSuspensionFact,
)
    requires
        dependencies.len() == 1,
        dependencies[0]@.len() == 0,
        stored.class@.len() == 1,
        event.class@.len() == 0,
    ensures indexed_step_accepts_spec(
        dependencies,
        seq![STATE_SUSPENDED],
        seq![Some(stored)],
        PROFILE_DURABLE_SUSPENSION_V1,
        0usize,
        ACTION_RESUME_SIGNAL,
        Some(event),
        Seq::<usize>::empty(),
        Seq::<usize>::empty(),
    )
{}

} // verus!
