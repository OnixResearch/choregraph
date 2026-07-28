// The imperative shell keeps explicit imports readable while the pure cores stay isolated.
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the imperative shell keeps explicit imports readable while the pure cores stay isolated"
)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use choregraph_cli::{
    Command, KamaciteCorpusEntry, build_kamacite_corpus, compile_nickel, parse_command,
    render_compilation_report, render_foundation_report, render_publication_candidate_report,
    render_release_bundle_report, render_release_verification_report,
    validate_publication_candidate, verify_release_bundle_file, write_release_bundle,
};

// This I/O shell returns bounded errors; pure corpus validation stays in the library core.
#[allow(
    assertion_density,
    reason = "this I/O shell returns bounded errors; pure corpus validation stays in the library core"
)]
fn write_corpus(output_directory: &Path, entries: &[KamaciteCorpusEntry]) -> Result<(), String> {
    if output_directory.as_os_str().is_empty() {
        return Err(String::from("Kamacite corpus output directory is empty"));
    }
    for entry in entries {
        let destination = output_directory.join(entry.relative_path);
        let parent = destination.parent().ok_or_else(|| {
            format!(
                "corpus destination has no parent: {}",
                destination.display()
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        fs::write(&destination, &entry.bytes)
            .map_err(|error| format!("cannot write {}: {error}", destination.display()))?;
        println!("{}", destination.display());
    }
    Ok(())
}

fn run(arguments: &[String]) -> Result<(), String> {
    match parse_command(arguments).map_err(String::from)? {
        Command::Boundary => {
            print!("{}", render_foundation_report());
            Ok(())
        }
        Command::CompileNickel { request } => {
            let compilation_result = compile_nickel(&request).map_err(|failure| {
                failure
                    .issues
                    .iter()
                    .map(|issue| format!("{:?}: {}", issue.code(), issue.summary()))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
            print!("{}", render_compilation_report(&compilation_result));
            Ok(())
        }
        Command::WriteKamaciteCorpus { output_directory } => {
            let entries = build_kamacite_corpus()?;
            write_corpus(Path::new(&output_directory), &entries)
        }
        Command::WriteReleaseBundle { request } => {
            let bundle_result = write_release_bundle(&request)
                .map_err(|failure| format!("release bundle rejected: {:?}", failure.code))?;
            print!("{}", render_release_bundle_report(&bundle_result));
            Ok(())
        }
        Command::VerifyReleaseBundle { request } => {
            let bundle_result = verify_release_bundle_file(&request)
                .map_err(|failure| format!("release bundle rejected: {:?}", failure.code))?;
            print!("{}", render_release_verification_report(&bundle_result));
            Ok(())
        }
        Command::ValidatePublicationCandidate { request } => {
            let candidate = validate_publication_candidate(&request)
                .map_err(|failure| format!("publication candidate rejected: {:?}", failure.code))?;
            print!("{}", render_publication_candidate_report(&candidate));
            Ok(())
        }
    }
}

// The CLI owns argument access; no library or verified core reads the ambient environment.
#[allow(
    ambient_env,
    reason = "the CLI owns argument access; no library or verified core reads the ambient environment"
)]
fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
