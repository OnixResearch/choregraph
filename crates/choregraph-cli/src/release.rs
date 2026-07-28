// The release shell reads fixed reviewed paths through capabilities and delegates all policy to the pure release core.
#![allow(
    unknown_lints,
    assertion_density,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    reason = "the release shell keeps one fixed artifact inventory and explicit capability-relative orchestration"
)]

use std::path::{Path, PathBuf};

use cap_std::fs::Dir;
use choregraph_release::{
    ACQUISITION_ADAPTER, ArtifactDigest, ArtifactObservation, BundleIdentity, PROJECT_LICENSE,
    PublicationCandidateInput, REQUIRED_NON_CLAIMS, REQUIRED_PACKAGES, ReleaseBundle, ReleaseRole,
    build_publication_candidate, build_release_bundle, expected_schema, expected_status,
    project_valence, verify_publication_candidate, verify_release_bundle,
    verify_valence_projection,
};

const MAX_ARTIFACT_BYTES: usize = 16_777_216;
const MAX_TOTAL_ARTIFACT_BYTES: usize = 67_108_864;
const MAX_MANIFEST_BYTES: usize = 1_048_576;
const ARTIFACT_COUNT: usize = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReleaseArtifactSpec {
    pub role: ReleaseRole,
    pub relative_path: &'static str,
}

