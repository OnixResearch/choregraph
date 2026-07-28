#![no_std]
#![allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    reason = "the closed release profile keeps exhaustive role mappings and explicit alloc value imports in one reviewable file"
)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub const RELEASE_BUNDLE_SCHEMA: &str = "choregraph.release-bundle.v1";
pub const RELEASE_PROFILE: &str = "choregraph.release-profile.v1";
pub const VALENCE_PROJECTION_SCHEMA: &str = "choregraph.valence-release-projection.v1";
pub const PUBLICATION_CANDIDATE_SCHEMA: &str = "choregraph.publication-candidate.v1";
pub const PROJECT_LICENSE: &str = "AGPL-3.0-or-later";
pub const ACQUISITION_ADAPTER: &str = "radicle-rid-exact-revision-v1";
const BUNDLE_IDENTITY_CONTEXT: &str = "choregraph release bundle identity v1";
const CANDIDATE_IDENTITY_CONTEXT: &str = "choregraph publication candidate identity v1";
const LENGTH_FRAME_BYTES: usize = size_of::<u64>();
const GIT_REVISION_HEX_BYTES: usize = 40;
const DIGEST_BYTES: usize = 32;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
const DIGEST_HEX_BYTES: usize = 64;
const NIBBLE_SHIFT_BITS: u32 = 4;
const HEX_ALPHA_OFFSET: u8 = 10;
const REQUIRED_ROLE_COUNT: usize = 36;

pub const REQUIRED_NON_CLAIMS: &[&str] = &[
    "bundle linkage does not prove verifier soundness",
    "bundle linkage does not prove compiler correctness",
    "bundle linkage does not prove Nickel semantic equivalence",
    "bundle linkage does not prove event truth or authorization freshness",
    "bundle linkage does not prove runtime effect success",
    "bundle linkage does not prove remote availability or whole-product correctness",
];

