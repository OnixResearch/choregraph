#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod production_verified;

use production_verified::{IndexedConcreteReferenceWitness, concrete_reference_witness};
use vstd::prelude::*;

verus! {

pub const EVENT_REQUEST_REFERENCE: usize = 7usize;
pub const EVENT_PAYLOAD_REFERENCE: usize = 11usize;
pub const MUTATED_PAYLOAD_REFERENCE: usize = 13usize;

// Failure-path evidence only. Real Verus must reject a changed payload reference.
pub fn concrete_reference_mutation_must_fail() -> (result: IndexedConcreteReferenceWitness)
    ensures
        result.request_reference == EVENT_REQUEST_REFERENCE,
        result.payload_reference == MUTATED_PAYLOAD_REFERENCE,
{
    concrete_reference_witness(EVENT_REQUEST_REFERENCE, EVENT_PAYLOAD_REFERENCE)
}

} // verus!
