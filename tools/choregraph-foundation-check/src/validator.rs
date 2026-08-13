pub(crate) const CORE_PACKAGE: &str = "choregraph-core";
const FOUNDATION_CHECK_PACKAGE: &str = "choregraph-foundation-check";
const REQUIRED_LICENSE: &str = "AGPL-3.0-or-later";
const REQUIRED_RUST_VERSION: &str = "1.96.0";
const PACKAGE_METADATA_ISSUE_LIMIT: usize = 3;
const ADJACENT_ISSUE_COUNT: usize = 2;
const BOUNDED_EXEC_PACKAGE: &str = "bounded-exec";
const BOUNDED_EXEC_VENDOR_PATH: &str = "vendor/bounded-exec-selected/crates/bounded-exec";
const CAP_ROOT_PACKAGE: &str = "cap-root-core";
const CAP_ROOT_VENDOR_PATH: &str = "vendor/cap-root-core-selected";
const EXECUTION_GRAPH_PACKAGE: &str = "execution-graph-core";
const EXECUTION_GRAPH_VENDOR_PATH: &str = "vendor/execution-graph-core-selected";

const PRODUCT_PACKAGES: &[&str] = &[
    "choregraph-choreography",
    CORE_PACKAGE,
    EXECUTION_GRAPH_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-release",
    "choregraph-seaglass",
    "choregraph-wasm",
    "choregraph-cli",
];

const CONSUMER_LIBRARY_PACKAGES: &[&str] = &[
    "choregraph-choreography",
    CORE_PACKAGE,
    EXECUTION_GRAPH_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-release",
    "choregraph-seaglass",
    "choregraph-wasm",
];

const BLAKE3_COMPATIBLE_REQUIREMENT: &str = "^1.8.2";
const SERDE_JSON_COMPATIBLE_REQUIREMENT: &str = "^1.0.149";

const REQUIRED_PACKAGES: &[&str] = &[
    "choregraph-choreography",
    CORE_PACKAGE,
    EXECUTION_GRAPH_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-release",
    "choregraph-seaglass",
    "choregraph-cli",
    "choregraph-wasm",
    FOUNDATION_CHECK_PACKAGE,
    "choregraph-wasm-conformance",
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

const NO_INTERNAL_DEPENDENCIES: &[&str] = &[];
const CHOREOGRAPHY_DEPENDENCIES: &[&str] = &["choregraph-kamacite"];
const CORE_ONLY: &[&str] = &[CORE_PACKAGE];
const SEAGLASS_DEPENDENCIES: &[&str] = &[CORE_PACKAGE, "choregraph-nickel"];
const WASM_DEPENDENCIES: &[&str] = &[CORE_PACKAGE, "choregraph-kamacite"];
const WASM_CONFORMANCE_DEPENDENCIES: &[&str] =
    &[CORE_PACKAGE, "choregraph-kamacite", "choregraph-wasm"];
const CLI_DEPENDENCIES: &[&str] = &[
    CORE_PACKAGE,
    "choregraph-kamacite",
    "choregraph-nickel",
    "choregraph-release",
    "choregraph-seaglass",
];

const REQUIRED_INTERNAL_EDGES: &[(&str, &str)] = &[
    ("choregraph-choreography", "choregraph-kamacite"),
    (EXECUTION_GRAPH_PACKAGE, CORE_PACKAGE),
    ("choregraph-kamacite", CORE_PACKAGE),
    ("choregraph-nickel", CORE_PACKAGE),
    ("choregraph-seaglass", CORE_PACKAGE),
    ("choregraph-wasm", CORE_PACKAGE),
    ("choregraph-wasm", "choregraph-kamacite"),
    ("choregraph-wasm-conformance", CORE_PACKAGE),
    ("choregraph-wasm-conformance", "choregraph-kamacite"),
    ("choregraph-wasm-conformance", "choregraph-wasm"),
    ("choregraph-cli", CORE_PACKAGE),
    ("choregraph-cli", "choregraph-kamacite"),
    ("choregraph-cli", "choregraph-nickel"),
    ("choregraph-cli", "choregraph-release"),
    ("choregraph-cli", "choregraph-seaglass"),
];

pub(crate) struct IssueFacts<'a> {
    pub(crate) code: &'static str,
    pub(crate) subject: &'a str,
    pub(crate) detail: String,
}

pub(crate) fn issue(facts: IssueFacts<'_>) -> crate::types::ValidationIssue {
    crate::types::ValidationIssue {
        code: facts.code,
        subject: String::from(facts.subject),
        detail: facts.detail,
    }
}

