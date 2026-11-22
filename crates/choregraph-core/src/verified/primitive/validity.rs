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

pub open spec fn dependencies_complete_spec(
    dependencies: Seq<usize>,
    states: Seq<u8>,
) -> bool {
    forall|index: int| 0 <= index < dependencies.len() ==> {
        let dependency = #[trigger] dependencies[index];
        dependency < states.len()
            && states[dependency as int] == crate::verified::codes::status::STATE_COMPLETED
    }
}

pub fn dependencies_complete(dependencies: &[usize], states: &[u8]) -> (result: bool)
    ensures result == dependencies_complete_spec(dependencies@, states@)
{
    let mut index: usize = 0;
    while index < dependencies.len()
        invariant
            0 <= index <= dependencies@.len(),
            forall|checked: int| 0 <= checked < index ==> {
                let dependency = #[trigger] dependencies@[checked];
                dependency < states@.len()
                    && states@[dependency as int] == crate::verified::codes::status::STATE_COMPLETED
            },
        decreases dependencies@.len() - index,
    {
        let dependency = dependencies[index];
        if dependency >= states.len() || states[dependency] != crate::verified::codes::status::STATE_COMPLETED {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn state_requires_completed_dependencies_spec(state: u8) -> bool {
    state == crate::verified::codes::status::STATE_READY
        || state == crate::verified::codes::status::STATE_RUNNING
        || state == crate::verified::codes::status::STATE_COMPLETED
        || state == crate::verified::codes::status::STATE_FAILED
}

pub open spec fn node_invariant_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    index: int,
) -> bool {
    crate::verified::codes::status::local_state_valid_spec(states[index])
        && dependencies[index].len() <= states.len()
        && (!state_requires_completed_dependencies_spec(states[index])
            || dependencies_complete_spec(dependencies[index]@, states))
}

pub open spec fn indexed_invariant_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
) -> bool {
    dependencies.len() == states.len()
        && forall|index: int| 0 <= index < states.len()
            ==> #[trigger] node_invariant_spec(dependencies, states, index)
}

pub fn indexed_invariant(dependencies: &[alloc::vec::Vec<usize>], states: &[u8]) -> (result: bool)
    ensures result == indexed_invariant_spec(dependencies@, states@)
{
    if dependencies.len() != states.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < states.len()
        invariant
            dependencies@.len() == states@.len(),
            0 <= index <= states@.len(),
            forall|checked: int| 0 <= checked < index
                ==> #[trigger] node_invariant_spec(dependencies@, states@, checked),
        decreases states@.len() - index,
    {
        let state = states[index];
        if !crate::verified::codes::status::local_state_valid(state) {
            assert(!node_invariant_spec(dependencies@, states@, index as int));
            return false;
        }
        let node_dependencies = &dependencies[index];
        if node_dependencies.len() > states.len() {
            assert(!node_invariant_spec(dependencies@, states@, index as int));
            return false;
        }
        if state_requires_completed_dependencies(state)
            && !dependencies_complete(node_dependencies, states)
        {
            assert(!node_invariant_spec(dependencies@, states@, index as int));
            return false;
        }
        assert(node_invariant_spec(dependencies@, states@, index as int));
        index = index + 1;
    }
    true
}

pub fn state_requires_completed_dependencies(state: u8) -> (result: bool)
    ensures result == state_requires_completed_dependencies_spec(state)
{
    state == crate::verified::codes::status::STATE_READY
        || state == crate::verified::codes::status::STATE_RUNNING
        || state == crate::verified::codes::status::STATE_COMPLETED
        || state == crate::verified::codes::status::STATE_FAILED
}

pub open spec fn terminal_states_preserved_spec(before: Seq<u8>, after: Seq<u8>) -> bool {
    before.len() == after.len()
        && forall|index: int| 0 <= index < before.len()
            && crate::verified::codes::status::terminal_state_spec(before[index])
            ==> #[trigger] after[index] == before[index]
}

} // verus!
