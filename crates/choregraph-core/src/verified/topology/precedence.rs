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
use vstd::prelude::*;

verus! {

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

pub open spec fn topological_order_spec(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
) -> bool {
    super::cardinality::all_nodes_once_spec(order, node_count)
        && edges_forward_spec(edges, node_count, order)
}

pub fn topological_order_valid(
    edges: &[(usize, usize)],
    node_count: usize,
    order: &[usize],
) -> (result: bool)
    ensures result == topological_order_spec(edges@, node_count, order@)
{
    super::cardinality::all_nodes_once(order, node_count) && edges_forward(edges, node_count, order)
}

} // verus!
