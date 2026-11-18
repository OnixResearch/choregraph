// Exact local publication candidate, without publication authority.
const CANDIDATE_IDENTITY_CONTEXT: &str = "choregraph publication candidate identity v1";
const GIT_REVISION_HEX_BYTES: usize = 40;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::evidence::{ArtifactDigest, DIGEST_BYTES};
use crate::linkage::{BundleIdentity, frame};
use crate::rejection::{Failure, IssueCode};
pub const PUBLICATION_CANDIDATE_SCHEMA: &str = "choregraph.publication-candidate.v1";
pub const PROJECT_LICENSE: &str = "AGPL-3.0-or-later";
pub const ACQUISITION_ADAPTER: &str = "radicle-rid-exact-revision-v1";
pub const REQUIRED_PACKAGES: &[&str] = &[
    "choregraph-cli",
    "choregraph-core",
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-release",
    "choregraph-seaglass",
];
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CandidateIdentity([u8; DIGEST_BYTES]);

impl CandidateIdentity {
    #[must_use]
    pub const fn bytes(self) -> [u8; DIGEST_BYTES] {
        self.0
    }

    #[must_use]
    pub fn hex(self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationCandidateInput {
    pub source_revision: String,
    pub source_archive: ArtifactDigest,
    pub license: String,
    pub packages: Vec<String>,
    pub acquisition_adapter: String,
    pub bundle: BundleIdentity,
    pub rollback_revision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationCandidate {
    pub schema: String,
    pub source_revision: String,
    pub source_archive: ArtifactDigest,
    pub license: String,
    pub packages: Vec<String>,
    pub acquisition_adapter: String,
    pub bundle: BundleIdentity,
    pub rollback_revision: String,
    pub identity: CandidateIdentity,
}

const fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn valid_revision(revision: &str) -> bool {
    revision.len() == GIT_REVISION_HEX_BYTES && revision.bytes().all(is_lower_hex)
}

fn candidate_identity(input: &PublicationCandidateInput) -> CandidateIdentity {
    let mut hasher = blake3::Hasher::new_derive_key(CANDIDATE_IDENTITY_CONTEXT);
    frame(&mut hasher, PUBLICATION_CANDIDATE_SCHEMA.as_bytes());
    frame(&mut hasher, input.source_revision.as_bytes());
    frame(&mut hasher, &input.source_archive.bytes());
    frame(&mut hasher, input.license.as_bytes());
    for package in &input.packages {
        frame(&mut hasher, package.as_bytes());
    }
    frame(&mut hasher, input.acquisition_adapter.as_bytes());
    frame(&mut hasher, &input.bundle.bytes());
    frame(&mut hasher, input.rollback_revision.as_bytes());
    CandidateIdentity(*hasher.finalize().as_bytes())
}

pub fn build_publication_candidate(
    mut input: PublicationCandidateInput,
) -> Result<PublicationCandidate, Failure> {
    if !valid_revision(&input.source_revision) {
        return Err(Failure::one(IssueCode::InvalidSourceRevision, None));
    }
    if !valid_revision(&input.rollback_revision) {
        return Err(Failure::one(IssueCode::InvalidRollbackRevision, None));
    }
    if input.source_archive.bytes() == [0; DIGEST_BYTES] {
        return Err(Failure::one(IssueCode::InvalidArchiveIdentity, None));
    }
    if input.license != PROJECT_LICENSE {
        return Err(Failure::one(IssueCode::LicenseMismatch, None));
    }
    input.packages.sort();
    let required_packages = REQUIRED_PACKAGES
        .iter()
        .map(|package| String::from(*package))
        .collect::<Vec<_>>();
    if input.packages != required_packages {
        return Err(Failure::one(IssueCode::PackageSetMismatch, None));
    }
    if input.acquisition_adapter != ACQUISITION_ADAPTER {
        return Err(Failure::one(IssueCode::AcquisitionAdapterMismatch, None));
    }
    let identity = candidate_identity(&input);
    debug_assert!(valid_revision(&input.source_revision));
    debug_assert_eq!(input.packages.len(), REQUIRED_PACKAGES.len());
    Ok(PublicationCandidate {
        schema: String::from(PUBLICATION_CANDIDATE_SCHEMA),
        source_revision: input.source_revision,
        source_archive: input.source_archive,
        license: input.license,
        packages: input.packages,
        acquisition_adapter: input.acquisition_adapter,
        bundle: input.bundle,
        rollback_revision: input.rollback_revision,
        identity,
    })
}

pub fn verify_publication_candidate(
    candidate: &PublicationCandidate,
    expected_input: PublicationCandidateInput,
) -> Result<(), Failure> {
    let expected = build_publication_candidate(expected_input)?;
    if candidate != &expected {
        return Err(Failure::one(IssueCode::CandidateMismatch, None));
    }
    Ok(())
}
