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

#[allow(
    unused_imports,
    reason = "Verus erases the consumers of these imports in Rust builds"
)]
use super::super::{
    codes::{action::*, profile::*, status::*},
    primitive::{support::*, transition::*, validity::*},
};
#[allow(
    unused_imports,
    reason = "Verus erases the consumers of these imports in Rust builds"
)]
use super::{decision::*, snapshot::*, target::*};
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
{}

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
{
    let target = step_target_spec(profile, states, node_index, action).unwrap();
    local_update_preserves_invariant(dependencies, states, profile, node_index, action, target);
    suspension_update_preserves_consistency(states, suspensions, profile, node_index, action, event_fact, target);
}

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
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node: usize,
    action: u8,
    fact: Option<IndexedSuspensionFact>,
    allowed: Seq<usize>,
    requested: Seq<usize>,
    first: StepOutcome,
    second: StepOutcome,
)
    requires
        indexed_decision_matches_spec(dependencies, states, suspensions, profile, node, action, fact, allowed, requested, first),
        indexed_decision_matches_spec(dependencies, states, suspensions, profile, node, action, fact, allowed, requested, second),
    ensures indexed_decision_view(first) == indexed_decision_view(second)
{}

pub proof fn cancel_never_completes(profile: u8, from: u8)
    ensures lifecycle_target_spec(profile, from, ACTION_CANCEL) != Some(STATE_COMPLETED)
{}

pub proof fn completed_requires_running_complete(profile: u8, from: u8, action: u8)
    requires lifecycle_target_spec(profile, from, action) == Some(STATE_COMPLETED)
    ensures from == STATE_RUNNING, action == ACTION_COMPLETE,
{}

// Stronger than the existence of a progress event: this works for each
// non-terminal node, including a restored wait with incomplete dependencies.
pub proof fn nonterminal_can_cancel(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node: usize,
)
    requires
        indexed_invariant_spec(dependencies, states),
        suspensions_consistent_spec(states, suspensions),
        profile == PROFILE_LINEAR_V1 || profile == PROFILE_DURABLE_SUSPENSION_V2,
        node < states.len(),
        !terminal_state_spec(states[node as int]),
        profile == PROFILE_LINEAR_V1 ==> states[node as int] != STATE_SUSPENDED,
    ensures
        indexed_step_accepts_spec(dependencies, states, suspensions, profile, node, ACTION_CANCEL, None, Seq::empty(), Seq::empty()),
        indexed_step_next_spec(dependencies, states, suspensions, profile, node, ACTION_CANCEL, None, Seq::empty(), Seq::empty())[node as int] == STATE_CANCELLED,
        indexed_step_next_suspensions_spec(dependencies, states, suspensions, profile, node, ACTION_CANCEL, None, Seq::empty(), Seq::empty())[node as int].is_none(),
{
    assert(node_invariant_spec(dependencies, states, node as int));
}

pub proof fn accepted_start_has_completed_dependencies(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
    profile: u8,
    node: usize,
    fact: Option<IndexedSuspensionFact>,
    allowed: Seq<usize>,
    requested: Seq<usize>,
)
    requires indexed_step_accepts_spec(dependencies, states, suspensions, profile, node, ACTION_START, fact, allowed, requested),
    ensures dependencies_complete_spec(dependencies[node as int]@, states),
{
    assert(lifecycle_target_spec(profile, states[node as int], ACTION_START) == Some(STATE_RUNNING));
    assert(node_invariant_spec(dependencies, states, node as int));
}

} // verus!
