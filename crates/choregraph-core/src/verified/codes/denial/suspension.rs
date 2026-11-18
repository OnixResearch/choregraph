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

pub const DENIAL_SUSPENSION_MISMATCH: u8 = 9u8;

pub const DENIAL_SUSPENSION_FACT_ABSENT: u8 = 10u8;

pub const DENIAL_SUSPENSION_FACT_PRESENT: u8 = 11u8;

pub const DENIAL_SUSPENSION_EFFECTS: u8 = 12u8;

pub const DENIAL_PROFILE_ACTION: u8 = 13u8;

} // verus!
