// Verus proof syntax and production-linked specifications require these local source shapes.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    clippy::assign_op_pattern,
    dead_code,
    nested_conditionals,
    non_trait_imports,
    raw_arithmetic_overflow,
    too_many_parameters,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus proof syntax and production-linked proof witnesses require these local source shapes"
)]

use vstd::prelude::*;

verus! {

pub const STATE_PENDING: u8 = 0u8;

pub const STATE_READY: u8 = 1u8;

pub const STATE_RUNNING: u8 = 2u8;

pub const STATE_COMPLETED: u8 = 3u8;

pub const STATE_FAILED: u8 = 4u8;

pub const STATE_CANCELLED: u8 = 5u8;

pub const STATE_SUSPENDED: u8 = 6u8;

pub open spec fn local_state_valid_spec(state: u8) -> bool {
    state == STATE_PENDING
        || state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
        || state == STATE_CANCELLED
        || state == STATE_SUSPENDED
}

pub open spec fn terminal_state_spec(state: u8) -> bool {
    state == STATE_COMPLETED || state == STATE_FAILED || state == STATE_CANCELLED
}

pub fn local_state_valid(state: u8) -> (result: bool)
    ensures result == local_state_valid_spec(state)
{
    state == STATE_PENDING
        || state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
        || state == STATE_CANCELLED
        || state == STATE_SUSPENDED
}

pub fn terminal_state(state: u8) -> (result: bool)
    ensures result == terminal_state_spec(state)
{
    state == STATE_COMPLETED || state == STATE_FAILED || state == STATE_CANCELLED
}

} // verus!