pub const REQUIRED_PACKAGES: &[&str] = &[
    "choregraph-cli",
    "choregraph-core",
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-release",
    "choregraph-seaglass",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReleaseRole {
    SourceTree,
    License,
    MachineSchema,
    ConcreteMachineSchema,
    ConcreteIntentSchema,
    LinkReceiptSchema,
    CoreSource,
    CoreLibrary,
    SingleStepProofSource,
    SingleStepProofResult,
    TraceProofSource,
    TraceProofResult,
    TrustedSurface,
    RustToolchain,
    VerusToolchain,
    SolverToolchain,
    NickelToolchain,
    NixInputs,
    DependencyLock,
    KamaciteMachine,
    KamaciteConcreteMachine,
    KamaciteConcreteIntent,
    KamaciteConcreteIntentReceipt,
    KamaciteLifecycle,
    KamaciteTraceReceipt,
    KamaciteExhaustivenessReceipt,
    KamaciteSourceReceipt,
    KamaciteProofReferenceReceipt,
    PositiveFixtureCorpus,
    NegativeFixtureCorpus,
    ExecutionGraphParity,
    LatticeDownstream,
    LatticeConcreteHandoff,
    CairnReceipt,
    OctetReceipt,
    NixReceipt,
}

pub const REQUIRED_ROLES: [ReleaseRole; REQUIRED_ROLE_COUNT] = [
    ReleaseRole::SourceTree,
    ReleaseRole::License,
    ReleaseRole::MachineSchema,
    ReleaseRole::ConcreteMachineSchema,
    ReleaseRole::ConcreteIntentSchema,
    ReleaseRole::LinkReceiptSchema,
    ReleaseRole::CoreSource,
    ReleaseRole::CoreLibrary,
    ReleaseRole::SingleStepProofSource,
    ReleaseRole::SingleStepProofResult,
    ReleaseRole::TraceProofSource,
    ReleaseRole::TraceProofResult,
    ReleaseRole::TrustedSurface,
    ReleaseRole::RustToolchain,
    ReleaseRole::VerusToolchain,
    ReleaseRole::SolverToolchain,
    ReleaseRole::NickelToolchain,
    ReleaseRole::NixInputs,
    ReleaseRole::DependencyLock,
    ReleaseRole::KamaciteMachine,
    ReleaseRole::KamaciteConcreteMachine,
    ReleaseRole::KamaciteConcreteIntent,
    ReleaseRole::KamaciteConcreteIntentReceipt,
    ReleaseRole::KamaciteLifecycle,
    ReleaseRole::KamaciteTraceReceipt,
    ReleaseRole::KamaciteExhaustivenessReceipt,
    ReleaseRole::KamaciteSourceReceipt,
    ReleaseRole::KamaciteProofReferenceReceipt,
    ReleaseRole::PositiveFixtureCorpus,
    ReleaseRole::NegativeFixtureCorpus,
    ReleaseRole::ExecutionGraphParity,
    ReleaseRole::LatticeDownstream,
    ReleaseRole::LatticeConcreteHandoff,
    ReleaseRole::CairnReceipt,
    ReleaseRole::OctetReceipt,
    ReleaseRole::NixReceipt,
];

impl ReleaseRole {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceTree => "source-tree",
            Self::License => "license",
            Self::MachineSchema => "machine-schema",
            Self::ConcreteMachineSchema => "concrete-machine-schema",
            Self::ConcreteIntentSchema => "concrete-intent-schema",
            Self::LinkReceiptSchema => "link-receipt-schema",
            Self::CoreSource => "core-source",
            Self::CoreLibrary => "core-library",
            Self::SingleStepProofSource => "single-step-proof-source",
            Self::SingleStepProofResult => "single-step-proof-result",
            Self::TraceProofSource => "trace-proof-source",
            Self::TraceProofResult => "trace-proof-result",
            Self::TrustedSurface => "trusted-surface",
            Self::RustToolchain => "rust-toolchain",
            Self::VerusToolchain => "verus-toolchain",
            Self::SolverToolchain => "solver-toolchain",
            Self::NickelToolchain => "nickel-toolchain",
            Self::NixInputs => "nix-inputs",
            Self::DependencyLock => "dependency-lock",
            Self::KamaciteMachine => "kamacite-machine",
            Self::KamaciteConcreteMachine => "kamacite-concrete-machine",
            Self::KamaciteConcreteIntent => "kamacite-concrete-intent",
            Self::KamaciteConcreteIntentReceipt => "kamacite-concrete-intent-receipt",
            Self::KamaciteLifecycle => "kamacite-lifecycle",
            Self::KamaciteTraceReceipt => "kamacite-trace-receipt",
            Self::KamaciteExhaustivenessReceipt => "kamacite-exhaustiveness-receipt",
            Self::KamaciteSourceReceipt => "kamacite-source-receipt",
            Self::KamaciteProofReferenceReceipt => "kamacite-proof-reference-receipt",
            Self::PositiveFixtureCorpus => "positive-fixture-corpus",
            Self::NegativeFixtureCorpus => "negative-fixture-corpus",
            Self::ExecutionGraphParity => "execution-graph-parity",
            Self::LatticeDownstream => "lattice-downstream",
            Self::LatticeConcreteHandoff => "lattice-concrete-handoff",
            Self::CairnReceipt => "cairn-receipt",
            Self::OctetReceipt => "octet-receipt",
            Self::NixReceipt => "nix-receipt",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArtifactStatus {
    Observed,
    Canonical,
    Verified,
    Passed,
}

impl ArtifactStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::Canonical => "canonical",
            Self::Verified => "verified",
            Self::Passed => "passed",
        }
    }
}

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|value| value.checked_add(HEX_ALPHA_OFFSET)),
        _ => None,
    }
}

