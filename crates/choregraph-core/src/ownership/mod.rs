// The epoch fence is a pure typed gate over caller-supplied ownership facts.
#![allow(
    unknown_lints,
    non_trait_imports,
    missing_const_fn,
    reason = "Rust 1.97 cannot const-evaluate the required BTreeMap, string, and MachineState lookups"
)]

// r[impl choregraph.epoch_fence.invariant]
// r[impl choregraph.epoch_fence.decision]
// r[impl choregraph.epoch_fence.proof]
use crate::id::NodeId;
use crate::model::{AdmittedMachine, Decision, Event, LocalState, MachineState};

/// Stable contract identity for the same-entity, equal-epoch, non-terminal gate.
pub const EPOCH_FENCE_CONTRACT_IDENTITY: &str =
    "blake3:791e39edd60b0ae0520bc57b08cc13e1b6d36fe0679faf798d8f939995afe63c";

/// Stable implementation identity for the proof-linked production decision.
pub const EPOCH_FENCE_IMPLEMENTATION_IDENTITY: &str =
    "blake3:208455a5a682f658e7a0c4fa0cdd3224a2cc0e44d11ca5615450224c972717c7";

/// Claim boundaries attached to every epoch-fence decision.
pub const EPOCH_FENCE_NON_CLAIMS: &[&str] = &[
    "the supplied current epoch is not independently observed or authorized",
    "epoch equality does not acquire or renew a lease",
    "the decision does not provide storage atomicity or durability",
    "the decision does not provide consensus or membership",
    "the decision does not prove a runtime has exactly one writer",
    "one-entity isolation does not prove cross-entity transaction safety",
];

/// One caller-supplied monotonic ownership epoch.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EntityEpoch(u64);

impl EntityEpoch {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Typed ownership facts supplied for one entity mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpochFence {
    entity: NodeId,
    current_epoch: EntityEpoch,
    offered_epoch: EntityEpoch,
}

impl EpochFence {
    #[must_use]
    pub const fn new(
        entity: NodeId,
        current_epoch: EntityEpoch,
        offered_epoch: EntityEpoch,
    ) -> Self {
        Self {
            entity,
            current_epoch,
            offered_epoch,
        }
    }

    #[must_use]
    pub const fn entity(&self) -> &NodeId {
        &self.entity
    }

    #[must_use]
    pub const fn current_epoch(&self) -> EntityEpoch {
        self.current_epoch
    }

    #[must_use]
    pub const fn offered_epoch(&self) -> EntityEpoch {
        self.offered_epoch
    }
}

/// Exact reason why an epoch fence rejected a mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EpochFenceRejection {
    UnknownEntity,
    EntityMismatch,
    EpochMismatch,
    TerminalMutation,
}

/// Pure disposition of one epoch-fence decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EpochFenceDisposition {
    Admit,
    Reject(EpochFenceRejection),
}

/// Public, bounded result of one epoch-fence decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpochFenceDecision {
    pub entity: NodeId,
    pub current_epoch: EntityEpoch,
    pub offered_epoch: EntityEpoch,
    pub disposition: EpochFenceDisposition,
    pub implementation_identity: &'static str,
    pub contract_identity: &'static str,
    pub required_non_claims: &'static [&'static str],
}

/// Result of the proof-linked fence and the production machine transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpochFencedTransition {
    pub fence: EpochFenceDecision,
    /// The delegated production transition, or `None` when the fence rejects.
    pub transition: Option<Decision>,
}

const fn is_terminal(state: LocalState) -> bool {
    matches!(
        state,
        LocalState::Completed | LocalState::Failed | LocalState::Cancelled
    )
}

const fn disposition_from_code(code: u8) -> EpochFenceDisposition {
    match code {
        crate::verified::EPOCH_FENCE_ADMIT => EpochFenceDisposition::Admit,
        crate::verified::EPOCH_FENCE_UNKNOWN_ENTITY => {
            EpochFenceDisposition::Reject(EpochFenceRejection::UnknownEntity)
        }
        crate::verified::EPOCH_FENCE_ENTITY_MISMATCH => {
            EpochFenceDisposition::Reject(EpochFenceRejection::EntityMismatch)
        }
        crate::verified::EPOCH_FENCE_EPOCH_MISMATCH => {
            EpochFenceDisposition::Reject(EpochFenceRejection::EpochMismatch)
        }
        crate::verified::EPOCH_FENCE_TERMINAL_MUTATION => {
            EpochFenceDisposition::Reject(EpochFenceRejection::TerminalMutation)
        }
        _ => EpochFenceDisposition::Reject(EpochFenceRejection::UnknownEntity),
    }
}

fn decision_code(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
    fence: &EpochFence,
) -> u8 {
    let is_entity_known = machine.node_indices.contains_key(fence.entity());
    let is_entity_match = event.node_id == fence.entity().as_str();
    let is_entity_terminal = state.state(fence.entity()).is_some_and(is_terminal);
    crate::verified::epoch_fence_decision(
        is_entity_known,
        is_entity_match,
        fence.current_epoch().get(),
        fence.offered_epoch().get(),
        is_entity_terminal,
    )
}

/// Decides whether supplied ownership facts fence one mutation.
///
/// This function reads no clock, storage, process, network, or ambient state.
/// The caller remains responsible for the truth and freshness of
/// `current_epoch`.
#[must_use]
pub fn decide_epoch_fence(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
    fence: &EpochFence,
) -> EpochFenceDecision {
    EpochFenceDecision {
        entity: fence.entity().clone(),
        current_epoch: fence.current_epoch(),
        offered_epoch: fence.offered_epoch(),
        disposition: disposition_from_code(decision_code(machine, state, event, fence)),
        implementation_identity: EPOCH_FENCE_IMPLEMENTATION_IDENTITY,
        contract_identity: EPOCH_FENCE_CONTRACT_IDENTITY,
        required_non_claims: EPOCH_FENCE_NON_CLAIMS,
    }
}

/// Applies one epoch fence before calling the production machine transition.
///
/// Rejection emits no replacement state or effect intent. Admission delegates
/// to [`crate::step`], so this wrapper contains no second transition table.
#[must_use]
pub fn step_epoch_fenced(
    machine: &AdmittedMachine,
    state: &MachineState,
    event: &Event,
    fence: &EpochFence,
) -> EpochFencedTransition {
    let fence_decision = decide_epoch_fence(machine, state, event, fence);
    let transition = if fence_decision.disposition == EpochFenceDisposition::Admit {
        Some(crate::step(machine, state, event))
    } else {
        None
    };
    EpochFencedTransition {
        fence: fence_decision,
        transition,
    }
}
