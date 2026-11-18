use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::lifecycle::{
    ConversionError, StepAction, StepState, compatibility_action, compatibility_state, core_action,
    core_state,
};

/// One selected canonical reference in the conversion-only v2 handoff.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedReferenceHandoff {
    pub slot_id: Option<String>,
    pub source_ref: Option<String>,
    pub profile: String,
    pub identity: String,
}

/// One concrete Choregraph intent converted without policy or runtime authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteIntentHandoff {
    pub declaration_key: String,
    pub node_id: String,
    pub from_state: StepState,
    pub action: StepAction,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: SelectedReferenceHandoff,
    pub payload: SelectedReferenceHandoff,
}

/// One conversion-only plan over an already admitted v2 machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteExecutionPlan {
    machine: choregraph_core::model::automaton::AdmittedMachine,
}

impl ConcreteExecutionPlan {
    #[must_use]
    pub const fn machine(&self) -> &choregraph_core::model::automaton::AdmittedMachine {
        &self.machine
    }
}

/// A v2 plan rejects every non-v2 machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConcretePlanError {
    WrongMachineSchema,
}

/// A v2 decision exposes converted intent data without dispatch authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConcreteMachineDecision {
    Accept {
        next: choregraph_core::model::automaton::MachineState,
        intents: Vec<ConcreteIntentHandoff>,
    },
    Deny {
        reason: choregraph_core::model::outcome::DenialReason,
    },
}
/// Accepts only an already admitted v2 machine for concrete-intent conversion.
pub fn admit_concrete_machine(
    machine: choregraph_core::model::automaton::AdmittedMachine,
) -> Result<ConcreteExecutionPlan, ConcretePlanError> {
    if machine.schema_version() != choregraph_core::model::semantics::MachineSchema::V2 {
        return Err(ConcretePlanError::WrongMachineSchema);
    }
    Ok(ConcreteExecutionPlan { machine })
}

fn selected_reference_handoff(
    slot_id: Option<&choregraph_core::id::bounded::SlotId>,
    source_ref: Option<&choregraph_core::id::bounded::SourceRef>,
    reference: &choregraph_core::concrete::reference::ConcreteReference,
) -> SelectedReferenceHandoff {
    SelectedReferenceHandoff {
        slot_id: slot_id.map(|value| value.as_str().to_string()),
        source_ref: source_ref.map(|value| value.as_str().to_string()),
        profile: reference.profile.as_str().to_string(),
        identity: reference.identity.as_str().to_string(),
    }
}

/// Converts one concrete intent without adding policy, occurrence, or execution facts.
pub fn convert_concrete_intent(
    intent: &choregraph_core::concrete::effect::ConcreteEffectIntent,
) -> Result<ConcreteIntentHandoff, ConversionError> {
    let from_state = compatibility_state(intent.transition.from)?;
    let action = compatibility_action(intent.transition.action)?;
    debug_assert_eq!(core_state(from_state), intent.transition.from);
    debug_assert_eq!(core_action(action), intent.transition.action);
    Ok(ConcreteIntentHandoff {
        declaration_key: intent.declaration_key.as_str().to_string(),
        node_id: intent.node_id.as_str().to_string(),
        from_state,
        action,
        operation: intent.operation.as_str().to_string(),
        resource: intent.resource.as_str().to_string(),
        ability: intent.ability.as_str().to_string(),
        request: selected_reference_handoff(
            intent.request_slot.as_ref(),
            intent.request_source_ref.as_ref(),
            &intent.request_reference,
        ),
        payload: selected_reference_handoff(
            intent.payload_slot.as_ref(),
            intent.payload_source_ref.as_ref(),
            &intent.payload_reference,
        ),
    })
}

/// Delegates one v2 event to the production kernel and converts accepted intents.
pub fn step_concrete_machine(
    plan: &ConcreteExecutionPlan,
    state: &choregraph_core::model::automaton::MachineState,
    event: &choregraph_core::concrete::event::ConcreteEvent,
) -> Result<ConcreteMachineDecision, ConversionError> {
    match choregraph_core::transition::step_concrete(&plan.machine, state, event) {
        choregraph_core::concrete::event::ConcreteDecision::Accept { next, intents } => {
            Ok(ConcreteMachineDecision::Accept {
                next,
                intents: intents
                    .iter()
                    .map(convert_concrete_intent)
                    .collect::<Result<_, _>>()?,
            })
        }
        choregraph_core::concrete::event::ConcreteDecision::Deny { reason } => {
            Ok(ConcreteMachineDecision::Deny { reason })
        }
    }
}
