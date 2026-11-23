//! Opt-in schema, exact rational probability annotations, bounds, receipts, and
//! closed non-claims for the synchronous role-protocol quantitative pilot.

use std::fmt;

use choregraph_choreography::model::participants::{LabelId, RoleId};
use serde::{Deserialize, Serialize};

pub const QUANTITATIVE_PILOT_SCHEMA: &str = "choregraph.role-choreography.quantitative-pilot.v1";
pub const QUANTITATIVE_PILOT_PROFILE: &str =
    "choregraph.role-choreography.profile.quantitative-mdp-pilot.v1";
pub const CLAIM_SCOPE: &str = "synchronous-source-model-pilot-only-not-asynchronous-runtime";
pub const PINNED_PRISM_NIXPKGS_REV: &str = "20535e48e12c86043b577b8518234ff5dbb26957";
pub const PINNED_PRISM_VERSION: &str = "4.9";
pub const PINNED_PRISM_LICENSE: &str = "GPL-2.0-or-later";

pub const DEFAULT_MAX_STATES: usize = 4_096;
pub const DEFAULT_MAX_TRANSITIONS: usize = 16_384;
pub const DEFAULT_TIMEOUT_MS: u64 = 15_000;

/// Closed non-claims attached to every quantitative pilot report.
pub const REQUIRED_NON_CLAIMS: [&str; 6] = [
    "no-asynchronous-delivery-semantics",
    "no-faults-or-retries-modeled",
    "no-dynamic-membership-modeled",
    "no-real-world-probability-calibration",
    "no-implementation-correspondence-theorem",
    "no-runtime-execution-or-authority-enabled",
];

/// Exact nonnegative rational probability `numerator / denominator` in `[0, 1]`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RationalProb {
    pub numerator: u64,
    pub denominator: u64,
}

impl RationalProb {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, AnalysisError> {
        if denominator == 0 {
            return Err(AnalysisError::InvalidProbability(String::from(
                "denominator must be positive",
            )));
        }
        if numerator > denominator {
            return Err(AnalysisError::InvalidProbability(format!(
                "probability {numerator}/{denominator} exceeds 1"
            )));
        }
        let divisor = gcd(numerator, denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    #[must_use]
    pub const fn one() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    #[must_use]
    pub fn as_prism_expr(self) -> String {
        if self.denominator == 1 {
            format!("{}", self.numerator)
        } else {
            format!("{}/{}", self.numerator, self.denominator)
        }
    }

    #[must_use]
    pub fn as_f64(self) -> f64 {
        (self.numerator as f64) / (self.denominator as f64)
    }
}

const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 { 1 } else { a }
}

/// Checks whether a list of exact rational probabilities sums strictly to 1.
pub fn validate_distribution(probs: &[RationalProb]) -> Result<(), AnalysisError> {
    if probs.is_empty() {
        return Err(AnalysisError::InvalidProbability(String::from(
            "probability distribution is empty",
        )));
    }
    let mut sum_num: u128 = 0;
    let mut sum_den: u128 = 1;
    for prob in probs {
        if prob.denominator == 0 || prob.numerator > prob.denominator {
            return Err(AnalysisError::InvalidProbability(format!(
                "invalid probability {}/{}",
                prob.numerator, prob.denominator
            )));
        }
        let num = u128::from(prob.numerator);
        let den = u128::from(prob.denominator);
        let next_num = sum_num
            .checked_mul(den)
            .and_then(|lhs| {
                num.checked_mul(sum_den)
                    .and_then(|rhs| lhs.checked_add(rhs))
            })
            .ok_or_else(|| {
                AnalysisError::InvalidProbability(String::from("rational sum overflow"))
            })?;
        let next_den = sum_den.checked_mul(den).ok_or_else(|| {
            AnalysisError::InvalidProbability(String::from("rational denominator overflow"))
        })?;
        let divisor = gcd_u128(next_num, next_den);
        sum_num = next_num / divisor;
        sum_den = next_den / divisor;
    }
    if sum_num != sum_den {
        return Err(AnalysisError::InvalidProbability(format!(
            "probabilities sum to {sum_num}/{sum_den}, expected 1"
        )));
    }
    Ok(())
}

const fn gcd_u128(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    if a == 0 { 1 } else { a }
}

/// Explicit provenance class for probability values; preferences and LLM
/// confidence values are never calibrated branch probabilities.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProbabilityProvenance {
    Hypothesis,
    ExternallyEstimated,
    ValidatedObservation,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BranchProbability {
    pub label: LabelId,
    pub probability: RationalProb,
}

/// Explicit probability annotation for one decider `InternalChoice` occurrence
/// (identified by `decider` and 0-based `choice_ordinal` in that decider's
/// local protocol preorder). Unannotated choices stay nondeterministic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceProbabilityAnnotation {
    pub decider: RoleId,
    pub choice_ordinal: usize,
    pub provenance: ProbabilityProvenance,
    pub branches: Vec<BranchProbability>,
}

