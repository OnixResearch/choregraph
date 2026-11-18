/// The stage of input offered to the restricted lowering boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringInputStage {
    EvaluatedData,
    NickelSource,
    DynamicImport,
    EnvironmentValue,
}

#[must_use]
pub const fn output_capability() -> choregraph_core::boundary::Capability {
    choregraph_core::boundary::Capability::ImmutableMachineData
}

#[must_use]
pub const fn accepts(stage: LoweringInputStage) -> bool {
    matches!(stage, LoweringInputStage::EvaluatedData)
}
