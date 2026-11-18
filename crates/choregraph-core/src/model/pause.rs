use crate::id::bounded::{NodeId, SuspensionClass};
use crate::id::reference::PayloadRef;
use alloc::string::String;

/// One untrusted durable-suspension declaration for restart-time state.
///
/// The class and reference are opaque bounded facts. Choregraph never parses
/// approvers, policies, credentials, clocks, or product payloads from them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuspensionDeclarationSpec {
    pub node_id: String,
    pub class: String,
    pub reference: String,
}

/// One exact admitted suspension declaration.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SuspensionDeclaration {
    pub node_id: NodeId,
    pub class: SuspensionClass,
    pub reference: PayloadRef,
}

/// One exact suspension record bound to a suspended node in machine state.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SuspensionRecord {
    pub class: SuspensionClass,
    pub reference: PayloadRef,
}

/// The raw suspension fact carried by one suspend or resume-signal event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuspensionFactSpec {
    pub class: String,
    pub reference: String,
}
