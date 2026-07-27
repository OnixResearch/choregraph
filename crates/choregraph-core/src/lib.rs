#![no_std]

extern crate alloc;

use alloc::vec::Vec;

/// A capability that can occur at a Choregraph trust boundary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    ImmutableMachineData,
    DeterministicDecision,
    FileSystem,
    Environment,
    Clock,
    Process,
    Network,
    NickelEvaluation,
    TokenVerification,
    Persistence,
    ReceiptIssuance,
    EffectExecution,
}

/// The only production implementation policy accepted by Choregraph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionImplementationPolicy {
    ChoregraphCoreOnly,
}

/// The foundation claim does not include machine or proof correctness.
pub const FOUNDATION_CLAIM: &str = "repository-boundary-only-unverified";

/// The transition implementation policy for every compatibility adapter.
pub const TRANSITION_IMPLEMENTATION_POLICY: TransitionImplementationPolicy =
    TransitionImplementationPolicy::ChoregraphCoreOnly;

const EXCLUDED_CAPABILITIES: &[Capability] = &[
    Capability::FileSystem,
    Capability::Environment,
    Capability::Clock,
    Capability::Process,
    Capability::Network,
    Capability::NickelEvaluation,
    Capability::TokenVerification,
    Capability::Persistence,
    Capability::ReceiptIssuance,
    Capability::EffectExecution,
];

/// Returns true only for capabilities owned by the future pure machine core.
#[must_use]
pub const fn permits(capability: Capability) -> bool {
    matches!(
        capability,
        Capability::ImmutableMachineData | Capability::DeterministicDecision
    )
}

/// Returns the explicit authority that remains outside the core.
#[must_use]
pub fn excluded_capabilities() -> Vec<Capability> {
    EXCLUDED_CAPABILITIES.to_vec()
}

#[cfg(test)]
mod tests {
    use super::{Capability, excluded_capabilities, permits};

    #[test]
    fn permits_pure_machine_capabilities() {
        assert!(permits(Capability::ImmutableMachineData));
        assert!(permits(Capability::DeterministicDecision));
    }

    #[test]
    fn rejects_every_excluded_authority() {
        for capability in excluded_capabilities() {
            assert!(!permits(capability));
        }
    }
}