/// Separate immutable sidecar attaching explicit probabilities to selected
/// choices without altering Choregraph's v1 source or Trellis projection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProbabilitySidecar {
    pub schema: String,
    pub numeric_representation: String,
    pub choices: Vec<ChoiceProbabilityAnnotation>,
}

impl ProbabilitySidecar {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema: String::from(QUANTITATIVE_PILOT_SCHEMA),
            numeric_representation: String::from("exact-rational-u64"),
            choices: Vec::new(),
        }
    }
}

/// Explicit payload/ownership abstraction policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OwnershipAbstraction {
    /// Rejects `SendOwned` / `ReceiveOwned` rather than silently dropping
    /// linear ownership semantics.
    RejectOwnershipTransfer,
}

/// Explicit scheduler/fairness assumption recorded on every property query.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchedulerAssumption {
    /// Ordinary PRISM MDP `Pmin`/`Pmax` quantification over all unrestricted
    /// memoryless/history-dependent adversaries (not a fair-scheduler lower bound).
    UnrestrictedAdversarial,
}

/// Explicit bounded vs unbounded quantitative property specifications.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum PropertySpec {
    BoundedCompletionMin { max_steps: u32 },
    BoundedCompletionMax { max_steps: u32 },
    UnboundedCompletionMin,
    UnboundedCompletionMax,
    DeadlockReachabilityMax,
    BranchLabelReachabilityMin { decider: RoleId, label: LabelId },
    BranchLabelReachabilityMax { decider: RoleId, label: LabelId },
}

impl PropertySpec {
    #[must_use]
    pub fn is_bounded(&self) -> bool {
        matches!(
            self,
            Self::BoundedCompletionMin { .. } | Self::BoundedCompletionMax { .. }
        )
    }

