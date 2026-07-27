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
    Command, KamaciteCorpusEntry, build_kamacite_corpus, parse_command, render_foundation_report,
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
        Command::WriteKamaciteCorpus { output_directory } => {
            let entries = build_kamacite_corpus()?;
            write_corpus(Path::new(&output_directory), &entries)
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
