use serde::{Deserialize, Serialize};

pub const ROLE_CHOREOGRAPHY_SCHEMA: &str = "choregraph.role-choreography.v1";

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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "RoleChoreographySource", deny_unknown_fields)]
pub struct RoleSource {
    pub schema: String,
    pub limits: super::bounds::Limits,
    pub imports: Vec<super::participants::ImportIdentity>,
    pub roles: Vec<super::participants::RoleDeclaration>,
    pub values: Vec<super::participants::ValueDeclaration>,
    pub protocol: super::tree::Node,
    pub non_claims: Vec<String>,
}
