#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the closed serde DTO keeps its serialization contracts visible beside the bounded domain model"
)]

use serde::{Deserialize, Serialize};

pub const ROLE_CHOREOGRAPHY_SCHEMA: &str = "choregraph.role-choreography.v1";
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

pub const REQUIRED_NON_CLAIMS: [&str; 10] = [
    "no-runtime-execution",
    "no-host-placement",
    "no-transport-delivery",
    "no-persistence",
    "no-authorization",
    "no-external-role-correctness",
    "no-exactly-once-transfer",
    "no-deadlock-freedom-under-faults",
    "no-production-readiness",
    "no-release-eligibility",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RoleId(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ValueId(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct LabelId(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct PayloadTag(pub u32);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RoleKind {
    Native,
    External {
        contract_identity: String,
        input_schema_identity: String,
        output_schema_identity: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDeclaration {
    pub id: RoleId,
    pub kind: RoleKind,
    pub span: SourceSpan,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValueDeclaration {
    pub id: ValueId,
    pub payload_tag: PayloadTag,
    pub initial_owner: RoleId,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImportIdentity {
    pub logical_id: String,
    pub blake3: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChoreographyLimits {
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

impl ChoreographyLimits {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceBranch {
    pub label: LabelId,
    pub body: ChoreographyNode,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ChoreographyNode {
    End {
        span: SourceSpan,
    },
    Communication {
        from: RoleId,
        to: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        next: Box<ChoreographyNode>,
        span: SourceSpan,
    },
    Transfer {
        from: RoleId,
        to: RoleId,
        label: LabelId,
        payload_tag: PayloadTag,
        value: ValueId,
        next: Box<ChoreographyNode>,
        span: SourceSpan,
    },
    Choice {
        decider: RoleId,
        branches: Vec<ChoiceBranch>,
        span: SourceSpan,
    },
}

impl ChoreographyNode {
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::End { span }
            | Self::Communication { span, .. }
            | Self::Transfer { span, .. }
            | Self::Choice { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RoleChoreographySource {
    pub schema: String,
    pub limits: ChoreographyLimits,
    pub imports: Vec<ImportIdentity>,
    pub roles: Vec<RoleDeclaration>,
    pub values: Vec<ValueDeclaration>,
    pub protocol: ChoreographyNode,
    pub non_claims: Vec<String>,
}
