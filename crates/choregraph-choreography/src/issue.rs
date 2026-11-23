#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the closed diagnostic DTO keeps its bounded model types together for direct safety review"
)]

pub(crate) mod projection;

use crate::model::bounds::{MAX_DIAGNOSTIC_TEXT_BYTES, MAX_DIAGNOSTICS};
use crate::model::participants::SourceSpan;
use serde::{Deserialize, Serialize};

const FORBIDDEN_DIAGNOSTIC_FRAGMENTS: [&str; 4] = ["secret", "token", "credential", "environment"];
const TRUNCATION_MARKER: &str = "role-choreography:truncated\n";

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
    PreferenceProfileNotSelected,
    PreferenceNonClaimMismatch,
    PreferenceLimitExceeded,
    InvalidPreferenceName,
    DuplicatePreferenceFactor,
    UnsortedPreferenceFactor,
    DuplicateWorldVariable,
    UnsortedWorldVariable,
    EmptyPreferenceAnnotations,
    DuplicateChoiceAnnotation,
    UnsortedChoiceAnnotation,
    EmptyChoiceFactors,
    UnsortedChoiceFactors,
    UnknownPreferenceFactor,
    UnknownWorldVariable,
    PreferenceFactorRoleMismatch,
    UnusedPreferenceDeclaration,
    InvalidPreferencePath,
    PreferenceTargetNotChoice,
    StaleChoiceDecider,
    PreferencePathNotLocallyAddressable,
    PreferenceLocalMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SafeReplacement {
    pub span: SourceSpan,
    pub replacement: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "ChoreographyDiagnostic", deny_unknown_fields)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub primary: SourceSpan,
    pub secondary: Vec<SourceSpan>,
    pub notes: Vec<String>,
    pub hints: Vec<String>,
    pub safe_replacement: Option<SafeReplacement>,
}

impl Diagnostic {
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
pub fn diagnostics_are_safe(diagnostics: &[Diagnostic]) -> bool {
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

pub fn normalize_diagnostics(diagnostics: &mut [Diagnostic]) {
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

fn format_secondary_line(secondary: &[SourceSpan]) -> String {
    let mut line = String::from("secondary: ");
    for (index, span) in secondary.iter().enumerate() {
        if index > 0 {
            line.push_str(", ");
        }
        line.push_str(&format!("{}..{}", span.start, span.end));
    }
    line.push('\n');
    debug_assert!(!secondary.is_empty());
    debug_assert!(line.ends_with('\n'));
    line
}

fn collect_render_lines(diagnostics: &[Diagnostic]) -> (Vec<String>, bool) {
    let mut lines = Vec::with_capacity(diagnostics.len().saturating_mul(4));
    let mut total_bytes: usize = 0;
    for diagnostic in diagnostics {
        let header = format!(
            "role-choreography:{:?}:{}..{}\n",
            diagnostic.code, diagnostic.primary.start, diagnostic.primary.end
        );
        total_bytes = total_bytes.saturating_add(header.len());
        lines.push(header);
        if total_bytes > MAX_DIAGNOSTIC_TEXT_BYTES {
            return (lines, true);
        }
        if !diagnostic.secondary.is_empty() {
            let secondary = format_secondary_line(&diagnostic.secondary);
            total_bytes = total_bytes.saturating_add(secondary.len());
            lines.push(secondary);
            if total_bytes > MAX_DIAGNOSTIC_TEXT_BYTES {
                return (lines, true);
            }
        }
        for note in &diagnostic.notes {
            let line = format!("note: {note}\n");
            total_bytes = total_bytes.saturating_add(line.len());
            lines.push(line);
            if total_bytes > MAX_DIAGNOSTIC_TEXT_BYTES {
                return (lines, true);
            }
        }
        for hint in &diagnostic.hints {
            let line = format!("hint: {hint}\n");
            total_bytes = total_bytes.saturating_add(line.len());
            lines.push(line);
            if total_bytes > MAX_DIAGNOSTIC_TEXT_BYTES {
                return (lines, true);
            }
        }
    }
    debug_assert!(total_bytes <= MAX_DIAGNOSTIC_TEXT_BYTES);
    debug_assert!(lines.len() >= diagnostics.len());
    (lines, false)
}

#[must_use]
pub fn render_safe_diagnostics(diagnostics: &[Diagnostic]) -> String {
    if !diagnostics_are_safe(diagnostics) {
        return String::from("role-choreography: unsafe diagnostic rejected");
    }
    let (lines, truncated) = collect_render_lines(diagnostics);
    let mut output = String::new();
    if !truncated {
        for line in lines {
            output.push_str(&line);
        }
        debug_assert!(output.len() <= MAX_DIAGNOSTIC_TEXT_BYTES);
        debug_assert!(!output.ends_with(TRUNCATION_MARKER));
        return output;
    }
    let budget = MAX_DIAGNOSTIC_TEXT_BYTES.saturating_sub(TRUNCATION_MARKER.len());
    for line in lines {
        if output.len().saturating_add(line.len()) > budget {
            break;
        }
        output.push_str(&line);
    }
    output.push_str(TRUNCATION_MARKER);
    debug_assert!(output.len() <= MAX_DIAGNOSTIC_TEXT_BYTES);
    debug_assert!(output.ends_with(TRUNCATION_MARKER));
    output
}
