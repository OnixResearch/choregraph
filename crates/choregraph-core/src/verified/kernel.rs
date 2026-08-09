// Verus suspension and indexed-step proofs share one bounded executable kernel.
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
    reason = "Verus suspension and indexed-step proofs share one bounded executable kernel"
)]

use super::*;

verus! {

pub open spec fn suspension_fact_matches_spec(
    stored: Option<IndexedSuspensionFact>,
    event: Option<IndexedSuspensionFact>,
) -> bool {
    stored.is_some() && event.is_some() && stored.unwrap().class@ == event.unwrap().class@
        && stored.unwrap().reference@ == event.unwrap().reference@
}

pub fn suspension_fact_matches(
    stored: &Option<IndexedSuspensionFact>,
    event: &Option<IndexedSuspensionFact>,
) -> (result: bool)
    ensures result == suspension_fact_matches_spec(*stored, *event)
{
    match (stored, event) {
        (Some(stored_fact), Some(event_fact)) => {
            let is_class_equal = bytes_equal(&stored_fact.class, &event_fact.class);
            let is_reference_equal = bytes_equal(&stored_fact.reference, &event_fact.reference);
            is_class_equal && is_reference_equal
        }
        _ => false,
    }
}

pub open spec fn step_target_spec(
    profile: u8,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
) -> Option<u8> {
    if node_index < states.len() {
        lifecycle_target_spec(profile, states[node_index as int], action)
    } else {
        None
    }
}

pub open spec fn step_dependencies_ready_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
) -> bool {
    (action != ACTION_MARK_READY && action != ACTION_START && action != ACTION_RESUME_SIGNAL)
        || dependencies_complete_spec(dependencies[node_index as int]@, states)
}

pub open spec fn step_next_states_spec(
    states: Seq<u8>,
    node_index: usize,
    target: u8,
) -> Seq<u8> {
    states.update(node_index as int, target)
}

pub open spec fn suspension_fact_shape_spec(
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
) -> bool {
    if suspension_action_spec(action) {
        event_fact.is_some()
    } else {
        event_fact.is_none()
    }
}

pub open spec fn suspension_effects_spec(action: u8, requested_intents: Seq<usize>) -> bool {
    !suspension_action_spec(action) || requested_intents.len() == 0
}

pub open spec fn suspension_binding_spec(
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
) -> bool {
    if action == ACTION_RESUME_SIGNAL {
        suspension_fact_matches_spec(suspensions[node_index as int], event_fact)
    } else {
        true
    }
}

pub open spec fn step_next_suspensions_spec(
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
) -> Seq<Option<IndexedSuspensionFact>> {
    if action == ACTION_SUSPEND {
        suspensions.update(node_index as int, event_fact)
    } else if action == ACTION_RESUME_SIGNAL {
        suspensions.update(node_index as int, None)
    } else {
        suspensions
    }
}

pub open spec fn indexed_step_accepts_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> bool {
    indexed_invariant_spec(dependencies, states)
        && suspensions_consistent_spec(states, suspensions)
        && profile_valid_spec(profile)
        && node_index < states.len()
        && step_target_spec(profile, states, node_index, action).is_some()
        && suspension_fact_shape_spec(action, event_fact)
        && suspension_effects_spec(action, requested_intents)
        && step_dependencies_ready_spec(dependencies, states, node_index, action)
        && suspension_binding_spec(suspensions, node_index, action, event_fact)
        && effects_allowed_spec(allowed_intents, requested_intents)
        && indexed_invariant_spec(
            dependencies,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        )
        && terminal_states_preserved_spec(
            states,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        )
        && suspensions_consistent_spec(
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(profile, states, node_index, action).unwrap(),
            ),
            step_next_suspensions_spec(suspensions, node_index, action, event_fact),
        )
}

