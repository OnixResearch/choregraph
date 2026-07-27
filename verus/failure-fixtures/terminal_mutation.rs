#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{
    terminal_states_preserved_spec, STATE_COMPLETED, STATE_RUNNING,
};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. Real Verus must reject terminal mutation.
pub proof fn terminal_mutation_must_fail()
    ensures terminal_states_preserved_spec(
        seq![STATE_COMPLETED],
        seq![STATE_RUNNING],
    ),
{}

} // verus!
