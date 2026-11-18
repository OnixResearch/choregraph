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

pub const DENIAL_NONE: u8 = 0u8;

pub const DENIAL_INVALID_STATE: u8 = 1u8;

pub const DENIAL_UNKNOWN_NODE: u8 = 2u8;

pub const DENIAL_TERMINAL_STATE: u8 = 3u8;

pub const DENIAL_INVALID_TRANSITION: u8 = 4u8;

pub const DENIAL_DEPENDENCIES_INCOMPLETE: u8 = 5u8;

pub const DENIAL_DUPLICATE_EFFECT: u8 = 6u8;

pub const DENIAL_UNDECLARED_EFFECT: u8 = 7u8;

pub const DENIAL_INVARIANT_WOULD_BREAK: u8 = 8u8;

} // verus!
