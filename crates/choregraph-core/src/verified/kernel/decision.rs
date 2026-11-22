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

use super::super::codes::denial::transition::*;
use super::super::primitive::support::IndexedSuspensionFact;
use vstd::prelude::*;

verus! {

// Only this verified module constructs indexed decisions. Public adapters consume
// one of its results rather than constructing a proof-bound result themselves.
#[derive(Debug, Eq, PartialEq)]
pub struct StepOutcome {
    pub accepted: bool,
    pub reason: u8,
    pub next_states: alloc::vec::Vec<u8>,
    pub intent_indices: alloc::vec::Vec<usize>,
    seal: (),
}

pub closed spec fn indexed_decision_view(result: StepOutcome) -> (bool, u8, Seq<u8>, Seq<usize>) {
    (result.accepted, result.reason, result.next_states@, result.intent_indices@)
}

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
        && (!super::snapshot::suspension_effects_forbidden_spec(profile, states[node_index as int], action)
            || requested_intents.len() == 0)
        && super::super::kernel::target::step_dependencies_ready_spec(dependencies, states, node_index, action)
        && super::super::kernel::snapshot::suspension_binding_spec(suspensions, node_index, action, event_fact)
        && super::super::primitive::membership::effects_allowed_spec(allowed_intents, requested_intents)
}

pub open spec fn indexed_precheck_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    effects_requested: bool,
) -> Result<u8, u8> {
    if !super::super::primitive::validity::indexed_invariant_spec(dependencies, states)
        || !super::super::primitive::support::suspensions_consistent_spec(states, suspensions)
        || !super::super::codes::profile::profile_valid_spec(profile)
    {
        Err(DENIAL_INVALID_STATE)
    } else if node_index >= states.len() {
        Err(DENIAL_UNKNOWN_NODE)
    } else if super::super::codes::status::terminal_state_spec(states[node_index as int]) {
        Err(DENIAL_TERMINAL_STATE)
    } else if super::super::codes::action::suspension_action_spec(action) && profile == super::super::codes::profile::PROFILE_LINEAR_V1 {
        Err(super::super::codes::denial::suspension::DENIAL_PROFILE_ACTION)
    } else if super::super::kernel::target::step_target_spec(profile, states, node_index, action).is_none() {
        Err(DENIAL_INVALID_TRANSITION)
    } else if super::super::codes::action::suspension_action_spec(action) && event_fact.is_none() {
        Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_ABSENT)
    } else if !super::super::codes::action::suspension_action_spec(action) && event_fact.is_some() {
        Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_PRESENT)
    } else if effects_requested && super::snapshot::suspension_effects_forbidden_spec(profile, states[node_index as int], action) {
        Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_EFFECTS)
    } else if !super::super::kernel::target::step_dependencies_ready_spec(dependencies, states, node_index, action) {
        Err(DENIAL_DEPENDENCIES_INCOMPLETE)
    } else if !super::super::kernel::snapshot::suspension_binding_spec(suspensions, node_index, action, event_fact) {
        Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_MISMATCH)
    } else {
        Ok(super::target::step_target_spec(profile, states, node_index, action).unwrap())
    }
}

