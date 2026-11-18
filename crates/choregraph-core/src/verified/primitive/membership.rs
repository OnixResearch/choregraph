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

pub open spec fn contains_from_spec(values: Seq<usize>, needle: usize, from: int) -> bool
    decreases values.len() - from
{
    if from < 0 || from >= values.len() {
        false
    } else {
        values[from] == needle || contains_from_spec(values, needle, from + 1)
    }
}

pub open spec fn contains_spec(values: Seq<usize>, needle: usize) -> bool {
    contains_from_spec(values, needle, 0)
}

pub fn contains(values: &[usize], needle: usize) -> (result: bool)
    ensures result == contains_spec(values@, needle)
{
    let mut index: usize = 0;
    while index < values.len()
        invariant
            0 <= index <= values@.len(),
            contains_spec(values@, needle)
                <==> contains_from_spec(values@, needle, index as int),
        decreases values@.len() - index,
    {
        if values[index] == needle {
            return true;
        }
        index = index + 1;
    }
    false
}

pub open spec fn all_requested_allowed_from_spec(
    allowed: Seq<usize>,
    requested: Seq<usize>,
    from: int,
) -> bool
    decreases requested.len() - from
{
    if from < 0 || from >= requested.len() {
        true
    } else {
        contains_spec(allowed, requested[from])
            && all_requested_allowed_from_spec(allowed, requested, from + 1)
    }
}

pub open spec fn all_requested_allowed_spec(
    allowed: Seq<usize>,
    requested: Seq<usize>,
) -> bool {
    all_requested_allowed_from_spec(allowed, requested, 0)
}

pub fn all_requested_allowed(allowed: &[usize], requested: &[usize]) -> (result: bool)
    ensures result == all_requested_allowed_spec(allowed@, requested@)
{
    let mut index: usize = 0;
    while index < requested.len()
        invariant
            0 <= index <= requested@.len(),
            all_requested_allowed_spec(allowed@, requested@)
                <==> all_requested_allowed_from_spec(allowed@, requested@, index as int),
        decreases requested@.len() - index,
    {
        if !contains(allowed, requested[index]) {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn not_in_suffix_spec(
    values: Seq<usize>,
    needle: usize,
    from: int,
) -> bool
    decreases values.len() - from
{
    if from < 0 || from >= values.len() {
        true
    } else {
        values[from] != needle && not_in_suffix_spec(values, needle, from + 1)
    }
}

pub open spec fn all_distinct_from_spec(values: Seq<usize>, from: int) -> bool
    decreases values.len() - from
{
    if from < 0 || from >= values.len() {
        true
    } else {
        not_in_suffix_spec(values, values[from], from + 1)
            && all_distinct_from_spec(values, from + 1)
    }
}

pub open spec fn all_distinct_spec(values: Seq<usize>) -> bool {
    all_distinct_from_spec(values, 0)
}

pub fn all_distinct(values: &[usize]) -> (result: bool)
    ensures result == all_distinct_spec(values@)
{
    let mut left: usize = 0;
    while left < values.len()
        invariant
            0 <= left <= values@.len(),
            all_distinct_spec(values@)
                <==> all_distinct_from_spec(values@, left as int),
        decreases values@.len() - left,
    {
        let mut right = left + 1;
        while right < values.len()
            invariant
                left < values@.len(),
                left < right <= values@.len(),
                all_distinct_spec(values@)
                    <==> all_distinct_from_spec(values@, left as int),
                not_in_suffix_spec(values@, values@[left as int], (left + 1) as int)
                    <==> not_in_suffix_spec(values@, values@[left as int], right as int),
            decreases values@.len() - right,
        {
            if values[left] == values[right] {
                proof {
                    assert(!not_in_suffix_spec(
                        values@,
                        values@[left as int],
                        right as int,
                    ));
                    assert(!not_in_suffix_spec(
                        values@,
                        values@[left as int],
                        (left + 1) as int,
                    ));
                    assert(!all_distinct_from_spec(values@, left as int));
                    assert(!all_distinct_spec(values@));
                }
                return false;
            }
            right = right + 1;
        }
        left = left + 1;
    }
    true
}

pub open spec fn effects_allowed_spec(allowed: Seq<usize>, requested: Seq<usize>) -> bool {
    all_requested_allowed_spec(allowed, requested) && all_distinct_spec(requested)
}

} // verus!
