const CORE_SOURCE_PATH: &str = "crates/choregraph-core/src";
const MAX_CORE_SOURCE_ENTRIES: usize = 4_096;
const INITIAL_DIRECTORY_CAPACITY: usize = 32;
const ADJACENT_ENTRY_COUNT: usize = 2;

fn parse_single_path(
    mut arguments: impl Iterator<Item = std::ffi::OsString>,
) -> Result<(&'static str, std::path::PathBuf), String> {
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
        "--root" => Ok(("root", std::path::PathBuf::from(value))),
        "--model-json" => Ok(("model-json", std::path::PathBuf::from(value))),
        _ => Err(format!("unknown foundation check option: {option}")),
    }
}

fn dependency_source(
    dependency_path: Option<&std::path::Path>,
    workspace_root: &std::path::Path,
    internal_package_paths: &std::collections::BTreeSet<std::path::PathBuf>,
) -> choregraph_foundation_check::types::DependencySource {
    let Some(path) = dependency_path else {
        return choregraph_foundation_check::types::DependencySource::Registry;
    };
    if internal_package_paths.contains(path) {
        return choregraph_foundation_check::types::DependencySource::Workspace;
    }
    if path.starts_with(workspace_root) {
        return choregraph_foundation_check::types::DependencySource::LocalPath;
    }
    choregraph_foundation_check::types::DependencySource::ExternalPath
}

