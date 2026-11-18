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

pub open spec fn indexed_step_accepts_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> bool {
    super::super::primitive::validity::indexed_invariant_spec(dependencies, states)
        && super::super::primitive::support::suspensions_consistent_spec(states, suspensions)
        && super::super::codes::profile::profile_valid_spec(profile)
        && node_index < states.len()
        && super::super::kernel::target::step_target_spec(profile, states, node_index, action).is_some()
        && super::super::kernel::snapshot::suspension_fact_shape_spec(action, event_fact)
        && super::super::kernel::snapshot::suspension_effects_spec(action, requested_intents)
        && super::super::kernel::target::step_dependencies_ready_spec(dependencies, states, node_index, action)
        && super::super::kernel::snapshot::suspension_binding_spec(suspensions, node_index, action, event_fact)
        && super::super::primitive::membership::effects_allowed_spec(allowed_intents, requested_intents)
        && super::super::primitive::validity::indexed_invariant_spec(
            dependencies,
            super::super::kernel::target::step_next_states_spec(
                states,
                node_index,
                super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        )
        && super::super::primitive::validity::terminal_states_preserved_spec(
            states,
            super::super::kernel::target::step_next_states_spec(
                states,
                node_index,
                super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        )
        && super::super::primitive::support::suspensions_consistent_spec(
            super::super::kernel::target::step_next_states_spec(
                states,
                node_index,
                super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
            ),
            super::super::kernel::snapshot::step_next_suspensions_spec(suspensions, node_index, action, event_fact),
        )
}

pub open spec fn indexed_step_reason_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> u8 {
    if !super::super::primitive::validity::indexed_invariant_spec(dependencies, states)
        || !super::super::primitive::support::suspensions_consistent_spec(states, suspensions)
        || !super::super::codes::profile::profile_valid_spec(profile)
    {
        super::super::codes::denial::transition::DENIAL_INVALID_STATE
    } else if node_index >= states.len() {
        super::super::codes::denial::transition::DENIAL_UNKNOWN_NODE
    } else if super::super::codes::status::terminal_state_spec(states[node_index as int]) {
        super::super::codes::denial::transition::DENIAL_TERMINAL_STATE
    } else if super::super::codes::action::suspension_action_spec(action) && profile == super::super::codes::profile::PROFILE_LINEAR_V1 {
        super::super::codes::denial::suspension::DENIAL_PROFILE_ACTION
    } else if super::super::kernel::target::step_target_spec(profile, states, node_index, action).is_none() {
        super::super::codes::denial::transition::DENIAL_INVALID_TRANSITION
    } else if super::super::codes::action::suspension_action_spec(action) && event_fact.is_none() {
        super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_ABSENT
    } else if !super::super::codes::action::suspension_action_spec(action) && event_fact.is_some() {
        super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_PRESENT
    } else if !super::super::kernel::snapshot::suspension_effects_spec(action, requested_intents) {
        super::super::codes::denial::suspension::DENIAL_SUSPENSION_EFFECTS
    } else if !super::super::kernel::target::step_dependencies_ready_spec(dependencies, states, node_index, action) {
        super::super::codes::denial::transition::DENIAL_DEPENDENCIES_INCOMPLETE
    } else if !super::super::kernel::snapshot::suspension_binding_spec(suspensions, node_index, action, event_fact) {
        super::super::codes::denial::suspension::DENIAL_SUSPENSION_MISMATCH
    } else if !super::super::primitive::membership::all_distinct_spec(requested_intents) {
        super::super::codes::denial::transition::DENIAL_DUPLICATE_EFFECT
    } else if !super::super::primitive::membership::all_requested_allowed_spec(allowed_intents, requested_intents) {
        super::super::codes::denial::transition::DENIAL_UNDECLARED_EFFECT
    } else {
        let target = super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap();
        let next = super::super::kernel::target::step_next_states_spec(states, node_index, target);
        if !super::super::primitive::validity::indexed_invariant_spec(dependencies, next)
            || !super::super::primitive::validity::terminal_states_preserved_spec(states, next)
            || !super::super::primitive::support::suspensions_consistent_spec(
                next,
                super::super::kernel::snapshot::step_next_suspensions_spec(suspensions, node_index, action, event_fact),
            )
        {
            super::super::codes::denial::transition::DENIAL_INVARIANT_WOULD_BREAK
        } else {
            super::super::codes::denial::transition::DENIAL_NONE
        }
    }
}

pub open spec fn indexed_step_next_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> Seq<u8> {
    if indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        action,
        event_fact,
        allowed_intents,
        requested_intents,
    ) {
        super::super::kernel::target::step_next_states_spec(
            states,
            node_index,
            super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
        )
    } else {
        Seq::empty()
    }
}

pub open spec fn indexed_step_next_suspensions_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> Seq<Option<super::super::primitive::support::IndexedSuspensionFact>> {
    if indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        action,
        event_fact,
        allowed_intents,
        requested_intents,
    ) {
        super::super::kernel::snapshot::step_next_suspensions_spec(suspensions, node_index, action, event_fact)
    } else {
        Seq::empty()
    }
}

