#![no_std]
#![allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    missing_const_fn,
    non_trait_imports,
    reason = "the closed release profile keeps exhaustive role mappings and explicit alloc value imports reviewable"
)]

extern crate alloc;

pub mod acquisition;
pub mod evidence;
pub mod linkage;
pub mod mapping;
pub mod profile;
pub mod rejection;

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;

    use crate::acquisition::{
        ACQUISITION_ADAPTER, PROJECT_LICENSE, PublicationCandidateInput, REQUIRED_PACKAGES,
        build_publication_candidate, verify_publication_candidate,
    };
    use crate::evidence::{
        ArtifactDigest, ArtifactObservation, ArtifactStatus, DIGEST_BYTES, DIGEST_HEX_BYTES,
    };
    use crate::linkage::{BundleIdentity, REQUIRED_NON_CLAIMS, build_bundle, verify_bundle};
    use crate::mapping::{project_valence, verify_valence_projection};
    use crate::profile::{
        REQUIRED_ROLE_COUNT, REQUIRED_ROLES, Role, expected_schema, expected_status,
    };
    use crate::rejection::{Failure, IssueCode};

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
    fn must<T>(result: Result<T, Failure>) -> T {
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
    fn issue_code<T>(result: Result<T, Failure>) -> IssueCode {
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
        let first = must(build_bundle(artifacts.clone()));
        let mut reversed = artifacts.clone();
        reversed.reverse();
        let second = must(build_bundle(reversed));
        assert_eq!(first, second);
        assert_eq!(first.artifacts.len(), REQUIRED_ROLE_COUNT);
        assert_eq!(first.non_claims.len(), REQUIRED_NON_CLAIMS.len());
        assert!(verify_bundle(&first, artifacts).is_ok());
    }

    #[test]
    fn missing_duplicate_malformed_unsupported_and_stale_inputs_fail() {
        let complete = complete_artifacts();
        let baseline = must(build_bundle(complete.clone()));

        let mut missing = complete.clone();
        missing.pop();
        assert_eq!(
            issue_code(build_bundle(missing)),
            IssueCode::ArtifactCountMismatch
        );

        let mut duplicate = complete.clone();
        duplicate[0].role = duplicate[1].role;
        assert_eq!(
            issue_code(build_bundle(duplicate)),
            IssueCode::DuplicateRole
        );

        let mut undeclared = complete.clone();
        undeclared.push(complete[0].clone());
        assert_eq!(
            issue_code(build_bundle(undeclared)),
            IssueCode::ArtifactCountMismatch
        );

        let mut malformed = complete.clone();
        malformed[0].relative_path = String::from("../escape");
        assert_eq!(
            issue_code(build_bundle(malformed)),
            IssueCode::InvalidRelativePath
        );

        let mut wrong_schema = complete.clone();
        wrong_schema[0].schema = String::from("unsupported.v9");
        assert_eq!(
            issue_code(build_bundle(wrong_schema)),
            IssueCode::UnsupportedSchema
        );

        let mut skipped = complete.clone();
        let mut is_changed = false;
        for artifact in &mut skipped {
            if artifact.role == Role::TraceProofResult {
                artifact.status = ArtifactStatus::Observed;
                is_changed = true;
            }
        }
        assert!(is_changed);
        assert_eq!(issue_code(build_bundle(skipped)), IssueCode::InvalidStatus);

        let mut stale = complete;
        stale[0].digest = ArtifactDigest::from_bytes([0x7f; DIGEST_BYTES]);
        assert_eq!(
            issue_code(verify_bundle(&baseline, stale)),
            IssueCode::ArtifactMismatch
        );

        let mut mismatched = baseline.clone();
        mismatched.identity = BundleIdentity::from_bytes([0x6f; DIGEST_BYTES]);
        assert_eq!(
            issue_code(verify_bundle(&mismatched, complete_artifacts())),
            IssueCode::BundleIdentityMismatch
        );
    }

    #[test]
    fn overclaim_and_projection_mutation_fail() {
        let artifacts = complete_artifacts();
        let mut bundle = must(build_bundle(artifacts.clone()));
        bundle.non_claims.pop();
        assert_eq!(
            issue_code(verify_bundle(&bundle, artifacts)),
            IssueCode::NonClaimMismatch
        );

        let valid_bundle = must(build_bundle(complete_artifacts()));
        let mut projection = project_valence(&valid_bundle);
        projection.entries[0].claim = String::from("whole-product-verified");
        assert_eq!(
            issue_code(verify_valence_projection(&valid_bundle, &projection)),
            IssueCode::ProjectionMismatch
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
            IssueCode::CandidateMismatch
        );

        let mut floating = candidate_input(bundle);
        floating.source_revision = String::from("main");
        assert_eq!(
            issue_code(build_publication_candidate(floating)),
            IssueCode::InvalidSourceRevision
        );

        let mut missing_package = candidate_input(bundle);
        missing_package.packages.pop();
        assert_eq!(
            issue_code(build_publication_candidate(missing_package)),
            IssueCode::PackageSetMismatch
        );

        let mut wrong_adapter = candidate_input(bundle);
        wrong_adapter.acquisition_adapter = String::from("github-floating");
        assert_eq!(
            issue_code(build_publication_candidate(wrong_adapter)),
            IssueCode::AcquisitionAdapterMismatch
        );
    }
}
