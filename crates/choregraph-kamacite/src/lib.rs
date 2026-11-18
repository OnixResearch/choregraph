//! Pure canonical artifact and receipt adapter for the selected Kamacite profile.
//!
//! The crate owns no filesystem, process, network, clock, policy, or runtime-effect authority.

pub mod aggregate;
pub mod boundary;
pub mod codec;
pub mod concrete;
pub mod identity;
pub mod lifecycle;
pub mod profile;
pub mod receipt;

#[cfg(test)]
#[path = "../test-support/internal.rs"]
mod tests;

// The reviewed producer correction is replayed from the exact base by the owned importer.
#[allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    dead_code,
    explicit_defaults,
    fragile_exhaustive_enum_match,
    function_length,
    missing_const_fn,
    non_trait_imports,
    path_segment_repetition,
    renamed_imports,
    unbounded_collection_growth,
    reason = "the complete codec retains its reviewed producer correction and reproducible import boundary"
)]
#[path = "../../../vendor/kamacite-core-selected/preserves/codec.rs"]
mod selected_kamacite_codec;
