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

#[path = "topology/admission.rs"]
pub(crate) mod admission;
#[path = "topology/bounds.rs"]
pub(crate) mod bounds;
#[path = "topology/cardinality.rs"]
pub(crate) mod cardinality;
#[path = "topology/precedence.rs"]
pub(crate) mod precedence;

#[cfg(test)]
mod tests {
    use super::precedence::topological_order_valid;

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
