// Readiness proofs own one deterministic filtered frontier over admitted indexed views.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    clippy::assign_op_pattern,
    dead_code,
    non_trait_imports,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus readiness witnesses use bounded vectors and explicit inductive proof steps"
)]

use super::*;

verus! {

// r[impl choregraph.proof.single_step.readiness_completeness]
pub open spec fn dependency_ready_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
) -> bool {
    node_index < dependencies.len()
        && node_index < states.len()
        && states[node_index as int] == STATE_PENDING
        && dependencies_complete_spec(dependencies[node_index as int]@, states)
}

// r[impl choregraph.proof.single_step.readiness_completeness]
pub fn dependency_ready(
    dependencies: &[Vec<usize>],
    states: &[u8],
    node_index: usize,
) -> (result: bool)
    ensures result == dependency_ready_spec(dependencies@, states@, node_index)
{
    if node_index >= dependencies.len() || node_index >= states.len() {
        return false;
    }
    states[node_index] == STATE_PENDING
        && dependencies_complete(&dependencies[node_index], states)
}

pub open spec fn ready_frontier_build_spec(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    from: int,
) -> Seq<usize>
    decreases order.len() - from,
{
    if from < 0 || from >= order.len() {
        seq![]
    } else {
        let node_index = order[from];
        if dependency_ready_spec(dependencies, states, node_index) {
            seq![node_index] + ready_frontier_build_spec(order, dependencies, states, from + 1)
        } else {
            ready_frontier_build_spec(order, dependencies, states, from + 1)
        }
    }
}

pub open spec fn ready_frontier_spec(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
) -> Seq<usize> {
    ready_frontier_build_spec(order, dependencies, states, 0)
}

// r[impl choregraph.proof.single_step.readiness_frontier]
pub fn ready_frontier(
    order: &[usize],
    dependencies: &[Vec<usize>],
    states: &[u8],
) -> (result: Vec<usize>)
    ensures result@ == ready_frontier_spec(order@, dependencies@, states@)
{
    let mut result = Vec::new();
    let mut index: usize = 0;
    let ghost full = ready_frontier_spec(order@, dependencies@, states@);
    while index < order.len()
        invariant
            0 <= index <= order@.len(),
            full == ready_frontier_spec(order@, dependencies@, states@),
            result@ + ready_frontier_build_spec(
                order@,
                dependencies@,
                states@,
                index as int,
            ) =~= full,
        decreases order@.len() - index,
    {
        let node_index = order[index];
        if dependency_ready(dependencies, states, node_index) {
            result.push(node_index);
        }
        index = index + 1;
    }
    result
}

pub open spec fn pairwise_distinct_spec(values: Seq<usize>) -> bool {
    forall|left: int, right: int|
        #![trigger values[left], values[right]]
        0 <= left < values.len()
            && 0 <= right < values.len()
            && left != right
            ==> values[left] != values[right]
}

proof fn ready_frontier_element_source(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    from: int,
    result_index: int,
)
    requires
        0 <= from,
        0 <= result_index < ready_frontier_build_spec(
            order,
            dependencies,
            states,
            from,
        ).len(),
    ensures
        ({
            let result = ready_frontier_build_spec(order, dependencies, states, from);
            let node_index = result[result_index];
            exists|position: int| #![trigger order[position]]
                from <= position < order.len()
                    && order[position] == node_index
                    && dependency_ready_spec(dependencies, states, node_index)
        }),
    decreases order.len() - from,
{
    let result = ready_frontier_build_spec(order, dependencies, states, from);
    if from < order.len() {
        let node_index = order[from];
        if dependency_ready_spec(dependencies, states, node_index) {
            let rest = ready_frontier_build_spec(order, dependencies, states, from + 1);
            assert(result =~= seq![node_index] + rest);
            if result_index > 0 {
                assert(result[result_index] == rest[result_index - 1]);
                ready_frontier_element_source(
                    order,
                    dependencies,
                    states,
                    from + 1,
                    result_index - 1,
                );
            }
        } else {
            ready_frontier_element_source(
                order,
                dependencies,
                states,
                from + 1,
                result_index,
            );
        }
    }
}

proof fn ready_frontier_build_contains_eligible(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    from: int,
    position: int,
)
    requires
        0 <= from <= position < order.len(),
        dependency_ready_spec(dependencies, states, order[position]),
    ensures ready_frontier_build_spec(order, dependencies, states, from).contains(order[position]),
    decreases position - from,
{
    let result = ready_frontier_build_spec(order, dependencies, states, from);
    let node_index = order[from];
    let rest = ready_frontier_build_spec(order, dependencies, states, from + 1);
    if from == position {
        assert(result =~= seq![node_index] + rest);
        assert(result[0] == order[position]);
        assert(result.contains(order[position]));
    } else {
        ready_frontier_build_contains_eligible(
            order,
            dependencies,
            states,
            from + 1,
            position,
        );
        if dependency_ready_spec(dependencies, states, node_index) {
            assert(result =~= seq![node_index] + rest);
            let witness = choose|witness: int| #![trigger rest[witness]]
                0 <= witness < rest.len() && rest[witness] == order[position];
            assert(result[witness + 1] == rest[witness]);
            assert(result.contains(order[position]));
        } else {
            assert(result =~= rest);
        }
    }
}

