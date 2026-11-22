#![no_std]
// This crate preserves the published compatibility surface and its exact legacy test fixtures.
#![allow(
    unknown_lints,
    ambiguous_params,
    explicit_defaults,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    numeric_units,
    reason = "the compatibility facade retains published signatures and exact legacy fixtures"
)]

// r[impl choregraph.execution_graph_compatibility.boundary]
extern crate alloc;

mod adapter;

pub mod admission;
pub mod aggregate;
pub mod lifecycle;
pub mod queries;
pub mod structure;

#[cfg(test)]
mod tests;
