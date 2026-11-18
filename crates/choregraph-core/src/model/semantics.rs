/// Local lifecycle states for one admitted node.
///
/// `Suspended` is reachable only under
/// [`DURABLE_SUSPENSION_V1`](crate::model::semantics::DURABLE_SUSPENSION_V1). The bounded
/// suspension facts live in [`MachineState`](crate::model::automaton::MachineState)
/// suspension records, not in this state tag, so this enum stays `Copy`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LocalState {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
    Suspended,
}

/// Events supported by the admitted lifecycle profile.
///
/// `Suspend` and `ResumeSignal` have a target only under
/// [`DURABLE_SUSPENSION_V1`](crate::model::semantics::DURABLE_SUSPENSION_V1).
/// Both actions always reject effect requests.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Action {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
    Suspend,
    ResumeSignal,
}

/// A local lifecycle transition failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalTransitionError {
    Terminal { from: LocalState, action: Action },
    Invalid { from: LocalState, action: Action },
}

/// One exact lifecycle transition selector.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TransitionSelector {
    pub from: LocalState,
    pub action: Action,
}

pub const MACHINE_SCHEMA_V1: &str = "choregraph.machine.v1";
pub const MACHINE_SCHEMA_V2: &str = "choregraph.machine.v2";
pub const LINEAR_LIFECYCLE_V1: &str = "choregraph.lifecycle.linear.v1";
pub const DURABLE_SUSPENSION_V1: &str = "choregraph.lifecycle.durable-suspension.v1";

/// The selected machine schema after admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MachineSchema {
    V1,
    V2,
}

/// The selected local lifecycle profile after admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleProfile {
    LinearV1,
    DurableSuspensionV1,
}
