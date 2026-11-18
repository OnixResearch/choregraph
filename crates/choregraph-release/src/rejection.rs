// Closed rejection vocabulary shared by release operations.
use alloc::vec;
use alloc::vec::Vec;

use crate::profile::Role;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IssueCode {
    ArtifactCountMismatch,
    MissingRole,
    DuplicateRole,
    InvalidRelativePath,
    UnsupportedSchema,
    InvalidStatus,
    BundleSchemaMismatch,
    BundleProfileMismatch,
    NonClaimMismatch,
    ArtifactMismatch,
    BundleIdentityMismatch,
    InvalidSourceRevision,
    InvalidRollbackRevision,
    InvalidArchiveIdentity,
    LicenseMismatch,
    PackageSetMismatch,
    AcquisitionAdapterMismatch,
    CandidateMismatch,
    ProjectionMismatch,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Issue {
    pub code: IssueCode,
    pub role: Option<Role>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Failure {
    pub issues: Vec<Issue>,
}

impl Failure {
    pub(crate) fn one(code: IssueCode, role: Option<Role>) -> Self {
        Self {
            issues: vec![Issue { code, role }],
        }
    }
}