fn decode_digest_hex(value: &str) -> Option<[u8; DIGEST_BYTES]> {
    if value.len() != DIGEST_HEX_BYTES {
        return None;
    }
    let mut decoded = [0u8; DIGEST_BYTES];
    for (index, pair) in value
        .as_bytes()
        .chunks_exact(HEX_CHARACTERS_PER_BYTE)
        .enumerate()
    {
        let high = decode_hex_nibble(pair[0])?;
        let low = decode_hex_nibble(pair[1])?;
        decoded[index] = high.checked_shl(NIBBLE_SHIFT_BITS)? | low;
    }
    Some(decoded)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactDigest([u8; DIGEST_BYTES]);

impl ArtifactDigest {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BundleIdentity([u8; DIGEST_BYTES]);

impl BundleIdentity {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    #[must_use]
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactObservation {
    pub role: ReleaseRole,
    pub relative_path: String,
    pub schema: String,
    pub status: ArtifactStatus,
    pub digest: ArtifactDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseBundle {
    pub schema: String,
    pub profile: String,
    pub artifacts: Vec<ArtifactObservation>,
    pub non_claims: Vec<String>,
    pub identity: BundleIdentity,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReleaseIssueCode {
    ArtifactCountMismatch,
    MissingRole,
    DuplicateRole,
    InvalidRelativePath,
    UnsupportedSchema,
    InvalidStatus,
    BundleSchemaMismatch,
    BundleProfileMismatch,
    NonClaimMismatch,
    ArtifactMismatch,
    BundleIdentityMismatch,
    InvalidSourceRevision,
    InvalidRollbackRevision,
    InvalidArchiveIdentity,
    LicenseMismatch,
    PackageSetMismatch,
    AcquisitionAdapterMismatch,
    CandidateMismatch,
    ProjectionMismatch,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReleaseIssue {
    pub code: ReleaseIssueCode,
    pub role: Option<ReleaseRole>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseFailure {
    pub issues: Vec<ReleaseIssue>,
}

impl ReleaseFailure {
    fn one(code: ReleaseIssueCode, role: Option<ReleaseRole>) -> Self {
        Self {
            issues: vec![ReleaseIssue { code, role }],
        }
    }
}

#[must_use]
pub const fn expected_status(role: ReleaseRole) -> ArtifactStatus {
    match role {
        ReleaseRole::MachineSchema
        | ReleaseRole::ConcreteMachineSchema
        | ReleaseRole::ConcreteIntentSchema
        | ReleaseRole::LinkReceiptSchema
        | ReleaseRole::KamaciteMachine
        | ReleaseRole::KamaciteConcreteMachine
        | ReleaseRole::KamaciteConcreteIntent
        | ReleaseRole::KamaciteConcreteIntentReceipt
        | ReleaseRole::KamaciteLifecycle
        | ReleaseRole::KamaciteTraceReceipt
        | ReleaseRole::KamaciteExhaustivenessReceipt
        | ReleaseRole::KamaciteSourceReceipt
        | ReleaseRole::KamaciteProofReferenceReceipt => ArtifactStatus::Canonical,
        ReleaseRole::SingleStepProofResult | ReleaseRole::TraceProofResult => {
            ArtifactStatus::Verified
        }
        ReleaseRole::ExecutionGraphParity
        | ReleaseRole::LatticeDownstream
        | ReleaseRole::CairnReceipt
        | ReleaseRole::OctetReceipt
        | ReleaseRole::NixReceipt => ArtifactStatus::Passed,
        ReleaseRole::SourceTree
        | ReleaseRole::License
        | ReleaseRole::CoreSource
        | ReleaseRole::CoreLibrary
        | ReleaseRole::SingleStepProofSource
        | ReleaseRole::TraceProofSource
        | ReleaseRole::TrustedSurface
        | ReleaseRole::RustToolchain
        | ReleaseRole::VerusToolchain
        | ReleaseRole::SolverToolchain
        | ReleaseRole::NickelToolchain
        | ReleaseRole::NixInputs
        | ReleaseRole::DependencyLock
        | ReleaseRole::PositiveFixtureCorpus
        | ReleaseRole::NegativeFixtureCorpus
        | ReleaseRole::LatticeConcreteHandoff => ArtifactStatus::Observed,
    }
}

#[must_use]
pub const fn expected_schema(role: ReleaseRole) -> &'static str {
    match role {
        ReleaseRole::SourceTree => "choregraph.source-baseline.v1",
        ReleaseRole::License => "spdx.license-text.v1",
        ReleaseRole::MachineSchema => "choregraph.machine-schema.v1",
        ReleaseRole::ConcreteMachineSchema => "choregraph.machine-schema.v2",
        ReleaseRole::ConcreteIntentSchema => "choregraph.concrete-intent-schema.v2",
        ReleaseRole::LinkReceiptSchema => "choregraph.link-receipt-schema.v1",
        ReleaseRole::CoreSource => "rust.source.v1",
        ReleaseRole::CoreLibrary => "rust.library-artifact.v1",
        ReleaseRole::SingleStepProofSource => "verus.single-step-source.v1",
        ReleaseRole::SingleStepProofResult => "choregraph.single-step-proof.v1",
        ReleaseRole::TraceProofSource => "verus.trace-replay-source.v1",
        ReleaseRole::TraceProofResult => "choregraph.trace-replay-proof.v1",
        ReleaseRole::TrustedSurface => "choregraph.trusted-surface.v1",
        ReleaseRole::RustToolchain => "rust-toolchain.toml.v1",
        ReleaseRole::VerusToolchain => "octet.verus-toolchain.v1",
        ReleaseRole::SolverToolchain => "octet.solver-toolchain.v1",
        ReleaseRole::NickelToolchain => "nickel.toolchain.v1",
        ReleaseRole::NixInputs => "nix.flake-inputs.v1",
        ReleaseRole::DependencyLock => "cargo.lock.v1",
        ReleaseRole::KamaciteMachine => "choregraph.machine.v1",
        ReleaseRole::KamaciteConcreteMachine => "choregraph.machine.v2",
        ReleaseRole::KamaciteConcreteIntent => "choregraph.concrete-intent.v2",
        ReleaseRole::KamaciteConcreteIntentReceipt => "choregraph.concrete-intent-receipt.v1",
        ReleaseRole::KamaciteLifecycle => "choregraph.lifecycle-fsm.v1",
        ReleaseRole::KamaciteTraceReceipt => "choregraph.fsm-trace-receipt.v1",
        ReleaseRole::KamaciteExhaustivenessReceipt => "choregraph.fsm-exhaustiveness-receipt.v1",
        ReleaseRole::KamaciteSourceReceipt => "choregraph.nickel-source-receipt.v1",
        ReleaseRole::KamaciteProofReferenceReceipt => "choregraph.proof-reference-receipt.v1",
        ReleaseRole::PositiveFixtureCorpus => "choregraph.positive-fixture-corpus.v1",
        ReleaseRole::NegativeFixtureCorpus => "choregraph.negative-fixture-corpus.v1",
        ReleaseRole::ExecutionGraphParity => "choregraph.execution-graph-parity.v1",
        ReleaseRole::LatticeDownstream => "choregraph.lattice-downstream.v1",
        ReleaseRole::LatticeConcreteHandoff => "choregraph.lattice-concrete-handoff.v1",
        ReleaseRole::CairnReceipt => "cairn.validation-receipt.v1",
        ReleaseRole::OctetReceipt => "octet.validation-receipt.v1",
        ReleaseRole::NixReceipt => "nix.flake-check-receipt.v1",
    }
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
fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
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

pub fn build_release_bundle(
    mut artifacts: Vec<ArtifactObservation>,
) -> Result<ReleaseBundle, ReleaseFailure> {
    if artifacts.len() != REQUIRED_ROLE_COUNT {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::ArtifactCountMismatch,
            None,
        ));
    }
    artifacts.sort_by_key(|artifact| artifact.role);
    for (index, required_role) in REQUIRED_ROLES.iter().enumerate() {
        let Some(artifact) = artifacts.get(index) else {
            return Err(ReleaseFailure::one(
                ReleaseIssueCode::MissingRole,
                Some(*required_role),
            ));
        };
        if artifact.role != *required_role {
            let code = if artifacts
                .iter()
                .filter(|candidate| candidate.role == artifact.role)
                .count()
                > 1
            {
                ReleaseIssueCode::DuplicateRole
            } else {
                ReleaseIssueCode::MissingRole
            };
            return Err(ReleaseFailure::one(code, Some(*required_role)));
        }
        if !valid_relative_path(&artifact.relative_path) {
            return Err(ReleaseFailure::one(
                ReleaseIssueCode::InvalidRelativePath,
                Some(artifact.role),
            ));
        }
        if artifact.schema != expected_schema(artifact.role) {
            return Err(ReleaseFailure::one(
                ReleaseIssueCode::UnsupportedSchema,
                Some(artifact.role),
            ));
        }
        if artifact.status != expected_status(artifact.role) {
            return Err(ReleaseFailure::one(
                ReleaseIssueCode::InvalidStatus,
                Some(artifact.role),
            ));
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
    Ok(ReleaseBundle {
        schema: String::from(RELEASE_BUNDLE_SCHEMA),
        profile: String::from(RELEASE_PROFILE),
        artifacts,
        non_claims,
        identity,
    })
}

pub fn verify_release_bundle(
    bundle: &ReleaseBundle,
    current_artifacts: Vec<ArtifactObservation>,
) -> Result<(), ReleaseFailure> {
    if bundle.schema != RELEASE_BUNDLE_SCHEMA {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::BundleSchemaMismatch,
            None,
        ));
    }
    if bundle.profile != RELEASE_PROFILE {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::BundleProfileMismatch,
            None,
        ));
    }
    let expected = build_release_bundle(current_artifacts)?;
    if bundle.non_claims != expected.non_claims {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::NonClaimMismatch,
            None,
        ));
    }
    if bundle.artifacts != expected.artifacts {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::ArtifactMismatch,
            None,
        ));
    }
    if bundle.identity != expected.identity {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::BundleIdentityMismatch,
            None,
        ));
    }
    debug_assert_eq!(bundle.artifacts.len(), REQUIRED_ROLE_COUNT);
    debug_assert_eq!(
        bundle.identity,
        bundle_identity(&bundle.artifacts, &bundle.non_claims)
    );
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValenceProjectionEntry {
    pub role: String,
    pub artifact_blake3: String,
    pub verification_role: String,
    pub claim: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValenceProjection {
    pub schema: String,
    pub bundle_blake3: String,
    pub entries: Vec<ValenceProjectionEntry>,
    pub non_claims: Vec<String>,
}

const fn projection_role(role: ReleaseRole) -> &'static str {
    match role {
        ReleaseRole::SingleStepProofResult | ReleaseRole::TraceProofResult => {
            "presented-proof-result"
        }
        ReleaseRole::MachineSchema
        | ReleaseRole::ConcreteMachineSchema
        | ReleaseRole::ConcreteIntentSchema
        | ReleaseRole::LinkReceiptSchema
        | ReleaseRole::KamaciteMachine
        | ReleaseRole::KamaciteConcreteMachine
        | ReleaseRole::KamaciteConcreteIntent
        | ReleaseRole::KamaciteConcreteIntentReceipt
        | ReleaseRole::KamaciteLifecycle
        | ReleaseRole::KamaciteTraceReceipt
        | ReleaseRole::KamaciteExhaustivenessReceipt
        | ReleaseRole::KamaciteSourceReceipt
        | ReleaseRole::KamaciteProofReferenceReceipt => "canonical-artifact",
        ReleaseRole::ExecutionGraphParity
        | ReleaseRole::LatticeDownstream
        | ReleaseRole::LatticeConcreteHandoff => "conformance-observation",
        ReleaseRole::CairnReceipt | ReleaseRole::OctetReceipt | ReleaseRole::NixReceipt => {
            "gate-observation"
        }
        _ => "identity-observation",
    }
}