pub open spec fn indexed_step_reason_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> u8 {
    if !indexed_invariant_spec(dependencies, states)
        || !suspensions_consistent_spec(states, suspensions)
        || !profile_valid_spec(profile)
    {
        DENIAL_INVALID_STATE
    } else if node_index >= states.len() {
        DENIAL_UNKNOWN_NODE
    } else if terminal_state_spec(states[node_index as int]) {
        DENIAL_TERMINAL_STATE
    } else if suspension_action_spec(action) && profile == PROFILE_LINEAR_V1 {
        DENIAL_PROFILE_ACTION
    } else if step_target_spec(profile, states, node_index, action).is_none() {
        DENIAL_INVALID_TRANSITION
    } else if suspension_action_spec(action) && event_fact.is_none() {
        DENIAL_SUSPENSION_FACT_ABSENT
    } else if !suspension_action_spec(action) && event_fact.is_some() {
        DENIAL_SUSPENSION_FACT_PRESENT
    } else if !suspension_effects_spec(action, requested_intents) {
        DENIAL_SUSPENSION_EFFECTS
    } else if !step_dependencies_ready_spec(dependencies, states, node_index, action) {
        DENIAL_DEPENDENCIES_INCOMPLETE
    } else if !suspension_binding_spec(suspensions, node_index, action, event_fact) {
        DENIAL_SUSPENSION_MISMATCH
    } else if !all_distinct_spec(requested_intents) {
        DENIAL_DUPLICATE_EFFECT
    } else if !all_requested_allowed_spec(allowed_intents, requested_intents) {
        DENIAL_UNDECLARED_EFFECT
    } else {
        let target = step_target_spec(profile, states, node_index, action).unwrap();
        let next = step_next_states_spec(states, node_index, target);
        if !indexed_invariant_spec(dependencies, next)
            || !terminal_states_preserved_spec(states, next)
            || !suspensions_consistent_spec(
                next,
                step_next_suspensions_spec(suspensions, node_index, action, event_fact),
            )
        {
            DENIAL_INVARIANT_WOULD_BREAK
        } else {
            DENIAL_NONE
        }
    }
}

pub open spec fn indexed_step_next_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
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
        step_next_states_spec(
            states,
            node_index,
            step_target_spec(profile, states, node_index, action).unwrap(),
        )
    } else {
        Seq::empty()
    }
}

pub open spec fn indexed_step_next_suspensions_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> Seq<Option<IndexedSuspensionFact>> {
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
        step_next_suspensions_spec(suspensions, node_index, action, event_fact)
    } else {
        Seq::empty()
    }
}

pub open spec fn indexed_step_intents_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
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

pub fn deny(reason: u8) -> (result: IndexedDecision)
    ensures
        !result.accepted,
        result.reason == reason,
        result.next_states@.len() == 0,
        result.intent_indices@.len() == 0,
{
    IndexedDecision {
        accepted: false,
        reason,
        next_states: Vec::new(),
        intent_indices: Vec::new(),
    }
}

// r[verify choregraph.machine.suspension_proof]
pub proof fn suspension_update_preserves_consistency(
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    target: u8,
)
    requires
        profile_valid_spec(profile),
        suspensions_consistent_spec(states, suspensions),
        node_index < states.len(),
        lifecycle_target_spec(profile, states[node_index as int], action) == Some(target),
        suspension_action_spec(action) ==> event_fact.is_some(),
    ensures suspensions_consistent_spec(
        step_next_states_spec(states, node_index, target),
        step_next_suspensions_spec(suspensions, node_index, action, event_fact),
    )
{
    let next_states = step_next_states_spec(states, node_index, target);
    let next_suspensions = step_next_suspensions_spec(suspensions, node_index, action, event_fact);
    assert forall|index: int| 0 <= index < next_states.len() implies (next_suspensions[index].is_some()
        == (next_states[index] == STATE_SUSPENDED)) by {
        if index == node_index as int {
            if action == ACTION_SUSPEND {
                assert(target == STATE_SUSPENDED);
            } else if action == ACTION_RESUME_SIGNAL {
                assert(target == STATE_READY);
            } else {
                assert(target != STATE_SUSPENDED);
            }
        }
    }
}

