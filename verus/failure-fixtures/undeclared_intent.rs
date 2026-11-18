#![no_std]

extern crate alloc;

#[path = "../../crates/choregraph-core/src/verified.rs"]
mod verified;

use verified::{primitive::membership::effects_allowed_spec};
use vstd::prelude::*;

verus! {

pub const DECLARED_INTENT: usize = 0usize;
pub const UNDECLARED_INTENT: usize = 1usize;

// Failure-path evidence only. Real Verus must reject an undeclared intent.
pub proof fn undeclared_intent_must_fail()
    ensures effects_allowed_spec(
        seq![DECLARED_INTENT],
        seq![UNDECLARED_INTENT],
    ),
{}

} // verus!
