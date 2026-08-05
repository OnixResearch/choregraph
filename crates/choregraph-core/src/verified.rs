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

// r[impl choregraph.proof.single_step.specifications]
// r[impl choregraph.proof.single_step.production_linkage]
// r[impl choregraph.proof.single_step.readiness]
// r[impl choregraph.proof.single_step.determinism]
// r[impl choregraph.proof.single_step.preservation]
// r[impl choregraph.proof.single_step.denial]
// r[impl choregraph.proof.single_step.trellis]
// r[impl choregraph.effect_v2.proof]
// r[verify choregraph.effect_v2.proof]
use alloc::vec::Vec;
use vstd::prelude::*;

verus! {

pub const STATE_PENDING: u8 = 0u8;
pub const STATE_READY: u8 = 1u8;
pub const STATE_RUNNING: u8 = 2u8;
pub const STATE_COMPLETED: u8 = 3u8;
pub const STATE_FAILED: u8 = 4u8;
pub const STATE_CANCELLED: u8 = 5u8;
pub const STATE_SUSPENDED: u8 = 6u8;

pub const ACTION_MARK_READY: u8 = 0u8;
pub const ACTION_START: u8 = 1u8;
pub const ACTION_COMPLETE: u8 = 2u8;
pub const ACTION_FAIL: u8 = 3u8;
pub const ACTION_CANCEL: u8 = 4u8;
pub const ACTION_SUSPEND: u8 = 5u8;
pub const ACTION_RESUME_SIGNAL: u8 = 6u8;

pub const PROFILE_LINEAR_V1: u8 = 0u8;
pub const PROFILE_DURABLE_SUSPENSION_V1: u8 = 1u8;

pub const BINDING_EXACT: u8 = 0u8;
pub const BINDING_SLOT: u8 = 1u8;

pub const DENIAL_NONE: u8 = 0u8;
pub const DENIAL_INVALID_STATE: u8 = 1u8;
pub const DENIAL_UNKNOWN_NODE: u8 = 2u8;
pub const DENIAL_TERMINAL_STATE: u8 = 3u8;
pub const DENIAL_INVALID_TRANSITION: u8 = 4u8;
pub const DENIAL_DEPENDENCIES_INCOMPLETE: u8 = 5u8;
pub const DENIAL_DUPLICATE_EFFECT: u8 = 6u8;
pub const DENIAL_UNDECLARED_EFFECT: u8 = 7u8;
pub const DENIAL_INVARIANT_WOULD_BREAK: u8 = 8u8;
pub const DENIAL_SUSPENSION_MISMATCH: u8 = 9u8;
pub const DENIAL_SUSPENSION_FACT_ABSENT: u8 = 10u8;
pub const DENIAL_SUSPENSION_FACT_PRESENT: u8 = 11u8;
pub const DENIAL_SUSPENSION_EFFECTS: u8 = 12u8;
pub const DENIAL_PROFILE_ACTION: u8 = 13u8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedSuspensionFact {
    pub class: Vec<u8>,
    pub reference: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IndexedDecision {
    pub accepted: bool,
    pub reason: u8,
    pub next_states: Vec<u8>,
    pub intent_indices: Vec<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IndexedConcreteReferenceWitness {
    pub request_reference: usize,
    pub payload_reference: usize,
}

pub open spec fn concrete_binding_accepts_spec(
    mode: u8,
    slot_matches: bool,
    source_matches: bool,
    exact_reference_matches: bool,
    slot_absent: bool,
    source_absent: bool,
) -> bool {
    if mode == BINDING_EXACT {
        slot_absent && source_absent && exact_reference_matches
    } else if mode == BINDING_SLOT {
        slot_matches && source_matches
    } else {
        false
    }
}

pub fn concrete_binding_accepts(
    mode: u8,
    slot_matches: bool,
    source_matches: bool,
    exact_reference_matches: bool,
    slot_absent: bool,
    source_absent: bool,
) -> (result: bool)
    ensures result == concrete_binding_accepts_spec(
        mode,
        slot_matches,
        source_matches,
        exact_reference_matches,
        slot_absent,
        source_absent,
    )
{
    if mode == BINDING_EXACT {
        slot_absent && source_absent && exact_reference_matches
    } else if mode == BINDING_SLOT {
        slot_matches && source_matches
    } else {
        false
    }
}

pub open spec fn concrete_references_preserved_spec(
    event_request_reference: usize,
    event_payload_reference: usize,
    intent_request_reference: usize,
    intent_payload_reference: usize,
) -> bool {
    intent_request_reference == event_request_reference
        && intent_payload_reference == event_payload_reference
}

pub fn concrete_reference_witness(
    event_request_reference: usize,
    event_payload_reference: usize,
) -> (result: IndexedConcreteReferenceWitness)
    ensures concrete_references_preserved_spec(
        event_request_reference,
        event_payload_reference,
        result.request_reference,
        result.payload_reference,
    )
{
    IndexedConcreteReferenceWitness {
        request_reference: event_request_reference,
        payload_reference: event_payload_reference,
    }
}

pub open spec fn local_state_valid_spec(state: u8) -> bool {
    state == STATE_PENDING
        || state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
        || state == STATE_CANCELLED
        || state == STATE_SUSPENDED
}

pub open spec fn terminal_state_spec(state: u8) -> bool {
    state == STATE_COMPLETED || state == STATE_FAILED || state == STATE_CANCELLED
}

pub open spec fn action_valid_spec(action: u8) -> bool {
    action == ACTION_MARK_READY
        || action == ACTION_START
        || action == ACTION_COMPLETE
        || action == ACTION_FAIL
        || action == ACTION_CANCEL
        || action == ACTION_SUSPEND
        || action == ACTION_RESUME_SIGNAL
}

pub open spec fn profile_valid_spec(profile: u8) -> bool {
    profile == PROFILE_LINEAR_V1 || profile == PROFILE_DURABLE_SUSPENSION_V1
}

pub open spec fn suspension_action_spec(action: u8) -> bool {
    action == ACTION_SUSPEND || action == ACTION_RESUME_SIGNAL
}

pub open spec fn linear_lifecycle_target_spec(from: u8, action: u8) -> Option<u8> {
    if from == STATE_PENDING && action == ACTION_MARK_READY {
        Some(STATE_READY)
    } else if from == STATE_READY && action == ACTION_START {
        Some(STATE_RUNNING)
    } else if from == STATE_RUNNING && action == ACTION_COMPLETE {
        Some(STATE_COMPLETED)
    } else if from == STATE_RUNNING && action == ACTION_FAIL {
        Some(STATE_FAILED)
    } else if (from == STATE_PENDING || from == STATE_READY || from == STATE_RUNNING)
        && action == ACTION_CANCEL
    {
        Some(STATE_CANCELLED)
    } else {
        None
    }
}

pub open spec fn lifecycle_target_spec(profile: u8, from: u8, action: u8) -> Option<u8> {
    if profile == PROFILE_LINEAR_V1 {
        linear_lifecycle_target_spec(from, action)
    } else if profile == PROFILE_DURABLE_SUSPENSION_V1 {
        if linear_lifecycle_target_spec(from, action).is_some() {
            linear_lifecycle_target_spec(from, action)
        } else if (from == STATE_READY || from == STATE_RUNNING) && action == ACTION_SUSPEND {
            Some(STATE_SUSPENDED)
        } else if from == STATE_SUSPENDED && action == ACTION_RESUME_SIGNAL {
            Some(STATE_READY)
        } else {
            None
        }
    } else {
        None
    }
}

pub const fn linear_lifecycle_target(from: u8, action: u8) -> (result: Option<u8>)
    ensures result == linear_lifecycle_target_spec(from, action)
{
    if from == STATE_PENDING && action == ACTION_MARK_READY {
        Some(STATE_READY)
    } else if from == STATE_READY && action == ACTION_START {
        Some(STATE_RUNNING)
    } else if from == STATE_RUNNING && action == ACTION_COMPLETE {
        Some(STATE_COMPLETED)
    } else if from == STATE_RUNNING && action == ACTION_FAIL {
        Some(STATE_FAILED)
    } else if (from == STATE_PENDING || from == STATE_READY || from == STATE_RUNNING)
        && action == ACTION_CANCEL
    {
        Some(STATE_CANCELLED)
    } else {
        None
    }
}

pub const fn lifecycle_target(profile: u8, from: u8, action: u8) -> (result: Option<u8>)
    ensures result == lifecycle_target_spec(profile, from, action)
{
    if profile == PROFILE_LINEAR_V1 {
        linear_lifecycle_target(from, action)
    } else if profile == PROFILE_DURABLE_SUSPENSION_V1 {
        let linear = linear_lifecycle_target(from, action);
        if linear.is_some() {
            linear
        } else if (from == STATE_READY || from == STATE_RUNNING) && action == ACTION_SUSPEND {
            Some(STATE_SUSPENDED)
        } else if from == STATE_SUSPENDED && action == ACTION_RESUME_SIGNAL {
            Some(STATE_READY)
        } else {
            None
        }
    } else {
        None
    }
}

pub open spec fn dependencies_complete_from_spec(
    dependencies: Seq<usize>,
    states: Seq<u8>,
    from: int,
) -> bool
    decreases dependencies.len() - from
{
    if from < 0 || from >= dependencies.len() {
        true
    } else {
        dependencies[from] < states.len()
            && states[dependencies[from] as int] == STATE_COMPLETED
            && dependencies_complete_from_spec(dependencies, states, from + 1)
    }
}

pub open spec fn dependencies_complete_spec(
    dependencies: Seq<usize>,
    states: Seq<u8>,
) -> bool {
    dependencies_complete_from_spec(dependencies, states, 0)
}

pub fn dependencies_complete(dependencies: &[usize], states: &[u8]) -> (result: bool)
    ensures result == dependencies_complete_spec(dependencies@, states@)
{
    let mut index: usize = 0;
    while index < dependencies.len()
        invariant
            0 <= index <= dependencies@.len(),
            dependencies_complete_spec(dependencies@, states@)
                <==> dependencies_complete_from_spec(dependencies@, states@, index as int),
        decreases dependencies@.len() - index,
    {
        let dependency = dependencies[index];
        if dependency >= states.len() || states[dependency] != STATE_COMPLETED {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn state_requires_completed_dependencies_spec(state: u8) -> bool {
    state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
}

pub open spec fn indexed_invariant_from_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    from: int,
) -> bool
    decreases states.len() - from
{
    if from < 0 || from >= states.len() {
        true
    } else {
        local_state_valid_spec(states[from])
            && dependencies[from].len() <= states.len()
            && (!state_requires_completed_dependencies_spec(states[from])
                || dependencies_complete_spec(dependencies[from]@, states))
            && indexed_invariant_from_spec(dependencies, states, from + 1)
    }
}

pub open spec fn indexed_invariant_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
) -> bool {
    dependencies.len() == states.len()
        && indexed_invariant_from_spec(dependencies, states, 0)
}

pub fn indexed_invariant(dependencies: &[Vec<usize>], states: &[u8]) -> (result: bool)
    ensures result == indexed_invariant_spec(dependencies@, states@)
{
    if dependencies.len() != states.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < states.len()
        invariant
            dependencies@.len() == states@.len(),
            0 <= index <= states@.len(),
            indexed_invariant_from_spec(dependencies@, states@, 0)
                <==> indexed_invariant_from_spec(dependencies@, states@, index as int),
        decreases states@.len() - index,
    {
        let state = states[index];
        if !local_state_valid(state) {
            return false;
        }
        let node_dependencies = &dependencies[index];
        if node_dependencies.len() > states.len() {
            return false;
        }
        if state_requires_completed_dependencies(state)
            && !dependencies_complete(node_dependencies, states)
        {
            return false;
        }
        index = index + 1;
    }
    true
}

pub fn local_state_valid(state: u8) -> (result: bool)
    ensures result == local_state_valid_spec(state)
{
    state == STATE_PENDING
        || state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
        || state == STATE_CANCELLED
        || state == STATE_SUSPENDED
}

pub fn state_requires_completed_dependencies(state: u8) -> (result: bool)
    ensures result == state_requires_completed_dependencies_spec(state)
{
    state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
}

pub open spec fn terminal_states_preserved_from_spec(
    before: Seq<u8>,
    after: Seq<u8>,
    from: int,
) -> bool
    decreases before.len() - from
{
    if from < 0 || from >= before.len() {
        true
    } else {
        (!terminal_state_spec(before[from]) || after[from] == before[from])
            && terminal_states_preserved_from_spec(before, after, from + 1)
    }
}

pub open spec fn terminal_states_preserved_spec(before: Seq<u8>, after: Seq<u8>) -> bool {
    before.len() == after.len()
        && terminal_states_preserved_from_spec(before, after, 0)
}

pub fn terminal_states_preserved(before: &[u8], after: &[u8]) -> (result: bool)
    ensures result == terminal_states_preserved_spec(before@, after@)
{
    if before.len() != after.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < before.len()
        invariant
            before@.len() == after@.len(),
            0 <= index <= before@.len(),
            terminal_states_preserved_from_spec(before@, after@, 0)
                <==> terminal_states_preserved_from_spec(before@, after@, index as int),
        decreases before@.len() - index,
    {
        let state = before[index];
        if terminal_state(state) && after[index] != state {
            return false;
        }
        index = index + 1;
    }
    true
}

pub fn terminal_state(state: u8) -> (result: bool)
    ensures result == terminal_state_spec(state)
{
    state == STATE_COMPLETED || state == STATE_FAILED || state == STATE_CANCELLED
}

pub open spec fn contains_from_spec(values: Seq<usize>, needle: usize, from: int) -> bool
    decreases values.len() - from
{
    if from < 0 || from >= values.len() {
        false
    } else {
        values[from] == needle || contains_from_spec(values, needle, from + 1)
    }
}

pub open spec fn contains_spec(values: Seq<usize>, needle: usize) -> bool {
    contains_from_spec(values, needle, 0)
}

pub fn contains(values: &[usize], needle: usize) -> (result: bool)
    ensures result == contains_spec(values@, needle)
{
    let mut index: usize = 0;
    while index < values.len()
        invariant
            0 <= index <= values@.len(),
            contains_spec(values@, needle)
                <==> contains_from_spec(values@, needle, index as int),
        decreases values@.len() - index,
    {
        if values[index] == needle {
            return true;
        }
        index = index + 1;
    }
    false
}

pub open spec fn all_requested_allowed_from_spec(
    allowed: Seq<usize>,
    requested: Seq<usize>,
    from: int,
) -> bool
    decreases requested.len() - from
{
    if from < 0 || from >= requested.len() {
        true
    } else {
        contains_spec(allowed, requested[from])
            && all_requested_allowed_from_spec(allowed, requested, from + 1)
    }
}

pub open spec fn all_requested_allowed_spec(
    allowed: Seq<usize>,
    requested: Seq<usize>,
) -> bool {
    all_requested_allowed_from_spec(allowed, requested, 0)
}

pub fn all_requested_allowed(allowed: &[usize], requested: &[usize]) -> (result: bool)
    ensures result == all_requested_allowed_spec(allowed@, requested@)
{
    let mut index: usize = 0;
    while index < requested.len()
        invariant
            0 <= index <= requested@.len(),
            all_requested_allowed_spec(allowed@, requested@)
                <==> all_requested_allowed_from_spec(allowed@, requested@, index as int),
        decreases requested@.len() - index,
    {
        if !contains(allowed, requested[index]) {
            return false;
        }
        index = index + 1;
    }
    true
}

pub open spec fn not_in_suffix_spec(
    values: Seq<usize>,
    needle: usize,
    from: int,
) -> bool
    decreases values.len() - from
{
    if from < 0 || from >= values.len() {
        true
    } else {
        values[from] != needle && not_in_suffix_spec(values, needle, from + 1)
    }
}

pub open spec fn all_distinct_from_spec(values: Seq<usize>, from: int) -> bool
    decreases values.len() - from
{
    if from < 0 || from >= values.len() {
        true
    } else {
        not_in_suffix_spec(values, values[from], from + 1)
            && all_distinct_from_spec(values, from + 1)
    }
}

pub open spec fn all_distinct_spec(values: Seq<usize>) -> bool {
    all_distinct_from_spec(values, 0)
}

pub fn all_distinct(values: &[usize]) -> (result: bool)
    ensures result == all_distinct_spec(values@)
{
    let mut left: usize = 0;
    while left < values.len()
        invariant
            0 <= left <= values@.len(),
            all_distinct_spec(values@)
                <==> all_distinct_from_spec(values@, left as int),
        decreases values@.len() - left,
    {
        let mut right = left + 1;
        while right < values.len()
            invariant
                left < values@.len(),
                left < right <= values@.len(),
                all_distinct_spec(values@)
                    <==> all_distinct_from_spec(values@, left as int),
                not_in_suffix_spec(values@, values@[left as int], (left + 1) as int)
                    <==> not_in_suffix_spec(values@, values@[left as int], right as int),
            decreases values@.len() - right,
        {
            if values[left] == values[right] {
                proof {
                    assert(!not_in_suffix_spec(
                        values@,
                        values@[left as int],
                        right as int,
                    ));
                    assert(!not_in_suffix_spec(
                        values@,
                        values@[left as int],
                        (left + 1) as int,
                    ));
                    assert(!all_distinct_from_spec(values@, left as int));
                    assert(!all_distinct_spec(values@));
                }
                return false;
            }
            right = right + 1;
        }
        left = left + 1;
    }
    true
}

pub open spec fn effects_allowed_spec(allowed: Seq<usize>, requested: Seq<usize>) -> bool {
    all_requested_allowed_spec(allowed, requested) && all_distinct_spec(requested)
}

pub fn copy_values(values: &[usize]) -> (result: Vec<usize>)
    ensures result@ == values@
{
    let mut result = Vec::new();
    let mut index: usize = 0;
    while index < values.len()
        invariant
            0 <= index <= values@.len(),
            result@ == values@.subrange(0, index as int),
        decreases values@.len() - index,
    {
        result.push(values[index]);
        index = index + 1;
    }
    result
}

pub fn copy_states(values: &[u8]) -> (result: Vec<u8>)
    ensures result@ == values@
{
    let mut result = Vec::new();
    let mut index: usize = 0;
    while index < values.len()
        invariant
            0 <= index <= values@.len(),
            result@ == values@.subrange(0, index as int),
        decreases values@.len() - index,
    {
        result.push(values[index]);
        index = index + 1;
    }
    result
}

pub open spec fn suspensions_consistent_spec(
    states: Seq<u8>,
    suspensions: Seq<Option<IndexedSuspensionFact>>,
) -> bool {
    &&& states.len() == suspensions.len()
    &&& forall|index: int| 0 <= index < states.len() ==> (suspensions[index].is_some()
        == (states[index] == STATE_SUSPENDED))
}

pub fn suspensions_consistent(
    states: &[u8],
    suspensions: &[Option<IndexedSuspensionFact>],
) -> (result: bool)
    ensures result == suspensions_consistent_spec(states@, suspensions@)
{
    if states.len() != suspensions.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < suspensions.len()
        invariant
            states@.len() == suspensions@.len(),
            0 <= index <= suspensions@.len(),
            forall|offset: int| 0 <= offset < index ==> (suspensions@[offset].is_some()
                == (states@[offset] == STATE_SUSPENDED)),
        decreases suspensions@.len() - index,
    {
        let has_record = suspensions[index].is_some();
        if has_record != (states[index] == STATE_SUSPENDED) {
            return false;
        }
        index = index + 1;
    }
    true
}

pub fn bytes_equal(left: &[u8], right: &[u8]) -> (result: bool)
    ensures result == (left@ == right@)
{
    if left.len() != right.len() {
        return false;
    }
    let mut index: usize = 0;
    while index < left.len()
        invariant
            left@.len() == right@.len(),
            0 <= index <= left@.len(),
            forall|offset: int| 0 <= offset < index ==> left@[offset] == right@[offset],
        decreases left@.len() - index,
    {
        if left[index] != right[index] {
            return false;
        }
        index = index + 1;
    }
    proof {
        assert(left@ =~= right@);
    }
    true
}

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
            let classes_equal = bytes_equal(&stored_fact.class, &event_fact.class);
            let references_equal = bytes_equal(&stored_fact.reference, &event_fact.reference);
            classes_equal && references_equal
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

#[allow(clippy::too_many_arguments, reason = "the kernel takes only explicit indexed inputs")]
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
    let suspension_action = action == ACTION_SUSPEND || action == ACTION_RESUME_SIGNAL;
    if suspension_action && profile == PROFILE_LINEAR_V1 {
        return deny(DENIAL_PROFILE_ACTION);
    }
    let target = match lifecycle_target(profile, from, action) {
        Some(target) => target,
        None => return deny(DENIAL_INVALID_TRANSITION),
    };
    if suspension_action && event_fact.is_none() {
        return deny(DENIAL_SUSPENSION_FACT_ABSENT);
    }
    if !suspension_action && event_fact.is_some() {
        return deny(DENIAL_SUSPENSION_FACT_PRESENT);
    }
    if suspension_action && !requested_intents.is_empty() {
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

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const DECLARED_INTENT_INDEX: usize = 3;

    #[test]
    fn indexed_step_accepts_ready_transition_with_declared_intent() {
        let dependencies = vec![vec![], vec![0]];
        let states = vec![STATE_COMPLETED, STATE_PENDING];
        let suspensions = vec![None, None];
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            1,
            ACTION_MARK_READY,
            &None,
            &allowed,
            &requested,
        );

        assert!(decision.accepted);
        assert_eq!(decision.reason, DENIAL_NONE);
        assert_eq!(decision.next_states, vec![STATE_COMPLETED, STATE_READY]);
        assert_eq!(decision.intent_indices, requested);
    }

    #[test]
    fn indexed_step_denies_incomplete_dependency_without_outputs() {
        let dependencies = vec![vec![], vec![0]];
        let states = vec![STATE_RUNNING, STATE_PENDING];
        let suspensions = vec![None, None];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            1,
            ACTION_MARK_READY,
            &None,
            &[],
            &[],
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_DEPENDENCIES_INCOMPLETE);
        assert!(decision.next_states.is_empty());
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn indexed_step_denies_duplicate_intents_without_outputs() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_PENDING];
        let suspensions = vec![None];
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX, DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            0,
            ACTION_CANCEL,
            &None,
            &allowed,
            &requested,
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_DUPLICATE_EFFECT);
        assert!(decision.next_states.is_empty());
        assert!(decision.intent_indices.is_empty());
    }

    fn suspension_fact(class: &[u8], reference: &[u8]) -> Option<IndexedSuspensionFact> {
        Some(IndexedSuspensionFact {
            class: class.to_vec(),
            reference: reference.to_vec(),
        })
    }

    #[test]
    fn durable_profile_suspend_records_fact_without_effects() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_READY];
        let suspensions = vec![None];
        let fact = suspension_fact(b"approval", b"blake3:00");

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_SUSPEND,
            &fact,
            &[],
            &[],
        );

        assert!(decision.accepted);
        assert_eq!(decision.next_states, vec![STATE_SUSPENDED]);
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn durable_profile_matching_resume_clears_fact() {
        let dependencies = vec![vec![]];
        let fact = suspension_fact(b"approval", b"blake3:00");
        let states = vec![STATE_SUSPENDED];
        let suspensions = vec![fact.clone()];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_RESUME_SIGNAL,
            &fact,
            &[],
            &[],
        );

        assert!(decision.accepted);
        assert_eq!(decision.next_states, vec![STATE_READY]);
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn durable_profile_mismatched_resume_denies() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_SUSPENDED];
        let suspensions = vec![suspension_fact(b"approval", b"blake3:00")];
        let wrong = suspension_fact(b"approval", b"blake3:ff");

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_RESUME_SIGNAL,
            &wrong,
            &[],
            &[],
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_SUSPENSION_MISMATCH);
        assert!(decision.next_states.is_empty());
    }

    #[test]
    fn linear_profile_rejects_suspension_actions() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_READY];
        let suspensions = vec![None];
        let fact = suspension_fact(b"approval", b"blake3:00");

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_LINEAR_V1,
            0,
            ACTION_SUSPEND,
            &fact,
            &[],
            &[],
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_PROFILE_ACTION);
    }

    #[test]
    fn suspension_action_rejects_effect_requests() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_READY];
        let suspensions = vec![None];
        let fact = suspension_fact(b"approval", b"blake3:00");
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            &suspensions,
            PROFILE_DURABLE_SUSPENSION_V1,
            0,
            ACTION_SUSPEND,
            &fact,
            &allowed,
            &requested,
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_SUSPENSION_EFFECTS);
    }
}
