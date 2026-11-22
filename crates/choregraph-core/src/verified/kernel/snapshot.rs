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

pub open spec fn suspension_fact_matches_spec(
    stored: Option<super::super::primitive::support::IndexedSuspensionFact>,
    event: Option<super::super::primitive::support::IndexedSuspensionFact>,
) -> bool {
    stored.is_some() && event.is_some() && stored.unwrap().class@ == event.unwrap().class@
        && stored.unwrap().reference@ == event.unwrap().reference@
}

pub fn suspension_fact_matches(
    stored: &Option<super::super::primitive::support::IndexedSuspensionFact>,
    event: &Option<super::super::primitive::support::IndexedSuspensionFact>,
) -> (result: bool)
    ensures result == suspension_fact_matches_spec(*stored, *event)
{
    match (stored, event) {
        (Some(stored_fact), Some(event_fact)) => {
            let is_class_equal = super::super::primitive::support::bytes_equal(&stored_fact.class, &event_fact.class);
            let is_reference_equal = super::super::primitive::support::bytes_equal(&stored_fact.reference, &event_fact.reference);
            is_class_equal && is_reference_equal
        }
        _ => false,
    }
}

pub open spec fn suspension_fact_shape_spec(
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
) -> bool {
    if super::super::codes::action::suspension_action_spec(action) {
        event_fact.is_some()
    } else {
        event_fact.is_none()
    }
}

pub open spec fn suspension_effects_forbidden_spec(profile: u8, from: u8, action: u8) -> bool {
    super::super::codes::action::suspension_action_spec(action)
        || (profile == super::super::codes::profile::PROFILE_DURABLE_SUSPENSION_V2
            && from == super::super::codes::status::STATE_SUSPENDED
            && action == super::super::codes::action::ACTION_CANCEL)
}

pub const fn suspension_effects_forbidden(profile: u8, from: u8, action: u8) -> (result: bool)
    ensures result == suspension_effects_forbidden_spec(profile, from, action)
{
    action == super::super::codes::action::ACTION_SUSPEND
        || action == super::super::codes::action::ACTION_RESUME_SIGNAL
        || (profile == super::super::codes::profile::PROFILE_DURABLE_SUSPENSION_V2
            && from == super::super::codes::status::STATE_SUSPENDED
            && action == super::super::codes::action::ACTION_CANCEL)
}

pub open spec fn suspension_binding_spec(
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
) -> bool {
    if action == super::super::codes::action::ACTION_RESUME_SIGNAL {
        suspension_fact_matches_spec(suspensions[node_index as int], event_fact)
    } else {
        true
    }
}

pub open spec fn step_next_suspensions_spec(
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
) -> Seq<Option<super::super::primitive::support::IndexedSuspensionFact>> {
    if action == super::super::codes::action::ACTION_SUSPEND {
        suspensions.update(node_index as int, event_fact)
    } else if action == super::super::codes::action::ACTION_RESUME_SIGNAL
        || action == super::super::codes::action::ACTION_CANCEL
    {
        suspensions.update(node_index as int, None)
    } else {
        suspensions
    }
}

// r[verify choregraph.machine.suspension_proof]
pub proof fn suspension_update_preserves_consistency(
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    target: u8,
)
    requires
        super::super::codes::profile::profile_valid_spec(profile),
        super::super::primitive::support::suspensions_consistent_spec(states, suspensions),
        node_index < states.len(),
        super::super::primitive::transition::lifecycle_target_spec(profile, states[node_index as int], action) == Some(target),
        super::super::codes::action::suspension_action_spec(action) ==> event_fact.is_some(),
    ensures super::super::primitive::support::suspensions_consistent_spec(
        super::super::kernel::target::step_next_states_spec(states, node_index, target),
        step_next_suspensions_spec(suspensions, node_index, action, event_fact),
    )
{
    let next_states = super::super::kernel::target::step_next_states_spec(states, node_index, target);
    let next_suspensions = step_next_suspensions_spec(suspensions, node_index, action, event_fact);
    assert forall|index: int| 0 <= index < next_states.len() implies (next_suspensions[index].is_some()
        == (next_states[index] == super::super::codes::status::STATE_SUSPENDED)) by {
        if index == node_index as int {
            if action == super::super::codes::action::ACTION_SUSPEND {
                assert(target == super::super::codes::status::STATE_SUSPENDED);
            } else if action == super::super::codes::action::ACTION_RESUME_SIGNAL {
                assert(target == super::super::codes::status::STATE_READY);
            } else {
                assert(target != super::super::codes::status::STATE_SUSPENDED);
            }
        }
    }
}

// r[verify choregraph.machine.suspension_proof]
pub proof fn suspension_transitions_emit_no_effects(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires
        super::super::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            action,
            event_fact,
            allowed_intents,
            requested_intents,
        ),
        node_index < states.len(),
        suspension_effects_forbidden_spec(profile, states[node_index as int], action),
    ensures super::super::kernel::decision::indexed_step_intents_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        action,
        event_fact,
        allowed_intents,
        requested_intents,
    ).len() == 0,
{}

} // verus!
