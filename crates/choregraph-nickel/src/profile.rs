// The closed profile publishes explicit reviewed defaults through the existing API.
#![allow(
    unknown_lints,
    explicit_defaults,
    non_trait_imports,
    reason = "the closed profile publishes explicit reviewed defaults through the existing API"
)]

// r[impl choregraph.nickel.profile]
use choregraph_core::AdmissionLimits;

pub const DEFAULT_MAX_EVALUATED_BYTES: usize = 1_048_576;
pub const DEFAULT_MAX_IMPORTS: usize = 128;
pub const DEFAULT_MAX_IMPORT_BYTES: usize = 1_048_576;
pub const DEFAULT_MAX_TOTAL_IMPORT_BYTES: usize = 8_388_608;
pub const DEFAULT_MAX_IMPORT_IDENTIFIER_BYTES: usize = 128;
pub const DEFAULT_MAX_IMPORT_PATH_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendLimits {
    pub max_evaluated_bytes: usize,
    pub max_imports: usize,
    pub max_import_bytes: usize,
    pub max_total_import_bytes: usize,
    pub max_import_identifier_bytes: usize,
    pub max_import_path_bytes: usize,
    pub machine_ceiling: AdmissionLimits,
}

impl Default for FrontendLimits {
    fn default() -> Self {
        Self {
            max_evaluated_bytes: DEFAULT_MAX_EVALUATED_BYTES,
            max_imports: DEFAULT_MAX_IMPORTS,
            max_import_bytes: DEFAULT_MAX_IMPORT_BYTES,
            max_total_import_bytes: DEFAULT_MAX_TOTAL_IMPORT_BYTES,
            max_import_identifier_bytes: DEFAULT_MAX_IMPORT_IDENTIFIER_BYTES,
            max_import_path_bytes: DEFAULT_MAX_IMPORT_PATH_BYTES,
            machine_ceiling: AdmissionLimits::default(),
        }
    }
}

#[must_use]
pub fn reviewed_frontend_limits() -> FrontendLimits {
    FrontendLimits::default()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForbiddenEvaluatorAuthority {
    Network,
    Process,
    Clock,
    Environment,
    UndeclaredImport,
    AmbientPath,
    SecretDiscovery,
}

pub const FORBIDDEN_EVALUATOR_AUTHORITIES: [ForbiddenEvaluatorAuthority; 7] = [
    ForbiddenEvaluatorAuthority::Network,
    ForbiddenEvaluatorAuthority::Process,
    ForbiddenEvaluatorAuthority::Clock,
    ForbiddenEvaluatorAuthority::Environment,
    ForbiddenEvaluatorAuthority::UndeclaredImport,
    ForbiddenEvaluatorAuthority::AmbientPath,
    ForbiddenEvaluatorAuthority::SecretDiscovery,
];
