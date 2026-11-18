#!/usr/bin/env -S nix shell github:nix-community/fenix/4e49ef62eedaa5c149d62b63b3f53844e1cc45d7#minimal.toolchain github:NixOS/nixpkgs/20535e48e12c86043b577b8518234ff5dbb26957#gcc -c cargo -q -Zscript

// Qualifies the historical runner semantics after the declared Seaglass Rust API
// namespace migration below, not drop-in Rust source compatibility. The verified
// base revision, test assertions, fixture data and selected runner tests stay fixed.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const LATTICE_REVISION: &str = "c8d56ad83a57287064edb85f58808f53e4bf4536";
const LEGACY_DEPENDENCY: &str =
    "seaglass-execution-graph = { path = \"vendor-cargo-deps/seaglass-execution-graph\" }";
const MANIFEST_PATH: &str = "Cargo.toml";
const CARGO_CONFIG_PATH: &str = ".cargo/config.toml";
const MINIMAL_CARGO_CONFIG: &str = "[net]\ngit-fetch-with-cli = true\n";
const ADAPTER_PATH: &str = "crates/choregraph-seaglass";
const RUNNER_TEST: &str = "test_runner_core";
const RUNNER_SOURCE_PATH: &str = "src/engine/runner.rs";
// Exact occurrences in LATTICE_REVISION only. Longer diagnostic paths precede
// their common prefix so each reviewed occurrence is migrated exactly once.
const RUNNER_NAMESPACE_MIGRATIONS: &[(&str, &str, usize)] = &[
    ("seaglass_execution_graph::ResumeDecision", "seaglass_execution_graph::aggregate::ResumeDecision", 1),
    ("seaglass_execution_graph::StepAction", "seaglass_execution_graph::lifecycle::StepAction", 1),
    ("seaglass_execution_graph::StepState", "seaglass_execution_graph::lifecycle::StepState", 2),
    ("seaglass_execution_graph::StepTransitionError", "seaglass_execution_graph::lifecycle::StepTransitionError", 1),
    ("seaglass_execution_graph::ready_nodes", "seaglass_execution_graph::aggregate::ready_nodes", 1),
    ("seaglass_execution_graph::transition_step_state", "seaglass_execution_graph::lifecycle::transition_step_state", 1),
    ("seaglass_execution_graph::is_terminal_state", "seaglass_execution_graph::lifecycle::is_terminal_state", 1),
    ("seaglass_execution_graph::ExecutionPlan", "seaglass_execution_graph::aggregate::ExecutionPlan", 1),
    ("seaglass_execution_graph::admit_graph", "seaglass_execution_graph::admission::admit_graph", 1),
    ("seaglass_execution_graph::ExecutionGraph", "seaglass_execution_graph::topology::ExecutionGraph", 2),
    ("seaglass_execution_graph::ExecutionNode", "seaglass_execution_graph::topology::ExecutionNode", 1),
    ("seaglass_execution_graph::ExecutionEdge", "seaglass_execution_graph::topology::ExecutionEdge", 1),
    ("seaglass_execution_graph::GraphDiagnosticCode", "seaglass_execution_graph::admission::GraphDiagnosticCode", 8),
    ("seaglass_execution_graph::GraphDiagnostic", "seaglass_execution_graph::admission::GraphDiagnostic", 1),
    ("seaglass_execution_graph::resume_decision", "seaglass_execution_graph::aggregate::resume_decision", 1),
];
const SELF_TEST_FLAG: &str = "--self-test";
const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SELF_TEST_ARGUMENT_COUNT: usize = 2;
const SINGLE_MATCH_COUNT: usize = 1;

struct DisposableRoot {
    path: PathBuf,
}

