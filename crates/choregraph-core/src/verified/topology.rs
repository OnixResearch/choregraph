// Verus topology proofs require these local indexed-arithmetic and specification shapes.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    clippy::assign_op_pattern,
    compound_condition,
    non_trait_imports,
    raw_arithmetic_overflow,
    usize_in_public_api,
    reason = "Verus topology proofs require these local indexed-arithmetic and specification shapes"
)]

// r[impl choregraph.proof.single_step.specifications]
// r[impl choregraph.proof.single_step.graph]
// r[impl choregraph.proof.single_step.production_linkage]
// r[impl choregraph.proof.single_step.trellis]
//
// Locally proved adaptation of the finite topological-order checker shape in
// Trellis src/topo_sort.rs at revision aead4527bfc3c714e71669bce656719dd2a6b2e2.
use alloc::vec::Vec;
use vstd::prelude::*;

verus! {

pub const OCCURRENCE_NONE: u8 = 0u8;
pub const OCCURRENCE_ONE: u8 = 1u8;
pub const OCCURRENCE_MANY: u8 = 2u8;

pub open spec fn occurrence_class_from_spec(
    order: Seq<usize>,
    node: usize,
    from: int,
) -> u8
    decreases order.len() - from
{
    if from < 0 || from >= order.len() {
        OCCURRENCE_NONE
    } else {
        let tail = occurrence_class_from_spec(order, node, from + 1);
        if order[from] != node {
            tail
        } else if tail == OCCURRENCE_NONE {
            OCCURRENCE_ONE
        } else {
            OCCURRENCE_MANY
        }
    }
}

pub open spec fn occurrence_class_spec(order: Seq<usize>, node: usize) -> u8 {
    occurrence_class_from_spec(order, node, 0)
}

pub fn occurrence_class(order: &[usize], node: usize) -> (result: u8)
    ensures result == occurrence_class_spec(order@, node)
{
    let mut result = OCCURRENCE_NONE;
    let mut index = order.len();
    while index > 0
        invariant
            0 <= index <= order@.len(),
            result == occurrence_class_from_spec(order@, node, index as int),
        decreases index,
    {
        index = index - 1;
        if order[index] == node {
            if result == OCCURRENCE_NONE {
                result = OCCURRENCE_ONE;
            } else {
                result = OCCURRENCE_MANY;
            }
        }
    }
    result
}

pub open spec fn all_nodes_once_from_spec(
    order: Seq<usize>,
    node_count: usize,
    node: int,
) -> bool
    decreases node_count as int - node
{
    if node < 0 || node >= node_count as int {
        true
    } else {
        occurrence_class_spec(order, node as usize) == OCCURRENCE_ONE
            && all_nodes_once_from_spec(order, node_count, node + 1)
    }
}

pub open spec fn all_nodes_once_spec(order: Seq<usize>, node_count: usize) -> bool {
    order.len() == node_count && all_nodes_once_from_spec(order, node_count, 0)
}

pub fn all_nodes_once(order: &[usize], node_count: usize) -> (result: bool)
    ensures result == all_nodes_once_spec(order@, node_count)
{
    if order.len() != node_count {
        return false;
    }
    let mut node: usize = 0;
    while node < node_count
        invariant
            order@.len() == node_count,
            0 <= node <= node_count,
            all_nodes_once_spec(order@, node_count)
                <==> all_nodes_once_from_spec(order@, node_count, node as int),
        decreases node_count - node,
    {
        if occurrence_class(order, node) != OCCURRENCE_ONE {
            return false;
        }
        node = node + 1;
    }
    true
}

pub open spec fn precedes_from_spec(
    order: Seq<usize>,
    source: usize,
    target: usize,
    source_seen: bool,
    from: int,
) -> bool
    decreases order.len() - from
{
    if from < 0 || from >= order.len() {
        false
    } else if order[from] == target {
        source_seen
    } else {
        precedes_from_spec(
            order,
            source,
            target,
            source_seen || order[from] == source,
            from + 1,
        )
    }
}

pub open spec fn precedes_spec(order: Seq<usize>, source: usize, target: usize) -> bool {
    precedes_from_spec(order, source, target, false, 0)
}

pub fn precedes(order: &[usize], source: usize, target: usize) -> (result: bool)
    ensures result == precedes_spec(order@, source, target)
{
    let mut source_seen = false;
    let mut index: usize = 0;
    while index < order.len()
        invariant
            0 <= index <= order@.len(),
            precedes_spec(order@, source, target)
                == precedes_from_spec(
                    order@,
                    source,
                    target,
                    source_seen,
                    index as int,
                ),
        decreases order@.len() - index,
    {
        let current = order[index];
        if current == target {
            return source_seen;
        }
        if current == source {
            source_seen = true;
        }
        index = index + 1;
    }
    false
}

pub open spec fn edges_forward_from_spec(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    from: int,
) -> bool
    decreases edges.len() - from
{
    if from < 0 || from >= edges.len() {
        true
    } else {
        let edge = edges[from];
        edge.0 < node_count
            && edge.1 < node_count
            && edge.0 != edge.1
            && precedes_spec(order, edge.0, edge.1)
            && edges_forward_from_spec(edges, node_count, order, from + 1)
    }
}

pub open spec fn edges_forward_spec(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
) -> bool {
    edges_forward_from_spec(edges, node_count, order, 0)
}

pub fn edges_forward(
    edges: &[(usize, usize)],
    node_count: usize,
    order: &[usize],
) -> (result: bool)
    ensures result == edges_forward_spec(edges@, node_count, order@)
{
    let mut index: usize = 0;
    while index < edges.len()
        invariant
            0 <= index <= edges@.len(),
            edges_forward_spec(edges@, node_count, order@)
                <==> edges_forward_from_spec(
                    edges@,
                    node_count,
                    order@,
                    index as int,
                ),
        decreases edges@.len() - index,
    {
        let edge = edges[index];
        if edge.0 >= node_count
            || edge.1 >= node_count
            || edge.0 == edge.1
            || !precedes(order, edge.0, edge.1)
        {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn indices_bounded_from_spec(
    indices: Seq<usize>,
    node_count: usize,
    from: int,
) -> bool
    decreases indices.len() - from
{
    if from < 0 || from >= indices.len() {
        true
    } else {
        indices[from] < node_count
            && indices_bounded_from_spec(indices, node_count, from + 1)
    }
}

pub open spec fn indices_bounded_spec(indices: Seq<usize>, node_count: usize) -> bool {
    indices_bounded_from_spec(indices, node_count, 0)
}

pub fn indices_bounded(indices: &[usize], node_count: usize) -> (result: bool)
    ensures result == indices_bounded_spec(indices@, node_count)
{
    let mut index: usize = 0;
    while index < indices.len()
        invariant
            0 <= index <= indices@.len(),
            indices_bounded_spec(indices@, node_count)
                <==> indices_bounded_from_spec(indices@, node_count, index as int),
        decreases indices@.len() - index,
    {
        if indices[index] >= node_count {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn dependency_indices_valid_from_spec(
    dependencies: Seq<Vec<usize>>,
    node_count: usize,
    from: int,
) -> bool
    decreases dependencies.len() - from
{
    if from < 0 || from >= dependencies.len() {
        true
    } else {
        indices_bounded_spec(dependencies[from]@, node_count)
            && dependency_indices_valid_from_spec(dependencies, node_count, from + 1)
    }
}

pub open spec fn dependency_indices_valid_spec(
    dependencies: Seq<Vec<usize>>,
    node_count: usize,
) -> bool {
    dependencies.len() == node_count
        && dependency_indices_valid_from_spec(dependencies, node_count, 0)
}

pub fn dependency_indices_valid(
    dependencies: &[Vec<usize>],
    node_count: usize,
) -> (result: bool)
    ensures result == dependency_indices_valid_spec(dependencies@, node_count)
{
    if dependencies.len() != node_count {
        return false;
    }
    let mut index: usize = 0;
    while index < dependencies.len()
        invariant
            dependencies@.len() == node_count,
            0 <= index <= dependencies@.len(),
            dependency_indices_valid_spec(dependencies@, node_count)
                <==> dependency_indices_valid_from_spec(
                    dependencies@,
                    node_count,
                    index as int,
                ),
        decreases dependencies@.len() - index,
    {
        if !indices_bounded(&dependencies[index], node_count) {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn topological_order_spec(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
) -> bool {
    all_nodes_once_spec(order, node_count)
        && edges_forward_spec(edges, node_count, order)
}

pub fn topological_order_valid(
    edges: &[(usize, usize)],
    node_count: usize,
    order: &[usize],
) -> (result: bool)
    ensures result == topological_order_spec(edges@, node_count, order@)
{
    all_nodes_once(order, node_count) && edges_forward(edges, node_count, order)
}

pub open spec fn admitted_graph_spec(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
) -> bool {
    topological_order_spec(edges, node_count, order)
        && dependency_indices_valid_spec(dependencies, node_count)
}

pub fn admitted_graph_valid(
    edges: &[(usize, usize)],
    node_count: usize,
    order: &[usize],
    dependencies: &[Vec<usize>],
) -> (result: bool)
    ensures result == admitted_graph_spec(edges@, node_count, order@, dependencies@)
{
    topological_order_valid(edges, node_count, order)
        && dependency_indices_valid(dependencies, node_count)
}

// r[verify choregraph.proof.single_step.graph]
pub proof fn admitted_topology_has_each_node_once_and_edges_forward(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    dependencies: Seq<Vec<usize>>,
)
    requires admitted_graph_spec(edges, node_count, order, dependencies)
    ensures
        all_nodes_once_spec(order, node_count),
        edges_forward_spec(edges, node_count, order),
        dependency_indices_valid_spec(dependencies, node_count),
{}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    const NODE_A: usize = 0;
    const NODE_B: usize = 1;
    const NODE_C: usize = 2;
    const NODE_D: usize = 3;
    const NODE_COUNT: usize = 4;

    #[test]
    fn topological_checker_accepts_a_diamond() {
        let edges = [
            (NODE_A, NODE_B),
            (NODE_A, NODE_C),
            (NODE_B, NODE_D),
            (NODE_C, NODE_D),
        ];
        let order = [NODE_A, NODE_B, NODE_C, NODE_D];

        assert!(topological_order_valid(&edges, NODE_COUNT, &order));
    }

    #[test]
    fn topological_checker_rejects_duplicates_and_reversed_edges() {
        let edges = [(NODE_A, NODE_B), (NODE_B, NODE_C)];
        let duplicate_order = [NODE_A, NODE_B, NODE_B];
        let reversed_order = [NODE_C, NODE_B, NODE_A];

        assert!(!topological_order_valid(
            &edges,
            duplicate_order.len(),
            &duplicate_order,
        ));
        assert!(!topological_order_valid(
            &edges,
            reversed_order.len(),
            &reversed_order,
        ));
    }
}
