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

pub open spec fn admitted_graph_spec(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    dependencies: Seq<alloc::vec::Vec<usize>>,
) -> bool {
    super::precedence::topological_order_spec(edges, node_count, order)
        && super::bounds::dependency_indices_valid_spec(dependencies, node_count)
}

pub fn admitted_graph_valid(
    edges: &[(usize, usize)],
    node_count: usize,
    order: &[usize],
    dependencies: &[alloc::vec::Vec<usize>],
) -> (result: bool)
    ensures result == admitted_graph_spec(edges@, node_count, order@, dependencies@)
{
    super::precedence::topological_order_valid(edges, node_count, order)
        && super::bounds::dependency_indices_valid(dependencies, node_count)
}

// r[verify choregraph.proof.single_step.graph]
pub proof fn admitted_topology_has_each_node_once_and_edges_forward(
    edges: Seq<(usize, usize)>,
    node_count: usize,
    order: Seq<usize>,
    dependencies: Seq<alloc::vec::Vec<usize>>,
)
    requires admitted_graph_spec(edges, node_count, order, dependencies)
    ensures
        super::cardinality::all_nodes_once_spec(order, node_count),
        super::precedence::edges_forward_spec(edges, node_count, order),
        super::bounds::dependency_indices_valid_spec(dependencies, node_count),
{}

} // verus!
