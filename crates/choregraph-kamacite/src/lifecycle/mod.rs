// Lifecycle artifacts preserve closed canonical tables and deterministic encoded ordering.
#![allow(
    unknown_lints,
    allocation_in_loop,
    assertion_density,
    function_length,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth,
    reason = "lifecycle artifacts preserve closed canonical tables and deterministic encoded ordering"
)]

// r[impl choregraph.interchange.kamacite.lifecycle]
pub mod conversion;
pub mod record;
