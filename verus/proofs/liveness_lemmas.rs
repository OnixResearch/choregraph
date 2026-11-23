// Bounded workflow liveness proofs over the production-linked indexed kernel.
vstd::prelude::verus! {

use vstd::prelude::*;

pub open spec fn all_terminal_spec(states: Seq<u8>) -> bool {
    forall|index: int| 0 <= index < states.len()
        ==> crate::verified::codes::status::terminal_state_spec(#[trigger] states[index])
}

pub open spec fn non_terminal_count_from_spec(states: Seq<u8>, from: int) -> nat
    decreases if from < states.len() { states.len() - from } else { 0 }
{
    if from < 0 || from >= states.len() {
        0
    } else {
        let here: nat = if crate::verified::codes::status::terminal_state_spec(states[from]) {
            0
        } else {
            1
        };
        here + non_terminal_count_from_spec(states, from + 1)
    }
}

pub open spec fn non_terminal_count_spec(states: Seq<u8>) -> nat {
    non_terminal_count_from_spec(states, 0)
}

pub open spec fn linear_node_rank_spec(state: u8) -> nat {
    if state == crate::verified::codes::status::STATE_PENDING {
        3
    } else if state == crate::verified::codes::status::STATE_READY {
        2
    } else if state == crate::verified::codes::status::STATE_RUNNING {
        1
    } else if state == crate::verified::codes::status::STATE_SUSPENDED {
        2
    } else {
        0
    }
}

pub open spec fn linear_steps_remaining_from_spec(states: Seq<u8>, from: int) -> nat
    decreases if from < states.len() { states.len() - from } else { 0 }
{
    if from < 0 || from >= states.len() {
        0
    } else {
        linear_node_rank_spec(states[from]) + linear_steps_remaining_from_spec(states, from + 1)
    }
}

pub open spec fn linear_steps_remaining_spec(states: Seq<u8>) -> nat {
    linear_steps_remaining_from_spec(states, 0)
}

pub open spec fn terminal_action_spec(action: u8) -> bool {
    action == crate::verified::codes::action::ACTION_COMPLETE
        || action == crate::verified::codes::action::ACTION_FAIL
        || action == crate::verified::codes::action::ACTION_CANCEL
}

pub open spec fn linear_progress_action_spec(action: u8) -> bool {
    action == crate::verified::codes::action::ACTION_MARK_READY
        || action == crate::verified::codes::action::ACTION_START
        || terminal_action_spec(action)
}

pub open spec fn no_failed_cancelled_or_suspended_spec(states: Seq<u8>) -> bool {
    forall|index: int| 0 <= index < states.len() ==> {
        let state = #[trigger] states[index];
        state == crate::verified::codes::status::STATE_PENDING
            || state == crate::verified::codes::status::STATE_READY
            || state == crate::verified::codes::status::STATE_RUNNING
            || state == crate::verified::codes::status::STATE_COMPLETED
    }
}

pub open spec fn dependencies_in_edges_spec(
    edges: Seq<(usize, usize)>,
    dependencies: Seq<alloc::vec::Vec<usize>>,
) -> bool {
    forall|target: int, dep_idx: int|
        0 <= target < dependencies.len() && 0 <= dep_idx < dependencies[target].len() ==> {
            let source = #[trigger] dependencies[target][dep_idx];
            exists|edge_idx: int|
                0 <= edge_idx < edges.len()
                    && (#[trigger] edges[edge_idx]).0 == source
                    && edges[edge_idx].1 == target as usize
        }
}

proof fn non_terminal_count_from_bounds(states: Seq<u8>, from: int)
    requires 0 <= from <= states.len()
    ensures
        non_terminal_count_from_spec(states, from) <= (states.len() - from) as nat,
        (non_terminal_count_from_spec(states, from) == 0)
            <==> (forall|index: int| from <= index < states.len()
                ==> crate::verified::codes::status::terminal_state_spec(#[trigger] states[index])),
    decreases states.len() - from
{
    if from < states.len() {
        non_terminal_count_from_bounds(states, from + 1);
    }
}

pub proof fn non_terminal_count_zero_iff_all_terminal(states: Seq<u8>)
    ensures
        non_terminal_count_spec(states) <= states.len(),
        (non_terminal_count_spec(states) == 0) <==> all_terminal_spec(states),
{
    non_terminal_count_from_bounds(states, 0);
}

proof fn linear_steps_remaining_from_bounds(states: Seq<u8>, from: int)
    requires 0 <= from <= states.len()
    ensures
        linear_steps_remaining_from_spec(states, from) <= 3 * (states.len() - from) as nat,
        (linear_steps_remaining_from_spec(states, from) == 0)
            <==> (forall|index: int| from <= index < states.len()
                ==> linear_node_rank_spec(#[trigger] states[index]) == 0),
    decreases states.len() - from
{
    if from < states.len() {
        linear_steps_remaining_from_bounds(states, from + 1);
    }
}

pub proof fn linear_steps_remaining_bounded_and_zero_iff_terminal(states: Seq<u8>)
    requires
        forall|index: int| 0 <= index < states.len()
            ==> crate::verified::codes::status::local_state_valid_spec(#[trigger] states[index]),
    ensures
        linear_steps_remaining_spec(states) <= 3 * states.len(),
        (linear_steps_remaining_spec(states) == 0) <==> all_terminal_spec(states),
{
    linear_steps_remaining_from_bounds(states, 0);
    assert forall|index: int| 0 <= index < states.len() implies
        (linear_node_rank_spec(#[trigger] states[index]) == 0)
            <==> crate::verified::codes::status::terminal_state_spec(states[index]) by {
        let state = states[index];
        assert(crate::verified::codes::status::local_state_valid_spec(state));
    };
}

proof fn non_terminal_count_update(
    states: Seq<u8>,
    node_index: int,
    new_state: u8,
    from: int,
)
    requires
        0 <= from <= states.len(),
        0 <= node_index < states.len(),
    ensures
        ({
            let updated = states.update(node_index, new_state);
            let old_here: nat = if crate::verified::codes::status::terminal_state_spec(states[node_index]) { 0 } else { 1 };
            let new_here: nat = if crate::verified::codes::status::terminal_state_spec(new_state) { 0 } else { 1 };
            if from <= node_index {
                non_terminal_count_from_spec(updated, from) + old_here
                    == non_terminal_count_from_spec(states, from) + new_here
            } else {
                non_terminal_count_from_spec(updated, from)
                    == non_terminal_count_from_spec(states, from)
            }
        }),
    decreases states.len() - from
{
    if from < states.len() {
        non_terminal_count_update(states, node_index, new_state, from + 1);
    }
}

proof fn linear_steps_remaining_update(
    states: Seq<u8>,
    node_index: int,
    new_state: u8,
    from: int,
)
    requires
        0 <= from <= states.len(),
        0 <= node_index < states.len(),
    ensures
        ({
            let updated = states.update(node_index, new_state);
            if from <= node_index {
                linear_steps_remaining_from_spec(updated, from) + linear_node_rank_spec(states[node_index])
                    == linear_steps_remaining_from_spec(states, from) + linear_node_rank_spec(new_state)
            } else {
                linear_steps_remaining_from_spec(updated, from)
                    == linear_steps_remaining_from_spec(states, from)
            }
        }),
    decreases states.len() - from
{
    if from < states.len() {
        linear_steps_remaining_update(states, node_index, new_state, from + 1);
    }
}

pub proof fn accepted_step_never_increases_non_terminal_count(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<crate::verified::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires
        crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            action,
            event_fact,
            allowed_intents,
            requested_intents,
        ),
    ensures
        non_terminal_count_spec(
            crate::verified::kernel::decision::indexed_step_next_spec(
                dependencies,
                states,
                suspensions,
                profile,
                node_index,
                action,
                event_fact,
                allowed_intents,
                requested_intents,
            ),
        ) <= non_terminal_count_spec(states),
{
    let target = crate::verified::kernel::target::step_target_spec(profile, states, node_index, action).unwrap();
    let next = crate::verified::kernel::target::step_next_states_spec(states, node_index, target);
    assert(next =~= states.update(node_index as int, target));
    assert(!crate::verified::codes::status::terminal_state_spec(states[node_index as int]));
    non_terminal_count_update(states, node_index as int, target, 0);
}

pub proof fn accepted_terminal_event_strictly_decreases_non_terminal_count(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<crate::verified::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires
        crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            action,
            event_fact,
            allowed_intents,
            requested_intents,
        ),
        terminal_action_spec(action),
    ensures
        non_terminal_count_spec(
            crate::verified::kernel::decision::indexed_step_next_spec(
                dependencies,
                states,
                suspensions,
                profile,
                node_index,
                action,
                event_fact,
                allowed_intents,
                requested_intents,
            ),
        ) + 1 == non_terminal_count_spec(states),
{
    let target = crate::verified::kernel::target::step_target_spec(profile, states, node_index, action).unwrap();
    let next = crate::verified::kernel::target::step_next_states_spec(states, node_index, target);
    assert(next =~= states.update(node_index as int, target));
    assert(!crate::verified::codes::status::terminal_state_spec(states[node_index as int]));
    assert(crate::verified::codes::status::terminal_state_spec(target));
    non_terminal_count_update(states, node_index as int, target, 0);
}

pub proof fn accepted_linear_progress_strictly_decreases_steps_remaining(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<crate::verified::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires
        crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            action,
            event_fact,
            allowed_intents,
            requested_intents,
        ),
        linear_progress_action_spec(action),
    ensures
        linear_steps_remaining_spec(
            crate::verified::kernel::decision::indexed_step_next_spec(
                dependencies,
                states,
                suspensions,
                profile,
                node_index,
                action,
                event_fact,
                allowed_intents,
                requested_intents,
            ),
        ) < linear_steps_remaining_spec(states),
{
    let target = crate::verified::kernel::target::step_target_spec(profile, states, node_index, action).unwrap();
    let next = crate::verified::kernel::target::step_next_states_spec(states, node_index, target);
    assert(next =~= states.update(node_index as int, target));
    assert(linear_node_rank_spec(target) < linear_node_rank_spec(states[node_index as int]));
    linear_steps_remaining_update(states, node_index as int, target, 0);
}

proof fn occurrence_one_has_unique_position(order: Seq<usize>, node: usize, from: int)
    requires
        0 <= from <= order.len(),
        crate::verified_topology::cardinality::occurrence_class_from_spec(order, node, from)
            == crate::verified_topology::cardinality::OCCURRENCE_ONE,
    ensures
        exists|pos: int|
            from <= pos < order.len()
                && #[trigger] order[pos] == node
                && (forall|other: int| from <= other < order.len() && other != pos
                    ==> order[other] != node),
    decreases order.len() - from
{
    if from < order.len() {
        let tail = crate::verified_topology::cardinality::occurrence_class_from_spec(order, node, from + 1);
        if order[from] == node {
            assert(tail == crate::verified_topology::cardinality::OCCURRENCE_NONE);
            occurrence_none_means_absent(order, node, from + 1);
            assert(order[from] == node);
        } else {
            occurrence_one_has_unique_position(order, node, from + 1);
            let pos = choose|pos: int|
                from + 1 <= pos < order.len()
                    && #[trigger] order[pos] == node
                    && (forall|other: int| from + 1 <= other < order.len() && other != pos
                        ==> order[other] != node);
            assert(order[pos] == node);
        }
    }
}

proof fn occurrence_none_means_absent(order: Seq<usize>, node: usize, from: int)
    requires
        0 <= from <= order.len(),
        crate::verified_topology::cardinality::occurrence_class_from_spec(order, node, from)
            == crate::verified_topology::cardinality::OCCURRENCE_NONE,
    ensures
        forall|pos: int| from <= pos < order.len() ==> #[trigger] order[pos] != node,
    decreases order.len() - from
{
    if from < order.len() {
        occurrence_none_means_absent(order, node, from + 1);
    }
}

proof fn all_nodes_once_implies_node_occurs_once(
    order: Seq<usize>,
    node_count: usize,
    from: int,
    node: usize,
)
    requires
        0 <= from <= node as int,
        (node as int) < node_count as int,
        crate::verified_topology::cardinality::all_nodes_once_from_spec(order, node_count, from),
    ensures
        crate::verified_topology::cardinality::occurrence_class_spec(order, node)
            == crate::verified_topology::cardinality::OCCURRENCE_ONE,
    decreases node as int - from
{
    if from < node as int {
        all_nodes_once_implies_node_occurs_once(order, node_count, from + 1, node);
    }
}

proof fn edges_forward_at_index(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    from: int,
    edge_idx: int,
)
    requires
        0 <= from <= edge_idx < edges.len(),
        crate::verified_topology::precedence::edges_forward_from_spec(edges, node_count, order, from),
    ensures
        edges[edge_idx].0 < node_count,
        edges[edge_idx].1 < node_count,
        edges[edge_idx].0 != edges[edge_idx].1,
        crate::verified_topology::precedence::precedes_spec(order, edges[edge_idx].0, edges[edge_idx].1),
    decreases edge_idx - from
{
    if from < edge_idx {
        edges_forward_at_index(edges, node_count, order, from + 1, edge_idx);
    }
}

proof fn precedes_from_implies_earlier_position(
    order: Seq<usize>,
    source: usize,
    target: usize,
    source_seen: bool,
    from: int,
    target_pos: int,
)
    requires
        0 <= from <= target_pos < order.len(),
        order[target_pos] == target,
        forall|other: int| from <= other < target_pos ==> #[trigger] order[other] != target,
        crate::verified_topology::precedence::precedes_from_spec(order, source, target, source_seen, from),
    ensures
        source_seen || exists|src_pos: int| from <= src_pos < target_pos && #[trigger] order[src_pos] == source,
    decreases target_pos - from
{
    if from < target_pos {
        assert(order[from] != target);
        precedes_from_implies_earlier_position(
            order,
            source,
            target,
            source_seen || order[from] == source,
            from + 1,
            target_pos,
        );
    }
}

proof fn earliest_non_completed_in_order(
    order: Seq<usize>,
    states: Seq<u8>,
    from: int,
    witness_pos: int,
)
    requires
        0 <= from <= witness_pos < order.len(),
        forall|pos: int| 0 <= pos < order.len() ==> #[trigger] order[pos] < states.len(),
        states[order[witness_pos] as int] != crate::verified::codes::status::STATE_COMPLETED,
    ensures
        exists|first_pos: int|
            from <= first_pos <= witness_pos
                && states[#[trigger] order[first_pos] as int] != crate::verified::codes::status::STATE_COMPLETED
                && (forall|earlier: int| from <= earlier < first_pos
                    ==> states[#[trigger] order[earlier] as int] == crate::verified::codes::status::STATE_COMPLETED),
    decreases witness_pos - from
{
    if states[order[from] as int] != crate::verified::codes::status::STATE_COMPLETED {
        assert(order[from] < states.len());
    } else {
        earliest_non_completed_in_order(order, states, from + 1, witness_pos);
        let first_pos = choose|first_pos: int|
            from + 1 <= first_pos <= witness_pos
                && states[#[trigger] order[first_pos] as int] != crate::verified::codes::status::STATE_COMPLETED
                && (forall|earlier: int| from + 1 <= earlier < first_pos
                    ==> states[#[trigger] order[earlier] as int] == crate::verified::codes::status::STATE_COMPLETED);
        assert(order[first_pos] < states.len());
    }
}

pub proof fn admitted_dag_has_enabled_progress_action(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
)
    requires
        crate::verified_topology::admission::admitted_graph_spec(edges, node_count, order, dependencies),
        dependencies_in_edges_spec(edges, dependencies),
        states.len() == node_count as int,
        crate::verified::primitive::validity::indexed_invariant_spec(dependencies, states),
        crate::verified::primitive::support::suspensions_consistent_spec(states, suspensions),
        crate::verified::codes::profile::profile_valid_spec(profile),
        no_failed_cancelled_or_suspended_spec(states),
        !all_terminal_spec(states),
    ensures
        exists|node_index: usize, action: u8|
            linear_progress_action_spec(action)
                && #[trigger] crate::verified::kernel::decision::indexed_step_accepts_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    node_index,
                    action,
                    None,
                    Seq::<usize>::empty(),
                    Seq::<usize>::empty(),
                )
                && linear_steps_remaining_spec(
                    crate::verified::kernel::decision::indexed_step_next_spec(
                        dependencies,
                        states,
                        suspensions,
                        profile,
                        node_index,
                        action,
                        None,
                        Seq::<usize>::empty(),
                        Seq::<usize>::empty(),
                    ),
                ) < linear_steps_remaining_spec(states),
{
    assert forall|pos: int| 0 <= pos < order.len() implies #[trigger] order[pos] < states.len() by {
        assert(crate::verified_topology::cardinality::all_nodes_once_spec(order, node_count));
        admitted_order_element_in_bounds(order, node_count, pos);
    };
    let non_term = choose|index: int|
        0 <= index < states.len()
            && !crate::verified::codes::status::terminal_state_spec(#[trigger] states[index]);
    all_nodes_once_implies_node_occurs_once(order, node_count, 0, non_term as usize);
    occurrence_one_has_unique_position(order, non_term as usize, 0);
    let witness_pos = choose|pos: int|
        0 <= pos < order.len()
            && #[trigger] order[pos] == non_term as usize
            && (forall|other: int| 0 <= other < order.len() && other != pos ==> order[other] != non_term as usize);
    earliest_non_completed_in_order(order, states, 0, witness_pos);
    let first_pos = choose|first_pos: int|
        0 <= first_pos <= witness_pos
            && states[#[trigger] order[first_pos] as int] != crate::verified::codes::status::STATE_COMPLETED
            && (forall|earlier: int| 0 <= earlier < first_pos
                ==> states[#[trigger] order[earlier] as int] == crate::verified::codes::status::STATE_COMPLETED);
    let u = order[first_pos];
    assert(u < states.len());
    assert(crate::verified::primitive::validity::node_invariant_spec(dependencies, states, u as int));
    if states[u as int] == crate::verified::codes::status::STATE_RUNNING {
        let a = crate::verified::codes::action::ACTION_COMPLETE;
        assert(crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies, states, suspensions, profile, u, a, None, Seq::<usize>::empty(), Seq::<usize>::empty(),
        ));
        accepted_linear_progress_strictly_decreases_steps_remaining(
            dependencies, states, suspensions, profile, u, a, None, Seq::<usize>::empty(), Seq::<usize>::empty(),
        );
    } else if states[u as int] == crate::verified::codes::status::STATE_READY {
        let a = crate::verified::codes::action::ACTION_START;
        assert(crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies, states, suspensions, profile, u, a, None, Seq::<usize>::empty(), Seq::<usize>::empty(),
        ));
        accepted_linear_progress_strictly_decreases_steps_remaining(
            dependencies, states, suspensions, profile, u, a, None, Seq::<usize>::empty(), Seq::<usize>::empty(),
        );
    } else {
        assert(states[u as int] == crate::verified::codes::status::STATE_PENDING);
        all_nodes_once_implies_node_occurs_once(order, node_count, 0, u);
        occurrence_one_has_unique_position(order, u, 0);
        let unique_pos = choose|pos: int|
            0 <= pos < order.len()
                && #[trigger] order[pos] == u
                && (forall|other: int| 0 <= other < order.len() && other != pos ==> order[other] != u);
        assert(unique_pos == first_pos);
        assert forall|dep_idx: int| 0 <= dep_idx < dependencies[u as int].len() implies {
            let dep = #[trigger] dependencies[u as int][dep_idx];
            dep < states.len() && states[dep as int] == crate::verified::codes::status::STATE_COMPLETED
        } by {
            let dep = dependencies[u as int][dep_idx];
            let edge_idx = choose|edge_idx: int|
                0 <= edge_idx < edges.len()
                    && (#[trigger] edges[edge_idx]).0 == dep
                    && edges[edge_idx].1 == u;
            edges_forward_at_index(edges, node_count, order, 0, edge_idx);
            precedes_from_implies_earlier_position(order, dep, u, false, 0, first_pos);
            let src_pos = choose|src_pos: int| 0 <= src_pos < first_pos && #[trigger] order[src_pos] == dep;
            assert(states[order[src_pos] as int] == crate::verified::codes::status::STATE_COMPLETED);
        };
        assert(crate::verified::primitive::validity::dependencies_complete_spec(dependencies[u as int]@, states));
        let a = crate::verified::codes::action::ACTION_MARK_READY;
        assert(crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies, states, suspensions, profile, u, a, None, Seq::<usize>::empty(), Seq::<usize>::empty(),
        ));
        accepted_linear_progress_strictly_decreases_steps_remaining(
            dependencies, states, suspensions, profile, u, a, None, Seq::<usize>::empty(), Seq::<usize>::empty(),
        );
    }
}

proof fn admitted_order_element_in_bounds(order: Seq<usize>, node_count: usize, pos: int)
    requires
        crate::verified_topology::cardinality::all_nodes_once_spec(order, node_count),
        0 <= pos < order.len(),
    ensures
        order[pos] < node_count,
{
    // Every node 0..node_count appears at a unique position in order, and order.len() == node_count.
    // Count how many positions in order hold values < node_count; it must equal node_count == order.len().
    let in_bounds_set = Set::new(|p: int| 0 <= p < order.len() && order[p] < node_count);
    let nodes_set = Set::new(|n: int| 0 <= n < node_count as int);
    let pos_of = |n: int| -> int {
        choose|p: int| 0 <= p < order.len() && order[p] == n as usize
    };
    assert forall|n: int| nodes_set.contains(n) implies
        in_bounds_set.contains(pos_of(n)) && order[pos_of(n)] == n as usize by {
        all_nodes_once_implies_node_occurs_once(order, node_count, 0, n as usize);
        occurrence_one_has_unique_position(order, n as usize, 0);
        let p = choose|p: int|
            0 <= p < order.len()
                && #[trigger] order[p] == n as usize
                && (forall|other: int| 0 <= other < order.len() && other != p ==> order[other] != n as usize);
        assert(order[p] == n as usize);
    };
    let mapped = nodes_set.map(pos_of);
    assert(mapped.subset_of(in_bounds_set));
    let full_set = Set::new(|p: int| 0 <= p < order.len());
    assert(in_bounds_set.subset_of(full_set));
    vstd::set_lib::lemma_int_range(0, node_count as int);
    vstd::set_lib::lemma_int_range(0, order.len() as int);
    assert(nodes_set =~= vstd::set_lib::set_int_range(0, node_count as int));
    assert(full_set =~= vstd::set_lib::set_int_range(0, order.len() as int));
    vstd::set_lib::lemma_map_size(nodes_set, mapped, pos_of);
    assert(mapped.len() == node_count as nat);
    vstd::set_lib::lemma_len_subset(mapped, in_bounds_set);
    vstd::set_lib::lemma_len_subset(in_bounds_set, full_set);
    assert(in_bounds_set.len() == full_set.len());
    if !in_bounds_set.contains(pos) {
        assert(full_set.contains(pos));
        assert(in_bounds_set.subset_of(full_set.remove(pos)));
        vstd::set_lib::lemma_len_subset(in_bounds_set, full_set.remove(pos));
        assert(false);
    }
}

pub proof fn ready_node_persistence_no_starvation(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    ready_node: usize,
    other_node: usize,
    other_action: u8,
    other_fact: Option<crate::verified::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires
        crate::verified::readiness::dependency_ready_spec(dependencies, states, ready_node),
        crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            other_node,
            other_action,
            other_fact,
            allowed_intents,
            requested_intents,
        ),
        other_node != ready_node,
    ensures
        crate::verified::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            ready_node,
            crate::verified::codes::action::ACTION_MARK_READY,
            None,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
        ({
            let next_states = crate::verified::kernel::decision::indexed_step_next_spec(
                dependencies,
                states,
                suspensions,
                profile,
                other_node,
                other_action,
                other_fact,
                allowed_intents,
                requested_intents,
            );
            let next_suspensions = crate::verified::kernel::decision::indexed_step_next_suspensions_spec(
                dependencies,
                states,
                suspensions,
                profile,
                other_node,
                other_action,
                other_fact,
                allowed_intents,
                requested_intents,
            );
            crate::verified::readiness::dependency_ready_spec(dependencies, next_states, ready_node)
                && crate::verified::kernel::decision::indexed_step_accepts_spec(
                    dependencies,
                    next_states,
                    next_suspensions,
                    profile,
                    ready_node,
                    crate::verified::codes::action::ACTION_MARK_READY,
                    None,
                    Seq::<usize>::empty(),
                    Seq::<usize>::empty(),
                )
        }),
{
    crate::verified::kernel::theorem::accepted_step_preserves_invariant_and_effects(
        dependencies,
        states,
        suspensions,
        profile,
        other_node,
        other_action,
        other_fact,
        allowed_intents,
        requested_intents,
    );
    let next_states = crate::verified::kernel::decision::indexed_step_next_spec(
        dependencies,
        states,
        suspensions,
        profile,
        other_node,
        other_action,
        other_fact,
        allowed_intents,
        requested_intents,
    );
    assert(next_states[ready_node as int] == states[ready_node as int]);
    assert forall|index: int| 0 <= index < dependencies[ready_node as int].len() implies {
        let dep = #[trigger] dependencies[ready_node as int][index];
        dep < next_states.len() && next_states[dep as int] == crate::verified::codes::status::STATE_COMPLETED
    } by {
        let dep = dependencies[ready_node as int][index];
        assert(states[dep as int] == crate::verified::codes::status::STATE_COMPLETED);
        assert(crate::verified::codes::status::terminal_state_spec(states[dep as int]));
        assert(next_states[dep as int] == states[dep as int]);
    };
}

pub proof fn fair_linear_progress_trace_bounded_termination(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<crate::verified_trace::specification::IndexedReplayEvent>,
    from: int,
)
    requires
        0 <= from <= events.len(),
        crate::verified::primitive::validity::indexed_invariant_spec(dependencies, states),
        crate::verified::primitive::support::suspensions_consistent_spec(states, suspensions),
        forall|index: int| from <= index < events.len()
            ==> linear_progress_action_spec((#[trigger] events[index]).action),
    ensures
        crate::verified_trace::specification::replay_prefix_len_from_spec(
            dependencies,
            states,
            suspensions,
            profile,
            events,
            from,
        ) <= linear_steps_remaining_spec(states),
        linear_steps_remaining_spec(states) <= 3 * states.len(),
        (crate::verified_trace::specification::replay_prefix_len_from_spec(
            dependencies,
            states,
            suspensions,
            profile,
            events,
            from,
        ) == linear_steps_remaining_spec(states))
            ==> all_terminal_spec(
                crate::verified_trace::specification::replay_final_states_from_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    events,
                    from,
                ),
            ),
    decreases events.len() - from
{
    assert forall|index: int| 0 <= index < states.len() implies
        crate::verified::codes::status::local_state_valid_spec(#[trigger] states[index]) by {
        assert(crate::verified::primitive::validity::node_invariant_spec(dependencies, states, index));
    };
    linear_steps_remaining_bounded_and_zero_iff_terminal(states);
    if from < events.len() {
        let event = events[from];
        if crate::verified_trace::specification::replay_event_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            event,
        ) {
            crate::verified::kernel::theorem::accepted_step_preserves_invariant_and_effects(
                dependencies,
                states,
                suspensions,
                profile,
                event.node_index,
                event.action,
                event.event_fact,
                event.allowed_intents@,
                event.requested_intents@,
            );
            accepted_linear_progress_strictly_decreases_steps_remaining(
                dependencies,
                states,
                suspensions,
                profile,
                event.node_index,
                event.action,
                event.event_fact,
                event.allowed_intents@,
                event.requested_intents@,
            );
            let next_states = crate::verified_trace::specification::replay_event_next_spec(
                dependencies,
                states,
                suspensions,
                profile,
                event,
            );
            let next_suspensions = crate::verified_trace::specification::replay_event_next_suspensions_spec(
                dependencies,
                states,
                suspensions,
                profile,
                event,
            );
            fair_linear_progress_trace_bounded_termination(
                dependencies,
                next_states,
                next_suspensions,
                profile,
                events,
                from + 1,
            );
        }
    }
}

} // verus!
