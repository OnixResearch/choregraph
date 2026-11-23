//! Pure admission of annotated sources under an explicitly selected profile.
//!
//! The v1 base is admitted by the unchanged `crate::admission::admit_source`.
//! Annotation checks run only after that base is admitted.

use serde::Deserialize;

pub const MAX_FACTORS: usize = 256;
pub const MAX_WORLD_VARIABLES: usize = 64;
pub const MAX_CHOICE_ANNOTATIONS: usize = 256;
pub const MAX_FACTORS_PER_CHOICE: usize = 64;
pub const MAX_PATH_STEPS: usize = crate::model::bounds::MAX_NESTING_DEPTH;
pub const MAX_NAME_BYTES: usize = 64;

/// An admitted v1 base plus validated, normalized preference annotations.
#[derive(Debug)]
pub struct Admitted {
    pub(crate) base: crate::admission::Admitted,
    pub(crate) annotations: super::model::AnnotationSet,
    pub(crate) non_claims: Vec<String>,
}

impl Admitted {
    /// The admitted cooperative base used for Trellis endpoint projection.
    #[must_use]
    pub const fn base(&self) -> &crate::admission::Admitted {
        &self.base
    }

    #[must_use]
    pub const fn annotations(&self) -> &super::model::AnnotationSet {
        &self.annotations
    }
}

#[derive(Deserialize)]
struct SchemaProbe {
    schema: String,
}

fn issue(
    code: crate::issue::DiagnosticCode,
    span: crate::model::participants::SourceSpan,
    note: &'static str,
) -> Vec<crate::issue::Diagnostic> {
    vec![crate::issue::Diagnostic::error(code, span, note)]
}

const fn origin() -> crate::model::participants::SourceSpan {
    crate::model::participants::SourceSpan { start: 0, end: 0 }
}

fn profile_refusal(span: crate::model::participants::SourceSpan) -> Vec<crate::issue::Diagnostic> {
    vec![
        crate::issue::Diagnostic::error(
            crate::issue::DiagnosticCode::PreferenceProfileNotSelected,
            span,
            "preference annotations require the explicitly selected annotation profile",
        )
        .with_hint("the cooperative baseline admits only unannotated v1 sources"),
    ]
}

/// The refusal returned when the preference profile is not selected.
#[must_use]
pub fn profile_not_selected() -> Vec<crate::issue::Diagnostic> {
    profile_refusal(origin())
}

/// Returns the profile refusal when evaluated bytes carry the annotated schema.
///
/// The cooperative baseline uses this to refuse annotated input precisely
/// instead of reporting a generic unknown-field failure.
#[must_use]
pub fn refusal_without_selection(input: &[u8]) -> Option<Vec<crate::issue::Diagnostic>> {
    let probe: SchemaProbe = serde_json::from_slice(input).ok()?;
    (probe.schema == super::model::ANNOTATED_SCHEMA).then(profile_not_selected)
}

/// Parses strict annotated evaluator output; unknown fields are rejected.
pub fn parse_evaluated_json(
    input: &[u8],
) -> Result<super::model::AnnotatedSource, Vec<crate::issue::Diagnostic>> {
    serde_json::from_slice(input).map_err(|_| {
        issue(
            crate::issue::DiagnosticCode::MalformedEvaluatedData,
            origin(),
            "annotated choreography data is malformed or has unknown fields",
        )
    })
}

fn valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let has_bounded_length = !bytes.is_empty() && bytes.len() <= MAX_NAME_BYTES;
    has_bounded_length
        && bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

/// Rejects an inverted span. The diagnostic uses the origin span because an
/// inverted span is itself unsafe to report.
fn validate_span(
    span: crate::model::participants::SourceSpan,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    if crate::issue::span_is_valid(span) {
        Ok(())
    } else {
        Err(issue(
            crate::issue::DiagnosticCode::InvalidSpan,
            origin(),
            "annotation source span end precedes its start",
        ))
    }
}

