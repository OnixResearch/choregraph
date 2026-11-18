#![no_std]

extern crate alloc;

mod admission;
pub mod analytics;
pub mod boundary;
pub mod concrete;
pub mod id;
pub mod invariant;
pub mod model;
pub mod ownership;
pub mod transition;
mod verified;
#[path = "verified/topology.rs"]
mod verified_topology;
#[path = "verified/trace.rs"]
mod verified_trace;
