mod capability;
mod corpus;
mod release;

pub use capability::{CompilationReport, CompileNickelRequest, OperatorRootSpec, compile_nickel};
pub use corpus::{KamaciteCorpusEntry, build_kamacite_corpus};
pub use release::{
    PublicationCandidateReport, ReleaseBundleReport, ReleaseShellFailure, ReleaseShellIssueCode,
    ValidatePublicationCandidateRequest, VerifyReleaseBundleRequest, WriteReleaseBundleRequest,
    validate_publication_candidate, verify_release_bundle_file, write_release_bundle,
};

/// A command accepted by the imperative shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Boundary,
    CompileNickel {
        request: Box<CompileNickelRequest>,
    },
    WriteKamaciteCorpus {
        output_directory: String,
    },
    WriteReleaseBundle {
        request: WriteReleaseBundleRequest,
    },
    VerifyReleaseBundle {
        request: VerifyReleaseBundleRequest,
    },
    ValidatePublicationCandidate {
        request: ValidatePublicationCandidateRequest,
    },
}

fn parse_operator_root(value: &str) -> Result<OperatorRootSpec, &'static str> {
    let (id, ambient_path) = value
        .split_once('=')
        .ok_or("import root must use <identifier>=<ambient-directory>")?;
    if id.is_empty() || ambient_path.is_empty() {
        return Err("import root identifier and ambient directory must be non-empty");
    }
    Ok(OperatorRootSpec {
        id: String::from(id),
        ambient_path: std::path::PathBuf::from(ambient_path),
    })
}

#[allow(
    unknown_lints,
    missing_const_fn,
    reason = "release command parsing constructs owned paths, which are not available in const evaluation"
)]
fn parse_release_command(arguments: &[String]) -> Option<Result<Command, &'static str>> {
    match arguments {
        [command, artifact_root, output_root, output_path] if command == "write-release-bundle" => {
            if artifact_root.is_empty() || output_root.is_empty() || output_path.is_empty() {
                return Some(Err("release roots and output path must be non-empty"));
            }
            Some(Ok(Command::WriteReleaseBundle {
                request: WriteReleaseBundleRequest {
                    artifact_root: std::path::PathBuf::from(artifact_root),
                    output_root: std::path::PathBuf::from(output_root),
                    output_path: std::path::PathBuf::from(output_path),
                },
            }))
        }
        [command, artifact_root, manifest_root, manifest_path]
            if command == "verify-release-bundle" =>
        {
            if artifact_root.is_empty() || manifest_root.is_empty() || manifest_path.is_empty() {
                return Some(Err("release roots and manifest path must be non-empty"));
            }
            Some(Ok(Command::VerifyReleaseBundle {
                request: VerifyReleaseBundleRequest {
                    artifact_root: std::path::PathBuf::from(artifact_root),
                    manifest_root: std::path::PathBuf::from(manifest_root),
                    manifest_path: std::path::PathBuf::from(manifest_path),
                },
            }))
        }
        [
            command,
            source_revision,
            source_archive,
            release_bundle,
            rollback_revision,
        ] if command == "validate-publication-candidate" => {
            Some(Ok(Command::ValidatePublicationCandidate {
                request: ValidatePublicationCandidateRequest {
                    source_revision: source_revision.clone(),
                    source_archive_blake3: source_archive.clone(),
                    release_bundle_blake3: release_bundle.clone(),
                    rollback_revision: rollback_revision.clone(),
                },
            }))
        }
        _ => None,
    }
}

