// Verus proof syntax and production-linked specifications require these local source shapes.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    clippy::assign_op_pattern,
    dead_code,
    nested_conditionals,
    non_trait_imports,
    raw_arithmetic_overflow,
    too_many_parameters,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus proof syntax and production-linked proof witnesses require these local source shapes"
)]

use vstd::prelude::*;

verus! {

pub const BINDING_EXACT: u8 = 0u8;

pub const BINDING_SLOT: u8 = 1u8;

#[derive(Debug, Eq, PartialEq)]
pub struct IndexedConcreteReferenceWitness {
    pub request_reference: usize,
    pub payload_reference: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcreteBindingFacts {
    pub slot_matches: bool,
    pub source_matches: bool,
    pub exact_reference_matches: bool,
    pub slot_absent: bool,
    pub source_absent: bool,
}

pub open spec fn concrete_binding_accepts_spec(
    mode: u8,
    facts: ConcreteBindingFacts,
) -> bool {
    if mode == BINDING_EXACT {
        facts.slot_absent && facts.source_absent && facts.exact_reference_matches
    } else if mode == BINDING_SLOT {
        facts.slot_matches && facts.source_matches
    } else {
        false
    }
}

pub fn concrete_binding_accepts(
    mode: u8,
    facts: ConcreteBindingFacts,
) -> (result: bool)
    ensures result == concrete_binding_accepts_spec(mode, facts)
{
    if mode == BINDING_EXACT {
        facts.slot_absent && facts.source_absent && facts.exact_reference_matches
    } else if mode == BINDING_SLOT {
        facts.slot_matches && facts.source_matches
    } else {
        false
    }
}

pub open spec fn concrete_references_preserved_spec(
    event_request_reference: usize,
    event_payload_reference: usize,
    intent_request_reference: usize,
    intent_payload_reference: usize,
) -> bool {
    intent_request_reference == event_request_reference
        && intent_payload_reference == event_payload_reference
}

pub fn concrete_reference_witness(
    event_request_reference: usize,
    event_payload_reference: usize,
) -> (result: IndexedConcreteReferenceWitness)
    ensures concrete_references_preserved_spec(
        event_request_reference,
        event_payload_reference,
        result.request_reference,
        result.payload_reference,
    )
{
    IndexedConcreteReferenceWitness {
        request_reference: event_request_reference,
        payload_reference: event_payload_reference,
    }
}

} // verus!
