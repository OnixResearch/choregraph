#![no_std]

use choregraph_core::Capability;

/// A possible implementation mode for the Seaglass compatibility surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityMode {
    DelegateToChoregraphCore,
    IndependentTransitionImplementation,
}

/// Returns the core capability that the future adapter delegates to.
#[must_use]
pub const fn delegated_capability() -> Capability {
    Capability::DeterministicDecision
}

/// Returns true only for a conversion-only delegate.
#[must_use]
pub const fn accepts(mode: CompatibilityMode) -> bool {
    matches!(mode, CompatibilityMode::DelegateToChoregraphCore)
}

#[cfg(test)]
mod tests {
    use super::{CompatibilityMode, accepts};

    #[test]
    fn accepts_delegation_to_core() {
        assert!(accepts(CompatibilityMode::DelegateToChoregraphCore));
    }

    #[test]
    fn rejects_an_independent_transition_core() {
        assert!(!accepts(
            CompatibilityMode::IndependentTransitionImplementation
        ));
    }
}
