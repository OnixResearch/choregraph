#![no_std]

use choregraph_core::Capability;

/// A claim that the Kamacite adapter can receive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterchangeClaim {
    CanonicalIdentityBinding,
    BehavioralProof,
    RuntimeAuthorization,
}

/// Returns the core capability that canonical artifacts can describe.
#[must_use]
pub const fn described_core_capability() -> Capability {
    Capability::ImmutableMachineData
}

/// Returns true only for identity linkage owned by Kamacite.
#[must_use]
pub const fn supports(claim: InterchangeClaim) -> bool {
    matches!(claim, InterchangeClaim::CanonicalIdentityBinding)
}

#[cfg(test)]
mod tests {
    use super::{InterchangeClaim, supports};

    #[test]
    fn supports_canonical_identity_binding() {
        assert!(supports(InterchangeClaim::CanonicalIdentityBinding));
    }

    #[test]
    fn rejects_behavioral_and_authority_claims() {
        assert!(!supports(InterchangeClaim::BehavioralProof));
        assert!(!supports(InterchangeClaim::RuntimeAuthorization));
    }
}
