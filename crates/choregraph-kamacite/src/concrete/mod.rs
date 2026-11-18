// Canonical v2 machine, concrete-intent, and linkage receipt adapters.
#![allow(
    unknown_lints,
    assertion_density,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "the canonical v2 adapter keeps explicit closed-schema conversion and linkage checks"
)]

// r[impl choregraph.effect_v2.kamacite]
// r[impl choregraph.effect_v2.references]
// r[impl choregraph.effect_v2.occurrence_identity]
pub mod conversion;
pub mod record;
