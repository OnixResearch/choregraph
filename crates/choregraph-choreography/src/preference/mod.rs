//! Default-off preference annotations on role-choreography `Choice` deciders.
//!
//! An annotated source wraps an unchanged `choregraph.role-choreography.v1`
//! base. The base is admitted and projected by the same functions as the
//! cooperative baseline; annotations only attach declared factor and world
//! metadata to each decider's projected `InternalChoice`.
//!
//! Annotations are metadata. They do not choose a branch, weigh factors, grant
//! authority, convert a label set into a Boolean, or map the protocol to a
//! game. Every report states that the game mapping is not formalized.
//!
//! r[impl choregraph.role_choreography.preference.profile]
//! r[impl choregraph.role_choreography.preference.source]
//! r[impl choregraph.role_choreography.preference.projection]
//! r[impl choregraph.role_choreography.preference.identity]
//! r[impl choregraph.role_choreography.preference.report]
//! r[impl choregraph.role_choreography.preference.boundary]

pub mod admission;
pub mod linkage;
pub mod model;
pub mod projection;

/// Selector string for the default cooperative baseline profile.
pub const BASELINE_PROFILE: &str = "choregraph.role-choreography.profile.cooperative-baseline.v1";

/// Selector string for the explicitly selected preference-annotation profile.
pub const ANNOTATED_PROFILE: &str =
    "choregraph.role-choreography.profile.preference-annotations.v1";

/// Report schema for the review summary of one annotated compilation.
pub const REPORT_SCHEMA: &str = "choregraph.role-choreography.preference-report.v1";

/// Profile chosen by the caller. The cooperative baseline is the default.
#[derive(Clone, Copy, Debug, Default, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename = "PreferenceProfileSelection", rename_all = "kebab-case")]
pub enum ProfileSelection {
    #[default]
    CooperativeBaseline,
    PreferenceAnnotations,
}

impl ProfileSelection {
    #[must_use]
    pub const fn selector(self) -> &'static str {
        if let Self::PreferenceAnnotations = self {
            ANNOTATED_PROFILE
        } else {
            BASELINE_PROFILE
        }
    }

    /// Parses an exact selector string; any other text selects nothing.
    #[must_use]
    pub fn from_selector(selector: &str) -> Option<Self> {
        [Self::CooperativeBaseline, Self::PreferenceAnnotations]
            .into_iter()
            .find(|candidate| candidate.selector() == selector)
    }
}

/// Status of endpoint projection for one annotated compilation.
#[derive(
    Clone, Copy, Debug, serde::Deserialize, Eq, Ord, PartialEq, PartialOrd, serde::Serialize,
)]
#[serde(rename = "PreferenceProjectionStatus", rename_all = "kebab-case")]
pub enum ProjectionStatus {
    Projected,
    Refused,
}

/// Result of one annotated compilation under an explicit profile selection.
#[derive(Debug)]
pub struct Report {
    selection: ProfileSelection,
    outcome: Result<Box<linkage::AnnotatedBundle>, crate::admission::Error>,
}

/// Serializable review summary of a [`Report`].
#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename = "PreferenceReport", deny_unknown_fields)]
pub struct Summary {
    pub schema: String,
    pub profile: String,
    pub status: ProjectionStatus,
    pub game_mapping: linkage::GameMapping,
    pub diagnostics: Vec<crate::issue::DiagnosticCode>,
    pub receipt: Option<crate::linkage::digest::Identity>,
    pub non_claims: Vec<String>,
}

impl Report {
    /// `Projected` when a complete annotated bundle exists, else `Refused`.
    #[must_use]
    pub const fn status(&self) -> ProjectionStatus {
        if self.outcome.is_ok() {
            ProjectionStatus::Projected
        } else {
            ProjectionStatus::Refused
        }
    }

    /// Always `NotFormalized`: no game mapping is checked or claimed.
    #[must_use]
    pub const fn game_mapping(&self) -> linkage::GameMapping {
        linkage::GameMapping::NotFormalized
    }

    #[must_use]
    pub const fn selection(&self) -> ProfileSelection {
        self.selection
    }

    #[must_use]
    pub fn bundle(&self) -> Option<&linkage::AnnotatedBundle> {
        self.outcome.as_deref().ok()
    }

    #[must_use]
    pub fn refusal(&self) -> Option<&crate::admission::Error> {
        self.outcome.as_ref().err()
    }

    #[must_use]
    pub fn summary(&self) -> Summary {
        let diagnostics = if let Some(crate::admission::Error::Diagnostics(items)) = self.refusal()
        {
            items.iter().map(|item| item.code).collect()
        } else {
            Vec::new()
        };
        let summary = Summary {
            schema: String::from(REPORT_SCHEMA),
            profile: String::from(self.selection.selector()),
            status: self.status(),
            game_mapping: self.game_mapping(),
            diagnostics,
            receipt: self
                .bundle()
                .map(|bundle| bundle.receipt().identity.clone()),
            non_claims: model::REQUIRED_NON_CLAIMS.map(String::from).to_vec(),
        };
        debug_assert_eq!(summary.receipt.is_some(), self.outcome.is_ok());
        debug_assert_eq!(summary.game_mapping, linkage::GameMapping::NotFormalized);
        summary
    }
}

fn compile(
    evaluated_bytes: &[u8],
    source_envelope_bytes: &[u8],
    selection: ProfileSelection,
) -> Result<Box<linkage::AnnotatedBundle>, crate::admission::Error> {
    let source = admission::parse_evaluated_json(evaluated_bytes)?;
    let admitted = admission::admit_source(source, selection)?;
    let materials =
        crate::linkage::ProjectionMaterials::reviewed(source_envelope_bytes, evaluated_bytes);
    linkage::build_bundle(&admitted, &materials)
        .map(Box::new)
        .map_err(Into::into)
}

/// Compiles annotated evaluator output under an explicit profile selection.
///
/// `ProfileSelection::CooperativeBaseline` (the default) refuses every input
/// with `PreferenceProfileNotSelected`; use `crate::compile_evaluated` for the
/// unannotated baseline.
#[must_use]
pub fn compile_evaluated(
    evaluated_bytes: &[u8],
    source_envelope_bytes: &[u8],
    selection: ProfileSelection,
) -> Report {
    let outcome = if selection == ProfileSelection::PreferenceAnnotations {
        compile(evaluated_bytes, source_envelope_bytes, selection)
    } else {
        Err(admission::profile_not_selected().into())
    };
    Report { selection, outcome }
}

#[cfg(test)]
mod tests;
