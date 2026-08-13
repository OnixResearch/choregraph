#![no_std]

//! Immutable branchable event history with no persistence or runtime authority.

extern crate alloc;

// r[impl choregraph.history.boundary]
// r[impl choregraph.history.boundary.pure]
// r[verify choregraph.history.boundary.rejects_authority]
// r[verify choregraph.history.supersession.rejects_dependency]
mod dag;
mod identity;
mod model;
mod refs;
mod traversal;

pub use dag::{Admission, HistoryGraph};
pub use identity::{DIGEST_BYTES, Digest, EventId};
pub use model::{
    EventClass, EventDraft, HISTORY_EVENT_VERSION_V1, HistoryError, HistoryEvent, HistoryLimits,
    MergeEnvelope, MetadataIdentity, ParentLink,
};
pub use refs::{
    BranchRef, Cursor, ReferenceMutationPlan, Snapshot, plan_branch_advance, validate_cursor,
};
pub use traversal::{ParentChoices, Projection, ProjectionKind, ancestor_closure, linear_lineage};

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
