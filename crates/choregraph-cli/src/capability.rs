// Capability I/O stays in this imperative shell; lowering and admission remain pure library work.
#![allow(
    unknown_lints,
    assertion_density,
    function_length,
    non_trait_imports,
    path_segment_repetition,
    reason = "the capability shell keeps explicit bounded I/O stages and local safety assertions visible"
)]

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use bounded_exec::{
    CommandSpec, Disposition, EnvironmentMode, ExecutionLimits, Input, OutcomePolicy, RunRequest,
    TerminationScope,
};
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_root_core::{AdmissionMode, RootRelativePath};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use choregraph_kamacite::{IdentityMaterials, build_artifact_set};
use choregraph_nickel::{
    DEFAULT_MAX_IMPORTS, EvaluatedSource, FrontendFailure, FrontendIssueCode, FrontendLimits,
    LoweredSourceFormat, ObservedImport, lower_evaluated_json, parse_evaluated_json,
    validate_restricted,
};

const CONTRACT_ENTRY_PATH: &str = "evaluated-graph-contract.ncl";
const CONTRACT_LIBRARY_PATH: &str = "restricted-graph-contracts.ncl";
const CONTRACT_ENTRY_BYTES: &[u8] =
    include_bytes!("../../../contracts/nickel/evaluated-graph-contract.ncl");
const CONTRACT_LIBRARY_BYTES: &[u8] =
    include_bytes!("../../../contracts/nickel/restricted-graph-contracts.ncl");
const PROOF_SET_BYTES: &[u8] =
    include_bytes!("../../../evidence/generated/trace-replay-proof.json");