pub const RELEASE_ARTIFACTS: [ReleaseArtifactSpec; ARTIFACT_COUNT] = [
    ReleaseArtifactSpec {
        role: ReleaseRole::SourceTree,
        relative_path: "evidence/generated/source-baseline.json",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::License,
        relative_path: "LICENSE",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::MachineSchema,
        relative_path: "schemas/preserves/choregraph-machine-v1.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::LinkReceiptSchema,
        relative_path: "schemas/preserves/choregraph-link-receipt-v1.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::CoreSource,
        relative_path: "evidence/release/observations/core-source.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::CoreLibrary,
        relative_path: "evidence/release/observations/core-library.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::SingleStepProofSource,
        relative_path: "verus/proofs/single_step.rs",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::SingleStepProofResult,
        relative_path: "evidence/generated/single-step-proof.json",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::TraceProofSource,
        relative_path: "verus/proofs/trace_replay.rs",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::TraceProofResult,
        relative_path: "evidence/generated/trace-replay-proof.json",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::TrustedSurface,
        relative_path: "evidence/release/observations/trusted-surface.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::RustToolchain,
        relative_path: "rust-toolchain.toml",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::VerusToolchain,
        relative_path: "evidence/release/observations/verus-toolchain.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::SolverToolchain,
        relative_path: "evidence/release/observations/solver-toolchain.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::NickelToolchain,
        relative_path: "evidence/release/observations/nickel-toolchain.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::NixInputs,
        relative_path: "flake.lock",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::DependencyLock,
        relative_path: "Cargo.lock",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::KamaciteMachine,
        relative_path: "fixtures/kamacite/positive/machine.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::KamaciteLifecycle,
        relative_path: "fixtures/kamacite/positive/lifecycle.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::KamaciteTraceReceipt,
        relative_path: "fixtures/kamacite/positive/trace.receipt.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::KamaciteExhaustivenessReceipt,
        relative_path: "fixtures/kamacite/positive/exhaustiveness.receipt.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::KamaciteSourceReceipt,
        relative_path: "fixtures/kamacite/positive/source.receipt.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::KamaciteProofReferenceReceipt,
        relative_path: "fixtures/kamacite/positive/proof-reference.receipt.preserves",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::PositiveFixtureCorpus,
        relative_path: "fixtures/kamacite/corpus.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::NegativeFixtureCorpus,
        relative_path: "evidence/release/observations/negative-corpus.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::ExecutionGraphParity,
        relative_path: "cairn/archive/1970-01-01-add-seaglass-lattice-conformance/evidence/parity-validation-2026-07-27.md",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::LatticeDownstream,
        relative_path: "cairn/archive/1970-01-01-add-seaglass-lattice-conformance/evidence/lattice-downstream-validation-2026-07-27.md",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::CairnReceipt,
        relative_path: "evidence/release/observations/cairn-receipt.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::OctetReceipt,
        relative_path: "evidence/release/observations/octet-receipt.ncl",
    },
    ReleaseArtifactSpec {
        role: ReleaseRole::NixReceipt,
        relative_path: "evidence/release/observations/nix-receipt.ncl",
    },
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteReleaseBundleRequest {
    pub artifact_root: PathBuf,
    pub output_root: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifyReleaseBundleRequest {
    pub artifact_root: PathBuf,
    pub manifest_root: PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatePublicationCandidateRequest {
    pub source_revision: String,
    pub source_archive_blake3: String,
    pub release_bundle_blake3: String,
    pub rollback_revision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicationCandidateReport {
    pub candidate_blake3: String,
    pub source_revision: String,
    pub source_archive_blake3: String,
    pub release_bundle_blake3: String,
    pub rollback_revision: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseBundleReport {
    pub bundle_blake3: String,
    pub artifact_count: usize,
    pub artifact_bytes: usize,
    pub manifest_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseShellIssueCode {
    RootUnavailable,
    InputUnavailable,
    ArtifactLimitExceeded,
    TotalLimitExceeded,
    LinkageRejected,
    OutputRejected,
    ManifestLimitExceeded,
    ManifestMismatch,
    InvalidIdentity,
    CandidateRejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseShellFailure {
    pub code: ReleaseShellIssueCode,
    pub role: Option<ReleaseRole>,
}

impl ReleaseShellFailure {
    const fn one(code: ReleaseShellIssueCode, role: Option<ReleaseRole>) -> Self {
        Self { code, role }
    }
}

fn open_root(path: &Path) -> Result<Dir, ReleaseShellFailure> {
    crate::capability::open_capability_dir(path)
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::RootUnavailable, None))
}

fn observe_artifacts(
    artifact_root: &Dir,
) -> Result<(Vec<ArtifactObservation>, usize), ReleaseShellFailure> {
    let mut observations = Vec::with_capacity(ARTIFACT_COUNT);
    let mut total_bytes = 0usize;
    for spec in RELEASE_ARTIFACTS {
        let bytes = crate::capability::read_relative(
            artifact_root,
            Path::new(spec.relative_path),
            MAX_ARTIFACT_BYTES,
        )
        .map_err(|_| {
            ReleaseShellFailure::one(ReleaseShellIssueCode::InputUnavailable, Some(spec.role))
        })?;
        total_bytes = total_bytes.checked_add(bytes.len()).ok_or_else(|| {
            ReleaseShellFailure::one(ReleaseShellIssueCode::TotalLimitExceeded, Some(spec.role))
        })?;
        if total_bytes > MAX_TOTAL_ARTIFACT_BYTES {
            return Err(ReleaseShellFailure::one(
                ReleaseShellIssueCode::TotalLimitExceeded,
                Some(spec.role),
            ));
        }
        observations.push(ArtifactObservation {
            role: spec.role,
            relative_path: String::from(spec.relative_path),
            schema: String::from(expected_schema(spec.role)),
            status: expected_status(spec.role),
            digest: ArtifactDigest::from_bytes(*blake3::hash(&bytes).as_bytes()),
        });
    }
    debug_assert_eq!(observations.len(), ARTIFACT_COUNT);
    debug_assert!(total_bytes <= MAX_TOTAL_ARTIFACT_BYTES);
    Ok((observations, total_bytes))
}

fn render_bundle(bundle: &ReleaseBundle) -> Vec<u8> {
    let projection = project_valence(bundle);
    let mut output = String::new();
    output.push_str("schema=");
    output.push_str(&bundle.schema);
    output.push('\n');
    output.push_str("profile=");
    output.push_str(&bundle.profile);
    output.push('\n');
    output.push_str("bundle-blake3=");
    output.push_str(&bundle.identity.hex());
    output.push('\n');
    for artifact in &bundle.artifacts {
        output.push_str("artifact=");
        output.push_str(artifact.role.label());
        output.push('\t');
        output.push_str(&artifact.relative_path);
        output.push('\t');
        output.push_str(&artifact.schema);
        output.push('\t');
        output.push_str(artifact.status.label());
        output.push('\t');
        output.push_str(&artifact.digest.hex());
        output.push('\n');
    }
    for entry in &projection.entries {
        output.push_str("valence-role=");
        output.push_str(&entry.role);
        output.push('\t');
        output.push_str(&entry.verification_role);
        output.push('\t');
        output.push_str(&entry.claim);
        output.push('\n');
    }
    for non_claim in REQUIRED_NON_CLAIMS {
        output.push_str("non-claim=");
        output.push_str(non_claim);
        output.push('\n');
    }
    output.into_bytes()
}

fn build_current_bundle(
    artifact_root: &Dir,
) -> Result<(ReleaseBundle, usize), ReleaseShellFailure> {
    let (observations, artifact_bytes) = observe_artifacts(artifact_root)?;
    let bundle = build_release_bundle(observations.clone())
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::LinkageRejected, None))?;
    verify_release_bundle(&bundle, observations)
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::LinkageRejected, None))?;
    let projection = project_valence(&bundle);
    verify_valence_projection(&bundle, &projection)
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::LinkageRejected, None))?;
    debug_assert_eq!(bundle.artifacts.len(), ARTIFACT_COUNT);
    debug_assert!(artifact_bytes <= MAX_TOTAL_ARTIFACT_BYTES);
    Ok((bundle, artifact_bytes))
}

struct ReleaseMeasurements {
    artifact_bytes: usize,
    manifest_bytes: usize,
}

