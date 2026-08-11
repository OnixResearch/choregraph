fn package(
    name: &str,
    dependencies: Vec<crate::types::DependencyModel>,
) -> crate::types::PackageModel {
    crate::types::PackageModel {
        name: String::from(name),
        license: String::from("AGPL-3.0-or-later"),
        rust_version: String::from("1.96.0"),
        publish_disabled: true,
        dependencies,
    }
}

fn workspace_dependency(name: &str) -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from(name),
        requirement: String::from("=0.1.0"),
        source: crate::types::DependencySource::Workspace,
        path: String::new(),
    }
}

fn blake3_dependency(requirement: &str) -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from("blake3"),
        requirement: String::from(requirement),
        source: crate::types::DependencySource::Registry,
        path: String::new(),
    }
}

fn serde_json_dependency(requirement: &str) -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from("serde_json"),
        requirement: String::from(requirement),
        source: crate::types::DependencySource::Registry,
        path: String::new(),
    }
}

fn reviewed_bounded_exec_dependency() -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from("bounded-exec"),
        requirement: String::from("=0.1.0"),
        source: crate::types::DependencySource::LocalPath,
        path: String::from("vendor/bounded-exec-selected/crates/bounded-exec"),
    }
}

fn reviewed_cap_root_dependency() -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from("cap-root-core"),
        requirement: String::from("=0.1.0"),
        source: crate::types::DependencySource::LocalPath,
        path: String::from("vendor/cap-root-core-selected"),
    }
}

fn reviewed_execution_graph_oracle() -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from("execution-graph-core"),
        requirement: String::from("=0.1.0"),
        source: crate::types::DependencySource::LocalPath,
        path: String::from("vendor/execution-graph-core-selected"),
    }
}

fn allowed_packages() -> Vec<crate::types::PackageModel> {
    vec![
        package("choregraph-core", vec![]),
        package(
            "execution-graph-core",
            vec![workspace_dependency("choregraph-core")],
        ),
        package(
            "choregraph-kamacite",
            vec![workspace_dependency("choregraph-core")],
        ),
        package(
            "choregraph-nickel",
            vec![workspace_dependency("choregraph-core")],
        ),
        package("choregraph-release", vec![]),
        package(
            "choregraph-seaglass",
            vec![
                workspace_dependency("choregraph-core"),
                workspace_dependency("choregraph-nickel"),
                reviewed_execution_graph_oracle(),
            ],
        ),
        package(
            "choregraph-wasm",
            vec![
                workspace_dependency("choregraph-core"),
                workspace_dependency("choregraph-kamacite"),
            ],
        ),
        package(
            "choregraph-cli",
            vec![
                workspace_dependency("choregraph-core"),
                workspace_dependency("choregraph-kamacite"),
                workspace_dependency("choregraph-nickel"),
                workspace_dependency("choregraph-release"),
                workspace_dependency("choregraph-seaglass"),
                reviewed_bounded_exec_dependency(),
                reviewed_cap_root_dependency(),
            ],
        ),
        package("choregraph-foundation-check", vec![]),
        package(
            "choregraph-wasm-conformance",
            vec![
                workspace_dependency("choregraph-core"),
                workspace_dependency("choregraph-kamacite"),
                workspace_dependency("choregraph-wasm"),
            ],
        ),
    ]
}

fn allowed_model() -> crate::types::WorkspaceModel {
    crate::types::WorkspaceModel {
        packages: allowed_packages(),
        core_sources: vec![crate::types::SourceUnit {
            path: String::from("crates/choregraph-core/src/lib.rs"),
            text: String::from("#![no_std]\nextern crate alloc;\n"),
        }],
        wasm_adapter_sources: vec![crate::types::SourceUnit {
            path: String::from("dispatch.rs"),
            text: [
                "validate_machine_artifact(",
                "validate_durable_machine_artifact(",
                "validate_concrete_machine_artifact(",
                "step(&context.machine",
                "step_concrete(&context.machine",
                "replay(input)",
                "replay_concrete(input)",
            ]
            .join("\n"),
        }],
        wasm_wit_sources: vec![crate::types::SourceUnit {
            path: String::from("kernel.wit"),
            text: String::from(
                "package onix:choregraph@1.0.0;\nworld kernel {\n  export call: func(request: list<u8>) -> list<u8>;\n}\n",
            ),
        }],
    }
}

fn contains_code(model: &crate::types::WorkspaceModel, code: &str) -> bool {
    crate::validator::validate_workspace(model)
        .iter()
        .any(|issue| issue.code == code)
}

fn add_dependency(
    model: &mut crate::types::WorkspaceModel,
    package_name: &str,
    dependency: crate::types::DependencyModel,
) -> bool {
    for package in &mut model.packages {
        if package.name == package_name {
            package.dependencies.push(dependency);
            return true;
        }
    }
    false
}

#[test]
fn accepts_the_reviewed_workspace_boundary() {
    assert!(crate::validator::validate_workspace(&allowed_model()).is_empty());
}

#[test]
fn accepts_compatible_public_dependency_requirements() {
    let mut model = allowed_model();
    assert!(add_dependency(
        &mut model,
        "choregraph-nickel",
        blake3_dependency("^1.8.2"),
    ));
    assert!(add_dependency(
        &mut model,
        "choregraph-nickel",
        serde_json_dependency("^1.0.149"),
    ));
    assert!(crate::validator::validate_workspace(&model).is_empty());
}

