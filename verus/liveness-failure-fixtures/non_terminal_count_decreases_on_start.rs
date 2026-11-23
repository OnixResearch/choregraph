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

proof fn start_falsely_decreases_non_terminal_count(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
)
    requires
        verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            verified::codes::action::ACTION_START,
            None,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
    ensures
        verified_liveness::non_terminal_count_spec(
            verified::kernel::decision::indexed_step_next_spec(
                dependencies,
                states,
                suspensions,
                profile,
                node_index,
                verified::codes::action::ACTION_START,
                None,
                Seq::<usize>::empty(),
                Seq::<usize>::empty(),
            ),
        ) < verified_liveness::non_terminal_count_spec(states),
{}

} // verus!
