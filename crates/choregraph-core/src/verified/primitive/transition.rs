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

use crate::verified::codes::action::*;
use crate::verified::codes::profile::*;
use crate::verified::codes::status::*;
use vstd::prelude::*;

verus! {

// This relation enumerates the permitted triples independently of executable dispatch.
pub open spec fn lifecycle_transition_spec(profile: u8, from: u8, action: u8, to: u8) -> bool {
    match (from, action, to) {
        (STATE_PENDING, ACTION_MARK_READY, STATE_READY)
        | (STATE_READY, ACTION_START, STATE_RUNNING)
        | (STATE_RUNNING, ACTION_COMPLETE, STATE_COMPLETED)
        | (STATE_RUNNING, ACTION_FAIL, STATE_FAILED)
        | (STATE_PENDING, ACTION_CANCEL, STATE_CANCELLED)
        | (STATE_READY, ACTION_CANCEL, STATE_CANCELLED)
        | (STATE_RUNNING, ACTION_CANCEL, STATE_CANCELLED) => profile_valid_spec(profile),
        (STATE_READY, ACTION_SUSPEND, STATE_SUSPENDED)
        | (STATE_RUNNING, ACTION_SUSPEND, STATE_SUSPENDED)
        | (STATE_SUSPENDED, ACTION_RESUME_SIGNAL, STATE_READY) => durable_profile_spec(profile),
        (STATE_SUSPENDED, ACTION_CANCEL, STATE_CANCELLED) => profile == PROFILE_DURABLE_SUSPENSION_V2,
        _ => false,
    }
}

pub open spec fn lifecycle_target_spec(profile: u8, from: u8, action: u8) -> Option<u8> {
    if lifecycle_transition_spec(profile, from, action, STATE_READY) {
        Some(STATE_READY)
    } else if lifecycle_transition_spec(profile, from, action, STATE_RUNNING) {
        Some(STATE_RUNNING)
    } else if lifecycle_transition_spec(profile, from, action, STATE_COMPLETED) {
        Some(STATE_COMPLETED)
    } else if lifecycle_transition_spec(profile, from, action, STATE_FAILED) {
        Some(STATE_FAILED)
    } else if lifecycle_transition_spec(profile, from, action, STATE_CANCELLED) {
        Some(STATE_CANCELLED)
    } else if lifecycle_transition_spec(profile, from, action, STATE_SUSPENDED) {
        Some(STATE_SUSPENDED)
    } else {
        None
    }
}

pub const fn lifecycle_target(profile: u8, from: u8, action: u8) -> (result: Option<u8>)
    ensures result == lifecycle_target_spec(profile, from, action)
{
    if !is_supported(profile) {
        return None;
    }
    match (from, action) {
        (STATE_PENDING, ACTION_MARK_READY) => Some(STATE_READY),
        (STATE_READY, ACTION_START) => Some(STATE_RUNNING),
        (STATE_RUNNING, ACTION_COMPLETE) => Some(STATE_COMPLETED),
        (STATE_RUNNING, ACTION_FAIL) => Some(STATE_FAILED),
        (STATE_PENDING, ACTION_CANCEL) => Some(STATE_CANCELLED),
        (STATE_READY | STATE_RUNNING, ACTION_CANCEL) => Some(STATE_CANCELLED),
        (STATE_READY, ACTION_SUSPEND) if has_suspension(profile) => Some(STATE_SUSPENDED),
        (STATE_RUNNING, ACTION_SUSPEND) if has_suspension(profile) => Some(STATE_SUSPENDED),
        (STATE_SUSPENDED, ACTION_RESUME_SIGNAL) if has_suspension(profile) => Some(STATE_READY),
        (STATE_SUSPENDED, ACTION_CANCEL) if profile == PROFILE_DURABLE_SUSPENSION_V2 => Some(STATE_CANCELLED),
        _ => None,
    }
}

} // verus!
