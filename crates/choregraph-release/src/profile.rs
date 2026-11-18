// Closed release role profile and exact expected artifact metadata.
pub(crate) const REQUIRED_ROLE_COUNT: usize = 36;

use crate::evidence::ArtifactStatus;
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Role {
    SourceTree,
    License,
    MachineSchema,
    ConcreteMachineSchema,
    ConcreteIntentSchema,
    LinkReceiptSchema,
    CoreSource,
    CoreLibrary,
    SingleStepProofSource,
    SingleStepProofResult,
    TraceProofSource,
    TraceProofResult,
    TrustedSurface,
    RustToolchain,
    VerusToolchain,
    SolverToolchain,
    NickelToolchain,
    NixInputs,
    DependencyLock,
    KamaciteMachine,
    KamaciteConcreteMachine,
    KamaciteConcreteIntent,
    KamaciteConcreteIntentReceipt,
    KamaciteLifecycle,
    KamaciteTraceReceipt,
    KamaciteExhaustivenessReceipt,
    KamaciteSourceReceipt,
    KamaciteProofReferenceReceipt,
    PositiveFixtureCorpus,
    NegativeFixtureCorpus,
    ExecutionGraphParity,
    LatticeDownstream,
    LatticeConcreteHandoff,
    CairnReceipt,
    OctetReceipt,
    NixReceipt,
}

pub const REQUIRED_ROLES: [Role; REQUIRED_ROLE_COUNT] = [
    Role::SourceTree,
    Role::License,
    Role::MachineSchema,
    Role::ConcreteMachineSchema,
    Role::ConcreteIntentSchema,
    Role::LinkReceiptSchema,
    Role::CoreSource,
    Role::CoreLibrary,
    Role::SingleStepProofSource,
    Role::SingleStepProofResult,
    Role::TraceProofSource,
    Role::TraceProofResult,
    Role::TrustedSurface,
    Role::RustToolchain,
    Role::VerusToolchain,
    Role::SolverToolchain,
    Role::NickelToolchain,
    Role::NixInputs,
    Role::DependencyLock,
    Role::KamaciteMachine,
    Role::KamaciteConcreteMachine,
    Role::KamaciteConcreteIntent,
    Role::KamaciteConcreteIntentReceipt,
    Role::KamaciteLifecycle,
    Role::KamaciteTraceReceipt,
    Role::KamaciteExhaustivenessReceipt,
    Role::KamaciteSourceReceipt,
    Role::KamaciteProofReferenceReceipt,
    Role::PositiveFixtureCorpus,
    Role::NegativeFixtureCorpus,
    Role::ExecutionGraphParity,
    Role::LatticeDownstream,
    Role::LatticeConcreteHandoff,
    Role::CairnReceipt,
    Role::OctetReceipt,
    Role::NixReceipt,
];

impl Role {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceTree => "source-tree",
            Self::License => "license",
            Self::MachineSchema => "machine-schema",
            Self::ConcreteMachineSchema => "concrete-machine-schema",
            Self::ConcreteIntentSchema => "concrete-intent-schema",
            Self::LinkReceiptSchema => "link-receipt-schema",
            Self::CoreSource => "core-source",
            Self::CoreLibrary => "core-library",
            Self::SingleStepProofSource => "single-step-proof-source",
            Self::SingleStepProofResult => "single-step-proof-result",
            Self::TraceProofSource => "trace-proof-source",
            Self::TraceProofResult => "trace-proof-result",
            Self::TrustedSurface => "trusted-surface",
            Self::RustToolchain => "rust-toolchain",
            Self::VerusToolchain => "verus-toolchain",
            Self::SolverToolchain => "solver-toolchain",
            Self::NickelToolchain => "nickel-toolchain",
            Self::NixInputs => "nix-inputs",
            Self::DependencyLock => "dependency-lock",
            Self::KamaciteMachine => "kamacite-machine",
            Self::KamaciteConcreteMachine => "kamacite-concrete-machine",
            Self::KamaciteConcreteIntent => "kamacite-concrete-intent",
            Self::KamaciteConcreteIntentReceipt => "kamacite-concrete-intent-receipt",
            Self::KamaciteLifecycle => "kamacite-lifecycle",
            Self::KamaciteTraceReceipt => "kamacite-trace-receipt",
            Self::KamaciteExhaustivenessReceipt => "kamacite-exhaustiveness-receipt",
            Self::KamaciteSourceReceipt => "kamacite-source-receipt",
            Self::KamaciteProofReferenceReceipt => "kamacite-proof-reference-receipt",
            Self::PositiveFixtureCorpus => "positive-fixture-corpus",
            Self::NegativeFixtureCorpus => "negative-fixture-corpus",
            Self::ExecutionGraphParity => "execution-graph-parity",
            Self::LatticeDownstream => "lattice-downstream",
            Self::LatticeConcreteHandoff => "lattice-concrete-handoff",
            Self::CairnReceipt => "cairn-receipt",
            Self::OctetReceipt => "octet-receipt",
            Self::NixReceipt => "nix-receipt",
        }
    }
}
#[must_use]
pub const fn expected_status(role: Role) -> ArtifactStatus {
    match role {
        Role::MachineSchema
        | Role::ConcreteMachineSchema
        | Role::ConcreteIntentSchema
        | Role::LinkReceiptSchema
        | Role::KamaciteMachine
        | Role::KamaciteConcreteMachine
        | Role::KamaciteConcreteIntent
        | Role::KamaciteConcreteIntentReceipt
        | Role::KamaciteLifecycle
        | Role::KamaciteTraceReceipt
        | Role::KamaciteExhaustivenessReceipt
        | Role::KamaciteSourceReceipt
        | Role::KamaciteProofReferenceReceipt => ArtifactStatus::Canonical,
        Role::SingleStepProofResult | Role::TraceProofResult => ArtifactStatus::Verified,
        Role::ExecutionGraphParity
        | Role::LatticeDownstream
        | Role::CairnReceipt
        | Role::OctetReceipt
        | Role::NixReceipt => ArtifactStatus::Passed,
        Role::SourceTree
        | Role::License
        | Role::CoreSource
        | Role::CoreLibrary
        | Role::SingleStepProofSource
        | Role::TraceProofSource
        | Role::TrustedSurface
        | Role::RustToolchain
        | Role::VerusToolchain
        | Role::SolverToolchain
        | Role::NickelToolchain
        | Role::NixInputs
        | Role::DependencyLock
        | Role::PositiveFixtureCorpus
        | Role::NegativeFixtureCorpus
        | Role::LatticeConcreteHandoff => ArtifactStatus::Observed,
    }
}