#[test]
fn rejects_exact_blake3_public_dependency_requirement() {
    let mut model = allowed_model();
    assert!(add_dependency(
        &mut model,
        "choregraph-nickel",
        blake3_dependency("=1.8.2"),
    ));
    assert!(contains_code(
        &model,
        "foundation.public-dependency-requirement"
    ));
}

#[test]
fn rejects_exact_serde_json_public_dependency_requirement() {
    let mut model = allowed_model();
    assert!(add_dependency(
        &mut model,
        "choregraph-nickel",
        serde_json_dependency("=1.0.149"),
    ));
    assert!(contains_code(
        &model,
        "foundation.public-dependency-requirement"
    ));
}

#[test]
fn rejects_compiler_floor_drift() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-core" {
            package.rust_version = String::from("1.97.1");
            is_changed = true;
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.rust-version"));
}

#[test]
fn rejects_a_forbidden_core_dependency() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-core" {
            package.dependencies.push(crate::types::DependencyModel {
                name: String::from("cap-std"),
                requirement: String::from("^4.0.2"),
                source: crate::types::DependencySource::Registry,
                path: String::new(),
            });
            is_changed = true;
        }
    }
    assert!(is_changed);
    assert!(contains_code(
        &model,
        "foundation.forbidden-core-dependency"
    ));
}

#[test]
fn rejects_forbidden_core_source() {
    let mut model = allowed_model();
    model.core_sources.push(crate::types::SourceUnit {
        path: String::from("crates/choregraph-core/src/io.rs"),
        text: String::from("use std::fs;"),
    });
    assert!(contains_code(&model, "foundation.forbidden-core-source"));
}

#[test]
fn rejects_reverse_dependency_direction() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-core" {
            package
                .dependencies
                .push(workspace_dependency("choregraph-nickel"));
            is_changed = true;
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.dependency-direction"));
}

#[test]
fn rejects_an_external_path_dependency() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-seaglass" {
            package.dependencies.push(crate::types::DependencyModel {
                name: String::from("execution-graph-core"),
                requirement: String::from("=0.1.0"),
                source: crate::types::DependencySource::ExternalPath,
                path: String::from("../execution-graph"),
            });
            is_changed = true;
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.path-dependency"));
}

#[test]
fn rejects_bounded_exec_from_an_unreviewed_path() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-cli" {
            for dependency in &mut package.dependencies {
                if dependency.name == "bounded-exec" {
                    dependency.path = String::from("../bounded-exec/crates/bounded-exec");
                    is_changed = true;
                }
            }
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.path-dependency"));
}

#[test]
fn rejects_execution_graph_oracle_from_an_unreviewed_path() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-seaglass" {
            for dependency in &mut package.dependencies {
                if dependency.name == "execution-graph-core" {
                    dependency.path = String::from("../execution-graph");
                    is_changed = true;
                }
            }
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.path-dependency"));
}

#[test]
fn rejects_cap_root_from_an_unreviewed_path() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-cli" {
            for dependency in &mut package.dependencies {
                if dependency.name == "cap-root-core" {
                    dependency.path = String::from("../cap-root/crates/cap-root-core");
                    is_changed = true;
                }
            }
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.path-dependency"));
}

#[test]
fn rejects_license_drift() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-seaglass" {
            package.license = String::from("MIT");
            is_changed = true;
        }
    }
    assert!(is_changed);
    assert!(contains_code(&model, "foundation.license"));
}

#[test]
fn accepts_legacy_models_without_a_selected_wasm_adapter() {
    let mut model = allowed_model();
    model
        .packages
        .retain(|package| package.name != "choregraph-wasm");
    model.wasm_adapter_sources.clear();
    model.wasm_wit_sources.clear();
    let issues = crate::validator::validate_workspace(&model);
    assert!(
        !issues
            .iter()
            .any(|issue| issue.code.starts_with("foundation.wasm-"))
    );
}

#[test]
fn rejects_missing_wasm_production_delegation() {
    let mut model = allowed_model();
    model.wasm_adapter_sources[0].text = String::from("fn call() {}");
    assert!(contains_code(&model, "foundation.wasm-missing-delegation"));
}

#[test]
fn rejects_wasm_persistent_handles_and_graph_specific_generation() {
    let mut model = allowed_model();
    model.wasm_adapter_sources.push(crate::types::SourceUnit {
        path: String::from("bad.rs"),
        text: String::from("static CACHE: OnceLock<()>; MachineSpec::new(Vec::new(), Vec::new());"),
    });
    assert!(contains_code(&model, "foundation.forbidden-wasm-source"));
}

#[test]
fn rejects_wasm_host_imports_and_extra_exports() {
    let mut model = allowed_model();
    model.wasm_wit_sources[0].text = String::from(
        "package onix:choregraph@1.0.0;\nworld kernel {\n  import clock: func();\n  export call: func(request: list<u8>) -> list<u8>;\n  export extra: func();\n}\n",
    );
    assert!(contains_code(&model, "foundation.wasm-host-import"));
    assert!(contains_code(&model, "foundation.wasm-export-surface"));
}
