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
    dependencies: Seq<alloc::vec::Vec<usize>>,
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
    dependencies: Seq<alloc::vec::Vec<usize>>,
    node_count: usize,
) -> bool {
    dependencies.len() == node_count
        && dependency_indices_valid_from_spec(dependencies, node_count, 0)
}

pub fn dependency_indices_valid(
    dependencies: &[alloc::vec::Vec<usize>],
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

} // verus!
