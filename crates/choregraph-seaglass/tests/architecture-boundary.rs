#![allow(
    unknown_lints,
    missing_const_fn,
    non_trait_imports,
    unbounded_collection_growth,
    reason = "the source architecture fixture collects finite static token inventories and one bounded test manifest"
)]

const ADAPTER_SOURCE: &str = include_str!("../src/lib.rs");
const ADAPTER_MANIFEST: &str = include_str!("../Cargo.toml");
const DEPENDENCY_HEADER: &str = "[dependencies]";
const DEV_DEPENDENCY_HEADER: &str = "[dev-dependencies]";
const REQUIRED_PRODUCTION_DEPENDENCY: &str = "choregraph-core.workspace = true";
const EXPECTED_PRODUCTION_DEPENDENCY_COUNT: usize = 1;

const REQUIRED_DELEGATE_CALLS: &[&str] = &[
    "choregraph_core::AdmittedMachine::admit",
    "choregraph_core::parallel_groups",
    "choregraph_core::critical_path",
    "choregraph_core::bottlenecks",
    "choregraph_core::lifecycle_target",
    "choregraph_core::validate_invariant",
    "choregraph_core::ready_nodes",
    "choregraph_core::resume_decision",
    "choregraph_core::step",
    "choregraph_core::would_create_cycle",
];
const FORBIDDEN_AUTHORITY_TOKENS: &[&str] = &[
    "std::fs",
    "std::net",
    "std::process",
    "SystemTime",
    "Command::new",
    "cap_std",
    "basalt",
    "ucan",
    "tokio",
];
const FORBIDDEN_ALGORITHM_TOKENS: &[&str] = &["\n    while ", "\n    loop {"];

#[derive(Clone, Debug, Eq, PartialEq)]
enum BoundaryViolation<'a> {
    MissingDelegate(&'a str),
    AuthorityToken(&'a str),
    IndependentLoop(&'a str),
    ProductionDependencyCount(usize),
    ProductionDependency(&'a str),
    MissingManifestSection,
}

fn audit_source(source: &str) -> Vec<BoundaryViolation<'static>> {
    let mut violations = Vec::new();
    for required in REQUIRED_DELEGATE_CALLS {
        if !source.contains(required) {
            violations.push(BoundaryViolation::MissingDelegate(required));
        }
    }
    for forbidden in FORBIDDEN_AUTHORITY_TOKENS {
        if source.contains(forbidden) {
            violations.push(BoundaryViolation::AuthorityToken(forbidden));
        }
    }
    for forbidden in FORBIDDEN_ALGORITHM_TOKENS {
        if source.contains(forbidden) {
            violations.push(BoundaryViolation::IndependentLoop(forbidden));
        }
    }
    violations
}

fn production_dependencies(manifest: &str) -> Result<Vec<&str>, BoundaryViolation<'_>> {
    let (_, after_header) = manifest
        .split_once(DEPENDENCY_HEADER)
        .ok_or(BoundaryViolation::MissingManifestSection)?;
    let (dependencies, _) = after_header
        .split_once(DEV_DEPENDENCY_HEADER)
        .ok_or(BoundaryViolation::MissingManifestSection)?;
    Ok(dependencies
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect())
}

fn audit_manifest(manifest: &str) -> Vec<BoundaryViolation<'_>> {
    let dependencies = match production_dependencies(manifest) {
        Ok(dependencies) => dependencies,
        Err(error) => return vec![error],
    };
    let mut violations = Vec::new();
    if dependencies.len() != EXPECTED_PRODUCTION_DEPENDENCY_COUNT {
        violations.push(BoundaryViolation::ProductionDependencyCount(
            dependencies.len(),
        ));
    }
    for dependency in dependencies {
        if dependency != REQUIRED_PRODUCTION_DEPENDENCY {
            violations.push(BoundaryViolation::ProductionDependency(dependency));
        }
    }
    violations
}

#[test]
fn production_adapter_delegates_without_runtime_authority() {
    assert_eq!(audit_source(ADAPTER_SOURCE), Vec::new());
    assert_eq!(audit_manifest(ADAPTER_MANIFEST), Vec::new());
}

#[test]
fn duplicate_algorithm_and_authority_tokens_fail_the_gate() {
    let invalid_source = "fn independent() {\n    while pending { std::fs::read(\"ambient\"); }\n}";
    let violations = audit_source(invalid_source);
    assert!(violations.iter().any(|violation| matches!(
        violation,
        BoundaryViolation::IndependentLoop("\n    while ")
    )));
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, BoundaryViolation::AuthorityToken("std::fs")))
    );
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, BoundaryViolation::MissingDelegate(_)))
    );
}

#[test]
fn extra_or_missing_production_dependencies_fail_the_gate() {
    let extra = "[dependencies]\nchoregraph-core.workspace = true\ntokio.workspace = true\n[dev-dependencies]\n";
    let violations = audit_manifest(extra);
    assert!(
        violations
            .iter()
            .any(|violation| matches!(violation, BoundaryViolation::ProductionDependencyCount(2)))
    );
    assert!(violations.iter().any(|violation| matches!(
        violation,
        BoundaryViolation::ProductionDependency("tokio.workspace = true")
    )));

    assert_eq!(
        audit_manifest("[package]\nname = \"missing-sections\"\n"),
        vec![BoundaryViolation::MissingManifestSection]
    );
}
