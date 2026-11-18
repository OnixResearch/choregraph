use serde::{Deserialize, Serialize};

pub const MAX_ROLES: usize = 64;
pub const MAX_VALUES: usize = 256;
pub const MAX_STEPS: usize = 4_096;
pub const MAX_BRANCHES: usize = 256;
pub const MAX_NESTING_DEPTH: usize = 128;
pub const MAX_IDENTIFIER_BYTES: usize = 256;
pub const MAX_IMPORTS: usize = 128;
pub const MAX_DIAGNOSTICS: usize = 128;
pub const MAX_DIAGNOSTIC_TEXT_BYTES: usize = 512;
pub const MAX_CANONICAL_OUTPUT_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "ChoreographyLimits", deny_unknown_fields)]
pub struct Limits {
    pub max_roles: usize,
    pub max_values: usize,
    pub max_steps: usize,
    pub max_branches: usize,
    pub max_nesting_depth: usize,
    pub max_identifier_bytes: usize,
    pub max_imports: usize,
    pub max_diagnostics: usize,
    pub max_diagnostic_text_bytes: usize,
    pub max_canonical_output_bytes: usize,
}

impl Limits {
    #[must_use]
    pub const fn reviewed() -> Self {
        Self {
            max_roles: MAX_ROLES,
            max_values: MAX_VALUES,
            max_steps: MAX_STEPS,
            max_branches: MAX_BRANCHES,
            max_nesting_depth: MAX_NESTING_DEPTH,
            max_identifier_bytes: MAX_IDENTIFIER_BYTES,
            max_imports: MAX_IMPORTS,
            max_diagnostics: MAX_DIAGNOSTICS,
            max_diagnostic_text_bytes: MAX_DIAGNOSTIC_TEXT_BYTES,
            max_canonical_output_bytes: MAX_CANONICAL_OUTPUT_BYTES,
        }
    }
}
