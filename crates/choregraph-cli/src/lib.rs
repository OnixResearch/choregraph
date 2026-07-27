use choregraph_core::{Capability, FOUNDATION_CLAIM, MACHINE_CLAIM, permits};
use choregraph_kamacite::{InterchangeClaim, supports as supports_interchange};
use choregraph_nickel::{LoweringInputStage, accepts as accepts_lowering};
use choregraph_seaglass::{CompatibilityMode, accepts as accepts_compatibility};

/// A command accepted by the current foundation shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Boundary,
}

/// Parses the command without reading process state.
pub fn parse_command(arguments: &[String]) -> Result<Command, &'static str> {
    match arguments {
        [] => Ok(Command::Boundary),
        [argument] if argument == "boundary" => Ok(Command::Boundary),
        _ => Err("usage: choregraph [boundary]"),
    }
}

/// Renders the bounded, unverified foundation report.
#[must_use]
pub fn render_foundation_report() -> String {
    let core_is_pure = permits(Capability::DeterministicDecision);
    let kamacite_is_identity_only =
        supports_interchange(InterchangeClaim::CanonicalIdentityBinding)
            && !supports_interchange(InterchangeClaim::BehavioralProof);
    let nickel_is_evaluated_data_only = accepts_lowering(LoweringInputStage::EvaluatedData)
        && !accepts_lowering(LoweringInputStage::NickelSource);
    let seaglass_is_delegate_only =
        accepts_compatibility(CompatibilityMode::DelegateToChoregraphCore)
            && !accepts_compatibility(CompatibilityMode::IndependentTransitionImplementation);

    format!(
        "status={FOUNDATION_CLAIM}\nmachine-status={MACHINE_CLAIM}\ncore-pure={core_is_pure}\nkamacite-identity-only={kamacite_is_identity_only}\nnickel-evaluated-data-only={nickel_is_evaluated_data_only}\nseaglass-delegate-only={seaglass_is_delegate_only}\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{Command, parse_command, render_foundation_report};

    #[test]
    fn parses_default_and_explicit_boundary_commands() {
        assert_eq!(parse_command(&[]), Ok(Command::Boundary));
        assert_eq!(
            parse_command(&[String::from("boundary")]),
            Ok(Command::Boundary)
        );
    }

    #[test]
    fn rejects_unknown_or_extra_arguments() {
        assert!(parse_command(&[String::from("run")]).is_err());
        assert!(parse_command(&[String::from("boundary"), String::from("extra")]).is_err());
    }

    #[test]
    fn report_keeps_the_foundation_claim_bounded() {
        let report = render_foundation_report();
        assert!(report.contains("status=repository-boundary-only-unverified"));
        assert!(report.contains("seaglass-delegate-only=true"));
    }
}
