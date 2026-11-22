// Verus trace specifications fold the production indexed step relation without dispatch output.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    non_trait_imports,
    raw_arithmetic_overflow,
    too_many_parameters,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus trace specifications fold the production indexed step relation without dispatch output"
)]

// r[impl choregraph.proof.trace.model]
// r[impl choregraph.proof.trace.replay]
// r[verify choregraph.proof.trace.lemmas]
// r[verify choregraph.proof.trace.preservation]
// r[verify choregraph.proof.trace.determinism]
// r[verify choregraph.proof.trace.failure]
// r[verify choregraph.machine.suspension_replay]
use vstd::prelude::*;

verus! {

// r[verify choregraph.proof.trace.lemmas]
pub proof fn empty_trace_base(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
)
    requires
        crate::verified::primitive::validity::indexed_invariant_spec(dependencies, states),
        crate::verified::primitive::support::suspensions_consistent_spec(states, suspensions),
    ensures
        super::specification::replay_prefix_len_from_spec(
            dependencies,
            states,
            suspensions,
            profile,
            Seq::<super::specification::IndexedReplayEvent>::empty(),
            0,
        ) == 0,
        super::specification::replay_final_states_from_spec(
            dependencies,
            states,
            suspensions,
            profile,
            Seq::<super::specification::IndexedReplayEvent>::empty(),
            0,
        ) == states,
        super::specification::replay_final_suspensions_from_spec(
            dependencies,
            states,
            suspensions,
            profile,
            Seq::<super::specification::IndexedReplayEvent>::empty(),
            0,
        ) == suspensions,
        super::specification::replay_failure_index_from_spec(
            dependencies,
            states,
            suspensions,
            profile,
            Seq::<super::specification::IndexedReplayEvent>::empty(),
            0,
        ).is_none(),
        crate::verified::primitive::validity::indexed_invariant_spec(
            dependencies,
            super::specification::replay_final_states_from_spec(
                dependencies,
                states,
                suspensions,
                profile,
                Seq::<super::specification::IndexedReplayEvent>::empty(),
                0,
            ),
        ),
{}

// r[verify choregraph.proof.trace.lemmas]
pub proof fn admitted_trace_respects_limit(
    events: Seq<super::specification::IndexedReplayEvent>,
    max_events: int,
    machine_binding_matches: bool,
    initial_state_binding_matches: bool,
)
    requires super::specification::trace_admitted_spec(
        events,
        max_events,
        machine_binding_matches,
        initial_state_binding_matches,
    )
    ensures
        events.len() <= max_events,
        machine_binding_matches,
        initial_state_binding_matches,
{}

// r[verify choregraph.proof.trace.lemmas]
pub proof fn replay_prefix_length_bounded(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<super::specification::IndexedReplayEvent>,
    from: int,
)
    requires 0 <= from <= events.len()
    ensures
        0 <= super::specification::replay_prefix_len_from_spec(dependencies, states, suspensions, profile, events, from),
        super::specification::replay_prefix_len_from_spec(dependencies, states, suspensions, profile, events, from)
            <= events.len() - from,
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if super::specification::replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            replay_prefix_length_bounded(
                dependencies,
                super::specification::replay_event_next_spec(dependencies, states, suspensions, profile, event),
                super::specification::replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            );
        }
    }
}

// r[verify choregraph.proof.trace.preservation]
pub proof fn finite_replay_preserves_invariant(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<super::specification::IndexedReplayEvent>,
    from: int,
)
    requires
        0 <= from <= events.len(),
        crate::verified::primitive::validity::indexed_invariant_spec(dependencies, states),
        crate::verified::primitive::support::suspensions_consistent_spec(states, suspensions),
    ensures
        crate::verified::primitive::validity::indexed_invariant_spec(
            dependencies,
            super::specification::replay_final_states_from_spec(dependencies, states, suspensions, profile, events, from),
        ),
        crate::verified::primitive::support::suspensions_consistent_spec(
            super::specification::replay_final_states_from_spec(dependencies, states, suspensions, profile, events, from),
            super::specification::replay_final_suspensions_from_spec(
                dependencies,
                states,
                suspensions,
                profile,
                events,
                from,
            ),
        ),
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if super::specification::replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            crate::verified::kernel::theorem::accepted_step_preserves_invariant_and_effects(
                dependencies,
                states,
                suspensions,
                profile,
                event.node_index,
                event.action,
                event.event_fact,
                event.allowed_intents@,
                event.requested_intents@,
            );
            assert(crate::verified::primitive::validity::indexed_invariant_spec(
                dependencies,
                super::specification::replay_event_next_spec(dependencies, states, suspensions, profile, event),
            ));
            assert(crate::verified::primitive::support::suspensions_consistent_spec(
                super::specification::replay_event_next_spec(dependencies, states, suspensions, profile, event),
                super::specification::replay_event_next_suspensions_spec(dependencies, states, suspensions, profile, event),
            ));
            finite_replay_preserves_invariant(
                dependencies,
                super::specification::replay_event_next_spec(dependencies, states, suspensions, profile, event),
                super::specification::replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            );
        }
    }
}

