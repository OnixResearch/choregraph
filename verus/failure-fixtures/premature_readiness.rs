#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use verified::{primitive::validity::dependencies_complete_spec, codes::status::STATE_COMPLETED, codes::status::STATE_RUNNING};
use vstd::prelude::*;

verus! {

// Failure-path evidence only. Real Verus must reject this false readiness claim.
pub proof fn premature_readiness_must_fail()
    ensures dependencies_complete_spec(seq![0usize], seq![STATE_RUNNING]),
{
    assert(STATE_RUNNING != STATE_COMPLETED);
}

} // verus!
