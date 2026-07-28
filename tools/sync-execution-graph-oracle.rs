#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/choregraph-sync-execution-graph-oracle nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" "github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#gcc" -c cargo -q -Zscript

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SELF_TEST_ARGUMENT_COUNT: usize = 2;
const SELF_TEST_FLAG: &str = "--self-test";
const EXECUTION_GRAPH_REVISION: &str = "03736f1ec46c377ff86b451260ad68aa70ff3b0b";
const DESTINATION_DIRECTORY: &str = "vendor/execution-graph-core-selected";
const IMPORT_ENTRY_COUNT: usize = 3;

const IMPORT_ENTRIES: &[(&str, &str)] = &[
    ("Cargo.toml", "Cargo.toml"),
    ("src/lib.rs", "src/lib.rs"),
    ("LICENSE", "LICENSE"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImportEntry {
    source_path: &'static str,
    destination: PathBuf,
}

fn import_plan(destination_root: &Path) -> Result<Vec<ImportEntry>, String> {
    if destination_root.as_os_str().is_empty() {
        return Err(String::from("destination root must be non-empty"));
    }
    let destination = destination_root.join(DESTINATION_DIRECTORY);
    let plan = IMPORT_ENTRIES
        .iter()
        .map(|(source_path, destination_name)| ImportEntry {
            source_path,
            destination: destination.join(destination_name),
        })
        .collect::<Vec<_>>();
    if plan.len() != IMPORT_ENTRY_COUNT {
        return Err(String::from(
            "execution-graph import inventory count drifted",
        ));
    }
    Ok(plan)
}

fn read_git_object(repository: &Path, source_path: &str) -> Result<Vec<u8>, String> {
    let object = format!("{EXECUTION_GRAPH_REVISION}:{source_path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .arg("show")
        .arg(&object)
        .output()
        .map_err(|error| format!("cannot run git show for {object}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git show failed for {object}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

fn execute_import(source_repository: &Path, plan: &[ImportEntry]) -> Result<(), String> {
    if !source_repository.join(".git").exists() {
        return Err(format!(
            "source is not a Git repository: {}",
            source_repository.display()
        ));
    }
    for entry in plan {
        let bytes = read_git_object(source_repository, entry.source_path)?;
        let parent = entry
            .destination
            .parent()
            .ok_or_else(|| format!("destination has no parent: {}", entry.destination.display()))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        fs::write(&entry.destination, bytes)
            .map_err(|error| format!("cannot write {}: {error}", entry.destination.display()))?;
        println!(
            "{}@{}:{} -> {}",
            source_repository.display(),
            EXECUTION_GRAPH_REVISION,
            entry.source_path,
            entry.destination.display()
        );
    }
    Ok(())
}

fn self_test() -> Result<(), String> {
    let destination = Path::new("/tmp/choregraph-execution-graph-oracle-destination");
    let plan = import_plan(destination)?;
    if plan.len() != IMPORT_ENTRY_COUNT {
        return Err(String::from("positive import plan has the wrong size"));
    }
    if !plan
        .iter()
        .all(|entry| entry.destination.starts_with(destination))
    {
        return Err(String::from("positive import plan escaped its destination"));
    }
    if import_plan(Path::new("")).is_ok() {
        return Err(String::from("empty destination unexpectedly passed"));
    }
    Ok(())
}

fn run(arguments: &[String]) -> Result<(), String> {
    if arguments.len() == SELF_TEST_ARGUMENT_COUNT && arguments[1] == SELF_TEST_FLAG {
        return self_test();
    }
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err(String::from(
            "usage: sync-execution-graph-oracle.rs <execution-graph-repository> <choregraph-root>",
        ));
    }
    let source_repository = Path::new(&arguments[1]);
    let destination_root = Path::new(&arguments[2]);
    let plan = import_plan(destination_root)?;
    execute_import(source_repository, &plan)
}

fn main() -> ExitCode {
    let arguments = env::args().collect::<Vec<_>>();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
