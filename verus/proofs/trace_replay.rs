#![no_std]

// r[verify choregraph.proof.trace.lemmas]
// r[verify choregraph.proof.trace.preservation]
// r[verify choregraph.proof.trace.determinism]
// r[verify choregraph.proof.trace.failure]
// r[verify choregraph.proof.trace.replay]
// r[verify choregraph.proof.trace.evidence]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;
