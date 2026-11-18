#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use verified::{codes::action::ACTION_MARK_READY, primitive::support::IndexedSuspensionFact, readiness::dependency_ready_spec, kernel::decision::indexed_step_accepts_spec, kernel::theorem::mark_ready_context_spec};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. An otherwise valid eligible MarkReady event cannot be rejected.
pub proof fn readiness_rejects_eligible_mark_ready_must_fail(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
)
    requires
        mark_ready_context_spec(dependencies, states, suspensions, profile, node_index),
        dependency_ready_spec(dependencies, states, node_index),
    ensures !indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        ACTION_MARK_READY,
        None,
        Seq::<usize>::empty(),
        Seq::<usize>::empty(),
    ),
{}

} // verus!
