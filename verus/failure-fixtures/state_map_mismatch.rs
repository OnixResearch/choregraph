#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use alloc::vec::Vec;
use verified::{primitive::validity::indexed_invariant_spec, codes::status::STATE_PENDING};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. Real Verus must reject unequal graph/state sizes.
pub proof fn state_map_mismatch_must_fail()
    ensures indexed_invariant_spec(
        Seq::<Vec<usize>>::empty(),
        seq![STATE_PENDING],
    ),
{}

} // verus!