impl DisposableRoot {
    fn create() -> Result<Self, String> {
        let path = env::temp_dir().join(format!(
            "choregraph-lattice-downstream-{}",
            std::process::id()
        ));
        if path.exists() {
            return Err(format!(
                "disposable downstream path already exists: {}",
                path.display()
            ));
        }
        fs::create_dir(&path)
            .map_err(|error| format!("cannot create {}: {error}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for DisposableRoot {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.path);
    }
}

fn dependency_replacement(adapter: &Path) -> Result<String, String> {
    let adapter = adapter
        .to_str()
        .ok_or_else(|| String::from("adapter path is not UTF-8"))?;
    if adapter.contains(['"', '\n', '\r']) {
        return Err(String::from(
            "adapter path contains an unsupported character",
        ));
    }
    Ok(format!(
        "seaglass-execution-graph = {{ package = \"choregraph-seaglass\", path = \"{adapter}\" }}"
    ))
}

fn patched_manifest(manifest: &str, replacement: &str) -> Result<String, String> {
    let match_count = manifest.matches(LEGACY_DEPENDENCY).count();
    if match_count != SINGLE_MATCH_COUNT {
        return Err(format!(
            "expected one reviewed Seaglass dependency, found {match_count}"
        ));
    }
    Ok(manifest.replacen(LEGACY_DEPENDENCY, replacement, SINGLE_MATCH_COUNT))
}

fn exact_revision(output: &[u8]) -> Result<(), String> {
    let observed = std::str::from_utf8(output)
        .map_err(|_| String::from("git revision output is not UTF-8"))?
        .trim();
    if observed != LATTICE_REVISION {
        return Err(format!(
            "Lattice revision mismatch: expected {LATTICE_REVISION}, observed {observed}"
        ));
    }
    Ok(())
}

fn checked_output(command: &mut Command, description: &str) -> Result<Vec<u8>, String> {
    let output = command
        .output()
        .map_err(|error| format!("cannot run {description}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{description} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

fn verify_revision(repository: &Path) -> Result<(), String> {
    let revision_selector = format!("{LATTICE_REVISION}^{{commit}}");
    let output = checked_output(
        Command::new("git")
            .arg("-C")
            .arg(repository)
            .arg("rev-parse")
            .arg(revision_selector),
        "git revision verification",
    )?;
    exact_revision(&output)
}

fn extract_revision(repository: &Path, destination: &Path) -> Result<(), String> {
    let archive = destination.join("lattice.tar");
    checked_output(
        Command::new("git")
            .arg("-C")
            .arg(repository)
            .arg("archive")
            .arg("--format=tar")
            .arg(format!("--output={}", archive.display()))
            .arg(LATTICE_REVISION),
        "git archive",
    )?;
    checked_output(
        Command::new("tar")
            .arg("-xf")
            .arg(&archive)
            .arg("-C")
            .arg(destination),
        "archive extraction",
    )?;
    fs::remove_file(&archive)
        .map_err(|error| format!("cannot remove {}: {error}", archive.display()))?;
    Ok(())
}

fn select_disposable_linker_config(destination: &Path) -> Result<(), String> {
    let config_path = destination.join(CARGO_CONFIG_PATH);
    fs::write(&config_path, MINIMAL_CARGO_CONFIG)
        .map_err(|error| format!("cannot write {}: {error}", config_path.display()))
}

fn patch_dependency(destination: &Path, choregraph_root: &Path) -> Result<(), String> {
    let manifest_path = destination.join(MANIFEST_PATH);
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
    let adapter = choregraph_root
        .join(ADAPTER_PATH)
        .canonicalize()
        .map_err(|error| {
            format!(
                "cannot resolve Choregraph adapter under {}: {error}",
                choregraph_root.display()
            )
        })?;
    let replacement = dependency_replacement(&adapter)?;
    let patched = patched_manifest(&manifest, &replacement)?;
    fs::write(&manifest_path, patched)
        .map_err(|error| format!("cannot write {}: {error}", manifest_path.display()))
}

fn migrate_historical_runner_namespace(destination: &Path) -> Result<(), String> {
    let runner_path = destination.join(RUNNER_SOURCE_PATH);
    let mut source = fs::read_to_string(&runner_path)
        .map_err(|error| format!("cannot read {}: {error}", runner_path.display()))?;
    for &(old, new, expected_count) in RUNNER_NAMESPACE_MIGRATIONS {
        let observed_count = source.matches(old).count();
        if observed_count != expected_count {
            return Err(format!(
                "historical runner API migration for {LATTICE_REVISION}: expected {expected_count} occurrences of {old}, found {observed_count}"
            ));
        }
        source = source.replacen(old, new, expected_count);
    }
    fs::write(&runner_path, source)
        .map_err(|error| format!("cannot write {}: {error}", runner_path.display()))
}

fn run_runner_tests(destination: &Path) -> Result<(), String> {
    let status = Command::new("cargo")
        .current_dir(destination)
        .arg("test")
        .arg("--test")
        .arg(RUNNER_TEST)
        .status()
        .map_err(|error| format!("cannot run disposable Lattice tests: {error}"))?;
    if !status.success() {
        return Err(format!(
            "disposable Lattice runner tests failed with status {status}"
        ));
    }
    Ok(())
}

fn self_test() -> Result<(), String> {
    let replacement = dependency_replacement(Path::new("/tmp/choregraph-seaglass"))?;
    let positive = format!("[dependencies]\n{LEGACY_DEPENDENCY}\n");
    let patched = patched_manifest(&positive, &replacement)?;
    if !patched.contains(&replacement) || patched.contains(LEGACY_DEPENDENCY) {
        return Err(String::from("positive manifest replacement failed"));
    }
    if patched_manifest("[dependencies]\n", &replacement).is_ok() {
        return Err(String::from("missing dependency unexpectedly passed"));
    }
    let duplicate = format!("{LEGACY_DEPENDENCY}\n{LEGACY_DEPENDENCY}\n");
    if patched_manifest(&duplicate, &replacement).is_ok() {
        return Err(String::from("duplicate dependency unexpectedly passed"));
    }
    if dependency_replacement(Path::new("bad\"path")).is_ok() {
        return Err(String::from("unsafe adapter path unexpectedly passed"));
    }
    exact_revision(format!("{LATTICE_REVISION}\n").as_bytes())?;
    if exact_revision(b"floating-main\n").is_ok() {
        return Err(String::from("floating revision unexpectedly passed"));
    }
    Ok(())
}

fn run(arguments: &[String]) -> Result<(), String> {
    if arguments.len() == SELF_TEST_ARGUMENT_COUNT && arguments[1] == SELF_TEST_FLAG {
        return self_test();
    }
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err(String::from(
            "usage: check-lattice-downstream <lattice-git-repository> <choregraph-root>",
        ));
    }
    let lattice_repository = Path::new(&arguments[1]).canonicalize().map_err(|error| {
        format!(
            "cannot resolve Lattice repository {}: {error}",
            arguments[1]
        )
    })?;
    let choregraph_root = Path::new(&arguments[2])
        .canonicalize()
        .map_err(|error| format!("cannot resolve Choregraph root {}: {error}", arguments[2]))?;
    if !lattice_repository.join(".git").exists() {
        return Err(format!(
            "Lattice source is not a Git repository: {}",
            lattice_repository.display()
        ));
    }

    verify_revision(&lattice_repository)?;
    let disposable = DisposableRoot::create()?;
    extract_revision(&lattice_repository, &disposable.path)?;
    select_disposable_linker_config(&disposable.path)?;
    patch_dependency(&disposable.path, &choregraph_root)?;
    migrate_historical_runner_namespace(&disposable.path)?;
    run_runner_tests(&disposable.path)?;

    println!("lattice_revision={LATTICE_REVISION}");
    println!("adapter=choregraph-seaglass");
    println!("rust_api_migration=seaglass-semantic-owner-paths");
    println!("qualification=historical-runner-semantics-after-rust-api-migration");
    println!("runner_tests=passed");
    println!("source_fallback=none");
    Ok(())
}

fn main() -> ExitCode {
    let arguments = env::args().collect::<Vec<_>>();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