fn report(bundle: &ReleaseBundle, measurements: ReleaseMeasurements) -> ReleaseBundleReport {
    ReleaseBundleReport {
        bundle_blake3: bundle.identity.hex(),
        artifact_count: bundle.artifacts.len(),
        artifact_bytes: measurements.artifact_bytes,
        manifest_bytes: measurements.manifest_bytes,
    }
}

pub fn write_release_bundle(
    request: &WriteReleaseBundleRequest,
) -> Result<ReleaseBundleReport, ReleaseShellFailure> {
    let artifact_root = open_root(&request.artifact_root)?;
    let output_root = open_root(&request.output_root)?;
    let (bundle, artifact_bytes) = build_current_bundle(&artifact_root)?;
    let manifest = render_bundle(&bundle);
    if manifest.len() > MAX_MANIFEST_BYTES {
        return Err(ReleaseShellFailure::one(
            ReleaseShellIssueCode::ManifestLimitExceeded,
            None,
        ));
    }
    crate::capability::publish_atomic(&output_root, &request.output_path, &manifest)
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::OutputRejected, None))?;
    debug_assert_eq!(bundle.artifacts.len(), ARTIFACT_COUNT);
    debug_assert!(!manifest.is_empty());
    Ok(report(
        &bundle,
        ReleaseMeasurements {
            artifact_bytes,
            manifest_bytes: manifest.len(),
        },
    ))
}

pub fn verify_release_bundle_file(
    request: &VerifyReleaseBundleRequest,
) -> Result<ReleaseBundleReport, ReleaseShellFailure> {
    let artifact_root = open_root(&request.artifact_root)?;
    let manifest_root = open_root(&request.manifest_root)?;
    let observed_manifest = crate::capability::read_relative(
        &manifest_root,
        &request.manifest_path,
        MAX_MANIFEST_BYTES,
    )
    .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::InputUnavailable, None))?;
    let (bundle, artifact_bytes) = build_current_bundle(&artifact_root)?;
    let expected_manifest = render_bundle(&bundle);
    if observed_manifest != expected_manifest {
        return Err(ReleaseShellFailure::one(
            ReleaseShellIssueCode::ManifestMismatch,
            None,
        ));
    }
    debug_assert_eq!(observed_manifest.len(), expected_manifest.len());
    debug_assert_eq!(bundle.artifacts.len(), ARTIFACT_COUNT);
    Ok(report(
        &bundle,
        ReleaseMeasurements {
            artifact_bytes,
            manifest_bytes: observed_manifest.len(),
        },
    ))
}