#[allow(
    unknown_lints,
    missing_const_fn,
    reason = "stable Rust cannot compare string patterns in const functions"
)]
fn allowed_internal_dependencies(package: &str) -> Option<&'static [&'static str]> {
    match package {
        CORE_PACKAGE | "choregraph-release" | FOUNDATION_CHECK_PACKAGE => {
            Some(NO_INTERNAL_DEPENDENCIES)
        }
        "choregraph-choreography" => Some(CHOREOGRAPHY_DEPENDENCIES),
        EXECUTION_GRAPH_PACKAGE | "choregraph-kamacite" | "choregraph-nickel" => Some(CORE_ONLY),
        "choregraph-seaglass" => Some(SEAGLASS_DEPENDENCIES),
        "choregraph-wasm" => Some(WASM_DEPENDENCIES),
        "choregraph-wasm-conformance" => Some(WASM_CONFORMANCE_DEPENDENCIES),
        "choregraph-cli" => Some(CLI_DEPENDENCIES),
        _ => None,
    }
}

fn has_dependency(package: &crate::types::PackageModel, dependency_name: &str) -> bool {
    for dependency in &package.dependencies {
        if dependency.name == dependency_name
            && dependency.source == crate::types::DependencySource::Workspace
        {
            return true;
        }
    }
    false
}

#[allow(
    unknown_lints,
    missing_const_fn,
    reason = "stable Rust cannot match string patterns in const functions"
)]
fn required_public_dependency_requirement(dependency_name: &str) -> Option<&'static str> {
    match dependency_name {
        "blake3" => Some(BLAKE3_COMPATIBLE_REQUIREMENT),
        "serde_json" => Some(SERDE_JSON_COMPATIBLE_REQUIREMENT),
        _ => None,
    }
}

#[allow(
    unknown_lints,
    missing_const_fn,
    reason = "stable Rust cannot compare owned dependency metadata in const functions"
)]
fn has_invalid_public_dependency_requirement(
    package: &crate::types::PackageModel,
    dependency: &crate::types::DependencyModel,
) -> bool {
    if !CONSUMER_LIBRARY_PACKAGES.contains(&package.name.as_str())
        || dependency.source != crate::types::DependencySource::Registry
    {
        return false;
    }
    let Some(required) = required_public_dependency_requirement(&dependency.name) else {
        return false;
    };
    dependency.requirement != required
}

fn validate_required_packages(
    packages: &std::collections::BTreeMap<&str, &crate::types::PackageModel>,
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = Vec::with_capacity(REQUIRED_PACKAGES.len());
    for required in REQUIRED_PACKAGES {
        if !packages.contains_key(required) {
            issues.push(issue(IssueFacts {
                code: "foundation.missing-package",
                subject: required,
                detail: String::from("required workspace package is absent"),
            }));
        }
    }
    issues
}

fn validate_package_metadata(
    package: &crate::types::PackageModel,
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = Vec::with_capacity(PACKAGE_METADATA_ISSUE_LIMIT);
    if package.license != REQUIRED_LICENSE {
        issues.push(issue(IssueFacts {
            code: "foundation.license",
            subject: &package.name,
            detail: String::from("package license is not AGPL-3.0-or-later"),
        }));
    }
    if package.rust_version != REQUIRED_RUST_VERSION {
        issues.push(issue(IssueFacts {
            code: "foundation.rust-version",
            subject: &package.name,
            detail: String::from("package Rust version differs from the pinned workspace version"),
        }));
    }
    if !package.publish_disabled {
        issues.push(issue(IssueFacts {
            code: "foundation.premature-publish",
            subject: &package.name,
            detail: String::from("foundation packages must remain unpublished"),
        }));
    }
    debug_assert!(issues.len() <= PACKAGE_METADATA_ISSUE_LIMIT);
    debug_assert!(issues.iter().all(|item| item.subject == package.name));
    issues
}

#[allow(
    unknown_lints,
    missing_const_fn,
    reason = "stable Rust cannot compare owned dependency names and paths in a const function"
)]
fn is_reviewed_path_dependency(
    package: &crate::types::PackageModel,
    dependency: &crate::types::DependencyModel,
) -> bool {
    let is_bounded_exec = package.name == "choregraph-cli"
        && dependency.name == BOUNDED_EXEC_PACKAGE
        && dependency.path == BOUNDED_EXEC_VENDOR_PATH;
    let is_cap_root = package.name == "choregraph-cli"
        && dependency.name == CAP_ROOT_PACKAGE
        && dependency.path == CAP_ROOT_VENDOR_PATH;
    let is_execution_graph_oracle = package.name == "choregraph-seaglass"
        && dependency.name == EXECUTION_GRAPH_PACKAGE
        && dependency.path == EXECUTION_GRAPH_VENDOR_PATH;
    dependency.source == crate::types::DependencySource::LocalPath
        && (is_bounded_exec || is_cap_root || is_execution_graph_oracle)
}