fn parse_non_release_command(arguments: &[String]) -> Result<Command, &'static str> {
    match arguments {
        [] => Ok(Command::Boundary),
        [argument] if argument == "boundary" => Ok(Command::Boundary),
        [
            command,
            evaluator,
            source_root,
            source_path,
            contract_root,
            scratch_root,
            output_root,
            output_path,
            root_arguments @ ..,
        ] if command == "compile-nickel" => {
            let root_specs = root_arguments
                .iter()
                .map(|root| parse_operator_root(root))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Command::CompileNickel {
                request: Box::new(CompileNickelRequest {
                    evaluator: std::path::PathBuf::from(evaluator),
                    source_root: std::path::PathBuf::from(source_root),
                    source_path: std::path::PathBuf::from(source_path),
                    contract_root: std::path::PathBuf::from(contract_root),
                    scratch_root: std::path::PathBuf::from(scratch_root),
                    output_root: std::path::PathBuf::from(output_root),
                    output_path: std::path::PathBuf::from(output_path),
                    import_roots: root_specs,
                    limits: choregraph_nickel::reviewed_frontend_limits(),
                }),
            })
        }
        [command, output_directory] if command == "write-kamacite-corpus" => {
            if output_directory.is_empty() {
                return Err("Kamacite corpus output directory must be non-empty");
            }
            Ok(Command::WriteKamaciteCorpus {
                output_directory: output_directory.clone(),
            })
        }
        _ => Err(
            "usage: choregraph [boundary | compile-nickel <evaluator> <source-root> <source-path> <contract-root> <scratch-root> <output-root> <output-path> [<import-id>=<import-root> ...] | write-kamacite-corpus <output-directory> | write-release-bundle <artifact-root> <output-root> <output-path> | verify-release-bundle <artifact-root> <manifest-root> <manifest-path> | validate-publication-candidate <source-revision> <source-archive-blake3> <release-bundle-blake3> <rollback-revision>]",
        ),
    }
}

/// Parses a command without reading process state.
pub fn parse_command(arguments: &[String]) -> Result<Command, &'static str> {
    match parse_release_command(arguments) {
        Some(release_command) => release_command,
        None => parse_non_release_command(arguments),
    }
}

/// Renders the bounded, unverified foundation report.
// This pure report composes claim predicates; their owner functions assert local invariants.
#[allow(
    unknown_lints,
    assertion_density,
    reason = "this pure report composes claim predicates; their owner functions assert local invariants"
)]
#[must_use]
pub fn render_foundation_report() -> String {
    let foundation_claim = choregraph_core::FOUNDATION_CLAIM;
    let machine_claim = choregraph_core::MACHINE_CLAIM;
    let is_core_pure = choregraph_core::permits(choregraph_core::Capability::DeterministicDecision);
    let is_kamacite_identity_only =
        choregraph_kamacite::supports(
            choregraph_kamacite::InterchangeClaim::CanonicalIdentityBinding,
        ) && !choregraph_kamacite::supports(choregraph_kamacite::InterchangeClaim::BehavioralProof);
    let is_nickel_evaluated_data_only =
        choregraph_nickel::accepts(choregraph_nickel::LoweringInputStage::EvaluatedData)
            && !choregraph_nickel::accepts(choregraph_nickel::LoweringInputStage::NickelSource);
    let is_seaglass_delegate_only = choregraph_seaglass::accepts(
        choregraph_seaglass::CompatibilityMode::DelegateToChoregraphCore,
    ) && !choregraph_seaglass::accepts(
        choregraph_seaglass::CompatibilityMode::IndependentTransitionImplementation,
    );

    format!(
        "status={foundation_claim}\nmachine-status={machine_claim}\ncore-pure={is_core_pure}\nkamacite-identity-only={is_kamacite_identity_only}\nnickel-evaluated-data-only={is_nickel_evaluated_data_only}\nseaglass-delegate-only={is_seaglass_delegate_only}\n"
    )
}

#[must_use]
pub fn render_release_bundle_report(report: &ReleaseBundleReport) -> String {
    format!(
        "status=bundled\nbundle-blake3={}\nartifacts={}\nartifact-bytes={}\nmanifest-bytes={}\n",
        report.bundle_blake3, report.artifact_count, report.artifact_bytes, report.manifest_bytes,
    )
}

pub fn render_release_verification_report(report: &ReleaseBundleReport) -> String {
    format!(
        "status=verified\nbundle-blake3={}\nartifacts={}\nartifact-bytes={}\nmanifest-bytes={}\n",
        report.bundle_blake3, report.artifact_count, report.artifact_bytes, report.manifest_bytes,
    )
}

pub fn render_publication_candidate_report(report: &PublicationCandidateReport) -> String {
    format!(
        "status=local-candidate\ncandidate-blake3={}\nsource-revision={}\nsource-archive-blake3={}\nrelease-bundle-blake3={}\nrollback-revision={}\n",
        report.candidate_blake3,
        report.source_revision,
        report.source_archive_blake3,
        report.release_bundle_blake3,
        report.rollback_revision,
    )
}

