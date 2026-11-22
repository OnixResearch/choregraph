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
use choregraph_kamacite::aggregate::assembly::{IdentityMaterials, build_artifact_set};

const CONTRACT_ENTRY_PATH: &str = "evaluated-graph-contract.ncl";
const CONTRACT_LIBRARY_PATH: &str = "restricted-graph-contracts.ncl";
const CONTRACT_ENTRY_BYTES: &[u8] =
    include_bytes!("../../../contracts/nickel/evaluated-graph-contract.ncl");
const CONTRACT_LIBRARY_BYTES: &[u8] =
    include_bytes!("../../../contracts/nickel/restricted-graph-contracts.ncl");
const PROOF_SET_BYTES: &[u8] =
    include_bytes!("../../../evidence/generated/trace-replay-proof.json");
const GENERATOR_BYTES: &[u8] = include_bytes!("capability.rs");
const TOOLCHAIN_BYTES: &[u8] = include_bytes!("../../../rust-toolchain.toml");
const TEMP_FILE_SUFFIX: &str = ".choregraph-tmp";
const EVALUATOR_TIMEOUT_MS: u64 = 10_000;
const EVALUATOR_STDERR_MAX_BYTES: usize = 16_384;
const EVALUATOR_POLL_INTERVAL_MS: u64 = 10;
const EVALUATOR_TEARDOWN_TIMEOUT_MS: u64 = 2_000;
const SUCCESS_EXIT_CODE: i32 = 0;

