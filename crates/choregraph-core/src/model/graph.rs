use crate::id::bounded::NodeId;
use alloc::string::String;
use alloc::vec::Vec;

/// One untrusted node declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeSpec {
    pub id: String,
    pub operations: Vec<String>,
}

impl NodeSpec {
    #[must_use]
    pub fn new(id: impl Into<String>, operations: Vec<String>) -> Self {
        Self {
            id: id.into(),
            operations,
        }
    }
}

/// One untrusted directed dependency edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EdgeSpec {
    pub source: String,
    pub target: String,
}

impl EdgeSpec {
    #[must_use]
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

/// One normalized immutable edge.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
}