fn validate_non_claims(
    non_claims: &[String],
    root: crate::model::participants::SourceSpan,
) -> Result<Vec<String>, Vec<crate::issue::Diagnostic>> {
    let mut sorted = non_claims.to_vec();
    sorted.sort_unstable();
    let is_exact = sorted.len() == super::model::REQUIRED_NON_CLAIMS.len()
        && sorted
            .iter()
            .zip(super::model::REQUIRED_NON_CLAIMS)
            .all(|(declared, required)| declared == required);
    if !is_exact {
        return Err(issue(
            crate::issue::DiagnosticCode::PreferenceNonClaimMismatch,
            root,
            "annotation non-claims must match the reviewed closed set",
        ));
    }
    debug_assert!(sorted.is_sorted());
    debug_assert_eq!(sorted.len(), super::model::REQUIRED_NON_CLAIMS.len());
    Ok(sorted)
}

fn validate_counts(
    annotations: &super::model::AnnotationSet,
    root: crate::model::participants::SourceSpan,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let is_bounded =
        annotations.factors.len() <= MAX_FACTORS && annotations.worlds.len() <= MAX_WORLD_VARIABLES;
    if !is_bounded || annotations.choices.len() > MAX_CHOICE_ANNOTATIONS {
        return Err(issue(
            crate::issue::DiagnosticCode::PreferenceLimitExceeded,
            root,
            "annotation count exceeds the reviewed limit",
        ));
    }
    if annotations.choices.is_empty() {
        return Err(issue(
            crate::issue::DiagnosticCode::EmptyPreferenceAnnotations,
            root,
            "annotated source annotates no choice",
        ));
    }
    debug_assert!(annotations.factors.len() <= MAX_FACTORS);
    debug_assert!(annotations.choices.len() <= MAX_CHOICE_ANNOTATIONS);
    Ok(())
}

fn validate_factor_order(
    factors: &[super::model::FactorDeclaration],
    index: usize,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let (Some(factor), Some(earlier)) = (factors.get(index), factors.get(..index)) else {
        return Ok(());
    };
    let is_duplicate = earlier
        .last()
        .is_some_and(|previous| previous.id == factor.id)
        || earlier
            .iter()
            .any(|previous| previous.role == factor.role && previous.name == factor.name);
    if is_duplicate {
        return Err(issue(
            crate::issue::DiagnosticCode::DuplicatePreferenceFactor,
            factor.span,
            "factor identity or role-local name is duplicated",
        ));
    }
    if earlier
        .last()
        .is_some_and(|previous| previous.id > factor.id)
    {
        return Err(issue(
            crate::issue::DiagnosticCode::UnsortedPreferenceFactor,
            factor.span,
            "factors are not in strict identity order",
        ));
    }
    debug_assert!(earlier.iter().all(|previous| previous.id < factor.id));
    debug_assert!(index < factors.len());
    Ok(())
}

fn validate_factor(
    factor: &super::model::FactorDeclaration,
    roles: &[crate::model::participants::RoleDeclaration],
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    validate_span(factor.span)?;
    if !valid_name(&factor.name) {
        return Err(issue(
            crate::issue::DiagnosticCode::InvalidPreferenceName,
            factor.span,
            "factor name must be lowercase kebab text within the reviewed size",
        ));
    }
    if roles.iter().any(|role| role.id == factor.role) {
        Ok(())
    } else {
        Err(issue(
            crate::issue::DiagnosticCode::UndeclaredRole,
            factor.span,
            "factor names an undeclared role",
        ))
    }
}

fn validate_world(
    worlds: &[super::model::WorldDeclaration],
    index: usize,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let (Some(world), Some(earlier)) = (worlds.get(index), worlds.get(..index)) else {
        return Ok(());
    };
    validate_span(world.span)?;
    if !valid_name(&world.name) {
        return Err(issue(
            crate::issue::DiagnosticCode::InvalidPreferenceName,
            world.span,
            "world variable name must be lowercase kebab text within the reviewed size",
        ));
    }
    let is_duplicate = earlier
        .last()
        .is_some_and(|previous| previous.id == world.id)
        || earlier.iter().any(|previous| previous.name == world.name);
    if is_duplicate {
        return Err(issue(
            crate::issue::DiagnosticCode::DuplicateWorldVariable,
            world.span,
            "world variable identity or name is duplicated",
        ));
    }
    if earlier
        .last()
        .is_some_and(|previous| previous.id > world.id)
    {
        return Err(issue(
            crate::issue::DiagnosticCode::UnsortedWorldVariable,
            world.span,
            "world variables are not in strict identity order",
        ));
    }
    debug_assert!(earlier.iter().all(|previous| previous.id < world.id));
    debug_assert!(valid_name(&world.name));
    Ok(())
}