pub open spec fn indexed_step_reason_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> u8 {
    match indexed_precheck_spec(dependencies, states, suspensions, profile, node_index, action, event_fact, requested_intents.len() != 0) {
        Err(reason) => reason,
        Ok(_) => if !super::super::primitive::membership::all_distinct_spec(requested_intents) {
            DENIAL_DUPLICATE_EFFECT
        } else if !super::super::primitive::membership::all_requested_allowed_spec(allowed_intents, requested_intents) {
            DENIAL_UNDECLARED_EFFECT
        } else {
            DENIAL_NONE
        },
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

pub open spec fn indexed_decision_matches_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node: usize,
    action: u8,
    fact: Option<IndexedSuspensionFact>,
    allowed: Seq<usize>,
    requested: Seq<usize>,
    result: StepOutcome,
) -> bool {
    let view = indexed_decision_view(result);
    view.0 == indexed_step_accepts_spec(dependencies, states, suspensions, profile, node, action, fact, allowed, requested)
        && view.1 == indexed_step_reason_spec(dependencies, states, suspensions, profile, node, action, fact, allowed, requested)
        && view.2 == indexed_step_next_spec(dependencies, states, suspensions, profile, node, action, fact, allowed, requested)
        && view.3 == indexed_step_intents_spec(dependencies, states, suspensions, profile, node, action, fact, allowed, requested)
}

fn deny(reason: u8) -> (result: StepOutcome)
    ensures
        !result.accepted,
        result.reason == reason,
        result.next_states@ == Seq::<u8>::empty(),
        result.intent_indices@ == Seq::<usize>::empty(),
{
    StepOutcome {
        accepted: false,
        reason,
        next_states: alloc::vec::Vec::new(),
        intent_indices: alloc::vec::Vec::new(),
        seal: (),
    }
}

/// Runs the same guard order without constructing a next-state or an intent vector.
#[allow(clippy::too_many_arguments, function_length, reason = "explicit indexed guard inputs")]
pub fn indexed_step_precheck(
    dependencies: &[alloc::vec::Vec<usize>],
    states: &[u8],
    suspensions: &[Option<IndexedSuspensionFact>],
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: &Option<IndexedSuspensionFact>,
    effects_requested: bool,
) -> (result: Result<u8, u8>)
    ensures result == indexed_precheck_spec(dependencies@, states@, suspensions@, profile, node_index, action, *event_fact, effects_requested)
{
    if !super::super::primitive::validity::indexed_invariant(dependencies, states)
        || !super::super::primitive::support::suspensions_consistent(states, suspensions)
        || !super::super::codes::profile::is_supported(profile)
    {
        return Err(DENIAL_INVALID_STATE);
    }
    if node_index >= states.len() {
        return Err(DENIAL_UNKNOWN_NODE);
    }
    let from = states[node_index];
    if super::super::codes::status::terminal_state(from) {
        return Err(DENIAL_TERMINAL_STATE);
    }
    let is_suspension_action = action == super::super::codes::action::ACTION_SUSPEND || action == super::super::codes::action::ACTION_RESUME_SIGNAL;
    if is_suspension_action && profile == super::super::codes::profile::PROFILE_LINEAR_V1 {
        return Err(super::super::codes::denial::suspension::DENIAL_PROFILE_ACTION);
    }
    let target = match super::super::primitive::transition::lifecycle_target(profile, from, action) {
        Some(target) => target,
        None => return Err(DENIAL_INVALID_TRANSITION),
    };
    if is_suspension_action && event_fact.is_none() {
        return Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_ABSENT);
    }
    if !is_suspension_action && event_fact.is_some() {
        return Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_FACT_PRESENT);
    }
    if effects_requested && super::snapshot::suspension_effects_forbidden(profile, from, action) {
        return Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_EFFECTS);
    }
    if super::target::action_needs_readiness(action)
        && !super::super::primitive::validity::dependencies_complete(&dependencies[node_index], states)
    {
        return Err(DENIAL_DEPENDENCIES_INCOMPLETE);
    }
    if action == super::super::codes::action::ACTION_RESUME_SIGNAL
        && !super::snapshot::suspension_fact_matches(&suspensions[node_index], event_fact)
    {
        return Err(super::super::codes::denial::suspension::DENIAL_SUSPENSION_MISMATCH);
    }
    Ok(target)
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
) -> (result: StepOutcome)
    ensures
        indexed_decision_view(result).0 ==> super::super::primitive::validity::indexed_invariant_spec(dependencies@, indexed_decision_view(result).2),
        indexed_decision_view(result).0 ==> super::super::primitive::validity::terminal_states_preserved_spec(states@, indexed_decision_view(result).2),
        indexed_decision_view(result).0 ==> super::super::primitive::support::suspensions_consistent_spec(
            indexed_decision_view(result).2,
            super::super::kernel::snapshot::step_next_suspensions_spec(suspensions@, node_index, action, *event_fact),
        ),
        indexed_decision_view(result).0 ==> super::super::primitive::membership::effects_allowed_spec(allowed_intents@, indexed_decision_view(result).3),
        indexed_decision_view(result).0 ==> indexed_decision_view(result).3 == requested_intents@,
        !indexed_decision_view(result).0 ==> indexed_decision_view(result).2.len() == 0,
        !indexed_decision_view(result).0 ==> indexed_decision_view(result).3.len() == 0,
        indexed_decision_view(result).1 != DENIAL_INVARIANT_WOULD_BREAK,
        indexed_decision_matches_spec(dependencies@, states@, suspensions@, profile, node_index, action, *event_fact, allowed_intents@, requested_intents@, result),
{
    let target = match indexed_step_precheck(dependencies, states, suspensions, profile, node_index, action, event_fact, !requested_intents.is_empty()) {
        Ok(target) => target,
        Err(reason) => return deny(reason),
    };
    if !super::super::primitive::membership::all_distinct(requested_intents) {
        return deny(super::super::codes::denial::transition::DENIAL_DUPLICATE_EFFECT);
    }
    if !super::super::primitive::membership::all_requested_allowed(allowed_intents, requested_intents) {
        return deny(super::super::codes::denial::transition::DENIAL_UNDECLARED_EFFECT);
    }

    proof {
        super::target::local_update_preserves_invariant(dependencies@, states@, profile, node_index, action, target);
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
    let mut next_states = super::super::primitive::support::copy_states(states);
    next_states.set(node_index, target);
    let intent_indices = super::super::primitive::support::copy_values(requested_intents);
    StepOutcome {
        accepted: true,
        reason: super::super::codes::denial::transition::DENIAL_NONE,
        next_states,
        intent_indices,
        seal: (),
    }
}

} // verus!
