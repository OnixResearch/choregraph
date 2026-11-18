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

} // verus!
