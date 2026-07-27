mod capability;
mod corpus;

pub use capability::{CompilationReport, CompileNickelRequest, OperatorRootSpec, compile_nickel};
pub use corpus::{KamaciteCorpusEntry, build_kamacite_corpus};

/// A command accepted by the imperative shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Boundary,
    CompileNickel { request: Box<CompileNickelRequest> },
    WriteKamaciteCorpus { output_directory: String },
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

/// Parses a command without reading process state.
pub fn parse_command(arguments: &[String]) -> Result<Command, &'static str> {
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
            "usage: choregraph [boundary | compile-nickel <evaluator> <source-root> <source-path> <contract-root> <scratch-root> <output-root> <output-path> [<import-id>=<import-root> ...] | write-kamacite-corpus <output-directory>]",
        ),
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
