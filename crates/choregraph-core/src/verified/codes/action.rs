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

pub const ACTION_MARK_READY: u8 = 0u8;

pub const ACTION_START: u8 = 1u8;

pub const ACTION_COMPLETE: u8 = 2u8;

pub const ACTION_FAIL: u8 = 3u8;

pub const ACTION_CANCEL: u8 = 4u8;

pub const ACTION_SUSPEND: u8 = 5u8;

pub const ACTION_RESUME_SIGNAL: u8 = 6u8;

pub open spec fn action_valid_spec(action: u8) -> bool {
    action == ACTION_MARK_READY
        || action == ACTION_START
        || action == ACTION_COMPLETE
        || action == ACTION_FAIL
        || action == ACTION_CANCEL
        || action == ACTION_SUSPEND
        || action == ACTION_RESUME_SIGNAL
}

pub open spec fn suspension_action_spec(action: u8) -> bool {
    action == ACTION_SUSPEND || action == ACTION_RESUME_SIGNAL
}

} // verus!
