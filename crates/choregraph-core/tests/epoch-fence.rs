// These integration fixtures use direct domain imports, production defaults, and failing assertions.
#![allow(
    unknown_lints,
    explicit_defaults,
    no_panic,
    non_trait_imports,
    raw_arithmetic_overflow,
    reason = "integration fixtures keep reviewed epoch and rejection notation readable"
)]

use std::collections::BTreeMap;

use choregraph_core::{
    Action, AdmissionLimits, AdmittedMachine, Decision, DenialReason,
    EPOCH_FENCE_CONTRACT_IDENTITY, EPOCH_FENCE_IMPLEMENTATION_IDENTITY, EPOCH_FENCE_NON_CLAIMS,
    EntityEpoch, EpochFence, EpochFenceDisposition, EpochFenceRejection, Event, LocalState,
    MachineSpec, MachineState, NodeId, NodeSpec, step_epoch_fenced,
};

const ENTITY_A: &str = "entity/a";
const ENTITY_B: &str = "entity/b";
const UNKNOWN_ENTITY: &str = "entity/unknown";
const CURRENT_EPOCH: u64 = 8;
const STALE_EPOCH: u64 = CURRENT_EPOCH - 1;
const FUTURE_EPOCH: u64 = CURRENT_EPOCH + 1;

fn machine() -> AdmittedMachine {
    let spec = MachineSpec::new(
        vec![
            NodeSpec::new(ENTITY_A, Vec::new()),
            NodeSpec::new(ENTITY_B, Vec::new()),
        ],
        Vec::new(),
    );
    match AdmittedMachine::admit(spec, AdmissionLimits::default()) {
        Ok(machine) => machine,
        Err(diagnostics) => panic!("epoch-fence machine admission failed: {diagnostics:?}"),
    }
}

fn entity(value: &str) -> NodeId {
    match NodeId::parse(value, AdmissionLimits::default().max_identifier_bytes) {
        Ok(entity) => entity,
        Err(error) => panic!("entity identifier failed: {error:?}"),
    }
}

fn fence(value: &str, offered_epoch: u64) -> EpochFence {
    EpochFence::new(
        entity(value),
        EntityEpoch::new(CURRENT_EPOCH),
        EntityEpoch::new(offered_epoch),
    )
}

#[test]
fn current_epoch_admits_exactly_one_entity_through_production_step() {
    let machine = machine();
    let initial = machine.initial_state();
    let result = step_epoch_fenced(
        &machine,
        &initial,
        &Event::new(ENTITY_A, Action::MarkReady),
        &fence(ENTITY_A, CURRENT_EPOCH),
    );

    assert_eq!(result.fence.disposition, EpochFenceDisposition::Admit);
    assert_eq!(result.fence.entity.as_str(), ENTITY_A);
    assert_eq!(result.fence.current_epoch.get(), CURRENT_EPOCH);
    assert_eq!(result.fence.offered_epoch.get(), CURRENT_EPOCH);
    assert_eq!(
        result.fence.implementation_identity,
        EPOCH_FENCE_IMPLEMENTATION_IDENTITY
    );
    assert_eq!(
        result.fence.contract_identity,
        EPOCH_FENCE_CONTRACT_IDENTITY
    );
    assert_eq!(result.fence.required_non_claims, EPOCH_FENCE_NON_CLAIMS);
    let Some(Decision::Accept { next, intents }) = result.transition else {
        panic!("current epoch must reach the production transition");
    };
    assert_eq!(next.state(&entity(ENTITY_A)), Some(LocalState::Ready));
    assert_eq!(next.state(&entity(ENTITY_B)), Some(LocalState::Pending));
    assert!(intents.is_empty());
}

