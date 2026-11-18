#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/choregraph-sync-kamacite-codec nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" "github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#gcc" "github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#b3sum" -c cargo -q -Zscript

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SELF_TEST_ARGUMENT_COUNT: usize = 2;
const SELF_TEST_FLAG: &str = "--self-test";
const KAMACITE_REVISION: &str = "6245a52498f7de511df660eabec3f9b350338cb6";
const DESTINATION_DIRECTORY: &str = "vendor/kamacite-core-selected";
const IMPORT_ENTRY_COUNT: usize = 3;
const CODEC_SOURCE_PATH: &str = "crates/kamacite-core/src/preserves_codec.rs";
const CODEC_TESTS_DESTINATION: &str = "preserves/tests.rs";
const CORRECTION_PATH: &str = "tools/kamacite-codec-correction.patch";
const CORRECTION_BLAKE3: &str = "af29bedfead5c029c5c930675c9f41cf9856230f9429cc941f6700ae613f2b4d";
const CORRECTED_CODEC_BLAKE3: &str =
    "6001e5509f44b1659d94d91a2a9c829df0492c0b734da79a485d7585b4d7ed20";
const CORRECTED_TESTS_BLAKE3: &str =
    "06ceb627b178d9029f7c1270d2a5696d3f0079aac7077283089192fbffd2d86a";
const PATCH_PACKAGE: &str = "github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#patch";