proof fn ready_frontier_no_duplicates_from(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    from: int,
)
    requires
        0 <= from,
        pairwise_distinct_spec(order),
    ensures pairwise_distinct_spec(ready_frontier_build_spec(
        order,
        dependencies,
        states,
        from,
    )),
    decreases order.len() - from,
{
    let result = ready_frontier_build_spec(order, dependencies, states, from);
    if from < order.len() {
        let node_index = order[from];
        if dependency_ready_spec(dependencies, states, node_index) {
            ready_frontier_no_duplicates_from(order, dependencies, states, from + 1);
            let rest = ready_frontier_build_spec(order, dependencies, states, from + 1);
            assert(result =~= seq![node_index] + rest);
            assert forall|result_index: int| #![trigger rest[result_index]]
                0 <= result_index < rest.len() implies rest[result_index] != node_index by {
                ready_frontier_element_source(
                    order,
                    dependencies,
                    states,
                    from + 1,
                    result_index,
                );
                let position = choose|position: int| #![trigger order[position]]
                    from + 1 <= position < order.len()
                        && order[position] == rest[result_index]
                        && dependency_ready_spec(dependencies, states, rest[result_index]);
                assert(order[from] != order[position]);
            };
            assert forall|left: int, right: int|
                #![trigger result[left], result[right]]
                0 <= left < result.len()
                    && 0 <= right < result.len()
                    && left != right
            implies result[left] != result[right] by {
                if left == 0 && right > 0 {
                    assert(result[left] == node_index);
                    assert(result[right] == rest[right - 1]);
                } else if right == 0 && left > 0 {
                    assert(result[right] == node_index);
                    assert(result[left] == rest[left - 1]);
                } else if left > 0 && right > 0 {
                    assert(result[left] == rest[left - 1]);
                    assert(result[right] == rest[right - 1]);
                }
            };
        } else {
            ready_frontier_no_duplicates_from(order, dependencies, states, from + 1);
        }
    }
}

// r[verify choregraph.proof.single_step.readiness_frontier]
pub proof fn ready_frontier_contains_each_eligible_node_once(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    requires pairwise_distinct_spec(order)
    ensures pairwise_distinct_spec(ready_frontier_spec(order, dependencies, states))
{
    ready_frontier_no_duplicates_from(order, dependencies, states, 0);
}

// r[verify choregraph.proof.single_step.readiness_frontier]
pub proof fn ready_frontier_contains_every_eligible_node(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    ensures forall|position: int| #![trigger order[position]]
        0 <= position < order.len()
            && dependency_ready_spec(dependencies, states, order[position])
            ==> ready_frontier_spec(order, dependencies, states).contains(order[position])
{
    assert forall|position: int| #![trigger order[position]]
        0 <= position < order.len()
            && dependency_ready_spec(dependencies, states, order[position])
        implies ready_frontier_spec(order, dependencies, states).contains(order[position]) by {
        ready_frontier_build_contains_eligible(
            order,
            dependencies,
            states,
            0,
            position,
        );
    };
}

// r[verify choregraph.proof.single_step.readiness_frontier]
pub proof fn ready_frontier_contains_only_eligible_nodes(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    ensures forall|result_index: int|
        #![trigger ready_frontier_spec(order, dependencies, states)[result_index]]
        0 <= result_index < ready_frontier_spec(order, dependencies, states).len()
            ==> exists|position: int| #![trigger order[position]]
                0 <= position < order.len()
                    && order[position]
                        == ready_frontier_spec(order, dependencies, states)[result_index]
                    && dependency_ready_spec(
                        dependencies,
                        states,
                        ready_frontier_spec(order, dependencies, states)[result_index],
                    )
{
    let result = ready_frontier_spec(order, dependencies, states);
    assert forall|result_index: int| #![trigger result[result_index]]
        0 <= result_index < result.len() implies
        exists|position: int| #![trigger order[position]]
            0 <= position < order.len()
                && order[position] == result[result_index]
                && dependency_ready_spec(dependencies, states, result[result_index]) by {
        ready_frontier_element_source(order, dependencies, states, 0, result_index);
    };
}

pub open spec fn frontier_is_admitted_order_filter_spec(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    frontier: Seq<usize>,
) -> bool {
    frontier == ready_frontier_spec(order, dependencies, states)
}

// r[verify choregraph.proof.single_step.readiness_frontier]
pub proof fn ready_frontier_preserves_admitted_order(
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    ensures frontier_is_admitted_order_filter_spec(
        order,
        dependencies,
        states,
        ready_frontier_spec(order, dependencies, states),
    )
{}

} // verus!
