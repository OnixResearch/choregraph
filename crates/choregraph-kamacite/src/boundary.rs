pub const CLAIM_BOUNDARY: &str = "canonical encoding and exact identity linkage; not behavioral proof, authority, or effect success";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterchangeClaim {
    CanonicalIdentityBinding,
    BehavioralProof,
    RuntimeAuthority,
}

#[must_use]
pub const fn supports(claim: InterchangeClaim) -> bool {
    matches!(claim, InterchangeClaim::CanonicalIdentityBinding)
}
