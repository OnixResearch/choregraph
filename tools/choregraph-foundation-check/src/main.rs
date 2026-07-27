use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cargo_metadata::{MetadataCommand, Package};
use choregraph_foundation_check::{
    DependencyModel, DependencySource, PackageModel, SourceUnit, WorkspaceModel, validate_workspace,
};

const CORE_SOURCE_PATH: &str = "crates/choregraph-core/src";

fn parse_single_path(
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<(&'static str, PathBuf), String> {
    let option = arguments.next().ok_or_else(|| {
        String::from("usage: choregraph-foundation-check (--root PATH | --model-json PATH)")
    })?;
    let value = arguments
        .next()
        .ok_or_else(|| String::from("missing path after foundation check option"))?;
    if arguments.next().is_some() {
        return Err(String::from("foundation check accepts one input path"));
    }

    let option = option
        .to_str()
        .ok_or_else(|| String::from("foundation check option is not UTF-8"))?;
    match option {
        "--root" => Ok(("root", PathBuf::from(value))),
        "--model-json" => Ok(("model-json", PathBuf::from(value))),
        _ => Err(format!("unknown foundation check option: {option}")),
    }
}

fn dependency_source(
    dependency_name: &str,
    dependency_path: Option<&Path>,
    workspace_root: &Path,
    internal_packages: &BTreeSet<String>,
) -> DependencySource {
    let Some(path) = dependency_path else {
        return DependencySource::Registry;
    };
    if internal_packages.contains(dependency_name) {
        return DependencySource::Workspace;
    }
    if path.starts_with(workspace_root) {
        return DependencySource::LocalPath;
    }
    DependencySource::ExternalPath
}

fn normalize_package(
    package: &Package,
    workspace_root: &Path,
    internal_packages: &BTreeSet<String>,
) -> PackageModel {
    let mut dependencies = Vec::new();
    for dependency in &package.dependencies {
        let name = dependency.name.to_string();
        let source = dependency_source(
            &name,
            dependency.path.as_ref().map(|path| path.as_std_path()),
            workspace_root,
            internal_packages,
        );
        dependencies.push(DependencyModel { name, source });
    }
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));

    PackageModel {
        name: package.name.to_string(),
        license: package.license.clone().unwrap_or_default(),
        rust_version: package
            .rust_version
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        publish_disabled: package.publish.as_ref().is_some_and(Vec::is_empty),
        dependencies,
    }
}

fn collect_source_units(
    source_root: &Path,
    current: &Path,
    units: &mut Vec<SourceUnit>,
) -> Result<(), String> {
    let mut entries = Vec::new();
    let directory = fs::read_dir(current)
        .map_err(|error| format!("cannot read {}: {error}", current.display()))?;
    for entry in directory {
        let entry = entry.map_err(|error| {
            format!(
                "cannot read directory entry in {}: {error}",
                current.display()
            )
        })?;
        entries.push(entry);
    }
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
        if file_type.is_dir() {
            collect_source_units(source_root, &path, units)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        let relative = path
            .strip_prefix(source_root)
            .map_err(|error| format!("cannot normalize {}: {error}", path.display()))?;
        units.push(SourceUnit {
            path: relative.display().to_string(),
            text,
        });
    }
    Ok(())
}

fn load_workspace(root: &Path) -> Result<WorkspaceModel, String> {
    let mut command = MetadataCommand::new();
    command.current_dir(root);
    command.no_deps();
    let metadata = command
        .exec()
        .map_err(|error| format!("cargo metadata failed: {error}"))?;

    let workspace_root = metadata.workspace_root.as_std_path();
    let mut internal_packages = BTreeSet::new();
    for package in &metadata.packages {
        if metadata.workspace_members.contains(&package.id) {
            internal_packages.insert(package.name.to_string());
        }
    }

    let mut packages = Vec::new();
    for package in &metadata.packages {
        if metadata.workspace_members.contains(&package.id) {
            packages.push(normalize_package(
                package,
                workspace_root,
                &internal_packages,
            ));
        }
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));

    let source_root = workspace_root.join(CORE_SOURCE_PATH);
    let mut core_sources = Vec::new();
    collect_source_units(&source_root, &source_root, &mut core_sources)?;
    core_sources.sort_by(|left, right| left.path.cmp(&right.path));

    Ok(WorkspaceModel {
        packages,
        core_sources,
    })
}

fn load_model_json(path: &Path) -> Result<WorkspaceModel, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read fixture {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("cannot parse fixture {}: {error}", path.display()))
}

fn run() -> Result<(), String> {
    let (input_kind, path) = parse_single_path(env::args_os().skip(1))?;
    let model = match input_kind {
        "root" => load_workspace(&path)?,
        "model-json" => load_model_json(&path)?,
        _ => return Err(String::from("internal foundation input error")),
    };

    let issues = validate_workspace(&model);
    if issues.is_empty() {
        println!("foundation-boundary: passed");
        return Ok(());
    }

    for issue in issues {
        eprintln!("{}: {}: {}", issue.code, issue.subject, issue.detail);
    }
    Err(String::from("foundation-boundary: failed"))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
