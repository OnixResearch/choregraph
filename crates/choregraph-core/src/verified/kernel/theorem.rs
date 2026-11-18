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

// r[impl choregraph.proof.single_step.readiness_transition]
pub open spec fn mark_ready_context_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
) -> bool {
    super::super::primitive::validity::indexed_invariant_spec(dependencies, states)
        && super::super::primitive::support::suspensions_consistent_spec(states, suspensions)
        && super::super::codes::profile::profile_valid_spec(profile)
        && node_index < dependencies.len()
        && node_index < states.len()
        && states[node_index as int] == super::super::codes::status::STATE_PENDING
        && super::super::primitive::validity::indexed_invariant_spec(
            dependencies,
            super::super::kernel::target::step_next_states_spec(states, node_index, super::super::codes::status::STATE_READY),
        )
        && super::super::primitive::validity::terminal_states_preserved_spec(
            states,
            super::super::kernel::target::step_next_states_spec(states, node_index, super::super::codes::status::STATE_READY),
        )
        && super::super::primitive::support::suspensions_consistent_spec(
            super::super::kernel::target::step_next_states_spec(states, node_index, super::super::codes::status::STATE_READY),
            suspensions,
        )
}

// r[verify choregraph.proof.single_step.readiness_transition]
pub proof fn mark_ready_acceptance_is_exact(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
)
    requires mark_ready_context_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
    )
    ensures super::super::kernel::decision::indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        node_index,
        super::super::codes::action::ACTION_MARK_READY,
        None,
        Seq::<usize>::empty(),
        Seq::<usize>::empty(),
    ) == super::super::readiness::dependency_ready_spec(dependencies, states, node_index)
{}

// r[verify choregraph.proof.single_step.readiness]
// r[verify choregraph.proof.single_step.readiness_completeness]
pub proof fn readiness_is_sound(dependencies: Seq<usize>, states: Seq<u8>)
    requires super::super::primitive::validity::dependencies_complete_spec(dependencies, states)
    ensures forall|index: int| #![trigger dependencies[index]]
        0 <= index < dependencies.len() ==> {
            let dependency = dependencies[index];
            dependency < states.len() && states[dependency as int] == super::super::codes::status::STATE_COMPLETED
        }
{
    readiness_is_sound_from(dependencies, states, 0);
}

proof fn readiness_is_sound_from(dependencies: Seq<usize>, states: Seq<u8>, from: int)
    requires
        0 <= from <= dependencies.len(),
        super::super::primitive::validity::dependencies_complete_from_spec(dependencies, states, from),
    ensures forall|index: int| #![trigger dependencies[index]]
        from <= index < dependencies.len() ==> {
            let dependency = dependencies[index];
            dependency < states.len() && states[dependency as int] == super::super::codes::status::STATE_COMPLETED
        }
    decreases dependencies.len() - from
{
    if from < dependencies.len() {
        readiness_is_sound_from(dependencies, states, from + 1);
    }
}

