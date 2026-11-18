use crate::id::bounded::NodeId;
use alloc::vec::Vec;

/// Resume behavior compatible with the current execution-graph vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeDecision {
    Execute,
    AwaitDependencies,
    AwaitSignal,
    SkipCompleted,
    RejectTerminalMutation,
    BlockedByFailedDependency,
    UnknownNode,
    InvalidState,
}

/// One deterministic graph level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParallelGroup {
    pub node_ids: Vec<NodeId>,
    pub level: usize,
    pub can_parallelize: bool,
}

/// One node ranked by transitive dependent count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bottleneck {
    pub node_id: NodeId,
    pub dependent_count: usize,
}
