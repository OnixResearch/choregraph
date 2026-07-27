use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

const CORE_PACKAGE: &str = "choregraph-core";
const FOUNDATION_CHECK_PACKAGE: &str = "choregraph-foundation-check";
const REQUIRED_LICENSE: &str = "AGPL-3.0-or-later";
const REQUIRED_RUST_VERSION: &str = "1.97.1";

const PRODUCT_PACKAGES: &[&str] = &[
    CORE_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-seaglass",
    "choregraph-cli",
];

const REQUIRED_PACKAGES: &[&str] = &[
    CORE_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-seaglass",
    "choregraph-cli",
    FOUNDATION_CHECK_PACKAGE,
];

const CORE_FORBIDDEN_DEPENDENCIES: &[&str] = &[
    "basalt",
    "cap-std",
    "kamacite-core",
    "nickel-lang",
    "nickel-lang-core",
    "reqwest",
    "tokio",
    "ucan",
    "valence",
];

const CORE_FORBIDDEN_SOURCE_TOKENS: &[&str] = &[
    "std::env",
    "std::fs",
    "std::net",
    "std::process",
    "std::time",
    "async ",
    "eprintln!",
    "println!",
    "tokio::",
    "reqwest::",
    "cap_std::",
    "nickel_lang",
];

const NO_INTERNAL_DEPENDENCIES: &[&str] = &[];
const CORE_ONLY: &[&str] = &[CORE_PACKAGE];
const CLI_DEPENDENCIES: &[&str] = &[
    CORE_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-seaglass",
];

const REQUIRED_INTERNAL_EDGES: &[(&str, &str)] = &[
    ("choregraph-kamacite", CORE_PACKAGE),
    ("choregraph-nickel", CORE_PACKAGE),
    ("choregraph-seaglass", CORE_PACKAGE),
    ("choregraph-cli", CORE_PACKAGE),
    ("choregraph-cli", "choregraph-kamacite"),
    ("choregraph-cli", "choregraph-nickel"),
    ("choregraph-cli", "choregraph-seaglass"),
];

/// A normalized workspace model consumed by the pure validator.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WorkspaceModel {
    pub packages: Vec<PackageModel>,
    pub core_sources: Vec<SourceUnit>,
}

/// A normalized package and its direct dependency declarations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PackageModel {
    pub name: String,
    pub license: String,
    pub rust_version: String,
    pub publish_disabled: bool,
    pub dependencies: Vec<DependencyModel>,
}

/// A normalized direct dependency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DependencyModel {
    pub name: String,
    pub source: DependencySource,
}

/// The acquisition class of one dependency declaration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DependencySource {
    Workspace,
    Registry,
    LocalPath,
    ExternalPath,
}

/// One in-memory core source unit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SourceUnit {
    pub path: String,
    pub text: String,
}

/// A deterministic foundation validation issue.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidationIssue {
    pub code: &'static str,
    pub subject: String,
    pub detail: String,
}

fn issue(code: &'static str, subject: &str, detail: &str) -> ValidationIssue {
    ValidationIssue {
        code,
        subject: String::from(subject),
        detail: String::from(detail),
    }
}

fn allowed_internal_dependencies(package: &str) -> Option<&'static [&'static str]> {
    match package {
        CORE_PACKAGE | FOUNDATION_CHECK_PACKAGE => Some(NO_INTERNAL_DEPENDENCIES),
        "choregraph-kamacite" | "choregraph-nickel" | "choregraph-seaglass" => Some(CORE_ONLY),
        "choregraph-cli" => Some(CLI_DEPENDENCIES),
        _ => None,
    }
}

fn has_dependency(package: &PackageModel, dependency_name: &str) -> bool {
    for dependency in &package.dependencies {
        if dependency.name == dependency_name && dependency.source == DependencySource::Workspace {
            return true;
        }
    }
    false
}

fn validate_packages(
    model: &WorkspaceModel,
    packages: &BTreeMap<&str, &PackageModel>,
    issues: &mut Vec<ValidationIssue>,
) {
    for required in REQUIRED_PACKAGES {
        if !packages.contains_key(required) {
            issues.push(issue(
                "foundation.missing-package",
                required,
                "required workspace package is absent",
            ));
        }
    }

    for package in &model.packages {
        if !REQUIRED_PACKAGES.contains(&package.name.as_str()) {
            issues.push(issue(
                "foundation.unexpected-package",
                &package.name,
                "workspace package is not in the reviewed foundation layout",
            ));
            continue;
        }

        if package.license != REQUIRED_LICENSE {
            issues.push(issue(
                "foundation.license",
                &package.name,
                "package license is not AGPL-3.0-or-later",
            ));
        }
        if package.rust_version != REQUIRED_RUST_VERSION {
            issues.push(issue(
                "foundation.rust-version",
                &package.name,
                "package Rust version differs from the pinned workspace version",
            ));
        }
        if !package.publish_disabled {
            issues.push(issue(
                "foundation.premature-publish",
                &package.name,
                "foundation packages must remain unpublished",
            ));
        }

        let Some(allowed) = allowed_internal_dependencies(&package.name) else {
            continue;
        };

        for dependency in &package.dependencies {
            if package.name == CORE_PACKAGE
                && CORE_FORBIDDEN_DEPENDENCIES.contains(&dependency.name.as_str())
            {
                issues.push(issue(
                    "foundation.forbidden-core-dependency",
                    &dependency.name,
                    "core dependency imports excluded authority",
                ));
            }

            if PRODUCT_PACKAGES.contains(&package.name.as_str())
                && matches!(
                    dependency.source,
                    DependencySource::LocalPath | DependencySource::ExternalPath
                )
            {
                issues.push(issue(
                    "foundation.path-dependency",
                    &dependency.name,
                    "product package uses an unapproved path dependency",
                ));
            }

            if dependency.source == DependencySource::Workspace
                && !allowed.contains(&dependency.name.as_str())
            {
                issues.push(issue(
                    "foundation.dependency-direction",
                    &package.name,
                    &format!("dependency on {} is not allowed", dependency.name),
                ));
            }
        }
    }

    for (package_name, dependency_name) in REQUIRED_INTERNAL_EDGES {
        let Some(package) = packages.get(package_name) else {
            continue;
        };
        if !has_dependency(package, dependency_name) {
            issues.push(issue(
                "foundation.missing-required-edge",
                package_name,
                &format!("required dependency on {dependency_name} is absent"),
            ));
        }
    }
}

