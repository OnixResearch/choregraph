#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/topology.rs"]
mod verified_topology;

#[path = "../../crates/choregraph-core/src/verified/trace.rs"]
mod verified_trace;

#[path = "liveness_lemmas.rs"]
mod verified_liveness;
