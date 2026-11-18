#![no_std]

// r[verify choregraph.proof.single_step.specifications]
// r[verify choregraph.proof.single_step.production_linkage]
// r[verify choregraph.proof.single_step.trellis]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

#[path = "../../crates/choregraph-core/src/verified/topology.rs"]
mod verified_topology;
