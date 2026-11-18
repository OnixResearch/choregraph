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

pub open spec fn dependencies_complete_from_spec(
    dependencies: Seq<usize>,
    states: Seq<u8>,
    from: int,
) -> bool
    decreases dependencies.len() - from
{
    if from < 0 || from >= dependencies.len() {
        true
    } else {
        dependencies[from] < states.len()
            && states[dependencies[from] as int] == crate::verified::codes::status::STATE_COMPLETED
            && dependencies_complete_from_spec(dependencies, states, from + 1)
    }
}

pub open spec fn dependencies_complete_spec(
    dependencies: Seq<usize>,
    states: Seq<u8>,
) -> bool {
    dependencies_complete_from_spec(dependencies, states, 0)
}

pub fn dependencies_complete(dependencies: &[usize], states: &[u8]) -> (result: bool)
    ensures result == dependencies_complete_spec(dependencies@, states@)
{
    let mut index: usize = 0;
    while index < dependencies.len()
        invariant
            0 <= index <= dependencies@.len(),
            dependencies_complete_spec(dependencies@, states@)
                <==> dependencies_complete_from_spec(dependencies@, states@, index as int),
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

pub open spec fn indexed_invariant_from_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    from: int,
) -> bool
    decreases states.len() - from
{
    if from < 0 || from >= states.len() {
        true
    } else {
        crate::verified::codes::status::local_state_valid_spec(states[from])
            && dependencies[from].len() <= states.len()
            && (!state_requires_completed_dependencies_spec(states[from])
                || dependencies_complete_spec(dependencies[from]@, states))
            && indexed_invariant_from_spec(dependencies, states, from + 1)
    }
}

pub open spec fn indexed_invariant_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
) -> bool {
    dependencies.len() == states.len()
        && indexed_invariant_from_spec(dependencies, states, 0)
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
            indexed_invariant_from_spec(dependencies@, states@, 0)
                <==> indexed_invariant_from_spec(dependencies@, states@, index as int),
        decreases states@.len() - index,
    {
        let state = states[index];
        if !crate::verified::codes::status::local_state_valid(state) {
            return false;
        }
        let node_dependencies = &dependencies[index];
        if node_dependencies.len() > states.len() {
            return false;
        }
        if state_requires_completed_dependencies(state)
            && !dependencies_complete(node_dependencies, states)
        {
            return false;
        }
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

pub open spec fn terminal_states_preserved_from_spec(
    before: Seq<u8>,
    after: Seq<u8>,
    from: int,
) -> bool
    decreases before.len() - from
{
    if from < 0 || from >= before.len() {
        true
    } else {
        (!crate::verified::codes::status::terminal_state_spec(before[from]) || after[from] == before[from])
            && terminal_states_preserved_from_spec(before, after, from + 1)
    }
}

pub open spec fn terminal_states_preserved_spec(before: Seq<u8>, after: Seq<u8>) -> bool {
    before.len() == after.len()
        && terminal_states_preserved_from_spec(before, after, 0)
}

pub fn terminal_states_preserved(before: &[u8], after: &[u8]) -> (result: bool)
    ensures result == terminal_states_preserved_spec(before@, after@)
{
    if before.len() != after.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < before.len()
        invariant
            before@.len() == after@.len(),
            0 <= index <= before@.len(),
            terminal_states_preserved_from_spec(before@, after@, 0)
                <==> terminal_states_preserved_from_spec(before@, after@, index as int),
        decreases before@.len() - index,
    {
        let state = before[index];
        if crate::verified::codes::status::terminal_state(state) && after[index] != state {
            return false;
        }
        index = index + 1;
    }
    true
}

} // verus!
