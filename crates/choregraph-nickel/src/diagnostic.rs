// Bounded diagnostics keep stable exhaustive codes and one redaction path.
#![allow(
    unknown_lints,
    fragile_exhaustive_enum_match,
    function_length,
    non_trait_imports,
    reason = "bounded diagnostics keep stable exhaustive codes and one redaction path"
)]

// r[impl choregraph.nickel.diagnostics]
use serde::{Deserialize, Serialize};

pub const MAX_SAFE_SUMMARY_BYTES: usize = 160;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrontendIssueCode {
    EvaluatedInputTooLarge,
    MalformedEvaluatedValue,
    EvaluatorRejected,
    UnsupportedSourceFormat,
    UnsupportedSchema,
    UnsupportedLifecycle,
    UnsupportedEvents,
    InvalidLimits,
    MachineLimitExceedsProfile,
    TooManyImports,
    ImportTotalBytesExceeded,
    InvalidImportIdentifier,
    InvalidImportPath,
    UnsupportedImportMediaClass,
    InvalidImportHash,
    DuplicateImportDeclaration,
    DuplicateImportObservation,
    MissingImportObservation,
    UndeclaredImportObservation,
    ImportByteLimitExceeded,
    ImportHashMismatch,
    InvalidLatticePort,
    DuplicateLatticePort,
    MissingLatticePort,
    WrongLatticePortDirection,
    DuplicateLatticeConfig,
    MissingLatticeConfig,
    UnknownLatticeConfigNode,
    MachineAdmissionRejected,
    EmptyIdentityMaterial,
    StaleTranslationReceipt,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FrontendIssue {
    pub(crate) code: FrontendIssueCode,
    pub(crate) summary: &'static str,
}

impl FrontendIssue {
    #[must_use]
    pub const fn new(code: FrontendIssueCode) -> Self {
        Self {
            code,
            summary: summary(code),
        }
    }

    #[must_use]
    pub const fn code(self) -> FrontendIssueCode {
        self.code
    }

    #[must_use]
    pub const fn summary(self) -> &'static str {
        self.summary
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontendFailure {
    pub issues: Vec<FrontendIssue>,
}

impl FrontendFailure {
    #[must_use]
    pub fn one(code: FrontendIssueCode) -> Self {
        Self {
            issues: vec![FrontendIssue::new(code)],
        }
    }
}

#[must_use]
pub fn redact_evaluator_error(_raw_error: &str) -> FrontendIssue {
    FrontendIssue::new(FrontendIssueCode::EvaluatorRejected)
}

#[must_use]
pub const fn summary(code: FrontendIssueCode) -> &'static str {
    match code {
        FrontendIssueCode::EvaluatedInputTooLarge => {
            "evaluated input exceeds the selected byte limit"
        }
        FrontendIssueCode::MalformedEvaluatedValue => {
            "evaluated input does not match the closed data contract"
        }
        FrontendIssueCode::EvaluatorRejected => {
            "restricted evaluator rejected the selected source input"
        }
        FrontendIssueCode::UnsupportedSourceFormat => {
            "evaluated input uses an unsupported source format"
        }
        FrontendIssueCode::UnsupportedSchema => "evaluated input uses an unsupported schema",
        FrontendIssueCode::UnsupportedLifecycle => {
            "evaluated input uses an unsupported lifecycle profile"
        }
        FrontendIssueCode::UnsupportedEvents => {
            "evaluated input does not declare the exact lifecycle event set"
        }
        FrontendIssueCode::InvalidLimits => "evaluated input contains an invalid limit policy",
        FrontendIssueCode::MachineLimitExceedsProfile => {
            "machine limits exceed the frontend ceiling"
        }
        FrontendIssueCode::TooManyImports => "declared import count exceeds the selected limit",
        FrontendIssueCode::ImportTotalBytesExceeded => {
            "declared import bytes exceed the selected total limit"
        }
        FrontendIssueCode::InvalidImportIdentifier => {
            "import identifier is invalid or exceeds its limit"
        }
        FrontendIssueCode::InvalidImportPath => {
            "import path is outside the restricted relative-path profile"
        }
        FrontendIssueCode::UnsupportedImportMediaClass => "import media class is not supported",
        FrontendIssueCode::InvalidImportHash => "import identity is not a lowercase BLAKE3 value",
        FrontendIssueCode::DuplicateImportDeclaration => {
            "import closure contains a duplicate declaration"
        }
        FrontendIssueCode::DuplicateImportObservation => {
            "import closure contains a duplicate observation"
        }
        FrontendIssueCode::MissingImportObservation => "declared import has no observed bytes",
        FrontendIssueCode::UndeclaredImportObservation => {
            "observed import is not in the declared closure"
        }
        FrontendIssueCode::ImportByteLimitExceeded => {
            "observed import exceeds its declared byte limit"
        }
        FrontendIssueCode::ImportHashMismatch => {
            "observed import bytes do not match the declared BLAKE3 value"
        }
        FrontendIssueCode::InvalidLatticePort => "Lattice port declaration is invalid",
        FrontendIssueCode::DuplicateLatticePort => "Lattice port identifier is duplicated",
        FrontendIssueCode::MissingLatticePort => "Lattice edge references an unknown port",
        FrontendIssueCode::WrongLatticePortDirection => {
            "Lattice edge does not connect output to input"
        }
        FrontendIssueCode::DuplicateLatticeConfig => {
            "Lattice node has duplicate operation configuration"
        }
        FrontendIssueCode::MissingLatticeConfig => "Lattice node has no operation configuration",
        FrontendIssueCode::UnknownLatticeConfigNode => {
            "Lattice operation configuration references an unknown node"
        }
        FrontendIssueCode::MachineAdmissionRejected => {
            "lowered graph failed canonical machine admission"
        }
        FrontendIssueCode::EmptyIdentityMaterial => "translation identity material is empty",
        FrontendIssueCode::StaleTranslationReceipt => {
            "translation receipt does not match current identity material"
        }
    }
}
