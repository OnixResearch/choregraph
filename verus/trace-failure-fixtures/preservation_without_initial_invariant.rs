#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

verus! {

proof fn replay_is_safe_without_an_initial_invariant(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    ensures verified::indexed_invariant_spec(
        dependencies,
        verified_trace::replay_final_states_from_spec(
            dependencies,
            states,
            Seq::<verified_trace::IndexedReplayEvent>::empty(),
            0,
        ),
    )
{}

} // verus!