// Verus ties this body to one complete postcondition set, so keep the proof boundary intact.
#[allow(
    clippy::too_many_arguments,
    function_length,
    reason = "the kernel takes only explicit indexed inputs"
)]
pub fn indexed_step(
    dependencies: &[Vec<usize>],
    states: &[u8],
    suspensions: &[Option<IndexedSuspensionFact>],
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: &Option<IndexedSuspensionFact>,
    allowed_intents: &[usize],
    requested_intents: &[usize],
) -> (result: IndexedDecision)
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
        result.accepted ==> indexed_invariant_spec(dependencies@, result.next_states@),
        result.accepted ==> terminal_states_preserved_spec(states@, result.next_states@),
        result.accepted ==> suspensions_consistent_spec(
            result.next_states@,
            step_next_suspensions_spec(suspensions@, node_index, action, *event_fact),
        ),
        result.accepted ==> effects_allowed_spec(allowed_intents@, result.intent_indices@),
        result.accepted ==> result.intent_indices@ == requested_intents@,
        !result.accepted ==> result.next_states@.len() == 0,
        !result.accepted ==> result.intent_indices@.len() == 0,
{
    if !indexed_invariant(dependencies, states)
        || !suspensions_consistent(states, suspensions)
        || !(profile == PROFILE_LINEAR_V1 || profile == PROFILE_DURABLE_SUSPENSION_V1)
    {
        return deny(DENIAL_INVALID_STATE);
    }
    if node_index >= states.len() {
        return deny(DENIAL_UNKNOWN_NODE);
    }
    let from = states[node_index];
    if terminal_state(from) {
        return deny(DENIAL_TERMINAL_STATE);
    }
    let is_suspension_action = action == ACTION_SUSPEND || action == ACTION_RESUME_SIGNAL;
    if is_suspension_action && profile == PROFILE_LINEAR_V1 {
        return deny(DENIAL_PROFILE_ACTION);
    }
    let target = match lifecycle_target(profile, from, action) {
        Some(target) => target,
        None => return deny(DENIAL_INVALID_TRANSITION),
    };
    if is_suspension_action && event_fact.is_none() {
        return deny(DENIAL_SUSPENSION_FACT_ABSENT);
    }
    if !is_suspension_action && event_fact.is_some() {
        return deny(DENIAL_SUSPENSION_FACT_PRESENT);
    }
    if is_suspension_action && !requested_intents.is_empty() {
        return deny(DENIAL_SUSPENSION_EFFECTS);
    }
    if (action == ACTION_MARK_READY || action == ACTION_START || action == ACTION_RESUME_SIGNAL)
        && !dependencies_complete(&dependencies[node_index], states)
    {
        return deny(DENIAL_DEPENDENCIES_INCOMPLETE);
    }
    if action == ACTION_RESUME_SIGNAL
        && !suspension_fact_matches(&suspensions[node_index], event_fact)
    {
        return deny(DENIAL_SUSPENSION_MISMATCH);
    }
    if !all_distinct(requested_intents) {
        return deny(DENIAL_DUPLICATE_EFFECT);
    }
    if !all_requested_allowed(allowed_intents, requested_intents) {
        return deny(DENIAL_UNDECLARED_EFFECT);
    }

    let mut next_states = copy_states(states);
    next_states.set(node_index, target);
    if !indexed_invariant(dependencies, &next_states) || !terminal_states_preserved(states, &next_states)
    {
        return deny(DENIAL_INVARIANT_WOULD_BREAK);
    }
    proof {
        suspension_update_preserves_consistency(
            states@,
            suspensions@,
            profile,
            node_index,
            action,
            *event_fact,
            target,
        );
    }
    let intent_indices = copy_values(requested_intents);
    IndexedDecision {
        accepted: true,
        reason: DENIAL_NONE,
        next_states,
        intent_indices,
    }
}

// r[verify choregraph.proof.single_step.readiness]
pub proof fn readiness_is_sound(dependencies: Seq<usize>, states: Seq<u8>)
    requires dependencies_complete_spec(dependencies, states)
    ensures forall|index: int| #![trigger dependencies[index]]
        0 <= index < dependencies.len() ==> {
            let dependency = dependencies[index];
            dependency < states.len() && states[dependency as int] == STATE_COMPLETED
        }
{
    readiness_is_sound_from(dependencies, states, 0);
}

proof fn readiness_is_sound_from(dependencies: Seq<usize>, states: Seq<u8>, from: int)
    requires
        0 <= from <= dependencies.len(),
        dependencies_complete_from_spec(dependencies, states, from),
    ensures forall|index: int| #![trigger dependencies[index]]
        from <= index < dependencies.len() ==> {
            let dependency = dependencies[index];
            dependency < states.len() && states[dependency as int] == STATE_COMPLETED
        }
    decreases dependencies.len() - from
{
    if from < dependencies.len() {
        readiness_is_sound_from(dependencies, states, from + 1);
    }
}

