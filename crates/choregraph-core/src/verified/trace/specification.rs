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

// The proof event type remains executable syntax so Verus can project vector fields.
#[allow(
    dead_code,
    reason = "the production-linked proof type is constructed only by Verus proof roots"
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedReplayEvent {
    pub node_index: usize,
    pub action: u8,
    pub event_fact: Option<crate::verified::primitive::support::IndexedSuspensionFact>,
    pub allowed_intents: alloc::vec::Vec<usize>,
    pub requested_intents: alloc::vec::Vec<usize>,
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
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    event: IndexedReplayEvent,
) -> bool {
    crate::verified::kernel::decision::indexed_step_accepts_spec(
        dependencies,
        states,
        suspensions,
        profile,
        event.node_index,
        event.action,
        event.event_fact,
        event.allowed_intents@,
        event.requested_intents@,
    )
}

pub open spec fn replay_event_next_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    event: IndexedReplayEvent,
) -> Seq<u8> {
    crate::verified::kernel::decision::indexed_step_next_spec(
        dependencies,
        states,
        suspensions,
        profile,
        event.node_index,
        event.action,
        event.event_fact,
        event.allowed_intents@,
        event.requested_intents@,
    )
}

pub open spec fn replay_event_next_suspensions_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    event: IndexedReplayEvent,
) -> Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>> {
    crate::verified::kernel::decision::indexed_step_next_suspensions_spec(
        dependencies,
        states,
        suspensions,
        profile,
        event.node_index,
        event.action,
        event.event_fact,
        event.allowed_intents@,
        event.requested_intents@,
    )
}

pub open spec fn replay_prefix_len_from_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> int
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        0
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            1 + replay_prefix_len_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, suspensions, profile, event),
                replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            )
        } else {
            0
        }
    }
}

pub open spec fn replay_final_states_from_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Seq<u8>
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        states
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            replay_final_states_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, suspensions, profile, event),
                replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            )
        } else {
            states
        }
    }
}

pub open spec fn replay_final_suspensions_from_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        suspensions
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            replay_final_suspensions_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, suspensions, profile, event),
                replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            )
        } else {
            suspensions
        }
    }
}

pub open spec fn replay_failure_index_from_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Option<int>
    decreases events.len() - from
{
    if from < 0 || from >= events.len() {
        None
    } else {
        let event = events[from];
        if replay_event_accepts_spec(dependencies, states, suspensions, profile, event) {
            replay_failure_index_from_spec(
                dependencies,
                replay_event_next_spec(dependencies, states, suspensions, profile, event),
                replay_event_next_suspensions_spec(
                    dependencies,
                    states,
                    suspensions,
                    profile,
                    event,
                ),
                profile,
                events,
                from + 1,
            )
        } else {
            Some(from)
        }
    }
}

pub open spec fn replay_dispatch_intents_spec(
    dependencies: Seq<alloc::vec::Vec<usize>>,
    states: Seq<u8>,
    suspensions: Seq<Option<crate::verified::primitive::support::IndexedSuspensionFact>>,
    profile: u8,
    events: Seq<IndexedReplayEvent>,
    from: int,
) -> Seq<usize> {
    Seq::empty()
}

} // verus!
