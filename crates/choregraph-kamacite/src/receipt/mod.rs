// Receipt validation keeps explicit exhaustive policy checks and canonical compatibility paths.
#![allow(
    unknown_lints,
    assertion_density,
    compound_condition,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    reason = "receipt validation keeps explicit exhaustive policy checks and canonical compatibility paths"
)]

// r[impl choregraph.interchange.kamacite.receipts]
pub mod record;
pub mod validation;
