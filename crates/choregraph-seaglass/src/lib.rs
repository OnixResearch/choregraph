#![no_std]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "legacy-shaped DTO names remain concise while owner paths stay explicit at conversion boundaries"
)]

extern crate alloc;

// r[impl choregraph.conformance.adapter]
// r[impl choregraph.conformance.inventory]
pub mod admission;
pub mod aggregate;
pub mod boundary;
pub mod lifecycle;
pub mod topology;
pub mod v2;
