// Canonical bundle admission, identity, and verification.
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::evidence::{ArtifactObservation, DIGEST_BYTES, decode_digest_hex};
use crate::profile::{REQUIRED_ROLE_COUNT, REQUIRED_ROLES, expected_schema, expected_status};
use crate::rejection::{Failure, IssueCode};

const BUNDLE_IDENTITY_CONTEXT: &str = "choregraph release bundle identity v1";
const LENGTH_FRAME_BYTES: usize = size_of::<u64>();

pub const RELEASE_BUNDLE_SCHEMA: &str = "choregraph.release-bundle.v1";
pub const RELEASE_PROFILE: &str = "choregraph.release-profile.v1";
pub const REQUIRED_NON_CLAIMS: &[&str] = &[
    "bundle linkage does not prove verifier soundness",
    "bundle linkage does not prove compiler correctness",
    "bundle linkage does not prove Nickel semantic equivalence",
    "bundle linkage does not prove event truth or authorization freshness",
    "bundle linkage does not prove runtime effect success",
    "bundle linkage does not prove remote availability or whole-product correctness",
];
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BundleIdentity([u8; DIGEST_BYTES]);

impl BundleIdentity {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
    #[inline(always)]
    pub const fn bytes(self) -> [u8; DIGEST_BYTES] {
        self.0
    }

    #[must_use]
    pub fn from_hex(value: &str) -> Option<Self> {
        decode_digest_hex(value).map(Self)
    }

    #[must_use]
    pub fn hex(self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bundle {
    pub schema: String,
    pub profile: String,
    pub artifacts: Vec<ArtifactObservation>,
    pub non_claims: Vec<String>,
    pub identity: BundleIdentity,
}
fn valid_path_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-')
}

fn valid_relative_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || !path.bytes().all(valid_path_byte) {
        return false;
    }
    path.split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

#[expect(
    clippy::manual_unwrap_or,
    reason = "explicit overflow handling keeps framed identity behavior visible"
)]
pub(crate) fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length_bytes = match u64::try_from(bytes.len()) {
        Ok(length_bytes) => length_bytes,
        Err(_) => u64::MAX,
    };
    hasher.update(&length_bytes.to_le_bytes()[..LENGTH_FRAME_BYTES]);
    hasher.update(bytes);
}

fn bundle_identity(artifacts: &[ArtifactObservation], non_claims: &[String]) -> BundleIdentity {
    let mut hasher = blake3::Hasher::new_derive_key(BUNDLE_IDENTITY_CONTEXT);
    frame(&mut hasher, RELEASE_BUNDLE_SCHEMA.as_bytes());
    frame(&mut hasher, RELEASE_PROFILE.as_bytes());
    for artifact in artifacts {
        frame(&mut hasher, artifact.role.label().as_bytes());
        frame(&mut hasher, artifact.relative_path.as_bytes());
        frame(&mut hasher, artifact.schema.as_bytes());
        frame(&mut hasher, artifact.status.label().as_bytes());
        frame(&mut hasher, &artifact.digest.bytes());
    }
    for non_claim in non_claims {
        frame(&mut hasher, non_claim.as_bytes());
    }
    BundleIdentity::from_bytes(*hasher.finalize().as_bytes())
}

pub fn build_bundle(mut artifacts: Vec<ArtifactObservation>) -> Result<Bundle, Failure> {
    if artifacts.len() != REQUIRED_ROLE_COUNT {
        return Err(Failure::one(IssueCode::ArtifactCountMismatch, None));
    }
    artifacts.sort_by_key(|artifact| artifact.role);
    for (index, required_role) in REQUIRED_ROLES.iter().enumerate() {
        let Some(artifact) = artifacts.get(index) else {
            return Err(Failure::one(IssueCode::MissingRole, Some(*required_role)));
        };
        if artifact.role != *required_role {
            let code = if artifacts
                .iter()
                .filter(|candidate| candidate.role == artifact.role)
                .count()
                > 1
            {
                IssueCode::DuplicateRole
            } else {
                IssueCode::MissingRole
            };
            return Err(Failure::one(code, Some(*required_role)));
        }
        if !valid_relative_path(&artifact.relative_path) {
            return Err(Failure::one(
                IssueCode::InvalidRelativePath,
                Some(artifact.role),
            ));
        }
        if artifact.schema != expected_schema(artifact.role) {
            return Err(Failure::one(
                IssueCode::UnsupportedSchema,
                Some(artifact.role),
            ));
        }
        if artifact.status != expected_status(artifact.role) {
            return Err(Failure::one(IssueCode::InvalidStatus, Some(artifact.role)));
        }
    }
    let non_claims = REQUIRED_NON_CLAIMS
        .iter()
        .map(|value| String::from(*value))
        .collect::<Vec<_>>();
    let identity = bundle_identity(&artifacts, &non_claims);
    debug_assert_eq!(artifacts.len(), REQUIRED_ROLE_COUNT);
    debug_assert!(
        artifacts
            .iter()
            .zip(REQUIRED_ROLES)
            .all(|(artifact, required)| artifact.role == required)
    );
    Ok(Bundle {
        schema: String::from(RELEASE_BUNDLE_SCHEMA),
        profile: String::from(RELEASE_PROFILE),
        artifacts,
        non_claims,
        identity,
    })
}

pub fn verify_bundle(
    bundle: &Bundle,
    current_artifacts: Vec<ArtifactObservation>,
) -> Result<(), Failure> {
    if bundle.schema != RELEASE_BUNDLE_SCHEMA {
        return Err(Failure::one(IssueCode::BundleSchemaMismatch, None));
    }
    if bundle.profile != RELEASE_PROFILE {
        return Err(Failure::one(IssueCode::BundleProfileMismatch, None));
    }
    let expected = build_bundle(current_artifacts)?;
    if bundle.non_claims != expected.non_claims {
        return Err(Failure::one(IssueCode::NonClaimMismatch, None));
    }
    if bundle.artifacts != expected.artifacts {
        return Err(Failure::one(IssueCode::ArtifactMismatch, None));
    }
    if bundle.identity != expected.identity {
        return Err(Failure::one(IssueCode::BundleIdentityMismatch, None));
    }
    debug_assert_eq!(bundle.artifacts.len(), REQUIRED_ROLE_COUNT);
    debug_assert_eq!(
        bundle.identity,
        bundle_identity(&bundle.artifacts, &bundle.non_claims)
    );
    Ok(())
}