const IMPORT_ENTRIES: &[(&str, &str, &str)] = &[
    (
        CODEC_SOURCE_PATH,
        "preserves/codec.rs",
        "bbb09dd44cae9aaac5226d7298633600480ff1e1776717b05138c422167056a0",
    ),
    (
        "crates/kamacite-core/LICENSE-APACHE",
        "LICENSE-APACHE",
        "83cb3a2fcf829b6138e095b083016c34ddcdfa07b68d38782722c14fcf85ace6",
    ),
    (
        "crates/kamacite-core/LICENSE-MIT",
        "LICENSE-MIT",
        "4b370111beabfcbb64bc16b8e84411a741280be5df1324f95f9014268f36c6af",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
struct ImportEntry {
    source_path: &'static str,
    source_blake3: &'static str,
    destination: PathBuf,
}

fn import_plan(destination_root: &Path) -> Result<Vec<ImportEntry>, String> {
    if destination_root.as_os_str().is_empty() {
        return Err(String::from("destination root must be non-empty"));
    }
    let destination = destination_root.join(DESTINATION_DIRECTORY);
    let plan = IMPORT_ENTRIES
        .iter()
        .map(
            |(source_path, destination_name, source_blake3)| ImportEntry {
                source_path,
                source_blake3,
                destination: destination.join(destination_name),
            },
        )
        .collect::<Vec<_>>();
    if plan.len() != IMPORT_ENTRY_COUNT {
        return Err(String::from("Kamacite import inventory count drifted"));
    }
    Ok(plan)
}

fn read_git_object(repository: &Path, source_path: &str) -> Result<Vec<u8>, String> {
    let object = format!("{KAMACITE_REVISION}:{source_path}");
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

fn verify_blake3(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    let mut child = Command::new("b3sum")
        .arg("--no-names")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run b3sum for {label}: {error}"))?;
    let write_result = child
        .stdin
        .take()
        .ok_or_else(|| format!("b3sum stdin unavailable for {label}"))?
        .write_all(bytes);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("cannot wait for b3sum for {label}: {error}"))?;
    write_result.map_err(|error| format!("cannot hash {label}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "b3sum failed for {label}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let actual = String::from_utf8(output.stdout)
        .map_err(|error| format!("invalid b3sum output for {label}: {error}"))?;
    if actual.trim() != expected {
        return Err(format!(
            "BLAKE3 mismatch for {label}: expected {expected}, got {}",
            actual.trim()
        ));
    }
    Ok(())
}

fn correct_codec(base: &[u8], correction: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    let scratch = env::temp_dir().join(format!("choregraph-kamacite-codec-{}", std::process::id()));
    fs::create_dir(&scratch)
        .map_err(|error| format!("cannot create {}: {error}", scratch.display()))?;
    let result = (|| {
        fs::write(scratch.join("preserves_codec.rs"), base)
            .map_err(|error| format!("cannot stage exact codec base: {error}"))?;
        fs::create_dir(scratch.join("preserves_codec"))
            .map_err(|error| format!("cannot stage codec test directory: {error}"))?;
        fs::write(scratch.join("preserves_codec/tests.rs"), [])
            .map_err(|error| format!("cannot stage empty codec test member: {error}"))?;
        fs::write(scratch.join("correction.patch"), correction)
            .map_err(|error| format!("cannot stage reviewed correction: {error}"))?;
        let output = Command::new("nix")
            .args(["shell", PATCH_PACKAGE, "-c", "patch"])
            .current_dir(&scratch)
            .args([
                "--batch",
                "--forward",
                "--fuzz=0",
                "--silent",
                "--posix",
                "--strip=1",
                "--input=correction.patch",
            ])
            .output()
            .map_err(|error| format!("cannot run pinned GNU patch: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "reviewed codec correction failed: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let source = fs::read(scratch.join("preserves_codec.rs"))
            .map_err(|error| format!("cannot read corrected codec source: {error}"))?;
        let tests = fs::read(scratch.join("preserves_codec/tests.rs"))
            .map_err(|error| format!("cannot read corrected codec tests: {error}"))?;
        verify_blake3(&source, CORRECTED_CODEC_BLAKE3, "corrected codec source")?;
        verify_blake3(&tests, CORRECTED_TESTS_BLAKE3, "corrected codec tests")?;
        Ok((source, tests))
    })();
    let cleanup = fs::remove_dir_all(&scratch)
        .map_err(|error| format!("cannot remove {}: {error}", scratch.display()));
    let members = result?;
    cleanup?;
    Ok(members)
}

fn write_member(destination: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| format!("destination has no parent: {}", destination.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    fs::write(destination, bytes)
        .map_err(|error| format!("cannot write {}: {error}", destination.display()))
}

fn execute_import(
    source_repository: &Path,
    destination_root: &Path,
    plan: &[ImportEntry],
) -> Result<(), String> {
    if !source_repository.join(".git").exists() {
        return Err(format!(
            "source is not a Git repository: {}",
            source_repository.display()
        ));
    }
    let correction_path = destination_root.join(CORRECTION_PATH);
    let correction = fs::read(&correction_path)
        .map_err(|error| format!("cannot read {}: {error}", correction_path.display()))?;
    verify_blake3(&correction, CORRECTION_BLAKE3, CORRECTION_PATH)?;
    for entry in plan {
        let bytes = read_git_object(source_repository, entry.source_path)?;
        verify_blake3(&bytes, entry.source_blake3, entry.source_path)?;
        let bytes = if entry.source_path == CODEC_SOURCE_PATH {
            let (source, tests) = correct_codec(&bytes, &correction)?;
            let tests_destination = destination_root
                .join(DESTINATION_DIRECTORY)
                .join(CODEC_TESTS_DESTINATION);
            write_member(&tests_destination, &tests)?;
            let obsolete_directory = destination_root
                .join(DESTINATION_DIRECTORY)
                .join("preserves/codec");
            if obsolete_directory.exists() {
                fs::remove_dir_all(&obsolete_directory)
                    .map_err(|error| format!("cannot remove obsolete codec layout: {error}"))?;
            }
            println!(
                "reviewed codec correction -> {}",
                tests_destination.display()
            );
            source
        } else {
            bytes
        };
        write_member(&entry.destination, &bytes)?;
        println!(
            "{}@{}:{} -> {}",
            source_repository.display(),
            KAMACITE_REVISION,
            entry.source_path,
            entry.destination.display()
        );
    }
    Ok(())
}

fn self_test() -> Result<(), String> {
    let destination = Path::new("/tmp/choregraph-kamacite-destination");
    let plan = import_plan(destination)?;
    assert_eq!(plan.len(), IMPORT_ENTRY_COUNT);
    assert!(
        plan.iter()
            .all(|entry| entry.destination.starts_with(destination))
    );
    assert!(import_plan(Path::new("")).is_err());
    println!("Kamacite codec import self-test passed");
    Ok(())
}

fn run(arguments: &[String]) -> Result<(), String> {
    if arguments.len() == SELF_TEST_ARGUMENT_COUNT && arguments[1] == SELF_TEST_FLAG {
        return self_test();
    }
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err(String::from(
            "usage: sync-kamacite-codec.rs <kamacite-repository> <choregraph-root>",
        ));
    }
    let source_repository = Path::new(&arguments[1]);
    let destination_root = Path::new(&arguments[2]);
    let plan = import_plan(destination_root)?;
    execute_import(source_repository, destination_root, &plan)
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
