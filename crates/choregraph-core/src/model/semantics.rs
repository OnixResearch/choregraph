/// Local lifecycle states for one admitted node.
///
/// `Suspended` is reachable only under the durable suspension profiles. The bounded
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
/// The durable profiles permit `Suspend` and `ResumeSignal` without effect requests.
/// [`DURABLE_SUSPENSION_V2`] also permits effect-free `Cancel` from `Suspended`.
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
pub const DURABLE_SUSPENSION_V2: &str = "choregraph.lifecycle.durable-suspension.v2";

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
    DurableSuspensionV2,
}

impl LifecycleProfile {
    /// Whether this profile admits suspension facts and restorable waits.
    #[must_use]
    pub const fn supports_suspension(self) -> bool {
        matches!(self, Self::DurableSuspensionV1 | Self::DurableSuspensionV2)
    }
}
