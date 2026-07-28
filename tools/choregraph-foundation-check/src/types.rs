/// A normalized workspace model consumed by the pure validator.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
pub struct WorkspaceModel {
    pub packages: Vec<PackageModel>,
    pub core_sources: Vec<SourceUnit>,
}

/// A normalized package and its direct dependency declarations.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
pub struct PackageModel {
    pub name: String,
    pub license: String,
    pub rust_version: String,
    pub publish_disabled: bool,
    pub dependencies: Vec<DependencyModel>,
}

/// A normalized direct dependency.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
pub struct DependencyModel {
    pub name: String,
    pub requirement: String,
    pub source: DependencySource,
    pub path: String,
}

/// The acquisition class of one dependency declaration.
#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DependencySource {
    Workspace,
    Registry,
    LocalPath,
    ExternalPath,
}

/// One in-memory core source unit.
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq)]
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