// r[verify choregraph.proof.single_step.preservation]
pub proof fn accepted_step_preserves_invariant_and_effects(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        action,
        event_fact,
        allowed_intents,
        requested_intents,
    )
    ensures
        indexed_invariant_spec(
            dependencies,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        ),
        effects_allowed_spec(allowed_intents, requested_intents),
        terminal_states_preserved_spec(
            states,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        ),
        suspensions_consistent_spec(
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(profile, states, node_index, action).unwrap(),
            ),
            step_next_suspensions_spec(suspensions, node_index, action, event_fact),
        ),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn suspension_transitions_emit_no_effects(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires
        indexed_step_accepts_spec(
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
        suspension_action_spec(action),
    ensures indexed_step_intents_spec(
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

// r[verify choregraph.machine.suspension_proof]
pub proof fn suspended_state_blocks_execution_actions(profile: u8)
    requires profile_valid_spec(profile)
    ensures
        lifecycle_target_spec(profile, STATE_SUSPENDED, ACTION_MARK_READY).is_none(),
        lifecycle_target_spec(profile, STATE_SUSPENDED, ACTION_START).is_none(),
        lifecycle_target_spec(profile, STATE_SUSPENDED, ACTION_SUSPEND).is_none(),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn accepted_suspend_records_exact_fact(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    event_fact: Option<IndexedSuspensionFact>,
)
    requires
        indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            ACTION_SUSPEND,
            event_fact,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
    ensures
        step_next_states_spec(states, node_index, STATE_SUSPENDED)[node_index as int]
            == STATE_SUSPENDED,
        step_next_suspensions_spec(suspensions, node_index, ACTION_SUSPEND, event_fact)[
            node_index as int
        ] == event_fact,
        event_fact.is_some(),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn accepted_resume_requires_exact_fact_and_clears(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    event_fact: Option<IndexedSuspensionFact>,
)
    requires
        indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            ACTION_RESUME_SIGNAL,
            event_fact,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
    ensures
        states[node_index as int] == STATE_SUSPENDED,
        suspension_fact_matches_spec(suspensions[node_index as int], event_fact),
        step_next_states_spec(states, node_index, STATE_READY)[node_index as int] == STATE_READY,
        step_next_suspensions_spec(suspensions, node_index, ACTION_RESUME_SIGNAL, event_fact)[
            node_index as int
        ].is_none(),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn mismatched_resume_is_denied(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    event_fact: Option<IndexedSuspensionFact>,
)
    requires
        indexed_invariant_spec(dependencies, states),
        suspensions_consistent_spec(states, suspensions),
        profile == PROFILE_DURABLE_SUSPENSION_V1,
        node_index < states.len(),
        states[node_index as int] == STATE_SUSPENDED,
        event_fact.is_some(),
        !suspension_fact_matches_spec(suspensions[node_index as int], event_fact),
    ensures
        !indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            ACTION_RESUME_SIGNAL,
            event_fact,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
{}

// r[verify choregraph.proof.single_step.denial]
pub proof fn denial_has_no_replacement_or_intents(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    action: u8,
    event_fact: Option<IndexedSuspensionFact>,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires !indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        action,
        event_fact,
        allowed_intents,
        requested_intents,
    )
    ensures
        indexed_step_next_spec(
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
        indexed_step_next_suspensions_spec(
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
        indexed_step_intents_spec(
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

// r[verify choregraph.proof.single_step.determinism]
pub proof fn equal_explicit_inputs_have_equal_decisions(
    dependencies_a: Seq<Vec<usize>>,
    dependencies_b: Seq<Vec<usize>>,
    states_a: Seq<u8>,
    states_b: Seq<u8>,
    suspensions_a: Seq<Option<IndexedSuspensionFact>>,
    suspensions_b: Seq<Option<IndexedSuspensionFact>>,
    profile_a: u8,
    profile_b: u8,
    node_index_a: usize,
    node_index_b: usize,
    action_a: u8,
    action_b: u8,
    event_fact_a: Option<IndexedSuspensionFact>,
    event_fact_b: Option<IndexedSuspensionFact>,
    allowed_intents_a: Seq<usize>,
    allowed_intents_b: Seq<usize>,
    requested_intents_a: Seq<usize>,
    requested_intents_b: Seq<usize>,
)
    requires
        dependencies_a == dependencies_b,
        states_a == states_b,
        suspensions_a == suspensions_b,
        profile_a == profile_b,
        node_index_a == node_index_b,
        action_a == action_b,
        event_fact_a == event_fact_b,
        allowed_intents_a == allowed_intents_b,
        requested_intents_a == requested_intents_b,
    ensures
        indexed_step_accepts_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_accepts_spec(
            dependencies_b,
            states_b,
            suspensions_b,
            profile_b,
            node_index_b,
            action_b,
            event_fact_b,
            allowed_intents_b,
            requested_intents_b,
        ),
        indexed_step_reason_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_reason_spec(
            dependencies_b,
            states_b,
            suspensions_b,
            profile_b,
            node_index_b,
            action_b,
            event_fact_b,
            allowed_intents_b,
            requested_intents_b,
        ),
        indexed_step_next_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_next_spec(
            dependencies_b,
            states_b,
            suspensions_b,
            profile_b,
            node_index_b,
            action_b,
            event_fact_b,
            allowed_intents_b,
            requested_intents_b,
        ),
        indexed_step_intents_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_intents_spec(
            dependencies_b,
            states_b,
            suspensions_b,
            profile_b,
            node_index_b,
            action_b,
            event_fact_b,
            allowed_intents_b,
            requested_intents_b,
        ),
{}

} // verus!