    #[must_use]
    pub fn pctl_formula(&self) -> String {
        match self {
            Self::BoundedCompletionMin { max_steps } => {
                format!("Pmin=? [ F<={max_steps} \"all_ended\" ]")
            }
            Self::BoundedCompletionMax { max_steps } => {
                format!("Pmax=? [ F<={max_steps} \"all_ended\" ]")
            }
            Self::UnboundedCompletionMin => String::from("Pmin=? [ F \"all_ended\" ]"),
            Self::UnboundedCompletionMax => String::from("Pmax=? [ F \"all_ended\" ]"),
            Self::DeadlockReachabilityMax => String::from("Pmax=? [ F \"protocol_deadlock\" ]"),
            Self::BranchLabelReachabilityMin { decider, label } => {
                format!("Pmin=? [ F \"branch_{}_{}\" ]", decider.0, label.0)
            }
            Self::BranchLabelReachabilityMax { decider, label } => {
                format!("Pmax=? [ F \"branch_{}_{}\" ]", decider.0, label.0)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateSpaceBounds {
    pub max_states: usize,
    pub max_transitions: usize,
    pub timeout_ms: u64,
}

impl Default for StateSpaceBounds {
    fn default() -> Self {
        Self {
            max_states: DEFAULT_MAX_STATES,
            max_transitions: DEFAULT_MAX_TRANSITIONS,
            timeout_ms: DEFAULT_TIMEOUT_MS,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolConfiguration {
    pub tool_name: String,
    pub expected_version: String,
    pub license: String,
    pub nixpkgs_revision: String,
    pub prism_binary: String,
    pub extra_args: Vec<String>,
}

impl ToolConfiguration {
    #[must_use]
    pub fn pinned_prism(prism_binary: impl Into<String>) -> Self {
        Self {
            tool_name: String::from("prism-model-checker"),
            expected_version: String::from(PINNED_PRISM_VERSION),
            license: String::from(PINNED_PRISM_LICENSE),
            nixpkgs_revision: String::from(PINNED_PRISM_NIXPKGS_REV),
            prism_binary: prism_binary.into(),
            extra_args: vec![String::from("-explicit")],
        }
    }
}

/// Complete manifest binding every input to one opt-in quantitative run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisManifest {
    pub schema: String,
    pub profile: String,
    pub semantics: String,
    pub requested_roles: Vec<RoleId>,
    pub ownership_abstraction: OwnershipAbstraction,
    pub scheduler_assumption: SchedulerAssumption,
    pub bounds: StateSpaceBounds,
    pub properties: Vec<PropertySpec>,
    pub tool: ToolConfiguration,
}

/// Domain-separated BLAKE3 receipt binding all inputs and exported subjects.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisReceipt {
    pub schema: String,
    pub source_and_endpoints_blake3: String,
    pub annotations_blake3: String,
    pub abstraction_blake3: String,
    pub properties_blake3: String,
    pub scheduler_blake3: String,
    pub tool_config_blake3: String,
    pub exported_model_blake3: String,
    pub receipt_blake3: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyResult {
    pub property: PropertySpec,
    pub formula: String,
    pub is_bounded: bool,
    pub scheduler_assumption: SchedulerAssumption,
    pub probability: f64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateInventory {
    pub states: usize,
    pub transitions: usize,
    pub probabilistic_choices: usize,
    pub nondeterministic_choices: usize,
    pub terminal_end_states: usize,
    pub deadlock_states: usize,
}

/// Retained artifacts for independent replay without re-running Trellis.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedSubject {
    pub prism_model: String,
    pub props_file: String,
    pub receipt: AnalysisReceipt,
}

/// Machine-readable report keeping probability vs nondeterminism, fair vs
/// unrestricted scheduling, bounded vs unbounded properties, and source-model
/// vs runtime claims strictly separate.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisReport {
    pub schema: String,
    pub profile: String,
    pub claim_scope: String,
    pub semantics: String,
    pub scheduler_assumption: SchedulerAssumption,
    pub provenance_classes: Vec<ProbabilityProvenance>,
    pub has_nondeterministic_choices: bool,
    pub has_probabilistic_choices: bool,
    pub inventory: StateInventory,
    pub conformance_verified: bool,
    pub results: Vec<PropertyResult>,
    pub receipt: AnalysisReceipt,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AnalysisError {
    ProfileNotSelected,
    InvalidSchema(String),
    IncompleteEndpointSet(String),
    UnsupportedConstruct(String),
    InvalidProbability(String),
    UnusedOrUnknownAnnotation(String),
    StateSpaceExhausted {
        observed_states: usize,
        observed_transitions: usize,
    },
    ConformanceMismatch(String),
    ToolUnavailable(String),
    ToolTimeout {
        timeout_ms: u64,
    },
    ToolFailed {
        exit_code: Option<i32>,
        stderr: String,
    },
    MalformedCheckerOutput(String),
    ReceiptMismatch(String),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AnalysisError {}