pub fn validate_publication_candidate(
    request: &ValidatePublicationCandidateRequest,
) -> Result<PublicationCandidateReport, ReleaseShellFailure> {
    let source_archive = ArtifactDigest::from_hex(&request.source_archive_blake3)
        .ok_or_else(|| ReleaseShellFailure::one(ReleaseShellIssueCode::InvalidIdentity, None))?;
    let bundle = BundleIdentity::from_hex(&request.release_bundle_blake3)
        .ok_or_else(|| ReleaseShellFailure::one(ReleaseShellIssueCode::InvalidIdentity, None))?;
    let input = PublicationCandidateInput {
        source_revision: request.source_revision.clone(),
        source_archive,
        license: String::from(PROJECT_LICENSE),
        packages: REQUIRED_PACKAGES
            .iter()
            .map(|package| String::from(*package))
            .collect(),
        acquisition_adapter: String::from(ACQUISITION_ADAPTER),
        bundle,
        rollback_revision: request.rollback_revision.clone(),
    };
    let candidate = build_publication_candidate(input.clone())
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::CandidateRejected, None))?;
    verify_publication_candidate(&candidate, input)
        .map_err(|_| ReleaseShellFailure::one(ReleaseShellIssueCode::CandidateRejected, None))?;
    debug_assert_eq!(candidate.source_revision, request.source_revision);
    debug_assert_eq!(candidate.rollback_revision, request.rollback_revision);
    Ok(PublicationCandidateReport {
        candidate_blake3: candidate.identity.hex(),
        source_revision: candidate.source_revision,
        source_archive_blake3: candidate.source_archive.hex(),
        release_bundle_blake3: candidate.bundle.hex(),
        rollback_revision: candidate.rollback_revision,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const FIXTURE_BYTES: &[u8] = b"reviewed release artifact";
    const SOURCE_REVISION: &str = "1111111111111111111111111111111111111111";
    const ROLLBACK_REVISION: &str = "2222222222222222222222222222222222222222";
    const SOURCE_ARCHIVE: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    const RELEASE_BUNDLE: &str = "4444444444444444444444444444444444444444444444444444444444444444";

    #[allow(
        unknown_lints,
        no_panic,
        reason = "positive shell fixtures must stop when bounded local setup or execution fails"
    )]
    fn must<T, E: core::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected success, got {error:?}"),
        }
    }

    #[allow(
        unknown_lints,
        no_panic,
        reason = "negative shell fixtures must stop when a rejected request unexpectedly succeeds"
    )]
    fn issue_code<T>(result: Result<T, ReleaseShellFailure>) -> ReleaseShellIssueCode {
        match result {
            Err(failure) => failure.code,
            Ok(_) => panic!("expected release shell failure"),
        }
    }

    fn populate_artifacts(root: &Path) -> Result<(), String> {
        for spec in RELEASE_ARTIFACTS {
            let path = root.join(spec.relative_path);
            let parent = path
                .parent()
                .ok_or_else(|| String::from("fixture artifact has no parent"))?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("cannot create fixture directory: {error}"))?;
            fs::write(path, FIXTURE_BYTES)
                .map_err(|error| format!("cannot write fixture artifact: {error}"))?;
        }
        Ok(())
    }

    #[test]
    fn capability_shell_writes_one_deterministic_bundle_without_replacement() {
        let artifact_root = must(tempfile::tempdir());
        let output_root = must(tempfile::tempdir());
        must(populate_artifacts(artifact_root.path()));
        let request = WriteReleaseBundleRequest {
            artifact_root: artifact_root.path().to_path_buf(),
            output_root: output_root.path().to_path_buf(),
            output_path: PathBuf::from("release.bundle"),
        };
        let report = must(write_release_bundle(&request));
        assert_eq!(report.artifact_count, ARTIFACT_COUNT);
        assert!(report.artifact_bytes > 0);
        assert!(report.manifest_bytes > 0);
        let manifest = must(fs::read(output_root.path().join("release.bundle")));
        assert!(manifest.starts_with(b"schema=choregraph.release-bundle.v1\n"));
        let verify_request = VerifyReleaseBundleRequest {
            artifact_root: artifact_root.path().to_path_buf(),
            manifest_root: output_root.path().to_path_buf(),
            manifest_path: PathBuf::from("release.bundle"),
        };
        let verified = must(verify_release_bundle_file(&verify_request));
        assert_eq!(verified.bundle_blake3, report.bundle_blake3);
        assert_eq!(
            issue_code(write_release_bundle(&request)),
            ReleaseShellIssueCode::OutputRejected
        );
        must(fs::write(
            artifact_root
                .path()
                .join(RELEASE_ARTIFACTS[0].relative_path),
            b"stale artifact",
        ));
        assert_eq!(
            issue_code(verify_release_bundle_file(&verify_request)),
            ReleaseShellIssueCode::ManifestMismatch
        );
    }

    #[test]
    fn local_publication_candidate_is_exact_and_fail_closed() {
        let request = ValidatePublicationCandidateRequest {
            source_revision: String::from(SOURCE_REVISION),
            source_archive_blake3: String::from(SOURCE_ARCHIVE),
            release_bundle_blake3: String::from(RELEASE_BUNDLE),
            rollback_revision: String::from(ROLLBACK_REVISION),
        };
        let candidate = must(validate_publication_candidate(&request));
        assert_eq!(candidate.source_revision, SOURCE_REVISION);
        assert_eq!(candidate.source_archive_blake3, SOURCE_ARCHIVE);
        assert_eq!(candidate.release_bundle_blake3, RELEASE_BUNDLE);
        assert_eq!(candidate.rollback_revision, ROLLBACK_REVISION);

        let mut floating = request.clone();
        floating.source_revision = String::from("main");
        assert_eq!(
            issue_code(validate_publication_candidate(&floating)),
            ReleaseShellIssueCode::CandidateRejected
        );

        let mut malformed = request;
        malformed.release_bundle_blake3 = String::from("not-a-digest");
        assert_eq!(
            issue_code(validate_publication_candidate(&malformed)),
            ReleaseShellIssueCode::InvalidIdentity
        );
    }

    #[test]
    fn capability_shell_rejects_missing_artifact_and_relative_root() {
        let output_root = must(tempfile::tempdir());
        let relative = WriteReleaseBundleRequest {
            artifact_root: PathBuf::from("relative"),
            output_root: output_root.path().to_path_buf(),
            output_path: PathBuf::from("release.bundle"),
        };
        assert_eq!(
            issue_code(write_release_bundle(&relative)),
            ReleaseShellIssueCode::RootUnavailable
        );

        let artifact_root = must(tempfile::tempdir());
        let missing = WriteReleaseBundleRequest {
            artifact_root: artifact_root.path().to_path_buf(),
            output_root: output_root.path().to_path_buf(),
            output_path: PathBuf::from("release.bundle"),
        };
        assert_eq!(
            issue_code(write_release_bundle(&missing)),
            ReleaseShellIssueCode::InputUnavailable
        );
    }
}
