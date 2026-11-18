#![no_std]

//! Immutable branchable event history with no persistence or runtime authority.

extern crate alloc;

// r[impl choregraph.history.boundary]
// r[impl choregraph.history.boundary.pure]
// r[verify choregraph.history.boundary.rejects_authority]
// r[verify choregraph.history.supersession.rejects_dependency]
pub mod dag;
pub mod identity;
pub mod model;
pub mod refs;
pub mod traversal;

/// The bounded semantic claim of this crate.
pub const HISTORY_CLAIM: &str =
    "pure immutable graph admission, explicit lineage selection, and reference mutation plans";

/// Authority that remains with consumers and hosts.
pub const HISTORY_NON_CLAIMS: &[&str] = &[
    "payload truth or authenticity",
    "application merge correctness",
    "persistence durability",
    "distributed convergence",
    "effect execution or success",
    "authorization",
    "consumer correctness",
    "release eligibility",
];