pub open spec fn indexed_step_intents_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> Seq<usize> {
    if indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        action,
        event_fact,
        allowed_intents,
        requested_intents,
    ) {
        requested_intents
    } else {
        Seq::empty()
    }
}

pub fn deny(reason: u8) -> (result: super::super::primitive::support::IndexedDecision)
    ensures
        !result.accepted,
        result.reason == reason,
        result.next_states@.len() == 0,
        result.intent_indices@.len() == 0,
{
    super::super::primitive::support::IndexedDecision {
        accepted: false,
        reason,
        next_states: alloc::vec::Vec::new(),
        intent_indices: alloc::vec::Vec::new(),
    }
}

// Verus ties this body to one complete postcondition set, so keep the proof boundary intact.
#[allow(
    clippy::too_many_arguments,
    function_length,
    reason = "the kernel takes only explicit indexed inputs"
)]
pub fn indexed_step(
    dependencies: &[alloc::vec::Vec<usize>],
    states: &[u8],
    suspensions: &[Option<super::super::primitive::support::IndexedSuspensionFact>],
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: &Option<super::super::primitive::support::IndexedSuspensionFact>,
    allowed_intents: &[usize],
    requested_intents: &[usize],
) -> (result: super::super::primitive::support::IndexedDecision)
    ensures
        result.accepted == indexed_step_accepts_spec(
            dependencies@,
            states@,
            suspensions@,
            profile,
            node_index,
            action,
            *event_fact,
            allowed_intents@,
            requested_intents@,
        ),
        result.reason == indexed_step_reason_spec(
            dependencies@,
            states@,
            suspensions@,
            profile,
            node_index,
            action,
            *event_fact,
            allowed_intents@,
            requested_intents@,
        ),
        result.next_states@ == indexed_step_next_spec(
            dependencies@,
            states@,
            suspensions@,
            profile,
            node_index,
            action,
            *event_fact,
            allowed_intents@,
            requested_intents@,
        ),
        result.intent_indices@ == indexed_step_intents_spec(
            dependencies@,
            states@,
            suspensions@,
            profile,
            node_index,
            action,
            *event_fact,
            allowed_intents@,
            requested_intents@,
        ),
        result.accepted ==> super::super::primitive::validity::indexed_invariant_spec(dependencies@, result.next_states@),
        result.accepted ==> super::super::primitive::validity::terminal_states_preserved_spec(states@, result.next_states@),
        result.accepted ==> super::super::primitive::support::suspensions_consistent_spec(
            result.next_states@,
            super::super::kernel::snapshot::step_next_suspensions_spec(suspensions@, node_index, action, *event_fact),
        ),
        result.accepted ==> super::super::primitive::membership::effects_allowed_spec(allowed_intents@, result.intent_indices@),
        result.accepted ==> result.intent_indices@ == requested_intents@,
        !result.accepted ==> result.next_states@.len() == 0,
        !result.accepted ==> result.intent_indices@.len() == 0,
{
    if !super::super::primitive::validity::indexed_invariant(dependencies, states)
        || !super::super::primitive::support::suspensions_consistent(states, suspensions)
        || !(profile == super::super::codes::profile::PROFILE_LINEAR_V1 || profile == super::super::codes::profile::PROFILE_DURABLE_SUSPENSION_V1)
    {
        return deny(super::super::codes::denial::transition::DENIAL_INVALID_STATE);
    }
    if node_index >= states.len() {
        return deny(super::super::codes::denial::transition::DENIAL_UNKNOWN_NODE);
    }
    let from = states[node_index];
    if super::super::codes::status::terminal_state(from) {
        return deny(super::super::codes::denial::transition::DENIAL_TERMINAL_STATE);
    }
    let is_suspension_action = action == super::super::codes::action::ACTION_SUSPEND || action == super::super::codes::action::ACTION_RESUME_SIGNAL;
    if is_suspension_action && profile == super::super::codes::profile::PROFILE_LINEAR_V1 {
        return deny(super::super::codes::denial::suspension::DENIAL_PROFILE_ACTION);
    }
    let target = match super::super::primitive::transition::lifecycle_target(profile, from, action) {
        Some(target) => target,
        None => return deny(super::super::codes::denial::transition::DENIAL_INVALID_TRANSITION),
    };
    if is_suspension_action && event_fact.is_none() {
        return deny(super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_ABSENT);
    }
    if !is_suspension_action && event_fact.is_some() {
        return deny(super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_PRESENT);
    }
    if is_suspension_action && !requested_intents.is_empty() {
        return deny(super::super::codes::denial::suspension::DENIAL_SUSPENSION_EFFECTS);
    }
    if (action == super::super::codes::action::ACTION_MARK_READY || action == super::super::codes::action::ACTION_START || action == super::super::codes::action::ACTION_RESUME_SIGNAL)
        && !super::super::primitive::validity::dependencies_complete(&dependencies[node_index], states)
    {
        return deny(super::super::codes::denial::transition::DENIAL_DEPENDENCIES_INCOMPLETE);
    }
    if action == super::super::codes::action::ACTION_RESUME_SIGNAL
        && !super::super::kernel::snapshot::suspension_fact_matches(&suspensions[node_index], event_fact)
    {
        return deny(super::super::codes::denial::suspension::DENIAL_SUSPENSION_MISMATCH);
    }
    if !super::super::primitive::membership::all_distinct(requested_intents) {
        return deny(super::super::codes::denial::transition::DENIAL_DUPLICATE_EFFECT);
    }
    if !super::super::primitive::membership::all_requested_allowed(allowed_intents, requested_intents) {
        return deny(super::super::codes::denial::transition::DENIAL_UNDECLARED_EFFECT);
    }

    let mut next_states = super::super::primitive::support::copy_states(states);
    next_states.set(node_index, target);
    if !super::super::primitive::validity::indexed_invariant(dependencies, &next_states) || !super::super::primitive::validity::terminal_states_preserved(states, &next_states)
    {
        return deny(super::super::codes::denial::transition::DENIAL_INVARIANT_WOULD_BREAK);
    }
    proof {
        super::super::kernel::snapshot::suspension_update_preserves_consistency(
            states@,
            suspensions@,
            profile,
            node_index,
            action,
            *event_fact,
            target,
        );
    }
    let intent_indices = super::super::primitive::support::copy_values(requested_intents);
    super::super::primitive::support::IndexedDecision {
        accepted: true,
        reason: super::super::codes::denial::transition::DENIAL_NONE,
        next_states,
        intent_indices,
    }
}

} // verus!
