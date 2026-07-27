#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

verus! {

proof fn first_denial_is_reported_after_its_event(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<verified_trace::IndexedReplayEvent>,
)
    requires
        events.len() > 0,
        !verified_trace::replay_event_accepts_spec(
            dependencies,
            states,
            events[0],
        ),
    ensures verified_trace::replay_failure_index_from_spec(
        dependencies,
        states,
        events,
        0,
    ) == Some(1)
{}

} // verus!