/// Follows one structural path step through the global source tree.
pub(crate) fn next_node(
    node: &crate::model::tree::Node,
    step: super::model::PathStep,
) -> Option<&crate::model::tree::Node> {
    match (node, step) {
        (
            crate::model::tree::Node::Communication { next, .. }
            | crate::model::tree::Node::Transfer { next, .. },
            super::model::PathStep::Next,
        ) => Some(next),
        (
            crate::model::tree::Node::Choice { branches, .. },
            super::model::PathStep::Branch { label },
        ) => branches
            .iter()
            .find(|branch| branch.label == label)
            .map(|branch| &branch.body),
        _other => None,
    }
}

fn validate_target(
    protocol: &crate::model::tree::Node,
    choice: &super::model::ChoiceAnnotation,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    if choice.path.len() > MAX_PATH_STEPS {
        return Err(issue(
            crate::issue::DiagnosticCode::PreferenceLimitExceeded,
            choice.span,
            "annotation path exceeds the reviewed nesting limit",
        ));
    }
    let mut node = protocol;
    for step in &choice.path {
        node = next_node(node, *step).ok_or_else(|| {
            issue(
                crate::issue::DiagnosticCode::InvalidPreferencePath,
                choice.span,
                "annotation path step does not match the protocol tree",
            )
        })?;
    }
    let crate::model::tree::Node::Choice { decider, .. } = node else {
        return Err(issue(
            crate::issue::DiagnosticCode::PreferenceTargetNotChoice,
            choice.span,
            "annotation path does not end at a choice",
        ));
    };
    if *decider != choice.decider {
        return Err(issue(
            crate::issue::DiagnosticCode::StaleChoiceDecider,
            choice.span,
            "annotation decider differs from the choice decider",
        ));
    }
    debug_assert!(choice.path.len() <= MAX_PATH_STEPS);
    debug_assert_eq!(*decider, choice.decider);
    Ok(())
}

fn validate_choice_factor(
    annotations: &super::model::AnnotationSet,
    choice: &super::model::ChoiceAnnotation,
    factor: super::model::FactorId,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let Some(declaration) = annotations.factors.iter().find(|item| item.id == factor) else {
        return Err(issue(
            crate::issue::DiagnosticCode::UnknownPreferenceFactor,
            choice.span,
            "choice annotation names an undeclared factor",
        ));
    };
    if declaration.role == choice.decider {
        Ok(())
    } else {
        Err(issue(
            crate::issue::DiagnosticCode::PreferenceFactorRoleMismatch,
            choice.span,
            "factor belongs to a role other than the choice decider",
        ))
    }
}

fn validate_choice_factors(
    annotations: &super::model::AnnotationSet,
    choice: &super::model::ChoiceAnnotation,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    if choice.factors.is_empty() || choice.factors.len() > MAX_FACTORS_PER_CHOICE {
        return Err(issue(
            crate::issue::DiagnosticCode::EmptyChoiceFactors,
            choice.span,
            "choice annotation needs between one and the reviewed maximum factors",
        ));
    }
    if !choice.factors.is_sorted_by(|left, right| left < right) {
        return Err(issue(
            crate::issue::DiagnosticCode::UnsortedChoiceFactors,
            choice.span,
            "choice factors are duplicated or not in strict order",
        ));
    }
    for factor in &choice.factors {
        validate_choice_factor(annotations, choice, *factor)?;
    }
    let is_world_declared = choice
        .world
        .is_none_or(|world| annotations.worlds.iter().any(|item| item.id == world));
    if !is_world_declared {
        return Err(issue(
            crate::issue::DiagnosticCode::UnknownWorldVariable,
            choice.span,
            "choice annotation names an undeclared world variable",
        ));
    }
    debug_assert!(!choice.factors.is_empty());
    debug_assert!(choice.factors.len() <= MAX_FACTORS_PER_CHOICE);
    Ok(())
}

