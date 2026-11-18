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

pub open spec fn step_target_spec(
    profile: u8,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
) -> Option<u8> {
    if node_index < states.len() {
        super::super::primitive::transition::lifecycle_target_spec(profile, states[node_index as int], action)
    } else {
        None
    }
}

pub open spec fn step_dependencies_ready_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
) -> bool {
    (action != super::super::codes::action::ACTION_MARK_READY && action != super::super::codes::action::ACTION_START && action != super::super::codes::action::ACTION_RESUME_SIGNAL)
        || super::super::primitive::validity::dependencies_complete_spec(dependencies[node_index as int]@, states)
}

pub open spec fn step_next_states_spec(
    states: Seq<u8>,
    node_index: usize,
    target: u8,
) -> Seq<u8> {
    states.update(node_index as int, target)
}

} // verus!