pub fn render_compilation_report(report: &CompilationReport) -> String {
    format!(
        "status=compiled\nsource-format={:?}\nimports={}\nimport-bytes={}\nmachine-blake3={}\nartifact-bytes={}\n",
        report.source_format,
        report.import_count,
        report.import_bytes,
        report.machine_identity,
        report.artifact_bytes,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_default_and_explicit_boundary_commands() {
        assert_eq!(super::parse_command(&[]), Ok(super::Command::Boundary));
        assert_eq!(
            super::parse_command(&[String::from("boundary")]),
            Ok(super::Command::Boundary)
        );
        assert_eq!(
            super::parse_command(&[
                String::from("write-kamacite-corpus"),
                String::from("fixtures/kamacite"),
            ]),
            Ok(super::Command::WriteKamaciteCorpus {
                output_directory: String::from("fixtures/kamacite"),
            })
        );
        assert_eq!(
            super::parse_command(&[
                String::from("write-release-bundle"),
                String::from("/source"),
                String::from("/output"),
                String::from("release.bundle"),
            ]),
            Ok(super::Command::WriteReleaseBundle {
                request: super::WriteReleaseBundleRequest {
                    artifact_root: std::path::PathBuf::from("/source"),
                    output_root: std::path::PathBuf::from("/output"),
                    output_path: std::path::PathBuf::from("release.bundle"),
                },
            })
        );
        assert_eq!(
            super::parse_command(&[
                String::from("verify-release-bundle"),
                String::from("/source"),
                String::from("/output"),
                String::from("release.bundle"),
            ]),
            Ok(super::Command::VerifyReleaseBundle {
                request: super::VerifyReleaseBundleRequest {
                    artifact_root: std::path::PathBuf::from("/source"),
                    manifest_root: std::path::PathBuf::from("/output"),
                    manifest_path: std::path::PathBuf::from("release.bundle"),
                },
            })
        );
        assert_eq!(
            super::parse_command(&[
                String::from("validate-publication-candidate"),
                String::from("source-revision"),
                String::from("source-archive"),
                String::from("release-bundle"),
                String::from("rollback-revision"),
            ]),
            Ok(super::Command::ValidatePublicationCandidate {
                request: super::ValidatePublicationCandidateRequest {
                    source_revision: String::from("source-revision"),
                    source_archive_blake3: String::from("source-archive"),
                    release_bundle_blake3: String::from("release-bundle"),
                    rollback_revision: String::from("rollback-revision"),
                },
            })
        );
    }

    #[test]
    fn parses_capability_compile_command_with_operator_roots() {
        let arguments = [
            "compile-nickel",
            "/nix/store/nickel/bin/nickel",
            "/operator/source",
            "graph/main.ncl",
            "/operator/contracts",
            "/operator/scratch",
            "/operator/output",
            "machines/main.preserves",
            "source=/operator/imports",
        ]
        .map(String::from);
        let command = match super::parse_command(&arguments) {
            Ok(command) => command,
            Err(error) => panic!("compile command failed: {error}"),
        };
        let super::Command::CompileNickel { request } = command else {
            panic!("compile command returned another variant");
        };
        assert_eq!(request.source_path, std::path::Path::new("graph/main.ncl"));
        assert_eq!(request.import_roots.len(), 1);
        assert_eq!(request.import_roots[0].id, "source");
    }

    #[test]
    fn rejects_unknown_incomplete_or_extra_arguments() {
        assert!(super::parse_command(&[String::from("run")]).is_err());
        assert!(super::parse_command(&[String::from("write-kamacite-corpus")]).is_err());
        assert!(super::parse_command(&[String::from("boundary"), String::from("extra")]).is_err());
        let malformed_root = [
            "compile-nickel",
            "/bin/false",
            "/source",
            "graph.ncl",
            "/contracts",
            "/scratch",
            "/output",
            "machine.preserves",
            "missing-separator",
        ]
        .map(String::from);
        assert!(super::parse_command(&malformed_root).is_err());
    }

    #[test]
    fn report_keeps_the_foundation_claim_bounded() {
        let foundation_output_text = super::render_foundation_report();
        assert!(foundation_output_text.contains("status=repository-boundary-only-unverified"));
        assert!(foundation_output_text.contains("seaglass-delegate-only=true"));
    }
}
