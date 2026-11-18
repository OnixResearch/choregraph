use crate::concrete::effect::{ConcreteEffectIntent, ConcreteEffectRequest};
use crate::model::automaton::MachineState;
use crate::model::outcome::DenialReason;
use crate::model::pause::SuspensionFactSpec;
use crate::model::semantics::Action;
use alloc::string::String;
use alloc::vec::Vec;

/// One bounded v2 lifecycle event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteEvent {
    pub node_id: String,
    pub action: Action,
    pub effects: Vec<ConcreteEffectRequest>,
    pub suspension: Option<SuspensionFactSpec>,
}

impl ConcreteEvent {
    #[must_use]
    pub fn new(node_id: impl Into<String>, action: Action) -> Self {
        Self {
            node_id: node_id.into(),
            action,
            effects: Vec::new(),
            suspension: None,
        }
    }

    #[must_use]
    pub fn suspension(
        node_id: impl Into<String>,
        action: Action,
        class: impl Into<String>,
        reference: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            action,
            effects: Vec::new(),
            suspension: Some(SuspensionFactSpec {
                class: class.into(),
                reference: reference.into(),
            }),
        }
    }
}

/// The result of one pure v2 transition request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConcreteDecision {
    Accept {
        next: MachineState,
        intents: Vec<ConcreteEffectIntent>,
    },
    Deny {
        reason: DenialReason,
    },
}
