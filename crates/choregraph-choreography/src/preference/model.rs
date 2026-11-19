//! Strict source DTOs for the default-off preference-annotation profile.
//!
//! Factors are declared names only. They carry no weights, utility function,
//! solver input, branch choice, Boolean label conversion, or authority.

use serde::{Deserialize, Serialize};

/// Exact `schema` value of an annotated source.
pub const ANNOTATED_SCHEMA: &str = "choregraph.role-choreography.preference-annotations.v1";

/// Closed, exact non-claim set every annotated source must declare.
pub const REQUIRED_NON_CLAIMS: [&str; 9] = [
    "no-branch-selection",
    "no-equilibrium-analysis",
    "no-game-semantics",
    "no-label-boolean-conversion",
    "no-preference-authority",
    "no-self-interested-projection-soundness",
    "no-solver",
    "no-utility-function",
    "no-world-distribution",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct FactorId(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct WorldId(pub u32);

/// One step from a protocol node to its continuation.
///
/// Communication and transfer nodes continue through `next`; choice nodes
/// continue through the branch with the exact `label`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(
    rename = "ChoicePathStep",
    tag = "kind",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum PathStep {
    Next,
    Branch {
        label: crate::model::participants::LabelId,
    },
}

/// A named preference factor declared by one role (Pact-style `values`).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "PreferenceFactor", deny_unknown_fields)]
pub struct FactorDeclaration {
    pub id: FactorId,
    pub role: crate::model::participants::RoleId,
    pub name: String,
    pub span: crate::model::participants::SourceSpan,
}

/// A named exogenous world variable. No distribution or outcome set is declared.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "WorldVariable", deny_unknown_fields)]
pub struct WorldDeclaration {
    pub id: WorldId,
    pub name: String,
    pub span: crate::model::participants::SourceSpan,
}

/// Declared factors, and at most one world variable, on one `Choice` decider.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "ChoiceAnnotation", deny_unknown_fields)]
pub struct ChoiceAnnotation {
    pub path: Vec<PathStep>,
    pub decider: crate::model::participants::RoleId,
    pub factors: Vec<FactorId>,
    pub world: Option<WorldId>,
    pub span: crate::model::participants::SourceSpan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "PreferenceAnnotationSet", deny_unknown_fields)]
pub struct AnnotationSet {
    pub factors: Vec<FactorDeclaration>,
    pub worlds: Vec<WorldDeclaration>,
    pub choices: Vec<ChoiceAnnotation>,
}

/// Annotated source: an unchanged v1 base plus a separate annotation set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "AnnotatedRoleChoreographySource", deny_unknown_fields)]
pub struct AnnotatedSource {
    pub schema: String,
    pub base: crate::model::schema::RoleSource,
    pub annotations: AnnotationSet,
    pub non_claims: Vec<String>,
}
