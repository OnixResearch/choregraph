#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

verus! {

proof fn empty_trace_mutates_state(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
)
    ensures verified_trace::specification::replay_final_states_from_spec(
        dependencies,
        states,
        suspensions,
        profile,
        Seq::<verified_trace::specification::IndexedReplayEvent>::empty(),
        0,
    ) != states
{}

} // verus!