fn validate_choice(
    annotations: &super::model::AnnotationSet,
    index: usize,
    protocol: &crate::model::tree::Node,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let Some(choice) = annotations.choices.get(index) else {
        return Ok(());
    };
    validate_span(choice.span)?;
    let previous = index
        .checked_sub(1)
        .and_then(|earlier| annotations.choices.get(earlier));
    if previous.is_some_and(|item| item.path == choice.path) {
        return Err(issue(
            crate::issue::DiagnosticCode::DuplicateChoiceAnnotation,
            choice.span,
            "choice is annotated more than once",
        ));
    }
    if previous.is_some_and(|item| item.path > choice.path) {
        return Err(issue(
            crate::issue::DiagnosticCode::UnsortedChoiceAnnotation,
            choice.span,
            "choice annotations are not in strict path order",
        ));
    }
    debug_assert!(previous.is_none_or(|item| item.path < choice.path));
    debug_assert!(crate::issue::span_is_valid(choice.span));
    validate_target(protocol, choice)?;
    validate_choice_factors(annotations, choice)
}

fn validate_usage(
    annotations: &super::model::AnnotationSet,
    root: crate::model::participants::SourceSpan,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    let is_factor_used = |id: super::model::FactorId| {
        annotations
            .choices
            .iter()
            .any(|choice| choice.factors.contains(&id))
    };
    let is_world_used = |id: super::model::WorldId| {
        annotations
            .choices
            .iter()
            .any(|choice| choice.world == Some(id))
    };
    let is_fully_used = annotations
        .factors
        .iter()
        .all(|factor| is_factor_used(factor.id))
        && annotations
            .worlds
            .iter()
            .all(|world| is_world_used(world.id));
    if !is_fully_used {
        return Err(issue(
            crate::issue::DiagnosticCode::UnusedPreferenceDeclaration,
            root,
            "every declared factor and world variable must annotate a choice",
        ));
    }
    debug_assert!(
        annotations
            .factors
            .iter()
            .all(|factor| is_factor_used(factor.id))
    );
    debug_assert!(
        annotations
            .worlds
            .iter()
            .all(|world| is_world_used(world.id))
    );
    Ok(())
}

fn validate_annotations(
    annotations: &super::model::AnnotationSet,
    base: &crate::admission::Admitted,
    root: crate::model::participants::SourceSpan,
) -> Result<(), Vec<crate::issue::Diagnostic>> {
    validate_counts(annotations, root)?;
    for (index, factor) in annotations.factors.iter().enumerate() {
        validate_factor(factor, &base.source.roles)?;
        validate_factor_order(&annotations.factors, index)?;
    }
    for index in 0..annotations.worlds.len() {
        validate_world(&annotations.worlds, index)?;
    }
    for index in 0..annotations.choices.len() {
        validate_choice(annotations, index, &base.source.protocol)?;
    }
    validate_usage(annotations, root)?;
    debug_assert!(annotations.factors.is_sorted_by_key(|factor| factor.id));
    debug_assert!(annotations.choices.len() <= MAX_CHOICE_ANNOTATIONS);
    Ok(())
}

/// Admits an annotated source only under the explicitly selected profile.
///
/// The cooperative baseline selection refuses every annotated source before
/// any other check runs.
pub fn admit_source(
    source: super::model::AnnotatedSource,
    selection: super::ProfileSelection,
) -> Result<Admitted, crate::admission::Error> {
    let root = source
        .base
        .protocol
        .span()
        .ok_or(crate::admission::Error::UnsupportedNode)?;
    if selection != super::ProfileSelection::PreferenceAnnotations {
        return Err(profile_refusal(root).into());
    }
    if source.schema != super::model::ANNOTATED_SCHEMA {
        return Err(issue(
            crate::issue::DiagnosticCode::UnsupportedSchema,
            root,
            "annotated choreography schema is unsupported",
        )
        .into());
    }
    let non_claims = validate_non_claims(&source.non_claims, root)?;
    let base = crate::admission::admit_source(source.base)?;
    validate_annotations(&source.annotations, &base, root)?;
    debug_assert_eq!(non_claims.len(), super::model::REQUIRED_NON_CLAIMS.len());
    debug_assert!(!source.annotations.choices.is_empty());
    Ok(Admitted {
        base,
        annotations: source.annotations,
        non_claims,
    })
}