pub fn project_valence(bundle: &ReleaseBundle) -> ValenceProjection {
    let entries = bundle
        .artifacts
        .iter()
        .map(|artifact| ValenceProjectionEntry {
            role: artifact.role.label().to_string(),
            artifact_blake3: artifact.digest.hex(),
            verification_role: projection_role(artifact.role).to_string(),
            claim: String::from("linkage-only-no-claim-promotion"),
        })
        .collect();
    ValenceProjection {
        schema: String::from(VALENCE_PROJECTION_SCHEMA),
        bundle_blake3: bundle.identity.hex(),
        entries,
        non_claims: bundle.non_claims.clone(),
    }
}

pub fn verify_valence_projection(
    bundle: &ReleaseBundle,
    projection: &ValenceProjection,
) -> Result<(), ReleaseFailure> {
    if projection != &project_valence(bundle) {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::ProjectionMismatch,
            None,
        ));
    }
    Ok(())
}

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
) -> Result<PublicationCandidate, ReleaseFailure> {
    if !valid_revision(&input.source_revision) {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::InvalidSourceRevision,
            None,
        ));
    }
    if !valid_revision(&input.rollback_revision) {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::InvalidRollbackRevision,
            None,
        ));
    }
    if input.source_archive.bytes() == [0; DIGEST_BYTES] {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::InvalidArchiveIdentity,
            None,
        ));
    }
    if input.license != PROJECT_LICENSE {
        return Err(ReleaseFailure::one(ReleaseIssueCode::LicenseMismatch, None));
    }
    input.packages.sort();
    let required_packages = REQUIRED_PACKAGES
        .iter()
        .map(|package| String::from(*package))
        .collect::<Vec<_>>();
    if input.packages != required_packages {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::PackageSetMismatch,
            None,
        ));
    }
    if input.acquisition_adapter != ACQUISITION_ADAPTER {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::AcquisitionAdapterMismatch,
            None,
        ));
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
) -> Result<(), ReleaseFailure> {
    let expected = build_publication_candidate(expected_input)?;
    if candidate != &expected {
        return Err(ReleaseFailure::one(
            ReleaseIssueCode::CandidateMismatch,
            None,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::*;

    const BASE_DIGEST_BYTE: u8 = 0x40;
    const SOURCE_ARCHIVE_BYTE: u8 = 0x55;
    const BUNDLE_BYTE: u8 = 0x66;
    const SOURCE_REVISION: &str = "1111111111111111111111111111111111111111";
    const ROLLBACK_REVISION: &str = "2222222222222222222222222222222222222222";

    #[allow(
        unknown_lints,
        no_panic,
        reason = "positive release fixtures must stop when complete reviewed input is rejected"
    )]
    fn must<T>(result: Result<T, ReleaseFailure>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected success, got {error:?}"),
        }
    }

    #[allow(
        unknown_lints,
        no_panic,
        reason = "negative release fixtures must stop when malformed input is accepted"
    )]
    fn issue_code<T>(result: Result<T, ReleaseFailure>) -> ReleaseIssueCode {
        match result {
            Err(error) => match error.issues.first() {
                Some(issue) => issue.code,
                None => panic!("release failure had no issue"),
            },
            Ok(_) => panic!("expected release failure"),
        }
    }

    #[expect(
        clippy::manual_unwrap_or,
        reason = "explicit bounded test-index conversion keeps fixture bytes reviewable"
    )]
    fn fixture_index_byte(index: usize) -> u8 {
        match u8::try_from(index) {
            Ok(index_byte) => index_byte,
            Err(_) => u8::MAX,
        }
    }

    fn complete_artifacts() -> Vec<ArtifactObservation> {
        REQUIRED_ROLES
            .iter()
            .enumerate()
            .map(|(index, role)| {
                let index_byte = fixture_index_byte(index);
                ArtifactObservation {
                    role: *role,
                    relative_path: format!("release/{}.artifact", role.label()),
                    schema: String::from(expected_schema(*role)),
                    status: expected_status(*role),
                    digest: ArtifactDigest::from_bytes(
                        [BASE_DIGEST_BYTE.saturating_add(index_byte); DIGEST_BYTES],
                    ),
                }
            })
            .collect()
    }

    fn candidate_input(bundle: BundleIdentity) -> PublicationCandidateInput {
        PublicationCandidateInput {
            source_revision: String::from(SOURCE_REVISION),
            source_archive: ArtifactDigest::from_bytes([SOURCE_ARCHIVE_BYTE; DIGEST_BYTES]),
            license: String::from(PROJECT_LICENSE),
            packages: REQUIRED_PACKAGES
                .iter()
                .rev()
                .map(|package| String::from(*package))
                .collect(),
            acquisition_adapter: String::from(ACQUISITION_ADAPTER),
            bundle,
            rollback_revision: String::from(ROLLBACK_REVISION),
        }
    }

    #[test]
    fn digest_hex_parsing_is_exact_and_lowercase() {
        let digest = ArtifactDigest::from_bytes([SOURCE_ARCHIVE_BYTE; DIGEST_BYTES]);
        let identity = BundleIdentity::from_bytes([BUNDLE_BYTE; DIGEST_BYTES]);
        assert_eq!(ArtifactDigest::from_hex(&digest.hex()), Some(digest));
        assert_eq!(BundleIdentity::from_hex(&identity.hex()), Some(identity));
        assert_eq!(ArtifactDigest::from_hex("short"), None);
        assert_eq!(
            BundleIdentity::from_hex(&"A".repeat(DIGEST_HEX_BYTES)),
            None
        );
    }

    #[test]
    fn complete_bundles_are_deterministic_and_verify() {
        let artifacts = complete_artifacts();
        let first = must(build_release_bundle(artifacts.clone()));
        let mut reversed = artifacts.clone();
        reversed.reverse();
        let second = must(build_release_bundle(reversed));
        assert_eq!(first, second);
        assert_eq!(first.artifacts.len(), REQUIRED_ROLE_COUNT);
        assert_eq!(first.non_claims.len(), REQUIRED_NON_CLAIMS.len());
        assert!(verify_release_bundle(&first, artifacts).is_ok());
    }

    #[test]
    fn missing_duplicate_malformed_unsupported_and_stale_inputs_fail() {
        let complete = complete_artifacts();
        let baseline = must(build_release_bundle(complete.clone()));

        let mut missing = complete.clone();
        missing.pop();
        assert_eq!(
            issue_code(build_release_bundle(missing)),
            ReleaseIssueCode::ArtifactCountMismatch
        );

        let mut duplicate = complete.clone();
        duplicate[0].role = duplicate[1].role;
        assert_eq!(
            issue_code(build_release_bundle(duplicate)),
            ReleaseIssueCode::DuplicateRole
        );

        let mut undeclared = complete.clone();
        undeclared.push(complete[0].clone());
        assert_eq!(
            issue_code(build_release_bundle(undeclared)),
            ReleaseIssueCode::ArtifactCountMismatch
        );

        let mut malformed = complete.clone();
        malformed[0].relative_path = String::from("../escape");
        assert_eq!(
            issue_code(build_release_bundle(malformed)),
            ReleaseIssueCode::InvalidRelativePath
        );

        let mut wrong_schema = complete.clone();
        wrong_schema[0].schema = String::from("unsupported.v9");
        assert_eq!(
            issue_code(build_release_bundle(wrong_schema)),
            ReleaseIssueCode::UnsupportedSchema
        );

        let mut skipped = complete.clone();
        let mut is_changed = false;
        for artifact in &mut skipped {
            if artifact.role == ReleaseRole::TraceProofResult {
                artifact.status = ArtifactStatus::Observed;
                is_changed = true;
            }
        }
        assert!(is_changed);
        assert_eq!(
            issue_code(build_release_bundle(skipped)),
            ReleaseIssueCode::InvalidStatus
        );

        let mut stale = complete;
        stale[0].digest = ArtifactDigest::from_bytes([0x7f; DIGEST_BYTES]);
        assert_eq!(
            issue_code(verify_release_bundle(&baseline, stale)),
            ReleaseIssueCode::ArtifactMismatch
        );

        let mut mismatched = baseline.clone();
        mismatched.identity = BundleIdentity::from_bytes([0x6f; DIGEST_BYTES]);
        assert_eq!(
            issue_code(verify_release_bundle(&mismatched, complete_artifacts())),
            ReleaseIssueCode::BundleIdentityMismatch
        );
    }

    #[test]
    fn overclaim_and_projection_mutation_fail() {
        let artifacts = complete_artifacts();
        let mut bundle = must(build_release_bundle(artifacts.clone()));
        bundle.non_claims.pop();
        assert_eq!(
            issue_code(verify_release_bundle(&bundle, artifacts)),
            ReleaseIssueCode::NonClaimMismatch
        );

        let valid_bundle = must(build_release_bundle(complete_artifacts()));
        let mut projection = project_valence(&valid_bundle);
        projection.entries[0].claim = String::from("whole-product-verified");
        assert_eq!(
            issue_code(verify_valence_projection(&valid_bundle, &projection)),
            ReleaseIssueCode::ProjectionMismatch
        );
    }

    #[test]
    fn publication_candidate_is_local_exact_and_fail_closed() {
        let bundle = BundleIdentity::from_bytes([BUNDLE_BYTE; DIGEST_BYTES]);
        let candidate = must(build_publication_candidate(candidate_input(bundle)));
        assert_eq!(candidate.packages.len(), REQUIRED_PACKAGES.len());
        assert_eq!(candidate.bundle, bundle);
        assert!(verify_publication_candidate(&candidate, candidate_input(bundle)).is_ok());

        let mut stale_candidate = candidate.clone();
        stale_candidate.rollback_revision = String::from(SOURCE_REVISION);
        assert_eq!(
            issue_code(verify_publication_candidate(
                &stale_candidate,
                candidate_input(bundle)
            )),
            ReleaseIssueCode::CandidateMismatch
        );

        let mut floating = candidate_input(bundle);
        floating.source_revision = String::from("main");
        assert_eq!(
            issue_code(build_publication_candidate(floating)),
            ReleaseIssueCode::InvalidSourceRevision
        );

        let mut missing_package = candidate_input(bundle);
        missing_package.packages.pop();
        assert_eq!(
            issue_code(build_publication_candidate(missing_package)),
            ReleaseIssueCode::PackageSetMismatch
        );

        let mut wrong_adapter = candidate_input(bundle);
        wrong_adapter.acquisition_adapter = String::from("github-floating");
        assert_eq!(
            issue_code(build_publication_candidate(wrong_adapter)),
            ReleaseIssueCode::AcquisitionAdapterMismatch
        );
    }
}
