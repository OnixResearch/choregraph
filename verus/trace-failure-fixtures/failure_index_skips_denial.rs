#![no_std]

extern crate alloc;

use alloc::vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

verus! {

proof fn first_denial_is_reported_after_its_event()
    ensures ({
        let dependencies = seq![vec![]];
        let states = seq![verified::STATE_PENDING];
        let events = seq![verified_trace::IndexedReplayEvent {
            node_index: 0,
            action: verified::ACTION_COMPLETE,
            allowed_intents: vec![],
            requested_intents: vec![],
        }];
        verified_trace::replay_failure_index_from_spec(
            dependencies,
            states,
            events,
            0,
        ) == Some(1)
    })
{}

} // verus!