// These are compiler inputs supplied together by the Linux flake package/dev shell,
// never runtime environment overrides. The manifest lists individual ELF files.
const PINNED_NICKEL: Option<&str> = option_env!("CHOREGRAPH_NICKEL_EVALUATOR");
const PINNED_BASH: Option<&str> = option_env!("CHOREGRAPH_NICKEL_BASH");
const PINNED_BWRAP: Option<&str> = option_env!("CHOREGRAPH_NICKEL_BWRAP");
const PINNED_RUNTIME: Option<&str> = option_env!("CHOREGRAPH_NICKEL_RUNTIME");
const SANDBOX_SCRIPT: &str = r#"
set -eu
bwrap=$1
nickel=$2
readarray -t runtime < "$3"
(( ${#runtime[@]} > 0 ))
mounts=(--ro-bind "$nickel" "$nickel")
for file in "${runtime[@]}"; do
    [[ $file == /nix/store/* && -f $file ]]
    mounts+=(--ro-bind "$file" "$file")
done
# Bash here-strings restore exactly the final LF stripped by contract_argument.
# Bubblewrap copies those descriptors into its own read-only mounts, not host paths.
exec "$bwrap" --unshare-user --disable-userns --unshare-ipc --unshare-pid \
    --unshare-net --unshare-uts --hostname choregraph --die-with-parent --cap-drop ALL --clearenv \
    "${mounts[@]}" --dir /work --chdir /work \
    --perms 0444 --ro-bind-data 3 /contracts/evaluated-graph-contract.ncl \
    --perms 0444 --ro-bind-data 4 /contracts/restricted-graph-contracts.ncl --remount-ro / \
    -- "$nickel" export --format json --color never --error-format text \
    --apply-contract /contracts/evaluated-graph-contract.ncl 3<<< "$4" 4<<< "$5"
"#;

type FrontendFailure = choregraph_nickel::diagnostic::FrontendFailure;

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
    pub limits: choregraph_nickel::profile::FrontendLimits,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilationReport {
    pub source_format: choregraph_nickel::lower::LoweredSourceFormat,
    pub import_count: usize,
    pub import_bytes: usize,
    pub machine_identity: String,
    pub artifact_bytes: usize,
}

struct OpenRoots {
    source: Dir,
    contract: Dir,
    output: Dir,
    root_registry: BTreeMap<String, Dir>,
}

struct LoadedImport {
    logical_id: String,
    bytes: Vec<u8>,
}

fn fail(
    code: choregraph_nickel::diagnostic::FrontendIssueCode,
) -> choregraph_nickel::diagnostic::FrontendFailure {
    choregraph_nickel::diagnostic::FrontendFailure::one(code)
}

pub(crate) fn open_capability_dir(path: &Path) -> Result<Dir, FrontendFailure> {
    if !path.is_absolute() {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityRootUnavailable,
        ));
    }
    Dir::open_ambient_dir(path, ambient_authority()).map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityRootUnavailable)
    })
}

fn open_roots(
    request: &CompileNickelRequest,
) -> Result<OpenRoots, choregraph_nickel::diagnostic::FrontendFailure> {
    let source = open_capability_dir(&request.source_root)?;
    let contract = open_capability_dir(&request.contract_root)?;
    // Keep checking the legacy operator argument, but never expose it to the evaluator.
    drop(open_capability_dir(&request.scratch_root)?);
    let output = open_capability_dir(&request.output_root)?;
    if request.import_roots.len() > choregraph_nickel::profile::DEFAULT_MAX_IMPORTS {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityRootUnavailable,
        ));
    }
    if request.import_roots.iter().any(|root| root.id.is_empty()) {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityRootUnavailable,
        ));
    }
    let root_entries = request
        .import_roots
        .iter()
        .map(|root| Ok((root.id.clone(), open_capability_dir(&root.ambient_path)?)))
        .collect::<Result<Vec<_>, choregraph_nickel::diagnostic::FrontendFailure>>()?;
    let root_registry = root_entries.into_iter().collect::<BTreeMap<_, _>>();
    if root_registry.len() != request.import_roots.len() {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityRootUnavailable,
        ));
    }
    Ok(OpenRoots {
        source,
        contract,
        output,
        root_registry,
    })
}

fn admitted_path(
    path: &Path,
) -> Result<RootRelativePath, choregraph_nickel::diagnostic::FrontendFailure> {
    RootRelativePath::new(path, AdmissionMode::NonEmptyRelative)
        .map_err(|_| fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityPathRejected))
}

fn open_parent(
    root: &Dir,
    path: &RootRelativePath,
    failure_code: choregraph_nickel::diagnostic::FrontendIssueCode,
) -> Result<(Dir, OsString), choregraph_nickel::diagnostic::FrontendFailure> {
    let file_name = path.file_name().map(OsStr::to_os_string).ok_or_else(|| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityPathRejected)
    })?;
    let mut parent = root.try_clone().map_err(|_| fail(failure_code))?;
    if let Some(parent_path) = path.parent() {
        for component in parent_path.components() {
            let Component::Normal(segment) = component else {
                return Err(fail(
                    choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityPathRejected,
                ));
            };
            parent = cap_fs_ext::DirExt::open_dir_nofollow(&parent, segment)
                .map_err(|_| fail(failure_code))?;
        }
    }
    Ok((parent, file_name))
}

pub(crate) fn read_relative(
    root: &Dir,
    relative_path: &Path,
    max_bytes: usize,
) -> Result<Vec<u8>, choregraph_nickel::diagnostic::FrontendFailure> {
    let admitted = admitted_path(relative_path)?;
    let (parent, file_name) = open_parent(
        root,
        &admitted,
        choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputUnavailable,
    )?;
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = parent.open_with(&file_name, &options).map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputUnavailable)
    })?;
    let metadata = file.metadata().map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputUnavailable)
    })?;
    if !metadata.is_file() {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputNotRegular,
        ));
    }
    let read_limit_bytes = max_bytes.checked_add(1).ok_or_else(|| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputTooLarge)
    })?;
    let read_limit_bytes = u64::try_from(read_limit_bytes).map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputTooLarge)
    })?;
    let mut bytes = Vec::new();
    file.take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(|_| {
            fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputUnavailable)
        })?;
    if bytes.len() > max_bytes {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputTooLarge,
        ));
    }
    Ok(bytes)
}

fn verify_contract(root: &Dir) -> Result<(), FrontendFailure> {
    let entry = read_relative(
        root,
        Path::new(CONTRACT_ENTRY_PATH),
        CONTRACT_ENTRY_BYTES.len(),
    )?;
    let library = read_relative(
        root,
        Path::new(CONTRACT_LIBRARY_PATH),
        CONTRACT_LIBRARY_BYTES.len(),
    )?;
    if entry != CONTRACT_ENTRY_BYTES || library != CONTRACT_LIBRARY_BYTES {
        return Err(evaluator_rejected());
    }
    Ok(())
}

fn evaluator_limits(
    frontend_limits: choregraph_nickel::profile::FrontendLimits,
) -> ExecutionLimits {
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
) -> Result<Vec<u8>, choregraph_nickel::diagnostic::FrontendFailure> {
    verify_contract(&roots.contract)?;
    run_sandboxed(&request.evaluator, request.limits, source)
}

fn evaluator_rejected() -> FrontendFailure {
    fail(choregraph_nickel::diagnostic::FrontendIssueCode::EvaluatorExecutionRejected)
}

fn contract_argument(bytes: &[u8]) -> Result<&str, FrontendFailure> {
    let text = std::str::from_utf8(bytes).map_err(|_| evaluator_rejected())?;
    if text.as_bytes().contains(&0) {
        return Err(evaluator_rejected());
    }
    text.strip_suffix('\n').ok_or_else(evaluator_rejected)
}

fn sandbox_command(evaluator: &Path, source: &[u8]) -> Result<CommandSpec, FrontendFailure> {
    let (true, Some(nickel), Some(bash), Some(bwrap), Some(runtime)) = (
        cfg!(target_os = "linux"),
        PINNED_NICKEL,
        PINNED_BASH,
        PINNED_BWRAP,
        PINNED_RUNTIME,
    ) else {
        return Err(evaluator_rejected());
    };
    if evaluator != Path::new("--pinned")
        && (!evaluator.is_absolute()
            || std::fs::canonicalize(evaluator).ok().as_deref() != Some(Path::new(nickel)))
    {
        return Err(evaluator_rejected());
    }
    Ok(CommandSpec {
        program: PathBuf::from(bash),
        args: [
            "--noprofile",
            "--norc",
            "-c",
            SANDBOX_SCRIPT,
            "choregraph-nickel-sandbox",
            bwrap,
            nickel,
            runtime,
            contract_argument(CONTRACT_ENTRY_BYTES)?,
            contract_argument(CONTRACT_LIBRARY_BYTES)?,
        ]
        .into_iter()
        .map(OsString::from)
        .collect(),
        current_dir: PathBuf::from("/"),
        environment_mode: EnvironmentMode::Clear,
        environment: Vec::new(),
        input: Input::Bytes(source.to_vec()),
    })
}

fn run_sandboxed(
    evaluator: &Path,
    limits: choregraph_nickel::profile::FrontendLimits,
    source: &[u8],
) -> Result<Vec<u8>, FrontendFailure> {
    let command = sandbox_command(evaluator, source)?;
    let policy = OutcomePolicy::new(vec![SUCCESS_EXIT_CODE], true, true)
        .map_err(|_| evaluator_rejected())?;
    let execution = bounded_exec::run(RunRequest {
        command,
        limits: evaluator_limits(limits),
        termination_scope: TerminationScope::ProcessGroup,
        outcome_policy: policy,
    })
    .map_err(|_| evaluator_rejected())?;
    if execution.disposition != Disposition::Succeeded {
        return Err(evaluator_rejected());
    }
    if execution.stdout.bytes.is_empty() || execution.stdout.truncated || execution.stderr.truncated
    {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::EvaluatorOutputRejected,
        ));
    }
    Ok(execution.stdout.bytes)
}

fn load_imports(
    source: &choregraph_nickel::model::admission::EvaluatedSource,
    roots: &OpenRoots,
    limits: choregraph_nickel::profile::FrontendLimits,
) -> Result<Vec<LoadedImport>, choregraph_nickel::diagnostic::FrontendFailure> {
    let declarations = source.imports()?;
    let mut loaded = Vec::with_capacity(declarations.len());
    for declaration in declarations {
        let root = roots
            .root_registry
            .get(&declaration.root_id)
            .ok_or_else(|| {
                fail(choregraph_nickel::diagnostic::FrontendIssueCode::UnknownCapabilityRoot)
            })?;
        let declared_max = usize::try_from(declaration.max_bytes).map_err(|_| {
            fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputTooLarge)
        })?;
        let selected_max = declared_max.min(limits.max_import_bytes);
        let bytes = read_relative(root, Path::new(&declaration.relative_path), selected_max)?;
        loaded.push(LoadedImport {
            logical_id: declaration.logical_id.clone(),
            bytes,
        });
    }
    Ok(loaded)
}

fn observations(imports: &[LoadedImport]) -> Vec<choregraph_nickel::imports::ObservedImport<'_>> {
    imports
        .iter()
        .map(|import| choregraph_nickel::imports::ObservedImport {
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

pub(crate) fn publish_atomic(
    output_root: &Dir,
    relative_path: &Path,
    bytes: &[u8],
) -> Result<(), choregraph_nickel::diagnostic::FrontendFailure> {
    let admitted = admitted_path(relative_path)?;
    let (parent, file_name) = open_parent(
        output_root,
        &admitted,
        choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityOutputUnavailable,
    )?;
    if parent.symlink_metadata(&file_name).is_ok() {
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityOutputExists,
        ));
    }
    let temporary = temporary_name(&file_name);
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut file = parent.open_with(&temporary, &options).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityOutputExists)
        } else {
            fail(choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityOutputUnavailable)
        }
    })?;
    if file.write_all(bytes).is_err() || file.sync_all().is_err() {
        remove_if_present(&parent, &temporary);
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::AtomicPublicationRejected,
        ));
    }
    if let Err(error) = parent.hard_link(&temporary, &parent, &file_name) {
        remove_if_present(&parent, &temporary);
        let code = if error.kind() == std::io::ErrorKind::AlreadyExists {
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityOutputExists
        } else {
            choregraph_nickel::diagnostic::FrontendIssueCode::AtomicPublicationRejected
        };
        return Err(fail(code));
    }
    if parent.remove_file(&temporary).is_err() {
        remove_if_present(&parent, &file_name);
        remove_if_present(&parent, &temporary);
        return Err(fail(
            choregraph_nickel::diagnostic::FrontendIssueCode::AtomicPublicationRejected,
        ));
    }
    Ok(())
}

fn compile_with_evaluator<F>(
    request: &CompileNickelRequest,
    evaluator: F,
) -> Result<CompilationReport, choregraph_nickel::diagnostic::FrontendFailure>
where
    F: FnOnce(
        &CompileNickelRequest,
        &OpenRoots,
        &[u8],
    ) -> Result<Vec<u8>, choregraph_nickel::diagnostic::FrontendFailure>,
{
    let roots = open_roots(request)?;
    let source = read_relative(
        &roots.source,
        &request.source_path,
        request.limits.max_evaluated_bytes,
    )?;
    let max_evaluated_bytes = u64::try_from(request.limits.max_evaluated_bytes).map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::NickelSourceTooLarge)
    })?;
    choregraph_nickel::source::validate_restricted(&source, max_evaluated_bytes)?;
    let evaluated = evaluator(request, &roots, &source)?;
    let parsed = choregraph_nickel::lower::parse_evaluated_json(&evaluated, max_evaluated_bytes)?;
    let closure_inputs = load_imports(&parsed, &roots, request.limits)?;
    let observed = observations(&closure_inputs);
    let lowered =
        choregraph_nickel::lower::lower_evaluated_json(&evaluated, &observed, request.limits)?;
    let adapter_material = crate::corpus::adapter_material().map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::ArtifactConstructionRejected)
    })?;
    let artifacts = build_artifact_set(
        &lowered.machine,
        IdentityMaterials {
            source_envelope: &source,
            proof_set: PROOF_SET_BYTES,
            generator: GENERATOR_BYTES,
            adapter: &adapter_material,
            toolchain: TOOLCHAIN_BYTES,
        },
    )
    .map_err(|_| {
        fail(choregraph_nickel::diagnostic::FrontendIssueCode::ArtifactConstructionRejected)
    })?;
    publish_atomic(
        &roots.output,
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
) -> Result<CompilationReport, choregraph_nickel::diagnostic::FrontendFailure> {
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

    const SOURCE_PATH: &str = "graph/source.ncl";
    const OUTPUT_PATH: &str = "artifacts/machine.preserves";
    const IMPORT_PATH: &str = "data/defaults.ncl";
    const MINIMAL_SOURCE: &[u8] = include_bytes!("../../../fixtures/nickel/positive/minimal.ncl");
    const MINIMAL_JSON: &[u8] = include_bytes!("../../../fixtures/nickel/evaluated/minimal.json");
    #[cfg(target_os = "linux")]
    const IMPORT_SOURCE: &[u8] =
        include_bytes!("../../../fixtures/nickel/positive/declared-import.ncl");
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
            limits: choregraph_nickel::profile::reviewed_frontend_limits(),
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
            choregraph_nickel::lower::LoweredSourceFormat::ChoregraphV1
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
    fn rejects_non_profile_sources_before_the_evaluator() {
        let fixture = fixture();
        must(fs::write(
            fixture.source_root.join(SOURCE_PATH),
            br#"{ value = m%"plain text"% }"#,
        ));
        let failure = must_err(compile_with_evaluator(&request(&fixture), |_, _, _| {
            panic!("source rejected by the profile reached evaluator setup")
        }));
        assert_eq!(
            failure.issues[0].code(),
            choregraph_nickel::diagnostic::FrontendIssueCode::ForbiddenNickelAuthority
        );
        assert!(!fixture.output_root.join(OUTPUT_PATH).exists());
    }

    #[test]
    fn rejects_unpinned_executable_without_publishing() {
        let fixture = fixture();
        assert_eq!(
            must_err(super::compile_nickel(&request(&fixture))),
            super::evaluator_rejected()
        );
        assert!(!fixture.output_root.join(OUTPUT_PATH).exists());
    }

    #[test]
    fn pinned_request_fails_closed_without_build_configuration() {
        if cfg!(target_os = "linux")
            && [
                super::PINNED_NICKEL,
                super::PINNED_BASH,
                super::PINNED_BWRAP,
                super::PINNED_RUNTIME,
            ]
            .iter()
            .all(Option::is_some)
        {
            return;
        }
        let fixture = fixture();
        let mut request = request(&fixture);
        request.evaluator = Path::new("--pinned").to_path_buf();
        assert_eq!(
            must_err(super::compile_nickel(&request)),
            super::evaluator_rejected()
        );
        assert!(!fixture.output_root.join(OUTPUT_PATH).exists());
    }

    #[test]
    fn detects_changed_contract_bytes() {
        let fixture = fixture();
        let root = must(super::open_capability_dir(&fixture.contract_root));
        must(super::verify_contract(&root));
        for name in [super::CONTRACT_ENTRY_PATH, super::CONTRACT_LIBRARY_PATH] {
            let path = fixture.contract_root.join(name);
            let original = must(fs::read(&path));
            let mut changed = original.clone();
            changed[0] ^= 1;
            must(fs::write(&path, &changed));
            assert_eq!(
                must_err(super::verify_contract(&root)),
                super::evaluator_rejected()
            );
            must(fs::write(&path, &original));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires the pinned Nix runtime and a Linux host that allows user namespaces"]
    fn linux_host_sandbox_acceptance() {
        for (source, json, import_count) in [
            (MINIMAL_SOURCE, MINIMAL_JSON, 0),
            (IMPORT_SOURCE, IMPORT_JSON, 1),
        ] {
            let fixture = fixture();
            must(fs::write(fixture.source_root.join(SOURCE_PATH), source));
            let mut request = request(&fixture);
            request.evaluator = Path::new("--pinned").to_path_buf();
            let expected = must(compile_with_evaluator(
                &request,
                |_, _, _| Ok(json.to_vec()),
            ));
            let expected_bytes = must(fs::read(fixture.output_root.join(OUTPUT_PATH)));
            must(fs::remove_file(fixture.output_root.join(OUTPUT_PATH)));
            let actual = must(super::compile_nickel(&request));
            assert_eq!(actual, expected);
            assert_eq!(actual.import_count, import_count);
            assert_eq!(
                must(fs::read(fixture.output_root.join(OUTPUT_PATH))),
                expected_bytes
            );
        }

        let fixture = fixture();
        let root = must(super::open_capability_dir(&fixture.contract_root));
        must(super::verify_contract(&root));
        must(fs::remove_file(
            fixture.contract_root.join(super::CONTRACT_ENTRY_PATH),
        ));
        must(fs::remove_file(
            fixture.contract_root.join(super::CONTRACT_LIBRARY_PATH),
        ));
        let evaluator = Path::new("--pinned");
        let limits = request(&fixture).limits;
        must(super::run_sandboxed(evaluator, limits, MINIMAL_SOURCE));

        // A harmless scratch fixture is readable by the test but not the evaluator.
        let scratch_source = fixture.scratch_root.join("sandbox-probe.ncl");
        must(fs::write(&scratch_source, MINIMAL_SOURCE));
        let path = must(
            scratch_source
                .to_str()
                .ok_or("non-UTF-8 scratch fixture path"),
        );
        let quoted_path = must(serde_json::to_string(path));
        let source_read = format!("import {quoted_path}");
        let denied = super::run_sandboxed(evaluator, limits, source_read.as_bytes());
        assert_eq!(must_err(denied), super::evaluator_rejected());
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
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityPathRejected
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
            choregraph_nickel::diagnostic::FrontendIssueCode::UnknownCapabilityRoot
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
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityOutputExists
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
            |_request, _roots, _source| {
                Err(fail(
                    choregraph_nickel::diagnostic::FrontendIssueCode::EvaluatorExecutionRejected,
                ))
            },
        ));
        assert_eq!(
            failure.issues[0].code(),
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputUnavailable
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
            choregraph_nickel::diagnostic::FrontendIssueCode::CapabilityInputTooLarge
        );
    }
}
