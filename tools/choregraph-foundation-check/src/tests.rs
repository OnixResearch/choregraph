fn package(
    name: &str,
    dependencies: Vec<crate::types::DependencyModel>,
) -> crate::types::PackageModel {
    crate::types::PackageModel {
        name: String::from(name),
        license: String::from("AGPL-3.0-or-later"),
        rust_version: String::from("1.97.1"),
        publish_disabled: true,
        dependencies,
    }
}

fn workspace_dependency(name: &str) -> crate::types::DependencyModel {
    crate::types::DependencyModel {
        name: String::from(name),
        source: crate::types::DependencySource::Workspace,
    }
}

fn allowed_model() -> crate::types::WorkspaceModel {
    crate::types::WorkspaceModel {
        packages: vec![
            package("choregraph-core", vec![]),
            package(
                "choregraph-kamacite",
                vec![workspace_dependency("choregraph-core")],
            ),
            package(
                "choregraph-nickel",
                vec![workspace_dependency("choregraph-core")],
            ),
            package(
                "choregraph-seaglass",
                vec![workspace_dependency("choregraph-core")],
            ),
            package(
                "choregraph-cli",
                vec![
                    workspace_dependency("choregraph-core"),
                    workspace_dependency("choregraph-kamacite"),
                    workspace_dependency("choregraph-nickel"),
                    workspace_dependency("choregraph-seaglass"),
                ],
            ),
            package("choregraph-foundation-check", vec![]),
        ],
        core_sources: vec![crate::types::SourceUnit {
            path: String::from("crates/choregraph-core/src/lib.rs"),
            text: String::from("#![no_std]\nextern crate alloc;\n"),
        }],
    }
}

fn contains_code(model: &crate::types::WorkspaceModel, code: &str) -> bool {
    crate::validator::validate_workspace(model)
        .iter()
        .any(|issue| issue.code == code)
}

#[test]
fn accepts_the_reviewed_workspace_boundary() {
    assert!(crate::validator::validate_workspace(&allowed_model()).is_empty());
}

#[test]
fn rejects_a_forbidden_core_dependency() {
    let mut model = allowed_model();
    let mut is_changed = false;
    for package in &mut model.packages {
        if package.name == "choregraph-core" {
            package.dependencies.push(crate::types::DependencyModel {
                name: String::from("cap-std"),
                source: crate::types::DependencySource::Registry,
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
                source: crate::types::DependencySource::ExternalPath,
            });
            is_changed = true;
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
