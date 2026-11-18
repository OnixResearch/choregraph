use crate::id::bounded::{AbilityId, NodeId, OperationId, RequestId, ResourceId};
use crate::id::reference::PayloadRef;
use crate::model::pause::SuspensionFactSpec;
use crate::model::semantics::{Action, TransitionSelector};
use alloc::string::String;
use alloc::vec::Vec;

/// One untrusted effect declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDeclarationSpec {
    pub node_id: String,
    pub transition: TransitionSelector,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

/// One exact admitted effect declaration.
///
/// A declaration permits a matching event request. It does not require every event to request it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EffectDeclaration {
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request: RequestId,
    pub payload: PayloadRef,
}

/// One raw effect request carried by an event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRequest {
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request: String,
    pub payload: String,
}

/// One declared intent emitted by an accepted transition.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EffectIntent {
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request: RequestId,
    pub payload: PayloadRef,
}

impl From<&EffectDeclaration> for EffectIntent {
    fn from(declaration: &EffectDeclaration) -> Self {
        Self {
            node_id: declaration.node_id.clone(),
            transition: declaration.transition,
            operation: declaration.operation.clone(),
            resource: declaration.resource.clone(),
            ability: declaration.ability.clone(),
            request: declaration.request.clone(),
            payload: declaration.payload.clone(),
        }
    }
}

/// One bounded lifecycle event.
///
/// `suspension` carries the opaque class and reference for `Suspend` and
/// `ResumeSignal` actions. It must be absent for every other action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub node_id: String,
    pub action: Action,
    pub effects: Vec<EffectRequest>,
    pub suspension: Option<SuspensionFactSpec>,
}

impl Event {
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
