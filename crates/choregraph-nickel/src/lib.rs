//! Pure revalidation and lowering for evaluated restricted Nickel data.
//!
//! Source evaluation and capability-relative file access remain shell responsibilities.

mod diagnostic;
mod identity;
mod imports;
mod lower;
mod model;
mod profile;
mod source;

pub use diagnostic::{
    FrontendFailure, FrontendIssue, FrontendIssueCode, MAX_SAFE_SUMMARY_BYTES,
    redact_evaluator_error, summary,
};
pub use identity::{
    EvaluatorReceipt, SourceReceipt, TranslationDomain, TranslationIdentity,
    TranslationIdentityChain, TranslationMaterials, TranslationReceipts,
    build_translation_receipts, validate_translation_receipts,
};
pub use imports::{ObservedImport, ValidatedImportClosure, validate_import_closure};
pub use lower::{LoweredSourceFormat, LoweringOutput, lower_evaluated_json, parse_evaluated_json};
pub use model::{
    CHOREGRAPH_EVALUATED_SCHEMA, EvaluatedEdge, EvaluatedEffect, EvaluatedLimits, EvaluatedNode,
    EvaluatedSource, ImportDeclaration, ImportMediaClass, LATTICE_EVALUATED_SCHEMA, LatticeNode,
    LatticeNodeConfig, LatticePort, LatticePortDirection, LatticePortEdge,
    RESTRICTED_EVALUATOR_PROFILE,
};
pub use profile::{
    DEFAULT_MAX_EVALUATED_BYTES, DEFAULT_MAX_IMPORT_BYTES, DEFAULT_MAX_IMPORT_IDENTIFIER_BYTES,
    DEFAULT_MAX_IMPORT_PATH_BYTES, DEFAULT_MAX_IMPORTS, DEFAULT_MAX_TOTAL_IMPORT_BYTES,
    FORBIDDEN_EVALUATOR_AUTHORITIES, ForbiddenEvaluatorAuthority, FrontendLimits,
    reviewed_frontend_limits,
};
pub use source::validate_restricted;

/// The stage of input offered to the restricted lowering boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringInputStage {
    EvaluatedData,
    NickelSource,
    DynamicImport,
    EnvironmentValue,
}

#[must_use]
pub const fn output_capability() -> choregraph_core::Capability {
    choregraph_core::Capability::ImmutableMachineData
}

#[must_use]
pub const fn accepts(stage: LoweringInputStage) -> bool {
    matches!(stage, LoweringInputStage::EvaluatedData)
}

#[cfg(test)]
mod tests;