// r[verify choregraph.machine.suspension_replay]
pub proof fn accepted_suspended_prefix_remains_consistent(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<super::specification::IndexedReplayEvent>,
    from: int,
)
    requires
        0 <= from <= events.len(),
        crate::verified::primitive::validity::indexed_invariant_spec(dependencies, states),
        crate::verified::primitive::support::suspensions_consistent_spec(states, suspensions),
    ensures
        crate::verified::primitive::support::suspensions_consistent_spec(
            super::specification::replay_final_states_from_spec(dependencies, states, suspensions, profile, events, from),
            super::specification::replay_final_suspensions_from_spec(
                dependencies,
                states,
                suspensions,
                profile,
                events,
                from,
            ),
        ),
    decreases events.len() - from
{
    finite_replay_preserves_invariant(dependencies, states, suspensions, profile, events, from);
}

// r[verify choregraph.proof.trace.failure]
pub proof fn first_failure_is_exact(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<super::specification::IndexedReplayEvent>,
    from: int,
)
    requires 0 <= from <= events.len()
    ensures
        match super::specification::replay_failure_index_from_spec(dependencies, states, suspensions, profile, events, from) {
            None => super::specification::replay_prefix_len_from_spec(dependencies, states, suspensions, profile, events, from)
                == events.len() - from,
            Some(index) => {
                &&& from <= index < events.len()
                &&& index == from
                    + super::specification::replay_prefix_len_from_spec(dependencies, states, suspensions, profile, events, from)
                &&& !super::specification::replay_event_accepts_spec(
                    dependencies,
                    super::specification::replay_final_states_from_spec(dependencies, states, suspensions, profile, events, from),
                    super::specification::replay_final_suspensions_from_spec(dependencies, states, suspensions, profile, events, from),
                    profile,
                    events[index],
                )
            }
        },
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if super::specification::replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            first_failure_is_exact(
                dependencies,
                super::specification::replay_event_next_spec(dependencies, states, suspensions, profile, event),
                super::specification::replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            );
        }
    }
}

// r[verify choregraph.proof.trace.determinism]
pub proof fn equal_replay_inputs_have_equal_decisions(
    dependencies_a: Seq<alloc::vec::Vec<usize>>,
    dependencies_b: Seq<alloc::vec::Vec<usize>>,
    states_a: Seq<u8>,
    states_b: Seq<u8>,
    suspensions_a: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    suspensions_b: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile_a: u8,
    profile_b: u8,
    events_a: Seq<super::specification::IndexedReplayEvent>,
    events_b: Seq<super::specification::IndexedReplayEvent>,
    from_a: int,
    from_b: int,
    max_events_a: int,
    max_events_b: int,
    machine_binding_matches_a: bool,
    machine_binding_matches_b: bool,
    initial_state_binding_matches_a: bool,
    initial_state_binding_matches_b: bool,
)
    requires
        dependencies_a == dependencies_b,
        states_a == states_b,
        suspensions_a == suspensions_b,
        profile_a == profile_b,
        events_a == events_b,
        from_a == from_b,
        max_events_a == max_events_b,
        machine_binding_matches_a == machine_binding_matches_b,
        initial_state_binding_matches_a == initial_state_binding_matches_b,
    ensures
        super::specification::trace_admitted_spec(
            events_a,
            max_events_a,
            machine_binding_matches_a,
            initial_state_binding_matches_a,
        ) == super::specification::trace_admitted_spec(
            events_b,
            max_events_b,
            machine_binding_matches_b,
            initial_state_binding_matches_b,
        ),
        super::specification::replay_prefix_len_from_spec(dependencies_a, states_a, suspensions_a, profile_a, events_a, from_a)
            == super::specification::replay_prefix_len_from_spec(dependencies_b, states_b, suspensions_b, profile_b, events_b, from_b),
        super::specification::replay_final_states_from_spec(dependencies_a, states_a, suspensions_a, profile_a, events_a, from_a)
            == super::specification::replay_final_states_from_spec(dependencies_b, states_b, suspensions_b, profile_b, events_b, from_b),
        super::specification::replay_final_suspensions_from_spec(dependencies_a, states_a, suspensions_a, profile_a, events_a, from_a)
            == super::specification::replay_final_suspensions_from_spec(dependencies_b, states_b, suspensions_b, profile_b, events_b, from_b),
        super::specification::replay_failure_index_from_spec(dependencies_a, states_a, suspensions_a, profile_a, events_a, from_a)
            == super::specification::replay_failure_index_from_spec(dependencies_b, states_b, suspensions_b, profile_b, events_b, from_b),
{}

// r[verify choregraph.proof.trace.replay]
pub proof fn replay_never_returns_dispatch_intents(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<super::specification::IndexedReplayEvent>,
    from: int,
)
    ensures super::specification::replay_dispatch_intents_spec(dependencies, states, suspensions, profile, events, from).len() == 0
{}

/// A replay checks each Start against the snapshot *before* that event, also
/// covering dependencies already completed in a supplied restart snapshot.
pub proof fn accepted_trace_starts_after_dependencies(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<super::specification::IndexedReplayEvent>,
    from: int,
)
    requires 0 <= from <= events.len()
    ensures super::specification::replay_starts_after_dependencies_spec(dependencies, states, suspensions, profile, events, from),
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if super::specification::replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            if event.action == crate::verified::codes::action::ACTION_START {
                crate::verified::kernel::theorem::accepted_start_has_completed_dependencies(
                    dependencies, states, suspensions, profile, event.node_index, event.event_fact,
                    event.allowed_intents@, event.requested_intents@,
                );
            }
            accepted_trace_starts_after_dependencies(
                dependencies,
                super::specification::replay_event_next_spec(dependencies, states, suspensions, profile, event),
                super::specification::replay_event_next_suspensions_spec(dependencies, states, suspensions, profile, event),
                profile, events, from + 1,
            );
        }
    }
}

} // verus!