#[must_use]
pub const fn expected_schema(role: Role) -> &'static str {
    match role {
        Role::SourceTree => "choregraph.source-baseline.v1",
        Role::License => "spdx.license-text.v1",
        Role::MachineSchema => "choregraph.machine-schema.v1",
        Role::ConcreteMachineSchema => "choregraph.machine-schema.v2",
        Role::ConcreteIntentSchema => "choregraph.concrete-intent-schema.v2",
        Role::LinkReceiptSchema => "choregraph.link-receipt-schema.v1",
        Role::CoreSource => "rust.source.v1",
        Role::CoreLibrary => "rust.library-artifact.v1",
        Role::SingleStepProofSource => "verus.single-step-source.v1",
        Role::SingleStepProofResult => "choregraph.single-step-proof.v1",
        Role::TraceProofSource => "verus.trace-replay-source.v1",
        Role::TraceProofResult => "choregraph.trace-replay-proof.v1",
        Role::TrustedSurface => "choregraph.trusted-surface.v1",
        Role::RustToolchain => "rust-toolchain.toml.v1",
        Role::VerusToolchain => "octet.verus-toolchain.v1",
        Role::SolverToolchain => "octet.solver-toolchain.v1",
        Role::NickelToolchain => "nickel.toolchain.v1",
        Role::NixInputs => "nix.flake-inputs.v1",
        Role::DependencyLock => "cargo.lock.v1",
        Role::KamaciteMachine => "choregraph.machine.v1",
        Role::KamaciteConcreteMachine => "choregraph.machine.v2",
        Role::KamaciteConcreteIntent => "choregraph.concrete-intent.v2",
        Role::KamaciteConcreteIntentReceipt => "choregraph.concrete-intent-receipt.v1",
        Role::KamaciteLifecycle => "choregraph.lifecycle-fsm.v1",
        Role::KamaciteTraceReceipt => "choregraph.fsm-trace-receipt.v1",
        Role::KamaciteExhaustivenessReceipt => "choregraph.fsm-exhaustiveness-receipt.v1",
        Role::KamaciteSourceReceipt => "choregraph.nickel-source-receipt.v1",
        Role::KamaciteProofReferenceReceipt => "choregraph.proof-reference-receipt.v1",
        Role::PositiveFixtureCorpus => "choregraph.positive-fixture-corpus.v1",
        Role::NegativeFixtureCorpus => "choregraph.negative-fixture-corpus.v1",
        Role::ExecutionGraphParity => "choregraph.execution-graph-parity.v1",
        Role::LatticeDownstream => "choregraph.lattice-downstream.v1",
        Role::LatticeConcreteHandoff => "choregraph.lattice-concrete-handoff.v1",
        Role::CairnReceipt => "cairn.validation-receipt.v1",
        Role::OctetReceipt => "octet.validation-receipt.v1",
        Role::NixReceipt => "nix.flake-check-receipt.v1",
    }
}