const GENERATOR_BYTES: &[u8] = include_bytes!("capability.rs");
const ADAPTER_BYTES: &[u8] = include_bytes!("../../choregraph-kamacite/src/aggregate.rs");
const TOOLCHAIN_BYTES: &[u8] = include_bytes!("../../../rust-toolchain.toml");
const TEMP_FILE_SUFFIX: &str = ".choregraph-tmp";
const EVALUATOR_TIMEOUT_MS: u64 = 10_000;
const EVALUATOR_STDERR_MAX_BYTES: usize = 16_384;
const EVALUATOR_POLL_INTERVAL_MS: u64 = 10;
const EVALUATOR_TEARDOWN_TIMEOUT_MS: u64 = 2_000;
const SUCCESS_EXIT_CODE: i32 = 0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperatorRootSpec {
    pub id: String,
    pub ambient_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompileNickelRequest {
    pub evaluator: PathBuf,
    pub source_root: PathBuf,
    pub source_path: PathBuf,
    pub contract_root: PathBuf,
    pub scratch_root: PathBuf,
    pub output_root: PathBuf,
    pub output_path: PathBuf,
    pub import_roots: Vec<OperatorRootSpec>,
    pub limits: FrontendLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationReport {
    pub source_format: LoweredSourceFormat,
    pub import_count: usize,
    pub import_bytes: usize,
    pub machine_identity: String,
    pub artifact_bytes: usize,
}

struct OpenRoot {
    ambient_path: PathBuf,
    dir: Dir,
}

struct OpenRoots {
    source: OpenRoot,
    contract: OpenRoot,
    scratch: OpenRoot,
    output: OpenRoot,
    root_registry: BTreeMap<String, OpenRoot>,
}

struct LoadedImport {
    logical_id: String,
    bytes: Vec<u8>,
}

fn fail(code: FrontendIssueCode) -> FrontendFailure {
    FrontendFailure::one(code)
}

fn open_root(path: &Path) -> Result<OpenRoot, FrontendFailure> {
    if !path.is_absolute() {
        return Err(fail(FrontendIssueCode::CapabilityRootUnavailable));
    }
    let dir = Dir::open_ambient_dir(path, ambient_authority())
        .map_err(|_| fail(FrontendIssueCode::CapabilityRootUnavailable))?;
    Ok(OpenRoot {
        ambient_path: path.to_path_buf(),
        dir,
    })
}

fn open_roots(request: &CompileNickelRequest) -> Result<OpenRoots, FrontendFailure> {
    let source = open_root(&request.source_root)?;
    let contract = open_root(&request.contract_root)?;
    let scratch = open_root(&request.scratch_root)?;
    let output = open_root(&request.output_root)?;
    if request.import_roots.len() > DEFAULT_MAX_IMPORTS {
        return Err(fail(FrontendIssueCode::CapabilityRootUnavailable));
    }
    if request.import_roots.iter().any(|root| root.id.is_empty()) {
        return Err(fail(FrontendIssueCode::CapabilityRootUnavailable));
    }
    let root_entries = request
        .import_roots
        .iter()
        .map(|root| Ok((root.id.clone(), open_root(&root.ambient_path)?)))
        .collect::<Result<Vec<_>, FrontendFailure>>()?;
    let root_registry = root_entries.into_iter().collect::<BTreeMap<_, _>>();
    if root_registry.len() != request.import_roots.len() {
        return Err(fail(FrontendIssueCode::CapabilityRootUnavailable));
    }
    Ok(OpenRoots {
        source,
        contract,
        scratch,
        output,
        root_registry,
    })
}

fn admitted_path(path: &Path) -> Result<RootRelativePath, FrontendFailure> {
    RootRelativePath::new(path, AdmissionMode::NonEmptyRelative)
        .map_err(|_| fail(FrontendIssueCode::CapabilityPathRejected))
}

fn open_parent(
    root: &Dir,
    path: &RootRelativePath,
    failure_code: FrontendIssueCode,
) -> Result<(Dir, OsString), FrontendFailure> {
    let file_name = path
        .file_name()
        .map(OsStr::to_os_string)
        .ok_or_else(|| fail(FrontendIssueCode::CapabilityPathRejected))?;
    let mut parent = root.try_clone().map_err(|_| fail(failure_code))?;
    if let Some(parent_path) = path.parent() {
        for component in parent_path.components() {
            let Component::Normal(segment) = component else {
                return Err(fail(FrontendIssueCode::CapabilityPathRejected));
            };
            parent = cap_fs_ext::DirExt::open_dir_nofollow(&parent, segment)
                .map_err(|_| fail(failure_code))?;
        }
    }
    Ok((parent, file_name))
}

fn read_relative(
    root: &Dir,
    relative_path: &Path,
    max_bytes: usize,
) -> Result<Vec<u8>, FrontendFailure> {
    let admitted = admitted_path(relative_path)?;
    let (parent, file_name) = open_parent(
        root,
        &admitted,
        FrontendIssueCode::CapabilityInputUnavailable,
    )?;
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = parent
        .open_with(&file_name, &options)
        .map_err(|_| fail(FrontendIssueCode::CapabilityInputUnavailable))?;
    let metadata = file
        .metadata()
        .map_err(|_| fail(FrontendIssueCode::CapabilityInputUnavailable))?;
    if !metadata.is_file() {
        return Err(fail(FrontendIssueCode::CapabilityInputNotRegular));
    }
    let read_limit_bytes = max_bytes
        .checked_add(1)
        .ok_or_else(|| fail(FrontendIssueCode::CapabilityInputTooLarge))?;
    let read_limit_bytes = u64::try_from(read_limit_bytes)
        .map_err(|_| fail(FrontendIssueCode::CapabilityInputTooLarge))?;
    let mut bytes = Vec::new();
    file.take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(|_| fail(FrontendIssueCode::CapabilityInputUnavailable))?;
    if bytes.len() > max_bytes {
        return Err(fail(FrontendIssueCode::CapabilityInputTooLarge));
    }
    Ok(bytes)
}

fn verify_contract(root: &OpenRoot) -> Result<PathBuf, FrontendFailure> {
    let entry = read_relative(
        &root.dir,
        Path::new(CONTRACT_ENTRY_PATH),
        CONTRACT_ENTRY_BYTES.len(),
    )?;
    let library = read_relative(
        &root.dir,
        Path::new(CONTRACT_LIBRARY_PATH),
        CONTRACT_LIBRARY_BYTES.len(),
    )?;
    if entry != CONTRACT_ENTRY_BYTES || library != CONTRACT_LIBRARY_BYTES {
        return Err(fail(FrontendIssueCode::EvaluatorExecutionRejected));
    }
    Ok(root.ambient_path.join(CONTRACT_ENTRY_PATH))
}

fn evaluator_limits(frontend_limits: FrontendLimits) -> ExecutionLimits {
    ExecutionLimits {
        timeout_ms: EVALUATOR_TIMEOUT_MS,
        stdin_max_bytes: frontend_limits.max_evaluated_bytes,
        stdout_max_bytes: frontend_limits.max_evaluated_bytes,
        stderr_max_bytes: EVALUATOR_STDERR_MAX_BYTES,
        poll_interval_ms: EVALUATOR_POLL_INTERVAL_MS,
        teardown_timeout_ms: EVALUATOR_TEARDOWN_TIMEOUT_MS,
    }
}

fn run_evaluator(
    request: &CompileNickelRequest,
    roots: &OpenRoots,
    source: &[u8],
) -> Result<Vec<u8>, FrontendFailure> {
    let contract_path = verify_contract(&roots.contract)?;
    let policy = OutcomePolicy::new(vec![SUCCESS_EXIT_CODE], true, true)
        .map_err(|_| fail(FrontendIssueCode::EvaluatorExecutionRejected))?;
    let execution = bounded_exec::run(RunRequest {
        command: CommandSpec {
            program: request.evaluator.clone(),
            args: vec![
                OsString::from("export"),
                OsString::from("--format"),
                OsString::from("json"),
                OsString::from("--color"),
                OsString::from("never"),
                OsString::from("--error-format"),
                OsString::from("text"),
                OsString::from("--apply-contract"),
                contract_path.into_os_string(),
            ],
            current_dir: roots.scratch.ambient_path.clone(),
            environment_mode: EnvironmentMode::Clear,
            environment: Vec::new(),
            input: Input::Bytes(source.to_vec()),
        },
        limits: evaluator_limits(request.limits),
        termination_scope: TerminationScope::ProcessGroup,
        outcome_policy: policy,
    })
    .map_err(|_| fail(FrontendIssueCode::EvaluatorExecutionRejected))?;
    if execution.disposition != Disposition::Succeeded {
        return Err(fail(FrontendIssueCode::EvaluatorExecutionRejected));
    }
    if execution.stdout.bytes.is_empty() || execution.stdout.truncated || execution.stderr.truncated
    {
        return Err(fail(FrontendIssueCode::EvaluatorOutputRejected));
    }
    Ok(execution.stdout.bytes)
}

fn load_imports(
    source: &EvaluatedSource,
    roots: &OpenRoots,
    limits: FrontendLimits,
) -> Result<Vec<LoadedImport>, FrontendFailure> {
    let mut loaded = Vec::with_capacity(source.imports().len());
    for declaration in source.imports() {
        let root = roots
            .root_registry
            .get(&declaration.root_id)
            .ok_or_else(|| fail(FrontendIssueCode::UnknownCapabilityRoot))?;
        let declared_max = usize::try_from(declaration.max_bytes)
            .map_err(|_| fail(FrontendIssueCode::CapabilityInputTooLarge))?;
        let selected_max = declared_max.min(limits.max_import_bytes);
        let bytes = read_relative(
            &root.dir,
            Path::new(&declaration.relative_path),
            selected_max,
        )?;
        loaded.push(LoadedImport {
            logical_id: declaration.logical_id.clone(),
            bytes,
        });
    }
    Ok(loaded)
}

fn observations(imports: &[LoadedImport]) -> Vec<ObservedImport<'_>> {
    imports
        .iter()
        .map(|import| ObservedImport {
            logical_id: &import.logical_id,
            bytes: &import.bytes,
        })
        .collect()
}

fn temporary_name(file_name: &OsStr) -> OsString {
    let mut temporary = OsString::from(".");
    temporary.push(file_name);
    temporary.push(TEMP_FILE_SUFFIX);
    temporary
}

fn remove_if_present(parent: &Dir, path: &OsStr) {
    let _ignored = parent.remove_file(path);
}

fn publish_atomic(
    output_root: &Dir,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), FrontendFailure> {
    let admitted = admitted_path(relative_path)?;
    let (parent, file_name) = open_parent(
        output_root,
        &admitted,
        FrontendIssueCode::CapabilityOutputUnavailable,
    )?;
    if parent.symlink_metadata(&file_name).is_ok() {
        return Err(fail(FrontendIssueCode::CapabilityOutputExists));
    }
    let temporary = temporary_name(&file_name);
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent.open_with(&temporary, &options).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            fail(FrontendIssueCode::CapabilityOutputExists)
        } else {
            fail(FrontendIssueCode::CapabilityOutputUnavailable)
        }
    })?;
    if file.write_all(bytes).is_err() || file.sync_all().is_err() {
        remove_if_present(&parent, &temporary);
        return Err(fail(FrontendIssueCode::AtomicPublicationRejected));
    }
    if let Err(error) = parent.hard_link(&temporary, &parent, &file_name) {
        remove_if_present(&parent, &temporary);
        let code = if error.kind() == std::io::ErrorKind::AlreadyExists {
            FrontendIssueCode::CapabilityOutputExists
        } else {
            FrontendIssueCode::AtomicPublicationRejected
        };
        return Err(fail(code));
    }
    if parent.remove_file(&temporary).is_err() {
        remove_if_present(&parent, &file_name);
        remove_if_present(&parent, &temporary);
        return Err(fail(FrontendIssueCode::AtomicPublicationRejected));
    }
    Ok(())
}

