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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedSuspensionFact {
    pub class: alloc::vec::Vec<u8>,
    pub reference: alloc::vec::Vec<u8>,
}

pub fn copy_values(values: &[usize]) -> (result: alloc::vec::Vec<usize>)
    ensures result@ == values@
{
    let mut result = alloc::vec::Vec::new();
    let mut index: usize = 0;
    while index < values.len()
        invariant
            0 <= index <= values@.len(),
            result@ == values@.subrange(0, index as int),
        decreases values@.len() - index,
    {
        result.push(values[index]);
        index = index + 1;
    }
    result
}

pub fn copy_states(values: &[u8]) -> (result: alloc::vec::Vec<u8>)
    ensures result@ == values@
{
    let mut result = alloc::vec::Vec::new();
    let mut index: usize = 0;
    while index < values.len()
        invariant
            0 <= index <= values@.len(),
            result@ == values@.subrange(0, index as int),
        decreases values@.len() - index,
    {
        result.push(values[index]);
        index = index + 1;
    }
    result
}

pub open spec fn suspensions_consistent_spec(
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
) -> bool {
    &&& states.len() == suspensions.len()
    &&& forall|index: int| 0 <= index < states.len() ==> (suspensions[index].is_some()
        == (states[index] == crate::verified::codes::status::STATE_SUSPENDED))
}

pub fn suspensions_consistent(
    states: &[u8],
    suspensions: &[Option<IndexedSuspensionFact>],
) -> (result: bool)
    ensures result == suspensions_consistent_spec(states@, suspensions@)
{
    if states.len() != suspensions.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < suspensions.len()
        invariant
            states@.len() == suspensions@.len(),
            0 <= index <= suspensions@.len(),
            forall|offset: int| 0 <= offset < index ==> (suspensions@[offset].is_some()
                == (states@[offset] == crate::verified::codes::status::STATE_SUSPENDED)),
        decreases suspensions@.len() - index,
    {
        let has_record = suspensions[index].is_some();
        if has_record != (states[index] == crate::verified::codes::status::STATE_SUSPENDED) {
            return false;
        }
        index = index + 1;
    }
    true
}

pub fn bytes_equal(left: &[u8], right: &[u8]) -> (result: bool)
    ensures result == (left@ == right@)
{
    if left.len() != right.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < left.len()
        invariant
            left@.len() == right@.len(),
            0 <= index <= left@.len(),
            forall|offset: int| 0 <= offset < index ==> left@[offset] == right@[offset],
        decreases left@.len() - index,
    {
        if left[index] != right[index] {
            return false;
        }
        index = index + 1;
    }
    proof {
        assert(left@ =~= right@);
    }
    true
}

} // verus!
