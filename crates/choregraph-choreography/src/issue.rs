#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the closed diagnostic DTO keeps its bounded model types together for direct safety review"
)]

use crate::model::{MAX_DIAGNOSTIC_TEXT_BYTES, MAX_DIAGNOSTICS, SourceSpan};
use serde::{Deserialize, Serialize};

const FORBIDDEN_DIAGNOSTIC_FRAGMENTS: [&str; 4] = ["secret", "token", "credential", "environment"];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    Error,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCode {
    MalformedEvaluatedData,
    UnsupportedSchema,
    LimitsMismatch,
    RoleLimitExceeded,
    ValueLimitExceeded,
    StepLimitExceeded,
    BranchLimitExceeded,
    NestingLimitExceeded,
    ImportLimitExceeded,
    IdentifierLimitExceeded,
    DiagnosticLimitExceeded,
    CanonicalOutputLimitExceeded,
    DuplicateRole,
    DuplicateValue,
    DuplicateImport,
    UnsortedImport,
    EmptyRoles,
    UndeclaredRole,
    UnknownValue,
    SelfTransfer,
    StaleOwner,
    PayloadMismatch,
    DuplicateBranchLabel,
    UnsortedBranchLabel,
    EmptyChoice,
    InvalidRoleKind,
    InvalidSpan,
    InvalidImportIdentity,
    MissingNonClaim,
    UnsafeDiagnostic,
    TrellisOwnershipRejected,
    TrellisGlobalRejected,
    TrellisProjectionRejected,
    IncompleteLocalSet,
    DuplicateLocalRole,
    LocalOrderMismatch,
    ArtifactEncodingFailed,
    IdentityMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SafeReplacement {
    pub span: SourceSpan,
    pub replacement: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChoreographyDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub primary: SourceSpan,
    pub secondary: Vec<SourceSpan>,
    pub notes: Vec<String>,
    pub hints: Vec<String>,
    pub safe_replacement: Option<SafeReplacement>,
}

impl ChoreographyDiagnostic {
    #[must_use]
    pub fn error(code: DiagnosticCode, primary: SourceSpan, note: &'static str) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            primary,
            secondary: Vec::new(),
            notes: vec![String::from(note)],
            hints: Vec::new(),
            safe_replacement: None,
        }
    }

    #[must_use]
    pub fn with_secondary(mut self, span: SourceSpan) -> Self {
        self.secondary.push(span);
        self.secondary.sort_unstable();
        self.secondary.dedup();
        self
    }

    #[must_use]
    pub fn with_hint(mut self, hint: &'static str) -> Self {
        self.hints.push(String::from(hint));
        self
    }
}

#[must_use]
pub fn span_is_valid(span: SourceSpan) -> bool {
    span.start <= span.end
}

#[must_use]
pub fn safe_diagnostic_text(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    !text.is_empty()
        && text.len() <= MAX_DIAGNOSTIC_TEXT_BYTES
        && !text.contains('/')
        && !text.contains('\\')
        && !text.contains('=')
        && !FORBIDDEN_DIAGNOSTIC_FRAGMENTS
            .iter()
            .any(|fragment| normalized.contains(fragment))
}

#[must_use]
pub fn diagnostics_are_safe(diagnostics: &[ChoreographyDiagnostic]) -> bool {
    diagnostics.len() <= MAX_DIAGNOSTICS
        && diagnostics.iter().all(|diagnostic| {
            span_is_valid(diagnostic.primary)
                && diagnostic.secondary.iter().copied().all(span_is_valid)
                && diagnostic.secondary.is_sorted()
                && diagnostic.notes.is_sorted()
                && diagnostic.hints.is_sorted()
                && diagnostic
                    .notes
                    .iter()
                    .all(|note| safe_diagnostic_text(note))
                && diagnostic
                    .hints
                    .iter()
                    .all(|hint| safe_diagnostic_text(hint))
                && diagnostic
                    .safe_replacement
                    .as_ref()
                    .is_none_or(|replacement| {
                        span_is_valid(replacement.span)
                            && safe_diagnostic_text(&replacement.replacement)
                    })
        })
}

pub fn normalize_diagnostics(diagnostics: &mut [ChoreographyDiagnostic]) {
    for diagnostic in diagnostics.iter_mut() {
        diagnostic.secondary.sort_unstable();
        diagnostic.secondary.dedup();
        diagnostic.notes.sort_unstable();
        diagnostic.notes.dedup();
        diagnostic.hints.sort_unstable();
        diagnostic.hints.dedup();
    }
    diagnostics.sort_unstable_by_key(|diagnostic| {
        (
            diagnostic.primary.start,
            diagnostic.primary.end,
            diagnostic.code,
        )
    });
}

#[must_use]
pub fn render_safe_diagnostics(diagnostics: &[ChoreographyDiagnostic]) -> String {
    if !diagnostics_are_safe(diagnostics) {
        return String::from("role-choreography: unsafe diagnostic rejected");
    }
    let mut output = String::new();
    for diagnostic in diagnostics {
        let line = format!(
            "role-choreography:{:?}:{}..{}\n",
            diagnostic.code, diagnostic.primary.start, diagnostic.primary.end
        );
        if output.len().saturating_add(line.len()) > MAX_DIAGNOSTIC_TEXT_BYTES {
            break;
        }
        output.push_str(&line);
    }
    output
}
