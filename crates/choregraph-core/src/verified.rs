// Verus proof syntax and production-linked specifications require these local source shapes.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    clippy::assign_op_pattern,
    nested_conditionals,
    non_trait_imports,
    raw_arithmetic_overflow,
    too_many_parameters,
    unbounded_collection_growth,
    usize_in_public_api,
    reason = "Verus proof syntax and production-linked specifications require these local source shapes"
)]

// r[impl choregraph.proof.single_step.specifications]
// r[impl choregraph.proof.single_step.production_linkage]
// r[impl choregraph.proof.single_step.readiness]
// r[impl choregraph.proof.single_step.determinism]
// r[impl choregraph.proof.single_step.preservation]
// r[impl choregraph.proof.single_step.denial]
// r[impl choregraph.proof.single_step.trellis]
use alloc::vec::Vec;
use vstd::prelude::*;

verus! {

pub const STATE_PENDING: u8 = 0u8;
pub const STATE_READY: u8 = 1u8;
pub const STATE_RUNNING: u8 = 2u8;
pub const STATE_COMPLETED: u8 = 3u8;
pub const STATE_FAILED: u8 = 4u8;
pub const STATE_CANCELLED: u8 = 5u8;

pub const ACTION_MARK_READY: u8 = 0u8;
pub const ACTION_START: u8 = 1u8;
pub const ACTION_COMPLETE: u8 = 2u8;
pub const ACTION_FAIL: u8 = 3u8;
pub const ACTION_CANCEL: u8 = 4u8;

pub const DENIAL_NONE: u8 = 0u8;
pub const DENIAL_INVALID_STATE: u8 = 1u8;
pub const DENIAL_UNKNOWN_NODE: u8 = 2u8;
pub const DENIAL_TERMINAL_STATE: u8 = 3u8;
pub const DENIAL_INVALID_TRANSITION: u8 = 4u8;
pub const DENIAL_DEPENDENCIES_INCOMPLETE: u8 = 5u8;
pub const DENIAL_DUPLICATE_EFFECT: u8 = 6u8;
pub const DENIAL_UNDECLARED_EFFECT: u8 = 7u8;
pub const DENIAL_INVARIANT_WOULD_BREAK: u8 = 8u8;

#[derive(Debug, Eq, PartialEq)]
pub struct IndexedDecision {
    pub accepted: bool,
    pub reason: u8,
    pub next_states: Vec<u8>,
    pub intent_indices: Vec<usize>,
}

pub open spec fn local_state_valid_spec(state: u8) -> bool {
    state == STATE_PENDING
        || state == STATE_READY
        || state == STATE_RUNNING
        || state == STATE_COMPLETED
        || state == STATE_FAILED
        || state == STATE_CANCELLED
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
}

pub open spec fn lifecycle_target_spec(from: u8, action: u8) -> Option<u8> {
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

pub const fn lifecycle_target(from: u8, action: u8) -> (result: Option<u8>)
    ensures result == lifecycle_target_spec(from, action)
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

pub open spec fn step_target_spec(states: Seq<u8>, node_index: usize, action: u8) -> Option<u8> {
    if node_index < states.len() {
        lifecycle_target_spec(states[node_index as int], action)
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
    action != ACTION_MARK_READY && action != ACTION_START
        || dependencies_complete_spec(dependencies[node_index as int]@, states)
}

pub open spec fn step_next_states_spec(
    states: Seq<u8>,
    node_index: usize,
    target: u8,
) -> Seq<u8> {
    states.update(node_index as int, target)
}

pub open spec fn indexed_step_accepts_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> bool {
    indexed_invariant_spec(dependencies, states)
        && node_index < states.len()
        && step_target_spec(states, node_index, action).is_some()
        && step_dependencies_ready_spec(dependencies, states, node_index, action)
        && effects_allowed_spec(allowed_intents, requested_intents)
        && indexed_invariant_spec(
            dependencies,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(states, node_index, action).unwrap(),
            ),
        )
        && terminal_states_preserved_spec(
            states,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(states, node_index, action).unwrap(),
            ),
        )
}

pub open spec fn indexed_step_reason_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> u8 {
    if !indexed_invariant_spec(dependencies, states) {
        DENIAL_INVALID_STATE
    } else if node_index >= states.len() {
        DENIAL_UNKNOWN_NODE
    } else if terminal_state_spec(states[node_index as int]) {
        DENIAL_TERMINAL_STATE
    } else if step_target_spec(states, node_index, action).is_none() {
        DENIAL_INVALID_TRANSITION
    } else if !step_dependencies_ready_spec(dependencies, states, node_index, action) {
        DENIAL_DEPENDENCIES_INCOMPLETE
    } else if !all_distinct_spec(requested_intents) {
        DENIAL_DUPLICATE_EFFECT
    } else if !all_requested_allowed_spec(allowed_intents, requested_intents) {
        DENIAL_UNDECLARED_EFFECT
    } else {
        let target = step_target_spec(states, node_index, action).unwrap();
        let next = step_next_states_spec(states, node_index, target);
        if !indexed_invariant_spec(dependencies, next)
            || !terminal_states_preserved_spec(states, next)
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
    node_index: usize,
    action: u8,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> Seq<u8> {
    if indexed_step_accepts_spec(
        dependencies,
        states,
        node_index,
        action,
        allowed_intents,
        requested_intents,
    ) {
        step_next_states_spec(
            states,
            node_index,
            step_target_spec(states, node_index, action).unwrap(),
        )
    } else {
        Seq::empty()
    }
}

pub open spec fn indexed_step_intents_spec(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
) -> Seq<usize> {
    if indexed_step_accepts_spec(
        dependencies,
        states,
        node_index,
        action,
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

pub fn indexed_step(
    dependencies: &[Vec<usize>],
    states: &[u8],
    node_index: usize,
    action: u8,
    allowed_intents: &[usize],
    requested_intents: &[usize],
) -> (result: IndexedDecision)
    ensures
        result.accepted == indexed_step_accepts_spec(
            dependencies@,
            states@,
            node_index,
            action,
            allowed_intents@,
            requested_intents@,
        ),
        result.reason == indexed_step_reason_spec(
            dependencies@,
            states@,
            node_index,
            action,
            allowed_intents@,
            requested_intents@,
        ),
        result.next_states@ == indexed_step_next_spec(
            dependencies@,
            states@,
            node_index,
            action,
            allowed_intents@,
            requested_intents@,
        ),
        result.intent_indices@ == indexed_step_intents_spec(
            dependencies@,
            states@,
            node_index,
            action,
            allowed_intents@,
            requested_intents@,
        ),
        result.accepted ==> indexed_invariant_spec(dependencies@, result.next_states@),
        result.accepted ==> terminal_states_preserved_spec(states@, result.next_states@),
        result.accepted ==> effects_allowed_spec(allowed_intents@, result.intent_indices@),
        result.accepted ==> result.intent_indices@ == requested_intents@,
        !result.accepted ==> result.next_states@.len() == 0,
        !result.accepted ==> result.intent_indices@.len() == 0,
{
    if !indexed_invariant(dependencies, states) {
        return deny(DENIAL_INVALID_STATE);
    }
    if node_index >= states.len() {
        return deny(DENIAL_UNKNOWN_NODE);
    }
    let from = states[node_index];
    if terminal_state(from) {
        return deny(DENIAL_TERMINAL_STATE);
    }
    let target = match lifecycle_target(from, action) {
        Some(target) => target,
        None => return deny(DENIAL_INVALID_TRANSITION),
    };
    if (action == ACTION_MARK_READY || action == ACTION_START)
        && !dependencies_complete(&dependencies[node_index], states)
    {
        return deny(DENIAL_DEPENDENCIES_INCOMPLETE);
    }
    if !all_distinct(requested_intents) {
        return deny(DENIAL_DUPLICATE_EFFECT);
    }
    if !all_requested_allowed(allowed_intents, requested_intents) {
        return deny(DENIAL_UNDECLARED_EFFECT);
    }

    let mut next_states = copy_states(states);
    next_states.set(node_index, target);
    if !indexed_invariant(dependencies, &next_states)
        || !terminal_states_preserved(states, &next_states)
    {
        return deny(DENIAL_INVARIANT_WOULD_BREAK);
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
    node_index: usize,
    action: u8,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires indexed_step_accepts_spec(
        dependencies,
        states,
        node_index,
        action,
        allowed_intents,
        requested_intents,
    )
    ensures
        indexed_invariant_spec(
            dependencies,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(states, node_index, action).unwrap(),
            ),
        ),
        effects_allowed_spec(allowed_intents, requested_intents),
        terminal_states_preserved_spec(
            states,
            step_next_states_spec(
                states,
                node_index,
                step_target_spec(states, node_index, action).unwrap(),
            ),
        ),
{}

// r[verify choregraph.proof.single_step.denial]
pub proof fn denial_has_no_replacement_or_intents(
    dependencies: Seq<Vec<usize>>,
    states: Seq<u8>,
    node_index: usize,
    action: u8,
    allowed_intents: Seq<usize>,
    requested_intents: Seq<usize>,
)
    requires !indexed_step_accepts_spec(
        dependencies,
        states,
        node_index,
        action,
        allowed_intents,
        requested_intents,
    )
    ensures
        indexed_step_next_spec(
            dependencies,
            states,
            node_index,
            action,
            allowed_intents,
            requested_intents,
        ).len() == 0,
        indexed_step_intents_spec(
            dependencies,
            states,
            node_index,
            action,
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
    node_index_a: usize,
    node_index_b: usize,
    action_a: u8,
    action_b: u8,
    allowed_intents_a: Seq<usize>,
    allowed_intents_b: Seq<usize>,
    requested_intents_a: Seq<usize>,
    requested_intents_b: Seq<usize>,
)
    requires
        dependencies_a == dependencies_b,
        states_a == states_b,
        node_index_a == node_index_b,
        action_a == action_b,
        allowed_intents_a == allowed_intents_b,
        requested_intents_a == requested_intents_b,
    ensures
        indexed_step_accepts_spec(
            dependencies_a,
            states_a,
            node_index_a,
            action_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_accepts_spec(
            dependencies_b,
            states_b,
            node_index_b,
            action_b,
            allowed_intents_b,
            requested_intents_b,
        ),
        indexed_step_reason_spec(
            dependencies_a,
            states_a,
            node_index_a,
            action_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_reason_spec(
            dependencies_b,
            states_b,
            node_index_b,
            action_b,
            allowed_intents_b,
            requested_intents_b,
        ),
        indexed_step_next_spec(
            dependencies_a,
            states_a,
            node_index_a,
            action_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_next_spec(
            dependencies_b,
            states_b,
            node_index_b,
            action_b,
            allowed_intents_b,
            requested_intents_b,
        ),
        indexed_step_intents_spec(
            dependencies_a,
            states_a,
            node_index_a,
            action_a,
            allowed_intents_a,
            requested_intents_a,
        ) == indexed_step_intents_spec(
            dependencies_b,
            states_b,
            node_index_b,
            action_b,
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
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            1,
            ACTION_MARK_READY,
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

        let decision = indexed_step(&dependencies, &states, 1, ACTION_MARK_READY, &[], &[]);

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_DEPENDENCIES_INCOMPLETE);
        assert!(decision.next_states.is_empty());
        assert!(decision.intent_indices.is_empty());
    }

    #[test]
    fn indexed_step_denies_duplicate_intents_without_outputs() {
        let dependencies = vec![vec![]];
        let states = vec![STATE_PENDING];
        let allowed = vec![DECLARED_INTENT_INDEX];
        let requested = vec![DECLARED_INTENT_INDEX, DECLARED_INTENT_INDEX];

        let decision = indexed_step(
            &dependencies,
            &states,
            0,
            ACTION_CANCEL,
            &allowed,
            &requested,
        );

        assert!(!decision.accepted);
        assert_eq!(decision.reason, DENIAL_DUPLICATE_EFFECT);
        assert!(decision.next_states.is_empty());
        assert!(decision.intent_indices.is_empty());
    }
}
