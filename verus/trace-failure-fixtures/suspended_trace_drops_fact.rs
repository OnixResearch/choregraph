#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

verus! {

// Failure-path evidence only. Real Verus must reject a replay that drops the
// suspension fact recorded by an accepted suspend event.
proof fn suspended_trace_keeps_stale_suspensions(
    dependencies: Seq<Vec<usize>>,
    event: verified_trace::IndexedReplayEvent,
)
    requires
        dependencies.len() == 1,
        dependencies[0]@.len() == 0,
        event.node_index == 0,
        event.action == verified::ACTION_SUSPEND,
        event.event_fact.is_some(),
        event.allowed_intents@.len() == 0,
        event.requested_intents@.len() == 0,
    ensures verified_trace::replay_final_suspensions_from_spec(
        dependencies,
        seq![verified::STATE_READY],
        seq![None],
        verified::PROFILE_DURABLE_SUSPENSION_V1,
        seq![event],
        0,
    ) == seq![None]
{}

} // verus!