fn compile_with_evaluator<F>(
    request: &CompileNickelRequest,
    evaluator: F,
) -> Result<CompilationReport, FrontendFailure>
where
    F: FnOnce(&CompileNickelRequest, &OpenRoots, &[u8]) -> Result<Vec<u8>, FrontendFailure>,
{
    let roots = open_roots(request)?;
    let source = read_relative(
        &roots.source.dir,
        &request.source_path,
        request.limits.max_evaluated_bytes,
    )?;
    let max_evaluated_bytes = u64::try_from(request.limits.max_evaluated_bytes)
        .map_err(|_| fail(FrontendIssueCode::NickelSourceTooLarge))?;
    validate_restricted(&source, max_evaluated_bytes)?;
    let evaluated = evaluator(request, &roots, &source)?;
    let parsed = parse_evaluated_json(&evaluated, max_evaluated_bytes)?;
    let closure_inputs = load_imports(&parsed, &roots, request.limits)?;
    let observed = observations(&closure_inputs);
    let lowered = lower_evaluated_json(&evaluated, &observed, request.limits)?;
    let artifacts = build_artifact_set(
        &lowered.machine,
        IdentityMaterials {
            source_envelope: &source,
            proof_set: PROOF_SET_BYTES,
            generator: GENERATOR_BYTES,
            adapter: ADAPTER_BYTES,
            toolchain: TOOLCHAIN_BYTES,
        },
    )
    .map_err(|_| fail(FrontendIssueCode::ArtifactConstructionRejected))?;
    publish_atomic(
        &roots.output.dir,
        &request.output_path,
        &artifacts.machine_artifact.packed_bytes,
    )?;
    Ok(CompilationReport {
        source_format: lowered.source_format,
        import_count: lowered.import_closure.import_count,
        import_bytes: lowered.import_closure.total_bytes,
        machine_identity: artifacts.identities.machine.hex,
        artifact_bytes: artifacts.machine_artifact.packed_bytes.len(),
    })
}