fn validate_core_sources(model: &WorkspaceModel, issues: &mut Vec<ValidationIssue>) {
    let mut has_no_std_marker = false;
    let mut has_alloc_marker = false;

    for source in &model.core_sources {
        if source.text.contains("#![no_std]") {
            has_no_std_marker = true;
        }
        if source.text.contains("extern crate alloc") {
            has_alloc_marker = true;
        }

        for token in CORE_FORBIDDEN_SOURCE_TOKENS {
            if source.text.contains(token) {
                issues.push(issue(
                    "foundation.forbidden-core-source",
                    &source.path,
                    &format!("forbidden source token: {token}"),
                ));
            }
        }
    }

    if !has_no_std_marker {
        issues.push(issue(
            "foundation.core-no-std",
            CORE_PACKAGE,
            "core source does not declare no_std",
        ));
    }
    if !has_alloc_marker {
        issues.push(issue(
            "foundation.core-alloc",
            CORE_PACKAGE,
            "core source does not declare alloc",
        ));
    }
}

/// Validates only in-memory workspace and source data.
#[must_use]
pub fn validate_workspace(model: &WorkspaceModel) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut packages = BTreeMap::new();
    let mut duplicate_names = BTreeSet::new();

    for package in &model.packages {
        if packages.insert(package.name.as_str(), package).is_some() {
            duplicate_names.insert(package.name.as_str());
        }
    }

    for duplicate in duplicate_names {
        issues.push(issue(
            "foundation.duplicate-package",
            duplicate,
            "workspace package name occurs more than once",
        ));
    }

    validate_packages(model, &packages, &mut issues);
    validate_core_sources(model, &mut issues);
    issues.sort();
    issues.dedup();
    issues
}

#[cfg(test)]
mod tests {
    use super::{
        DependencyModel, DependencySource, PackageModel, SourceUnit, WorkspaceModel,
        validate_workspace,
    };

    fn package(name: &str, dependencies: Vec<DependencyModel>) -> PackageModel {
        PackageModel {
            name: String::from(name),
            license: String::from("AGPL-3.0-or-later"),
            rust_version: String::from("1.97.1"),
            publish_disabled: true,
            dependencies,
        }
    }

    fn workspace_dependency(name: &str) -> DependencyModel {
        DependencyModel {
            name: String::from(name),
            source: DependencySource::Workspace,
        }
    }

    fn allowed_model() -> WorkspaceModel {
        WorkspaceModel {
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
            core_sources: vec![SourceUnit {
                path: String::from("crates/choregraph-core/src/lib.rs"),
                text: String::from("#![no_std]\nextern crate alloc;\n"),
            }],
        }
    }

    fn contains_code(model: &WorkspaceModel, code: &str) -> bool {
        validate_workspace(model)
            .iter()
            .any(|issue| issue.code == code)
    }

    #[test]
    fn accepts_the_reviewed_workspace_boundary() {
        assert!(validate_workspace(&allowed_model()).is_empty());
    }

    #[test]
    fn rejects_a_forbidden_core_dependency() {
        let mut model = allowed_model();
        let mut changed = false;
        for package in &mut model.packages {
            if package.name == "choregraph-core" {
                package.dependencies.push(DependencyModel {
                    name: String::from("cap-std"),
                    source: DependencySource::Registry,
                });
                changed = true;
            }
        }
        assert!(changed);
        assert!(contains_code(
            &model,
            "foundation.forbidden-core-dependency"
        ));
    }

    #[test]
    fn rejects_forbidden_core_source() {
        let mut model = allowed_model();
        model.core_sources.push(SourceUnit {
            path: String::from("crates/choregraph-core/src/io.rs"),
            text: String::from("use std::fs;"),
        });
        assert!(contains_code(&model, "foundation.forbidden-core-source"));
    }

    #[test]
    fn rejects_reverse_dependency_direction() {
        let mut model = allowed_model();
        let mut changed = false;
        for package in &mut model.packages {
            if package.name == "choregraph-core" {
                package
                    .dependencies
                    .push(workspace_dependency("choregraph-nickel"));
                changed = true;
            }
        }
        assert!(changed);
        assert!(contains_code(&model, "foundation.dependency-direction"));
    }

    #[test]
    fn rejects_an_external_path_dependency() {
        let mut model = allowed_model();
        let mut changed = false;
        for package in &mut model.packages {
            if package.name == "choregraph-seaglass" {
                package.dependencies.push(DependencyModel {
                    name: String::from("execution-graph-core"),
                    source: DependencySource::ExternalPath,
                });
                changed = true;
            }
        }
        assert!(changed);
        assert!(contains_code(&model, "foundation.path-dependency"));
    }

    #[test]
    fn rejects_license_drift() {
        let mut model = allowed_model();
        let mut changed = false;
        for package in &mut model.packages {
            if package.name == "choregraph-seaglass" {
                package.license = String::from("MIT");
                changed = true;
            }
        }
        assert!(changed);
        assert!(contains_code(&model, "foundation.license"));
    }
}