fn normalize_package(
    package: &cargo_metadata::Package,
    workspace_root: &std::path::Path,
    internal_package_paths: &std::collections::BTreeSet<std::path::PathBuf>,
) -> choregraph_foundation_check::types::PackageModel {
    let mut dependencies = Vec::with_capacity(package.dependencies.len());
    for dependency in &package.dependencies {
        let name = dependency.name.to_string();
        let dependency_path = dependency.path.as_ref().map(|path| path.as_std_path());
        let source = dependency_source(dependency_path, workspace_root, internal_package_paths);
        let path = dependency_path
            .map(|dependency_path| {
                dependency_path
                    .strip_prefix(workspace_root)
                    .unwrap_or(dependency_path)
                    .display()
                    .to_string()
            })
            .unwrap_or_default();
        dependencies.push(choregraph_foundation_check::types::DependencyModel {
            name,
            requirement: dependency.req.to_string(),
            source,
            path,
        });
    }
    dependencies.sort_by(|left, right| left.name.cmp(&right.name));
    debug_assert!(dependencies.len() <= package.dependencies.len());
    debug_assert!(
        dependencies
            .windows(ADJACENT_ENTRY_COUNT)
            .all(|pair| pair[0].name <= pair[1].name)
    );

    choregraph_foundation_check::types::PackageModel {
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

fn read_sorted_entries(current: &std::path::Path) -> Result<Vec<std::fs::DirEntry>, String> {
    let mut entries = Vec::with_capacity(INITIAL_DIRECTORY_CAPACITY);
    let directory = std::fs::read_dir(current)
        .map_err(|error| format!("cannot read {}: {error}", current.display()))?;
    for entry in directory {
        if entries.len() >= MAX_CORE_SOURCE_ENTRIES {
            return Err(format!(
                "directory entry limit exceeded under {}",
                current.display()
            ));
        }
        let entry = entry.map_err(|error| {
            format!(
                "cannot read directory entry in {}: {error}",
                current.display()
            )
        })?;
        entries.push(entry);
    }
    entries.sort_by_key(std::fs::DirEntry::path);
    debug_assert!(entries.len() <= MAX_CORE_SOURCE_ENTRIES);
    debug_assert!(
        entries
            .windows(ADJACENT_ENTRY_COUNT)
            .all(|pair| pair[0].path() <= pair[1].path())
    );
    Ok(entries)
}

fn collect_source_units(
    source_root: &std::path::Path,
) -> Result<Vec<choregraph_foundation_check::types::SourceUnit>, String> {
    let mut pending = Vec::with_capacity(MAX_CORE_SOURCE_ENTRIES);
    let mut units = Vec::with_capacity(MAX_CORE_SOURCE_ENTRIES);
    pending.push(source_root.to_path_buf());

    for _entry_index in 0..MAX_CORE_SOURCE_ENTRIES {
        let Some(current) = pending.pop() else {
            break;
        };
        for entry in read_sorted_entries(&current)? {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
            if file_type.is_dir() {
                if pending.len() >= MAX_CORE_SOURCE_ENTRIES {
                    return Err(String::from("core source directory limit exceeded"));
                }
                pending.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            if units.len() >= MAX_CORE_SOURCE_ENTRIES {
                return Err(String::from("core source file limit exceeded"));
            }
            let text = std::fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            let relative = path
                .strip_prefix(source_root)
                .map_err(|error| format!("cannot normalize {}: {error}", path.display()))?;
            units.push(choregraph_foundation_check::types::SourceUnit {
                path: relative.display().to_string(),
                text,
            });
        }
    }
    if !pending.is_empty() {
        return Err(String::from("core source traversal limit exceeded"));
    }
    units.sort_by(|left, right| left.path.cmp(&right.path));
    debug_assert!(pending.is_empty());
    debug_assert!(units.len() <= MAX_CORE_SOURCE_ENTRIES);
    Ok(units)
}

fn load_workspace(
    root: &std::path::Path,
) -> Result<choregraph_foundation_check::types::WorkspaceModel, String> {
    let mut command = cargo_metadata::MetadataCommand::new();
    command.current_dir(root);
    command.no_deps();
    let metadata = command
        .exec()
        .map_err(|error| format!("cargo metadata failed: {error}"))?;

    let workspace_root = metadata.workspace_root.as_std_path();
    let mut internal_package_paths = std::collections::BTreeSet::new();
    for package in &metadata.packages {
        if metadata.workspace_members.contains(&package.id) {
            let package_path = package
                .manifest_path
                .parent()
                .ok_or_else(|| format!("package {} has no manifest parent", package.name))?;
            internal_package_paths.insert(package_path.as_std_path().to_path_buf());
        }
    }

    let mut packages = Vec::with_capacity(metadata.packages.len());
    for package in &metadata.packages {
        if metadata.workspace_members.contains(&package.id) {
            packages.push(normalize_package(
                package,
                workspace_root,
                &internal_package_paths,
            ));
        }
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));

    let source_root = workspace_root.join(CORE_SOURCE_PATH);
    let core_sources = collect_source_units(&source_root)?;
    debug_assert!(packages.len() <= metadata.packages.len());
    debug_assert!(core_sources.len() <= MAX_CORE_SOURCE_ENTRIES);
    Ok(choregraph_foundation_check::types::WorkspaceModel {
        packages,
        core_sources,
    })
}

fn load_model_json(
    path: &std::path::Path,
) -> Result<choregraph_foundation_check::types::WorkspaceModel, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("cannot read fixture {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("cannot parse fixture {}: {error}", path.display()))
}

fn run(arguments: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    let (input_kind, path) = parse_single_path(arguments)?;
    let model = match input_kind {
        "root" => load_workspace(&path)?,
        "model-json" => load_model_json(&path)?,
        _ => return Err(String::from("internal foundation input error")),
    };

    let issues = choregraph_foundation_check::validator::validate_workspace(&model);
    if issues.is_empty() {
        println!("foundation-boundary: passed");
        return Ok(());
    }

    for issue in issues {
        eprintln!("{}: {}: {}", issue.code, issue.subject, issue.detail);
    }
    Err(String::from("foundation-boundary: failed"))
}

#[allow(
    unknown_lints,
    ambient_env,
    reason = "the CLI shell must capture process arguments before calling explicit-input logic"
)]
fn main() -> std::process::ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    const WORKSPACE_ROOT: &str = "/workspace";
    const ACTIVE_PACKAGE_PATH: &str = "/workspace/crates/execution-graph-core";
    const VENDORED_PACKAGE_PATH: &str = "/workspace/vendor/execution-graph-core-selected";

    fn internal_paths() -> std::collections::BTreeSet<std::path::PathBuf> {
        std::collections::BTreeSet::from([std::path::PathBuf::from(ACTIVE_PACKAGE_PATH)])
    }

    #[test]
    fn classifies_the_exact_workspace_package_path_as_internal() {
        assert_eq!(
            super::dependency_source(
                Some(std::path::Path::new(ACTIVE_PACKAGE_PATH)),
                std::path::Path::new(WORKSPACE_ROOT),
                &internal_paths(),
            ),
            choregraph_foundation_check::types::DependencySource::Workspace
        );
    }

    #[test]
    fn does_not_classify_a_same_name_vendor_path_as_internal() {
        assert_eq!(
            super::dependency_source(
                Some(std::path::Path::new(VENDORED_PACKAGE_PATH)),
                std::path::Path::new(WORKSPACE_ROOT),
                &internal_paths(),
            ),
            choregraph_foundation_check::types::DependencySource::LocalPath
        );
    }
}
