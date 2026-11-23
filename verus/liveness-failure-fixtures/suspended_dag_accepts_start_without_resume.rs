#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use vstd::prelude::*;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/topology.rs"]
mod verified_topology;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

#[path = "../proofs/liveness_lemmas.rs"]
mod verified_liveness;

verus! {

proof fn suspended_node_falsely_accepts_start_without_resume(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<verified::primitive::support::IndexedSuspensionFact>>,
    node_index: usize,
)
    requires
        verified::primitive::validity::indexed_invariant_spec(dependencies, states),
        verified::primitive::support::suspensions_consistent_spec(states, suspensions),
        node_index < states.len(),
        states[node_index as int] == verified::codes::status::STATE_SUSPENDED,
    ensures
        verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            verified::codes::profile::PROFILE_DURABLE_SUSPENSION_V1,
            node_index,
            verified::codes::action::ACTION_START,
            None,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
{}

} // verus!
