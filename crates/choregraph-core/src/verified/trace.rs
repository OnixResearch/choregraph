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
use alloc::vec::Vec;
use vstd::prelude::*;

verus! {

// The proof event type remains executable syntax so Verus can project vector fields.
#[allow(
    dead_code,
    reason = "the production-linked proof type is constructed only by Verus proof roots"
)]
#[derive(Debug, Eq, PartialEq)]
pub struct IndexedReplayEvent {
    pub node_index: usize,
    pub action: u8,
    pub allowed_intents: Vec<usize>,
    pub requested_intents: Vec<usize>,
}

pub open spec fn trace_admitted_spec(
    events: Seq<IndexedReplayEvent>,
    max_events: int,
    machine_binding_matches: bool,
    initial_state_binding_matches: bool,
) -> bool {
    0 <= max_events
        && events.len() <= max_events
        && machine_binding_matches
        && initial_state_binding_matches
}

pub open spec fn replay_event_accepts_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    event: IndexedReplayEvent,
) -> bool {
    crate::verified::indexed_step_accepts_spec(
        dependencies,
        states,
        event.node_index,
        event.action,
        event.allowed_intents@,
        event.requested_intents@,
    )
}

pub open spec fn replay_event_next_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    event: IndexedReplayEvent,
) -> Seq<u8> {
    crate::verified::indexed_step_next_spec(
        dependencies,
        states,
        event.node_index,
        event.action,
        event.allowed_intents@,
        event.requested_intents@,
    )
}

pub open spec fn replay_prefix_len_from_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> int
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        0
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, event) {
            1 + replay_prefix_len_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
                events,
                from + 1,
            )
        } else {
            0
        }
    }
}

pub open spec fn replay_final_states_from_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Seq<u8>
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        states
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, event) {
            replay_final_states_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
                events,
                from + 1,
            )
        } else {
            states
        }
    }
}

pub open spec fn replay_failure_index_from_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Option<int>
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        None
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, event) {
            replay_failure_index_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
                events,
                from + 1,
            )
        } else {
            Some(from)
        }
    }
}

pub open spec fn replay_dispatch_intents_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Seq<usize> {
    Seq::empty()
}

// r[verify choregraph.proof.trace.lemmas]
pub proof fn empty_trace_base(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
)
    requires crate::verified::indexed_invariant_spec(dependencies, states)
    ensures
        replay_prefix_len_from_spec(
            dependencies,
            states,
            Seq::<IndexedReplayEvent>::empty(),
            0,
        ) == 0,
        replay_final_states_from_spec(
            dependencies,
            states,
            Seq::<IndexedReplayEvent>::empty(),
            0,
        ) == states,
        replay_failure_index_from_spec(
            dependencies,
            states,
            Seq::<IndexedReplayEvent>::empty(),
            0,
        ).is_none(),
        crate::verified::indexed_invariant_spec(
            dependencies,
            replay_final_states_from_spec(
                dependencies,
                states,
                Seq::<IndexedReplayEvent>::empty(),
                0,
            ),
        ),
{}

// r[verify choregraph.proof.trace.lemmas]
pub proof fn admitted_trace_respects_limit(
    events: Seq<IndexedReplayEvent>,
    max_events: int,
    machine_binding_matches: bool,
    initial_state_binding_matches: bool,
)
    requires trace_admitted_spec(
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
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
)
    requires 0 <= from <= events.len()
    ensures
        0 <= replay_prefix_len_from_spec(dependencies, states, events, from),
        replay_prefix_len_from_spec(dependencies, states, events, from)
            <= events.len() - from,
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, event) {
            replay_prefix_length_bounded(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
                events,
                from + 1,
            );
        }
    }
}

// r[verify choregraph.proof.trace.preservation]
pub proof fn finite_replay_preserves_invariant(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
)
    requires
        0 <= from <= events.len(),
        crate::verified::indexed_invariant_spec(dependencies, states),
    ensures crate::verified::indexed_invariant_spec(
        dependencies,
        replay_final_states_from_spec(dependencies, states, events, from),
    )
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, event) {
            crate::verified::accepted_step_preserves_invariant_and_effects(
                dependencies,
                states,
                event.node_index,
                event.action,
                event.allowed_intents@,
                event.requested_intents@,
            );
            assert(crate::verified::indexed_invariant_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
            ));
            finite_replay_preserves_invariant(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
                events,
                from + 1,
            );
        }
    }
}

// r[verify choregraph.proof.trace.failure]
pub proof fn first_failure_is_exact(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
)
    requires 0 <= from <= events.len()
    ensures
        match replay_failure_index_from_spec(dependencies, states, events, from) {
            None => replay_prefix_len_from_spec(dependencies, states, events, from)
                == events.len() - from,
            Some(index) => {
                &&& from <= index < events.len()
                &&& index == from
                    + replay_prefix_len_from_spec(dependencies, states, events, from)
                &&& !replay_event_accepts_spec(
                    dependencies,
                    replay_final_states_from_spec(dependencies, states, events, from),
                    events[index],
                )
            }
        },
    decreases events.len() - from
{
    if from < events.len() {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, event) {
            first_failure_is_exact(
                dependencies,
                replay_event_next_spec(dependencies, states, event),
                events,
                from + 1,
            );
        }
    }
}

// r[verify choregraph.proof.trace.determinism]
pub proof fn equal_replay_inputs_have_equal_decisions(
    dependencies_a: Seq<Vec<usize>>,
    dependencies_b: Seq<Vec<usize>>,
    states_a: Seq<u8>,
    states_b: Seq<u8>,
    events_a: Seq<IndexedReplayEvent>,
    events_b: Seq<IndexedReplayEvent>,
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
        events_a == events_b,
        from_a == from_b,
        max_events_a == max_events_b,
        machine_binding_matches_a == machine_binding_matches_b,
        initial_state_binding_matches_a == initial_state_binding_matches_b,
    ensures
        trace_admitted_spec(
            events_a,
            max_events_a,
            machine_binding_matches_a,
            initial_state_binding_matches_a,
        ) == trace_admitted_spec(
            events_b,
            max_events_b,
            machine_binding_matches_b,
            initial_state_binding_matches_b,
        ),
        replay_prefix_len_from_spec(dependencies_a, states_a, events_a, from_a)
            == replay_prefix_len_from_spec(dependencies_b, states_b, events_b, from_b),
        replay_final_states_from_spec(dependencies_a, states_a, events_a, from_a)
            == replay_final_states_from_spec(dependencies_b, states_b, events_b, from_b),
        replay_failure_index_from_spec(dependencies_a, states_a, events_a, from_a)
            == replay_failure_index_from_spec(dependencies_b, states_b, events_b, from_b),
{}

// r[verify choregraph.proof.trace.replay]
pub proof fn replay_never_returns_dispatch_intents(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    events: Seq<IndexedReplayEvent>,
    from: int,
)
    ensures replay_dispatch_intents_spec(dependencies, states, events, from).len() == 0
{}

} // verus!
