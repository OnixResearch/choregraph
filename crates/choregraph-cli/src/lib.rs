mod corpus;

pub use corpus::{KamaciteCorpusEntry, build_kamacite_corpus};

/// A command accepted by the imperative shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Boundary,
    WriteKamaciteCorpus { output_directory: String },
}

/// Parses a command without reading process state.
pub fn parse_command(arguments: &[String]) -> Result<Command, &'static str> {
    match arguments {
        [] => Ok(Command::Boundary),
        [argument] if argument == "boundary" => Ok(Command::Boundary),
        [command, output_directory] if command == "write-kamacite-corpus" => {
            if output_directory.is_empty() {
                return Err("Kamacite corpus output directory must be non-empty");
            }
            Ok(Command::WriteKamaciteCorpus {
                output_directory: output_directory.clone(),
            })
        }
        _ => Err("usage: choregraph [boundary | write-kamacite-corpus <output-directory>]"),
    }
}

/// Renders the bounded, unverified foundation report.
// This pure report composes claim predicates; their owner functions assert local invariants.
#[allow(
    unknown_lints,
    assertion_density,
    reason = "this pure report composes claim predicates; their owner functions assert local invariants"
)]
#[must_use]
pub fn render_foundation_report() -> String {
    let foundation_claim = choregraph_core::FOUNDATION_CLAIM;
    let machine_claim = choregraph_core::MACHINE_CLAIM;
    let is_core_pure = choregraph_core::permits(choregraph_core::Capability::DeterministicDecision);
    let is_kamacite_identity_only =
        choregraph_kamacite::supports(
            choregraph_kamacite::InterchangeClaim::CanonicalIdentityBinding,
        ) && !choregraph_kamacite::supports(choregraph_kamacite::InterchangeClaim::BehavioralProof);
    let is_nickel_evaluated_data_only =
        choregraph_nickel::accepts(choregraph_nickel::LoweringInputStage::EvaluatedData)
            && !choregraph_nickel::accepts(choregraph_nickel::LoweringInputStage::NickelSource);
    let is_seaglass_delegate_only = choregraph_seaglass::accepts(
        choregraph_seaglass::CompatibilityMode::DelegateToChoregraphCore,
    ) && !choregraph_seaglass::accepts(
        choregraph_seaglass::CompatibilityMode::IndependentTransitionImplementation,
    );

    format!(
        "status={foundation_claim}\nmachine-status={machine_claim}\ncore-pure={is_core_pure}\nkamacite-identity-only={is_kamacite_identity_only}\nnickel-evaluated-data-only={is_nickel_evaluated_data_only}\nseaglass-delegate-only={is_seaglass_delegate_only}\n"
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn parses_default_and_explicit_boundary_commands() {
        assert_eq!(super::parse_command(&[]), Ok(super::Command::Boundary));
        assert_eq!(
            super::parse_command(&[String::from("boundary")]),
            Ok(super::Command::Boundary)
        );
        assert_eq!(
            super::parse_command(&[
                String::from("write-kamacite-corpus"),
                String::from("fixtures/kamacite"),
            ]),
            Ok(super::Command::WriteKamaciteCorpus {
                output_directory: String::from("fixtures/kamacite"),
            })
        );
    }

    #[test]
    fn rejects_unknown_incomplete_or_extra_arguments() {
        assert!(super::parse_command(&[String::from("run")]).is_err());
        assert!(super::parse_command(&[String::from("write-kamacite-corpus")]).is_err());
        assert!(super::parse_command(&[String::from("boundary"), String::from("extra")]).is_err());
    }

    #[test]
    fn report_keeps_the_foundation_claim_bounded() {
        let foundation_output_text = super::render_foundation_report();
        assert!(foundation_output_text.contains("status=repository-boundary-only-unverified"));
        assert!(foundation_output_text.contains("seaglass-delegate-only=true"));
    }
}
