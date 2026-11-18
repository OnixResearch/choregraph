use crate::concrete::reference::{
    ConcreteBindingInput, ConcreteReference, ReferenceBinding, ReferenceBindingSpec,
};
use crate::id::bounded::{
    AbilityId, DeclarationKey, NodeId, OperationId, ResourceId, SlotId, SourceRef,
};
use crate::model::semantics::TransitionSelector;
use alloc::string::String;

/// One untrusted v2 effect declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteEffectDeclarationSpec {
    pub declaration_key: String,
    pub node_id: String,
    pub transition: TransitionSelector,
    pub operation: String,
    pub resource: String,
    pub ability: String,
    pub request_binding: ReferenceBindingSpec,
    pub payload_binding: ReferenceBindingSpec,
}

/// One admitted immutable v2 effect declaration.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConcreteEffectDeclaration {
    pub declaration_key: DeclarationKey,
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request_binding: ReferenceBinding,
    pub payload_binding: ReferenceBinding,
}

/// One v2 effect request carried by an event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConcreteEffectRequest {
    pub declaration_key: String,
    pub request: ConcreteBindingInput,
    pub payload: ConcreteBindingInput,
}

/// One concrete intent emitted by an accepted v2 transition.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ConcreteEffectIntent {
    pub declaration_key: DeclarationKey,
    pub node_id: NodeId,
    pub transition: TransitionSelector,
    pub operation: OperationId,
    pub resource: ResourceId,
    pub ability: AbilityId,
    pub request_slot: Option<SlotId>,
    pub request_source_ref: Option<SourceRef>,
    pub request_reference: ConcreteReference,
    pub payload_slot: Option<SlotId>,
    pub payload_source_ref: Option<SourceRef>,
    pub payload_reference: ConcreteReference,
}
