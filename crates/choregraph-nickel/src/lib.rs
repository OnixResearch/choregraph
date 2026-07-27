#![no_std]

use choregraph_core::Capability;

/// The stage of input offered to the restricted lowering boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringInputStage {
    EvaluatedData,
    NickelSource,
    DynamicImport,
    EnvironmentValue,
}

/// Returns the core capability produced by successful future lowering.
#[must_use]
pub const fn output_capability() -> Capability {
    Capability::ImmutableMachineData
}

/// Returns true only for already evaluated, data-oriented input.
#[must_use]
pub const fn accepts(stage: LoweringInputStage) -> bool {
    matches!(stage, LoweringInputStage::EvaluatedData)
}

#[cfg(test)]
mod tests {
    use super::{LoweringInputStage, accepts};

    #[test]
    fn accepts_evaluated_data() {
        assert!(accepts(LoweringInputStage::EvaluatedData));
    }

    #[test]
    fn rejects_ambient_evaluation_inputs() {
        assert!(!accepts(LoweringInputStage::NickelSource));
        assert!(!accepts(LoweringInputStage::DynamicImport));
        assert!(!accepts(LoweringInputStage::EnvironmentValue));
    }
}
