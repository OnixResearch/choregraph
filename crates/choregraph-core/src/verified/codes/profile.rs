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

pub const PROFILE_LINEAR_V1: u8 = 0u8;

pub const PROFILE_DURABLE_SUSPENSION_V1: u8 = 1u8;

pub const PROFILE_DURABLE_SUSPENSION_V2: u8 = 2u8;

pub open spec fn durable_profile_spec(profile: u8) -> bool {
    profile == PROFILE_DURABLE_SUSPENSION_V1 || profile == PROFILE_DURABLE_SUSPENSION_V2
}

pub open spec fn profile_valid_spec(profile: u8) -> bool {
    profile == PROFILE_LINEAR_V1 || durable_profile_spec(profile)
}

pub const fn has_suspension(profile: u8) -> (result: bool)
    ensures result == durable_profile_spec(profile)
{
    profile == PROFILE_DURABLE_SUSPENSION_V1 || profile == PROFILE_DURABLE_SUSPENSION_V2
}

pub const fn is_supported(profile: u8) -> (result: bool)
    ensures result == profile_valid_spec(profile)
{
    profile == PROFILE_LINEAR_V1 || has_suspension(profile)
}

} // verus!