fn validate_dependency(
    package: &crate::types::PackageModel,
    dependency: &crate::types::DependencyModel,
    allowed: &[&str],
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = Vec::with_capacity(PACKAGE_METADATA_ISSUE_LIMIT);
    if package.name == CORE_PACKAGE
        && CORE_FORBIDDEN_DEPENDENCIES.contains(&dependency.name.as_str())
    {
        issues.push(issue(IssueFacts {
            code: "foundation.forbidden-core-dependency",
            subject: &dependency.name,
            detail: String::from("core dependency imports excluded authority"),
        }));
    }
    if PRODUCT_PACKAGES.contains(&package.name.as_str())
        && matches!(
            dependency.source,
            crate::types::DependencySource::LocalPath
                | crate::types::DependencySource::ExternalPath
        )
        && !is_reviewed_path_dependency(package, dependency)
    {
        issues.push(issue(IssueFacts {
            code: "foundation.path-dependency",
            subject: &dependency.name,
            detail: String::from("product package uses an unapproved path dependency"),
        }));
    }
    if dependency.source == crate::types::DependencySource::Workspace
        && !allowed.contains(&dependency.name.as_str())
    {
        issues.push(issue(IssueFacts {
            code: "foundation.dependency-direction",
            subject: &package.name,
            detail: format!("dependency on {} is not allowed", dependency.name),
        }));
    }
    if has_invalid_public_dependency_requirement(package, dependency) {
        issues.push(issue(IssueFacts {
            code: "foundation.public-dependency-requirement",
            subject: &package.name,
            detail: format!(
                "public dependency {} uses unreviewed requirement {}",
                dependency.name, dependency.requirement
            ),
        }));
    }
    debug_assert!(issues.len() <= PACKAGE_METADATA_ISSUE_LIMIT);
    debug_assert!(
        issues
            .iter()
            .all(|item| item.code.starts_with("foundation."))
    );
    issues
}

fn validate_package(package: &crate::types::PackageModel) -> Vec<crate::types::ValidationIssue> {
    if !REQUIRED_PACKAGES.contains(&package.name.as_str()) {
        return vec![issue(IssueFacts {
            code: "foundation.unexpected-package",
            subject: &package.name,
            detail: String::from("workspace package is not in the reviewed foundation layout"),
        })];
    }
    let Some(allowed) = allowed_internal_dependencies(&package.name) else {
        return Vec::new();
    };
    let maximum_issue_count = PACKAGE_METADATA_ISSUE_LIMIT.saturating_add(
        package
            .dependencies
            .len()
            .saturating_mul(PACKAGE_METADATA_ISSUE_LIMIT),
    );
    let mut issues = Vec::with_capacity(maximum_issue_count);
    issues.extend(validate_package_metadata(package));
    for dependency in &package.dependencies {
        issues.extend(validate_dependency(package, dependency, allowed));
    }
    debug_assert!(issues.len() <= maximum_issue_count);
    debug_assert!(
        issues
            .iter()
            .all(|item| item.code.starts_with("foundation."))
    );
    issues
}

fn validate_required_edges(
    packages: &std::collections::BTreeMap<&str, &crate::types::PackageModel>,
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = Vec::with_capacity(REQUIRED_INTERNAL_EDGES.len());
    for (package_name, dependency_name) in REQUIRED_INTERNAL_EDGES {
        let Some(package) = packages.get(package_name) else {
            continue;
        };
        if !has_dependency(package, dependency_name) {
            issues.push(issue(IssueFacts {
                code: "foundation.missing-required-edge",
                subject: package_name,
                detail: format!("required dependency on {dependency_name} is absent"),
            }));
        }
    }
    issues
}

fn validate_packages(
    model: &crate::types::WorkspaceModel,
    packages: &std::collections::BTreeMap<&str, &crate::types::PackageModel>,
) -> Vec<crate::types::ValidationIssue> {
    let mut issues = validate_required_packages(packages);
    for package in &model.packages {
        issues.extend(validate_package(package));
    }
    issues.extend(validate_required_edges(packages));
    issues
}

/// Validates only in-memory workspace and source data.
#[must_use]
pub fn validate_workspace(
    model: &crate::types::WorkspaceModel,
) -> Vec<crate::types::ValidationIssue> {
    let mut packages = std::collections::BTreeMap::new();
    let mut duplicate_names = std::collections::BTreeSet::new();
    for package in &model.packages {
        if packages.insert(package.name.as_str(), package).is_some() {
            duplicate_names.insert(package.name.as_str());
        }
    }
    let mut issues = Vec::with_capacity(model.packages.len());
    for duplicate in duplicate_names {
        issues.push(issue(IssueFacts {
            code: "foundation.duplicate-package",
            subject: duplicate,
            detail: String::from("workspace package name occurs more than once"),
        }));
    }
    issues.extend(validate_packages(model, &packages));
    issues.extend(crate::source::validate_sources(model));
    issues.sort();
    issues.dedup();
    debug_assert!(
        issues
            .windows(ADJACENT_ISSUE_COUNT)
            .all(|pair| pair[0] < pair[1])
    );
    debug_assert!(issues.len() <= issues.capacity());
    issues
}
