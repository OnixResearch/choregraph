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

pub open spec fn linear_lifecycle_target_spec(from: u8, action: u8) -> Option<u8> {
    if from == crate::verified::codes::status::STATE_PENDING && action == crate::verified::codes::action::ACTION_MARK_READY {
        Some(crate::verified::codes::status::STATE_READY)
    } else if from == crate::verified::codes::status::STATE_READY && action == crate::verified::codes::action::ACTION_START {
        Some(crate::verified::codes::status::STATE_RUNNING)
    } else if from == crate::verified::codes::status::STATE_RUNNING && action == crate::verified::codes::action::ACTION_COMPLETE {
        Some(crate::verified::codes::status::STATE_COMPLETED)
    } else if from == crate::verified::codes::status::STATE_RUNNING && action == crate::verified::codes::action::ACTION_FAIL {
        Some(crate::verified::codes::status::STATE_FAILED)
    } else if (from == crate::verified::codes::status::STATE_PENDING || from == crate::verified::codes::status::STATE_READY || from == crate::verified::codes::status::STATE_RUNNING)
        && action == crate::verified::codes::action::ACTION_CANCEL
    {
        Some(crate::verified::codes::status::STATE_CANCELLED)
    } else {
        None
    }
}

pub open spec fn lifecycle_target_spec(profile: u8, from: u8, action: u8) -> Option<u8> {
    if profile == crate::verified::codes::profile::PROFILE_LINEAR_V1 {
        linear_lifecycle_target_spec(from, action)
    } else if profile == crate::verified::codes::profile::PROFILE_DURABLE_SUSPENSION_V1 {
        if linear_lifecycle_target_spec(from, action).is_some() {
            linear_lifecycle_target_spec(from, action)
        } else if (from == crate::verified::codes::status::STATE_READY || from == crate::verified::codes::status::STATE_RUNNING) && action == crate::verified::codes::action::ACTION_SUSPEND {
            Some(crate::verified::codes::status::STATE_SUSPENDED)
        } else if from == crate::verified::codes::status::STATE_SUSPENDED && action == crate::verified::codes::action::ACTION_RESUME_SIGNAL {
            Some(crate::verified::codes::status::STATE_READY)
        } else {
            None
        }
    } else {
        None
    }
}

pub const fn linear_lifecycle_target(from: u8, action: u8) -> (result: Option<u8>)
    ensures result == linear_lifecycle_target_spec(from, action)
{
    if from == crate::verified::codes::status::STATE_PENDING && action == crate::verified::codes::action::ACTION_MARK_READY {
        Some(crate::verified::codes::status::STATE_READY)
    } else if from == crate::verified::codes::status::STATE_READY && action == crate::verified::codes::action::ACTION_START {
        Some(crate::verified::codes::status::STATE_RUNNING)
    } else if from == crate::verified::codes::status::STATE_RUNNING && action == crate::verified::codes::action::ACTION_COMPLETE {
        Some(crate::verified::codes::status::STATE_COMPLETED)
    } else if from == crate::verified::codes::status::STATE_RUNNING && action == crate::verified::codes::action::ACTION_FAIL {
        Some(crate::verified::codes::status::STATE_FAILED)
    } else if (from == crate::verified::codes::status::STATE_PENDING || from == crate::verified::codes::status::STATE_READY || from == crate::verified::codes::status::STATE_RUNNING)
        && action == crate::verified::codes::action::ACTION_CANCEL
    {
        Some(crate::verified::codes::status::STATE_CANCELLED)
    } else {
        None
    }
}

pub const fn lifecycle_target(profile: u8, from: u8, action: u8) -> (result: Option<u8>)
    ensures result == lifecycle_target_spec(profile, from, action)
{
    if profile == crate::verified::codes::profile::PROFILE_LINEAR_V1 {
        linear_lifecycle_target(from, action)
    } else if profile == crate::verified::codes::profile::PROFILE_DURABLE_SUSPENSION_V1 {
        let linear = linear_lifecycle_target(from, action);
        if linear.is_some() {
            linear
        } else if (from == crate::verified::codes::status::STATE_READY || from == crate::verified::codes::status::STATE_RUNNING) && action == crate::verified::codes::action::ACTION_SUSPEND {
            Some(crate::verified::codes::status::STATE_SUSPENDED)
        } else if from == crate::verified::codes::status::STATE_SUSPENDED && action == crate::verified::codes::action::ACTION_RESUME_SIGNAL {
            Some(crate::verified::codes::status::STATE_READY)
        } else {
            None
        }
    } else {
        None
    }
}

} // verus!
