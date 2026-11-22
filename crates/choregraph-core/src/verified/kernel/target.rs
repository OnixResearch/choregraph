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

#[allow(
    unused_imports,
    reason = "Verus erases the consumers of these imports in Rust builds"
)]
use super::super::{codes::status::*, primitive::validity::*};
use vstd::prelude::*;

verus! {

pub open spec fn action_needs_readiness_spec(action: u8) -> bool {
    action == super::super::codes::action::ACTION_MARK_READY
        || action == super::super::codes::action::ACTION_START
        || action == super::super::codes::action::ACTION_RESUME_SIGNAL
}

pub const fn action_needs_readiness(action: u8) -> (result: bool)
    ensures result == action_needs_readiness_spec(action)
{
    action == super::super::codes::action::ACTION_MARK_READY
        || action == super::super::codes::action::ACTION_START
        || action == super::super::codes::action::ACTION_RESUME_SIGNAL
}

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
    !action_needs_readiness_spec(action)
        || super::super::primitive::validity::dependencies_complete_spec(dependencies[node_index as int]@, states)
}

pub open spec fn step_next_states_spec(
    states: Seq<u8>,
    node_index: usize,
    target: u8,
) -> Seq<u8> {
    states.update(node_index as int, target)
}

// No dependency that was complete can change: Completed is terminal. A newly
// ready target must additionally have passed the action's dependency guard.
pub proof fn local_update_preserves_invariant(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    profile: u8,
    node: usize,
    action: u8,
    target: u8,
)
    requires
        indexed_invariant_spec(dependencies, states),
        node < states.len(),
        step_target_spec(profile, states, node, action) == Some(target),
        step_dependencies_ready_spec(dependencies, states, node, action),
    ensures
        indexed_invariant_spec(dependencies, step_next_states_spec(states, node, target)),
        terminal_states_preserved_spec(states, step_next_states_spec(states, node, target)),
{
    let from = states[node as int];
    let next = step_next_states_spec(states, node, target);
    assert(!terminal_state_spec(from));
    assert(local_state_valid_spec(target));
    assert(state_requires_completed_dependencies_spec(target)
        ==> state_requires_completed_dependencies_spec(from) || action_needs_readiness_spec(action));
    assert forall|index: int| 0 <= index < states.len()
        && terminal_state_spec(states[index]) implies #[trigger] next[index] == states[index] by {
        assert(index != node as int);
    }
    assert forall|index: int| 0 <= index < next.len()
        implies #[trigger] node_invariant_spec(dependencies, next, index) by {
        assert(node_invariant_spec(dependencies, states, index));
        if state_requires_completed_dependencies_spec(next[index]) {
            if index != node as int {
                assert(next[index] == states[index]);
            }
            let deps = dependencies[index]@;
            assert(dependencies_complete_spec(deps, states));
            assert forall|slot: int| 0 <= slot < deps.len() implies {
                let dependency = #[trigger] deps[slot];
                dependency < next.len() && next[dependency as int] == STATE_COMPLETED
            } by {
                let dependency = deps[slot];
                assert(states[dependency as int] == STATE_COMPLETED);
                assert(dependency != node);
            }
            assert(dependencies_complete_spec(deps, next));
        }
    }
}

} // verus!
