#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use verified::{
    codes::{action::ACTION_CANCEL, profile::PROFILE_LINEAR_V1},
    kernel::decision::{StepOutcome, indexed_decision_view, indexed_step},
};
use vstd::prelude::*;

verus! {

pub const FORBIDDEN_DENIAL_INTENT_COUNT: usize = 1usize;

// Failure-path evidence only. Real Verus must reject denial output data.
pub fn denial_with_intent_must_fail() -> (result: StepOutcome)
    ensures
        !indexed_decision_view(result).0,
        indexed_decision_view(result).3.len() == FORBIDDEN_DENIAL_INTENT_COUNT,
{
    indexed_step(&[], &[], &[], PROFILE_LINEAR_V1, 0, ACTION_CANCEL, &None, &[], &[])
}

} // verus!