#[test]
fn stale_and_future_epochs_fail_closed_without_replacement_state() {
    let machine = machine();
    let initial = machine.initial_state();
    for offered_epoch in [STALE_EPOCH, FUTURE_EPOCH] {
        let result = step_epoch_fenced(
            &machine,
            &initial,
            &Event::new(ENTITY_A, Action::MarkReady),
            &fence(ENTITY_A, offered_epoch),
        );
        assert_eq!(
            result.fence.disposition,
            EpochFenceDisposition::Reject(EpochFenceRejection::EpochMismatch)
        );
        assert!(result.transition.is_none());
        assert_eq!(initial.state(&entity(ENTITY_A)), Some(LocalState::Pending));
        assert_eq!(initial.state(&entity(ENTITY_B)), Some(LocalState::Pending));
    }
}

#[test]
fn cross_entity_and_unknown_entity_mutations_are_rejected() {
    let machine = machine();
    let initial = machine.initial_state();
    let cross_entity = step_epoch_fenced(
        &machine,
        &initial,
        &Event::new(ENTITY_B, Action::MarkReady),
        &fence(ENTITY_A, CURRENT_EPOCH),
    );
    assert_eq!(
        cross_entity.fence.disposition,
        EpochFenceDisposition::Reject(EpochFenceRejection::EntityMismatch)
    );
    assert!(cross_entity.transition.is_none());

    let unknown = step_epoch_fenced(
        &machine,
        &initial,
        &Event::new(UNKNOWN_ENTITY, Action::MarkReady),
        &fence(UNKNOWN_ENTITY, CURRENT_EPOCH),
    );
    assert_eq!(
        unknown.fence.disposition,
        EpochFenceDisposition::Reject(EpochFenceRejection::UnknownEntity)
    );
    assert!(unknown.transition.is_none());
    assert_eq!(initial, machine.initial_state());
}

#[test]
fn terminal_mutation_is_rejected_before_production_transition() {
    let machine = machine();
    let terminal = MachineState::new(BTreeMap::from([
        (entity(ENTITY_A), LocalState::Completed),
        (entity(ENTITY_B), LocalState::Pending),
    ]));
    let result = step_epoch_fenced(
        &machine,
        &terminal,
        &Event::new(ENTITY_A, Action::Cancel),
        &fence(ENTITY_A, CURRENT_EPOCH),
    );
    assert_eq!(
        result.fence.disposition,
        EpochFenceDisposition::Reject(EpochFenceRejection::TerminalMutation)
    );
    assert!(result.transition.is_none());
    assert_eq!(
        terminal.state(&entity(ENTITY_A)),
        Some(LocalState::Completed)
    );
}

#[test]
fn admitted_fence_does_not_override_other_machine_denials() {
    let machine = machine();
    let initial = machine.initial_state();
    let result = step_epoch_fenced(
        &machine,
        &initial,
        &Event::new(ENTITY_A, Action::Start),
        &fence(ENTITY_A, CURRENT_EPOCH),
    );
    assert_eq!(result.fence.disposition, EpochFenceDisposition::Admit);
    assert_eq!(
        result.transition,
        Some(Decision::Deny {
            reason: DenialReason::InvalidTransition {
                from: LocalState::Pending,
                action: Action::Start,
            },
        })
    );
}

#[test]
fn identity_and_non_claims_stay_bounded_and_canonical() {
    assert!(EPOCH_FENCE_CONTRACT_IDENTITY.starts_with("blake3:"));
    assert!(EPOCH_FENCE_IMPLEMENTATION_IDENTITY.starts_with("blake3:"));
    assert_eq!(
        EPOCH_FENCE_CONTRACT_IDENTITY.len(),
        choregraph_core::BLAKE3_REFERENCE_BYTES
    );
    assert_eq!(
        EPOCH_FENCE_IMPLEMENTATION_IDENTITY.len(),
        choregraph_core::BLAKE3_REFERENCE_BYTES
    );
    assert!(!EPOCH_FENCE_NON_CLAIMS.is_empty());
    for non_claim in EPOCH_FENCE_NON_CLAIMS {
        assert!(!non_claim.is_empty());
    }
}
