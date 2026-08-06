#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

verus! {

proof fn replay_returns_dispatch_authority(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<verified::IndexedSuspensionFact>>,
    events: Seq<verified_trace::IndexedReplayEvent>,
)
    ensures verified_trace::replay_dispatch_intents_spec(
        dependencies,
        states,
        suspensions,
        verified::PROFILE_LINEAR_V1,
        events,
        0,
    ).len() > 0
{}

} // verus!