// r[verify choregraph.proof.single_step.preservation]
pub proof fn accepted_step_preserves_invariant_and_effects(
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
    requires super::super::kernel::decision::indexed_step_accepts_spec(
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
        super::super::primitive::validity::indexed_invariant_spec(
            dependencies,
            super::super::kernel::target::step_next_states_spec(
                states,
                node_index,
                super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        ),
        super::super::primitive::membership::effects_allowed_spec(allowed_intents, requested_intents),
        super::super::primitive::validity::terminal_states_preserved_spec(
            states,
            super::super::kernel::target::step_next_states_spec(
                states,
                node_index,
                super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
            ),
        ),
        super::super::primitive::support::suspensions_consistent_spec(
            super::super::kernel::target::step_next_states_spec(
                states,
                node_index,
                super::super::kernel::target::step_target_spec(profile, states, node_index, action).unwrap(),
            ),
            super::super::kernel::snapshot::step_next_suspensions_spec(suspensions, node_index, action, event_fact),
        ),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn suspended_state_blocks_execution_actions(profile: u8)
    requires super::super::codes::profile::profile_valid_spec(profile)
    ensures
        super::super::primitive::transition::lifecycle_target_spec(profile, super::super::codes::status::STATE_SUSPENDED, super::super::codes::action::ACTION_MARK_READY).is_none(),
        super::super::primitive::transition::lifecycle_target_spec(profile, super::super::codes::status::STATE_SUSPENDED, super::super::codes::action::ACTION_START).is_none(),
        super::super::primitive::transition::lifecycle_target_spec(profile, super::super::codes::status::STATE_SUSPENDED, super::super::codes::action::ACTION_SUSPEND).is_none(),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn accepted_suspend_records_exact_fact(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
)
    requires
        super::super::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            super::super::codes::action::ACTION_SUSPEND,
            event_fact,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
    ensures
        super::super::kernel::target::step_next_states_spec(states, node_index, super::super::codes::status::STATE_SUSPENDED)[node_index as int]
            == super::super::codes::status::STATE_SUSPENDED,
        super::super::kernel::snapshot::step_next_suspensions_spec(suspensions, node_index, super::super::codes::action::ACTION_SUSPEND, event_fact)[
            node_index as int
        ] == event_fact,
        event_fact.is_some(),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn accepted_resume_requires_exact_fact_and_clears(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
)
    requires
        super::super::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            super::super::codes::action::ACTION_RESUME_SIGNAL,
            event_fact,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
    ensures
        states[node_index as int] == super::super::codes::status::STATE_SUSPENDED,
        super::super::kernel::snapshot::suspension_fact_matches_spec(suspensions[node_index as int], event_fact),
        super::super::kernel::target::step_next_states_spec(states, node_index, super::super::codes::status::STATE_READY)[node_index as int] == super::super::codes::status::STATE_READY,
        super::super::kernel::snapshot::step_next_suspensions_spec(suspensions, node_index, super::super::codes::action::ACTION_RESUME_SIGNAL, event_fact)[
            node_index as int
        ].is_none(),
{}

// r[verify choregraph.machine.suspension_proof]
pub proof fn mismatched_resume_is_denied(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    node_index: usize,
    event_fact: Option<super::super::primitive::support::IndexedSuspensionFact>,
)
    requires
        super::super::primitive::validity::indexed_invariant_spec(dependencies, states),
        super::super::primitive::support::suspensions_consistent_spec(states, suspensions),
        profile == super::super::codes::profile::PROFILE_DURABLE_SUSPENSION_V1,
        node_index < states.len(),
        states[node_index as int] == super::super::codes::status::STATE_SUSPENDED,
        event_fact.is_some(),
        !super::super::kernel::snapshot::suspension_fact_matches_spec(suspensions[node_index as int], event_fact),
    ensures
        !super::super::kernel::decision::indexed_step_accepts_spec(
            dependencies,
            states,
            suspensions,
            profile,
            node_index,
            super::super::codes::action::ACTION_RESUME_SIGNAL,
            event_fact,
            Seq::<usize>::empty(),
            Seq::<usize>::empty(),
        ),
{}

// r[verify choregraph.proof.single_step.denial]
pub proof fn denial_has_no_replacement_or_intents(
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
    requires !super::super::kernel::decision::indexed_step_accepts_spec(
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
        super::super::kernel::decision::indexed_step_next_spec(
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
        super::super::kernel::decision::indexed_step_next_suspensions_spec(
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
        super::super::kernel::decision::indexed_step_intents_spec(
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
    dependencies_a: Seq<alloc::vec::Vec<usize>>,
    dependencies_b: Seq<alloc::vec::Vec<usize>>,
    states_a: Seq<u8>,
    states_b: Seq<u8>,
    suspensions_a: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    suspensions_b: Seq<Option<super::super::primitive::support::IndexedSuspensionFact>>,
    profile_a: u8,
    profile_b: u8,
    node_index_a: usize,
    node_index_b: usize,
    action_a: u8,
    action_b: u8,
    event_fact_a: Option<super::super::primitive::support::IndexedSuspensionFact>,
    event_fact_b: Option<super::super::primitive::support::IndexedSuspensionFact>,
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
        super::super::kernel::decision::indexed_step_accepts_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == super::super::kernel::decision::indexed_step_accepts_spec(
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
        super::super::kernel::decision::indexed_step_reason_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == super::super::kernel::decision::indexed_step_reason_spec(
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
        super::super::kernel::decision::indexed_step_next_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == super::super::kernel::decision::indexed_step_next_spec(
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
        super::super::kernel::decision::indexed_step_intents_spec(
            dependencies_a,
            states_a,
            suspensions_a,
            profile_a,
            node_index_a,
            action_a,
            event_fact_a,
            allowed_intents_a,
            requested_intents_a,
        ) == super::super::kernel::decision::indexed_step_intents_spec(
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