pub fn compile_nickel(
    request: &CompileNickelRequest,
) -> Result<CompilationReport, FrontendFailure> {
    compile_with_evaluator(request, run_evaluator)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::{
        CompileNickelRequest, OperatorRootSpec, compile_with_evaluator, fail, read_relative,
    };
    use choregraph_nickel::{FrontendIssueCode, LoweredSourceFormat};

    const SOURCE_PATH: &str = "graph/source.ncl";
    const OUTPUT_PATH: &str = "artifacts/machine.preserves";
    const IMPORT_PATH: &str = "data/defaults.ncl";
    const MINIMAL_SOURCE: &[u8] = include_bytes!("../../../fixtures/nickel/positive/minimal.ncl");
    const MINIMAL_JSON: &[u8] = include_bytes!("../../../fixtures/nickel/evaluated/minimal.json");
    const IMPORT_JSON: &[u8] =
        include_bytes!("../../../fixtures/nickel/evaluated/declared-import.json");

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("expected success, got {error:?}"),
        }
    }

    fn must_err<T: std::fmt::Debug, E>(result: Result<T, E>) -> E {
        match result {
            Ok(value) => panic!("expected failure, got {value:?}"),
            Err(error) => error,
        }
    }

    struct Fixture {
        temporary: TempDir,
        source_root: std::path::PathBuf,
        contract_root: std::path::PathBuf,
        scratch_root: std::path::PathBuf,
        output_root: std::path::PathBuf,
    }

    fn fixture() -> Fixture {
        let temporary = must(TempDir::new());
        let source_root = temporary.path().join("source");
        let contract_root = temporary.path().join("contract");
        let scratch_root = temporary.path().join("scratch");
        let output_root = temporary.path().join("output");
        must(fs::create_dir_all(source_root.join("graph")));
        must(fs::create_dir_all(source_root.join("data")));
        must(fs::create_dir_all(&contract_root));
        must(fs::create_dir_all(&scratch_root));
        must(fs::create_dir_all(output_root.join("artifacts")));
        must(fs::write(source_root.join(SOURCE_PATH), MINIMAL_SOURCE));
        must(fs::write(source_root.join(IMPORT_PATH), []));
        must(fs::write(
            contract_root.join(super::CONTRACT_ENTRY_PATH),
            super::CONTRACT_ENTRY_BYTES,
        ));
        must(fs::write(
            contract_root.join(super::CONTRACT_LIBRARY_PATH),
            super::CONTRACT_LIBRARY_BYTES,
        ));
        Fixture {
            temporary,
            source_root,
            contract_root,
            scratch_root,
            output_root,
        }
    }

    fn request(fixture: &Fixture) -> CompileNickelRequest {
        CompileNickelRequest {
            evaluator: Path::new("/bin/false").to_path_buf(),
            source_root: fixture.source_root.clone(),
            source_path: Path::new(SOURCE_PATH).to_path_buf(),
            contract_root: fixture.contract_root.clone(),
            scratch_root: fixture.scratch_root.clone(),
            output_root: fixture.output_root.clone(),
            output_path: Path::new(OUTPUT_PATH).to_path_buf(),
            import_roots: vec![OperatorRootSpec {
                id: String::from("source"),
                ambient_path: fixture.source_root.clone(),
            }],
            limits: choregraph_nickel::reviewed_frontend_limits(),
        }
    }

    #[test]
    fn compiles_capability_relative_source_without_dispatch_authority() {
        let fixture = fixture();
        let request = request(&fixture);
        let compilation_result = must(compile_with_evaluator(
            &request,
            |_request, _roots, source| {
                assert_eq!(source, MINIMAL_SOURCE);
                Ok(MINIMAL_JSON.to_vec())
            },
        ));
        assert_eq!(
            compilation_result.source_format,
            LoweredSourceFormat::ChoregraphV1
        );
        assert_eq!(compilation_result.import_count, 0);
        assert_eq!(compilation_result.import_bytes, 0);
        assert!(!compilation_result.machine_identity.is_empty());
        assert!(compilation_result.artifact_bytes > 0);
        let artifact = must(fs::read(fixture.output_root.join(OUTPUT_PATH)));
        assert_eq!(artifact.len(), compilation_result.artifact_bytes);
    }

    #[test]
    fn loads_declared_imports_from_operator_selected_roots() {
        let fixture = fixture();
        let request = request(&fixture);
        let compilation_result = must(compile_with_evaluator(
            &request,
            |_request, _roots, _source| Ok(IMPORT_JSON.to_vec()),
        ));
        assert_eq!(compilation_result.import_count, 1);
        assert_eq!(compilation_result.import_bytes, 0);
    }

    #[test]
    fn rejects_ambient_graph_paths_before_import_io() {
        let fixture = fixture();
        let request = request(&fixture);
        let mut value: serde_json::Value = must(serde_json::from_slice(IMPORT_JSON));
        value["imports"][0]["relative_path"] =
            serde_json::Value::String(String::from("/etc/passwd"));
        let evaluated = must(serde_json::to_vec(&value));
        let failure = must_err(compile_with_evaluator(
            &request,
            |_request, _roots, _source| Ok(evaluated),
        ));
        assert_eq!(
            failure.issues[0].code(),
            FrontendIssueCode::CapabilityPathRejected
        );
        assert!(!fixture.output_root.join(OUTPUT_PATH).exists());
    }

    #[test]
    fn rejects_unknown_roots_and_existing_outputs() {
        let unknown_root_fixture = fixture();
        let mut unknown_root_request = request(&unknown_root_fixture);
        unknown_root_request.import_roots.clear();
        let failure = must_err(compile_with_evaluator(
            &unknown_root_request,
            |_request, _roots, _source| Ok(IMPORT_JSON.to_vec()),
        ));
        assert_eq!(
            failure.issues[0].code(),
            FrontendIssueCode::UnknownCapabilityRoot
        );

        let fixture = fixture();
        let request = request(&fixture);
        must(fs::write(
            fixture.output_root.join(OUTPUT_PATH),
            b"existing",
        ));
        let failure = must_err(compile_with_evaluator(
            &request,
            |_request, _roots, _source| Ok(MINIMAL_JSON.to_vec()),
        ));
        assert_eq!(
            failure.issues[0].code(),
            FrontendIssueCode::CapabilityOutputExists
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_inputs_without_reading_targets() {
        use std::os::unix::fs::symlink;

        let fixture = fixture();
        let outside = fixture.temporary.path().join("outside.ncl");
        must(fs::write(&outside, MINIMAL_SOURCE));
        must(fs::remove_file(fixture.source_root.join(SOURCE_PATH)));
        must(symlink(&outside, fixture.source_root.join(SOURCE_PATH)));
        let request = request(&fixture);
        let failure = must_err(compile_with_evaluator(
            &request,
            |_request, _roots, _source| Err(fail(FrontendIssueCode::EvaluatorExecutionRejected)),
        ));
        assert_eq!(
            failure.issues[0].code(),
            FrontendIssueCode::CapabilityInputUnavailable
        );
    }

    #[test]
    fn bounded_reads_reject_oversized_files() {
        let fixture = fixture();
        let root = must(cap_std::fs::Dir::open_ambient_dir(
            &fixture.source_root,
            cap_std::ambient_authority(),
        ));
        let failure = must_err(read_relative(&root, Path::new(SOURCE_PATH), 1));
        assert_eq!(
            failure.issues[0].code(),
            FrontendIssueCode::CapabilityInputTooLarge
        );
    }
}
