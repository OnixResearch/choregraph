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
// r[impl choregraph.proof.single_step.readiness_completeness]
// r[impl choregraph.proof.single_step.readiness_frontier]
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

pub const EPOCH_FENCE_ADMIT: u8 = 0u8;
pub const EPOCH_FENCE_UNKNOWN_ENTITY: u8 = 1u8;
pub const EPOCH_FENCE_ENTITY_MISMATCH: u8 = 2u8;
pub const EPOCH_FENCE_EPOCH_MISMATCH: u8 = 3u8;
pub const EPOCH_FENCE_TERMINAL_MUTATION: u8 = 4u8;

// r[impl choregraph.epoch_fence.proof]
// r[verify choregraph.epoch_fence.proof]
pub open spec fn epoch_fence_decision_spec(
    entity_known: bool,
    entity_matches: bool,
    current_epoch: u64,
    offered_epoch: u64,
    terminal: bool,
) -> u8 {
    if !entity_known {
        EPOCH_FENCE_UNKNOWN_ENTITY
    } else if !entity_matches {
        EPOCH_FENCE_ENTITY_MISMATCH
    } else if current_epoch != offered_epoch {
        EPOCH_FENCE_EPOCH_MISMATCH
    } else if terminal {
        EPOCH_FENCE_TERMINAL_MUTATION
    } else {
        EPOCH_FENCE_ADMIT
    }
}

pub const fn epoch_fence_decision(
    entity_known: bool,
    entity_matches: bool,
    current_epoch: u64,
    offered_epoch: u64,
    terminal: bool,
) -> (result: u8)
    ensures result == epoch_fence_decision_spec(
        entity_known,
        entity_matches,
        current_epoch,
        offered_epoch,
        terminal,
    )
{
    if !entity_known {
        EPOCH_FENCE_UNKNOWN_ENTITY
    } else if !entity_matches {
        EPOCH_FENCE_ENTITY_MISMATCH
    } else if current_epoch != offered_epoch {
        EPOCH_FENCE_EPOCH_MISMATCH
    } else if terminal {
        EPOCH_FENCE_TERMINAL_MUTATION
    } else {
        EPOCH_FENCE_ADMIT
    }
}

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


} // verus!

#[path = "verified/readiness.rs"]
mod readiness;
pub(crate) use readiness::*;

#[path = "verified/kernel.rs"]
mod kernel;
pub(crate) use kernel::*;

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
